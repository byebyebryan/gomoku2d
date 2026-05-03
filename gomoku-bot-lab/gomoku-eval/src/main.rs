use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use gomoku_bot::{RandomBot, SearchBot};
use gomoku_core::{Color, GameResult, Move, RuleConfig, Variant};
use gomoku_eval::arena::{run_match_series_with_limits, MatchEndReason, MatchLimits, MatchResult};
use gomoku_eval::report::{
    render_tournament_report_html_with_options, ReportRenderOptions, TournamentReport,
    TournamentRunReport,
};
use gomoku_eval::scenario::{
    run_tactical_scenarios, ScenarioSearchConfig, TacticalScenarioResult, TACTICAL_SCENARIO_CASES,
};
use gomoku_eval::seed::derive_seed;
use gomoku_eval::tournament::{
    default_thread_count, run_round_robin_parallel, TournamentBotFactory, TournamentOptions,
};

#[path = "../../benchmarks/search_configs.rs"]
mod search_configs;

#[derive(Parser, Debug)]
#[command(name = "gomoku-eval", about = "Evaluation harness for Gomoku bots")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Args, Debug, Clone)]
struct EvalOptions {
    /// Rule variant: "renju" (default) or "freestyle"
    #[arg(long, default_value = "renju")]
    rule: String,

    /// Per-move search budget for search bots, in milliseconds
    #[arg(long)]
    search_time_ms: Option<u64>,

    /// Per-move Linux thread CPU-time budget for search bots, in milliseconds
    #[arg(long)]
    search_cpu_time_ms: Option<u64>,

    /// Stop a game after this many moves and record it as a draw
    #[arg(long)]
    max_moves: Option<usize>,

    /// Stop a game after this wall-clock duration and record it as a draw
    #[arg(long)]
    max_game_ms: Option<u64>,

    /// Base seed for reproducible random bots and tournament openings
    #[arg(long, default_value_t = 0)]
    seed: u64,
}

struct EvalContext {
    config: RuleConfig,
    rule_label: &'static str,
    limits: MatchLimits,
    search_time_ms: Option<u64>,
    search_cpu_time_ms: Option<u64>,
    seed: u64,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run N games between two different bots
    Versus {
        #[command(flatten)]
        options: EvalOptions,

        #[arg(long, default_value = "baseline")]
        bot_a: String,

        #[arg(long, default_value = "random")]
        bot_b: String,

        #[arg(long, default_value_t = 10)]
        games: u32,

        #[arg(long)]
        replay_dir: Option<PathBuf>,
    },
    /// Run N games of a single bot playing against itself
    SelfPlay {
        #[command(flatten)]
        options: EvalOptions,

        #[arg(long, default_value = "baseline")]
        bot: String,

        #[arg(long, default_value_t = 10)]
        games: u32,

        #[arg(long)]
        replay_dir: Option<PathBuf>,
    },
    /// Render a saved tournament JSON report to standalone HTML
    ReportHtml {
        #[arg(long)]
        input: PathBuf,

        #[arg(long)]
        output: PathBuf,

        /// Link to the raw JSON from the rendered HTML; defaults to the input file name
        #[arg(long)]
        json_href: Option<String>,
    },
    /// Run a round-robin tournament among a list of bots
    Tournament {
        #[command(flatten)]
        options: EvalOptions,

        /// Comma-separated list of bots (e.g. "random,fast,balanced,deep,baseline-5")
        #[arg(long)]
        bots: String,

        /// Number of games each pair plays
        #[arg(long, default_value_t = 2)]
        games_per_pair: u32,

        #[arg(long)]
        replay_dir: Option<PathBuf>,

        /// Write reusable tournament report JSON
        #[arg(long)]
        report_json: Option<PathBuf>,

        /// Number of seeded random opening plies before bots take over
        #[arg(long, default_value_t = 4)]
        opening_plies: usize,

        /// Worker threads used to run tournament games
        #[arg(long)]
        threads: Option<usize>,
    },
    /// Run focused one-move tactical diagnostics against search configs
    TacticalScenarios {
        /// Comma-separated search configs (e.g. "search-d2,search-d3,search-d5")
        #[arg(long, default_value = "search-d2,search-d3,search-d5")]
        bots: String,

        /// Per-move search budget for search bots, in milliseconds
        #[arg(long)]
        search_time_ms: Option<u64>,

        /// Per-move Linux thread CPU-time budget for search bots, in milliseconds
        #[arg(long)]
        search_cpu_time_ms: Option<u64>,

        /// Write reusable tactical scenario report JSON
        #[arg(long)]
        report_json: Option<PathBuf>,
    },
}

