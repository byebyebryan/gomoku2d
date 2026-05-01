use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::sync::Arc;

use gomoku_bot::{RandomBot, SearchBot};
use gomoku_core::{Color, GameResult, Move, RuleConfig, Variant};
use gomoku_eval::arena::{run_match_series_with_limits, MatchEndReason, MatchLimits, MatchResult};
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

    /// Rule variant: "renju" (default) or "freestyle"
    #[arg(long, global = true, default_value = "renju")]
    rule: String,

    /// Per-move search budget for search bots, in milliseconds
    #[arg(long, global = true)]
    search_time_ms: Option<u64>,

    /// Per-move Linux thread CPU-time budget for search bots, in milliseconds
    #[arg(long, global = true)]
    search_cpu_time_ms: Option<u64>,

    /// Stop a game after this many moves and record it as a draw
    #[arg(long, global = true)]
    max_moves: Option<usize>,

    /// Stop a game after this wall-clock duration and record it as a draw
    #[arg(long, global = true)]
    max_game_ms: Option<u64>,

    /// Base seed for reproducible random bots and tournament openings
    #[arg(long, global = true, default_value_t = 0)]
    seed: u64,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run N games between two different bots
    Versus {
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
        #[arg(long, default_value = "baseline")]
        bot: String,

        #[arg(long, default_value_t = 10)]
        games: u32,

        #[arg(long)]
        replay_dir: Option<PathBuf>,
    },
    /// Run a round-robin tournament among a list of bots
    Tournament {
        /// Comma-separated list of bots (e.g. "random,fast,balanced,deep,baseline-5")
        #[arg(long)]
        bots: String,

        /// Number of games each pair plays
        #[arg(long, default_value_t = 2)]
        games_per_pair: u32,

        #[arg(long)]
        replay_dir: Option<PathBuf>,

        /// Number of seeded random opening plies before bots take over
        #[arg(long, default_value_t = 4)]
        opening_plies: usize,

        /// Worker threads used to run tournament games
        #[arg(long)]
        threads: Option<usize>,
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

fn main() {
    let cli = Cli::parse();

    let variant = match cli.rule.as_str() {
        "renju" => Variant::Renju,
        "freestyle" => Variant::Freestyle,
        other => exit_with_error(format!(
            "Unknown rule variant '{other}'. Use 'renju' or 'freestyle'."
        )),
    };
    let rule_label = variant_label(&variant);
    let config = RuleConfig {
        variant,
        ..Default::default()
    };
    let limits = MatchLimits {
        max_moves: cli.max_moves,
        max_game_ms: cli.max_game_ms,
    };
    let search_time_ms = cli.search_time_ms;
    let search_cpu_time_ms = cli.search_cpu_time_ms;
    let seed = cli.seed;

    match cli.command {
        Commands::Versus {
            bot_a,
            bot_b,
            games,
            replay_dir,
        } => {
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
            bot,
            games,
            replay_dir,
        } => {
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
            bots,
            games_per_pair,
            replay_dir,
            opening_plies,
            threads,
        } => {
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
            println!("Threads: {}", threads.unwrap_or_else(default_thread_count));
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
            let results = run_round_robin_parallel(
                &factories,
                games_per_pair,
                config,
                TournamentOptions {
                    limits,
                    seed,
                    opening_plies,
                    threads: threads.unwrap_or_else(default_thread_count),
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

            println!("\n--- Standings ---");
            for (name, rating) in results.elo_tracker.get_sorted_ratings() {
                let w = results.wins.get(&name).unwrap_or(&0);
                let d = results.draws.get(&name).unwrap_or(&0);
                let l = results.losses.get(&name).unwrap_or(&0);
                let avg_ms = results.avg_time_per_move_ms(&name);
                let avg_nodes = results.avg_nodes_per_search_move(&name);
                println!(
                    "{:<15} | Elo: {:>6.1} | W: {:>3} | D: {:>3} | L: {:>3} | Avg: {:>7.2} ms | Nodes: {:>9.0}",
                    name, rating, w, d, l, avg_ms, avg_nodes
                );
            }

            println!("\n--- End reasons ---");
            for reason in [
                MatchEndReason::Natural,
                MatchEndReason::MaxMoves,
                MatchEndReason::MaxGameTime,
            ] {
                let count = results.end_reasons.get(&reason).unwrap_or(&0);
                if *count > 0 {
                    println!("{:<15} {}", reason.label(), count);
                }
            }
        }
    }
}
