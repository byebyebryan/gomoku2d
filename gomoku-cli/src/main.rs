use std::time::Instant;

use clap::Parser;
use gomoku_core::{Board, Color, GameResult, Move, RuleConfig};
use gomoku_bot::{Bot, RandomBot, SearchBot};
use gomoku_core::Replay;

#[derive(Parser, Debug)]
#[command(name = "gomoku-cli", about = "Run a Gomoku match between bots")]
struct Args {
    /// Bot for Black: "random" or "baseline"
    #[arg(long, default_value = "baseline")]
    black: String,

    /// Bot for White: "random" or "baseline"
    #[arg(long, default_value = "random")]
    white: String,

    /// Fixed search depth (used when bot is "baseline" and no time budget is set)
    #[arg(long, default_value_t = 5)]
    depth: i32,

    /// Time budget per move in milliseconds (overrides depth if set)
    #[arg(long)]
    time_ms: Option<u64>,

    /// Write replay JSON to this path
    #[arg(long)]
    replay: Option<String>,

    /// Suppress per-move board printing
    #[arg(long)]
    quiet: bool,
}

fn make_bot(name: &str, depth: i32, time_ms: Option<u64>) -> Box<dyn Bot> {
    match name {
        "random" => Box::new(RandomBot::new()),
        "baseline" => {
            if let Some(ms) = time_ms {
                Box::new(SearchBot::with_time(ms))
            } else {
                Box::new(SearchBot::new(depth))
            }
        }
        other => {
            eprintln!("Unknown bot '{}'. Using random.", other);
            Box::new(RandomBot::new())
        }
    }
}

fn print_board(board: &Board) {
    let size = board.config.board_size;
    print!("   ");
    for c in 0..size {
        let label = (b'A' + c as u8) as char;
        if c + 1 < size { print!("{} ", label); } else { print!("{}", label); }
    }
    println!();
    for row in 0..size {
        print!("{:2} ", row + 1);
        for col in 0..size {
            let ch = board.cell(row, col).map_or('.', Color::to_char);
            if col + 1 < size { print!("{} ", ch); } else { print!("{}", ch); }
        }
        println!();
    }
    println!();
}

fn color_name(c: Color) -> &'static str {
    match c { Color::Black => "Black", Color::White => "White" }
}

fn main() {
    let args = Args::parse();

    let config = RuleConfig::default();
    let mut board = Board::new(config.clone());
    let mut replay = Replay::new(config, &args.black, &args.white);

    let mut black_bot = make_bot(&args.black, args.depth, args.time_ms);
    let mut white_bot = make_bot(&args.white, args.depth, args.time_ms);

    println!("Black: {}  |  White: {}", black_bot.name(), white_bot.name());
    println!();

    let start = Instant::now();
    let mut move_num = 1;

    loop {
        if !args.quiet {
            print_board(&board);
        }

        let mv: Move = match board.current_player {
            Color::Black => black_bot.choose_move(&board),
            Color::White => white_bot.choose_move(&board),
        };

        let player = board.current_player;
        let bot_name = match player {
            Color::Black => black_bot.name(),
            Color::White => white_bot.name(),
        };
        let col_label = (b'A' + mv.col as u8) as char;
        println!(
            "Move {:3}  {}  {} {}{}",
            move_num,
            color_name(player),
            bot_name,
            col_label,
            mv.row + 1
        );

        replay.push_move(mv);
        let result = board.apply_move(mv).expect("bot played illegal move");

        match result {
            GameResult::Ongoing => {}
            GameResult::Winner(w) => {
                if !args.quiet { print_board(&board); }
                println!("\n=== {} wins! ===", color_name(w));
                replay.finish(&result, Some(start.elapsed().as_millis() as u64));
                break;
            }
            GameResult::Draw => {
                if !args.quiet { print_board(&board); }
                println!("\n=== Draw! ===");
                replay.finish(&result, Some(start.elapsed().as_millis() as u64));
                break;
            }
        }

        move_num += 1;
    }

    println!("\nTotal moves: {}  |  Time: {:.2}s", board.history.len(), start.elapsed().as_secs_f64());

    if let Some(path) = &args.replay {
        match replay.to_json() {
            Ok(json) => {
                std::fs::write(path, &json).unwrap_or_else(|e| eprintln!("Failed to write replay: {e}"));
                println!("Replay written to: {}", path);
            }
            Err(e) => eprintln!("Failed to serialize replay: {e}"),
        }
    }
}