type BotFactory = TournamentBotFactory;
type NamedBotFactory = (String, BotFactory);

fn make_bot_factory(
    spec: &str,
    search_time_ms: Option<u64>,
    search_cpu_time_ms: Option<u64>,
) -> Result<BotFactory, String> {
    let spec = spec.to_string();
    if spec == "random" {
        return Ok(Arc::new(|seed| Box::new(RandomBot::seeded(seed))));
    }

    if let Some(config) =
        search_configs::search_config_from_lab_spec(&spec, 5, search_time_ms, search_cpu_time_ms)
    {
        return Ok(Arc::new(move |_| Box::new(SearchBot::with_config(config))));
    }

    Err(format!(
        "Unknown bot type: '{spec}'. Use random, baseline, baseline-N, fast, balanced, or deep."
    ))
}

fn exit_with_error(message: impl AsRef<str>) -> ! {
    eprintln!("{}", message.as_ref());
    std::process::exit(2);
}

fn variant_label(variant: &Variant) -> &'static str {
    match variant {
        Variant::Freestyle => "freestyle",
        Variant::Renju => "renju",
    }
}

fn eval_context(options: &EvalOptions) -> EvalContext {
    let variant = match options.rule.as_str() {
        "renju" => Variant::Renju,
        "freestyle" => Variant::Freestyle,
        other => exit_with_error(format!(
            "Unknown rule variant '{other}'. Use 'renju' or 'freestyle'."
        )),
    };
    let rule_label = variant_label(&variant);
    EvalContext {
        config: RuleConfig {
            variant,
            ..Default::default()
        },
        rule_label,
        limits: MatchLimits {
            max_moves: options.max_moves,
            max_game_ms: options.max_game_ms,
        },
        search_time_ms: options.search_time_ms,
        search_cpu_time_ms: options.search_cpu_time_ms,
        seed: options.seed,
    }
}

fn print_move_progress(move_num: usize, game_idx: u32, player: Color, mv: Move, time_ms: u64) {
    let time_str = if time_ms >= 1000 {
        format!("{:.1}s", time_ms as f64 / 1000.0)
    } else {
        format!("{}ms", time_ms)
    };
    println!(
        "  Game {:3}  move {:3}  {} {:3}  ({})",
        game_idx + 1,
        move_num,
        match player {
            Color::Black => "B",
            Color::White => "W",
        },
        mv.to_notation(),
        time_str,
    );
}

fn end_reason_suffix(reason: MatchEndReason) -> String {
    match reason {
        MatchEndReason::Natural => String::new(),
        reason => format!(" ({})", reason.label()),
    }
}

fn print_game_result(i: u32, total: u32, mr: &MatchResult) {
    let suffix = end_reason_suffix(mr.end_reason);
    match &mr.result {
        GameResult::Winner(c) => println!("  Game {:3}/{:3}  {:?} wins{}", i + 1, total, c, suffix),
        GameResult::Draw => println!("  Game {:3}/{:3}  Draw{}", i + 1, total, suffix),
        GameResult::Ongoing => unreachable!(),
    }
}

fn parse_search_config_specs(
    specs: &str,
    search_time_ms: Option<u64>,
    search_cpu_time_ms: Option<u64>,
) -> Result<Vec<ScenarioSearchConfig>, String> {
    let names: Vec<String> = specs
        .split(',')
        .map(|spec| spec.trim().to_string())
        .filter(|spec| !spec.is_empty())
        .collect();

    if names.is_empty() {
        return Err("At least one search config is required.".to_string());
    }

    names
        .into_iter()
        .map(|name| {
            let config = search_configs::search_config_from_lab_spec(
                &name,
                5,
                search_time_ms,
                search_cpu_time_ms,
            )
            .ok_or_else(|| {
                format!("Unknown search config: '{name}'. Use search-dN, fast, balanced, or deep.")
            })?;
            Ok(ScenarioSearchConfig { id: name, config })
        })
        .collect()
}

