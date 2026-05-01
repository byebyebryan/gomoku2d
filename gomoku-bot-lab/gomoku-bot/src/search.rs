use instant::Instant;
use std::collections::HashMap;
use std::time::Duration;

use crate::Bot;
use gomoku_core::{Board, Color, GameResult, Move, Variant, ZobristTable, DIRS};

// ZobristTable is provided by gomoku-core with a stable shared seed,
// so hashes are consistent between the search and replay recording.

fn hash_board(zt: &ZobristTable, board: &Board) -> u64 {
    let size = board.config.board_size;
    let mut h = 0u64;
    for row in 0..size {
        for col in 0..size {
            if let Some(color) = board.cell(row, col) {
                h ^= zt.piece(row, col, color);
            }
        }
    }
    if board.current_player == Color::White {
        h ^= zt.turn;
    }
    h
}

// --- Transposition table ---

#[derive(Clone, Copy)]
enum TTFlag {
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Clone, Copy)]
struct TTEntry {
    depth: i32,
    score: i32,
    flag: TTFlag,
    best_move: Option<Move>,
}

// --- Static evaluation ---

fn score_line(counts: &[i32; 6], open_ends: &[i32; 6]) -> i32 {
    let mut score = 0i32;
    for len in 2..=5usize {
        let c = counts[len];
        if c == 0 {
            continue;
        }
        let o = open_ends[len];
        let base: i32 = match len {
            5 => 1_000_000,
            4 => 10_000,
            3 => 1_000,
            2 => 100,
            _ => 0,
        };
        score += base * c * (o + 1);
    }
    score
}

fn evaluate(board: &Board, color: Color) -> i32 {
    if let GameResult::Winner(w) = &board.result {
        return if *w == color { 2_000_000 } else { -2_000_000 };
    }
    if board.result == GameResult::Draw {
        return 0;
    }

    let size = board.config.board_size;
    let win_len = board.config.win_length as isize;

    let mut my_score = 0i32;
    let mut opp_score = 0i32;
    let opp = color.opponent();

    for &player in &[color, opp] {
        let mut counts = [0i32; 6];
        let mut open_ends = [0i32; 6];

        for &(dr, dc) in &DIRS {
            for row in 0..size as isize {
                for col in 0..size as isize {
                    // Only start a new run from the "back" end to avoid double-counting
                    let pr = row - dr;
                    let pc = col - dc;
                    let back_in_bounds =
                        pr >= 0 && pr < size as isize && pc >= 0 && pc < size as isize;
                    if back_in_bounds && board.cell(pr as usize, pc as usize) == Some(player) {
                        continue;
                    }
                    if board.cell(row as usize, col as usize) != Some(player) {
                        continue;
                    }
                    // Count run length
                    let mut len = 0isize;
                    let (mut r, mut c) = (row, col);
                    while r >= 0
                        && r < size as isize
                        && c >= 0
                        && c < size as isize
                        && board.cell(r as usize, c as usize) == Some(player)
                    {
                        len += 1;
                        r += dr;
                        c += dc;
                    }
                    if len >= win_len {
                        if player == color {
                            return 2_000_000;
                        } else {
                            return -2_000_000;
                        }
                    }
                    if len < 2 {
                        continue;
                    }
                    // Count open ends
                    let mut ends = 0i32;
                    // Before
                    let (br, bc) = (row - dr, col - dc);
                    if br >= 0
                        && br < size as isize
                        && bc >= 0
                        && bc < size as isize
                        && board.cell(br as usize, bc as usize).is_none()
                    {
                        ends += 1;
                    }
                    // After
                    if r >= 0
                        && r < size as isize
                        && c >= 0
                        && c < size as isize
                        && board.cell(r as usize, c as usize).is_none()
                    {
                        ends += 1;
                    }
                    if ends > 0 {
                        let idx = len.min(5) as usize;
                        counts[idx] += 1;
                        open_ends[idx] += ends;
                    }
                }
            }
        }

        let s = score_line(&counts, &open_ends);
        if player == color {
            my_score += s;
        } else {
            opp_score += s;
        }
    }

    my_score - opp_score
}

