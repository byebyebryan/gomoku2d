use clap::{Parser, Subcommand};
use std::path::PathBuf;

use gomoku_bot::{Bot, RandomBot, SearchBot};
use gomoku_core::{Color, GameResult, Move, RuleConfig, Variant};
use gomoku_eval::arena::run_match_series;
use gomoku_eval::tournament::run_round_robin;

#[path = "../../benchmarks/search_configs.rs"]
mod search_configs;

#[derive(Parser, Debug)]
#[command(name = "gomoku-eval", about = "Evaluation harness for Gomoku bots")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Rule variant: "freestyle" (default) or "renju"
    #[arg(long, global = true, default_value = "freestyle")]
    rule: String,
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
    },
}

type BotFactory = Box<dyn Fn() -> Box<dyn Bot>>;
type NamedBotFactory = (String, BotFactory);

fn make_bot_factory(spec: &str) -> BotFactory {
    let spec = spec.to_string();
    Box::new(move || -> Box<dyn Bot> {
        if spec == "random" {
            return Box::new(RandomBot::new());
        }

        if let Some(config) = search_configs::search_config_from_lab_spec(&spec, 5, None) {
            return Box::new(SearchBot::with_config(config));
        }

        eprintln!("Unknown bot type: '{}'. Falling back to random.", spec);
        Box::new(RandomBot::new())
    })
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

fn print_game_result(i: u32, total: u32, result: &GameResult) {
    match result {
        GameResult::Winner(c) => println!("  Game {:3}/{:3}  {:?} wins", i + 1, total, c),
        GameResult::Draw => println!("  Game {:3}/{:3}  Draw", i + 1, total),
        GameResult::Ongoing => unreachable!(),
    }
}

fn main() {
    let cli = Cli::parse();

    let variant = match cli.rule.as_str() {
        "renju" => Variant::Renju,
        "freestyle" => Variant::Freestyle,
        other => {
            eprintln!("Unknown rule variant '{}'. Using freestyle.", other);
            Variant::Freestyle
        }
    };
    let config = RuleConfig {
        variant,
        ..Default::default()
    };

    match cli.command {
        Commands::Versus {
            bot_a,
            bot_b,
            games,
            replay_dir,
        } => {
            println!("--- Versus: {} vs {} ({} games) ---", bot_a, bot_b, games);

            if let Some(dir) = &replay_dir {
                std::fs::create_dir_all(dir).unwrap();
            }

            let make_bots = || {
                let a = make_bot_factory(&bot_a)();
                let b = make_bot_factory(&bot_b)();
                (a, b)
            };

            let stats = run_match_series(
                make_bots,
                games,
                config,
                print_move_progress,
                |i, result, replay| {
                    print_game_result(i, games, result);
                    if let Some(dir) = &replay_dir {
                        let path = dir.join(format!("match_{:03}.json", i + 1));
                        if let Ok(json) = replay.to_json() {
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

            if let Some(dir) = &replay_dir {
                std::fs::create_dir_all(dir).unwrap();
            }

            let bot_spec = bot.clone();
            let make_bots = move || {
                let a = make_bot_factory(&bot_spec)();
                let b = make_bot_factory(&bot_spec)();
                (a, b)
            };

            let stats = run_match_series(
                make_bots,
                games,
                config,
                print_move_progress,
                |i, result, replay| {
                    print_game_result(i, games, result);
                    if let Some(dir) = &replay_dir {
                        let path = dir.join(format!("selfplay_{:03}.json", i + 1));
                        if let Ok(json) = replay.to_json() {
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
        } => {
            let bot_names: Vec<String> = bots.split(',').map(|s| s.trim().to_string()).collect();
            if bot_names.len() < 2 {
                eprintln!("Tournament requires at least 2 bots.");
                return;
            }

            println!("--- Tournament ---");
            println!("Bots: {:?}", bot_names);
            println!("Games per pair: {}", games_per_pair);
            println!();

            if let Some(dir) = &replay_dir {
                std::fs::create_dir_all(dir).unwrap();
            }

            let mut factories: Vec<NamedBotFactory> = vec![];
            for name in &bot_names {
                factories.push((name.clone(), make_bot_factory(name)));
            }

            let mut match_idx = 0;
            let results = run_round_robin(
                &factories,
                games_per_pair,
                config,
                |black_name, white_name, result, replay| {
                    match_idx += 1;
                    print!(
                        "Match {:3}: {} (B) vs {} (W) - ",
                        match_idx, black_name, white_name
                    );
                    match result {
                        GameResult::Winner(Color::Black) => println!("{} wins", black_name),
                        GameResult::Winner(Color::White) => println!("{} wins", white_name),
                        GameResult::Draw => println!("Draw"),
                        GameResult::Ongoing => unreachable!(),
                    }

                    if let Some(dir) = &replay_dir {
                        let path = dir.join(format!("match_{:03}.json", match_idx));
                        if let Ok(json) = replay.to_json() {
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
                println!(
                    "{:<15} | Elo: {:>6.1} | W: {:>3} | D: {:>3} | L: {:>3}",
                    name, rating, w, d, l
                );
            }
        }
    }
}