fn print_tactical_scenario_result(result: &TacticalScenarioResult) {
    let status = if result.passed { "PASS" } else { "FAIL" };
    let expected = result.expected_moves.join("/");
    println!(
        "{:<5} {:<10} {:<28} actual {:<3} expected {:<7} depth {:>2} nodes {:>8} safety {:>5} eval {:>7} cand r/s {:>5}/{:<5} legal r/s {:>6}/{:<6} tt {:>5}/{:<5} cut {:>5} time {:>4}ms",
        status,
        result.config_id,
        result.case_id,
        result.actual_move,
        expected,
        result.metrics.depth_reached,
        result.metrics.nodes,
        result.metrics.safety_nodes,
        result.metrics.eval_calls,
        result.metrics.root_candidate_generations,
        result.metrics.search_candidate_generations,
        result.metrics.root_legality_checks,
        result.metrics.search_legality_checks,
        result.metrics.tt_hits,
        result.metrics.tt_cutoffs,
        result.metrics.beta_cutoffs,
        result.metrics.time_ms
    );
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Versus {
            options,
            bot_a,
            bot_b,
            games,
            replay_dir,
        } => {
            let EvalContext {
                config,
                rule_label,
                limits,
                search_time_ms,
                search_cpu_time_ms,
                seed,
            } = eval_context(&options);
            println!("--- Versus: {} vs {} ({} games) ---", bot_a, bot_b, games);
            println!("Rule: {rule_label}");
            if let Some(ms) = search_time_ms {
                println!("Search time budget: {ms} ms/move");
            }
            if let Some(ms) = search_cpu_time_ms {
                println!("Search CPU-time budget: {ms} ms/move");
            }

            if let Some(dir) = &replay_dir {
                std::fs::create_dir_all(dir).unwrap();
            }

            let bot_a_factory = match make_bot_factory(&bot_a, search_time_ms, search_cpu_time_ms) {
                Ok(factory) => factory,
                Err(err) => exit_with_error(err),
            };
            let bot_b_factory = match make_bot_factory(&bot_b, search_time_ms, search_cpu_time_ms) {
                Ok(factory) => factory,
                Err(err) => exit_with_error(err),
            };
            let mut game_seed_idx = 0u64;
            let make_bots = || {
                let current_seed_idx = game_seed_idx;
                game_seed_idx += 1;
                let a = bot_a_factory(derive_seed(seed, [current_seed_idx, 0]));
                let b = bot_b_factory(derive_seed(seed, [current_seed_idx, 1]));
                (a, b)
            };

            let stats = run_match_series_with_limits(
                make_bots,
                games,
                config,
                limits,
                print_move_progress,
                |i, mr| {
                    print_game_result(i, games, mr);
                    if let Some(dir) = &replay_dir {
                        let path = dir.join(format!("match_{:03}.json", i + 1));
                        if let Ok(json) = mr.replay.to_json() {
                            std::fs::write(&path, &json)
                                .unwrap_or_else(|e| eprintln!("Failed to write replay: {e}"));
                        }
                    }
                },
            );

            println!("\n--- Results ---");
            println!("{} wins: {}", bot_a, stats.bot_a_wins);
            println!("{} wins: {}", bot_b, stats.bot_b_wins);
            println!("Draws: {}", stats.draws);

            let a_avg_time = if stats.bot_a_moves > 0 {
                stats.bot_a_time_ms as f64 / stats.bot_a_moves as f64
            } else {
                0.0
            };
            let b_avg_time = if stats.bot_b_moves > 0 {
                stats.bot_b_time_ms as f64 / stats.bot_b_moves as f64
            } else {
                0.0
            };

            println!("\nAvg time per move:");
            println!("{}: {:.2} ms", bot_a, a_avg_time);
            println!("{}: {:.2} ms", bot_b, b_avg_time);
        }
        Commands::SelfPlay {
            options,
            bot,
            games,
            replay_dir,
        } => {
            let EvalContext {
                config,
                rule_label,
                limits,
                search_time_ms,
                search_cpu_time_ms,
                seed,
            } = eval_context(&options);
            println!("--- Self-Play: {} vs {} ({} games) ---", bot, bot, games);
            println!("Rule: {rule_label}");
            if let Some(ms) = search_time_ms {
                println!("Search time budget: {ms} ms/move");
            }
            if let Some(ms) = search_cpu_time_ms {
                println!("Search CPU-time budget: {ms} ms/move");
            }

            if let Some(dir) = &replay_dir {
                std::fs::create_dir_all(dir).unwrap();
            }

            let bot_factory = match make_bot_factory(&bot, search_time_ms, search_cpu_time_ms) {
                Ok(factory) => factory,
                Err(err) => exit_with_error(err),
            };
            let mut game_seed_idx = 0u64;
            let make_bots = move || {
                let current_seed_idx = game_seed_idx;
                game_seed_idx += 1;
                let a = bot_factory(derive_seed(seed, [current_seed_idx, 0]));
                let b = bot_factory(derive_seed(seed, [current_seed_idx, 1]));
                (a, b)
            };

            let stats = run_match_series_with_limits(
                make_bots,
                games,
                config,
                limits,
                print_move_progress,
                |i, mr| {
                    print_game_result(i, games, mr);
                    if let Some(dir) = &replay_dir {
                        let path = dir.join(format!("selfplay_{:03}.json", i + 1));
                        if let Ok(json) = mr.replay.to_json() {
                            std::fs::write(&path, &json)
                                .unwrap_or_else(|e| eprintln!("Failed to write replay: {e}"));
                        }
                    }
                },
            );

            let black_wins = stats.bot_a_wins;
            let white_wins = stats.bot_b_wins;

            println!("\n--- Results ---");
            println!("Black wins: {}", black_wins);
            println!("White wins: {}", white_wins);
            println!("Draws: {}", stats.draws);

            let avg_time = if stats.bot_a_moves + stats.bot_b_moves > 0 {
                (stats.bot_a_time_ms + stats.bot_b_time_ms) as f64
                    / (stats.bot_a_moves + stats.bot_b_moves) as f64
            } else {
                0.0
            };

            println!("\nAvg time per move: {:.2} ms", avg_time);
        }
        Commands::Tournament {
            options,
            bots,
            games_per_pair,
            replay_dir,
            report_json,
            opening_plies,
            threads,
        } => {
            let EvalContext {
                config,
                rule_label,
                limits,
                search_time_ms,
                search_cpu_time_ms,
                seed,
            } = eval_context(&options);
            let bot_names: Vec<String> = bots
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if bot_names.len() < 2 {
                exit_with_error("Tournament requires at least 2 bots.");
            }
            if games_per_pair % 2 != 0 {
                eprintln!(
                    "Warning: odd games-per-pair leaves each pair with uneven color coverage."
                );
            }

            println!("--- Tournament ---");
            println!("Bots: {:?}", bot_names);
            println!("Rule: {rule_label}");
            println!("Games per pair: {}", games_per_pair);
            println!("Seed: {}", seed);
            println!("Opening plies: {}", opening_plies);
            let threads = threads.unwrap_or_else(default_thread_count);
            println!("Threads: {}", threads);
            if let Some(ms) = search_time_ms {
                println!("Search time budget: {ms} ms/move");
            }
            if let Some(ms) = search_cpu_time_ms {
                println!("Search CPU-time budget: {ms} ms/move");
            }
            if let Some(max_moves) = limits.max_moves {
                println!("Max moves: {max_moves}");
            }
            if let Some(max_game_ms) = limits.max_game_ms {
                println!("Max game time: {max_game_ms} ms");
            }
            println!();

            if let Some(dir) = &replay_dir {
                std::fs::create_dir_all(dir).unwrap();
            }

            let mut factories: Vec<NamedBotFactory> = vec![];
            for name in &bot_names {
                match make_bot_factory(name, search_time_ms, search_cpu_time_ms) {
                    Ok(factory) => factories.push((name.clone(), factory)),
                    Err(err) => exit_with_error(err),
                }
            }

            let mut match_idx = 0;
            let tournament_start = Instant::now();
            let results = run_round_robin_parallel(
                &factories,
                games_per_pair,
                config.clone(),
                TournamentOptions {
                    limits,
                    seed,
                    opening_plies,
                    threads,
                },
                |black_name, white_name, mr| {
                    match_idx += 1;
                    print!(
                        "Match {:3}: {} (B) vs {} (W) - ",
                        match_idx, black_name, white_name
                    );
                    let suffix = end_reason_suffix(mr.end_reason);
                    match mr.result {
                        GameResult::Winner(Color::Black) => {
                            println!("{} wins{}", black_name, suffix)
                        }
                        GameResult::Winner(Color::White) => {
                            println!("{} wins{}", white_name, suffix)
                        }
                        GameResult::Draw => println!("Draw{}", suffix),
                        GameResult::Ongoing => unreachable!(),
                    }

                    if let Some(dir) = &replay_dir {
                        let path = dir.join(format!("match_{:03}.json", match_idx));
                        if let Ok(json) = mr.replay.to_json() {
                            std::fs::write(&path, &json)
                                .unwrap_or_else(|e| eprintln!("Failed to write replay: {e}"));
                        }
                    }
                },
            );
            let total_wall_time_ms = Some(tournament_start.elapsed().as_millis() as u64);

            let report = match TournamentReport::from_results(
                TournamentRunReport {
                    bots: bot_names,
                    rules: config,
                    games_per_pair,
                    seed,
                    opening_plies,
                    threads,
                    search_time_ms,
                    search_cpu_time_ms,
                    max_moves: limits.max_moves,
                    max_game_ms: limits.max_game_ms,
                    total_wall_time_ms,
                },
                &results,
            ) {
                Ok(report) => report,
                Err(err) => exit_with_error(format!("Failed to build tournament report: {err}")),
            };

            if let Some(path) = &report_json {
                let json = report.to_json().unwrap_or_else(|err| {
                    exit_with_error(format!("Failed to serialize report: {err}"))
                });
                std::fs::write(path, json).unwrap_or_else(|err| {
                    exit_with_error(format!("Failed to write report: {err}"))
                });
                println!("\nReport JSON: {}", path.display());
            }

            println!("\n--- Standings ---");
            for row in &report.standings {
                println!(
                    "{:<15} | Seq: {:>6.1} | Avg: {:>6.1} | W: {:>3} | D: {:>3} | L: {:>3} | ms: {:>7.2} | nodes: {:>9.0} | depth: {:>4.2} | budget: {:>5.1}%",
                    row.bot,
                    row.sequential_elo,
                    row.shuffled_elo_avg,
                    row.wins,
                    row.draws,
                    row.losses,
                    row.avg_search_time_ms,
                    row.avg_nodes,
                    row.avg_depth,
                    row.budget_exhausted_rate * 100.0
                );
            }

            println!("\n--- Pairwise ---");
            for row in &report.pairwise {
                println!(
                    "{:<15} / {:<15} | {:>3}-{:>3}-{:>3} | score {:>5.1}-{:>5.1}",
                    row.bot_a,
                    row.bot_b,
                    row.wins_a,
                    row.draws,
                    row.wins_b,
                    row.score_a,
                    row.score_b
                );
            }

            println!("\n--- Color split ---");
            for row in &report.color_splits {
                println!(
                    "{:<15} (B) vs {:<15} (W) | B: {:>3} | W: {:>3} | D: {:>3}",
                    row.black, row.white, row.black_wins, row.white_wins, row.draws
                );
            }

            println!("\n--- End reasons ---");
            for reason in &report.end_reasons {
                println!("{:<15} {}", reason.key, reason.count);
            }
        }
        Commands::ReportHtml {
            input,
            output,
            json_href,
        } => {
            let json = std::fs::read_to_string(&input)
                .unwrap_or_else(|err| exit_with_error(format!("Failed to read report: {err}")));
            let report = TournamentReport::from_json(&json)
                .unwrap_or_else(|err| exit_with_error(format!("Failed to parse report: {err}")));
            let raw_json_href = json_href.or_else(|| {
                input
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(str::to_string)
            });
            let html = render_tournament_report_html_with_options(
                &report,
                &ReportRenderOptions { raw_json_href },
            );
            std::fs::write(&output, html).unwrap_or_else(|err| {
                exit_with_error(format!("Failed to write HTML report: {err}"))
            });
            println!("HTML report: {}", output.display());
        }
        Commands::TacticalScenarios {
            bots,
            search_time_ms,
            search_cpu_time_ms,
            report_json,
        } => {
            let configs = parse_search_config_specs(&bots, search_time_ms, search_cpu_time_ms)
                .unwrap_or_else(|err| exit_with_error(err));

            println!("--- Tactical Scenarios ---");
            println!(
                "Configs: {:?}",
                configs
                    .iter()
                    .map(|config| config.id.as_str())
                    .collect::<Vec<_>>()
            );
            println!("Cases: {}", TACTICAL_SCENARIO_CASES.len());
            if let Some(ms) = search_time_ms {
                println!("Search time budget: {ms} ms/move");
            }
            if let Some(ms) = search_cpu_time_ms {
                println!("Search CPU-time budget: {ms} ms/move");
            }
            println!();

            let report = run_tactical_scenarios(&configs, TACTICAL_SCENARIO_CASES);
            for result in &report.results {
                print_tactical_scenario_result(result);
            }

            println!(
                "\n--- Summary ---\n{} passed / {} total ({} failed)",
                report.passed, report.total, report.failed
            );

            if let Some(path) = &report_json {
                let json = report.to_json().unwrap_or_else(|err| {
                    exit_with_error(format!("Failed to serialize tactical report: {err}"))
                });
                std::fs::write(path, json).unwrap_or_else(|err| {
                    exit_with_error(format!("Failed to write tactical report: {err}"))
                });
                println!("Report JSON: {}", path.display());
            }
        }
    }
}