// --- Candidate move generation ---

fn candidate_moves(board: &Board, radius: usize) -> Vec<Move> {
    if board.result != GameResult::Ongoing {
        return Vec::new();
    }

    let size = board.config.board_size;
    let mut seen = vec![false; size * size];
    let mut moves = Vec::new();
    let mut has_stones = false;

    // Generate candidates from the actual board position rather than move history.
    // This keeps search robust for reconstructed boards (e.g. snapshots sent to a worker).
    for row in 0..size {
        for col in 0..size {
            if board.cell(row, col).is_none() {
                continue;
            }

            has_stones = true;

            let rmin = row.saturating_sub(radius);
            let rmax = (row + radius).min(size - 1);
            let cmin = col.saturating_sub(radius);
            let cmax = (col + radius).min(size - 1);
            for r in rmin..=rmax {
                for c in cmin..=cmax {
                    let idx = r * size + c;
                    if !seen[idx] && board.cell(r, c).is_none() {
                        seen[idx] = true;
                        moves.push(Move { row: r, col: c });
                    }
                }
            }
        }
    }

    if !has_stones {
        let center = size / 2;
        return vec![Move {
            row: center,
            col: center,
        }];
    }

    moves
}

fn needs_renju_legality_check(board: &Board, color: Color) -> bool {
    board.config.variant == Variant::Renju && color == Color::Black
}

fn allows_opponent_forcing_reply(
    board: &mut Board,
    mv: Move,
    candidate_radius: usize,
    deadline: Option<Instant>,
) -> Option<bool> {
    let current = board.current_player;
    if needs_renju_legality_check(board, current) && !board.is_legal(mv) {
        return Some(false);
    }

    if deadline.is_some_and(|limit| Instant::now() >= limit) {
        return None;
    }

    let opponent = current.opponent();
    board.apply_move(mv).unwrap();

    let mut dangerous = false;
    let mut timed_out = false;
    if !matches!(board.result, GameResult::Winner(winner) if winner == current) {
        for reply in candidate_moves(board, candidate_radius) {
            if deadline.is_some_and(|limit| Instant::now() >= limit) {
                timed_out = true;
                break;
            }
            if needs_renju_legality_check(board, opponent) && !board.is_legal(reply) {
                continue;
            }

            board.apply_move(reply).unwrap();
            let forcing = matches!(board.result, GameResult::Winner(winner) if winner == opponent)
                || board.has_multiple_immediate_winning_moves_for(opponent);
            board.undo_move(reply);
            if forcing {
                dangerous = true;
                break;
            }
        }
    }

    board.undo_move(mv);

    if timed_out {
        None
    } else {
        Some(dangerous)
    }
}

fn root_candidate_moves(
    board: &Board,
    candidate_radius: usize,
    enable_prefilter: bool,
    deadline: Option<Instant>,
) -> Vec<Move> {
    let mut moves = candidate_moves(board, candidate_radius);
    if needs_renju_legality_check(board, board.current_player) {
        moves.retain(|&mv| board.is_legal(mv));
    }
    if moves.is_empty() || !enable_prefilter {
        return moves;
    }

    let mut working = board.clone();
    let mut safe_moves: Vec<Move> = Vec::with_capacity(moves.len());
    for mv in moves.iter().copied() {
        if deadline.is_some_and(|limit| Instant::now() >= limit) {
            return moves;
        }

        match allows_opponent_forcing_reply(&mut working, mv, candidate_radius, deadline) {
            Some(false) => safe_moves.push(mv),
            Some(true) => {}
            None => return moves,
        }
    }

    if safe_moves.is_empty() {
        moves
    } else {
        safe_moves
    }
}

// --- Negamax with alpha-beta (incremental Zobrist hash) ---

