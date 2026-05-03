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

    for &(dr, dc) in &DIRS {
        for row in 0..size as isize {
            for col in 0..size as isize {
                let Some(player) = board.cell(row as usize, col as usize) else {
                    continue;
                };

                // Only score a contiguous run once, from its back end.
                let pr = row - dr;
                let pc = col - dc;
                if pr >= 0
                    && pr < size as isize
                    && pc >= 0
                    && pc < size as isize
                    && board.cell(pr as usize, pc as usize) == Some(player)
                {
                    continue;
                }

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
                    return if player == color {
                        2_000_000
                    } else {
                        -2_000_000
                    };
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
                    && board.cell(br as usize, bc as usize).is_none()
                {
                    ends += 1;
                }
                if r >= 0
                    && r < size as isize
                    && c >= 0
                    && c < size as isize
                    && board.cell(r as usize, c as usize).is_none()
                {
                    ends += 1;
                }
                if ends > 0 {
                    let score_idx = if player == color { 0 } else { 1 };
                    let len_idx = len.min(5) as usize;
                    counts[score_idx][len_idx] += 1;
                    open_ends[score_idx][len_idx] += ends;
                }
            }
        }
    }

    score_line(&counts[0], &open_ends[0]) - score_line(&counts[1], &open_ends[1])
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
                    if back_in_bounds && board.cell(pr as usize, pc as usize) == Some(player) {
                        continue;
                    }
                    if board.cell(row as usize, col as usize) != Some(player) {
                        continue;
                    }

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

                    let mut ends = 0i32;
                    let (br, bc) = (row - dr, col - dc);
                    if br >= 0
                        && br < size as isize
                        && bc >= 0
                        && bc < size as isize
                        && board.cell(br as usize, bc as usize).is_none()
                    {
                        ends += 1;
                    }
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
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
    pub tt_hits: u64,
    pub tt_cutoffs: u64,
    pub beta_cutoffs: u64,
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

    fn trace(self) -> serde_json::Value {
        serde_json::json!({
            "eval_calls": self.eval_calls,
            "candidate_generations": self.candidate_generations,
            "candidate_moves_total": self.candidate_moves_total,
            "candidate_moves_max": self.candidate_moves_max,
            "root_candidate_generations": self.root_candidate_generations,
            "root_candidate_moves_total": self.root_candidate_moves_total,
            "root_candidate_moves_max": self.root_candidate_moves_max,
            "search_candidate_generations": self.search_candidate_generations,
            "search_candidate_moves_total": self.search_candidate_moves_total,
            "search_candidate_moves_max": self.search_candidate_moves_max,
            "legality_checks": self.legality_checks,
            "illegal_moves_skipped": self.illegal_moves_skipped,
            "root_legality_checks": self.root_legality_checks,
            "root_illegal_moves_skipped": self.root_illegal_moves_skipped,
            "search_legality_checks": self.search_legality_checks,
            "search_illegal_moves_skipped": self.search_illegal_moves_skipped,
            "tt_hits": self.tt_hits,
            "tt_cutoffs": self.tt_cutoffs,
            "beta_cutoffs": self.beta_cutoffs,
        })
    }
}

fn evaluate_counted(board: &Board, color: Color, metrics: &mut SearchMetrics) -> i32 {
    metrics.eval_calls += 1;
    evaluate(board, color)
}

