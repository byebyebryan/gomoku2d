use gomoku_bot::Bot;
use gomoku_core::{Board, Color, GameResult, Move, Replay, RuleConfig};
use std::time::Instant;

/// Per-player timing accumulated during a single match.
pub struct MatchTiming {
    pub black_time_ms: u128,
    pub black_moves: u32,
    pub white_time_ms: u128,
    pub white_moves: u32,
}

/// Result of running a single match between two bots.
pub struct MatchResult {
    pub result: GameResult,
    pub replay: Replay,
    pub timing: MatchTiming,
}

/// Runs a single match between two bots, calling `on_move` after each move
/// with the move number (1-based), the player who moved, and the move itself.
pub fn run_match(
    black: &mut dyn Bot,
    white: &mut dyn Bot,
    config: RuleConfig,
    mut on_move: impl FnMut(usize, Color, Move, u64),
) -> MatchResult {
    let mut board = Board::new(config.clone());
    let mut replay = Replay::new(config, black.name(), white.name());
    let mut timing = MatchTiming {
        black_time_ms: 0,
        black_moves: 0,
        white_time_ms: 0,
        white_moves: 0,
    };

    let start = Instant::now();
    let mut move_num: usize = 0;

    loop {
        let player = board.current_player;
        let move_start = Instant::now();
        let mv: Move = match player {
            Color::Black => black.choose_move(&board),
            Color::White => white.choose_move(&board),
        };
        let move_time_ms = move_start.elapsed().as_millis();
        let trace = match player {
            Color::Black => black.trace(),
            Color::White => white.trace(),
        };

        match player {
            Color::Black => {
                timing.black_time_ms += move_time_ms;
                timing.black_moves += 1;
            }
            Color::White => {
                timing.white_time_ms += move_time_ms;
                timing.white_moves += 1;
            }
        }

        let result = board.apply_move(mv).expect("bot played illegal move");
        replay.push_move(mv, move_time_ms as u64, board.hash(), trace);

        move_num += 1;
        on_move(move_num, player, mv, move_time_ms as u64);

        if result != GameResult::Ongoing {
            replay.finish(&result, Some(start.elapsed().as_millis() as u64));
            return MatchResult {
                result,
                replay,
                timing,
            };
        }
    }
}

pub struct SeriesStats {
    pub bot_a_wins: u32,
    pub bot_b_wins: u32,
    pub draws: u32,
    pub bot_a_time_ms: u128,
    pub bot_b_time_ms: u128,
    pub bot_a_moves: u32,
    pub bot_b_moves: u32,
}

impl Default for SeriesStats {
    fn default() -> Self {
        Self::new()
    }
}

impl SeriesStats {
    pub fn new() -> Self {
        SeriesStats {
            bot_a_wins: 0,
            bot_b_wins: 0,
            draws: 0,
            bot_a_time_ms: 0,
            bot_b_time_ms: 0,
            bot_a_moves: 0,
            bot_b_moves: 0,
        }
    }
}

/// Runs a head-to-head series between Bot A and Bot B for `games` number of games.
/// Automatically alternates who plays Black to ensure fairness.
pub fn run_match_series<F>(
    mut make_bots: F,
    games: u32,
    config: RuleConfig,
    mut on_move: impl FnMut(usize, u32, Color, Move, u64),
    mut on_game_end: impl FnMut(u32, &GameResult, &Replay),
) -> SeriesStats
where
    F: FnMut() -> (Box<dyn Bot>, Box<dyn Bot>),
{
    let mut stats = SeriesStats::new();

    for i in 0..games {
        let (mut bot_a, mut bot_b) = make_bots();

        let bot_a_plays_black = i % 2 == 0;
        let game_idx = i;
        let mr = if bot_a_plays_black {
            run_match(
                bot_a.as_mut(),
                bot_b.as_mut(),
                config.clone(),
                |move_num, player, mv, time_ms| on_move(move_num, game_idx, player, mv, time_ms),
            )
        } else {
            run_match(
                bot_b.as_mut(),
                bot_a.as_mut(),
                config.clone(),
                |move_num, player, mv, time_ms| on_move(move_num, game_idx, player, mv, time_ms),
            )
        };

        match mr.result {
            GameResult::Winner(Color::Black) => {
                if bot_a_plays_black {
                    stats.bot_a_wins += 1;
                } else {
                    stats.bot_b_wins += 1;
                }
            }
            GameResult::Winner(Color::White) => {
                if bot_a_plays_black {
                    stats.bot_b_wins += 1;
                } else {
                    stats.bot_a_wins += 1;
                }
            }
            GameResult::Draw => {
                stats.draws += 1;
            }
            GameResult::Ongoing => unreachable!(),
        }

        // Accumulate timing from the match directly.
        let (a_time, a_moves, b_time, b_moves) = if bot_a_plays_black {
            (
                mr.timing.black_time_ms,
                mr.timing.black_moves,
                mr.timing.white_time_ms,
                mr.timing.white_moves,
            )
        } else {
            (
                mr.timing.white_time_ms,
                mr.timing.white_moves,
                mr.timing.black_time_ms,
                mr.timing.black_moves,
            )
        };
        stats.bot_a_time_ms += a_time;
        stats.bot_a_moves += a_moves;
        stats.bot_b_time_ms += b_time;
        stats.bot_b_moves += b_moves;

        on_game_end(i, &mr.result, &mr.replay);
    }

    stats
}

#[cfg(test)]
mod tests {
    use super::*;
    use gomoku_bot::RandomBot;
    use gomoku_core::Variant;

    #[test]
    fn test_run_match() {
        let config = RuleConfig {
            variant: Variant::Freestyle,
            ..Default::default()
        };
        let mut black = RandomBot::new();
        let mut white = RandomBot::new();

        let mr = run_match(&mut black, &mut white, config, |_, _, _, _| {});

        assert!(mr.result != GameResult::Ongoing);
        assert!(!mr.replay.moves.is_empty());
        assert!(mr.timing.black_moves > 0);
        assert!(mr.timing.white_moves > 0);
    }
}