#[allow(clippy::too_many_arguments)]
fn negamax(
    board: &mut Board,
    hash: u64,
    depth: i32,
    mut alpha: i32,
    beta: i32,
    color: Color,
    root_color: Color,
    tt: &mut HashMap<u64, TTEntry>,
    zobrist: &ZobristTable,
    candidate_radius: usize,
    nodes: &mut u64,
) -> (i32, Option<Move>) {
    *nodes += 1;

    if let Some(entry) = tt.get(&hash) {
        if entry.depth >= depth {
            match entry.flag {
                TTFlag::Exact => return (entry.score, entry.best_move),
                TTFlag::LowerBound => {
                    if entry.score >= beta {
                        return (entry.score, entry.best_move);
                    }
                }
                TTFlag::UpperBound => {
                    if entry.score <= alpha {
                        return (entry.score, entry.best_move);
                    }
                }
            }
        }
    }

    if depth == 0 || board.result != GameResult::Ongoing {
        let sign = if color == root_color { 1 } else { -1 };
        return (sign * evaluate(board, root_color), None);
    }

    let moves = candidate_moves(board, candidate_radius);
    if moves.is_empty() {
        let sign = if color == root_color { 1 } else { -1 };
        return (sign * evaluate(board, root_color), None);
    }

    let orig_alpha = alpha;
    let mut best_score = i32::MIN + 1;
    let mut best_move: Option<Move> = None;

    // TT move ordering: try best move from TT first
    let tt_move = tt.get(&hash).and_then(|e| e.best_move);
    let ordered: Vec<Move> = if let Some(tm) = tt_move.filter(|tm| moves.contains(tm)) {
        std::iter::once(tm)
            .chain(moves.into_iter().filter(|&m| m != tm))
            .collect()
    } else {
        moves
    };

    let needs_legality_check = needs_renju_legality_check(board, color);
    for mv in ordered {
        if needs_legality_check && !board.is_legal(mv) {
            continue;
        }
        // Incrementally update hash: XOR in the placed piece and flip turn bit
        let child_hash = hash ^ zobrist.piece(mv.row, mv.col, color) ^ zobrist.turn;
        board.apply_move(mv).unwrap();
        let (score, _) = negamax(
            board,
            child_hash,
            depth - 1,
            -beta,
            -alpha,
            color.opponent(),
            root_color,
            tt,
            zobrist,
            candidate_radius,
            nodes,
        );
        let score = -score;
        board.undo_move(mv);

        if score > best_score {
            best_score = score;
            best_move = Some(mv);
        }
        if score > alpha {
            alpha = score;
        }
        if alpha >= beta {
            break;
        }
    }

    let flag = if best_score <= orig_alpha {
        TTFlag::UpperBound
    } else if best_score >= beta {
        TTFlag::LowerBound
    } else {
        TTFlag::Exact
    };
    tt.insert(
        hash,
        TTEntry {
            depth,
            score: best_score,
            flag,
            best_move,
        },
    );

    (best_score, best_move)
}

#[allow(clippy::too_many_arguments)]
fn search_root(
    board: &mut Board,
    hash: u64,
    depth: i32,
    root_moves: &[Move],
    color: Color,
    tt: &mut HashMap<u64, TTEntry>,
    zobrist: &ZobristTable,
    candidate_radius: usize,
    nodes: &mut u64,
) -> (i32, Option<Move>) {
    *nodes += 1;

    let mut alpha = i32::MIN + 1;
    let beta = i32::MAX;
    let orig_alpha = alpha;
    let mut best_score = i32::MIN + 1;
    let mut best_move: Option<Move> = None;

    let tt_move = tt.get(&hash).and_then(|entry| entry.best_move);
    let ordered: Vec<Move> = if let Some(tm) = tt_move.filter(|tm| root_moves.contains(tm)) {
        std::iter::once(tm)
            .chain(root_moves.iter().copied().filter(|&m| m != tm))
            .collect()
    } else {
        root_moves.to_vec()
    };

    let needs_legality_check = needs_renju_legality_check(board, color);
    for mv in ordered {
        if needs_legality_check && !board.is_legal(mv) {
            continue;
        }

        let child_hash = hash ^ zobrist.piece(mv.row, mv.col, color) ^ zobrist.turn;
        board.apply_move(mv).unwrap();
        let (score, _) = negamax(
            board,
            child_hash,
            depth - 1,
            -beta,
            -alpha,
            color.opponent(),
            color,
            tt,
            zobrist,
            candidate_radius,
            nodes,
        );
        let score = -score;
        board.undo_move(mv);

        if score > best_score {
            best_score = score;
            best_move = Some(mv);
        }
        if score > alpha {
            alpha = score;
        }
        if alpha >= beta {
            break;
        }
    }

    let flag = if best_score <= orig_alpha {
        TTFlag::UpperBound
    } else if best_score >= beta {
        TTFlag::LowerBound
    } else {
        TTFlag::Exact
    };
    tt.insert(
        hash,
        TTEntry {
            depth,
            score: best_score,
            flag,
            best_move,
        },
    );

    (best_score, best_move)
}

