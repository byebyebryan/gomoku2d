use instant::Instant;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::Duration;

use crate::corridor;
use crate::tactical::{
    local_threat_facts_after_move, LocalThreatKind, SearchThreatPolicy, TacticalMoveAnnotation,
};
use crate::Bot;
use gomoku_core::{Board, Color, GameResult, Move, Variant, ZobristTable, DIRS};

#[cfg(test)]
use crate::tactical::{LocalThreatFact, LocalThreatOrigin};

// ZobristTable is provided by gomoku-core with a stable shared seed,
// so hashes are consistent between the search and replay recording.

#[cfg(target_os = "linux")]
fn thread_cpu_time() -> Option<Duration> {
    let mut ts = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    let ok = unsafe { libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, &mut ts) == 0 };
    if ok {
        Some(Duration::new(ts.tv_sec as u64, ts.tv_nsec as u32))
    } else {
        None
    }
}

#[cfg(not(target_os = "linux"))]
fn thread_cpu_time() -> Option<Duration> {
    None
}

#[derive(Clone, Copy)]
struct SearchDeadline {
    wall_deadline: Option<Instant>,
    cpu_start: Option<Duration>,
    cpu_budget: Option<Duration>,
}

impl SearchDeadline {
    fn new(
        wall_start: Instant,
        wall_budget: Option<Duration>,
        cpu_start: Option<Duration>,
        cpu_budget: Option<Duration>,
    ) -> Self {
        Self {
            wall_deadline: wall_budget.map(|budget| wall_start + budget),
            cpu_start,
            cpu_budget,
        }
    }

    fn expired(self) -> bool {
        if self
            .wall_deadline
            .is_some_and(|deadline| Instant::now() >= deadline)
        {
            return true;
        }

        if let (Some(start), Some(budget), Some(now)) =
            (self.cpu_start, self.cpu_budget, thread_cpu_time())
        {
            return now.saturating_sub(start) >= budget;
        }

        false
    }
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

fn evaluate_static(board: &Board, color: Color, static_eval: StaticEvaluation) -> i32 {
    match static_eval {
        StaticEvaluation::LineShapeEval => evaluate(board, color),
        StaticEvaluation::PatternEval => evaluate_pattern(board, color),
    }
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

    let mut counts = [[0i32; 6]; 2];
    let mut open_ends = [[0i32; 6]; 2];
    let mut terminal_score = None;

    for &(dr, dc) in &DIRS {
        board.for_each_occupied(|row, col, player| {
            if terminal_score.is_some() {
                return;
            }

            let row = row as isize;
            let col = col as isize;

            // Only score a contiguous run once, from its back end.
            let pr = row - dr;
            let pc = col - dc;
            if pr >= 0
                && pr < size as isize
                && pc >= 0
                && pc < size as isize
                && board.has_color(pr as usize, pc as usize, player)
            {
                return;
            }

            let mut len = 0isize;
            let (mut r, mut c) = (row, col);
            while r >= 0
                && r < size as isize
                && c >= 0
                && c < size as isize
                && board.has_color(r as usize, c as usize, player)
            {
                len += 1;
                r += dr;
                c += dc;
            }

            if len >= win_len {
                terminal_score = Some(if player == color {
                    2_000_000
                } else {
                    -2_000_000
                });
                return;
            }
            if len < 2 {
                return;
            }

            let mut ends = 0i32;
            let (br, bc) = (row - dr, col - dc);
            if br >= 0
                && br < size as isize
                && bc >= 0
                && bc < size as isize
                && board.is_empty(br as usize, bc as usize)
            {
                ends += 1;
            }
            if r >= 0
                && r < size as isize
                && c >= 0
                && c < size as isize
                && board.is_empty(r as usize, c as usize)
            {
                ends += 1;
            }
            if ends > 0 {
                let score_idx = if player == color { 0 } else { 1 };
                let len_idx = len.min(5) as usize;
                counts[score_idx][len_idx] += 1;
                open_ends[score_idx][len_idx] += ends;
            }
        });

        if let Some(score) = terminal_score {
            return score;
        }
    }

    score_line(&counts[0], &open_ends[0]) - score_line(&counts[1], &open_ends[1])
}

fn evaluate_pattern(board: &Board, color: Color) -> i32 {
    if let GameResult::Winner(w) = &board.result {
        return if *w == color { 2_000_000 } else { -2_000_000 };
    }
    if board.result == GameResult::Draw {
        return 0;
    }

    let scores = pattern_scores(board);
    match color {
        Color::Black => scores.black - scores.white,
        Color::White => scores.white - scores.black,
    }
}

#[derive(Default)]
struct PatternScores {
    black: i32,
    white: i32,
}

fn pattern_scores(board: &Board) -> PatternScores {
    let size = board.config.board_size as isize;
    let mut scores = PatternScores::default();
    let mut legality_cache = PatternLegalityCache::new(board);

    for &(dr, dc) in &DIRS {
        for row in 0..size {
            for col in 0..size {
                let end_row = row + dr * 4;
                let end_col = col + dc * 4;
                if !in_bounds(board, end_row, end_col) {
                    continue;
                }

                let mut black_count = 0usize;
                let mut white_count = 0usize;
                let mut empty_moves = [Move { row: 0, col: 0 }; 5];
                let mut empty_count = 0usize;
                for offset in 0..5isize {
                    let r = (row + dr * offset) as usize;
                    let c = (col + dc * offset) as usize;
                    match board.cell(r, c) {
                        Some(Color::Black) => black_count += 1,
                        Some(Color::White) => white_count += 1,
                        None => {
                            empty_moves[empty_count] = Move { row: r, col: c };
                            empty_count += 1;
                        }
                    }
                }

                if black_count > 0 && white_count > 0 {
                    continue;
                }

                if black_count >= 2 {
                    scores.black += score_pattern_window(
                        black_count,
                        legality_cache.count_legal_black_moves(board, &empty_moves[..empty_count]),
                    );
                } else if white_count >= 2 {
                    scores.white += score_pattern_window(white_count, empty_count as i32);
                }
            }
        }
    }

    scores
}

fn score_pattern_window(player_count: usize, legal_empty_count: i32) -> i32 {
    if player_count >= 5 {
        return 1_000_000;
    }
    if legal_empty_count == 0 {
        return 0;
    }

    match player_count {
        4 => 12_000 * legal_empty_count,
        3 => 1_000 * legal_empty_count,
        2 => 80 * legal_empty_count,
        _ => 0,
    }
}

struct PatternLegalityCache {
    board_size: usize,
    renju_black: Option<Vec<Option<bool>>>,
}

impl PatternLegalityCache {
    fn new(board: &Board) -> Self {
        let needs_exact_renju_black = board.config.variant == Variant::Renju;
        let renju_black = needs_exact_renju_black
            .then(|| vec![None; board.config.board_size * board.config.board_size]);

        Self {
            board_size: board.config.board_size,
            renju_black,
        }
    }

    fn count_legal_black_moves(&mut self, board: &Board, moves: &[Move]) -> i32 {
        moves
            .iter()
            .filter(|&&mv| self.is_legal_black_move(board, mv))
            .count() as i32
    }

