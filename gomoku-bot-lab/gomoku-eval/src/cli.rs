use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use gomoku_bot::{lab_spec, RandomBot, SearchBot};
use gomoku_core::{Color, GameResult, Move, Replay, RuleConfig, Variant};
use gomoku_eval::analysis::{analyze_replay, AnalysisOptions, DEFAULT_MAX_SCAN_PLIES};
use gomoku_eval::analysis_batch::{
    published_analysis_report_from_batch, run_analysis_batch_replays_with_progress,
    run_analysis_batch_with_options, AnalysisBatchReport, AnalysisBatchRunOptions,
    PublishedAnalysisMatchSummary, PublishedAnalysisSectionInput, ReplayAnalysisInput,
};
use gomoku_eval::analysis_fixture::{
    run_analysis_fixtures, AnalysisFixtureReport, AnalysisFixtureResult,
};
use gomoku_eval::analysis_report::{
    report_match_to_replay, select_report_matches, ReportReplayMatch, ReportReplaySelection,
    ReportReplaySource,
};
use gomoku_eval::arena::{run_match_series_with_limits, MatchEndReason, MatchLimits, MatchResult};
use gomoku_eval::budget::{PooledCpuBudgetConfig, PooledSearchBot};
use gomoku_eval::opening::{OpeningPolicy, CENTERED_SUITE_MAX_PLIES};
use gomoku_eval::renju_rules::{
    all_renju_rule_fixtures, run_renju_rule_fixtures, RenjuRuleFixtureResult, RenjuRuleReport,
};
use gomoku_eval::report::{
    AnchorReferenceReport, PublishedTournamentReport, TournamentReport, TournamentRunReport,
};
use gomoku_eval::seed::derive_seed;
use gomoku_eval::tournament::{
    default_thread_count, round_robin_pairs, run_scheduled_pairs_parallel, TournamentBotFactory,
    TournamentOptions, TournamentPair,
};

mod analysis;
mod options;
mod output;
mod tournament;

use analysis::*;
use options::*;
use output::*;
use tournament::*;

