use std::collections::HashMap;
use std::time::{Duration, Instant};

use gomoku_core::{Board, Color, Move, GameResult, DIRS, ZobristTable};
use crate::Bot;

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
enum TTFlag { Exact, LowerBound, UpperBound }

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
        if c == 0 { continue; }
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
                    let back_in_bounds = pr >= 0 && pr < size as isize && pc >= 0 && pc < size as isize;
                    if back_in_bounds && board.cell(pr as usize, pc as usize) == Some(player) {
                        continue;
                    }
                    if board.cell(row as usize, col as usize) != Some(player) {
                        continue;
                    }
                    // Count run length
                    let mut len = 0isize;
                    let (mut r, mut c) = (row, col);
                    while r >= 0 && r < size as isize && c >= 0 && c < size as isize
                        && board.cell(r as usize, c as usize) == Some(player)
                    {
                        len += 1;
                        r += dr;
                        c += dc;
                    }
                    if len >= win_len {
                        if player == color { return 2_000_000; } else { return -2_000_000; }
                    }
                    if len < 2 { continue; }
                    // Count open ends
                    let mut ends = 0i32;
                    // Before
                    let (br, bc) = (row - dr, col - dc);
                    if br >= 0 && br < size as isize && bc >= 0 && bc < size as isize
                        && board.cell(br as usize, bc as usize).is_none()
                    {
                        ends += 1;
                    }
                    // After
                    if r >= 0 && r < size as isize && c >= 0 && c < size as isize
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
        if player == color { my_score += s; } else { opp_score += s; }
    }

    my_score - opp_score
}

// --- Candidate move generation ---

fn candidate_moves(board: &Board, radius: usize) -> Vec<Move> {
    let size = board.config.board_size;
    if board.history.is_empty() {
        let center = size / 2;
        return vec![Move { row: center, col: center }];
    }

    // Flat bool array avoids 2D allocation overhead — this is called once per negamax node.
    let mut seen = vec![false; size * size];
    let mut moves = Vec::new();

    for &mv in &board.history {
        let rmin = mv.row.saturating_sub(radius);
        let rmax = (mv.row + radius).min(size - 1);
        let cmin = mv.col.saturating_sub(radius);
        let cmax = (mv.col + radius).min(size - 1);
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
    moves
}

// --- Negamax with alpha-beta (incremental Zobrist hash) ---

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

    let moves = candidate_moves(board, 2);
    if moves.is_empty() {
        let sign = if color == root_color { 1 } else { -1 };
        return (sign * evaluate(board, root_color), None);
    }

    let orig_alpha = alpha;
    let mut best_score = i32::MIN + 1;
    let mut best_move: Option<Move> = None;

    // TT move ordering: try best move from TT first
    let tt_move = tt.get(&hash).and_then(|e| e.best_move);
    let ordered: Vec<Move> = if let Some(tm) = tt_move {
        std::iter::once(tm)
            .chain(moves.into_iter().filter(|&m| m != tm))
            .collect()
    } else {
        moves
    };

    for mv in ordered {
        if !board.is_legal(mv) { continue; }
        // Incrementally update hash: XOR in the placed piece and flip turn bit
        let child_hash = hash
            ^ zobrist.piece(mv.row, mv.col, color)
            ^ zobrist.turn;
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
    tt.insert(hash, TTEntry { depth, score: best_score, flag, best_move });

    (best_score, best_move)
}

// --- SearchBot ---

#[derive(Debug, Clone)]
pub struct SearchInfo {
    pub depth_reached: i32,
    pub nodes: u64,
    pub score: i32,
}

pub struct SearchBot {
    max_depth: i32,
    time_budget: Option<Duration>,
    tt: HashMap<u64, TTEntry>,
    zobrist: ZobristTable,
    pub last_info: Option<SearchInfo>,
}

impl SearchBot {
    pub fn new(max_depth: i32) -> Self {
        Self::build(max_depth, None)
    }

    pub fn with_time(budget_ms: u64) -> Self {
        Self::build(20, Some(Duration::from_millis(budget_ms)))
    }

    fn build(max_depth: i32, time_budget: Option<Duration>) -> Self {
        use gomoku_core::RuleConfig;
        let board_size = RuleConfig::default().board_size;
        Self {
            max_depth,
            time_budget,
            tt: HashMap::new(),
            zobrist: ZobristTable::new(board_size),
            last_info: None,
        }
    }
}

impl Bot for SearchBot {
    fn name(&self) -> &str {
        "baseline"
    }

    fn trace(&self) -> Option<serde_json::Value> {
        self.last_info.as_ref().map(|info| serde_json::json!({
            "depth": info.depth_reached,
            "nodes": info.nodes,
            "score": info.score,
        }))
    }

    fn choose_move(&mut self, board: &Board) -> Move {
        let color = board.current_player;
        let mut working = board.clone();
        let start = Instant::now();
        // Compute hash once at root; children update it incrementally
        let root_hash = hash_board(&self.zobrist, board);
        let center = board.config.board_size / 2;
        let mut best_move = candidate_moves(board, 2)
            .into_iter()
            .next()
            .unwrap_or(Move { row: center, col: center });
        let mut best_score = i32::MIN + 1;
        let mut depth_reached = 0;
        let mut total_nodes = 0u64;

        for depth in 1..=self.max_depth {
            let mut nodes = 0u64;
            let (score, mv) = negamax(
                &mut working,
                root_hash,
                depth,
                i32::MIN + 1,
                i32::MAX,
                color,
                color,
                &mut self.tt,
                &self.zobrist,
                &mut nodes,
            );
            total_nodes += nodes;

            if let Some(m) = mv {
                best_move = m;
                best_score = score;
            }
            depth_reached = depth;

            if let Some(budget) = self.time_budget {
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
        assert!(mv == (Move { row: 0, col: 4 }) || mv == (Move { row: 0, col: 5 }),
            "Expected block at (0,4), got {:?}", mv);
    }
}
