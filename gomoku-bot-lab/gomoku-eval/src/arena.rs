use gomoku_bot::Bot;
use gomoku_core::{Board, Color, GameResult, Move, Replay, RuleConfig};
use std::time::Instant;

/// Optional evaluation-side safety limits for bot matches.
///
/// These limits do not change Gomoku rules. If a limit is reached before the
/// board reaches a terminal state, the eval harness records the game as a draw.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MatchLimits {
    pub max_moves: Option<usize>,
    pub max_game_ms: Option<u64>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MatchSetup {
    /// Pre-applied opening moves before bots take over.
    pub opening_moves: Vec<Move>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MatchEndReason {
    Natural,
    MaxMoves,
    MaxGameTime,
}

impl MatchEndReason {
    pub fn label(self) -> &'static str {
        match self {
            MatchEndReason::Natural => "natural",
            MatchEndReason::MaxMoves => "max moves",
            MatchEndReason::MaxGameTime => "max game time",
        }
    }
}

/// Per-player timing accumulated during a single match.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MatchTiming {
    pub black_time_ms: u128,
    pub black_moves: u32,
    pub white_time_ms: u128,
    pub white_moves: u32,
}

/// Result of running a single match between two bots.
#[derive(Debug, Clone)]
pub struct MatchResult {
    pub result: GameResult,
    pub replay: Replay,
    pub timing: MatchTiming,
    pub end_reason: MatchEndReason,
}

fn match_end_reason(
    result: &GameResult,
    move_num: usize,
    limits: MatchLimits,
    start: &Instant,
    check_time_limit: bool,
) -> Option<MatchEndReason> {
    if *result != GameResult::Ongoing {
        return Some(MatchEndReason::Natural);
    }
    if limits
        .max_moves
        .is_some_and(|max_moves| move_num >= max_moves)
    {
        return Some(MatchEndReason::MaxMoves);
    }
    if check_time_limit
        && limits
            .max_game_ms
            .is_some_and(|max_ms| start.elapsed().as_millis() >= max_ms as u128)
    {
        return Some(MatchEndReason::MaxGameTime);
    }
    None
}

fn finalize_match(
    mut replay: Replay,
    board_result: GameResult,
    timing: MatchTiming,
    end_reason: MatchEndReason,
    start: &Instant,
) -> MatchResult {
    let result = if end_reason == MatchEndReason::Natural {
        board_result
    } else {
        GameResult::Draw
    };
    replay.finish(&result, Some(start.elapsed().as_millis() as u64));
    MatchResult {
        result,
        replay,
        timing,
        end_reason,
    }
}

/// Runs a single match between two bots, calling `on_move` after each move
/// with the move number (1-based), the player who moved, and the move itself.
pub fn run_match(
    black: &mut dyn Bot,
    white: &mut dyn Bot,
    config: RuleConfig,
    on_move: impl FnMut(usize, Color, Move, u64),
) -> MatchResult {
    run_match_with_limits(black, white, config, MatchLimits::default(), on_move)
}

pub fn run_match_with_limits(
    black: &mut dyn Bot,
    white: &mut dyn Bot,
    config: RuleConfig,
    limits: MatchLimits,
    mut on_move: impl FnMut(usize, Color, Move, u64),
) -> MatchResult {
    run_match_with_setup(
        black,
        white,
        config,
        limits,
        MatchSetup::default(),
        &mut on_move,
    )
}