// --- SearchBot ---

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchBotConfig {
    pub max_depth: i32,
    pub time_budget_ms: Option<u64>,
    pub candidate_radius: usize,
    pub root_prefilter: bool,
}

impl SearchBotConfig {
    pub const fn custom_depth(max_depth: i32) -> Self {
        Self {
            max_depth,
            time_budget_ms: None,
            candidate_radius: 2,
            root_prefilter: true,
        }
    }

    pub const fn custom_time_budget(time_budget_ms: u64) -> Self {
        Self {
            max_depth: 20,
            time_budget_ms: Some(time_budget_ms),
            candidate_radius: 2,
            root_prefilter: true,
        }
    }

    fn time_budget(self) -> Option<Duration> {
        self.time_budget_ms.map(Duration::from_millis)
    }

    fn trace(self) -> serde_json::Value {
        serde_json::json!({
            "max_depth": self.max_depth,
            "time_budget_ms": self.time_budget_ms,
            "candidate_radius": self.candidate_radius,
            "root_prefilter": self.root_prefilter,
        })
    }
}

#[derive(Debug, Clone)]
pub struct SearchInfo {
    pub depth_reached: i32,
    pub nodes: u64,
    pub score: i32,
}

pub struct SearchBot {
    config: SearchBotConfig,
    tt: HashMap<u64, TTEntry>,
    zobrist: ZobristTable,
    pub last_info: Option<SearchInfo>,
}

impl SearchBot {
    pub fn new(max_depth: i32) -> Self {
        Self::with_config(SearchBotConfig::custom_depth(max_depth))
    }

    pub fn with_time(budget_ms: u64) -> Self {
        Self::with_config(SearchBotConfig::custom_time_budget(budget_ms))
    }

    pub fn with_config(config: SearchBotConfig) -> Self {
        use gomoku_core::RuleConfig;
        let board_size = RuleConfig::default().board_size;
        Self {
            config,
            tt: HashMap::new(),
            zobrist: ZobristTable::new(board_size),
            last_info: None,
        }
    }

    pub fn config(&self) -> SearchBotConfig {
        self.config
    }
}

impl Bot for SearchBot {
    fn name(&self) -> &str {
        "baseline"
    }

    fn trace(&self) -> Option<serde_json::Value> {
        self.last_info.as_ref().map(|info| {
            serde_json::json!({
                "config": self.config.trace(),
                "depth": info.depth_reached,
                "nodes": info.nodes,
                "score": info.score,
            })
        })
    }