#[doc(hidden)]
pub fn pipeline_bench_evaluate(board: &Board, color: Color) -> i32 {
    evaluate(board, color)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
struct TacticalMoveFeatures {
    is_legal: bool,
    immediate_win: bool,
    immediate_block: bool,
    open_four: bool,
    blocked_four: bool,
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
    let mut after = board.clone();
    after.apply_move(mv).unwrap();
    let shape = analyze_shapes_through_move(&after, mv, player);
    let immediate_wins_after = after.immediate_winning_moves_for(player).len();

    TacticalMoveFeatures {
        is_legal,
        immediate_win: board.immediate_winning_moves_for(player).contains(&mv),
        immediate_block: board.immediate_winning_moves_for(opponent).contains(&mv),
        open_four: shape.open_four,
        blocked_four: shape.blocked_four,
        open_three: shape.open_three,
        broken_three: shape.broken_three,
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

#[derive(Debug, Clone, Copy, Default)]
struct TacticalShapeFeatures {
    open_four: bool,
    blocked_four: bool,
    open_three: bool,
    broken_three: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
enum LocalThreatKind {
    Five,
    OpenFour,
    SimpleFour,
    OpenThree,
    BrokenThree,
}

#[cfg_attr(not(test), allow(dead_code))]
impl LocalThreatKind {
    fn rank(self) -> u8 {
        match self {
            LocalThreatKind::Five => 5,
            LocalThreatKind::OpenFour => 4,
            LocalThreatKind::SimpleFour => 3,
            LocalThreatKind::OpenThree => 2,
            LocalThreatKind::BrokenThree => 1,
        }
    }

    fn is_forcing(self) -> bool {
        !matches!(self, LocalThreatKind::BrokenThree)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
struct LocalThreatFact {
    player: Color,
    kind: LocalThreatKind,
    gain_square: Move,
    defense_squares: Vec<Move>,
    rest_squares: Vec<Move>,
}

#[cfg_attr(not(test), allow(dead_code))]
impl LocalThreatFact {
    fn is_forcing(&self) -> bool {
        self.kind.is_forcing()
    }
}

#[cfg_attr(not(test), allow(dead_code))]
fn local_threat_facts_after_move(board: &Board, mv: Move) -> Vec<LocalThreatFact> {
    let player = board.current_player;
    if !board.is_legal(mv) {
        return Vec::new();
    }

    let mut after = board.clone();
    if after.apply_move(mv).is_err() {
        return Vec::new();
    }

    let mut facts = DIRS
        .iter()
        .filter_map(|&(dr, dc)| local_threat_fact_in_direction(&after, mv, player, dr, dc))
        .collect::<Vec<_>>();
    facts.sort_by_key(|fact| std::cmp::Reverse(fact.kind.rank()));
    facts
}

#[cfg_attr(not(test), allow(dead_code))]
fn local_threat_fact_in_direction(
    board: &Board,
    mv: Move,
    player: Color,
    dr: isize,
    dc: isize,
) -> Option<LocalThreatFact> {
    let before = count_player_in_direction(board, mv, -dr, -dc, player);
    let after = count_player_in_direction(board, mv, dr, dc, player);
    let run_len = before + 1 + after;

    if run_len >= board.config.win_length {
        return Some(LocalThreatFact {
            player,
            kind: LocalThreatKind::Five,
            gain_square: mv,
            defense_squares: Vec::new(),
            rest_squares: Vec::new(),
        });
    }

    let four_completion_squares = four_completion_squares_through_move(board, mv, dr, dc, player);
    match four_completion_squares.len() {
        2.. => {
            return Some(LocalThreatFact {
                player,
                kind: LocalThreatKind::OpenFour,
                gain_square: mv,
                defense_squares: four_completion_squares,
                rest_squares: Vec::new(),
            });
        }
        1 => {
            return Some(LocalThreatFact {
                player,
                kind: LocalThreatKind::SimpleFour,
                gain_square: mv,
                defense_squares: four_completion_squares,
                rest_squares: Vec::new(),
            });
        }
        0 => {}
    }

    let mut open_ends = Vec::new();
    if let Some(open_before) = empty_offset_move(board, mv, -dr, -dc, before + 1) {
        open_ends.push(open_before);
    }
    if let Some(open_after) = empty_offset_move(board, mv, dr, dc, after + 1) {
        open_ends.push(open_after);
    }

    match (run_len, open_ends.len()) {
        (3, 2) => Some(LocalThreatFact {
            player,
            kind: LocalThreatKind::OpenThree,
            gain_square: mv,
            defense_squares: open_ends,
            rest_squares: Vec::new(),
        }),
        _ => {
            let rest_squares = broken_three_rest_squares_through_move(board, mv, dr, dc, player);
            if rest_squares.is_empty() {
                None
            } else {
                Some(LocalThreatFact {
                    player,
                    kind: LocalThreatKind::BrokenThree,
                    gain_square: mv,
                    defense_squares: Vec::new(),
                    rest_squares,
                })
            }
        }
    }
}

#[cfg_attr(not(test), allow(dead_code))]
fn four_completion_squares_through_move(
    board: &Board,
    mv: Move,
    dr: isize,
    dc: isize,
    player: Color,
) -> Vec<Move> {
    let win_len = board.config.win_length as isize;
    let mut completions = Vec::new();

    for start in -(win_len - 1)..=0 {
        let mut player_count = 0usize;
        let mut empty_square = None;
        let mut blocked = false;

        for offset in start..start + win_len {
            let row = mv.row as isize + dr * offset;
            let col = mv.col as isize + dc * offset;
            if !in_bounds(board, row, col) {
                blocked = true;
                break;
            }

            match board.cell(row as usize, col as usize) {
                Some(color) if color == player => player_count += 1,
                None if empty_square.is_none() => {
                    empty_square = Some(Move {
                        row: row as usize,
                        col: col as usize,
                    });
                }
                None => {
                    blocked = true;
                    break;
                }
                _ => {
                    blocked = true;
                    break;
                }
            }
        }

        let Some(empty_square) = empty_square else {
            continue;
        };
        if !blocked
            && player_count == board.config.win_length.saturating_sub(1)
            && !completions.contains(&empty_square)
        {
            completions.push(empty_square);
        }
    }

    completions.sort_by_key(|mv| (mv.row, mv.col));
    completions
}

fn analyze_shapes_through_move(board: &Board, mv: Move, player: Color) -> TacticalShapeFeatures {
    let mut features = TacticalShapeFeatures::default();
    for &(dr, dc) in &DIRS {
        let (run_len, open_ends) = contiguous_run_through_move(board, mv, dr, dc, player);
        if run_len == 4 && open_ends == 2 {
            features.open_four = true;
        } else if run_len == 4 && open_ends == 1 {
            features.blocked_four = true;
        } else if run_len == 3 && open_ends == 2 {
            features.open_three = true;
        }

        if is_broken_three_through_move(board, mv, dr, dc, player) {
            features.broken_three = true;
        }
    }
    features
}

fn contiguous_run_through_move(
    board: &Board,
    mv: Move,
    dr: isize,
    dc: isize,
    player: Color,
) -> (usize, usize) {
    let before = count_player_in_direction(board, mv, -dr, -dc, player);
    let after = count_player_in_direction(board, mv, dr, dc, player);
    let open_before = offset_cell_is_empty(board, mv, -dr, -dc, before + 1);
    let open_after = offset_cell_is_empty(board, mv, dr, dc, after + 1);

    (
        before + 1 + after,
        usize::from(open_before) + usize::from(open_after),
    )
}

fn count_player_in_direction(
    board: &Board,
    mv: Move,
    dr: isize,
    dc: isize,
    player: Color,
) -> usize {
    let mut count = 0usize;
    let mut row = mv.row as isize + dr;
    let mut col = mv.col as isize + dc;
    while in_bounds(board, row, col) && board.cell(row as usize, col as usize) == Some(player) {
        count += 1;
        row += dr;
        col += dc;
    }
    count
}

fn offset_cell_is_empty(board: &Board, mv: Move, dr: isize, dc: isize, distance: usize) -> bool {
    let row = mv.row as isize + dr * distance as isize;
    let col = mv.col as isize + dc * distance as isize;
    in_bounds(board, row, col) && board.cell(row as usize, col as usize).is_none()
}

#[cfg_attr(not(test), allow(dead_code))]
fn empty_offset_move(
    board: &Board,
    mv: Move,
    dr: isize,
    dc: isize,
    distance: usize,
) -> Option<Move> {
    let row = mv.row as isize + dr * distance as isize;
    let col = mv.col as isize + dc * distance as isize;
    if in_bounds(board, row, col) && board.cell(row as usize, col as usize).is_none() {
        Some(Move {
            row: row as usize,
            col: col as usize,
        })
    } else {
        None
    }
}

fn in_bounds(board: &Board, row: isize, col: isize) -> bool {
    let size = board.config.board_size as isize;
    row >= 0 && row < size && col >= 0 && col < size
}

fn is_broken_three_through_move(
    board: &Board,
    mv: Move,
    dr: isize,
    dc: isize,
    player: Color,
) -> bool {
    !broken_three_rest_squares_through_move(board, mv, dr, dc, player).is_empty()
}

#[cfg_attr(not(test), allow(dead_code))]
fn broken_three_rest_squares_through_move(
    board: &Board,
    mv: Move,
    dr: isize,
    dc: isize,
    player: Color,
) -> Vec<Move> {
    let mut rest_squares = Vec::new();

    for start in -4isize..=0 {
        let mut player_offsets = Vec::new();
        let mut empty_offsets = Vec::new();
        let mut blocked = false;

        for offset in start..start + 5 {
            let row = mv.row as isize + dr * offset;
            let col = mv.col as isize + dc * offset;
            if !in_bounds(board, row, col) {
                blocked = true;
                break;
            }

            match board.cell(row as usize, col as usize) {
                Some(color) if color == player => player_offsets.push(offset),
                None => empty_offsets.push(offset),
                _ => {
                    blocked = true;
                    break;
                }
            }
        }

        if blocked || player_offsets.len() != 3 || empty_offsets.len() != 2 {
            continue;
        }
        if player_offsets.windows(2).all(|pair| pair[1] == pair[0] + 1) {
            continue;
        }

        for offset in empty_offsets {
            if virtual_run_len(board, mv, dr, dc, offset, player) < 4 {
                continue;
            }

            let row = mv.row as isize + dr * offset;
            let col = mv.col as isize + dc * offset;
            if !in_bounds(board, row, col) {
                continue;
            }

            let rest = Move {
                row: row as usize,
                col: col as usize,
            };
            if !rest_squares.contains(&rest) {
                rest_squares.push(rest);
            }
        }
    }

    rest_squares.sort_by_key(|mv| (mv.row, mv.col));
    rest_squares
}

fn virtual_run_len(
    board: &Board,
    mv: Move,
    dr: isize,
    dc: isize,
    virtual_offset: isize,
    player: Color,
) -> usize {
    1 + virtual_count_in_direction(board, mv, dr, dc, virtual_offset, -1, player)
        + virtual_count_in_direction(board, mv, dr, dc, virtual_offset, 1, player)
}

fn virtual_count_in_direction(
    board: &Board,
    mv: Move,
    dr: isize,
    dc: isize,
    virtual_offset: isize,
    step: isize,
    player: Color,
) -> usize {
    let mut count = 0usize;
    let mut offset = virtual_offset + step;
    loop {
        let row = mv.row as isize + dr * offset;
        let col = mv.col as isize + dc * offset;
        if !in_bounds(board, row, col) || board.cell(row as usize, col as usize) != Some(player) {
            break;
        }
        count += 1;
        offset += step;
    }
    count
}

// --- Candidate move generation ---

const STACK_SEEN_WORDS: usize = 4;
const STACK_SEEN_CELLS: usize = STACK_SEEN_WORDS * u64::BITS as usize;

fn candidate_moves(board: &Board, radius: usize) -> Vec<Move> {
    if board.result != GameResult::Ongoing {
        return Vec::new();
    }

    let size = board.config.board_size;
    let cell_count = size * size;
    let mut moves = Vec::new();
    let has_stones = if cell_count <= STACK_SEEN_CELLS {
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

fn mark_candidate_moves(board: &Board, radius: usize, seen: &mut [u64]) -> bool {
    let size = board.config.board_size;
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
                    if board.cell(r, c).is_none() {
                        mark_seen(seen, idx);
                    }
                }
            }
        }
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

#[doc(hidden)]
pub fn pipeline_bench_candidate_moves(board: &Board, radius: usize) -> Vec<Move> {
    candidate_moves(board, radius)
}

fn candidate_moves_counted(
    board: &Board,
    radius: usize,
    metrics: &mut SearchMetrics,
    phase: SearchMetricPhase,
) -> Vec<Move> {
    let moves = candidate_moves(board, radius);
    metrics.record_candidates(moves.len(), phase);
    moves
}

fn candidate_moves_from_source_counted(
    board: &Board,
    candidate_source: CandidateSource,
    metrics: &mut SearchMetrics,
    phase: SearchMetricPhase,
) -> Vec<Move> {
    candidate_moves_counted(board, candidate_source.radius(), metrics, phase)
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

fn order_moves(moves: Vec<Move>, move_ordering: MoveOrdering, tt_move: Option<Move>) -> Vec<Move> {
    match move_ordering {
        MoveOrdering::TranspositionFirstBoardOrder => {
            if let Some(tm) = tt_move.filter(|tm| moves.contains(tm)) {
                std::iter::once(tm)
                    .chain(moves.into_iter().filter(|&m| m != tm))
                    .collect()
            } else {
                moves
            }
        }
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
    nodes: &mut u64,
    metrics: &mut SearchMetrics,
    deadline: SearchDeadline,
) -> (i32, Option<Move>, bool) {
    *nodes += 1;

    if deadline.expired() {
        let sign = if color == root_color { 1 } else { -1 };
        return (
            sign * evaluate_counted(board, root_color, metrics),
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
                    return (entry.score, entry.best_move, false);
                }
                TTFlag::LowerBound => {
                    if entry.score >= beta {
                        metrics.tt_cutoffs += 1;
                        return (entry.score, entry.best_move, false);
                    }
                }
                TTFlag::UpperBound => {
                    if entry.score <= alpha {
                        metrics.tt_cutoffs += 1;
                        return (entry.score, entry.best_move, false);
                    }
                }
            }
        }
    }

    if depth == 0 || board.result != GameResult::Ongoing {
        let sign = if color == root_color { 1 } else { -1 };
        return (
            sign * evaluate_counted(board, root_color, metrics),
            None,
            false,
        );
    }

    let moves = candidate_moves_from_source_counted(
        board,
        candidate_source,
        metrics,
        SearchMetricPhase::Search,
    );
    if moves.is_empty() {
        let sign = if color == root_color { 1 } else { -1 };
        return (
            sign * evaluate_counted(board, root_color, metrics),
            None,
            false,
        );
    }

    let orig_alpha = alpha;
    let mut best_score = i32::MIN + 1;
    let mut best_move: Option<Move> = None;

    let tt_move = tt.get(&hash).and_then(|e| e.best_move);
    let ordered = order_moves(moves, move_ordering, tt_move);

    let needs_legality_check = needs_legality_gate(board, color, legality_gate);
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
        // Incrementally update hash: XOR in the placed piece and flip turn bit
        let child_hash = hash ^ zobrist.piece(mv.row, mv.col, color) ^ zobrist.turn;
        board.apply_trusted_legal_move(mv);
        let (score, _, child_timed_out) = negamax(
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
            nodes,
            metrics,
            deadline,
        );
        let score = -score;
        board.undo_move(mv);

        if child_timed_out {
            timed_out = true;
        }
        if score > best_score {
            best_score = score;
            best_move = Some(mv);
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
        let sign = if color == root_color { 1 } else { -1 };
        return (
            sign * evaluate_counted(board, root_color, metrics),
            None,
            timed_out,
        );
    }

    if timed_out {
        return (best_score, best_move, true);
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

    (best_score, best_move, false)
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
    nodes: &mut u64,
    metrics: &mut SearchMetrics,
    deadline: SearchDeadline,
) -> (i32, Option<Move>, bool) {
    *nodes += 1;
    if deadline.expired() {
        return (evaluate_counted(board, color, metrics), None, true);
    }

    let mut alpha = i32::MIN + 1;
    let beta = i32::MAX;
    let orig_alpha = alpha;
    let mut best_score = i32::MIN + 1;
    let mut best_move: Option<Move> = None;

    let tt_move = tt.get(&hash).and_then(|entry| entry.best_move);
    let ordered = order_moves(root_moves.to_vec(), move_ordering, tt_move);

    let mut timed_out = false;
    for mv in ordered {
        if deadline.expired() {
            timed_out = true;
            break;
        }

        let child_hash = hash ^ zobrist.piece(mv.row, mv.col, color) ^ zobrist.turn;
        board.apply_trusted_legal_move(mv);
        let (score, _, child_timed_out) = negamax(
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
            nodes,
            metrics,
            deadline,
        );
        let score = -score;
        board.undo_move(mv);

        if child_timed_out {
            timed_out = true;
        }
        if score > best_score {
            best_score = score;
            best_move = Some(mv);
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
        return (evaluate_counted(board, color, metrics), None, timed_out);
    }

    if timed_out {
        return (best_score, best_move, true);
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

    (best_score, best_move, false)
}

// --- SearchBot ---

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidateSource {
    NearAll { radius: usize },
}

impl CandidateSource {
    const fn radius(self) -> usize {
        match self {
            CandidateSource::NearAll { radius } => radius,
        }
    }

    fn name(self) -> String {
        match self {
            CandidateSource::NearAll { radius } => format!("near_all_r{radius}"),
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
}

impl SafetyGate {
    const fn name(self) -> &'static str {
        match self {
            SafetyGate::None => "none",
            SafetyGate::OpponentReplySearchProbe => "opponent_reply_search_probe",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveOrdering {
    TranspositionFirstBoardOrder,
}

impl MoveOrdering {
    const fn name(self) -> &'static str {
        match self {
            MoveOrdering::TranspositionFirstBoardOrder => "tt_first_board_order",
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
}

impl StaticEvaluation {
    const fn name(self) -> &'static str {
        match self {
            StaticEvaluation::LineShapeEval => "line_shape_eval",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchBotConfig {
    pub max_depth: i32,
    pub time_budget_ms: Option<u64>,
    pub cpu_time_budget_ms: Option<u64>,
    pub candidate_radius: usize,
    pub safety_gate: SafetyGate,
    pub move_ordering: MoveOrdering,
    pub search_algorithm: SearchAlgorithm,
    pub static_eval: StaticEvaluation,
}

impl SearchBotConfig {
    pub const fn custom_depth(max_depth: i32) -> Self {
        Self {
            max_depth,
            time_budget_ms: None,
            cpu_time_budget_ms: None,
            candidate_radius: 2,
            safety_gate: SafetyGate::OpponentReplySearchProbe,
            move_ordering: MoveOrdering::TranspositionFirstBoardOrder,
            search_algorithm: SearchAlgorithm::AlphaBetaIterativeDeepening,
            static_eval: StaticEvaluation::LineShapeEval,
        }
    }

    pub const fn custom_time_budget(time_budget_ms: u64) -> Self {
        Self {
            max_depth: 20,
            time_budget_ms: Some(time_budget_ms),
            cpu_time_budget_ms: None,
            candidate_radius: 2,
            safety_gate: SafetyGate::OpponentReplySearchProbe,
            move_ordering: MoveOrdering::TranspositionFirstBoardOrder,
            search_algorithm: SearchAlgorithm::AlphaBetaIterativeDeepening,
            static_eval: StaticEvaluation::LineShapeEval,
        }
    }

    pub const fn custom_cpu_time_budget(cpu_time_budget_ms: u64) -> Self {
        Self {
            max_depth: 20,
            time_budget_ms: None,
            cpu_time_budget_ms: Some(cpu_time_budget_ms),
            candidate_radius: 2,
            safety_gate: SafetyGate::OpponentReplySearchProbe,
            move_ordering: MoveOrdering::TranspositionFirstBoardOrder,
            search_algorithm: SearchAlgorithm::AlphaBetaIterativeDeepening,
            static_eval: StaticEvaluation::LineShapeEval,
        }
    }

    fn time_budget(self) -> Option<Duration> {
        self.time_budget_ms.map(Duration::from_millis)
    }

    fn cpu_time_budget(self) -> Option<Duration> {
        self.cpu_time_budget_ms.map(Duration::from_millis)
    }

    pub const fn candidate_source(self) -> CandidateSource {
        CandidateSource::NearAll {
            radius: self.candidate_radius,
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
            "candidate_source": self.candidate_source().name(),
            "legality_gate": self.legality_gate().name(),
            "safety_gate": self.safety_gate().name(),
            "move_ordering": self.move_ordering.name(),
            "search_algorithm": self.search_algorithm.name(),
            "static_eval": self.static_eval.name(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct SearchInfo {
    pub depth_reached: i32,
    pub nodes: u64,
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
            serde_json::json!({
                "config": self.config.trace(),
                "depth": info.depth_reached,
                "nodes": info.nodes,
                "safety_nodes": info.safety_nodes,
                "total_nodes": info.nodes + info.safety_nodes,
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
        let root_hash = hash_board(&self.zobrist, board);
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

        for depth in 1..=self.config.max_depth {
            if deadline.expired() {
                budget_exhausted = true;
                break;
            }
            let mut nodes = 0u64;
            let (score, mv, timed_out) = search_root(
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
                &mut nodes,
                &mut metrics,
                deadline,
            );
            total_nodes += nodes;

            if !timed_out {
                if let Some(m) = mv {
                    best_move = m;
                    best_score = score;
                }
                depth_reached = depth;
            } else if depth_reached == 0 {
                if let Some(m) = mv {
                    best_move = m;
                    best_score = score;
                }
            }

            if timed_out {
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
    fn explicit_config_constructors_preserve_legacy_defaults() {
        let baseline = SearchBotConfig::custom_depth(3);
        assert_eq!(SearchBot::new(3).config(), baseline);
        assert_eq!(
            baseline.candidate_source(),
            CandidateSource::NearAll { radius: 2 }
        );
        assert_eq!(baseline.legality_gate(), LegalityGate::ExactRules);
        assert_eq!(baseline.safety_gate(), SafetyGate::OpponentReplySearchProbe);
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
            safety_gate: SafetyGate::None,
            move_ordering: MoveOrdering::TranspositionFirstBoardOrder,
            search_algorithm: SearchAlgorithm::AlphaBetaIterativeDeepening,
            static_eval: StaticEvaluation::LineShapeEval,
        };
        assert_eq!(SearchBot::with_config(config).config(), config);
        assert_eq!(
            config.candidate_source(),
            CandidateSource::NearAll { radius: 3 }
        );
        assert_eq!(config.safety_gate, SafetyGate::None);
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
            "opponent_reply_search_probe"
        );
        assert_eq!(trace["config"]["move_ordering"], "tt_first_board_order");
        assert_eq!(trace["config"]["search_algorithm"], "alpha_beta_id");
        assert_eq!(trace["config"]["static_eval"], "line_shape_eval");
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
            safety_gate: SafetyGate::None,
            move_ordering: MoveOrdering::TranspositionFirstBoardOrder,
            search_algorithm: SearchAlgorithm::AlphaBetaIterativeDeepening,
            static_eval: StaticEvaluation::LineShapeEval,
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
    fn tactical_analyzer_labels_open_and_blocked_fours() {
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
        assert!(!open_four.blocked_four);

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

        let blocked_four = analyze_tactical_move(&board, Move { row: 7, col: 10 });
        assert!(!blocked_four.open_four);
        assert!(blocked_four.blocked_four);
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
    fn local_threat_facts_report_five_open_four_and_simple_four() {
        let mut five_board = Board::new(RuleConfig::default());
        apply_moves(
            &mut five_board,
            &["H8", "A1", "I8", "C1", "J8", "E1", "K8", "G1"],
        );

        let five = local_threat_facts_after_move(&five_board, mv("L8"));
        assert_eq!(
            five,
            vec![LocalThreatFact {
                player: Color::Black,
                kind: LocalThreatKind::Five,
                gain_square: mv("L8"),
                defense_squares: vec![],
                rest_squares: vec![],
            }]
        );
        assert!(five[0].is_forcing());

        let mut open_four_board = Board::new(RuleConfig::default());
        apply_moves(&mut open_four_board, &["H8", "A1", "I8", "C1", "J8", "E1"]);

        let open_four = local_threat_facts_after_move(&open_four_board, mv("K8"));
        assert_eq!(
            open_four,
            vec![LocalThreatFact {
                player: Color::Black,
                kind: LocalThreatKind::OpenFour,
                gain_square: mv("K8"),
                defense_squares: vec![mv("G8"), mv("L8")],
                rest_squares: vec![],
            }]
        );
        assert!(open_four[0].is_forcing());

        let mut simple_four_board = Board::new(RuleConfig::default());
        apply_moves(
            &mut simple_four_board,
            &["H8", "G8", "I8", "A1", "J8", "C1"],
        );

        let simple_four = local_threat_facts_after_move(&simple_four_board, mv("K8"));
        assert_eq!(
            simple_four,
            vec![LocalThreatFact {
                player: Color::Black,
                kind: LocalThreatKind::SimpleFour,
                gain_square: mv("K8"),
                defense_squares: vec![mv("L8")],
                rest_squares: vec![],
            }]
        );
        assert!(simple_four[0].is_forcing());
    }

    #[test]
    fn local_threat_facts_report_gap_four_as_simple_four() {
        let mut board = Board::new(RuleConfig::default());
        apply_moves(&mut board, &["H8", "A1", "I8", "C1", "L8", "E1"]);

        let gap_four = local_threat_facts_after_move(&board, mv("J8"));
        assert_eq!(
            gap_four,
            vec![LocalThreatFact {
                player: Color::Black,
                kind: LocalThreatKind::SimpleFour,
                gain_square: mv("J8"),
                defense_squares: vec![mv("K8")],
                rest_squares: vec![],
            }]
        );
        assert!(gap_four[0].is_forcing());
    }

    #[test]
    fn local_threat_facts_report_open_three_and_broken_three() {
        let mut open_three_board = Board::new(RuleConfig::default());
        apply_moves(&mut open_three_board, &["H8", "A1", "I8", "C1"]);

        let open_three = local_threat_facts_after_move(&open_three_board, mv("J8"));
        assert_eq!(
            open_three,
            vec![LocalThreatFact {
                player: Color::Black,
                kind: LocalThreatKind::OpenThree,
                gain_square: mv("J8"),
                defense_squares: vec![mv("G8"), mv("K8")],
                rest_squares: vec![],
            }]
        );
        assert!(open_three[0].is_forcing());

        let mut broken_three_board = Board::new(RuleConfig::default());
        apply_moves(&mut broken_three_board, &["H8", "A1", "K8", "C1"]);

        let broken_three = local_threat_facts_after_move(&broken_three_board, mv("J8"));
        assert_eq!(
            broken_three,
            vec![LocalThreatFact {
                player: Color::Black,
                kind: LocalThreatKind::BrokenThree,
                gain_square: mv("J8"),
                defense_squares: vec![],
                rest_squares: vec![mv("I8")],
            }]
        );
        assert!(!broken_three[0].is_forcing());
    }

    #[test]
    fn local_threat_facts_skip_quiet_and_illegal_moves() {
        let empty = Board::new(RuleConfig::default());
        assert!(local_threat_facts_after_move(&empty, mv("H8")).is_empty());

        let mut occupied = Board::new(RuleConfig::default());
        occupied.apply_move(mv("H8")).unwrap();
        assert!(local_threat_facts_after_move(&occupied, mv("H8")).is_empty());
    }

    #[test]
    fn forced_line_classifier_prioritizes_current_immediate_win() {
        let scenario = scenarios::SCENARIOS
            .iter()
            .find(|scenario| scenario.id == "attack_wins_race")
            .expect("expected attack race scenario");
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
            .find(|scenario| scenario.id == "immediate_block")
            .expect("expected immediate block scenario");
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
            .find(|scenario| scenario.id == "immediate_win")
            .expect("expected immediate win scenario");
        let board = scenario.board();

        let winning = classify_threat_after_move(&board, mv("G8"));
        assert_eq!(winning.kind, ThreatAfterMoveKind::WinsNow);
        assert!(winning.winning_replies.is_empty());

        let illegal = classify_threat_after_move(&board, mv("H8"));
        assert_eq!(illegal.kind, ThreatAfterMoveKind::Illegal);
        assert!(illegal.winning_replies.is_empty());

        let mut blocked_four_board = Board::new(RuleConfig::default());
        apply_moves(
            &mut blocked_four_board,
            &["H8", "G8", "I8", "A1", "J8", "C1"],
        );
        let single = classify_threat_after_move(&blocked_four_board, mv("K8"));
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