pub fn run_match_with_setup(
    black: &mut dyn Bot,
    white: &mut dyn Bot,
    config: RuleConfig,
    limits: MatchLimits,
    setup: MatchSetup,
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
    for mv in setup.opening_moves {
        let player = board.current_player;
        let result = board
            .apply_move(mv)
            .expect("opening bot played illegal move");
        replay.push_move(mv, 0, board.hash(), None);

        move_num += 1;
        on_move(move_num, player, mv, 0);

        if let Some(end_reason) = match_end_reason(&result, move_num, limits, &start, false) {
            return finalize_match(replay, result, timing, end_reason, &start);
        }
    }

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

        if let Some(end_reason) = match_end_reason(&result, move_num, limits, &start, true) {
            return finalize_match(replay, result, timing, end_reason, &start);
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
    run_match_series_with_limits(
        &mut make_bots,
        games,
        config,
        MatchLimits::default(),
        &mut on_move,
        |game_idx, mr| on_game_end(game_idx, &mr.result, &mr.replay),
    )
}

pub fn run_match_series_with_limits<F>(
    mut make_bots: F,
    games: u32,
    config: RuleConfig,
    limits: MatchLimits,
    mut on_move: impl FnMut(usize, u32, Color, Move, u64),
    mut on_game_end: impl FnMut(u32, &MatchResult),
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
            run_match_with_limits(
                bot_a.as_mut(),
                bot_b.as_mut(),
                config.clone(),
                limits,
                |move_num, player, mv, time_ms| on_move(move_num, game_idx, player, mv, time_ms),
            )
        } else {
            run_match_with_limits(
                bot_b.as_mut(),
                bot_a.as_mut(),
                config.clone(),
                limits,
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

        on_game_end(i, &mr);
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
        assert_eq!(mr.end_reason, MatchEndReason::Natural);
    }

    #[test]
    fn max_moves_caps_match_as_draw() {
        let config = RuleConfig {
            variant: Variant::Freestyle,
            ..Default::default()
        };
        let mut black = RandomBot::new();
        let mut white = RandomBot::new();

        let mr = run_match_with_limits(
            &mut black,
            &mut white,
            config,
            MatchLimits {
                max_moves: Some(1),
                max_game_ms: None,
            },
            |_, _, _, _| {},
        );

        assert_eq!(mr.result, GameResult::Draw);
        assert_eq!(mr.end_reason, MatchEndReason::MaxMoves);
        assert_eq!(mr.replay.moves.len(), 1);
    }

    #[test]
    fn natural_opening_win_precedes_move_cap() {
        let config = RuleConfig {
            variant: Variant::Freestyle,
            ..Default::default()
        };
        let mut black = RandomBot::new();
        let mut white = RandomBot::new();
        let opening_moves = ["H8", "A1", "I8", "A2", "J8", "A3", "K8", "B1", "L8"]
            .into_iter()
            .map(|notation| Move::from_notation(notation).expect("opening move should parse"))
            .collect();
        let mut callbacks = Vec::new();

        let mr = run_match_with_setup(
            &mut black,
            &mut white,
            config,
            MatchLimits {
                max_moves: Some(9),
                max_game_ms: Some(0),
            },
            MatchSetup { opening_moves },
            |move_num, player, mv, move_time_ms| {
                callbacks.push((move_num, player, mv, move_time_ms));
            },
        );

        assert_eq!(mr.result, GameResult::Winner(Color::Black));
        assert_eq!(mr.end_reason, MatchEndReason::Natural);
        assert_eq!(mr.replay.moves.len(), 9);
        assert_eq!(callbacks.len(), 9);
        assert_eq!(
            callbacks[8],
            (9, Color::Black, Move::from_notation("L8").unwrap(), 0)
        );
        assert_eq!(mr.timing, MatchTiming::default());
    }

    #[test]
    fn zero_move_limit_records_first_bot_move_before_ending() {
        let config = RuleConfig {
            variant: Variant::Freestyle,
            ..Default::default()
        };
        let mut black = RandomBot::new();
        let mut white = RandomBot::new();
        let mut callbacks = Vec::new();

        let mr = run_match_with_limits(
            &mut black,
            &mut white,
            config,
            MatchLimits {
                max_moves: Some(0),
                max_game_ms: Some(0),
            },
            |move_num, player, mv, move_time_ms| {
                callbacks.push((move_num, player, mv, move_time_ms));
            },
        );

        assert_eq!(mr.result, GameResult::Draw);
        assert_eq!(mr.end_reason, MatchEndReason::MaxMoves);
        assert_eq!(mr.replay.moves.len(), 1);
        assert_eq!(callbacks.len(), 1);
        assert_eq!(callbacks[0].0, 1);
        assert_eq!(mr.timing.black_moves + mr.timing.white_moves, 1);
    }

    #[test]
    fn zero_game_time_limit_is_checked_after_first_bot_move() {
        let config = RuleConfig {
            variant: Variant::Freestyle,
            ..Default::default()
        };
        let mut black = RandomBot::new();
        let mut white = RandomBot::new();
        let mut callbacks = Vec::new();

        let mr = run_match_with_limits(
            &mut black,
            &mut white,
            config,
            MatchLimits {
                max_moves: None,
                max_game_ms: Some(0),
            },
            |move_num, player, mv, move_time_ms| {
                callbacks.push((move_num, player, mv, move_time_ms));
            },
        );

        assert_eq!(mr.result, GameResult::Draw);
        assert_eq!(mr.end_reason, MatchEndReason::MaxGameTime);
        assert_eq!(mr.replay.moves.len(), 1);
        assert_eq!(callbacks.len(), 1);
        assert_eq!(callbacks[0].0, 1);
        assert_eq!(mr.timing.black_moves + mr.timing.white_moves, 1);
    }
}