    fn choose_move(&mut self, board: &Board) -> Move {
        let color = board.current_player;
        let mut working = board.clone();
        let start = Instant::now();
        let time_budget = self.config.time_budget();
        // Compute hash once at root; children update it incrementally
        let root_hash = hash_board(&self.zobrist, board);
        let center = board.config.board_size / 2;
        let prefilter_deadline = time_budget.map(|budget| start + budget / 2);
        let root_moves = root_candidate_moves(
            board,
            self.config.candidate_radius,
            self.config.root_prefilter,
            prefilter_deadline,
        );
        let mut best_move = root_moves
            .first()
            .copied()
            .or_else(|| {
                candidate_moves(board, self.config.candidate_radius)
                    .into_iter()
                    .next()
            })
            .unwrap_or(Move {
                row: center,
                col: center,
            });
        let mut best_score = i32::MIN + 1;
        let mut depth_reached = 0;
        let mut total_nodes = 0u64;

        for depth in 1..=self.config.max_depth {
            let mut nodes = 0u64;
            let (score, mv) = search_root(
                &mut working,
                root_hash,
                depth,
                &root_moves,
                color,
                &mut self.tt,
                &self.zobrist,
                self.config.candidate_radius,
                &mut nodes,
            );
            total_nodes += nodes;

            if let Some(m) = mv {
                best_move = m;
                best_score = score;
            }
            depth_reached = depth;

            if let Some(budget) = time_budget {
                if start.elapsed() >= budget / 2 {
                    break;
                }
            }
            // Early exit on forced win/loss
            if best_score.abs() >= 1_000_000 {
                break;
            }
        }

        self.last_info = Some(SearchInfo {
            depth_reached,
            nodes: total_nodes,
            score: best_score,
        });

        best_move
    }
}

#[cfg(test)]
#[path = "../../benchmarks/scenarios.rs"]
mod scenarios;

#[cfg(test)]
mod tests {
    use super::*;
    use gomoku_core::RuleConfig;

    #[test]
    fn finds_immediate_win() {
        // Black has 4 in a row; SearchBot should complete to 5
        let mut board = Board::new(RuleConfig::default());
        // Black: (7,7),(7,8),(7,9),(7,10) — White: safe spots
        for i in 0..4usize {
            board.apply_move(Move { row: 7, col: 7 + i }).unwrap();
            board.apply_move(Move { row: 0, col: i }).unwrap();
        }
        let mut bot = SearchBot::new(3);
        let mv = bot.choose_move(&board);
        // Should play (7,11) or (7,6) to complete the five
        assert!(mv == (Move { row: 7, col: 11 }) || mv == (Move { row: 7, col: 6 }));
    }

    #[test]
    fn blocks_opponent_win() {
        // White has 4 in a row; it's Black's turn — Black must block
        let mut board = Board::new(RuleConfig::default());
        // Black center, White: (0,0)-(0,3)
        board.apply_move(Move { row: 7, col: 7 }).unwrap(); // Black
        for i in 0..4usize {
            board.apply_move(Move { row: 0, col: i }).unwrap(); // White
            if i < 3 {
                board.apply_move(Move { row: 14, col: i }).unwrap(); // Black filler
            }
        }
        // Board state: White has (0,0),(0,1),(0,2),(0,3) — threat at (0,4) or (0,-1)
        let mut bot = SearchBot::new(3);
        let mv = bot.choose_move(&board);
        assert!(
            mv == (Move { row: 0, col: 4 }) || mv == (Move { row: 0, col: 5 }),
            "Expected block at (0,4), got {:?}",
            mv
        );
    }

    #[test]
    fn blocks_forcing_open_three_instead_of_greedy_extension() {
        // Black has an open three on row 7. White also has a tempting diagonal
        // extension at (4,4), but taking it would allow Black to create an
        // unstoppable open four on the next move.
        let mut board = Board::new(RuleConfig::default());
        for mv in [
            Move { row: 7, col: 7 },
            Move { row: 3, col: 3 },
            Move { row: 7, col: 8 },
            Move { row: 5, col: 5 },
            Move { row: 7, col: 9 },
        ] {
            board.apply_move(mv).unwrap();
        }

        let mut bot = SearchBot::new(3);
        let mv = bot.choose_move(&board);

        assert!(
            mv == (Move { row: 7, col: 6 }) || mv == (Move { row: 7, col: 10 }),
            "Expected block at (7,6) or (7,10), got {:?}",
            mv
        );
    }