pub fn run() {
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
                search_budget_mode,
                search_cpu_reserve_ms,
                search_cpu_max_move_ms,
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
            if search_budget_mode == CliSearchBudgetMode::Pooled {
                println!(
                    "{}",
                    pooled_budget_label(search_cpu_reserve_ms, search_cpu_max_move_ms)
                );
            }

            if let Some(dir) = &replay_dir {
                std::fs::create_dir_all(dir).unwrap();
            }

            let bot_a_factory = match make_bot_factory(
                &bot_a,
                search_time_ms,
                search_cpu_time_ms,
                search_budget_mode,
                search_cpu_reserve_ms,
                search_cpu_max_move_ms,
            ) {
                Ok(factory) => factory,
                Err(err) => exit_with_error(err),
            };
            let bot_b_factory = match make_bot_factory(
                &bot_b,
                search_time_ms,
                search_cpu_time_ms,
                search_budget_mode,
                search_cpu_reserve_ms,
                search_cpu_max_move_ms,
            ) {
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
                search_budget_mode,
                search_cpu_reserve_ms,
                search_cpu_max_move_ms,
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
            if search_budget_mode == CliSearchBudgetMode::Pooled {
                println!(
                    "{}",
                    pooled_budget_label(search_cpu_reserve_ms, search_cpu_max_move_ms)
                );
            }

            if let Some(dir) = &replay_dir {
                std::fs::create_dir_all(dir).unwrap();
            }

            let bot_factory = match make_bot_factory(
                &bot,
                search_time_ms,
                search_cpu_time_ms,
                search_budget_mode,
                search_cpu_reserve_ms,
                search_cpu_max_move_ms,
            ) {
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
            schedule,
            bots,
            candidate,
            candidates,
            anchors,
            anchor_report,
            games_per_pair,
            replay_dir,
            report_json,
            published_report_json,
            opening_plies,
            opening_policy,
            threads,
            fail_on_shadow_mismatch,
        } => {
            let EvalContext {
                config,
                rule_label,
                limits,
                search_time_ms,
                search_cpu_time_ms,
                search_budget_mode,
                search_cpu_reserve_ms,
                search_cpu_max_move_ms,
                seed,
            } = eval_context(&options);
            let anchor_report_display = anchor_report
                .as_ref()
                .map(|path| path.to_string_lossy().into_owned());
            let TournamentPlan {
                bot_names,
                anchor_names,
                anchor_report: planned_anchor_report,
                pairs,
            } = tournament_plan(
                schedule,
                bots.as_deref(),
                candidate.as_deref(),
                candidates.as_deref(),
                anchors.as_deref(),
                anchor_report_display.as_deref(),
            )
            .unwrap_or_else(|err| exit_with_error(err));
            if games_per_pair % 2 != 0 {
                eprintln!(
                    "Warning: odd games-per-pair leaves each pair with uneven color coverage."
                );
            }
            let opening_policy: OpeningPolicy = opening_policy.into();
            if opening_policy == OpeningPolicy::CenteredSuite
                && opening_plies > CENTERED_SUITE_MAX_PLIES
            {
                exit_with_error(format!(
                    "centered-suite openings support at most {CENTERED_SUITE_MAX_PLIES} plies"
                ));
            }

            println!("--- Tournament ---");
            println!("Schedule: {}", schedule.label());
            println!("Bots: {:?}", bot_names);
            println!("Rule: {rule_label}");
            println!("Pairings: {}", pairs.len());
            println!("Games per pair: {}", games_per_pair);
            println!("Seed: {}", seed);
            println!(
                "Opening: {}, {} plies",
                opening_policy.label(),
                opening_plies
            );
            let threads = threads.unwrap_or_else(default_thread_count);
            println!("Threads: {}", threads);
            let total_games = pairs.len() * games_per_pair as usize;
            let progress_interval = tournament_progress_interval(total_games);
            if let Some(progress_interval) = progress_interval {
                println!("Progress: every {progress_interval} completed game(s)");
            }
            if let Some(ms) = search_time_ms {
                println!("Search time budget: {ms} ms/move");
            }
            if let Some(ms) = search_cpu_time_ms {
                println!("Search CPU-time budget: {ms} ms/move");
            }
            if search_budget_mode == CliSearchBudgetMode::Pooled {
                println!(
                    "{}",
                    pooled_budget_label(search_cpu_reserve_ms, search_cpu_max_move_ms)
                );
            }
            if let Some(max_moves) = limits.max_moves {
                println!("Max moves: {max_moves}");
            }
            if let Some(max_game_ms) = limits.max_game_ms {
                println!("Max game time: {max_game_ms} ms");
            }
            let mut run_report = TournamentRunReport {
                bots: bot_names.clone(),
                schedule: schedule.label().to_string(),
                rules: config.clone(),
                games_per_pair,
                seed,
                opening_plies,
                opening_policy: opening_policy.label().to_string(),
                threads,
                search_time_ms,
                search_cpu_time_ms,
                search_budget_mode: search_budget_mode.label().to_string(),
                search_cpu_reserve_ms: (search_budget_mode == CliSearchBudgetMode::Pooled)
                    .then_some(search_cpu_reserve_ms),
                search_cpu_max_move_ms: (search_budget_mode == CliSearchBudgetMode::Pooled)
                    .then_some(search_cpu_max_move_ms)
                    .flatten(),
                max_moves: limits.max_moves,
                max_game_ms: limits.max_game_ms,
                total_wall_time_ms: None,
            };
            let reference_anchors = match (&anchor_report, planned_anchor_report) {
                (Some(path), Some(source_path)) => {
                    println!("Anchor report: {source_path}");
                    let reference = load_anchor_reference(path, source_path, &anchor_names)
                        .unwrap_or_else(|err| exit_with_error(err));
                    reference
                        .validate_compatible_run(&run_report)
                        .unwrap_or_else(|err| exit_with_error(err));
                    Some(reference)
                }
                _ => None,
            };
            println!();

            if let Some(dir) = &replay_dir {
                std::fs::create_dir_all(dir).unwrap();
            }

            let mut factories: Vec<NamedBotFactory> = vec![];
            for name in &bot_names {
                match make_bot_factory(
                    name,
                    search_time_ms,
                    search_cpu_time_ms,
                    search_budget_mode,
                    search_cpu_reserve_ms,
                    search_cpu_max_move_ms,
                ) {
                    Ok(factory) => factories.push((name.clone(), factory)),
                    Err(err) => exit_with_error(err),
                }
            }

            let mut match_idx = 0;
            let tournament_start = Instant::now();
            let results = run_scheduled_pairs_parallel(
                &factories,
                &pairs,
                games_per_pair,
                config.clone(),
                TournamentOptions {
                    limits,
                    seed,
                    opening_plies,
                    opening_policy,
                    threads,
                    progress_interval,
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
            run_report.total_wall_time_ms = total_wall_time_ms;

            let mut report = match TournamentReport::from_results(run_report, &results) {
                Ok(report) => report,
                Err(err) => exit_with_error(format!("Failed to build tournament report: {err}")),
            };
            report.reference_anchors = reference_anchors;

            if let Some(path) = &report_json {
                let json = report.to_json().unwrap_or_else(|err| {
                    exit_with_error(format!("Failed to serialize report: {err}"))
                });
                std::fs::write(path, json).unwrap_or_else(|err| {
                    exit_with_error(format!("Failed to write report: {err}"))
                });
                println!("\nReport JSON: {}", path.display());
            }
            if let Some(path) = &published_report_json {
                let published_report = PublishedTournamentReport::from_tournament_report(&report);
                let json = published_report.to_json().unwrap_or_else(|err| {
                    exit_with_error(format!("Failed to serialize published report: {err}"))
                });
                std::fs::write(path, json).unwrap_or_else(|err| {
                    exit_with_error(format!("Failed to write published report: {err}"))
                });
                println!("\nPublished report JSON: {}", path.display());
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

            let shadow_mismatches = report.shadow_mismatch_count();
            if fail_on_shadow_mismatch && shadow_mismatches > 0 {
                eprintln!(
                    "Rolling frontier shadow guard failed: {shadow_mismatches} mismatch(es)."
                );
                std::process::exit(1);
            }
        }
        Commands::ReportJson { input, output } => {
            let json = std::fs::read_to_string(&input)
                .unwrap_or_else(|err| exit_with_error(format!("Failed to read report: {err}")));
            let value: serde_json::Value = serde_json::from_str(&json)
                .unwrap_or_else(|err| exit_with_error(format!("Failed to parse report: {err}")));
            let published_report =
                match value.get("report_kind").and_then(serde_json::Value::as_str) {
                    Some("tournament") => {
                        let report = TournamentReport::from_json(&json).unwrap_or_else(|err| {
                            exit_with_error(format!("Failed to parse tournament report: {err}"))
                        });
                        PublishedTournamentReport::from_tournament_report(&report)
                    }
                    Some("published_tournament") => {
                        let report =
                            PublishedTournamentReport::from_json(&json).unwrap_or_else(|err| {
                                exit_with_error(format!("Failed to parse published report: {err}"))
                            });
                        PublishedTournamentReport::from_published_report(&report)
                    }
                    Some(other) => exit_with_error(format!("Unsupported report kind: {other}")),
                    None => exit_with_error("Report is missing report_kind"),
                };
            let json = published_report.to_json().unwrap_or_else(|err| {
                exit_with_error(format!("Failed to serialize published report: {err}"))
            });
            std::fs::write(&output, json).unwrap_or_else(|err| {
                exit_with_error(format!("Failed to write published report: {err}"))
            });
            println!("Published report JSON: {}", output.display());
        }
        Commands::TacticalScenarios {
            bots,
            search_time_ms,
            search_cpu_time_ms,
            report_json,
        } => {
            let failed = crate::scenario_cli::run_tactical_scenarios_command(
                &bots,
                search_time_ms,
                search_cpu_time_ms,
                report_json.as_deref(),
            )
            .unwrap_or_else(|err| exit_with_error(err));
            if failed {
                std::process::exit(1);
            }
        }
        Commands::LethalScenarios {
            report_json,
            show_boards,
        } => {
            let failed = crate::scenario_cli::run_lethal_scenarios_command(
                report_json.as_deref(),
                show_boards,
            )
            .unwrap_or_else(|err| exit_with_error(err));
            if failed {
                std::process::exit(1);
            }
        }
        Commands::RenjuRules {
            report_json,
            show_boards,
        } => {
            println!("--- Renju Rule Fixtures ---");
            let fixtures = all_renju_rule_fixtures()
                .unwrap_or_else(|err| exit_with_error(format!("Failed to load fixtures: {err}")));
            println!("Cases: {}", fixtures.len());
            println!();

            let report = run_renju_rule_fixtures(&fixtures)
                .unwrap_or_else(|err| exit_with_error(format!("Failed to run fixtures: {err}")));
            for result in &report.results {
                print_renju_rule_fixture_result(result);
                if show_boards {
                    print_renju_rule_fixture_board(result);
                    println!();
                }
            }

            print_renju_rule_report_summary(&report);

            if let Some(path) = &report_json {
                let json = report.to_json().unwrap_or_else(|err| {
                    exit_with_error(format!("Failed to serialize Renju rule report: {err}"))
                });
                std::fs::write(path, json).unwrap_or_else(|err| {
                    exit_with_error(format!("Failed to write Renju rule report: {err}"))
                });
                println!("Report JSON: {}", path.display());
            }

            if report.failed > 0 {
                std::process::exit(1);
            }
        }
        Commands::AnalyzeReplay {
            input,
            output,
            max_depth,
            max_scan_plies,
        } => {
            let json = std::fs::read_to_string(&input)
                .unwrap_or_else(|err| exit_with_error(format!("Failed to read replay: {err}")));
            let replay = Replay::from_json(&json)
                .unwrap_or_else(|err| exit_with_error(format!("Failed to parse replay: {err}")));
            let analysis = analyze_replay(
                &replay,
                AnalysisOptions {
                    max_depth,
                    max_scan_plies: Some(max_scan_plies),
                },
            )
            .unwrap_or_else(|err| exit_with_error(format!("Failed to analyze replay: {err}")));
            let output_json = serde_json::to_string_pretty(&analysis).unwrap_or_else(|err| {
                exit_with_error(format!("Failed to serialize analysis: {err}"))
            });

            if let Some(output) = output {
                std::fs::write(&output, output_json).unwrap_or_else(|err| {
                    exit_with_error(format!("Failed to write analysis: {err}"))
                });
                println!("Analysis JSON: {}", output.display());
            } else {
                println!("{output_json}");
            }
        }
        Commands::AnalyzeReplayBatch {
            replay_dir,
            report_json,
            max_depth,
            max_scan_plies,
            include_proof_details,
        } => {
            let report = run_analysis_batch_with_options(
                &replay_dir,
                AnalysisBatchRunOptions {
                    analysis: AnalysisOptions {
                        max_depth,
                        max_scan_plies: Some(max_scan_plies),
                    },
                    include_proof_details,
                },
            )
            .unwrap_or_else(|err| {
                exit_with_error(format!("Failed to run replay analysis batch: {err}"))
            });

            print_analysis_batch_report_summary(&report);

            if let Some(path) = report_json {
                let json = serde_json::to_string_pretty(&report).unwrap_or_else(|err| {
                    exit_with_error(format!("Failed to serialize analysis batch report: {err}"))
                });
                std::fs::write(&path, json).unwrap_or_else(|err| {
                    exit_with_error(format!("Failed to write analysis batch report: {err}"))
                });
                println!("Report JSON: {}", path.display());
            }
            if report.failed > 0 {
                std::process::exit(1);
            }
        }
        Commands::AnalyzeReportReplays {
            report,
            selector,
            entrant_a,
            entrant_b,
            sample_size,
            report_json,
            published_report_json,
            max_depth,
            max_scan_plies,
            include_proof_details,
        } => {
            let json = std::fs::read_to_string(&report).unwrap_or_else(|err| {
                exit_with_error(format!("Failed to read tournament report: {err}"))
            });
            let report_source = ReportReplaySource::from_json(&json).unwrap_or_else(|err| {
                exit_with_error(format!("Failed to parse tournament report: {err}"))
            });
            let default_top_two = selector == CliReportReplaySelector::HeadToHead
                && entrant_a.is_none()
                && entrant_b.is_none();
            let section_plans = report_replay_section_plans(
                &report_source,
                selector,
                entrant_a,
                entrant_b,
                sample_size,
            )
            .unwrap_or_else(|err| exit_with_error(err));
            let published_sections = published_analysis_sections_from_plans(&section_plans);
            let inputs = flatten_report_replay_sections(&report_source, &section_plans);
            let source = match selector {
                CliReportReplaySelector::HeadToHead => {
                    let section = section_plans
                        .first()
                        .unwrap_or_else(|| exit_with_error("missing head-to-head section"));
                    report_replay_source_label(
                        &report,
                        &section.entrant_a,
                        &section.entrant_b,
                        default_top_two,
                    )
                }
                CliReportReplaySelector::PresetTriangle => {
                    format!("{}:{}", report.display(), selector.label())
                }
            };
            let progress_interval = tournament_progress_interval(inputs.len());
            let batch_report = run_analysis_batch_replays_with_progress(
                source,
                inputs,
                AnalysisBatchRunOptions {
                    analysis: AnalysisOptions {
                        max_depth,
                        max_scan_plies: Some(max_scan_plies),
                    },
                    include_proof_details: include_proof_details || published_report_json.is_some(),
                },
                progress_interval,
            );

            print_analysis_batch_report_summary(&batch_report);

            if let Some(path) = report_json {
                let json = serde_json::to_string_pretty(&batch_report).unwrap_or_else(|err| {
                    exit_with_error(format!("Failed to serialize analysis batch report: {err}"))
                });
                std::fs::write(&path, json).unwrap_or_else(|err| {
                    exit_with_error(format!("Failed to write analysis batch report: {err}"))
                });
                println!("Report JSON: {}", path.display());
            }
            if let Some(path) = published_report_json {
                let published_report = published_analysis_report_from_batch(
                    report.display().to_string(),
                    Some(&report_source.provenance),
                    selector.label().to_string(),
                    &batch_report,
                    &published_sections,
                )
                .unwrap_or_else(|err| {
                    exit_with_error(format!("Failed to build published analysis report: {err}"))
                });
                let json = published_report.to_json().unwrap_or_else(|err| {
                    exit_with_error(format!(
                        "Failed to serialize published analysis report: {err}"
                    ))
                });
                std::fs::write(&path, json).unwrap_or_else(|err| {
                    exit_with_error(format!("Failed to write published analysis report: {err}"))
                });
                println!("Published report JSON: {}", path.display());
            }
            if batch_report.failed > 0 {
                std::process::exit(1);
            }
        }
        Commands::AnalysisFixtures {
            report_json,
            max_depth,
            max_scan_plies,
        } => {
            let report = run_analysis_fixtures(AnalysisOptions {
                max_depth,
                max_scan_plies: Some(max_scan_plies),
            })
            .unwrap_or_else(|err| {
                exit_with_error(format!("Failed to run analysis fixtures: {err}"))
            });

            for result in &report.results {
                print_analysis_fixture_result(result);
            }
            print_analysis_fixture_report_summary(&report);

            if let Some(path) = report_json {
                let json = serde_json::to_string_pretty(&report).unwrap_or_else(|err| {
                    exit_with_error(format!(
                        "Failed to serialize analysis fixture report: {err}"
                    ))
                });
                std::fs::write(&path, json).unwrap_or_else(|err| {
                    exit_with_error(format!("Failed to write analysis fixture report: {err}"))
                });
                println!("Report JSON: {}", path.display());
            }
            if report.failed > 0 {
                std::process::exit(1);
            }
        }
    }
}

#[cfg(test)]
mod tests;