    fn is_legal_black_move(&mut self, board: &Board, mv: Move) -> bool {
        let Some(cache) = &mut self.renju_black else {
            return true;
        };

        let index = mv.row * self.board_size + mv.col;
        if let Some(is_legal) = cache[index] {
            return is_legal;
        }

        let is_legal = board.is_legal_for_color(mv, Color::Black);
        cache[index] = Some(is_legal);
        is_legal
    }
}

#[cfg(test)]
fn evaluate_reference(board: &Board, color: Color) -> i32 {
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
                    let pr = row - dr;
                    let pc = col - dc;
                    let back_in_bounds =
                        pr >= 0 && pr < size as isize && pc >= 0 && pc < size as isize;
                    if back_in_bounds && board.has_color(pr as usize, pc as usize, player) {
                        continue;
                    }
                    if !board.has_color(row as usize, col as usize, player) {
                        continue;
                    }

                    let mut len = 0isize;
                    let (mut r, mut c) = (row, col);
                    while r >= 0
                        && r < size as isize
                        && c >= 0
                        && c < size as isize
                        && board.has_color(r as usize, c as usize, player)
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

                    let mut ends = 0i32;
                    let (br, bc) = (row - dr, col - dc);
                    if br >= 0
                        && br < size as isize
                        && bc >= 0
                        && bc < size as isize
                        && board.is_empty(br as usize, bc as usize)
                    {
                        ends += 1;
                    }
                    if r >= 0
                        && r < size as isize
                        && c >= 0
                        && c < size as isize
                        && board.is_empty(r as usize, c as usize)
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct SearchMetrics {
    pub eval_calls: u64,
    pub candidate_generations: u64,
    pub candidate_moves_total: u64,
    pub candidate_moves_max: u64,
    pub root_candidate_generations: u64,
    pub root_candidate_moves_total: u64,
    pub root_candidate_moves_max: u64,
    pub search_candidate_generations: u64,
    pub search_candidate_moves_total: u64,
    pub search_candidate_moves_max: u64,
    pub legality_checks: u64,
    pub illegal_moves_skipped: u64,
    pub root_legality_checks: u64,
    pub root_illegal_moves_skipped: u64,
    pub search_legality_checks: u64,
    pub search_illegal_moves_skipped: u64,
    pub tactical_annotations: u64,
    pub root_tactical_annotations: u64,
    pub search_tactical_annotations: u64,
    pub child_limit_applications: u64,
    pub root_child_limit_applications: u64,
    pub search_child_limit_applications: u64,
    pub child_cap_hits: u64,
    pub root_child_cap_hits: u64,
    pub search_child_cap_hits: u64,
    pub child_moves_before_total: u64,
    pub root_child_moves_before_total: u64,
    pub search_child_moves_before_total: u64,
    pub child_moves_before_max: u64,
    pub root_child_moves_before_max: u64,
    pub search_child_moves_before_max: u64,
    pub child_moves_after_total: u64,
    pub root_child_moves_after_total: u64,
    pub search_child_moves_after_total: u64,
    pub child_moves_after_max: u64,
    pub root_child_moves_after_max: u64,
    pub search_child_moves_after_max: u64,
    pub tt_hits: u64,
    pub tt_cutoffs: u64,
    pub beta_cutoffs: u64,
    pub corridor_entry_checks: u64,
    pub corridor_entries_accepted: u64,
    pub corridor_own_entries_accepted: u64,
    pub corridor_opponent_entries_accepted: u64,
    pub corridor_nodes: u64,
    pub corridor_branch_probes: u64,
    pub corridor_resume_searches: u64,
    pub corridor_width_exits: u64,
    pub corridor_depth_exits: u64,
    pub corridor_neutral_exits: u64,
    pub corridor_terminal_exits: u64,
    pub corridor_plies_followed: u64,
    pub corridor_own_plies_followed: u64,
    pub corridor_opponent_plies_followed: u64,
    pub corridor_max_depth: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SearchMetricPhase {
    Root,
    Search,
}

impl SearchMetrics {
    fn record_candidates(&mut self, count: usize, phase: SearchMetricPhase) {
        let count = count as u64;
        self.candidate_generations += 1;
        self.candidate_moves_total += count;
        self.candidate_moves_max = self.candidate_moves_max.max(count);

        match phase {
            SearchMetricPhase::Root => {
                self.root_candidate_generations += 1;
                self.root_candidate_moves_total += count;
                self.root_candidate_moves_max = self.root_candidate_moves_max.max(count);
            }
            SearchMetricPhase::Search => {
                self.search_candidate_generations += 1;
                self.search_candidate_moves_total += count;
                self.search_candidate_moves_max = self.search_candidate_moves_max.max(count);
            }
        }
    }

    fn record_legality(&mut self, legal: bool, phase: SearchMetricPhase) -> bool {
        self.legality_checks += 1;
        if !legal {
            self.illegal_moves_skipped += 1;
        }

        match phase {
            SearchMetricPhase::Root => {
                self.root_legality_checks += 1;
                if !legal {
                    self.root_illegal_moves_skipped += 1;
                }
            }
            SearchMetricPhase::Search => {
                self.search_legality_checks += 1;
                if !legal {
                    self.search_illegal_moves_skipped += 1;
                }
            }
        }

        legal
    }

    fn record_tactical_annotation(&mut self, phase: SearchMetricPhase) {
        self.tactical_annotations += 1;
        match phase {
            SearchMetricPhase::Root => self.root_tactical_annotations += 1,
            SearchMetricPhase::Search => self.search_tactical_annotations += 1,
        }
    }

    fn record_child_limit(&mut self, before: usize, after: usize, phase: SearchMetricPhase) {
        let before = before as u64;
        let after = after as u64;

        self.child_limit_applications += 1;
        self.child_moves_before_total += before;
        self.child_moves_before_max = self.child_moves_before_max.max(before);
        self.child_moves_after_total += after;
        self.child_moves_after_max = self.child_moves_after_max.max(after);
        if after < before {
            self.child_cap_hits += 1;
        }

        match phase {
            SearchMetricPhase::Root => {
                self.root_child_limit_applications += 1;
                self.root_child_moves_before_total += before;
                self.root_child_moves_before_max = self.root_child_moves_before_max.max(before);
                self.root_child_moves_after_total += after;
                self.root_child_moves_after_max = self.root_child_moves_after_max.max(after);
                if after < before {
                    self.root_child_cap_hits += 1;
                }
            }
            SearchMetricPhase::Search => {
                self.search_child_limit_applications += 1;
                self.search_child_moves_before_total += before;
                self.search_child_moves_before_max = self.search_child_moves_before_max.max(before);
                self.search_child_moves_after_total += after;
                self.search_child_moves_after_max = self.search_child_moves_after_max.max(after);
                if after < before {
                    self.search_child_cap_hits += 1;
                }
            }
        }
    }

    fn record_corridor_entry(&mut self, side: CorridorPortalSide) {
        self.corridor_entries_accepted += 1;
        match side {
            CorridorPortalSide::Own => self.corridor_own_entries_accepted += 1,
            CorridorPortalSide::Opponent => self.corridor_opponent_entries_accepted += 1,
        }
    }

    fn record_corridor_ply(&mut self, side: CorridorPortalSide) {
        self.corridor_plies_followed += 1;
        match side {
            CorridorPortalSide::Own => self.corridor_own_plies_followed += 1,
            CorridorPortalSide::Opponent => self.corridor_opponent_plies_followed += 1,
        }
    }

    fn record_corridor_node(&mut self, depth_reached: u32) {
        self.corridor_nodes += 1;
        self.corridor_max_depth = self.corridor_max_depth.max(depth_reached);
    }

    fn trace(self) -> serde_json::Value {
        serde_json::to_value(self).expect("search metrics should serialize")
    }
}

fn evaluate_counted(
    board: &Board,
    color: Color,
    static_eval: StaticEvaluation,
    metrics: &mut SearchMetrics,
) -> i32 {
    metrics.eval_calls += 1;
    evaluate_static(board, color, static_eval)
}

fn evaluate_leaf_counted(
    board: &Board,
    color: Color,
    root_color: Color,
    static_eval: StaticEvaluation,
    metrics: &mut SearchMetrics,
) -> i32 {
    let sign = if color == root_color { 1 } else { -1 };
    sign * evaluate_counted(board, root_color, static_eval, metrics)
}

#[doc(hidden)]
pub fn pipeline_bench_evaluate(board: &Board, color: Color) -> i32 {
    evaluate(board, color)
}

#[doc(hidden)]
pub fn pipeline_bench_evaluate_static(
    board: &Board,
    color: Color,
    static_eval: StaticEvaluation,
) -> i32 {
    evaluate_static(board, color, static_eval)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
struct TacticalMoveFeatures {
    is_legal: bool,
    immediate_win: bool,
    immediate_block: bool,
    open_four: bool,
    closed_four: bool,
    open_three: bool,
    broken_three: bool,
    double_threat: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
fn analyze_tactical_move(board: &Board, mv: Move) -> TacticalMoveFeatures {
    let is_legal = board.is_legal(mv);
    if !is_legal {
        return TacticalMoveFeatures::default();
    }

    let player = board.current_player;
    let opponent = player.opponent();
    let immediate_wins_before = board.immediate_winning_moves_for(player).len();
    let local_threats = local_threat_facts_after_move(board, mv);
    let mut after = board.clone();
    after.apply_move(mv).unwrap();
    let immediate_wins_after = after.immediate_winning_moves_for(player).len();

    TacticalMoveFeatures {
        is_legal,
        immediate_win: board.immediate_winning_moves_for(player).contains(&mv),
        immediate_block: board.immediate_winning_moves_for(opponent).contains(&mv),
        open_four: local_threats
            .iter()
            .any(|fact| fact.kind == LocalThreatKind::OpenFour),
        closed_four: local_threats
            .iter()
            .any(|fact| fact.kind == LocalThreatKind::ClosedFour),
        open_three: local_threats
            .iter()
            .any(|fact| fact.kind == LocalThreatKind::OpenThree),
        broken_three: local_threats
            .iter()
            .any(|fact| fact.kind == LocalThreatKind::BrokenThree),
        double_threat: immediate_wins_after >= 2 && immediate_wins_after > immediate_wins_before,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
enum ForcedLineKind {
    ImmediateWin,
    ForcedBlock,
    UnblockableImmediateLoss,
    OpponentMultiThreat,
    Quiet,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
struct ForcedLineState {
    player: Color,
    kind: ForcedLineKind,
    immediate_wins: Vec<Move>,
    opponent_wins: Vec<Move>,
    legal_blocks: Vec<Move>,
}

#[cfg_attr(not(test), allow(dead_code))]
impl ForcedLineState {
    fn forced_block(&self) -> Option<Move> {
        if self.kind == ForcedLineKind::ForcedBlock {
            self.legal_blocks.first().copied()
        } else {
            None
        }
    }
}

#[cfg_attr(not(test), allow(dead_code))]
fn classify_forced_line_state(board: &Board) -> ForcedLineState {
    let player = board.current_player;
    let immediate_wins = board.immediate_winning_moves_for(player);
    let opponent_wins = board.immediate_winning_moves_for(player.opponent());
    let legal_blocks = opponent_wins
        .iter()
        .copied()
        .filter(|&mv| board.is_legal(mv))
        .collect::<Vec<_>>();
    let kind = if !immediate_wins.is_empty() {
        ForcedLineKind::ImmediateWin
    } else {
        match opponent_wins.len() {
            0 => ForcedLineKind::Quiet,
            1 if legal_blocks.len() == 1 => ForcedLineKind::ForcedBlock,
            1 => ForcedLineKind::UnblockableImmediateLoss,
            _ => ForcedLineKind::OpponentMultiThreat,
        }
    };

    ForcedLineState {
        player,
        kind,
        immediate_wins,
        opponent_wins,
        legal_blocks,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
enum ThreatAfterMoveKind {
    Illegal,
    WinsNow,
    SingleThreat,
    MultiThreat,
    Quiet,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
struct ThreatAfterMoveState {
    player: Color,
    kind: ThreatAfterMoveKind,
    winning_replies: Vec<Move>,
}

#[cfg_attr(not(test), allow(dead_code))]
fn classify_threat_after_move(board: &Board, mv: Move) -> ThreatAfterMoveState {
    let player = board.current_player;
    if !board.is_legal(mv) {
        return ThreatAfterMoveState {
            player,
            kind: ThreatAfterMoveKind::Illegal,
            winning_replies: Vec::new(),
        };
    }

    let mut after = board.clone();
    let result = after.apply_move(mv).unwrap();
    if matches!(result, GameResult::Winner(winner) if winner == player) {
        return ThreatAfterMoveState {
            player,
            kind: ThreatAfterMoveKind::WinsNow,
            winning_replies: Vec::new(),
        };
    }

    let winning_replies = after.immediate_winning_moves_for(player);
    let kind = match winning_replies.len() {
        0 => ThreatAfterMoveKind::Quiet,
        1 => ThreatAfterMoveKind::SingleThreat,
        _ => ThreatAfterMoveKind::MultiThreat,
    };

    ThreatAfterMoveState {
        player,
        kind,
        winning_replies,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
fn annotate_tactical_move(board: &Board, mv: Move) -> TacticalMoveAnnotation {
    SearchThreatPolicy.annotation_for_move(board, mv)
}

fn annotate_legal_tactical_move(board: &Board, mv: Move) -> TacticalMoveAnnotation {
    SearchThreatPolicy.annotation_for_move(board, mv)
}

fn annotate_legal_tactical_move_counted(
    board: &Board,
    mv: Move,
    metrics: &mut SearchMetrics,
    phase: SearchMetricPhase,
) -> TacticalMoveAnnotation {
    metrics.record_tactical_annotation(phase);
    annotate_legal_tactical_move(board, mv)
}

fn in_bounds(board: &Board, row: isize, col: isize) -> bool {
    let size = board.config.board_size as isize;
    row >= 0 && row < size && col >= 0 && col < size
}

// --- Candidate move generation ---

const STACK_SEEN_WORDS: usize = 4;
const STACK_SEEN_CELLS: usize = STACK_SEEN_WORDS * u64::BITS as usize;
const DEFAULT_BOARD_SIZE: usize = 15;

#[derive(Debug)]
struct CandidateMaskSet {
    size: usize,
    words: usize,
    masks: Vec<[u64; STACK_SEEN_WORDS]>,
}

static DEFAULT_CANDIDATE_MASKS_R1: OnceLock<CandidateMaskSet> = OnceLock::new();
static DEFAULT_CANDIDATE_MASKS_R2: OnceLock<CandidateMaskSet> = OnceLock::new();
static DEFAULT_CANDIDATE_MASKS_R3: OnceLock<CandidateMaskSet> = OnceLock::new();

fn candidate_moves(board: &Board, radius: usize) -> Vec<Move> {
    if board.result != GameResult::Ongoing {
        return Vec::new();
    }

    let size = board.config.board_size;
    let cell_count = size * size;
    let mut moves = Vec::new();
    let has_stones = if let Some(masks) = candidate_masks(size, radius) {
        let mut seen = [0u64; STACK_SEEN_WORDS];
        let mut occupied = [0u64; STACK_SEEN_WORDS];
        let has_stones = mark_candidate_moves_from_masks(board, masks, &mut seen, &mut occupied);
        collect_marked_candidates(board, &seen, &mut moves);
        has_stones
    } else if cell_count <= STACK_SEEN_CELLS {
        let mut seen = [0u64; STACK_SEEN_WORDS];
        let has_stones = mark_candidate_moves(board, radius, &mut seen);
        collect_marked_candidates(board, &seen, &mut moves);
        has_stones
    } else {
        let mut seen = vec![0u64; cell_count.div_ceil(u64::BITS as usize)];
        let has_stones = mark_candidate_moves(board, radius, &mut seen);
        collect_marked_candidates(board, &seen, &mut moves);
        has_stones
    };

    if !has_stones {
        let center = size / 2;
        return vec![Move {
            row: center,
            col: center,
        }];
    }

    moves
}

fn candidate_moves_from_source(board: &Board, candidate_source: CandidateSource) -> Vec<Move> {
    match candidate_source {
        CandidateSource::NearAll { radius } => candidate_moves(board, radius),
        CandidateSource::NearSelfOpponent {
            self_radius,
            opponent_radius,
        } => candidate_moves_from_current_and_opponent(board, self_radius, opponent_radius),
    }
}

fn candidate_moves_from_current_and_opponent(
    board: &Board,
    self_radius: usize,
    opponent_radius: usize,
) -> Vec<Move> {
    if self_radius == opponent_radius {
        return candidate_moves(board, self_radius);
    }
    if board.result != GameResult::Ongoing {
        return Vec::new();
    }

    let size = board.config.board_size;
    let cell_count = size * size;
    let mut moves = Vec::new();
    let has_stones = if cell_count <= STACK_SEEN_CELLS {
        let mut seen = [0u64; STACK_SEEN_WORDS];
        let mut occupied = [0u64; STACK_SEEN_WORDS];
        let has_stones = mark_candidate_moves_from_current_and_opponent(
            board,
            self_radius,
            opponent_radius,
            &mut seen,
            &mut occupied,
        );
        collect_marked_candidates(board, &seen, &mut moves);
        has_stones
    } else {
        let mut seen = vec![0u64; cell_count.div_ceil(u64::BITS as usize)];
        let mut occupied = vec![0u64; cell_count.div_ceil(u64::BITS as usize)];
        let has_stones = mark_candidate_moves_from_current_and_opponent(
            board,
            self_radius,
            opponent_radius,
            &mut seen,
            &mut occupied,
        );
        collect_marked_candidates(board, &seen, &mut moves);
        has_stones
    };

    if !has_stones {
        let center = size / 2;
        return vec![Move {
            row: center,
            col: center,
        }];
    }

    moves
}

fn candidate_masks(size: usize, radius: usize) -> Option<&'static CandidateMaskSet> {
    (size == DEFAULT_BOARD_SIZE && (1..=3).contains(&radius))
        .then(|| default_candidate_masks(radius))
}

fn default_candidate_masks(radius: usize) -> &'static CandidateMaskSet {
    match radius {
        1 => DEFAULT_CANDIDATE_MASKS_R1
            .get_or_init(|| build_candidate_masks(DEFAULT_BOARD_SIZE, radius)),
        2 => DEFAULT_CANDIDATE_MASKS_R2
            .get_or_init(|| build_candidate_masks(DEFAULT_BOARD_SIZE, radius)),
        3 => DEFAULT_CANDIDATE_MASKS_R3
            .get_or_init(|| build_candidate_masks(DEFAULT_BOARD_SIZE, radius)),
        _ => panic!("default candidate masks are only available for radius 1-3"),
    }
}

fn build_candidate_masks(size: usize, radius: usize) -> CandidateMaskSet {
    let words = (size * size).div_ceil(u64::BITS as usize);
    debug_assert!(words <= STACK_SEEN_WORDS);

    let mut masks = Vec::with_capacity(size * size);
    for row in 0..size {
        for col in 0..size {
            let mut mask = [0u64; STACK_SEEN_WORDS];
            let rmin = row.saturating_sub(radius);
            let rmax = (row + radius).min(size - 1);
            let cmin = col.saturating_sub(radius);
            let cmax = (col + radius).min(size - 1);
            for r in rmin..=rmax {
                for c in cmin..=cmax {
                    mark_seen(&mut mask, r * size + c);
                }
            }
            masks.push(mask);
        }
    }

    CandidateMaskSet { size, words, masks }
}

fn mark_candidate_moves_from_masks(
    board: &Board,
    masks: &CandidateMaskSet,
    seen: &mut [u64],
    occupied: &mut [u64],
) -> bool {
    let size = board.config.board_size;
    debug_assert_eq!(size, masks.size);
    let mut has_stones = false;

    board.for_each_occupied(|row, col, _| {
        has_stones = true;
        let idx = row * size + col;
        mark_seen(occupied, idx);
        for (seen_word, mask_word) in seen.iter_mut().zip(masks.masks[idx]).take(masks.words) {
            *seen_word |= mask_word;
        }
    });

    for (seen_word, occupied_word) in seen.iter_mut().zip(occupied.iter()).take(masks.words) {
        *seen_word &= !occupied_word;
    }

    has_stones
}

fn mark_candidate_moves(board: &Board, radius: usize, seen: &mut [u64]) -> bool {
    let size = board.config.board_size;
    let mut has_stones = false;

    board.for_each_occupied(|row, col, _| {
        has_stones = true;

        let rmin = row.saturating_sub(radius);
        let rmax = (row + radius).min(size - 1);
        let cmin = col.saturating_sub(radius);
        let cmax = (col + radius).min(size - 1);
        for r in rmin..=rmax {
            for c in cmin..=cmax {
                let idx = r * size + c;
                if board.is_empty(r, c) {
                    mark_seen(seen, idx);
                }
            }
        }
    });

    has_stones
}

fn mark_candidate_moves_from_current_and_opponent(
    board: &Board,
    self_radius: usize,
    opponent_radius: usize,
    seen: &mut [u64],
    occupied: &mut [u64],
) -> bool {
    let size = board.config.board_size;
    let current = board.current_player;
    let mut has_stones = false;

    board.for_each_occupied(|row, col, color| {
        has_stones = true;
        let idx = row * size + col;
        mark_seen(occupied, idx);
        let radius = if color == current {
            self_radius
        } else {
            opponent_radius
        };

        if let Some(masks) = candidate_masks(size, radius) {
            for (seen_word, mask_word) in seen.iter_mut().zip(masks.masks[idx]).take(masks.words) {
                *seen_word |= mask_word;
            }
        } else {
            let rmin = row.saturating_sub(radius);
            let rmax = (row + radius).min(size - 1);
            let cmin = col.saturating_sub(radius);
            let cmax = (col + radius).min(size - 1);
            for r in rmin..=rmax {
                for c in cmin..=cmax {
                    mark_seen(seen, r * size + c);
                }
            }
        }
    });

    for (seen_word, occupied_word) in seen.iter_mut().zip(occupied.iter()) {
        *seen_word &= !occupied_word;
    }

    has_stones
}

fn collect_marked_candidates(board: &Board, seen: &[u64], moves: &mut Vec<Move>) {
    let size = board.config.board_size;
    let cell_count = size * size;
    moves.reserve(size * size);

    for (word_idx, &word) in seen.iter().enumerate() {
        let mut bits = word;
        while bits != 0 {
            let bit_idx = bits.trailing_zeros() as usize;
            let idx = word_idx * u64::BITS as usize + bit_idx;
            if idx >= cell_count {
                return;
            }
            moves.push(Move {
                row: idx / size,
                col: idx % size,
            });
            bits &= bits - 1;
        }
    }
}

fn mark_seen(seen: &mut [u64], idx: usize) {
    let word = idx / u64::BITS as usize;
    let bit = 1u64 << (idx % u64::BITS as usize);
    seen[word] |= bit;
}

#[cfg(test)]
fn mask_contains(mask: [u64; STACK_SEEN_WORDS], mv: Move, size: usize) -> bool {
    let idx = mv.row * size + mv.col;
    let word = idx / u64::BITS as usize;
    let bit = 1u64 << (idx % u64::BITS as usize);
    mask[word] & bit != 0
}

#[cfg(test)]
fn candidate_moves_reference(board: &Board, radius: usize) -> Vec<Move> {
    if board.result != GameResult::Ongoing {
        return Vec::new();
    }

    let size = board.config.board_size;
    let mut seen = vec![false; size * size];
    let mut moves = Vec::new();
    let mut has_stones = false;

    for row in 0..size {
        for col in 0..size {
            if board.is_empty(row, col) {
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
                    if !seen[idx] && board.is_empty(r, c) {
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

#[cfg(test)]
fn candidate_moves_from_source_reference(
    board: &Board,
    self_radius: usize,
    opponent_radius: usize,
) -> Vec<Move> {
    if board.result != GameResult::Ongoing {
        return Vec::new();
    }

    let size = board.config.board_size;
    let mut seen = vec![false; size * size];
    let mut moves = Vec::new();
    let mut has_stones = false;
    let current = board.current_player;

    for row in 0..size {
        for col in 0..size {
            let Some(color) = board.cell(row, col) else {
                continue;
            };
            has_stones = true;
            let radius = if color == current {
                self_radius
            } else {
                opponent_radius
            };

            let rmin = row.saturating_sub(radius);
            let rmax = (row + radius).min(size - 1);
            let cmin = col.saturating_sub(radius);
            let cmax = (col + radius).min(size - 1);
            for r in rmin..=rmax {
                for c in cmin..=cmax {
                    let idx = r * size + c;
                    if !seen[idx] && board.is_empty(r, c) {
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

#[doc(hidden)]
pub fn pipeline_bench_candidate_moves(board: &Board, radius: usize) -> Vec<Move> {
    candidate_moves(board, radius)
}

fn candidate_moves_from_source_counted(
    board: &Board,
    candidate_source: CandidateSource,
    metrics: &mut SearchMetrics,
    phase: SearchMetricPhase,
) -> Vec<Move> {
    let moves = candidate_moves_from_source(board, candidate_source);
    metrics.record_candidates(moves.len(), phase);
    moves
}

fn needs_renju_legality_check(board: &Board, color: Color) -> bool {
    board.config.variant == Variant::Renju && color == Color::Black
}

fn needs_legality_gate(board: &Board, color: Color, legality_gate: LegalityGate) -> bool {
    match legality_gate {
        LegalityGate::ExactRules => needs_renju_legality_check(board, color),
    }
}

fn legal_by_gate_counted(
    board: &Board,
    mv: Move,
    legality_gate: LegalityGate,
    metrics: &mut SearchMetrics,
    phase: SearchMetricPhase,
) -> bool {
    match legality_gate {
        LegalityGate::ExactRules => metrics.record_legality(board.is_legal(mv), phase),
    }
}

fn allows_opponent_forcing_reply(
    board: &mut Board,
    mv: Move,
    candidate_source: CandidateSource,
    legality_gate: LegalityGate,
    deadline: SearchDeadline,
    safety_nodes: &mut u64,
    metrics: &mut SearchMetrics,
) -> Option<bool> {
    let current = board.current_player;

    if deadline.expired() {
        return None;
    }
    *safety_nodes += 1;

    let opponent = current.opponent();
    // Root moves are legality-filtered before the safety gate; only opponent
    // replies need fresh legality checks here.
    board.apply_trusted_legal_move(mv);

    let mut dangerous = false;
    let mut timed_out = false;
    if !matches!(board.result, GameResult::Winner(winner) if winner == current) {
        for reply in candidate_moves_from_source_counted(
            board,
            candidate_source,
            metrics,
            SearchMetricPhase::Root,
        ) {
            if deadline.expired() {
                timed_out = true;
                break;
            }
            if needs_legality_gate(board, opponent, legality_gate)
                && !legal_by_gate_counted(
                    board,
                    reply,
                    legality_gate,
                    metrics,
                    SearchMetricPhase::Root,
                )
            {
                continue;
            }

            *safety_nodes += 1;
            board.apply_trusted_legal_move(reply);
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

fn root_candidate_moves_with_metrics(
    board: &Board,
    candidate_source: CandidateSource,
    legality_gate: LegalityGate,
    safety_gate: SafetyGate,
    deadline: SearchDeadline,
    metrics: &mut SearchMetrics,
) -> (Vec<Move>, u64, bool) {
    let mut moves = candidate_moves_from_source_counted(
        board,
        candidate_source,
        metrics,
        SearchMetricPhase::Root,
    );
    if needs_legality_gate(board, board.current_player, legality_gate) {
        moves.retain(|&mv| {
            legal_by_gate_counted(board, mv, legality_gate, metrics, SearchMetricPhase::Root)
        });
    }

    apply_safety_gate_to_root_candidates(
        board,
        moves,
        candidate_source,
        legality_gate,
        safety_gate,
        deadline,
        metrics,
    )
}

fn apply_safety_gate_to_root_candidates(
    board: &Board,
    moves: Vec<Move>,
    candidate_source: CandidateSource,
    legality_gate: LegalityGate,
    safety_gate: SafetyGate,
    deadline: SearchDeadline,
    metrics: &mut SearchMetrics,
) -> (Vec<Move>, u64, bool) {
    match safety_gate {
        SafetyGate::None => (moves, 0, false),
        SafetyGate::OpponentReplySearchProbe => opponent_reply_search_probe_root_candidates(
            board,
            moves,
            candidate_source,
            legality_gate,
            deadline,
            metrics,
        ),
        SafetyGate::OpponentReplyLocalThreatProbe => {
            opponent_reply_local_threat_probe_root_candidates(
                board,
                moves,
                candidate_source,
                legality_gate,
                deadline,
                metrics,
            )
        }
    }
}

fn opponent_reply_search_probe_root_candidates(
    board: &Board,
    moves: Vec<Move>,
    candidate_source: CandidateSource,
    legality_gate: LegalityGate,
    deadline: SearchDeadline,
    metrics: &mut SearchMetrics,
) -> (Vec<Move>, u64, bool) {
    if moves.is_empty() {
        return (moves, 0, false);
    }

    let mut working = board.clone();
    let mut safe_moves: Vec<Move> = Vec::with_capacity(moves.len());
    let mut safety_nodes = 0u64;
    for mv in moves.iter().copied() {
        if deadline.expired() {
            return (moves, safety_nodes, true);
        }

        match allows_opponent_forcing_reply(
            &mut working,
            mv,
            candidate_source,
            legality_gate,
            deadline,
            &mut safety_nodes,
            metrics,
        ) {
            Some(false) => safe_moves.push(mv),
            Some(true) => {}
            None => return (moves, safety_nodes, true),
        }
    }

    if safe_moves.is_empty() {
        (moves, safety_nodes, false)
    } else {
        (safe_moves, safety_nodes, false)
    }
}

fn allows_opponent_local_forcing_reply(
    board: &mut Board,
    mv: Move,
    candidate_source: CandidateSource,
    legality_gate: LegalityGate,
    deadline: SearchDeadline,
    safety_nodes: &mut u64,
    metrics: &mut SearchMetrics,
) -> Option<bool> {
    let current = board.current_player;

    if deadline.expired() {
        return None;
    }
    *safety_nodes += 1;

    let opponent = current.opponent();
    board.apply_trusted_legal_move(mv);

    let mut dangerous = false;
    let mut timed_out = false;
    if !matches!(board.result, GameResult::Winner(winner) if winner == current) {
        for reply in candidate_moves_from_source_counted(
            board,
            candidate_source,
            metrics,
            SearchMetricPhase::Root,
        ) {
            if deadline.expired() {
                timed_out = true;
                break;
            }
            if needs_legality_gate(board, opponent, legality_gate)
                && !legal_by_gate_counted(
                    board,
                    reply,
                    legality_gate,
                    metrics,
                    SearchMetricPhase::Root,
                )
            {
                continue;
            }

            *safety_nodes += 1;
            debug_assert_eq!(board.current_player, opponent);
            if local_reply_creates_immediate_or_multi_threat(board, reply, metrics) {
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

fn local_reply_creates_immediate_or_multi_threat(
    board: &Board,
    reply: Move,
    metrics: &mut SearchMetrics,
) -> bool {
    annotate_legal_tactical_move_counted(board, reply, metrics, SearchMetricPhase::Root)
        .creates_immediate_or_multi_threat()
}

fn opponent_reply_local_threat_probe_root_candidates(
    board: &Board,
    moves: Vec<Move>,
    candidate_source: CandidateSource,
    legality_gate: LegalityGate,
    deadline: SearchDeadline,
    metrics: &mut SearchMetrics,
) -> (Vec<Move>, u64, bool) {
    if moves.is_empty() {
        return (moves, 0, false);
    }

    let mut working = board.clone();
    let mut safe_moves: Vec<Move> = Vec::with_capacity(moves.len());
    let mut safety_nodes = 0u64;
    for mv in moves.iter().copied() {
        if deadline.expired() {
            return (moves, safety_nodes, true);
        }

        match allows_opponent_local_forcing_reply(
            &mut working,
            mv,
            candidate_source,
            legality_gate,
            deadline,
            &mut safety_nodes,
            metrics,
        ) {
            Some(false) => safe_moves.push(mv),
            Some(true) => {}
            None => return (moves, safety_nodes, true),
        }
    }

    if safe_moves.is_empty() {
        (moves, safety_nodes, false)
    } else {
        (safe_moves, safety_nodes, false)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OrderedMove {
    mv: Move,
    must_keep: bool,
}

fn order_root_moves(
    board: &Board,
    moves: Vec<Move>,
    move_ordering: MoveOrdering,
    tt_move: Option<Move>,
    metrics: &mut SearchMetrics,
    phase: SearchMetricPhase,
) -> Vec<Move> {
    match move_ordering {
        MoveOrdering::TranspositionFirstBoardOrder => order_tt_first(moves, tt_move),
        MoveOrdering::TacticalFirst => {
            order_moves_tactical_first(board, moves, tt_move, metrics, phase)
                .into_iter()
                .map(|ordered| ordered.mv)
                .collect()
        }
    }
}

fn order_search_moves(
    board: &Board,
    moves: Vec<Move>,
    move_ordering: MoveOrdering,
    tt_move: Option<Move>,
    child_limit: Option<usize>,
    metrics: &mut SearchMetrics,
    phase: SearchMetricPhase,
) -> Vec<Move> {
    match move_ordering {
        MoveOrdering::TranspositionFirstBoardOrder => {
            let moves = order_tt_first(moves, tt_move);
            apply_plain_child_limit(moves, child_limit, metrics, phase)
        }
        MoveOrdering::TacticalFirst => {
            let ordered = order_moves_tactical_first(board, moves, tt_move, metrics, phase);
            apply_child_limit(ordered, child_limit, metrics, phase)
        }
    }
}

fn order_tt_first(mut moves: Vec<Move>, tt_move: Option<Move>) -> Vec<Move> {
    let Some(tt_move) = tt_move else {
        return moves;
    };

    if let Some(index) = moves.iter().position(|&mv| mv == tt_move) {
        if index > 0 {
            let tt_move = moves.remove(index);
            moves.insert(0, tt_move);
        }
    }

    moves
}

fn order_moves_tactical_first(
    board: &Board,
    moves: Vec<Move>,
    tt_move: Option<Move>,
    metrics: &mut SearchMetrics,
    phase: SearchMetricPhase,
) -> Vec<OrderedMove> {
    let opponent_wins = board.immediate_winning_moves_for(board.current_player.opponent());
    let mut scored = moves
        .into_iter()
        .enumerate()
        .map(|(index, mv)| {
            let annotation = annotate_legal_tactical_move_counted(board, mv, metrics, phase);
            let (score, must_keep) =
                tactical_ordering_score(&annotation, opponent_wins.contains(&mv));
            (index, mv, score, must_keep, Some(mv) == tt_move)
        })
        .collect::<Vec<_>>();

    scored.sort_by(|a, b| {
        b.2.cmp(&a.2)
            .then_with(|| b.4.cmp(&a.4))
            .then_with(|| a.0.cmp(&b.0))
    });
    scored
        .into_iter()
        .map(|(_, mv, _, must_keep, _)| OrderedMove { mv, must_keep })
        .collect()
}

fn tactical_ordering_score(
    annotation: &TacticalMoveAnnotation,
    immediate_block: bool,
) -> (i32, bool) {
    let search_policy = SearchThreatPolicy;
    let mut score = if immediate_block { 90_000 } else { 0 };
    let mut must_keep = immediate_block;
    for fact in annotation.local_threats.iter() {
        score = score.max(search_policy.ordering_score(fact.kind));
        must_keep |= search_policy.is_must_keep(fact);
    }

    (score, must_keep)
}

fn apply_child_limit(
    ordered: Vec<OrderedMove>,
    child_limit: Option<usize>,
    metrics: &mut SearchMetrics,
    phase: SearchMetricPhase,
) -> Vec<Move> {
    let Some(limit) = child_limit else {
        return ordered.into_iter().map(|ordered| ordered.mv).collect();
    };
    let limit = limit.max(1);
    let before = ordered.len();
    let moves = ordered
        .into_iter()
        .enumerate()
        .filter_map(|(index, ordered)| {
            if index < limit || ordered.must_keep {
                Some(ordered.mv)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    metrics.record_child_limit(before, moves.len(), phase);
    moves
}

fn apply_plain_child_limit(
    mut moves: Vec<Move>,
    child_limit: Option<usize>,
    metrics: &mut SearchMetrics,
    phase: SearchMetricPhase,
) -> Vec<Move> {
    let Some(limit) = child_limit else {
        return moves;
    };
    let limit = limit.max(1);
    let before = moves.len();
    moves.truncate(limit);
    metrics.record_child_limit(before, moves.len(), phase);
    moves
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SearchOutcome {
    score: i32,
    best_move: Option<Move>,
    timed_out: bool,
    corridor_extra_plies: u32,
}

impl SearchOutcome {
    fn new(score: i32, best_move: Option<Move>, timed_out: bool) -> Self {
        Self {
            score,
            best_move,
            timed_out,
            corridor_extra_plies: 0,
        }
    }
}

fn terminal_score_for_winner(winner: Color, color: Color, root_color: Color) -> i32 {
    let root_score = if winner == root_color {
        2_000_000
    } else {
        -2_000_000
    };
    if color == root_color {
        root_score
    } else {
        -root_score
    }
}

#[allow(clippy::too_many_arguments)]
fn search_child_after_move(
    board: &mut Board,
    hash: u64,
    depth: i32,
    alpha: i32,
    beta: i32,
    color: Color,
    root_color: Color,
    tt: &mut HashMap<u64, TTEntry>,
    zobrist: &ZobristTable,
    candidate_source: CandidateSource,
    legality_gate: LegalityGate,
    move_ordering: MoveOrdering,
    child_limit: Option<usize>,
    corridor_portals: CorridorPortalConfig,
    static_eval: StaticEvaluation,
    nodes: &mut u64,
    metrics: &mut SearchMetrics,
    deadline: SearchDeadline,
    portal_side: CorridorPortalSide,
    portal_config: CorridorPortalSideConfig,
    corridor_entry: bool,
) -> SearchOutcome {
    if corridor_entry
        && portal_config.enabled
        && portal_config.max_depth > 0
        && portal_config.max_reply_width > 0
    {
        metrics.record_corridor_entry(portal_side);
        return corridor_portal_search(
            board,
            hash,
            depth,
            alpha,
            beta,
            color,
            root_color,
            color.opponent(),
            portal_side,
            portal_config,
            0,
            tt,
            zobrist,
            candidate_source,
            legality_gate,
            move_ordering,
            child_limit,
            corridor_portals,
            static_eval,
            nodes,
            metrics,
            deadline,
        );
    }

    negamax(
        board,
        hash,
        depth,
        alpha,
        beta,
        color,
        root_color,
        tt,
        zobrist,
        candidate_source,
        legality_gate,
        move_ordering,
        child_limit,
        corridor_portals,
        static_eval,
        nodes,
        metrics,
        deadline,
    )
}

#[allow(clippy::too_many_arguments)]
fn resume_normal_search_after_corridor(
    board: &mut Board,
    hash: u64,
    depth: i32,
    alpha: i32,
    beta: i32,
    color: Color,
    root_color: Color,
    _tt: &mut HashMap<u64, TTEntry>,
    zobrist: &ZobristTable,
    candidate_source: CandidateSource,
    legality_gate: LegalityGate,
    move_ordering: MoveOrdering,
    child_limit: Option<usize>,
    _corridor_portals: CorridorPortalConfig,
    static_eval: StaticEvaluation,
    nodes: &mut u64,
    metrics: &mut SearchMetrics,
    deadline: SearchDeadline,
) -> SearchOutcome {
    metrics.corridor_resume_searches += 1;
    let mut resume_tt = HashMap::new();
    negamax(
        board,
        hash,
        depth,
        alpha,
        beta,
        color,
        root_color,
        &mut resume_tt,
        zobrist,
        candidate_source,
        legality_gate,
        move_ordering,
        child_limit,
        CorridorPortalConfig::DISABLED,
        static_eval,
        nodes,
        metrics,
        deadline,
    )
}

#[allow(clippy::too_many_arguments)]
fn corridor_portal_search(
    board: &mut Board,
    hash: u64,
    depth: i32,
    mut alpha: i32,
    beta: i32,
    color: Color,
    root_color: Color,
    attacker: Color,
    portal_side: CorridorPortalSide,
    portal_config: CorridorPortalSideConfig,
    portal_depth_used: usize,
    tt: &mut HashMap<u64, TTEntry>,
    zobrist: &ZobristTable,
    candidate_source: CandidateSource,
    legality_gate: LegalityGate,
    move_ordering: MoveOrdering,
    child_limit: Option<usize>,
    corridor_portals: CorridorPortalConfig,
    static_eval: StaticEvaluation,
    nodes: &mut u64,
    metrics: &mut SearchMetrics,
    deadline: SearchDeadline,
) -> SearchOutcome {
    metrics.record_corridor_node(portal_depth_used as u32);

    if deadline.expired() {
        return SearchOutcome::new(
            evaluate_leaf_counted(board, color, root_color, static_eval, metrics),
            None,
            true,
        );
    }

    if board.result != GameResult::Ongoing {
        metrics.corridor_terminal_exits += 1;
        return SearchOutcome::new(
            evaluate_leaf_counted(board, color, root_color, static_eval, metrics),
            None,
            false,
        );
    }

    if portal_depth_used >= portal_config.max_depth {
        metrics.corridor_depth_exits += 1;
        return resume_normal_search_after_corridor(
            board,
            hash,
            depth,
            alpha,
            beta,
            color,
            root_color,
            tt,
            zobrist,
            candidate_source,
            legality_gate,
            move_ordering,
            child_limit,
            corridor_portals,
            static_eval,
            nodes,
            metrics,
            deadline,
        );
    }

    let moves = if color == attacker {
        corridor::materialized_attacker_corridor_moves(board, attacker)
    } else {
        let replies = corridor::narrow_corridor_reply_moves(board, attacker);
        if replies.len() > portal_config.max_reply_width {
            metrics.corridor_width_exits += 1;
            return resume_normal_search_after_corridor(
                board,
                hash,
                depth,
                alpha,
                beta,
                color,
                root_color,
                tt,
                zobrist,
                candidate_source,
                legality_gate,
                move_ordering,
                child_limit,
                corridor_portals,
                static_eval,
                nodes,
                metrics,
                deadline,
            );
        }
        if replies.is_empty() && !board.immediate_winning_moves_for(attacker).is_empty() {
            metrics.corridor_terminal_exits += 1;
            return SearchOutcome {
                score: terminal_score_for_winner(attacker, color, root_color),
                best_move: None,
                timed_out: false,
                corridor_extra_plies: 1,
            };
        }
        replies
    };

    if moves.is_empty() {
        metrics.corridor_neutral_exits += 1;
        return resume_normal_search_after_corridor(
            board,
            hash,
            depth,
            alpha,
            beta,
            color,
            root_color,
            tt,
            zobrist,
            candidate_source,
            legality_gate,
            move_ordering,
            child_limit,
            corridor_portals,
            static_eval,
            nodes,
            metrics,
            deadline,
        );
    }

    metrics.corridor_branch_probes += moves.len() as u64;
    let mut best_score = i32::MIN + 1;
    let mut best_move = None;
    let mut best_extra_plies = 0u32;
    let mut timed_out = false;

    for mv in moves {
        if deadline.expired() {
            timed_out = true;
            break;
        }

        let child_hash = hash ^ zobrist.piece(mv.row, mv.col, color) ^ zobrist.turn;
        board.apply_trusted_legal_move(mv);
        metrics.record_corridor_ply(portal_side);
        let child = corridor_portal_search(
            board,
            child_hash,
            depth,
            -beta,
            -alpha,
            color.opponent(),
            root_color,
            attacker,
            portal_side,
            portal_config,
            portal_depth_used + 1,
            tt,
            zobrist,
            candidate_source,
            legality_gate,
            move_ordering,
            child_limit,
            corridor_portals,
            static_eval,
            nodes,
            metrics,
            deadline,
        );
        let score = -child.score;
        board.undo_move(mv);

        if child.timed_out {
            timed_out = true;
        }
        if score > best_score {
            best_score = score;
            best_move = Some(mv);
            best_extra_plies = child.corridor_extra_plies.saturating_add(1);
        }
        if score > alpha {
            alpha = score;
        }
        if alpha >= beta {
            metrics.beta_cutoffs += 1;
            break;
        }
        if timed_out {
            break;
        }
    }

    if best_move.is_none() {
        return SearchOutcome::new(
            evaluate_leaf_counted(board, color, root_color, static_eval, metrics),
            None,
            timed_out,
        );
    }

    SearchOutcome {
        score: best_score,
        best_move,
        timed_out,
        corridor_extra_plies: best_extra_plies,
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
    candidate_source: CandidateSource,
    legality_gate: LegalityGate,
    move_ordering: MoveOrdering,
    child_limit: Option<usize>,
    corridor_portals: CorridorPortalConfig,
    static_eval: StaticEvaluation,
    nodes: &mut u64,
    metrics: &mut SearchMetrics,
    deadline: SearchDeadline,
) -> SearchOutcome {
    *nodes += 1;

    if deadline.expired() {
        return SearchOutcome::new(
            evaluate_leaf_counted(board, color, root_color, static_eval, metrics),
            None,
            true,
        );
    }

    if let Some(entry) = tt.get(&hash) {
        metrics.tt_hits += 1;
        if entry.depth >= depth {
            match entry.flag {
                TTFlag::Exact => {
                    metrics.tt_cutoffs += 1;
                    return SearchOutcome::new(entry.score, entry.best_move, false);
                }
                TTFlag::LowerBound => {
                    if entry.score >= beta {
                        metrics.tt_cutoffs += 1;
                        return SearchOutcome::new(entry.score, entry.best_move, false);
                    }
                }
                TTFlag::UpperBound => {
                    if entry.score <= alpha {
                        metrics.tt_cutoffs += 1;
                        return SearchOutcome::new(entry.score, entry.best_move, false);
                    }
                }
            }
        }
    }

    if depth == 0 || board.result != GameResult::Ongoing {
        return SearchOutcome::new(
            evaluate_leaf_counted(board, color, root_color, static_eval, metrics),
            None,
            false,
        );
    }

    let mut moves = candidate_moves_from_source_counted(
        board,
        candidate_source,
        metrics,
        SearchMetricPhase::Search,
    );
    let mut needs_legality_check = needs_legality_gate(board, color, legality_gate);
    if (move_ordering == MoveOrdering::TacticalFirst || child_limit.is_some())
        && needs_legality_check
    {
        moves.retain(|&mv| {
            legal_by_gate_counted(board, mv, legality_gate, metrics, SearchMetricPhase::Search)
        });
        needs_legality_check = false;
    }
    if moves.is_empty() {
        return SearchOutcome::new(
            evaluate_leaf_counted(board, color, root_color, static_eval, metrics),
            None,
            false,
        );
    }

    let orig_alpha = alpha;
    let mut best_score = i32::MIN + 1;
    let mut best_move: Option<Move> = None;
    let mut best_corridor_extra_plies = 0u32;

    let tt_move = tt.get(&hash).and_then(|e| e.best_move);
    let ordered = order_search_moves(
        board,
        moves,
        move_ordering,
        tt_move,
        child_limit,
        metrics,
        SearchMetricPhase::Search,
    );

    let mut timed_out = false;
    for mv in ordered {
        if deadline.expired() {
            timed_out = true;
            break;
        }
        if needs_legality_check
            && !legal_by_gate_counted(board, mv, legality_gate, metrics, SearchMetricPhase::Search)
        {
            continue;
        }
        let portal_side = CorridorPortalSide::for_player(color, root_color);
        let portal_config = corridor_portals.for_side(portal_side);
        let corridor_entry = if portal_config.enabled {
            metrics.corridor_entry_checks += 1;
            corridor::is_corridor_attacker_move(board, color, mv)
        } else {
            false
        };
        // Incrementally update hash: XOR in the placed piece and flip turn bit
        let child_hash = hash ^ zobrist.piece(mv.row, mv.col, color) ^ zobrist.turn;
        board.apply_trusted_legal_move(mv);
        let child_outcome = search_child_after_move(
            board,
            child_hash,
            depth - 1,
            -beta,
            -alpha,
            color.opponent(),
            root_color,
            tt,
            zobrist,
            candidate_source,
            legality_gate,
            move_ordering,
            child_limit,
            corridor_portals,
            static_eval,
            nodes,
            metrics,
            deadline,
            portal_side,
            portal_config,
            corridor_entry,
        );
        let score = -child_outcome.score;
        board.undo_move(mv);

        if child_outcome.timed_out {
            timed_out = true;
        }
        if score > best_score {
            best_score = score;
            best_move = Some(mv);
            best_corridor_extra_plies = child_outcome.corridor_extra_plies;
        }
        if score > alpha {
            alpha = score;
        }
        if alpha >= beta {
            metrics.beta_cutoffs += 1;
            break;
        }
        if timed_out {
            break;
        }
    }

    if best_move.is_none() {
        return SearchOutcome::new(
            evaluate_leaf_counted(board, color, root_color, static_eval, metrics),
            None,
            timed_out,
        );
    }

    if timed_out {
        return SearchOutcome {
            score: best_score,
            best_move,
            timed_out: true,
            corridor_extra_plies: best_corridor_extra_plies,
        };
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

    SearchOutcome {
        score: best_score,
        best_move,
        timed_out: false,
        corridor_extra_plies: best_corridor_extra_plies,
    }
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
    candidate_source: CandidateSource,
    legality_gate: LegalityGate,
    move_ordering: MoveOrdering,
    child_limit: Option<usize>,
    corridor_portals: CorridorPortalConfig,
    static_eval: StaticEvaluation,
    nodes: &mut u64,
    metrics: &mut SearchMetrics,
    deadline: SearchDeadline,
) -> SearchOutcome {
    *nodes += 1;
    if deadline.expired() {
        return SearchOutcome::new(
            evaluate_counted(board, color, static_eval, metrics),
            None,
            true,
        );
    }

    let mut alpha = i32::MIN + 1;
    let beta = i32::MAX;
    let orig_alpha = alpha;
    let mut best_score = i32::MIN + 1;
    let mut best_move: Option<Move> = None;
    let mut best_corridor_extra_plies = 0u32;

    let tt_move = tt.get(&hash).and_then(|entry| entry.best_move);
    let ordered = order_root_moves(
        board,
        root_moves.to_vec(),
        move_ordering,
        tt_move,
        metrics,
        SearchMetricPhase::Root,
    );

    let mut timed_out = false;
    for mv in ordered {
        if deadline.expired() {
            timed_out = true;
            break;
        }

        let portal_side = CorridorPortalSide::Own;
        let portal_config = corridor_portals.for_side(portal_side);
        let corridor_entry = if portal_config.enabled {
            metrics.corridor_entry_checks += 1;
            corridor::is_corridor_attacker_move(board, color, mv)
        } else {
            false
        };
        let child_hash = hash ^ zobrist.piece(mv.row, mv.col, color) ^ zobrist.turn;
        board.apply_trusted_legal_move(mv);
        let child_outcome = search_child_after_move(
            board,
            child_hash,
            depth - 1,
            -beta,
            -alpha,
            color.opponent(),
            color,
            tt,
            zobrist,
            candidate_source,
            legality_gate,
            move_ordering,
            child_limit,
            corridor_portals,
            static_eval,
            nodes,
            metrics,
            deadline,
            portal_side,
            portal_config,
            corridor_entry,
        );
        let score = -child_outcome.score;
        board.undo_move(mv);

        if child_outcome.timed_out {
            timed_out = true;
        }
        if score > best_score {
            best_score = score;
            best_move = Some(mv);
            best_corridor_extra_plies = child_outcome.corridor_extra_plies;
        }
        if score > alpha {
            alpha = score;
        }
        if alpha >= beta {
            metrics.beta_cutoffs += 1;
            break;
        }
        if timed_out {
            break;
        }
    }

    if best_move.is_none() {
        return SearchOutcome::new(
            evaluate_counted(board, color, static_eval, metrics),
            None,
            timed_out,
        );
    }

    if timed_out {
        return SearchOutcome {
            score: best_score,
            best_move,
            timed_out: true,
            corridor_extra_plies: best_corridor_extra_plies,
        };
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

    SearchOutcome {
        score: best_score,
        best_move,
        timed_out: false,
        corridor_extra_plies: best_corridor_extra_plies,
    }
}

// --- SearchBot ---

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidateSource {
    NearAll {
        radius: usize,
    },
    NearSelfOpponent {
        self_radius: usize,
        opponent_radius: usize,
    },
}

impl CandidateSource {
    fn name(self) -> String {
        match self {
            CandidateSource::NearAll { radius } => format!("near_all_r{radius}"),
            CandidateSource::NearSelfOpponent {
                self_radius,
                opponent_radius,
            } => format!("near_self_r{self_radius}_opponent_r{opponent_radius}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegalityGate {
    ExactRules,
}

impl LegalityGate {
    const fn name(self) -> &'static str {
        match self {
            LegalityGate::ExactRules => "exact_rules",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafetyGate {
    None,
    OpponentReplySearchProbe,
    OpponentReplyLocalThreatProbe,
}

impl SafetyGate {
    const fn name(self) -> &'static str {
        match self {
            SafetyGate::None => "none",
            SafetyGate::OpponentReplySearchProbe => "opponent_reply_search_probe",
            SafetyGate::OpponentReplyLocalThreatProbe => "opponent_reply_local_threat_probe",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveOrdering {
    TranspositionFirstBoardOrder,
    TacticalFirst,
}

impl MoveOrdering {
    const fn name(self) -> &'static str {
        match self {
            MoveOrdering::TranspositionFirstBoardOrder => "tt_first_board_order",
            MoveOrdering::TacticalFirst => "tactical_first",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchAlgorithm {
    AlphaBetaIterativeDeepening,
}

impl SearchAlgorithm {
    const fn name(self) -> &'static str {
        match self {
            SearchAlgorithm::AlphaBetaIterativeDeepening => "alpha_beta_id",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaticEvaluation {
    LineShapeEval,
    PatternEval,
}

impl StaticEvaluation {
    const fn name(self) -> &'static str {
        match self {
            StaticEvaluation::LineShapeEval => "line_shape_eval",
            StaticEvaluation::PatternEval => "pattern_eval",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct CorridorPortalSideConfig {
    pub enabled: bool,
    pub max_depth: usize,
    pub max_reply_width: usize,
}

impl CorridorPortalSideConfig {
    pub const DISABLED: Self = Self {
        enabled: false,
        max_depth: 0,
        max_reply_width: 0,
    };

    fn trace(self) -> serde_json::Value {
        serde_json::json!({
            "enabled": self.enabled,
            "max_depth": self.max_depth,
            "max_reply_width": self.max_reply_width,
        })
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct CorridorPortalConfig {
    pub own: CorridorPortalSideConfig,
    pub opponent: CorridorPortalSideConfig,
}

impl CorridorPortalConfig {
    pub const DISABLED: Self = Self {
        own: CorridorPortalSideConfig::DISABLED,
        opponent: CorridorPortalSideConfig::DISABLED,
    };

    const fn for_side(self, side: CorridorPortalSide) -> CorridorPortalSideConfig {
        match side {
            CorridorPortalSide::Own => self.own,
            CorridorPortalSide::Opponent => self.opponent,
        }
    }

    fn trace(self) -> serde_json::Value {
        serde_json::json!({
            "own": self.own.trace(),
            "opponent": self.opponent.trace(),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CorridorPortalSide {
    Own,
    Opponent,
}

impl CorridorPortalSide {
    const fn for_player(player: Color, root_color: Color) -> Self {
        if matches!(
            (player, root_color),
            (Color::Black, Color::Black) | (Color::White, Color::White)
        ) {
            Self::Own
        } else {
            Self::Opponent
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchBotConfig {
    pub max_depth: i32,
    pub time_budget_ms: Option<u64>,
    pub cpu_time_budget_ms: Option<u64>,
    pub candidate_radius: usize,
    pub candidate_opponent_radius: Option<usize>,
    pub safety_gate: SafetyGate,
    pub move_ordering: MoveOrdering,
    pub child_limit: Option<usize>,
    pub search_algorithm: SearchAlgorithm,
    pub static_eval: StaticEvaluation,
    pub corridor_portals: CorridorPortalConfig,
}

impl SearchBotConfig {
    pub const fn custom_depth(max_depth: i32) -> Self {
        Self {
            max_depth,
            time_budget_ms: None,
            cpu_time_budget_ms: None,
            candidate_radius: 2,
            candidate_opponent_radius: None,
            safety_gate: SafetyGate::OpponentReplyLocalThreatProbe,
            move_ordering: MoveOrdering::TranspositionFirstBoardOrder,
            child_limit: None,
            search_algorithm: SearchAlgorithm::AlphaBetaIterativeDeepening,
            static_eval: StaticEvaluation::LineShapeEval,
            corridor_portals: CorridorPortalConfig::DISABLED,
        }
    }

    pub const fn custom_time_budget(time_budget_ms: u64) -> Self {
        Self {
            max_depth: 20,
            time_budget_ms: Some(time_budget_ms),
            cpu_time_budget_ms: None,
            candidate_radius: 2,
            candidate_opponent_radius: None,
            safety_gate: SafetyGate::OpponentReplyLocalThreatProbe,
            move_ordering: MoveOrdering::TranspositionFirstBoardOrder,
            child_limit: None,
            search_algorithm: SearchAlgorithm::AlphaBetaIterativeDeepening,
            static_eval: StaticEvaluation::LineShapeEval,
            corridor_portals: CorridorPortalConfig::DISABLED,
        }
    }

    pub const fn custom_cpu_time_budget(cpu_time_budget_ms: u64) -> Self {
        Self {
            max_depth: 20,
            time_budget_ms: None,
            cpu_time_budget_ms: Some(cpu_time_budget_ms),
            candidate_radius: 2,
            candidate_opponent_radius: None,
            safety_gate: SafetyGate::OpponentReplyLocalThreatProbe,
            move_ordering: MoveOrdering::TranspositionFirstBoardOrder,
            child_limit: None,
            search_algorithm: SearchAlgorithm::AlphaBetaIterativeDeepening,
            static_eval: StaticEvaluation::LineShapeEval,
            corridor_portals: CorridorPortalConfig::DISABLED,
        }
    }

    fn time_budget(self) -> Option<Duration> {
        self.time_budget_ms.map(Duration::from_millis)
    }

    fn cpu_time_budget(self) -> Option<Duration> {
        self.cpu_time_budget_ms.map(Duration::from_millis)
    }

    pub const fn candidate_source(self) -> CandidateSource {
        match self.candidate_opponent_radius {
            Some(opponent_radius) if opponent_radius != self.candidate_radius => {
                CandidateSource::NearSelfOpponent {
                    self_radius: self.candidate_radius,
                    opponent_radius,
                }
            }
            _ => CandidateSource::NearAll {
                radius: self.candidate_radius,
            },
        }
    }

    pub const fn legality_gate(self) -> LegalityGate {
        LegalityGate::ExactRules
    }

    pub const fn safety_gate(self) -> SafetyGate {
        self.safety_gate
    }

    fn trace(self) -> serde_json::Value {
        serde_json::json!({
            "max_depth": self.max_depth,
            "time_budget_ms": self.time_budget_ms,
            "cpu_time_budget_ms": self.cpu_time_budget_ms,
            "candidate_radius": self.candidate_radius,
            "candidate_opponent_radius": self.candidate_opponent_radius,
            "candidate_source": self.candidate_source().name(),
            "legality_gate": self.legality_gate().name(),
            "safety_gate": self.safety_gate().name(),
            "move_ordering": self.move_ordering.name(),
            "child_limit": self.child_limit,
            "search_algorithm": self.search_algorithm.name(),
            "static_eval": self.static_eval.name(),
            "corridor_portals": self.corridor_portals.trace(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct SearchInfo {
    pub depth_reached: i32,
    pub nodes: u64,
    pub corridor_extra_plies: u32,
    pub safety_nodes: u64,
    pub metrics: SearchMetrics,
    pub score: i32,
    pub budget_exhausted: bool,
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
            let total_nodes = info.nodes + info.safety_nodes + info.metrics.corridor_nodes;
            let effective_depth = info
                .depth_reached
                .saturating_add(info.corridor_extra_plies as i32);
            serde_json::json!({
                "config": self.config.trace(),
                "depth": info.depth_reached,
                "nominal_depth": self.config.max_depth,
                "effective_depth": effective_depth,
                "corridor_extra_plies": info.corridor_extra_plies,
                "nodes": info.nodes,
                "safety_nodes": info.safety_nodes,
                "corridor": {
                    "search_nodes": info.metrics.corridor_nodes,
                    "branch_probes": info.metrics.corridor_branch_probes,
                    "max_depth_reached": info.metrics.corridor_max_depth,
                    "extra_plies": info.corridor_extra_plies,
                    "resume_searches": info.metrics.corridor_resume_searches,
                    "width_exits": info.metrics.corridor_width_exits,
                    "depth_exits": info.metrics.corridor_depth_exits,
                    "neutral_exits": info.metrics.corridor_neutral_exits,
                    "terminal_exits": info.metrics.corridor_terminal_exits,
                },
                "total_nodes": total_nodes,
                "metrics": info.metrics.trace(),
                "score": info.score,
                "budget_exhausted": info.budget_exhausted,
            })
        })
    }

    fn choose_move(&mut self, board: &Board) -> Move {
        let color = board.current_player;
        let mut working = board.clone();
        let mut metrics = SearchMetrics::default();
        let start = Instant::now();
        let time_budget = self.config.time_budget();
        let cpu_time_budget = self.config.cpu_time_budget();
        let cpu_start = cpu_time_budget.and_then(|_| thread_cpu_time());
        let deadline = SearchDeadline::new(start, time_budget, cpu_start, cpu_time_budget);
        // Compute hash once at root; children update it incrementally
        let root_hash = board.hash_with(&self.zobrist);
        let center = board.config.board_size / 2;
        let candidate_source = self.config.candidate_source();
        let legality_gate = self.config.legality_gate();
        let safety_gate = self.config.safety_gate();
        let move_ordering = self.config.move_ordering;
        let safety_deadline = SearchDeadline::new(
            start,
            time_budget.map(|budget| budget / 2),
            cpu_start,
            cpu_time_budget.map(|budget| budget / 2),
        );
        let (root_moves, safety_nodes, mut budget_exhausted) = root_candidate_moves_with_metrics(
            board,
            candidate_source,
            legality_gate,
            safety_gate,
            safety_deadline,
            &mut metrics,
        );
        let mut best_move = root_moves
            .first()
            .copied()
            .or_else(|| {
                candidate_moves_from_source_counted(
                    board,
                    candidate_source,
                    &mut metrics,
                    SearchMetricPhase::Search,
                )
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
        let mut best_corridor_extra_plies = 0u32;

        for depth in 1..=self.config.max_depth {
            if deadline.expired() {
                budget_exhausted = true;
                break;
            }
            let mut nodes = 0u64;
            let outcome = search_root(
                &mut working,
                root_hash,
                depth,
                &root_moves,
                color,
                &mut self.tt,
                &self.zobrist,
                candidate_source,
                legality_gate,
                move_ordering,
                self.config.child_limit,
                self.config.corridor_portals,
                self.config.static_eval,
                &mut nodes,
                &mut metrics,
                deadline,
            );
            total_nodes += nodes;

            if !outcome.timed_out {
                if let Some(m) = outcome.best_move {
                    best_move = m;
                    best_score = outcome.score;
                    best_corridor_extra_plies = outcome.corridor_extra_plies;
                }
                depth_reached = depth;
            } else if depth_reached == 0 {
                if let Some(m) = outcome.best_move {
                    best_move = m;
                    best_score = outcome.score;
                    best_corridor_extra_plies = outcome.corridor_extra_plies;
                }
            }

            if outcome.timed_out {
                budget_exhausted = true;
                break;
            }
            // Early exit on forced win/loss
            if best_score.abs() >= 1_000_000 {
                break;
            }
        }

        self.last_info = Some(SearchInfo {
            depth_reached,
            nodes: total_nodes,
            corridor_extra_plies: best_corridor_extra_plies,
            safety_nodes,
            metrics,
            score: best_score,
            budget_exhausted,
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

    fn mv(notation: &str) -> Move {
        Move::from_notation(notation).unwrap()
    }

    fn apply_moves(board: &mut Board, moves: &[&str]) {
        for &notation in moves {
            board.apply_move(mv(notation)).unwrap();
        }
    }

    #[test]
    fn optimized_eval_matches_reference_on_benchmark_scenarios() {
        for scenario in scenarios::SCENARIOS {
            let board = scenario.board();
            for color in [Color::Black, Color::White] {
                assert_eq!(
                    evaluate(&board, color),
                    evaluate_reference(&board, color),
                    "scenario '{}' diverged for {:?}",
                    scenario.id,
                    color
                );
            }
        }
    }

    #[test]
    fn optimized_pattern_eval_matches_reference_on_benchmark_scenarios() {
        for scenario in scenarios::SCENARIOS {
            let board = scenario.board();
            for color in [Color::Black, Color::White] {
                assert_eq!(
                    evaluate_static(&board, color, StaticEvaluation::PatternEval),
                    evaluate_pattern_reference(&board, color),
                    "scenario '{}' diverged for {:?}",
                    scenario.id,
                    color
                );
            }
        }
    }

    fn evaluate_pattern_reference(board: &Board, color: Color) -> i32 {
        if let GameResult::Winner(w) = &board.result {
            return if *w == color { 2_000_000 } else { -2_000_000 };
        }
        if board.result == GameResult::Draw {
            return 0;
        }

        pattern_score_for_player_reference(board, color)
            - pattern_score_for_player_reference(board, color.opponent())
    }

    fn pattern_score_for_player_reference(board: &Board, player: Color) -> i32 {
        let size = board.config.board_size as isize;
        let mut score = 0i32;

        for &(dr, dc) in &DIRS {
            for row in 0..size {
                for col in 0..size {
                    let end_row = row + dr * 4;
                    let end_col = col + dc * 4;
                    if !in_bounds(board, end_row, end_col) {
                        continue;
                    }

                    let mut player_count = 0usize;
                    let mut empty_moves = [Move { row: 0, col: 0 }; 5];
                    let mut empty_count = 0usize;
                    let mut blocked = false;
                    for offset in 0..5isize {
                        let r = (row + dr * offset) as usize;
                        let c = (col + dc * offset) as usize;
                        match board.cell(r, c) {
                            Some(color) if color == player => player_count += 1,
                            Some(_) => {
                                blocked = true;
                                break;
                            }
                            None => {
                                empty_moves[empty_count] = Move { row: r, col: c };
                                empty_count += 1;
                            }
                        }
                    }

                    if blocked || player_count < 2 {
                        continue;
                    }

                    let legal_empty_count = empty_moves[..empty_count]
                        .iter()
                        .filter(|&&mv| board.is_legal_for_color(mv, player))
                        .count() as i32;
                    if legal_empty_count == 0 {
                        continue;
                    }

                    score += match player_count {
                        5.. => 1_000_000,
                        4 => 12_000 * legal_empty_count,
                        3 => 1_000 * legal_empty_count,
                        2 => 80 * legal_empty_count,
                        _ => 0,
                    };
                }
            }
        }

        score
    }

    #[test]
    fn trusted_apply_matches_regular_apply_for_legal_candidates() {
        for scenario in scenarios::SCENARIOS {
            let board = scenario.board();
            for mv in candidate_moves(&board, 2)
                .into_iter()
                .filter(|&mv| board.is_legal(mv))
            {
                let mut regular = board.clone();
                let mut trusted = board.clone();

                let regular_result = regular.apply_move(mv).unwrap();
                let trusted_result = trusted.apply_trusted_legal_move(mv);

                assert_eq!(
                    trusted_result, regular_result,
                    "scenario '{}' result diverged for {:?}",
                    scenario.id, mv
                );
                assert_eq!(
                    trusted.to_fen(),
                    regular.to_fen(),
                    "scenario '{}' position diverged for {:?}",
                    scenario.id,
                    mv
                );
                assert_eq!(
                    trusted.result, regular.result,
                    "scenario '{}' game result diverged for {:?}",
                    scenario.id, mv
                );
                assert_eq!(
                    trusted.history, regular.history,
                    "scenario '{}' history diverged for {:?}",
                    scenario.id, mv
                );
            }
        }
    }

    #[test]
    fn optimized_candidates_match_reference_set() {
        for scenario in scenarios::SCENARIOS {
            let board = scenario.board();
            for radius in [1, 2, 3] {
                let mut optimized = candidate_moves(&board, radius);
                let mut reference = candidate_moves_reference(&board, radius);
                optimized.sort_by_key(|mv| (mv.row, mv.col));
                reference.sort_by_key(|mv| (mv.row, mv.col));

                assert_eq!(
                    optimized, reference,
                    "scenario '{}' candidate set diverged for radius {}",
                    scenario.id, radius
                );
            }
        }

        let empty = Board::new(RuleConfig::default());
        assert_eq!(
            candidate_moves(&empty, 2),
            candidate_moves_reference(&empty, 2),
            "empty board center candidate diverged"
        );
    }

    #[test]
    fn asymmetric_candidates_use_current_player_and_opponent_radii() {
        let mut board = Board::new(RuleConfig::default());
        apply_moves(&mut board, &["H8", "L12", "H9"]);
        assert_eq!(board.current_player, Color::White);

        let source = CandidateSource::NearSelfOpponent {
            self_radius: 2,
            opponent_radius: 1,
        };
        let mut metrics = SearchMetrics::default();
        let mut optimized = candidate_moves_from_source_counted(
            &board,
            source,
            &mut metrics,
            SearchMetricPhase::Root,
        );
        let mut reference = candidate_moves_from_source_reference(&board, 2, 1);
        optimized.sort_by_key(|mv| (mv.row, mv.col));
        reference.sort_by_key(|mv| (mv.row, mv.col));

        assert_eq!(optimized, reference);
        assert!(optimized.contains(&mv("J10")), "near white stone at L12");
        assert!(optimized.contains(&mv("G7")), "near black stones at H8/H9");
        assert!(
            !optimized.contains(&mv("F6")),
            "opponent radius 1 should not include radius-2 black frontier"
        );
        assert_eq!(metrics.root_candidate_generations, 1);
        assert_eq!(metrics.root_candidate_moves_total as usize, optimized.len());
    }

    #[test]
    fn optimized_candidates_emit_board_order() {
        for scenario in scenarios::SCENARIOS {
            let board = scenario.board();
            let candidates = candidate_moves(&board, 2);
            assert!(
                candidates
                    .windows(2)
                    .all(|pair| (pair[0].row, pair[0].col) <= (pair[1].row, pair[1].col)),
                "scenario '{}' candidates should use board order",
                scenario.id
            );
        }
    }

    #[test]
    fn candidate_radius_zero_uses_generic_candidate_path() {
        let mut board = Board::new(RuleConfig::default());
        apply_moves(&mut board, &["H8", "A1"]);

        assert_eq!(
            candidate_moves(&board, 0),
            candidate_moves_reference(&board, 0)
        );
    }

    #[test]
    fn default_candidate_masks_cover_nearby_cells_for_default_board() {
        let masks = default_candidate_masks(2);
        let center = mv("H8");
        let center_idx = center.row * masks.size + center.col;
        let center_mask = masks.masks[center_idx];

        assert_eq!(masks.size, 15);
        assert_eq!(masks.words, STACK_SEEN_WORDS);
        assert!(mask_contains(center_mask, mv("F6"), masks.size));
        assert!(mask_contains(center_mask, mv("J10"), masks.size));
        assert!(!mask_contains(center_mask, mv("E5"), masks.size));
    }

    #[test]
    fn tt_first_ordering_moves_hit_without_reordering_other_moves() {
        let moves = vec![mv("A1"), mv("B1"), mv("C1"), mv("D1")];

        assert_eq!(
            order_tt_first(moves.clone(), Some(mv("C1"))),
            vec![mv("C1"), mv("A1"), mv("B1"), mv("D1")]
        );
        assert_eq!(order_tt_first(moves.clone(), Some(mv("H8"))), moves);
    }

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
    fn safety_gate_reply_probe_falls_back_to_unfiltered_moves_when_deadline_has_elapsed() {
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

        let mut metrics = SearchMetrics::default();
        let moves = root_candidate_moves_with_metrics(
            &board,
            CandidateSource::NearAll { radius: 2 },
            LegalityGate::ExactRules,
            SafetyGate::OpponentReplySearchProbe,
            SearchDeadline::new(
                Instant::now() - Duration::from_millis(2),
                Some(Duration::from_millis(1)),
                None,
                None,
            ),
            &mut metrics,
        );

        let (moves, _, timed_out) = moves;
        assert_eq!(moves, expected);
        assert!(timed_out);
    }

    #[test]
    fn safety_gate_none_skips_opponent_reply_probe() {
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

        let mut metrics = SearchMetrics::default();
        let (moves, safety_nodes, timed_out) = root_candidate_moves_with_metrics(
            &board,
            CandidateSource::NearAll { radius: 2 },
            LegalityGate::ExactRules,
            SafetyGate::None,
            SearchDeadline::new(Instant::now(), Some(Duration::from_millis(100)), None, None),
            &mut metrics,
        );

        assert_eq!(moves, expected);
        assert_eq!(safety_nodes, 0);
        assert!(!timed_out);
    }

    #[test]
    fn safety_gate_local_threat_probe_filters_open_three_blunders() {
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

        let mut metrics = SearchMetrics::default();
        let (moves, safety_nodes, timed_out) = root_candidate_moves_with_metrics(
            &board,
            CandidateSource::NearAll { radius: 2 },
            LegalityGate::ExactRules,
            SafetyGate::OpponentReplyLocalThreatProbe,
            SearchDeadline::new(Instant::now(), Some(Duration::from_millis(100)), None, None),
            &mut metrics,
        );

        assert!(moves.contains(&Move { row: 7, col: 6 }));
        assert!(moves.contains(&Move { row: 7, col: 10 }));
        assert!(!moves.contains(&Move { row: 4, col: 4 }));
        assert!(safety_nodes > 0);
        assert!(!timed_out);
    }

    #[test]
    fn tactical_annotation_summarizes_local_threat_replies() {
        let mut board = Board::new(RuleConfig::default());
        apply_moves(&mut board, &["H8", "A1", "I8", "C1", "J8", "E1"]);

        let annotation = annotate_tactical_move(&board, mv("K8"));

        assert_eq!(annotation.player, Color::Black);
        assert_eq!(annotation.mv, mv("K8"));
        assert_eq!(
            annotation.local_threats,
            vec![LocalThreatFact {
                player: Color::Black,
                kind: LocalThreatKind::OpenFour,
                origin: LocalThreatOrigin::AfterMove(mv("K8")),
                defense_squares: vec![mv("G8"), mv("L8")],
                rest_squares: vec![],
            }]
        );
        assert!(annotation.creates_immediate_or_multi_threat());

        let quiet = annotate_tactical_move(&board, mv("B2"));
        assert!(!quiet.creates_immediate_or_multi_threat());
    }

    #[test]
    fn tactical_ordering_prioritizes_win_block_forcing_then_quiet_moves() {
        let mut win_board = Board::new(RuleConfig::default());
        apply_moves(
            &mut win_board,
            &["H8", "A1", "I8", "B1", "J8", "C1", "K8", "D1"],
        );
        let mut metrics = SearchMetrics::default();
        let ordered = order_moves_tactical_first(
            &win_board,
            vec![mv("B2"), mv("E1"), mv("L8")],
            None,
            &mut metrics,
            SearchMetricPhase::Root,
        );

        assert_eq!(
            ordered.iter().map(|ordered| ordered.mv).collect::<Vec<_>>(),
            vec![mv("L8"), mv("E1"), mv("B2")]
        );
        assert_eq!(metrics.root_tactical_annotations, 3);

        let mut shape_board = Board::new(RuleConfig::default());
        apply_moves(
            &mut shape_board,
            &["H8", "A1", "I8", "B1", "J8", "C1", "O15", "D1"],
        );
        let mut metrics = SearchMetrics::default();
        let ordered = order_moves_tactical_first(
            &shape_board,
            vec![mv("B2"), mv("K8"), mv("E1")],
            None,
            &mut metrics,
            SearchMetricPhase::Search,
        );

        assert_eq!(
            ordered.iter().map(|ordered| ordered.mv).collect::<Vec<_>>(),
            vec![mv("E1"), mv("K8"), mv("B2")]
        );
        assert_eq!(metrics.search_tactical_annotations, 3);
    }

    #[test]
    fn child_limit_preserves_must_keep_moves_after_nominal_cap() {
        let ordered = vec![
            OrderedMove {
                mv: mv("B2"),
                must_keep: false,
            },
            OrderedMove {
                mv: mv("C3"),
                must_keep: false,
            },
            OrderedMove {
                mv: mv("L8"),
                must_keep: true,
            },
        ];
        let mut metrics = SearchMetrics::default();

        let capped = apply_child_limit(ordered, Some(1), &mut metrics, SearchMetricPhase::Search);

        assert_eq!(capped, vec![mv("B2"), mv("L8")]);
        assert_eq!(metrics.search_child_cap_hits, 1);
        assert_eq!(metrics.search_child_moves_before_total, 3);
        assert_eq!(metrics.search_child_moves_after_total, 2);
    }

    #[test]
    fn child_limit_filters_renju_legality_before_capping_default_ordering() {
        let mut board = Board::new(RuleConfig {
            variant: Variant::Renju,
            ..Default::default()
        });
        apply_moves(
            &mut board,
            &[
                "A1", "A15", "C1", "C15", "D1", "E15", "E1", "G15", "F1", "I15",
            ],
        );
        assert_eq!(board.current_player, Color::Black);
        assert!(!board.is_legal(mv("B1")));
        assert_eq!(candidate_moves(&board, 2).first().copied(), Some(mv("B1")));

        let zobrist = ZobristTable::new(board.config.board_size);
        let hash = board.hash_with(&zobrist);
        let mut tt = HashMap::new();
        let mut nodes = 0;
        let mut metrics = SearchMetrics::default();
        let deadline = SearchDeadline::new(Instant::now(), None, None, None);

        let outcome = negamax(
            &mut board,
            hash,
            1,
            i32::MIN + 1,
            i32::MAX,
            Color::Black,
            Color::Black,
            &mut tt,
            &zobrist,
            CandidateSource::NearAll { radius: 2 },
            LegalityGate::ExactRules,
            MoveOrdering::TranspositionFirstBoardOrder,
            Some(1),
            CorridorPortalConfig::default(),
            StaticEvaluation::LineShapeEval,
            &mut nodes,
            &mut metrics,
            deadline,
        );

        let best_move = outcome
            .best_move
            .expect("legal moves after the illegal first candidate");
        assert!(board.is_legal(best_move));
        assert_ne!(best_move, mv("B1"));
        assert_eq!(metrics.search_child_cap_hits, 1);
        assert!(metrics.search_legality_checks > 1);
    }

    #[test]
    fn explicit_config_constructors_preserve_legacy_defaults() {
        let baseline = SearchBotConfig::custom_depth(3);
        assert_eq!(SearchBot::new(3).config(), baseline);
        assert_eq!(
            baseline.candidate_source(),
            CandidateSource::NearAll { radius: 2 }
        );
        assert_eq!(baseline.legality_gate(), LegalityGate::ExactRules);
        assert_eq!(
            baseline.safety_gate(),
            SafetyGate::OpponentReplyLocalThreatProbe
        );
        assert_eq!(baseline.corridor_portals, CorridorPortalConfig::default());
        assert_eq!(
            baseline.move_ordering,
            MoveOrdering::TranspositionFirstBoardOrder
        );
        assert_eq!(
            baseline.search_algorithm,
            SearchAlgorithm::AlphaBetaIterativeDeepening
        );
        assert_eq!(baseline.static_eval, StaticEvaluation::LineShapeEval);
        assert_eq!(
            SearchBot::with_time(250).config(),
            SearchBotConfig::custom_time_budget(250)
        );

        let config = SearchBotConfig {
            max_depth: 4,
            time_budget_ms: None,
            cpu_time_budget_ms: None,
            candidate_radius: 3,
            candidate_opponent_radius: None,
            safety_gate: SafetyGate::None,
            move_ordering: MoveOrdering::TranspositionFirstBoardOrder,
            child_limit: None,
            search_algorithm: SearchAlgorithm::AlphaBetaIterativeDeepening,
            static_eval: StaticEvaluation::LineShapeEval,
            corridor_portals: CorridorPortalConfig::default(),
        };
        assert_eq!(SearchBot::with_config(config).config(), config);
        assert_eq!(
            config.candidate_source(),
            CandidateSource::NearAll { radius: 3 }
        );
        assert_eq!(config.safety_gate, SafetyGate::None);

        let asymmetric = SearchBotConfig {
            candidate_radius: 2,
            candidate_opponent_radius: Some(1),
            ..SearchBotConfig::custom_depth(3)
        };
        assert_eq!(
            asymmetric.candidate_source(),
            CandidateSource::NearSelfOpponent {
                self_radius: 2,
                opponent_radius: 1
            }
        );
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
        let board = Board::new(RuleConfig {
            variant: Variant::Renju,
            ..Default::default()
        });
        let mut bot = SearchBot::with_config(SearchBotConfig::custom_depth(3));

        let _ = bot.choose_move(&board);
        let trace = bot.trace().expect("expected search trace");

        assert_eq!(trace["config"]["max_depth"], 3);
        assert_eq!(trace["config"]["candidate_radius"], 2);
        assert_eq!(trace["config"]["candidate_source"], "near_all_r2");
        assert_eq!(trace["config"]["legality_gate"], "exact_rules");
        assert_eq!(
            trace["config"]["safety_gate"],
            "opponent_reply_local_threat_probe"
        );
        assert_eq!(trace["config"]["move_ordering"], "tt_first_board_order");
        assert_eq!(trace["config"]["child_limit"], serde_json::Value::Null);
        assert_eq!(trace["config"]["search_algorithm"], "alpha_beta_id");
        assert_eq!(trace["config"]["static_eval"], "line_shape_eval");
        assert_eq!(
            trace["config"]["corridor_portals"],
            serde_json::json!({
                "own": {
                    "enabled": false,
                    "max_depth": 0,
                    "max_reply_width": 0,
                },
                "opponent": {
                    "enabled": false,
                    "max_depth": 0,
                    "max_reply_width": 0,
                },
            })
        );
        assert!(trace["nodes"].as_u64().unwrap() > 0);
        assert!(trace["total_nodes"].as_u64().unwrap() >= trace["nodes"].as_u64().unwrap());
        assert_eq!(trace["budget_exhausted"], false);
        assert_eq!(trace["depth"], 3);

        let metrics = &trace["metrics"];
        assert!(metrics["eval_calls"].as_u64().unwrap() > 0);
        assert!(metrics["candidate_generations"].as_u64().unwrap() > 0);
        assert!(
            metrics["candidate_moves_total"].as_u64().unwrap()
                >= metrics["candidate_moves_max"].as_u64().unwrap()
        );
        assert_eq!(
            metrics["candidate_generations"].as_u64().unwrap(),
            metrics["root_candidate_generations"].as_u64().unwrap()
                + metrics["search_candidate_generations"].as_u64().unwrap()
        );
        assert_eq!(
            metrics["candidate_moves_total"].as_u64().unwrap(),
            metrics["root_candidate_moves_total"].as_u64().unwrap()
                + metrics["search_candidate_moves_total"].as_u64().unwrap()
        );
        assert!(metrics["legality_checks"].as_u64().unwrap() > 0);
        assert_eq!(
            metrics["legality_checks"].as_u64().unwrap(),
            metrics["root_legality_checks"].as_u64().unwrap()
                + metrics["search_legality_checks"].as_u64().unwrap()
        );
        assert!(metrics["tt_hits"].as_u64().is_some());
        assert!(metrics["tt_cutoffs"].as_u64().is_some());
        assert!(metrics["beta_cutoffs"].as_u64().is_some());
        assert!(
            metrics["tactical_annotations"].as_u64().unwrap()
                >= metrics["root_tactical_annotations"].as_u64().unwrap()
        );
        assert_eq!(
            metrics["tactical_annotations"].as_u64().unwrap(),
            metrics["root_tactical_annotations"].as_u64().unwrap()
                + metrics["search_tactical_annotations"].as_u64().unwrap()
        );
        assert_eq!(metrics["corridor_entry_checks"], 0);
        assert_eq!(metrics["corridor_nodes"], 0);
    }

    #[test]
    fn trace_records_corridor_portal_config() {
        let board = Board::new(RuleConfig::default());
        let mut config = SearchBotConfig::custom_depth(1);
        config.corridor_portals.own = CorridorPortalSideConfig {
            enabled: true,
            max_depth: 4,
            max_reply_width: 3,
        };
        config.corridor_portals.opponent = CorridorPortalSideConfig {
            enabled: true,
            max_depth: 2,
            max_reply_width: 2,
        };
        let mut bot = SearchBot::with_config(config);

        let _ = bot.choose_move(&board);
        let trace = bot.trace().expect("expected search trace");

        assert_eq!(trace["config"]["corridor_portals"]["own"]["enabled"], true);
        assert_eq!(trace["config"]["corridor_portals"]["own"]["max_depth"], 4);
        assert_eq!(
            trace["config"]["corridor_portals"]["own"]["max_reply_width"],
            3
        );
        assert_eq!(
            trace["config"]["corridor_portals"]["opponent"]["enabled"],
            true
        );
        assert_eq!(
            trace["config"]["corridor_portals"]["opponent"]["max_depth"],
            2
        );
        assert_eq!(
            trace["config"]["corridor_portals"]["opponent"]["max_reply_width"],
            2
        );
        assert_eq!(trace["corridor"]["search_nodes"], 0);
        assert_eq!(trace["corridor"]["extra_plies"], 0);
        assert_eq!(trace["corridor_extra_plies"], 0);
        assert_eq!(trace["effective_depth"], trace["depth"]);
    }

    #[test]
    fn corridor_portal_activates_on_root_corridor_entry() {
        let mut board = Board::new(RuleConfig::default());
        apply_moves(
            &mut board,
            &["H8", "A1", "I8", "A2", "J8", "A3", "K8", "A4"],
        );
        assert_eq!(board.current_player, Color::Black);

        let mut config = SearchBotConfig::custom_depth(1);
        config.safety_gate = SafetyGate::None;
        config.corridor_portals.own = CorridorPortalSideConfig {
            enabled: true,
            max_depth: 4,
            max_reply_width: 3,
        };
        let mut bot = SearchBot::with_config(config);

        let _ = bot.choose_move(&board);
        let trace = bot.trace().expect("expected search trace");
        let metrics = &trace["metrics"];

        assert!(metrics["corridor_entry_checks"].as_u64().unwrap() > 0);
        assert!(metrics["corridor_entries_accepted"].as_u64().unwrap() > 0);
        assert!(metrics["corridor_own_entries_accepted"].as_u64().unwrap() > 0);
        assert!(trace["corridor"]["search_nodes"].as_u64().unwrap() > 0);
        assert!(trace["corridor"]["terminal_exits"].as_u64().unwrap() > 0);
    }

    #[test]
    fn corridor_portal_tracks_opponent_side_entries_below_root() {
        let mut board = Board::new(RuleConfig::default());
        apply_moves(&mut board, &["A1", "H8", "A2", "I8", "A3", "J8"]);
        assert_eq!(board.current_player, Color::Black);

        let mut config = SearchBotConfig::custom_depth(2);
        config.safety_gate = SafetyGate::None;
        config.corridor_portals.opponent = CorridorPortalSideConfig {
            enabled: true,
            max_depth: 2,
            max_reply_width: 3,
        };
        let mut bot = SearchBot::with_config(config);

        let _ = bot.choose_move(&board);
        let trace = bot.trace().expect("expected search trace");
        let metrics = &trace["metrics"];

        assert!(metrics["corridor_entry_checks"].as_u64().unwrap() > 0);
        assert!(
            metrics["corridor_opponent_entries_accepted"]
                .as_u64()
                .unwrap()
                > 0
        );
        assert_eq!(metrics["corridor_own_entries_accepted"], 0);
    }

    #[test]
    fn resumed_search_after_corridor_does_not_reenter_portals() {
        let mut board = Board::new(RuleConfig::default());
        apply_moves(&mut board, &["H8", "A1", "I8", "A2", "J8", "A3"]);
        assert_eq!(board.current_player, Color::Black);

        let mut portal_config = CorridorPortalConfig::default();
        portal_config.own = CorridorPortalSideConfig {
            enabled: true,
            max_depth: 2,
            max_reply_width: 3,
        };
        let mut tt = HashMap::new();
        let zobrist = ZobristTable::new(board.config.board_size);
        let hash = board.hash_with(&zobrist);
        let mut nodes = 0u64;
        let mut metrics = SearchMetrics::default();

        let _ = resume_normal_search_after_corridor(
            &mut board,
            hash,
            1,
            i32::MIN + 1,
            i32::MAX,
            Color::Black,
            Color::Black,
            &mut tt,
            &zobrist,
            CandidateSource::NearAll { radius: 2 },
            LegalityGate::ExactRules,
            MoveOrdering::TranspositionFirstBoardOrder,
            None,
            portal_config,
            StaticEvaluation::LineShapeEval,
            &mut nodes,
            &mut metrics,
            SearchDeadline::new(Instant::now(), Some(Duration::from_millis(100)), None, None),
        );

        assert_eq!(metrics.corridor_resume_searches, 1);
        assert_eq!(
            metrics.corridor_entries_accepted, 0,
            "resuming normal search from a corridor exit should not immediately re-enter portals"
        );
    }

    #[test]
    fn resumed_search_after_corridor_ignores_shared_transposition_table() {
        let mut board = Board::new(RuleConfig::default());
        apply_moves(&mut board, &["H8", "A1", "I8", "A2", "J8", "A3"]);
        assert_eq!(board.current_player, Color::Black);

        let zobrist = ZobristTable::new(board.config.board_size);
        let hash = board.hash_with(&zobrist);
        let poisoned_score = 1_234_567;
        let mut tt = HashMap::from([(
            hash,
            TTEntry {
                depth: 1,
                score: poisoned_score,
                flag: TTFlag::Exact,
                best_move: Some(mv("H9")),
            },
        )]);
        let mut nodes = 0u64;
        let mut metrics = SearchMetrics::default();

        let outcome = resume_normal_search_after_corridor(
            &mut board,
            hash,
            1,
            i32::MIN + 1,
            i32::MAX,
            Color::Black,
            Color::Black,
            &mut tt,
            &zobrist,
            CandidateSource::NearAll { radius: 2 },
            LegalityGate::ExactRules,
            MoveOrdering::TranspositionFirstBoardOrder,
            None,
            CorridorPortalConfig::default(),
            StaticEvaluation::LineShapeEval,
            &mut nodes,
            &mut metrics,
            SearchDeadline::new(Instant::now(), Some(Duration::from_millis(100)), None, None),
        );

        assert_ne!(
            outcome.score, poisoned_score,
            "resumed corridor searches must not reuse entries from the parent portal-enabled table"
        );
        assert_eq!(metrics.tt_hits, 0);
        assert_eq!(metrics.tt_cutoffs, 0);
        assert_eq!(
            tt.len(),
            1,
            "resume search should not write to the shared table"
        );
        assert_eq!(tt.get(&hash).map(|entry| entry.score), Some(poisoned_score));
    }

    #[test]
    fn trace_records_pattern_static_eval() {
        let board = Board::new(RuleConfig::default());
        let mut config = SearchBotConfig::custom_depth(1);
        config.static_eval = StaticEvaluation::PatternEval;
        let mut bot = SearchBot::with_config(config);

        let _ = bot.choose_move(&board);
        let trace = bot.trace().expect("expected search trace");

        assert_eq!(trace["config"]["static_eval"], "pattern_eval");
    }

    #[test]
    fn pipeline_bench_static_eval_supports_pattern_eval() {
        let board = Board::new(RuleConfig::default());

        assert_eq!(
            pipeline_bench_evaluate_static(&board, Color::Black, StaticEvaluation::PatternEval),
            evaluate_static(&board, Color::Black, StaticEvaluation::PatternEval)
        );
    }

    #[test]
    fn pattern_eval_downgrades_renju_forbidden_overline_completion() {
        let mut board = Board::new(RuleConfig {
            variant: Variant::Renju,
            ..Default::default()
        });
        apply_moves(
            &mut board,
            &[
                "A1", "G1", "C1", "A15", "D1", "C15", "E1", "E15", "F1", "G15",
            ],
        );

        assert_eq!(board.current_player, Color::Black);
        assert!(!board.is_legal(mv("B1")));

        let line_score = evaluate_static(&board, Color::Black, StaticEvaluation::LineShapeEval);
        let pattern_score = evaluate_static(&board, Color::Black, StaticEvaluation::PatternEval);

        assert!(
            pattern_score < line_score,
            "expected pattern eval to discount forbidden completion: line={line_score}, pattern={pattern_score}"
        );
    }

    #[test]
    fn trace_records_tactical_ordering_metrics() {
        let mut board = Board::new(RuleConfig::default());
        apply_moves(
            &mut board,
            &["H8", "A1", "I8", "B1", "J8", "C1", "O15", "D1"],
        );
        let mut config = SearchBotConfig::custom_depth(2);
        config.safety_gate = SafetyGate::None;
        config.move_ordering = MoveOrdering::TacticalFirst;
        let mut bot = SearchBot::with_config(config);

        let _ = bot.choose_move(&board);
        let trace = bot.trace().expect("expected search trace");
        let metrics = &trace["metrics"];

        assert_eq!(trace["config"]["move_ordering"], "tactical_first");
        assert!(metrics["root_tactical_annotations"].as_u64().unwrap() > 0);
        assert!(metrics["search_tactical_annotations"].as_u64().unwrap() > 0);
    }

    #[test]
    fn trace_records_child_limit_metrics() {
        let mut board = Board::new(RuleConfig::default());
        apply_moves(
            &mut board,
            &["H8", "A1", "I8", "B1", "J8", "C1", "O15", "D1"],
        );
        let mut config = SearchBotConfig::custom_depth(2);
        config.safety_gate = SafetyGate::None;
        config.move_ordering = MoveOrdering::TacticalFirst;
        config.child_limit = Some(4);
        let mut bot = SearchBot::with_config(config);

        let _ = bot.choose_move(&board);
        let trace = bot.trace().expect("expected search trace");
        let metrics = &trace["metrics"];

        assert_eq!(trace["config"]["child_limit"], 4);
        assert!(metrics["child_cap_hits"].as_u64().unwrap() > 0);
        assert_eq!(metrics["root_child_cap_hits"], 0);
        assert_eq!(metrics["root_child_moves_before_total"], 0);
        assert_eq!(metrics["root_child_moves_after_total"], 0);
        assert!(metrics["search_child_cap_hits"].as_u64().unwrap() > 0);
        assert!(
            metrics["search_child_moves_before_total"].as_u64().unwrap()
                > metrics["search_child_moves_after_total"].as_u64().unwrap()
        );
    }

    #[test]
    fn root_legality_filter_does_not_count_as_search_work() {
        let mut board = Board::new(RuleConfig {
            variant: Variant::Renju,
            ..Default::default()
        });
        apply_moves(&mut board, &["H8", "A1"]);

        let config = SearchBotConfig {
            max_depth: 1,
            time_budget_ms: None,
            cpu_time_budget_ms: None,
            candidate_radius: 2,
            candidate_opponent_radius: None,
            safety_gate: SafetyGate::None,
            move_ordering: MoveOrdering::TranspositionFirstBoardOrder,
            child_limit: None,
            search_algorithm: SearchAlgorithm::AlphaBetaIterativeDeepening,
            static_eval: StaticEvaluation::LineShapeEval,
            corridor_portals: CorridorPortalConfig::default(),
        };
        let mut bot = SearchBot::with_config(config);

        let chosen = bot.choose_move(&board);
        let info = bot
            .last_info
            .as_ref()
            .expect("expected search info after choose_move");

        assert!(board.is_legal(chosen));
        assert!(info.metrics.root_legality_checks > 0);
        assert_eq!(info.metrics.search_legality_checks, 0);
    }

    #[test]
    fn tactical_analyzer_identifies_immediate_win_and_block() {
        let mut board = Board::new(RuleConfig::default());
        for i in 0..4usize {
            board.apply_move(Move { row: 7, col: 7 + i }).unwrap();
            board.apply_move(Move { row: 0, col: i }).unwrap();
        }

        let winning = analyze_tactical_move(&board, Move { row: 7, col: 11 });
        assert!(winning.is_legal);
        assert!(winning.immediate_win);
        assert!(!winning.immediate_block);

        let mut board = Board::new(RuleConfig::default());
        board.apply_move(Move { row: 7, col: 7 }).unwrap();
        for i in 0..4usize {
            board.apply_move(Move { row: 0, col: i }).unwrap();
            if i < 3 {
                board.apply_move(Move { row: 14, col: i }).unwrap();
            }
        }

        let blocking = analyze_tactical_move(&board, Move { row: 0, col: 4 });
        assert!(blocking.is_legal);
        assert!(!blocking.immediate_win);
        assert!(blocking.immediate_block);
    }

    #[test]
    fn tactical_analyzer_labels_open_and_closed_fours() {
        let mut board = Board::new(RuleConfig::default());
        for mv in [
            Move { row: 7, col: 7 },
            Move { row: 0, col: 0 },
            Move { row: 7, col: 8 },
            Move { row: 0, col: 1 },
            Move { row: 7, col: 9 },
            Move { row: 0, col: 2 },
        ] {
            board.apply_move(mv).unwrap();
        }

        let open_four = analyze_tactical_move(&board, Move { row: 7, col: 10 });
        assert!(open_four.open_four);
        assert!(!open_four.closed_four);

        let mut board = Board::new(RuleConfig::default());
        for mv in [
            Move { row: 7, col: 7 },
            Move { row: 7, col: 6 },
            Move { row: 7, col: 8 },
            Move { row: 0, col: 0 },
            Move { row: 7, col: 9 },
            Move { row: 0, col: 1 },
        ] {
            board.apply_move(mv).unwrap();
        }

        let closed_four = analyze_tactical_move(&board, Move { row: 7, col: 10 });
        assert!(!closed_four.open_four);
        assert!(closed_four.closed_four);
    }

    #[test]
    fn tactical_analyzer_labels_three_shapes_and_double_threats() {
        let mut board = Board::new(RuleConfig::default());
        for mv in [
            Move { row: 7, col: 7 },
            Move { row: 0, col: 0 },
            Move { row: 7, col: 8 },
            Move { row: 0, col: 1 },
        ] {
            board.apply_move(mv).unwrap();
        }

        let open_three = analyze_tactical_move(&board, Move { row: 7, col: 9 });
        assert!(open_three.open_three);
        assert!(!open_three.broken_three);

        let mut boxed_three_board = Board::new(RuleConfig::default());
        apply_moves(&mut boxed_three_board, &["J9", "H9", "K9", "N9"]);

        let boxed_three = analyze_tactical_move(&boxed_three_board, mv("L9"));
        assert!(!boxed_three.open_three);

        let mut board = Board::new(RuleConfig::default());
        for mv in [
            Move { row: 7, col: 7 },
            Move { row: 0, col: 0 },
            Move { row: 7, col: 10 },
            Move { row: 0, col: 1 },
        ] {
            board.apply_move(mv).unwrap();
        }

        let broken_three = analyze_tactical_move(&board, Move { row: 7, col: 9 });
        assert!(!broken_three.open_three);
        assert!(broken_three.broken_three);

        let mut board = Board::new(RuleConfig::default());
        for mv in [
            Move { row: 7, col: 6 },
            Move { row: 0, col: 0 },
            Move { row: 7, col: 7 },
            Move { row: 0, col: 2 },
            Move { row: 7, col: 8 },
            Move { row: 0, col: 4 },
            Move { row: 6, col: 9 },
            Move { row: 2, col: 0 },
            Move { row: 8, col: 9 },
            Move { row: 2, col: 2 },
            Move { row: 9, col: 9 },
            Move { row: 2, col: 4 },
        ] {
            board.apply_move(mv).unwrap();
        }

        let fork = analyze_tactical_move(&board, Move { row: 7, col: 9 });
        assert!(fork.double_threat);

        let filler = analyze_tactical_move(&board, Move { row: 1, col: 1 });
        assert!(!filler.double_threat);
    }

    #[test]
    fn forced_line_classifier_prioritizes_current_immediate_win() {
        let scenario = scenarios::SCENARIOS
            .iter()
            .find(|scenario| scenario.id == "priority_complete_open_four_over_react_closed_four")
            .expect("expected priority complete-over-react scenario");
        let board = scenario.board();

        let state = classify_forced_line_state(&board);

        assert_eq!(state.player, Color::Black);
        assert_eq!(state.kind, ForcedLineKind::ImmediateWin);
        assert!(state.immediate_wins.contains(&mv("G8")));
        assert!(state.opponent_wins.contains(&mv("E1")));
        assert!(state.legal_blocks.contains(&mv("E1")));
        assert_eq!(state.forced_block(), None);
    }

    #[test]
    fn forced_line_classifier_identifies_single_forced_block() {
        let scenario = scenarios::SCENARIOS
            .iter()
            .find(|scenario| scenario.id == "local_react_closed_four")
            .expect("expected local react closed four scenario");
        let board = scenario.board();

        let state = classify_forced_line_state(&board);

        assert_eq!(state.kind, ForcedLineKind::ForcedBlock);
        assert!(state.immediate_wins.is_empty());
        assert_eq!(state.legal_blocks, vec![mv("E1")]);
        assert_eq!(state.forced_block(), Some(mv("E1")));
    }

    #[test]
    fn forced_line_classifier_does_not_force_illegal_renju_block() {
        let mut board = Board::new(RuleConfig {
            variant: Variant::Renju,
            ..Default::default()
        });
        apply_moves(
            &mut board,
            &["C3", "O15", "H6", "D4", "H7", "E5", "F8", "F6", "G8", "G7"],
        );

        assert_eq!(board.current_player, Color::Black);
        assert_eq!(
            board.immediate_winning_moves_for(Color::White),
            vec![mv("H8")]
        );
        assert!(!board.is_legal(mv("H8")));

        let state = classify_forced_line_state(&board);

        assert_eq!(state.kind, ForcedLineKind::UnblockableImmediateLoss);
        assert_eq!(state.opponent_wins, vec![mv("H8")]);
        assert!(state.legal_blocks.is_empty());
        assert_eq!(state.forced_block(), None);
    }

    #[test]
    fn forced_line_classifier_identifies_opponent_multi_threat() {
        let mut board = Board::new(RuleConfig::default());
        apply_moves(
            &mut board,
            &["O15", "H1", "M15", "I1", "K15", "J1", "I15", "K1"],
        );

        let state = classify_forced_line_state(&board);

        assert_eq!(state.player, Color::Black);
        assert_eq!(state.kind, ForcedLineKind::OpponentMultiThreat);
        assert!(state.immediate_wins.is_empty());
        assert!(state.opponent_wins.contains(&mv("G1")));
        assert!(state.opponent_wins.contains(&mv("L1")));
        assert_eq!(state.forced_block(), None);
    }

    #[test]
    fn threat_after_move_classifier_labels_win_threats_and_illegal_moves() {
        let scenario = scenarios::SCENARIOS
            .iter()
            .find(|scenario| scenario.id == "local_complete_open_four")
            .expect("expected local complete open four scenario");
        let board = scenario.board();

        let winning = classify_threat_after_move(&board, mv("G8"));
        assert_eq!(winning.kind, ThreatAfterMoveKind::WinsNow);
        assert!(winning.winning_replies.is_empty());

        let illegal = classify_threat_after_move(&board, mv("H8"));
        assert_eq!(illegal.kind, ThreatAfterMoveKind::Illegal);
        assert!(illegal.winning_replies.is_empty());

        let mut closed_four_board = Board::new(RuleConfig::default());
        apply_moves(
            &mut closed_four_board,
            &["H8", "G8", "I8", "A1", "J8", "C1"],
        );
        let single = classify_threat_after_move(&closed_four_board, mv("K8"));
        assert_eq!(single.kind, ThreatAfterMoveKind::SingleThreat);
        assert_eq!(single.winning_replies, vec![mv("L8")]);

        let mut open_four_board = Board::new(RuleConfig::default());
        apply_moves(&mut open_four_board, &["H8", "A1", "I8", "C1", "J8", "E1"]);
        let multi = classify_threat_after_move(&open_four_board, mv("K8"));
        assert_eq!(multi.kind, ThreatAfterMoveKind::MultiThreat);
        assert!(multi.winning_replies.contains(&mv("G8")));
        assert!(multi.winning_replies.contains(&mv("L8")));

        let quiet = classify_threat_after_move(&open_four_board, mv("B2"));
        assert_eq!(quiet.kind, ThreatAfterMoveKind::Quiet);
        assert!(quiet.winning_replies.is_empty());
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
            .find(|scenario| scenario.id == "local_complete_open_four")
            .expect("expected local complete open four benchmark scenario");
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
            .find(|scenario| scenario.id == "local_react_closed_four")
            .expect("expected local react closed four benchmark scenario");
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