    #[test]
    fn root_prefilter_falls_back_to_unfiltered_moves_when_deadline_has_elapsed() {
        let mut board = Board::new(RuleConfig::default());
        for mv in [
            Move { row: 7, col: 7 },
            Move { row: 3, col: 3 },
            Move { row: 7, col: 8 },
            Move { row: 5, col: 5 },
            Move { row: 7, col: 9 },
        ] {
            board.apply_move(mv).unwrap();
        }

        let expected: Vec<Move> = candidate_moves(&board, 2)
            .into_iter()
            .filter(|&mv| board.is_legal(mv))
            .collect();

        let moves = root_candidate_moves(
            &board,
            2,
            true,
            Some(Instant::now() - Duration::from_millis(1)),
        );

        assert_eq!(moves, expected);
    }

    #[test]
    fn explicit_config_constructors_preserve_legacy_defaults() {
        assert_eq!(SearchBot::new(3).config(), SearchBotConfig::custom_depth(3));
        assert_eq!(
            SearchBot::with_time(250).config(),
            SearchBotConfig::custom_time_budget(250)
        );

        let config = SearchBotConfig {
            max_depth: 4,
            time_budget_ms: None,
            candidate_radius: 3,
            root_prefilter: false,
        };
        assert_eq!(SearchBot::with_config(config).config(), config);
    }

    #[test]
    fn reports_root_node_in_search_info() {
        let board = Board::new(RuleConfig::default());
        let mut bot = SearchBot::new(1);

        let _ = bot.choose_move(&board);
        let info = bot
            .last_info
            .expect("expected search info after choose_move");

        assert_eq!(info.depth_reached, 1);
        assert_eq!(info.nodes, 2);
    }

    #[test]
    fn trace_records_search_config() {
        let board = Board::new(RuleConfig::default());
        let mut bot = SearchBot::with_config(SearchBotConfig::custom_depth(3));

        let _ = bot.choose_move(&board);
        let trace = bot.trace().expect("expected search trace");

        assert_eq!(trace["config"]["max_depth"], 3);
        assert_eq!(trace["config"]["candidate_radius"], 2);
        assert_eq!(trace["config"]["root_prefilter"], true);
        assert_eq!(trace["depth"], 3);
    }

    #[test]
    fn benchmark_scenarios_return_legal_moves() {
        for scenario in scenarios::SCENARIOS {
            let board = scenario.board();
            let mut bot = SearchBot::new(3);
            let mv = bot.choose_move(&board);

            assert!(
                board.is_legal(mv),
                "scenario '{}' returned illegal move {:?}",
                scenario.id,
                mv
            );
        }
    }

    #[test]
    fn behavior_cases_choose_expected_moves() {
        for case in scenarios::SEARCH_BEHAVIOR_CASES {
            let board = case.scenario().board();
            let config = match case.config_id {
                "balanced" => SearchBotConfig::custom_depth(3),
                other => panic!("unknown behavior config '{}'", other),
            };
            let mut bot = SearchBot::with_config(config);
            let expected_moves = case.expected_moves();
            let actual = bot.choose_move(&board);

            assert!(
                expected_moves.contains(&actual),
                "case '{}' expected one of {:?}, got {:?}: {}",
                case.id,
                expected_moves,
                actual,
                case.description
            );
        }
    }

    #[test]
    fn benchmark_immediate_win_anchor_plays_winning_move() {
        let scenario = scenarios::SCENARIOS
            .iter()
            .find(|scenario| scenario.id == "immediate_win")
            .expect("expected immediate-win benchmark scenario");
        let board = scenario.board();
        let winning_moves = board.immediate_winning_moves_for(board.current_player);
        let mut bot = SearchBot::new(3);

        assert!(
            winning_moves.contains(&bot.choose_move(&board)),
            "expected bot to choose one of {:?}",
            winning_moves
        );
    }

    #[test]
    fn benchmark_immediate_block_anchor_blocks_opponent_win() {
        let scenario = scenarios::SCENARIOS
            .iter()
            .find(|scenario| scenario.id == "immediate_block")
            .expect("expected immediate-block benchmark scenario");
        let board = scenario.board();
        let opponent_wins = board.immediate_winning_moves_for(board.current_player.opponent());
        let mut bot = SearchBot::new(3);

        assert!(
            opponent_wins.contains(&bot.choose_move(&board)),
            "expected bot to block one of {:?}",
            opponent_wins
        );
    }
}
