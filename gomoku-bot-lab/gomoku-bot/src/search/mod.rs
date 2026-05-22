use instant::Instant;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::Duration;

use crate::corridor;
use crate::frontier::{FrontierAnnotationSource, RollingFrontierFeatures, RollingThreatFrontier};
use crate::pattern::{evaluate_pattern_scan, PatternFrame};
use crate::tactical::{
    tactical_metrics_snapshot, CorridorThreatPolicy, LocalThreatKind, ScanThreatView,
    SearchThreatPolicy, TacticalMoveAnnotation, TacticalOrderingSummary, ThreatObligationKind,
    ThreatView,
};
use crate::viability::{direction_bit, scan_cell_null, scan_cell_viability};
use crate::Bot;
use gomoku_core::{
    renju_forbidden_metrics_snapshot, Board, Color, GameResult, Move, Variant, ZobristTable, DIRS,
};

#[cfg(test)]
use crate::tactical::{local_threat_facts_after_move, LocalThreatFact, LocalThreatOrigin};

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

#[derive(Clone, Copy)]
struct RootCandidateOptions {
    candidate_source: CandidateSource,
    null_cell_culling: NullCellCulling,
    legality_gate: LegalityGate,
    safety_gate: SafetyGate,
    threat_view_mode: ThreatViewMode,
    deadline: SearchDeadline,
}

#[derive(Clone, Copy)]
struct MoveOrderingOptions {
    move_ordering: MoveOrdering,
    child_limit: Option<usize>,
    threat_view_mode: ThreatViewMode,
    phase: SearchMetricPhase,
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
        StaticEvaluation::PatternEval => evaluate_pattern_scan(board, color),
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

mod metrics;

pub use metrics::SearchMetrics;
use metrics::{RenjuForbiddenMetricSource, SearchMetricPhase};

#[derive(Debug, Clone)]
struct SearchState {
    board: Board,
    frontier: Option<RollingThreatFrontier>,
    pattern_frame: Option<PatternFrame>,
    frontier_ordering_summary_memo: HashMap<FrontierAnnotationMemoKey, TacticalOrderingSummary>,
    hash: u64,
    hash_stack: Vec<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct FrontierAnnotationMemoKey {
    hash: u64,
    player: u8,
    row: usize,
    col: usize,
}

impl SearchState {
    #[cfg(test)]
    fn from_board(board: Board, zobrist: &ZobristTable) -> Self {
        Self::from_board_with_frontier(board, zobrist, true)
    }

    fn from_board_for_config(
        board: Board,
        zobrist: &ZobristTable,
        mode: ThreatViewMode,
        static_eval: StaticEvaluation,
        corridor_proof: CorridorProofConfig,
    ) -> Self {
        Self::from_board_with_frontier_features(
            board,
            zobrist,
            frontier_features_for_search(mode, corridor_proof),
            pattern_frame_for_search(mode, static_eval),
        )
    }

    #[cfg(test)]
    fn from_board_with_frontier(
        board: Board,
        zobrist: &ZobristTable,
        enable_frontier: bool,
    ) -> Self {
        Self::from_board_with_frontier_features(
            board,
            zobrist,
            enable_frontier.then_some(RollingFrontierFeatures::Full),
            false,
        )
    }

    fn from_board_with_frontier_features(
        board: Board,
        zobrist: &ZobristTable,
        frontier_features: Option<RollingFrontierFeatures>,
        enable_pattern_frame: bool,
    ) -> Self {
        let hash = board.hash_with(zobrist);
        let frontier = frontier_features
            .map(|features| RollingThreatFrontier::from_board_with_features(&board, features));
        let pattern_frame = enable_pattern_frame.then(|| PatternFrame::from_board(&board));
        Self {
            board,
            frontier,
            pattern_frame,
            frontier_ordering_summary_memo: HashMap::new(),
            hash,
            hash_stack: Vec::new(),
        }
    }

    fn board(&self) -> &Board {
        &self.board
    }

    fn threat_view(&self) -> &RollingThreatFrontier {
        self.frontier
            .as_ref()
            .expect("search state frontier requested when disabled")
    }

    fn threat_view_mut(&mut self) -> &mut RollingThreatFrontier {
        self.frontier
            .as_mut()
            .expect("search state frontier requested when disabled")
    }

    fn hash(&self) -> u64 {
        self.hash
    }

    fn frontier_annotation_memo_key(&self, player: Color, mv: Move) -> FrontierAnnotationMemoKey {
        FrontierAnnotationMemoKey {
            hash: self.hash,
            player: player as u8,
            row: mv.row,
            col: mv.col,
        }
    }

    #[cfg(test)]
    fn apply_trusted_legal_move(&mut self, mv: Move, zobrist: &ZobristTable) -> GameResult {
        self.apply_trusted_legal_move_inner(mv, zobrist, None)
    }

    fn apply_trusted_legal_move_counted(
        &mut self,
        mv: Move,
        zobrist: &ZobristTable,
        metrics: &mut SearchMetrics,
    ) -> GameResult {
        self.apply_trusted_legal_move_inner(mv, zobrist, Some(metrics))
    }

    fn apply_trusted_legal_move_inner(
        &mut self,
        mv: Move,
        zobrist: &ZobristTable,
        metrics: Option<&mut SearchMetrics>,
    ) -> GameResult {
        let color = self.board.current_player;
        self.hash_stack.push(self.hash);
        self.hash ^= zobrist.piece(mv.row, mv.col, color) ^ zobrist.turn;
        let board_result = self.board.apply_trusted_legal_move(mv);
        let mut metrics = metrics;
        if let Some(frontier) = &mut self.frontier {
            let start = Instant::now();
            let (frontier_result, timings) = frontier.apply_trusted_legal_move_profiled(mv);
            if let Some(metrics) = metrics.as_mut() {
                metrics.record_threat_view_frontier_rebuild(start.elapsed());
                metrics.record_threat_view_frontier_update_parts(timings);
            }
            debug_assert_eq!(
                board_result, frontier_result,
                "search state board/frontier result diverged after apply"
            );
        }
        if let Some(pattern_frame) = &mut self.pattern_frame {
            let renju_before = renju_forbidden_metrics_snapshot();
            let start = Instant::now();
            let pattern_result = pattern_frame.apply_trusted_legal_move(mv);
            if let Some(metrics) = metrics.as_mut() {
                metrics.record_pattern_frame_update(start.elapsed());
                metrics.record_renju_forbidden_source_delta(
                    RenjuForbiddenMetricSource::Pattern,
                    renju_before,
                );
            }
            debug_assert_eq!(
                board_result, pattern_result,
                "search state board/pattern-frame result diverged after apply"
            );
        }
        board_result
    }

    #[cfg(test)]
    fn undo_move(&mut self, mv: Move) {
        self.undo_move_inner(mv, None);
    }

    fn undo_move_counted(&mut self, mv: Move, metrics: &mut SearchMetrics) {
        self.undo_move_inner(mv, Some(metrics));
    }

    fn undo_move_inner(&mut self, mv: Move, metrics: Option<&mut SearchMetrics>) {
        self.board.undo_move(mv);
        let mut metrics = metrics;
        if let Some(frontier) = &mut self.frontier {
            let start = Instant::now();
            let timings = frontier.undo_move_profiled(mv);
            if let Some(metrics) = metrics.as_mut() {
                metrics.record_threat_view_frontier_rebuild(start.elapsed());
                metrics.record_threat_view_frontier_update_parts(timings);
            }
        }
        if let Some(pattern_frame) = &mut self.pattern_frame {
            let renju_before = renju_forbidden_metrics_snapshot();
            let start = Instant::now();
            pattern_frame.undo_move(mv);
            if let Some(metrics) = metrics.as_mut() {
                metrics.record_pattern_frame_update(start.elapsed());
                metrics.record_renju_forbidden_source_delta(
                    RenjuForbiddenMetricSource::Pattern,
                    renju_before,
                );
            }
        }
        self.hash = self
            .hash_stack
            .pop()
            .expect("search state undo_move called without matching apply");
    }
}

fn pattern_frame_for_search(mode: ThreatViewMode, static_eval: StaticEvaluation) -> bool {
    mode.uses_frontier() && static_eval == StaticEvaluation::PatternEval
}

fn frontier_features_for_search(
    mode: ThreatViewMode,
    corridor_proof: CorridorProofConfig,
) -> Option<RollingFrontierFeatures> {
    if !mode.uses_frontier() {
        return None;
    }
    if !corridor_proof.enabled {
        Some(RollingFrontierFeatures::TacticalOnly)
    } else {
        Some(RollingFrontierFeatures::Full)
    }
}

fn evaluate_counted(
    board: &Board,
    color: Color,
    static_eval: StaticEvaluation,
    metrics: &mut SearchMetrics,
) -> i32 {
    let renju_before =
        (static_eval == StaticEvaluation::PatternEval).then(renju_forbidden_metrics_snapshot);
    let start = Instant::now();
    let score = evaluate_static(board, color, static_eval);
    metrics.record_static_eval(static_eval, start.elapsed());
    if let Some(renju_before) = renju_before {
        metrics
            .record_renju_forbidden_source_delta(RenjuForbiddenMetricSource::Pattern, renju_before);
    }
    score
}

fn evaluate_state_counted(
    state: &SearchState,
    color: Color,
    static_eval: StaticEvaluation,
    metrics: &mut SearchMetrics,
) -> i32 {
    if static_eval == StaticEvaluation::PatternEval {
        if let Some(pattern_frame) = &state.pattern_frame {
            let start = Instant::now();
            let score = pattern_frame.score_for(color);
            let elapsed = start.elapsed();
            metrics.record_static_eval(static_eval, elapsed);
            metrics.record_pattern_frame_query(elapsed);

            #[cfg(any(test, debug_assertions))]
            {
                metrics.pattern_frame_shadow_checks += 1;
                let scan_score = evaluate_pattern_scan(state.board(), color);
                if scan_score != score {
                    metrics.pattern_frame_shadow_mismatches += 1;
                }
                debug_assert_eq!(
                    score, scan_score,
                    "cached pattern frame diverged from scan pattern eval"
                );
            }

            return score;
        }
    }

    evaluate_counted(state.board(), color, static_eval, metrics)
}

fn evaluate_leaf_counted(
    state: &SearchState,
    color: Color,
    root_color: Color,
    static_eval: StaticEvaluation,
    metrics: &mut SearchMetrics,
) -> i32 {
    let sign = if color == root_color { 1 } else { -1 };
    sign * evaluate_state_counted(state, root_color, static_eval, metrics)
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

fn tactical_ordering_summary_counted(
    state: &mut SearchState,
    mv: Move,
    threat_view_mode: ThreatViewMode,
    metrics: &mut SearchMetrics,
    phase: SearchMetricPhase,
) -> TacticalOrderingSummary {
    metrics.record_tactical_annotation(phase);
    tactical_ordering_summary_for_threat_view_mode(
        state,
        state.board().current_player,
        mv,
        threat_view_mode,
        metrics,
    )
}

fn tactical_ordering_summary_for_threat_view_mode(
    state: &mut SearchState,
    player: Color,
    mv: Move,
    mode: ThreatViewMode,
    metrics: &mut SearchMetrics,
) -> TacticalOrderingSummary {
    match mode {
        ThreatViewMode::Scan => {
            scan_tactical_ordering_summary_for_player_timed(state.board(), player, mv, metrics)
        }
        ThreatViewMode::Rolling => {
            rolling_frontier_tactical_ordering_summary_for_player_timed(state, player, mv, metrics)
        }
        ThreatViewMode::RollingShadow => {
            metrics.threat_view_shadow_checks += 1;
            let scan =
                scan_tactical_ordering_summary_for_player_timed(state.board(), player, mv, metrics);
            let frontier = rolling_frontier_tactical_ordering_summary_for_player_timed(
                state, player, mv, metrics,
            );
            if frontier != scan {
                metrics.threat_view_shadow_mismatches += 1;
            }
            scan
        }
    }
}

fn scan_tactical_ordering_summary_for_player_timed(
    board: &Board,
    player: Color,
    mv: Move,
    metrics: &mut SearchMetrics,
) -> TacticalOrderingSummary {
    let renju_before = renju_forbidden_metrics_snapshot();
    let start = Instant::now();
    let summary = SearchThreatPolicy.ordering_summary_for_legal_player(board, player, mv);
    metrics.record_threat_view_scan(start.elapsed());
    metrics.record_renju_forbidden_source_delta(RenjuForbiddenMetricSource::Threat, renju_before);
    summary
}

fn rolling_frontier_tactical_ordering_summary_for_player_timed(
    state: &mut SearchState,
    player: Color,
    mv: Move,
    metrics: &mut SearchMetrics,
) -> TacticalOrderingSummary {
    let renju_before = renju_forbidden_metrics_snapshot();
    let start = Instant::now();
    let key = state.frontier_annotation_memo_key(player, mv);
    if let Some(summary) = state.frontier_ordering_summary_memo.get(&key).copied() {
        metrics.record_threat_view_frontier_memo_annotation_query(start.elapsed());
        metrics
            .record_renju_forbidden_source_delta(RenjuForbiddenMetricSource::Threat, renju_before);
        return summary;
    }

    let (summary, source) = {
        let frontier = state.threat_view();
        frontier.search_ordering_summary_for_legal_player_with_source(player, mv)
    };
    metrics.record_threat_view_frontier_annotation_query(start.elapsed(), source);
    metrics.record_renju_forbidden_source_delta(RenjuForbiddenMetricSource::Threat, renju_before);
    if source == FrontierAnnotationSource::DirtyRecompute {
        state.frontier_ordering_summary_memo.insert(key, summary);
    }
    summary
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
    let start = Instant::now();
    let moves = candidate_moves_from_source(board, candidate_source);
    metrics.record_candidates(moves.len(), start.elapsed(), phase);
    moves
}

fn cull_null_cells_counted(
    board: &Board,
    frontier: Option<&RollingThreatFrontier>,
    moves: Vec<Move>,
    null_cell_culling: NullCellCulling,
    threat_view_mode: ThreatViewMode,
    metrics: &mut SearchMetrics,
    phase: SearchMetricPhase,
) -> Vec<Move> {
    if !null_cell_culling.enabled() || moves.is_empty() {
        return moves;
    }

    let start = Instant::now();
    let mut kept = Vec::with_capacity(moves.len());
    let mut culled = 0usize;
    for mv in moves {
        if is_null_cell_for_mode(board, frontier, mv, threat_view_mode, metrics) {
            culled += 1;
        } else {
            kept.push(mv);
        }
    }
    let checks = kept.len() + culled;
    metrics.record_null_cell_cull(checks, culled, start.elapsed(), phase);
    kept
}

fn is_null_cell_for_mode(
    board: &Board,
    frontier: Option<&RollingThreatFrontier>,
    mv: Move,
    threat_view_mode: ThreatViewMode,
    metrics: &mut SearchMetrics,
) -> bool {
    match threat_view_mode {
        ThreatViewMode::Scan => scan_cell_null(board, mv),
        ThreatViewMode::Rolling => frontier
            .map(|frontier| frontier.is_null_cell(mv))
            .unwrap_or_else(|| scan_cell_null(board, mv)),
        ThreatViewMode::RollingShadow => {
            let scan = scan_cell_null(board, mv);
            if let Some(frontier) = frontier {
                metrics.threat_view_shadow_checks += 1;
                let rolling = frontier.is_null_cell(mv);
                if scan != rolling {
                    metrics.threat_view_shadow_mismatches += 1;
                }
            }
            scan
        }
    }
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
        LegalityGate::ExactRules => {
            let renju_before = renju_forbidden_metrics_snapshot();
            let start = Instant::now();
            let legal = board.is_legal(mv);
            let accepted = metrics.record_legality(legal, start.elapsed(), phase);
            metrics.record_renju_forbidden_source_delta(
                RenjuForbiddenMetricSource::SearchGate,
                renju_before,
            );
            accepted
        }
    }
}

fn root_candidate_moves_with_metrics(
    board: &Board,
    options: RootCandidateOptions,
    metrics: &mut SearchMetrics,
) -> (Vec<Move>, u64, bool) {
    let mut moves = candidate_moves_from_source_counted(
        board,
        options.candidate_source,
        metrics,
        SearchMetricPhase::Root,
    );
    moves = cull_null_cells_counted(
        board,
        None,
        moves,
        options.null_cell_culling,
        options.threat_view_mode,
        metrics,
        SearchMetricPhase::Root,
    );
    if needs_legality_gate(board, board.current_player, options.legality_gate) {
        moves.retain(|&mv| {
            legal_by_gate_counted(
                board,
                mv,
                options.legality_gate,
                metrics,
                SearchMetricPhase::Root,
            )
        });
    }

    apply_safety_gate_to_root_candidates(
        board,
        moves,
        options.safety_gate,
        options.threat_view_mode,
        options.deadline,
        metrics,
    )
}

fn apply_safety_gate_to_root_candidates(
    board: &Board,
    moves: Vec<Move>,
    safety_gate: SafetyGate,
    threat_view_mode: ThreatViewMode,
    deadline: SearchDeadline,
    metrics: &mut SearchMetrics,
) -> (Vec<Move>, u64, bool) {
    match safety_gate {
        SafetyGate::None => (moves, 0, false),
        SafetyGate::CurrentObligation => {
            current_obligation_root_candidates(board, moves, threat_view_mode, deadline, metrics)
        }
    }
}

#[derive(Debug, Clone)]
struct SafetyFilterOutcome {
    moves: Vec<Move>,
    work_units: u64,
}

fn current_obligation_root_candidates(
    board: &Board,
    moves: Vec<Move>,
    threat_view_mode: ThreatViewMode,
    deadline: SearchDeadline,
    metrics: &mut SearchMetrics,
) -> (Vec<Move>, u64, bool) {
    if moves.is_empty() {
        return (moves, 0, false);
    }
    if deadline.expired() {
        return (moves, 0, true);
    }

    let outcome = match threat_view_mode {
        ThreatViewMode::Scan => {
            let view = ScanThreatView::new(board);
            let start = Instant::now();
            let outcome = current_obligation_safety_policy(board, &moves, &view);
            metrics.record_threat_view_scan(start.elapsed());
            outcome
        }
        ThreatViewMode::Rolling => {
            let start = Instant::now();
            let mut frontier = RollingThreatFrontier::from_board_with_features(
                board,
                RollingFrontierFeatures::Full,
            );
            metrics.record_threat_view_frontier_rebuild(start.elapsed());
            rolling_current_obligation_safety_policy(board, &moves, &mut frontier, metrics)
        }
        ThreatViewMode::RollingShadow => {
            metrics.threat_view_shadow_checks += 1;

            let scan_view = ScanThreatView::new(board);
            let start = Instant::now();
            let scan = current_obligation_safety_policy(board, &moves, &scan_view);
            metrics.record_threat_view_scan(start.elapsed());

            let start = Instant::now();
            let mut frontier = RollingThreatFrontier::from_board_with_features(
                board,
                RollingFrontierFeatures::Full,
            );
            metrics.record_threat_view_frontier_rebuild(start.elapsed());
            let rolling =
                rolling_current_obligation_safety_policy(board, &moves, &mut frontier, metrics);

            if scan.moves != rolling.moves {
                metrics.threat_view_shadow_mismatches += 1;
            }
            scan
        }
    };

    (outcome.moves, outcome.work_units, false)
}

fn current_obligation_safety_policy(
    board: &Board,
    moves: &[Move],
    view: &impl ThreatView,
) -> SafetyFilterOutcome {
    let current = board.current_player;
    if let Some(outcome) =
        immediate_win_safety_outcome(moves, view.immediate_winning_moves_for(current))
    {
        return outcome;
    }

    let opponent = current.opponent();
    if let Some(outcome) =
        immediate_win_safety_outcome(moves, view.immediate_winning_moves_for(opponent))
    {
        return outcome;
    }

    current_obligation_safety_policy_after_immediate(board, moves, view)
}

fn rolling_current_obligation_safety_policy(
    board: &Board,
    moves: &[Move],
    view: &mut RollingThreatFrontier,
    metrics: &mut SearchMetrics,
) -> SafetyFilterOutcome {
    let current = board.current_player;
    let start = Instant::now();
    let own_wins = view.immediate_winning_moves_for_cached(current);
    metrics.record_threat_view_frontier_immediate_win_query(start.elapsed());
    if let Some(outcome) = immediate_win_safety_outcome(moves, own_wins) {
        return outcome;
    }

    let opponent = current.opponent();
    let start = Instant::now();
    let opponent_wins = view.immediate_winning_moves_for_cached(opponent);
    metrics.record_threat_view_frontier_immediate_win_query(start.elapsed());
    if let Some(outcome) = immediate_win_safety_outcome(moves, opponent_wins) {
        return outcome;
    }

    let start = Instant::now();
    let outcome = current_obligation_safety_policy_after_immediate(board, moves, view);
    metrics.record_threat_view_frontier_query(start.elapsed());
    outcome
}

fn immediate_win_safety_outcome(
    moves: &[Move],
    winning_moves: Vec<Move>,
) -> Option<SafetyFilterOutcome> {
    let wins = moves_in_set(moves, &winning_moves);
    (!wins.is_empty()).then(|| SafetyFilterOutcome {
        moves: filtered_or_original(moves, wins),
        work_units: moves.len() as u64,
    })
}

fn current_obligation_safety_policy_after_immediate(
    board: &Board,
    moves: &[Move],
    view: &impl ThreatView,
) -> SafetyFilterOutcome {
    let opponent = board.current_player.opponent();
    let Some(obligation) = view.threat_obligation(opponent) else {
        return SafetyFilterOutcome {
            moves: moves.to_vec(),
            work_units: 0,
        };
    };
    if obligation.kind != ThreatObligationKind::Imminent {
        return SafetyFilterOutcome {
            moves: moves.to_vec(),
            work_units: 0,
        };
    }

    let replies = obligation.legal_replies;
    let mut work_units = moves.len() as u64;
    let filtered = moves
        .iter()
        .copied()
        .filter(|&mv| {
            if replies.contains(&mv) {
                return true;
            }
            work_units += 1;
            creates_counter_four(view.search_annotation_for_move(mv))
        })
        .collect::<Vec<_>>();

    SafetyFilterOutcome {
        moves: filtered_or_original(moves, filtered),
        work_units,
    }
}

fn moves_in_set(moves: &[Move], set: &[Move]) -> Vec<Move> {
    moves
        .iter()
        .copied()
        .filter(|mv| set.contains(mv))
        .collect()
}

fn filtered_or_original(original: &[Move], filtered: Vec<Move>) -> Vec<Move> {
    if filtered.is_empty() {
        original.to_vec()
    } else {
        filtered
    }
}

fn creates_counter_four(annotation: TacticalMoveAnnotation) -> bool {
    annotation.local_threats.into_iter().any(|fact| {
        matches!(
            fact.kind,
            LocalThreatKind::Five
                | LocalThreatKind::OpenFour
                | LocalThreatKind::ClosedFour
                | LocalThreatKind::BrokenFour
        )
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OrderedMove {
    mv: Move,
    must_keep: bool,
}

fn order_root_moves(
    state: &mut SearchState,
    moves: Vec<Move>,
    move_ordering: MoveOrdering,
    tt_move: Option<Move>,
    threat_view_mode: ThreatViewMode,
    metrics: &mut SearchMetrics,
) -> Vec<Move> {
    let stage_before = metrics.stage_snapshot();
    let start = Instant::now();
    let ordered = match move_ordering {
        MoveOrdering::TranspositionFirstBoardOrder => order_tt_first(moves, tt_move),
        MoveOrdering::TacticalFull | MoveOrdering::Tactical => order_moves_with_ordering(
            state,
            moves,
            tt_move,
            MoveOrderingOptions {
                move_ordering,
                child_limit: None,
                threat_view_mode,
                phase: SearchMetricPhase::Root,
            },
            metrics,
        )
        .into_iter()
        .map(|ordered| ordered.mv)
        .collect(),
    };
    metrics.record_ordering_scope(start.elapsed(), stage_before);
    ordered
}

fn order_search_moves(
    state: &mut SearchState,
    moves: Vec<Move>,
    move_ordering: MoveOrdering,
    tt_move: Option<Move>,
    child_limit: Option<usize>,
    threat_view_mode: ThreatViewMode,
    metrics: &mut SearchMetrics,
) -> Vec<Move> {
    let stage_before = metrics.stage_snapshot();
    let start = Instant::now();
    let ordered = match move_ordering {
        MoveOrdering::TranspositionFirstBoardOrder => {
            let moves = order_tt_first(moves, tt_move);
            apply_plain_child_limit(moves, child_limit, metrics, SearchMetricPhase::Search)
        }
        MoveOrdering::TacticalFull => {
            let ordered = order_moves_tactical_full(
                state,
                moves,
                tt_move,
                threat_view_mode,
                metrics,
                SearchMetricPhase::Search,
            );
            apply_child_limit(ordered, child_limit, metrics, SearchMetricPhase::Search)
        }
        MoveOrdering::Tactical => {
            let ordered = order_moves_tactical(
                state,
                moves,
                tt_move,
                child_limit,
                threat_view_mode,
                metrics,
                SearchMetricPhase::Search,
            );
            apply_child_limit(ordered, child_limit, metrics, SearchMetricPhase::Search)
        }
    };
    metrics.record_ordering_scope(start.elapsed(), stage_before);
    ordered
}

fn order_moves_with_ordering(
    state: &mut SearchState,
    moves: Vec<Move>,
    tt_move: Option<Move>,
    options: MoveOrderingOptions,
    metrics: &mut SearchMetrics,
) -> Vec<OrderedMove> {
    match options.move_ordering {
        MoveOrdering::TranspositionFirstBoardOrder => order_tt_first(moves, tt_move)
            .into_iter()
            .map(|mv| OrderedMove {
                mv,
                must_keep: false,
            })
            .collect(),
        MoveOrdering::TacticalFull => order_moves_tactical_full(
            state,
            moves,
            tt_move,
            options.threat_view_mode,
            metrics,
            options.phase,
        ),
        MoveOrdering::Tactical => order_moves_tactical(
            state,
            moves,
            tt_move,
            options.child_limit,
            options.threat_view_mode,
            metrics,
            options.phase,
        ),
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

fn order_moves_tactical_full(
    state: &mut SearchState,
    moves: Vec<Move>,
    tt_move: Option<Move>,
    threat_view_mode: ThreatViewMode,
    metrics: &mut SearchMetrics,
    phase: SearchMetricPhase,
) -> Vec<OrderedMove> {
    if moves.is_empty() {
        return Vec::new();
    }

    let board_size = state.board().config.board_size;
    let opponent = state.board().current_player.opponent();
    let opponent_immediate_wins = immediate_winning_move_mask_for_threat_view_mode(
        state,
        opponent,
        threat_view_mode,
        metrics,
    );
    let mut scored = moves
        .into_iter()
        .enumerate()
        .map(|(index, mv)| {
            let summary =
                tactical_ordering_summary_counted(state, mv, threat_view_mode, metrics, phase);
            let immediate_block = move_mask_contains(&opponent_immediate_wins, board_size, mv);
            let (score, must_keep) = tactical_ordering_score_from_summary(summary, immediate_block);
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

fn order_moves_tactical(
    state: &mut SearchState,
    moves: Vec<Move>,
    tt_move: Option<Move>,
    child_limit: Option<usize>,
    threat_view_mode: ThreatViewMode,
    metrics: &mut SearchMetrics,
    phase: SearchMetricPhase,
) -> Vec<OrderedMove> {
    if moves.is_empty() {
        return Vec::new();
    }

    if child_limit.is_none() {
        return order_moves_tactical_full(state, moves, tt_move, threat_view_mode, metrics, phase);
    }

    let board_size = state.board().config.board_size;
    let player = state.board().current_player;
    let opponent = player.opponent();
    let own_immediate_wins =
        immediate_winning_move_mask_for_threat_view_mode(state, player, threat_view_mode, metrics);
    let opponent_immediate_wins = immediate_winning_move_mask_for_threat_view_mode(
        state,
        opponent,
        threat_view_mode,
        metrics,
    );

    let mut scored = moves
        .into_iter()
        .enumerate()
        .map(|(index, mv)| {
            let own_win = move_mask_contains(&own_immediate_wins, board_size, mv);
            let immediate_block = move_mask_contains(&opponent_immediate_wins, board_size, mv);
            let (hard_score, hard_keep) = hard_tactical_ordering_score(own_win, immediate_block);
            let should_annotate = hard_keep
                || has_tactical_annotation_potential_for_mode(
                    state,
                    player,
                    mv,
                    threat_view_mode,
                    metrics,
                );
            (
                index,
                mv,
                hard_score,
                hard_keep,
                immediate_block,
                Some(mv) == tt_move,
                should_annotate,
            )
        })
        .collect::<Vec<_>>();

    for scored_move in scored.iter_mut() {
        if !scored_move.6 {
            continue;
        }

        let summary = tactical_ordering_summary_counted(
            state,
            scored_move.1,
            threat_view_mode,
            metrics,
            phase,
        );
        let (tactical_score, tactical_keep) =
            tactical_ordering_score_from_summary(summary, scored_move.4);
        if tactical_score > 0 || tactical_keep {
            scored_move.2 = tactical_score;
        }
        scored_move.3 |= tactical_keep;
    }

    scored.sort_by(|a, b| {
        b.2.cmp(&a.2)
            .then_with(|| b.5.cmp(&a.5))
            .then_with(|| a.0.cmp(&b.0))
    });
    scored
        .into_iter()
        .map(|(_, mv, _, must_keep, _, _, _)| OrderedMove { mv, must_keep })
        .collect()
}

fn hard_tactical_ordering_score(own_win: bool, immediate_block: bool) -> (i32, bool) {
    if own_win {
        (100_000, true)
    } else if immediate_block {
        (90_000, true)
    } else {
        (0, false)
    }
}

fn has_tactical_annotation_potential(board: &Board, player: Color, mv: Move) -> bool {
    let size = board.config.board_size;
    if size == 0 || mv.row >= size || mv.col >= size || !board.is_empty(mv.row, mv.col) {
        return false;
    }

    DIRS.iter()
        .any(|&(dr, dc)| axis_has_tactical_annotation_potential(board, player, mv, dr, dc))
}

fn has_tactical_annotation_potential_for_mode(
    state: &SearchState,
    player: Color,
    mv: Move,
    mode: ThreatViewMode,
    metrics: &mut SearchMetrics,
) -> bool {
    let board = state.board();
    match mode {
        ThreatViewMode::Scan => has_tactical_annotation_potential(board, player, mv),
        ThreatViewMode::Rolling => {
            let viability_mask = state
                .frontier
                .as_ref()
                .map(|frontier| frontier.viability_for(mv).mask_for(player))
                .unwrap_or_else(|| scan_cell_viability(board, mv).mask_for(player));
            has_tactical_annotation_potential_with_mask(board, player, mv, viability_mask)
        }
        ThreatViewMode::RollingShadow => {
            let scan = has_tactical_annotation_potential(board, player, mv);
            if let Some(frontier) = state.frontier.as_ref() {
                metrics.threat_view_shadow_checks += 1;
                let rolling = has_tactical_annotation_potential_with_mask(
                    board,
                    player,
                    mv,
                    frontier.viability_for(mv).mask_for(player),
                );
                if scan != rolling {
                    metrics.threat_view_shadow_mismatches += 1;
                }
            }
            scan
        }
    }
}

fn has_tactical_annotation_potential_with_mask(
    board: &Board,
    player: Color,
    mv: Move,
    viability_mask: u8,
) -> bool {
    let size = board.config.board_size;
    if viability_mask == 0
        || size == 0
        || mv.row >= size
        || mv.col >= size
        || !board.is_empty(mv.row, mv.col)
    {
        return false;
    }

    DIRS.iter()
        .enumerate()
        .filter(|(direction_index, _)| viability_mask & direction_bit(*direction_index) != 0)
        .any(|(_, &(dr, dc))| axis_has_tactical_annotation_potential(board, player, mv, dr, dc))
}

fn axis_has_tactical_annotation_potential(
    board: &Board,
    player: Color,
    mv: Move,
    dr: isize,
    dc: isize,
) -> bool {
    let size = board.config.board_size as isize;
    let row = mv.row as isize;
    let col = mv.col as isize;
    let opponent = player.opponent();

    for start in -4..=0 {
        let mut own_count = 1;
        let mut clean_window = true;

        for offset in start..start + 5 {
            let r = row + dr * offset;
            let c = col + dc * offset;
            if r < 0 || r >= size || c < 0 || c >= size {
                clean_window = false;
                break;
            }

            let r = r as usize;
            let c = c as usize;
            if r == mv.row && c == mv.col {
                continue;
            }
            if board.has_color(r, c, opponent) {
                clean_window = false;
                break;
            }
            if board.has_color(r, c, player) {
                own_count += 1;
            }
        }

        if clean_window && own_count >= 3 {
            return true;
        }
    }

    false
}

fn immediate_winning_move_mask_for_threat_view_mode(
    state: &mut SearchState,
    player: Color,
    mode: ThreatViewMode,
    metrics: &mut SearchMetrics,
) -> Vec<bool> {
    let size = state.board().config.board_size;
    let mut mask = vec![false; size * size];
    for mv in immediate_winning_moves_for_threat_view_mode(state, player, mode, metrics) {
        mask[mv.row * size + mv.col] = true;
    }
    mask
}

fn move_mask_contains(mask: &[bool], board_size: usize, mv: Move) -> bool {
    mv.row < board_size && mv.col < board_size && mask[mv.row * board_size + mv.col]
}

#[cfg(test)]
fn tactical_ordering_score(
    annotation: &TacticalMoveAnnotation,
    immediate_block: bool,
) -> (i32, bool) {
    let summary = SearchThreatPolicy.ordering_summary(annotation);
    tactical_ordering_score_from_summary(summary, immediate_block)
}

fn tactical_ordering_score_from_summary(
    summary: TacticalOrderingSummary,
    immediate_block: bool,
) -> (i32, bool) {
    let score = if immediate_block {
        summary.score.max(90_000)
    } else {
        summary.score
    };
    let must_keep = summary.must_keep || immediate_block;

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
}

impl SearchOutcome {
    fn new(score: i32, best_move: Option<Move>, timed_out: bool) -> Self {
        Self {
            score,
            best_move,
            timed_out,
        }
    }
}

const TERMINAL_SCORE_THRESHOLD: i32 = 1_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RootCandidateResult {
    mv: Move,
    score: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CorridorProofCandidate {
    mv: Move,
    rank: usize,
    score_gap: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateProofOutcome {
    ProvenWin,
    ProvenLoss,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CorridorProofCandidateResult {
    mv: Move,
    outcome: CandidateProofOutcome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CorridorProofDecisionReason {
    NoChange,
    ConfirmedWin,
    ChangedToWin,
    AvoidedLoss,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CorridorProofDecision {
    best_move: Move,
    reason: CorridorProofDecisionReason,
}

fn select_corridor_proof_candidates(
    root_results: &[RootCandidateResult],
    best_move: Move,
    max_candidates: usize,
) -> Vec<CorridorProofCandidate> {
    if max_candidates == 0 {
        return Vec::new();
    }

    let mut ranked = root_results.to_vec();
    ranked.sort_by_key(|result| std::cmp::Reverse(result.score));

    let Some(best_score) = ranked
        .iter()
        .find(|result| result.mv == best_move)
        .map(|result| result.score)
    else {
        return Vec::new();
    };

    let to_candidate = |rank: usize, result: RootCandidateResult| CorridorProofCandidate {
        mv: result.mv,
        rank,
        score_gap: best_score.saturating_sub(result.score).max(0) as u64,
    };

    let Some(best_candidate) = ranked
        .iter()
        .copied()
        .enumerate()
        .find(|(_, result)| result.mv == best_move)
        .map(|(index, result)| to_candidate(index + 1, result))
    else {
        return Vec::new();
    };

    let mut selected = Vec::with_capacity(max_candidates.min(root_results.len()));
    selected.push(best_candidate);
    for (index, result) in ranked.into_iter().enumerate() {
        if result.mv == best_move {
            continue;
        }
        let candidate = to_candidate(index + 1, result);
        if selected.len() >= max_candidates {
            break;
        }
        selected.push(candidate);
    }
    selected
}

fn resolve_corridor_proof_candidates(
    normal_best: Move,
    proofs: &[CorridorProofCandidateResult],
) -> CorridorProofDecision {
    if proofs
        .iter()
        .any(|proof| proof.mv == normal_best && proof.outcome == CandidateProofOutcome::ProvenWin)
    {
        return CorridorProofDecision {
            best_move: normal_best,
            reason: CorridorProofDecisionReason::ConfirmedWin,
        };
    }

    if let Some(proof) = proofs
        .iter()
        .find(|proof| proof.outcome == CandidateProofOutcome::ProvenWin)
    {
        return CorridorProofDecision {
            best_move: proof.mv,
            reason: CorridorProofDecisionReason::ChangedToWin,
        };
    }

    let normal_best_is_loss = proofs
        .iter()
        .any(|proof| proof.mv == normal_best && proof.outcome == CandidateProofOutcome::ProvenLoss);
    if normal_best_is_loss {
        if let Some(proof) = proofs.iter().find(|proof| {
            proof.mv != normal_best && proof.outcome != CandidateProofOutcome::ProvenLoss
        }) {
            return CorridorProofDecision {
                best_move: proof.mv,
                reason: CorridorProofDecisionReason::AvoidedLoss,
            };
        }
    }

    CorridorProofDecision {
        best_move: normal_best,
        reason: CorridorProofDecisionReason::NoChange,
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
fn run_corridor_proof_pass(
    board: &Board,
    root_color: Color,
    normal_best: Move,
    root_results: &[RootCandidateResult],
    corridor_proof: CorridorProofConfig,
    threat_view_mode: ThreatViewMode,
    zobrist: &ZobristTable,
    metrics: &mut SearchMetrics,
    deadline: SearchDeadline,
) -> CorridorProofDecision {
    let candidates = select_corridor_proof_candidates(
        root_results,
        normal_best,
        corridor_proof.proof_candidate_limit,
    );
    let mut proofs = Vec::with_capacity(candidates.len());

    for candidate in candidates {
        if deadline.expired() {
            metrics.corridor_proof_deadline_skips += 1;
            metrics.corridor_proof_deadline_exits += 1;
            break;
        }

        let mv = candidate.mv;
        metrics.corridor_proof_candidates_considered += 1;
        metrics.corridor_proof_candidate_rank_total += candidate.rank as u64;
        metrics.corridor_proof_candidate_rank_max = metrics
            .corridor_proof_candidate_rank_max
            .max(candidate.rank as u64);
        metrics.corridor_proof_candidate_score_gap_total += candidate.score_gap;
        metrics.corridor_proof_candidate_score_gap_max = metrics
            .corridor_proof_candidate_score_gap_max
            .max(candidate.score_gap);
        let outcome = prove_corridor_candidate(
            board,
            root_color,
            mv,
            corridor_proof,
            threat_view_mode,
            zobrist,
            metrics,
            deadline,
        );
        match outcome {
            CandidateProofOutcome::ProvenWin => {
                metrics.corridor_proof_wins += 1;
                metrics.corridor_proof_win_candidate_rank_total += candidate.rank as u64;
                metrics.corridor_proof_win_candidate_rank_max = metrics
                    .corridor_proof_win_candidate_rank_max
                    .max(candidate.rank as u64);
                metrics.corridor_proof_terminal_root_candidates += 1;
                metrics.corridor_proof_terminal_root_winning_candidates += 1;
            }
            CandidateProofOutcome::ProvenLoss => {
                metrics.corridor_proof_losses += 1;
                metrics.corridor_proof_terminal_root_candidates += 1;
                metrics.corridor_proof_terminal_root_losing_candidates += 1;
            }
            CandidateProofOutcome::Unknown => {
                metrics.corridor_proof_unknown += 1;
            }
        }

        proofs.push(CorridorProofCandidateResult { mv, outcome });
        if outcome == CandidateProofOutcome::ProvenWin {
            break;
        }
    }

    resolve_corridor_proof_candidates(normal_best, &proofs)
}

#[allow(clippy::too_many_arguments)]
fn prove_corridor_candidate(
    board: &Board,
    root_color: Color,
    mv: Move,
    corridor_proof: CorridorProofConfig,
    threat_view_mode: ThreatViewMode,
    zobrist: &ZobristTable,
    metrics: &mut SearchMetrics,
    deadline: SearchDeadline,
) -> CandidateProofOutcome {
    let mut state = SearchState::from_board_for_config(
        board.clone(),
        zobrist,
        threat_view_mode,
        StaticEvaluation::LineShapeEval,
        corridor_proof,
    );
    let result = state.apply_trusted_legal_move_counted(mv, zobrist, metrics);
    let outcome = match result {
        GameResult::Winner(winner) if winner == root_color => CandidateProofOutcome::ProvenWin,
        GameResult::Winner(_) => CandidateProofOutcome::ProvenLoss,
        GameResult::Draw => CandidateProofOutcome::Unknown,
        GameResult::Ongoing => {
            let color = state.board().current_player;
            if let Some(attacker) =
                corridor_proof_attacker(&mut state, color, threat_view_mode, metrics)
            {
                metrics.record_corridor_proof_check(true);
                let winner = prove_corridor_for_attacker(
                    &mut state,
                    color,
                    attacker,
                    CorridorSide::for_player(attacker, root_color),
                    corridor_proof,
                    0,
                    threat_view_mode,
                    zobrist,
                    metrics,
                    deadline,
                );
                match winner {
                    Some(winner) if winner == root_color => CandidateProofOutcome::ProvenWin,
                    Some(_) => CandidateProofOutcome::ProvenLoss,
                    None => CandidateProofOutcome::Unknown,
                }
            } else {
                metrics.record_corridor_proof_check(false);
                CandidateProofOutcome::Unknown
            }
        }
    };
    state.undo_move_counted(mv, metrics);
    outcome
}

fn corridor_proof_attacker(
    state: &mut SearchState,
    color: Color,
    threat_view_mode: ThreatViewMode,
    metrics: &mut SearchMetrics,
) -> Option<Color> {
    let opponent = color.opponent();
    if !immediate_winning_moves_for_threat_view_mode(state, color, threat_view_mode, metrics)
        .is_empty()
    {
        return Some(color);
    }
    if !immediate_winning_moves_for_threat_view_mode(state, opponent, threat_view_mode, metrics)
        .is_empty()
    {
        return Some(opponent);
    }
    if !narrow_corridor_reply_moves_for_threat_view_mode(state, opponent, threat_view_mode, metrics)
        .is_empty()
    {
        return Some(opponent);
    }
    if !narrow_corridor_reply_moves_for_threat_view_mode(state, color, threat_view_mode, metrics)
        .is_empty()
    {
        return Some(color);
    }
    None
}

#[allow(clippy::too_many_arguments)]
fn prove_corridor_for_attacker(
    state: &mut SearchState,
    color: Color,
    attacker: Color,
    side: CorridorSide,
    corridor_proof: CorridorProofConfig,
    depth_used: usize,
    threat_view_mode: ThreatViewMode,
    zobrist: &ZobristTable,
    metrics: &mut SearchMetrics,
    deadline: SearchDeadline,
) -> Option<Color> {
    metrics.record_corridor_node(depth_used as u32);

    if deadline.expired() {
        metrics.corridor_proof_deadline_exits += 1;
        return None;
    }

    if let GameResult::Winner(winner) = state.board().result {
        metrics.corridor_terminal_exits += 1;
        metrics.corridor_proof_terminal_exits += 1;
        return Some(winner);
    }
    if state.board().result == GameResult::Draw {
        metrics.corridor_neutral_exits += 1;
        metrics.corridor_proof_static_exits += 1;
        return None;
    }

    if depth_used >= corridor_proof.max_depth {
        metrics.corridor_depth_exits += 1;
        metrics.corridor_proof_depth_exits += 1;
        return None;
    }

    let moves = if color == attacker {
        materialized_attacker_corridor_moves_for_threat_view_mode(
            state,
            attacker,
            threat_view_mode,
            metrics,
        )
    } else {
        let replies = narrow_corridor_reply_moves_for_threat_view_mode(
            state,
            attacker,
            threat_view_mode,
            metrics,
        );
        if replies.len() > corridor_proof.max_reply_width {
            metrics.corridor_width_exits += 1;
            metrics.corridor_proof_static_exits += 1;
            return None;
        }
        if replies.is_empty()
            && !immediate_winning_moves_for_threat_view_mode(
                state,
                attacker,
                threat_view_mode,
                metrics,
            )
            .is_empty()
        {
            metrics.corridor_terminal_exits += 1;
            metrics.corridor_proof_terminal_exits += 1;
            return Some(attacker);
        }
        replies
    };

    if moves.is_empty() {
        metrics.corridor_neutral_exits += 1;
        metrics.corridor_proof_static_exits += 1;
        return None;
    }

    metrics.corridor_branch_probes += moves.len() as u64;
    if color == attacker {
        for mv in moves {
            if deadline.expired() {
                metrics.corridor_proof_deadline_exits += 1;
                return None;
            }
            state.apply_trusted_legal_move_counted(mv, zobrist, metrics);
            metrics.record_corridor_ply(side);
            let winner = prove_corridor_for_attacker(
                state,
                color.opponent(),
                attacker,
                side,
                corridor_proof,
                depth_used + 1,
                threat_view_mode,
                zobrist,
                metrics,
                deadline,
            );
            state.undo_move_counted(mv, metrics);
            if winner == Some(attacker) {
                return Some(attacker);
            }
        }
        None
    } else {
        for mv in moves {
            if deadline.expired() {
                metrics.corridor_proof_deadline_exits += 1;
                return None;
            }
            state.apply_trusted_legal_move_counted(mv, zobrist, metrics);
            metrics.record_corridor_ply(side);
            let winner = prove_corridor_for_attacker(
                state,
                color.opponent(),
                attacker,
                side,
                corridor_proof,
                depth_used + 1,
                threat_view_mode,
                zobrist,
                metrics,
                deadline,
            );
            state.undo_move_counted(mv, metrics);
            if winner != Some(attacker) {
                return None;
            }
        }
        Some(attacker)
    }
}

fn materialized_attacker_corridor_moves_for_threat_view_mode(
    state: &mut SearchState,
    attacker: Color,
    mode: ThreatViewMode,
    metrics: &mut SearchMetrics,
) -> Vec<Move> {
    match mode {
        ThreatViewMode::Scan => {
            scan_materialized_attacker_corridor_moves_timed(state.board(), attacker, metrics)
        }
        ThreatViewMode::Rolling => {
            rolling_materialized_attacker_corridor_moves(state, attacker, metrics)
        }
        ThreatViewMode::RollingShadow => {
            metrics.threat_view_shadow_checks += 1;
            let scan =
                scan_materialized_attacker_corridor_moves_timed(state.board(), attacker, metrics);
            let rolling = rolling_materialized_attacker_corridor_moves(state, attacker, metrics);
            if rolling != scan {
                metrics.threat_view_shadow_mismatches += 1;
            }
            scan
        }
    }
}

fn scan_materialized_attacker_corridor_moves_timed(
    board: &Board,
    attacker: Color,
    metrics: &mut SearchMetrics,
) -> Vec<Move> {
    let start = Instant::now();
    let moves = scan_materialized_attacker_corridor_moves(board, attacker);
    metrics.record_threat_view_scan(start.elapsed());
    moves
}

fn scan_materialized_attacker_corridor_moves(board: &Board, attacker: Color) -> Vec<Move> {
    if board.current_player != attacker || board.result != GameResult::Ongoing {
        return Vec::new();
    }

    let mut ranked = Vec::new();
    for mv in board.legal_moves() {
        let mut next = board.clone();
        if next.apply_move(mv).is_err() {
            continue;
        }
        let rank = match next.result {
            GameResult::Winner(winner) if winner == attacker => {
                CorridorThreatPolicy.rank(LocalThreatKind::Five)
            }
            GameResult::Winner(_) | GameResult::Draw => 0,
            GameResult::Ongoing => {
                ScanThreatView::new(&next).local_corridor_entry_rank(attacker, mv)
            }
        };

        if rank > 0 {
            ranked.push((mv, rank));
        }
    }

    highest_ranked_moves(ranked)
}

fn rolling_materialized_attacker_corridor_moves(
    state: &mut SearchState,
    attacker: Color,
    metrics: &mut SearchMetrics,
) -> Vec<Move> {
    if state.board().current_player != attacker || state.board().result != GameResult::Ongoing {
        return Vec::new();
    }

    let mut ranked = Vec::new();
    for mv in state.board().legal_moves() {
        let start = Instant::now();
        let rank = state
            .threat_view()
            .candidate_corridor_entry_rank(attacker, mv);
        metrics.record_threat_view_frontier_query(start.elapsed());
        if rank > 0 {
            ranked.push((mv, rank));
        }
    }

    highest_ranked_moves(ranked)
}

fn highest_ranked_moves(mut ranked: Vec<(Move, u8)>) -> Vec<Move> {
    let Some(best_rank) = ranked.iter().map(|(_, rank)| *rank).max() else {
        return Vec::new();
    };
    ranked.retain(|(_, rank)| *rank == best_rank);
    ranked.sort_by_key(|(mv, _)| (mv.row, mv.col));
    ranked.into_iter().map(|(mv, _)| mv).collect()
}

fn narrow_corridor_reply_moves_for_threat_view_mode(
    state: &mut SearchState,
    attacker: Color,
    mode: ThreatViewMode,
    metrics: &mut SearchMetrics,
) -> Vec<Move> {
    match mode {
        ThreatViewMode::Scan => {
            scan_narrow_corridor_reply_moves_timed(state.board(), attacker, metrics)
        }
        ThreatViewMode::Rolling => rolling_narrow_corridor_reply_moves(state, attacker, metrics),
        ThreatViewMode::RollingShadow => {
            metrics.threat_view_shadow_checks += 1;
            let scan = scan_narrow_corridor_reply_moves_timed(state.board(), attacker, metrics);
            let rolling = rolling_narrow_corridor_reply_moves(state, attacker, metrics);
            if rolling != scan {
                metrics.threat_view_shadow_mismatches += 1;
            }
            scan
        }
    }
}

fn immediate_winning_moves_for_threat_view_mode(
    state: &mut SearchState,
    player: Color,
    mode: ThreatViewMode,
    metrics: &mut SearchMetrics,
) -> Vec<Move> {
    match mode {
        ThreatViewMode::Scan => scan_immediate_winning_moves_timed(state.board(), player, metrics),
        ThreatViewMode::Rolling => rolling_immediate_winning_moves_timed(state, player, metrics),
        ThreatViewMode::RollingShadow => {
            metrics.threat_view_shadow_checks += 1;
            let scan = scan_immediate_winning_moves_timed(state.board(), player, metrics);
            let rolling = rolling_immediate_winning_moves_timed(state, player, metrics);
            if rolling != scan {
                metrics.threat_view_shadow_mismatches += 1;
            }
            scan
        }
    }
}

fn scan_immediate_winning_moves_timed(
    board: &Board,
    player: Color,
    metrics: &mut SearchMetrics,
) -> Vec<Move> {
    let renju_before = renju_forbidden_metrics_snapshot();
    let start = Instant::now();
    let moves = board.immediate_winning_moves_for(player);
    metrics.record_threat_view_scan(start.elapsed());
    metrics.record_renju_forbidden_source_delta(RenjuForbiddenMetricSource::Threat, renju_before);
    moves
}

fn rolling_immediate_winning_moves_timed(
    state: &mut SearchState,
    player: Color,
    metrics: &mut SearchMetrics,
) -> Vec<Move> {
    let renju_before = renju_forbidden_metrics_snapshot();
    let start = Instant::now();
    let moves = state
        .threat_view_mut()
        .immediate_winning_moves_for_cached(player);
    metrics.record_threat_view_frontier_immediate_win_query(start.elapsed());
    metrics.record_renju_forbidden_source_delta(RenjuForbiddenMetricSource::Threat, renju_before);
    moves
}

fn scan_narrow_corridor_reply_moves_timed(
    board: &Board,
    attacker: Color,
    metrics: &mut SearchMetrics,
) -> Vec<Move> {
    let renju_before = renju_forbidden_metrics_snapshot();
    let start = Instant::now();
    let moves = corridor::narrow_corridor_reply_moves(board, attacker);
    metrics.record_threat_view_scan(start.elapsed());
    metrics.record_renju_forbidden_source_delta(RenjuForbiddenMetricSource::Threat, renju_before);
    moves
}

fn rolling_narrow_corridor_reply_moves(
    state: &mut SearchState,
    attacker: Color,
    metrics: &mut SearchMetrics,
) -> Vec<Move> {
    let defender = attacker.opponent();
    let winning_squares = rolling_immediate_winning_moves_timed(state, attacker, metrics);
    if !winning_squares.is_empty() {
        let mut replies = Vec::new();
        for mv in winning_squares {
            let renju_before = renju_forbidden_metrics_snapshot();
            if state.board().is_legal_for_color(mv, defender) {
                push_unique_move(&mut replies, mv);
            }
            metrics.record_renju_forbidden_source_delta(
                RenjuForbiddenMetricSource::Threat,
                renju_before,
            );
        }
        for mv in rolling_immediate_winning_moves_timed(state, defender, metrics) {
            push_unique_move(&mut replies, mv);
        }
        return replies;
    }

    let renju_before = renju_forbidden_metrics_snapshot();
    let start = Instant::now();
    let replies = state.threat_view().defender_reply_moves(attacker, None);
    metrics.record_threat_view_frontier_query(start.elapsed());
    metrics.record_renju_forbidden_source_delta(RenjuForbiddenMetricSource::Threat, renju_before);
    replies
}

fn push_unique_move(moves: &mut Vec<Move>, mv: Move) {
    if !moves.contains(&mv) {
        moves.push(mv);
    }
}

// --- Negamax with alpha-beta (incremental Zobrist hash) ---

fn store_tt_entry(
    tt: &mut HashMap<u64, TTEntry>,
    max_tt_entries: Option<usize>,
    metrics: &mut SearchMetrics,
    hash: u64,
    entry: TTEntry,
) {
    let can_insert =
        tt.contains_key(&hash) || max_tt_entries.map_or(true, |limit| tt.len() < limit);
    if can_insert {
        tt.insert(hash, entry);
    } else {
        metrics.tt_insert_skips += 1;
    }
}

#[allow(clippy::too_many_arguments)]
fn negamax(
    state: &mut SearchState,
    depth: i32,
    mut alpha: i32,
    beta: i32,
    color: Color,
    root_color: Color,
    tt: &mut HashMap<u64, TTEntry>,
    max_tt_entries: Option<usize>,
    zobrist: &ZobristTable,
    candidate_source: CandidateSource,
    null_cell_culling: NullCellCulling,
    legality_gate: LegalityGate,
    move_ordering: MoveOrdering,
    child_limit: Option<usize>,
    threat_view_mode: ThreatViewMode,
    static_eval: StaticEvaluation,
    nodes: &mut u64,
    metrics: &mut SearchMetrics,
    deadline: SearchDeadline,
) -> SearchOutcome {
    *nodes += 1;
    let hash = state.hash();

    if deadline.expired() {
        return SearchOutcome::new(
            evaluate_leaf_counted(state, color, root_color, static_eval, metrics),
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

    if state.board().result != GameResult::Ongoing {
        return SearchOutcome::new(
            evaluate_leaf_counted(state, color, root_color, static_eval, metrics),
            None,
            false,
        );
    }

    if depth == 0 {
        return SearchOutcome::new(
            evaluate_leaf_counted(state, color, root_color, static_eval, metrics),
            None,
            false,
        );
    }

    let mut moves = candidate_moves_from_source_counted(
        state.board(),
        candidate_source,
        metrics,
        SearchMetricPhase::Search,
    );
    moves = cull_null_cells_counted(
        state.board(),
        state.frontier.as_ref(),
        moves,
        null_cell_culling,
        threat_view_mode,
        metrics,
        SearchMetricPhase::Search,
    );
    let mut needs_legality_check = needs_legality_gate(state.board(), color, legality_gate);
    if (matches!(
        move_ordering,
        MoveOrdering::TacticalFull | MoveOrdering::Tactical
    ) || child_limit.is_some())
        && needs_legality_check
    {
        moves.retain(|&mv| {
            legal_by_gate_counted(
                state.board(),
                mv,
                legality_gate,
                metrics,
                SearchMetricPhase::Search,
            )
        });
        needs_legality_check = false;
    }
    if moves.is_empty() {
        return SearchOutcome::new(
            evaluate_leaf_counted(state, color, root_color, static_eval, metrics),
            None,
            false,
        );
    }

    let orig_alpha = alpha;
    let mut best_score = i32::MIN + 1;
    let mut best_move: Option<Move> = None;

    let tt_move = tt.get(&hash).and_then(|e| e.best_move);
    let ordered = order_search_moves(
        state,
        moves,
        move_ordering,
        tt_move,
        child_limit,
        threat_view_mode,
        metrics,
    );

    let mut timed_out = false;
    for mv in ordered {
        if deadline.expired() {
            timed_out = true;
            break;
        }
        if needs_legality_check
            && !legal_by_gate_counted(
                state.board(),
                mv,
                legality_gate,
                metrics,
                SearchMetricPhase::Search,
            )
        {
            continue;
        }
        state.apply_trusted_legal_move_counted(mv, zobrist, metrics);
        let child_outcome = negamax(
            state,
            depth - 1,
            -beta,
            -alpha,
            color.opponent(),
            root_color,
            tt,
            max_tt_entries,
            zobrist,
            candidate_source,
            null_cell_culling,
            legality_gate,
            move_ordering,
            child_limit,
            threat_view_mode,
            static_eval,
            nodes,
            metrics,
            deadline,
        );
        let score = -child_outcome.score;
        state.undo_move_counted(mv, metrics);

        if child_outcome.timed_out {
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
        return SearchOutcome::new(
            evaluate_leaf_counted(state, color, root_color, static_eval, metrics),
            None,
            timed_out,
        );
    }

    if timed_out {
        return SearchOutcome {
            score: best_score,
            best_move,
            timed_out: true,
        };
    }

    let flag = if best_score <= orig_alpha {
        TTFlag::UpperBound
    } else if best_score >= beta {
        TTFlag::LowerBound
    } else {
        TTFlag::Exact
    };
    store_tt_entry(
        tt,
        max_tt_entries,
        metrics,
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
    }
}

#[allow(clippy::too_many_arguments)]
fn search_root(
    state: &mut SearchState,
    depth: i32,
    root_moves: &[Move],
    color: Color,
    tt: &mut HashMap<u64, TTEntry>,
    max_tt_entries: Option<usize>,
    zobrist: &ZobristTable,
    candidate_source: CandidateSource,
    null_cell_culling: NullCellCulling,
    legality_gate: LegalityGate,
    move_ordering: MoveOrdering,
    child_limit: Option<usize>,
    threat_view_mode: ThreatViewMode,
    static_eval: StaticEvaluation,
    nodes: &mut u64,
    metrics: &mut SearchMetrics,
    deadline: SearchDeadline,
    mut root_results: Option<&mut Vec<RootCandidateResult>>,
) -> SearchOutcome {
    *nodes += 1;
    let hash = state.hash();
    if deadline.expired() {
        return SearchOutcome::new(
            evaluate_state_counted(state, color, static_eval, metrics),
            None,
            true,
        );
    }

    let mut alpha = i32::MIN + 1;
    let beta = i32::MAX;
    let orig_alpha = alpha;
    let mut best_score = i32::MIN + 1;
    let mut best_move: Option<Move> = None;

    let tt_move = tt.get(&hash).and_then(|entry| entry.best_move);
    let ordered = order_root_moves(
        state,
        root_moves.to_vec(),
        move_ordering,
        tt_move,
        threat_view_mode,
        metrics,
    );

    let mut timed_out = false;
    for mv in ordered {
        if deadline.expired() {
            timed_out = true;
            break;
        }

        state.apply_trusted_legal_move_counted(mv, zobrist, metrics);
        let child_outcome = negamax(
            state,
            depth - 1,
            -beta,
            -alpha,
            color.opponent(),
            color,
            tt,
            max_tt_entries,
            zobrist,
            candidate_source,
            null_cell_culling,
            legality_gate,
            move_ordering,
            child_limit,
            threat_view_mode,
            static_eval,
            nodes,
            metrics,
            deadline,
        );
        let score = -child_outcome.score;
        state.undo_move_counted(mv, metrics);

        if let Some(results) = root_results.as_deref_mut() {
            results.push(RootCandidateResult { mv, score });
        }

        if child_outcome.timed_out {
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
        return SearchOutcome::new(
            evaluate_state_counted(state, color, static_eval, metrics),
            None,
            timed_out,
        );
    }

    if timed_out {
        return SearchOutcome {
            score: best_score,
            best_move,
            timed_out: true,
        };
    }

    let flag = if best_score <= orig_alpha {
        TTFlag::UpperBound
    } else if best_score >= beta {
        TTFlag::LowerBound
    } else {
        TTFlag::Exact
    };
    store_tt_entry(
        tt,
        max_tt_entries,
        metrics,
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
    }
}

// --- SearchBot ---

mod config;

use config::CorridorSide;
pub use config::{
    CandidateSource, CorridorProofConfig, LegalityGate, MoveOrdering, NullCellCulling, SafetyGate,
    SearchAlgorithm, SearchBotConfig, StaticEvaluation, ThreatViewMode,
};

#[derive(Debug, Clone)]
pub struct SearchInfo {
    pub depth_reached: i32,
    pub nodes: u64,
    pub safety_nodes: u64,
    pub metrics: SearchMetrics,
    pub score: i32,
    pub budget_exhausted: bool,
    pub elapsed_ms: u64,
    pub cpu_time_ms: Option<u64>,
    pub tt_entries: usize,
    pub tt_max_entries: Option<usize>,
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

    pub fn set_time_budgets(
        &mut self,
        time_budget_ms: Option<u64>,
        cpu_time_budget_ms: Option<u64>,
    ) {
        self.config.time_budget_ms = time_budget_ms;
        self.config.cpu_time_budget_ms = cpu_time_budget_ms;
    }
}

impl Bot for SearchBot {
    fn name(&self) -> &str {
        "baseline"
    }

    fn trace(&self) -> Option<serde_json::Value> {
        self.last_info.as_ref().map(|info| {
            let total_nodes = info.nodes + info.safety_nodes + info.metrics.corridor_nodes;
            serde_json::json!({
                "config": self.config.trace(),
                "depth": info.depth_reached,
                "nominal_depth": self.config.max_depth,
                "effective_depth": info.depth_reached,
                "nodes": info.nodes,
                "safety_nodes": info.safety_nodes,
                "corridor": {
                    "search_nodes": info.metrics.corridor_nodes,
                    "branch_probes": info.metrics.corridor_branch_probes,
                    "max_depth_reached": info.metrics.corridor_max_depth,
                    "width_exits": info.metrics.corridor_width_exits,
                    "depth_exits": info.metrics.corridor_depth_exits,
                    "neutral_exits": info.metrics.corridor_neutral_exits,
                    "terminal_exits": info.metrics.corridor_terminal_exits,
                },
                "tt": {
                    "entries": info.tt_entries,
                    "max_entries": info.tt_max_entries,
                    "insert_skips": info.metrics.tt_insert_skips,
                },
                "total_nodes": total_nodes,
                "metrics": info.metrics.trace(),
                "score": info.score,
                "budget_exhausted": info.budget_exhausted,
                "elapsed_ms": info.elapsed_ms,
                "cpu_time_ms": info.cpu_time_ms,
            })
        })
    }

    fn choose_move(&mut self, board: &Board) -> Move {
        let color = board.current_player;
        let mut metrics = SearchMetrics::default();
        let renju_metrics_before = renju_forbidden_metrics_snapshot();
        let tactical_metrics_before = tactical_metrics_snapshot();
        let start = Instant::now();
        let time_budget = self.config.time_budget();
        let cpu_time_budget = self.config.cpu_time_budget();
        let cpu_start = cpu_time_budget.and_then(|_| thread_cpu_time());
        let deadline = SearchDeadline::new(start, time_budget, cpu_start, cpu_time_budget);
        let center = board.config.board_size / 2;
        let candidate_source = self.config.candidate_source();
        let null_cell_culling = self.config.null_cell_culling;
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
            RootCandidateOptions {
                candidate_source,
                null_cell_culling,
                legality_gate,
                safety_gate,
                threat_view_mode: self.config.threat_view_mode,
                deadline: safety_deadline,
            },
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
        let mut completed_root_results = Vec::new();
        let mut state = SearchState::from_board_for_config(
            board.clone(),
            &self.zobrist,
            self.config.threat_view_mode,
            self.config.static_eval,
            CorridorProofConfig::DISABLED,
        );

        for depth in 1..=self.config.max_depth {
            if deadline.expired() {
                budget_exhausted = true;
                break;
            }
            let mut nodes = 0u64;
            let mut iteration_root_results = Vec::new();
            let outcome = search_root(
                &mut state,
                depth,
                &root_moves,
                color,
                &mut self.tt,
                self.config.max_tt_entries,
                &self.zobrist,
                candidate_source,
                null_cell_culling,
                legality_gate,
                move_ordering,
                self.config.child_limit,
                self.config.threat_view_mode,
                self.config.static_eval,
                &mut nodes,
                &mut metrics,
                deadline,
                Some(&mut iteration_root_results),
            );
            debug_assert_eq!(
                state.hash(),
                board.hash_with(&self.zobrist),
                "search state hash should return to root after each depth"
            );
            total_nodes += nodes;

            if !outcome.timed_out {
                if let Some(m) = outcome.best_move {
                    best_move = m;
                    best_score = outcome.score;
                }
                depth_reached = depth;
                completed_root_results = iteration_root_results;
            } else if depth_reached == 0 {
                if let Some(m) = outcome.best_move {
                    best_move = m;
                    best_score = outcome.score;
                }
            }

            if outcome.timed_out {
                budget_exhausted = true;
                break;
            }
            // Early exit on forced win/loss
            if best_score.abs() >= TERMINAL_SCORE_THRESHOLD {
                break;
            }
        }

        if self.config.corridor_proof.enabled
            && depth_reached == self.config.max_depth
            && !deadline.expired()
            && board.result == GameResult::Ongoing
        {
            metrics.corridor_proof_passes += 1;
            let stage_before = metrics.stage_snapshot();
            let proof_start = Instant::now();
            let decision = run_corridor_proof_pass(
                board,
                color,
                best_move,
                &completed_root_results,
                self.config.corridor_proof,
                self.config.threat_view_mode,
                &self.zobrist,
                &mut metrics,
                deadline,
            );
            metrics.record_proof_scope(proof_start.elapsed(), stage_before);

            match decision.reason {
                CorridorProofDecisionReason::NoChange => {}
                CorridorProofDecisionReason::ConfirmedWin => {
                    metrics.corridor_proof_completed += 1;
                    metrics.corridor_proof_move_confirmations += 1;
                    metrics.corridor_proof_terminal_root_overrides += 1;
                    metrics.corridor_proof_terminal_root_move_confirmations += 1;
                    best_score = terminal_score_for_winner(color, color, color);
                }
                CorridorProofDecisionReason::ChangedToWin => {
                    metrics.corridor_proof_completed += 1;
                    metrics.corridor_proof_move_changes += 1;
                    metrics.corridor_proof_terminal_root_overrides += 1;
                    metrics.corridor_proof_terminal_root_move_changes += 1;
                    best_move = decision.best_move;
                    best_score = terminal_score_for_winner(color, color, color);
                }
                CorridorProofDecisionReason::AvoidedLoss => {
                    metrics.corridor_proof_completed += 1;
                    metrics.corridor_proof_move_changes += 1;
                    best_move = decision.best_move;
                    if let Some(result) = completed_root_results
                        .iter()
                        .find(|result| result.mv == best_move)
                    {
                        best_score = result.score;
                    }
                }
            }

            if deadline.expired() {
                budget_exhausted = true;
            } else if decision.reason == CorridorProofDecisionReason::NoChange {
                metrics.corridor_proof_completed += 1;
            }
        }

        metrics.record_renju_forbidden_total_delta(renju_metrics_before);
        metrics.record_tactical_metric_total_delta(tactical_metrics_before);
        self.last_info = Some(SearchInfo {
            depth_reached,
            nodes: total_nodes,
            safety_nodes,
            metrics,
            score: best_score,
            budget_exhausted,
            elapsed_ms: start.elapsed().as_millis() as u64,
            cpu_time_ms: cpu_start.and_then(|start| {
                thread_cpu_time().map(|now| now.saturating_sub(start).as_millis() as u64)
            }),
            tt_entries: self.tt.len(),
            tt_max_entries: self.config.max_tt_entries,
        });

        best_move
    }
}

#[cfg(test)]
#[path = "../../../benchmarks/scenarios.rs"]
mod scenarios;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tactical::ScanThreatView;
    use gomoku_core::RuleConfig;

    fn mv(notation: &str) -> Move {
        Move::from_notation(notation).unwrap()
    }

    fn apply_moves(board: &mut Board, moves: &[&str]) {
        for &notation in moves {
            board.apply_move(mv(notation)).unwrap();
        }
    }

    fn apply_cell_moves(board: &mut Board, cells: &[usize]) {
        let size = board.config.board_size;
        for &cell in cells {
            board
                .apply_move(Move {
                    row: cell / size,
                    col: cell % size,
                })
                .unwrap();
        }
    }

    fn root_candidate_test_options(
        safety_gate: SafetyGate,
        deadline: SearchDeadline,
    ) -> RootCandidateOptions {
        RootCandidateOptions {
            candidate_source: CandidateSource::NearAll { radius: 2 },
            null_cell_culling: NullCellCulling::Disabled,
            legality_gate: LegalityGate::ExactRules,
            safety_gate,
            threat_view_mode: ThreatViewMode::Scan,
            deadline,
        }
    }

    struct SearchBehaviorCase {
        id: &'static str,
        scenario_id: &'static str,
        config_id: &'static str,
        expected_moves: &'static [&'static str],
        description: &'static str,
    }

    impl SearchBehaviorCase {
        fn scenario(&self) -> &'static scenarios::BenchScenario {
            scenarios::SCENARIOS
                .iter()
                .find(|scenario| scenario.id == self.scenario_id)
                .unwrap_or_else(|| {
                    panic!(
                        "behavior case '{}' references unknown scenario '{}'",
                        self.id, self.scenario_id
                    )
                })
        }

        fn expected_moves(&self) -> Vec<Move> {
            self.expected_moves
                .iter()
                .map(|notation| mv(notation))
                .collect()
        }
    }

    const SEARCH_BEHAVIOR_CASES: &[SearchBehaviorCase] = &[
        SearchBehaviorCase {
            id: "balanced_completes_open_four",
            scenario_id: "local_complete_open_four",
            config_id: "balanced",
            expected_moves: &["G8", "L8"],
            description: "Balanced should finish its own open four.",
        },
        SearchBehaviorCase {
            id: "balanced_reacts_closed_four",
            scenario_id: "local_react_closed_four",
            config_id: "balanced",
            expected_moves: &["E1"],
            description: "Balanced should answer the opponent's closed four.",
        },
        SearchBehaviorCase {
            id: "balanced_prevents_open_four_over_extending_three",
            scenario_id: "priority_prevent_open_four_over_extend_three",
            config_id: "balanced",
            expected_moves: &["G8", "K8"],
            description:
                "Balanced should prevent the opponent's open three instead of extending elsewhere.",
        },
        SearchBehaviorCase {
            id: "balanced_completes_four_before_reacting",
            scenario_id: "priority_complete_open_four_over_react_closed_four",
            config_id: "balanced",
            expected_moves: &["G8", "L8"],
            description: "Balanced should complete an open four when both sides threaten.",
        },
    ];

    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
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
            double_threat: immediate_wins_after >= 2
                && immediate_wins_after > immediate_wins_before,
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum ForcedLineKind {
        ImmediateWin,
        ForcedBlock,
        UnblockableImmediateLoss,
        OpponentMultiThreat,
        Quiet,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct ForcedLineState {
        player: Color,
        kind: ForcedLineKind,
        immediate_wins: Vec<Move>,
        opponent_wins: Vec<Move>,
        legal_blocks: Vec<Move>,
    }

    impl ForcedLineState {
        fn forced_block(&self) -> Option<Move> {
            if self.kind == ForcedLineKind::ForcedBlock {
                self.legal_blocks.first().copied()
            } else {
                None
            }
        }
    }

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
    enum ThreatAfterMoveKind {
        Illegal,
        WinsNow,
        SingleThreat,
        MultiThreat,
        Quiet,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct ThreatAfterMoveState {
        player: Color,
        kind: ThreatAfterMoveKind,
        winning_replies: Vec<Move>,
    }

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

    fn annotate_tactical_move(board: &Board, mv: Move) -> TacticalMoveAnnotation {
        SearchThreatPolicy.annotation_for_move(board, mv)
    }

    fn in_bounds(board: &Board, row: isize, col: isize) -> bool {
        let size = board.config.board_size as isize;
        row >= 0 && row < size && col >= 0 && col < size
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
    fn search_state_apply_undo_restores_board_hash_and_frontier() {
        let mut board = Board::new(RuleConfig {
            variant: Variant::Renju,
            ..RuleConfig::default()
        });
        apply_moves(&mut board, &["H8", "A1", "I8", "A2", "J8", "A3"]);
        let zobrist = ZobristTable::new(board.config.board_size);
        let original_fen = board.to_fen();
        let original_hash = board.hash_with(&zobrist);
        let mut state = SearchState::from_board(board, &zobrist);

        let played = mv("K8");
        state.apply_trusted_legal_move(played, &zobrist);

        assert_eq!(state.hash(), state.board().hash_with(&zobrist));
        assert!(state
            .threat_view()
            .has_move_local_corridor_entry(Color::Black, played));

        state.undo_move(played);

        assert_eq!(state.board().to_fen(), original_fen);
        assert_eq!(state.hash(), original_hash);
        assert_eq!(state.hash(), state.board().hash_with(&zobrist));
        assert_eq!(
            state.threat_view().active_corridor_threats(Color::Black),
            ScanThreatView::new(state.board()).active_corridor_threats(Color::Black)
        );
    }

    #[test]
    fn search_state_nested_apply_undo_keeps_frontier_in_sync_with_scan_view() {
        let mut board = Board::new(RuleConfig::default());
        apply_moves(&mut board, &["H8", "A1", "I8", "A2", "J8", "A3"]);
        let zobrist = ZobristTable::new(board.config.board_size);
        let mut state = SearchState::from_board(board, &zobrist);

        for played in [mv("K8"), mv("A4"), mv("L8")] {
            state.apply_trusted_legal_move(played, &zobrist);
            assert_eq!(state.hash(), state.board().hash_with(&zobrist));
            assert_eq!(
                state.threat_view().active_corridor_threats(Color::Black),
                ScanThreatView::new(state.board()).active_corridor_threats(Color::Black)
            );
        }

        for played in [mv("L8"), mv("A4"), mv("K8")] {
            state.undo_move(played);
            assert_eq!(state.hash(), state.board().hash_with(&zobrist));
            assert_eq!(
                state.threat_view().active_corridor_threats(Color::Black),
                ScanThreatView::new(state.board()).active_corridor_threats(Color::Black)
            );
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
    fn safety_gate_current_obligation_falls_back_to_unfiltered_moves_when_deadline_has_elapsed() {
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
            root_candidate_test_options(
                SafetyGate::CurrentObligation,
                SearchDeadline::new(
                    Instant::now() - Duration::from_millis(2),
                    Some(Duration::from_millis(1)),
                    None,
                    None,
                ),
            ),
            &mut metrics,
        );

        let (moves, _, timed_out) = moves;
        assert_eq!(moves, expected);
        assert!(timed_out);
    }

    #[test]
    fn safety_gate_none_skips_current_obligation_filter() {
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
            root_candidate_test_options(
                SafetyGate::None,
                SearchDeadline::new(Instant::now(), Some(Duration::from_millis(100)), None, None),
            ),
            &mut metrics,
        );

        assert_eq!(moves, expected);
        assert_eq!(safety_nodes, 0);
        assert!(!timed_out);
    }

    #[test]
    fn safety_gate_current_obligation_filters_existing_open_three_obligations() {
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
            root_candidate_test_options(
                SafetyGate::CurrentObligation,
                SearchDeadline::new(Instant::now(), Some(Duration::from_millis(100)), None, None),
            ),
            &mut metrics,
        );

        assert!(moves.contains(&Move { row: 7, col: 6 }));
        assert!(moves.contains(&Move { row: 7, col: 10 }));
        assert!(!moves.contains(&Move { row: 4, col: 4 }));
        assert!(safety_nodes > 0);
        assert!(!timed_out);
    }

    #[test]
    fn safety_gate_current_obligation_prefers_own_win_over_defense() {
        let mut board = Board::new(RuleConfig::default());
        apply_moves(
            &mut board,
            &["H8", "A1", "I8", "B1", "J8", "C1", "K8", "D1"],
        );
        assert_eq!(board.current_player, Color::Black);

        let mut metrics = SearchMetrics::default();
        let (moves, safety_nodes, timed_out) = root_candidate_moves_with_metrics(
            &board,
            root_candidate_test_options(
                SafetyGate::CurrentObligation,
                SearchDeadline::new(Instant::now(), Some(Duration::from_millis(100)), None, None),
            ),
            &mut metrics,
        );

        assert_eq!(moves, vec![mv("G8"), mv("L8")]);
        assert!(safety_nodes > 0);
        assert!(!timed_out);
    }

    #[test]
    fn safety_gate_current_obligation_allows_counter_fours_against_imminent_threat() {
        let mut board = Board::new(RuleConfig::default());
        apply_moves(&mut board, &["H8", "C4", "I8", "D4", "J8", "E4", "A15"]);
        assert_eq!(board.current_player, Color::White);

        let mut metrics = SearchMetrics::default();
        let (moves, safety_nodes, timed_out) = root_candidate_moves_with_metrics(
            &board,
            root_candidate_test_options(
                SafetyGate::CurrentObligation,
                SearchDeadline::new(Instant::now(), Some(Duration::from_millis(100)), None, None),
            ),
            &mut metrics,
        );

        assert!(moves.contains(&mv("G8")));
        assert!(moves.contains(&mv("K8")));
        assert!(moves.contains(&mv("B4")));
        assert!(moves.contains(&mv("F4")));
        assert!(!moves.contains(&mv("A14")));
        assert!(safety_nodes > 0);
        assert!(!timed_out);
    }

    #[test]
    fn safety_gate_current_obligation_leaves_quiet_root_candidates_unchanged() {
        let mut board = Board::new(RuleConfig::default());
        apply_moves(&mut board, &["H8", "H7", "G8", "I8"]);
        let expected: Vec<Move> = candidate_moves(&board, 2)
            .into_iter()
            .filter(|&mv| board.is_legal(mv))
            .collect();

        let mut metrics = SearchMetrics::default();
        let (moves, safety_nodes, timed_out) = root_candidate_moves_with_metrics(
            &board,
            root_candidate_test_options(
                SafetyGate::CurrentObligation,
                SearchDeadline::new(Instant::now(), Some(Duration::from_millis(100)), None, None),
            ),
            &mut metrics,
        );

        assert_eq!(moves, expected);
        assert_eq!(safety_nodes, 0);
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
        let zobrist = ZobristTable::new(win_board.config.board_size);
        let mut state = SearchState::from_board_with_frontier(win_board, &zobrist, false);
        let ordered = order_moves_tactical_full(
            &mut state,
            vec![mv("B2"), mv("E1"), mv("L8")],
            None,
            ThreatViewMode::Scan,
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
        let zobrist = ZobristTable::new(shape_board.config.board_size);
        let mut state = SearchState::from_board_with_frontier(shape_board, &zobrist, false);
        let ordered = order_moves_tactical_full(
            &mut state,
            vec![mv("B2"), mv("K8"), mv("E1")],
            None,
            ThreatViewMode::Scan,
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
    fn tactical_ordering_uses_one_opponent_win_query_for_blocks() {
        let mut board = Board::new(RuleConfig::default());
        apply_moves(
            &mut board,
            &["H8", "A1", "I8", "B1", "J8", "C1", "O15", "D1"],
        );

        let moves = vec![mv("B2"), mv("K8"), mv("E1")];
        let zobrist = ZobristTable::new(board.config.board_size);

        let mut scan_metrics = SearchMetrics::default();
        let mut scan_state = SearchState::from_board_with_frontier(board.clone(), &zobrist, false);
        let scan_ordered = order_moves_tactical_full(
            &mut scan_state,
            moves.clone(),
            None,
            ThreatViewMode::Scan,
            &mut scan_metrics,
            SearchMetricPhase::Search,
        );

        assert_eq!(
            scan_ordered
                .iter()
                .map(|ordered| ordered.mv)
                .collect::<Vec<_>>(),
            vec![mv("E1"), mv("K8"), mv("B2")]
        );
        assert_eq!(scan_metrics.search_tactical_annotations, moves.len() as u64);
        assert_eq!(
            scan_metrics.threat_view_scan_queries,
            moves.len() as u64 + 1,
            "scan ordering should query own annotations once per move and opponent wins once per ordering pass"
        );

        let mut rolling_metrics = SearchMetrics::default();
        let mut rolling_state = SearchState::from_board_with_frontier(board, &zobrist, true);
        let rolling_ordered = order_moves_tactical_full(
            &mut rolling_state,
            moves.clone(),
            None,
            ThreatViewMode::Rolling,
            &mut rolling_metrics,
            SearchMetricPhase::Search,
        );

        assert_eq!(rolling_ordered, scan_ordered);
        assert_eq!(
            rolling_metrics.search_tactical_annotations,
            moves.len() as u64
        );
        assert_eq!(
            rolling_metrics.threat_view_frontier_immediate_win_queries,
            1
        );
        assert_eq!(
            rolling_metrics.threat_view_frontier_queries,
            moves.len() as u64 + 1,
            "rolling ordering should query own annotations once per move and opponent wins once per ordering pass"
        );
    }

    #[test]
    fn tactical_without_child_cap_matches_full_tactical_ordering() {
        let mut board = Board::new(RuleConfig::default());
        apply_moves(
            &mut board,
            &["H8", "A1", "I8", "B1", "J8", "C1", "O15", "D1"],
        );
        let moves = vec![mv("B2"), mv("K8"), mv("E1"), mv("H9")];
        let zobrist = ZobristTable::new(board.config.board_size);

        let mut full_metrics = SearchMetrics::default();
        let mut full_state = SearchState::from_board_with_frontier(board.clone(), &zobrist, false);
        let full = order_moves_tactical_full(
            &mut full_state,
            moves.clone(),
            None,
            ThreatViewMode::Scan,
            &mut full_metrics,
            SearchMetricPhase::Search,
        );

        let mut staged_metrics = SearchMetrics::default();
        let mut staged_state = SearchState::from_board_with_frontier(board, &zobrist, false);
        let staged = order_moves_tactical(
            &mut staged_state,
            moves,
            None,
            None,
            ThreatViewMode::Scan,
            &mut staged_metrics,
            SearchMetricPhase::Search,
        );

        assert_eq!(staged, full);
        assert_eq!(
            staged_metrics.search_tactical_annotations,
            full_metrics.search_tactical_annotations
        );
    }

    #[test]
    fn tactical_annotates_tactical_potential_and_preserves_hard_blocks() {
        let mut board = Board::new(RuleConfig::default());
        apply_moves(
            &mut board,
            &["H8", "A1", "I8", "B1", "J8", "C1", "O15", "D1"],
        );
        let moves = vec![mv("B2"), mv("K8"), mv("E1"), mv("H9")];
        let zobrist = ZobristTable::new(board.config.board_size);
        let mut state = SearchState::from_board_with_frontier(board, &zobrist, false);
        let mut metrics = SearchMetrics::default();

        let ordered = order_moves_tactical(
            &mut state,
            moves,
            None,
            Some(1),
            ThreatViewMode::Scan,
            &mut metrics,
            SearchMetricPhase::Search,
        );
        let capped = apply_child_limit(ordered, Some(1), &mut metrics, SearchMetricPhase::Search);

        assert_eq!(capped.first().copied(), Some(mv("E1")));
        assert!(capped.contains(&mv("E1")));
        assert_eq!(
            metrics.search_tactical_annotations, 2,
            "tactical should annotate hard tactics and tactical-potential moves, not every child"
        );
    }

    #[test]
    fn tactical_annotation_potential_keeps_full_tactical_hits_on_benchmark_candidates() {
        for scenario in scenarios::SCENARIOS {
            let board = scenario.board();
            let player = board.current_player;
            let own_wins = board.immediate_winning_moves_for(player);
            let opponent_wins = board.immediate_winning_moves_for(player.opponent());

            for mv in candidate_moves(&board, 2) {
                if !board.is_legal_for_color(mv, player) {
                    continue;
                }

                let own_win = own_wins.contains(&mv);
                let immediate_block = opponent_wins.contains(&mv);
                let (_, hard_keep) = hard_tactical_ordering_score(own_win, immediate_block);
                let summary =
                    SearchThreatPolicy.ordering_summary_for_legal_player(&board, player, mv);
                let (tactical_score, tactical_keep) =
                    tactical_ordering_score_from_summary(summary, immediate_block);
                if tactical_score > 0 || tactical_keep {
                    assert!(
                        hard_keep || has_tactical_annotation_potential(&board, player, mv),
                        "scenario '{}' move {} has tactical score {} keep {} but failed potential filter",
                        scenario.id,
                        mv.to_notation(),
                        tactical_score,
                        tactical_keep
                    );
                }
            }
        }
    }

    #[test]
    fn tactical_annotation_potential_respects_viability_mask() {
        let mut board = Board::new(RuleConfig::default());
        apply_moves(&mut board, &["H8", "A1", "I8", "B1"]);
        let probe = mv("J8");

        assert!(has_tactical_annotation_potential(
            &board,
            Color::Black,
            probe
        ));
        assert!(!has_tactical_annotation_potential_with_mask(
            &board,
            Color::Black,
            probe,
            0
        ));
    }

    #[test]
    fn tactical_ordering_summary_matches_full_annotation_score() {
        let mut forcing_board = Board::new(RuleConfig::default());
        apply_moves(&mut forcing_board, &["H8", "A1", "I8", "B1", "J8", "C1"]);

        let renju_board = {
            let mut board = Board::new(RuleConfig {
                variant: Variant::Renju,
                ..RuleConfig::default()
            });
            apply_moves(
                &mut board,
                &["H8", "G8", "I8", "A1", "J8", "A2", "L8", "A3"],
            );
            board
        };

        let cases = [
            (forcing_board.clone(), Color::Black, mv("K8")),
            (forcing_board, Color::Black, mv("B2")),
            (renju_board, Color::Black, mv("M8")),
        ];

        for (board, player, probe) in cases {
            let annotation = SearchThreatPolicy.annotation_for_player(&board, player, probe);
            let expected = tactical_ordering_score(&annotation, false);
            let zobrist = ZobristTable::new(board.config.board_size);

            let mut scan_metrics = SearchMetrics::default();
            let mut scan_state =
                SearchState::from_board_with_frontier(board.clone(), &zobrist, false);
            let scan_summary = tactical_ordering_summary_for_threat_view_mode(
                &mut scan_state,
                player,
                probe,
                ThreatViewMode::Scan,
                &mut scan_metrics,
            );
            assert_eq!((scan_summary.score, scan_summary.must_keep), expected);

            let mut rolling_metrics = SearchMetrics::default();
            let mut rolling_state = SearchState::from_board_with_frontier(board, &zobrist, true);
            let rolling_summary = tactical_ordering_summary_for_threat_view_mode(
                &mut rolling_state,
                player,
                probe,
                ThreatViewMode::Rolling,
                &mut rolling_metrics,
            );
            assert_eq!((rolling_summary.score, rolling_summary.must_keep), expected);
        }
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
        let mut state = SearchState::from_board_with_frontier(board, &zobrist, false);
        let mut tt = HashMap::new();
        let mut nodes = 0;
        let mut metrics = SearchMetrics::default();
        let deadline = SearchDeadline::new(Instant::now(), None, None, None);

        let outcome = negamax(
            &mut state,
            1,
            i32::MIN + 1,
            i32::MAX,
            Color::Black,
            Color::Black,
            &mut tt,
            None,
            &zobrist,
            CandidateSource::NearAll { radius: 2 },
            NullCellCulling::Disabled,
            LegalityGate::ExactRules,
            MoveOrdering::TranspositionFirstBoardOrder,
            Some(1),
            ThreatViewMode::Scan,
            StaticEvaluation::LineShapeEval,
            &mut nodes,
            &mut metrics,
            deadline,
        );

        let best_move = outcome
            .best_move
            .expect("legal moves after the illegal first candidate");
        assert!(state.board().is_legal(best_move));
        assert_ne!(best_move, mv("B1"));
        assert_eq!(metrics.search_child_cap_hits, 1);
        assert!(metrics.search_legality_checks > 1);
    }

    #[test]
    fn explicit_config_constructors_preserve_current_defaults() {
        let baseline = SearchBotConfig::custom_depth(3);
        assert_eq!(SearchBot::new(3).config(), baseline);
        assert_eq!(
            baseline.candidate_source(),
            CandidateSource::NearAll { radius: 2 }
        );
        assert_eq!(baseline.legality_gate(), LegalityGate::ExactRules);
        assert_eq!(baseline.safety_gate(), SafetyGate::CurrentObligation);
        assert_eq!(baseline.null_cell_culling, NullCellCulling::Disabled);
        assert_eq!(baseline.max_tt_entries, None);
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
            max_tt_entries: None,
            candidate_radius: 3,
            candidate_opponent_radius: None,
            safety_gate: SafetyGate::None,
            move_ordering: MoveOrdering::TranspositionFirstBoardOrder,
            child_limit: None,
            search_algorithm: SearchAlgorithm::AlphaBetaIterativeDeepening,
            static_eval: StaticEvaluation::LineShapeEval,
            corridor_proof: CorridorProofConfig::DISABLED,
            threat_view_mode: ThreatViewMode::Scan,
            null_cell_culling: NullCellCulling::Enabled,
        };
        assert_eq!(SearchBot::with_config(config).config(), config);
        assert_eq!(
            config.candidate_source(),
            CandidateSource::NearAll { radius: 3 }
        );
        assert_eq!(config.safety_gate, SafetyGate::None);
        assert_eq!(config.null_cell_culling, NullCellCulling::Enabled);

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
    fn tt_cap_skips_new_entries_but_updates_existing_entries() {
        let mut tt = HashMap::new();
        let mut metrics = SearchMetrics::default();
        let first = TTEntry {
            depth: 1,
            score: 10,
            flag: TTFlag::Exact,
            best_move: Some(mv("H8")),
        };
        let replacement = TTEntry {
            depth: 2,
            score: 20,
            flag: TTFlag::Exact,
            best_move: Some(mv("I8")),
        };

        store_tt_entry(&mut tt, Some(1), &mut metrics, 1, first);
        store_tt_entry(&mut tt, Some(1), &mut metrics, 2, first);
        store_tt_entry(&mut tt, Some(1), &mut metrics, 1, replacement);

        assert_eq!(tt.len(), 1);
        assert!(!tt.contains_key(&2));
        assert_eq!(tt.get(&1).map(|entry| entry.depth), Some(2));
        assert_eq!(tt.get(&1).map(|entry| entry.score), Some(20));
        assert_eq!(metrics.tt_insert_skips, 1);
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
        assert_eq!(trace["config"]["safety_gate"], "current_obligation");
        assert_eq!(trace["config"]["move_ordering"], "tt_first_board_order");
        assert_eq!(trace["config"]["child_limit"], serde_json::Value::Null);
        assert_eq!(trace["config"]["max_tt_entries"], serde_json::Value::Null);
        assert_eq!(trace["config"]["search_algorithm"], "alpha_beta_id");
        assert_eq!(trace["config"]["static_eval"], "line_shape_eval");
        assert_eq!(trace["config"]["threat_view_mode"], "rolling");
        assert_eq!(trace["config"]["null_cell_culling"], "disabled");
        assert!(trace["nodes"].as_u64().unwrap() > 0);
        assert!(trace["total_nodes"].as_u64().unwrap() >= trace["nodes"].as_u64().unwrap());
        assert_eq!(trace["budget_exhausted"], false);
        assert_eq!(trace["depth"], 3);
        assert!(trace["tt"]["entries"].as_u64().is_some());
        assert_eq!(trace["tt"]["max_entries"], serde_json::Value::Null);

        let metrics = &trace["metrics"];
        assert!(metrics["eval_calls"].as_u64().unwrap() > 0);
        assert!(metrics["candidate_generations"].as_u64().unwrap() > 0);
        assert!(metrics["legality_checks"].as_u64().unwrap() > 0);
        assert!(metrics["tt_hits"].as_u64().is_some());
        assert!(metrics["tt_cutoffs"].as_u64().is_some());
        assert!(metrics["tt_insert_skips"].as_u64().is_some());
        assert!(metrics["beta_cutoffs"].as_u64().is_some());
        assert!(metrics["root_candidate_generations"].as_u64().is_some());
        assert!(metrics["search_candidate_generations"].as_u64().is_some());
        assert!(metrics["root_legality_checks"].as_u64().is_some());
        assert!(metrics["search_legality_checks"].as_u64().is_some());
        assert!(metrics["renju_forbidden_checks"].as_u64().is_some());
        assert!(metrics["renju_forbidden_ns"].as_u64().is_some());
        assert!(metrics["renju_forbidden_prefilter_checks"]
            .as_u64()
            .is_some());
        assert!(metrics["renju_forbidden_prefilter_ns"].as_u64().is_some());
        assert!(metrics["renju_effective_filter_calls"].as_u64().is_some());
        assert!(metrics["renju_effective_filter_ns"].as_u64().is_some());
        assert!(metrics["renju_effective_filter_continuation_checks"]
            .as_u64()
            .is_some());
        assert!(metrics["renju_effective_filter_continuation_ns"]
            .as_u64()
            .is_some());
        assert!(metrics["root_tactical_annotations"].as_u64().is_some());
        assert!(metrics["search_tactical_annotations"].as_u64().is_some());
    }

    #[test]
    fn trace_records_corridor_proof_config_and_metrics() {
        let recorded_leaf_loss = [
            112, 111, 127, 126, 97, 142, 113, 141, 82, 67, 96, 110, 94, 156, 171, 95, 128, 80, 65,
            140, 125, 139, 143, 138,
        ];
        let mut board = Board::new(RuleConfig::default());
        apply_cell_moves(&mut board, &recorded_leaf_loss[..4]);
        assert_eq!(board.current_player, Color::Black);

        let mut config = SearchBotConfig::custom_depth(3);
        config.safety_gate = SafetyGate::None;
        config.corridor_proof = CorridorProofConfig {
            enabled: true,
            max_depth: 4,
            max_reply_width: 3,
            proof_candidate_limit: CorridorProofConfig::DEFAULT_PROOF_CANDIDATE_LIMIT,
        };
        let mut bot = SearchBot::with_config(config);

        let _ = bot.choose_move(&board);
        let trace = bot.trace().expect("expected search trace");

        assert_eq!(trace["config"]["corridor_proof"]["enabled"], true);
        assert_eq!(trace["config"]["corridor_proof"]["max_depth"], 4);
        assert_eq!(trace["config"]["corridor_proof"]["max_reply_width"], 3);
        assert_eq!(trace["metrics"]["corridor_proof_passes"], 1);
        assert!(trace["metrics"]["corridor_proof_checks"].as_u64().unwrap() > 0);
        assert!(trace["metrics"]["corridor_proof_active"].as_u64().unwrap() > 0);
        assert!(
            trace["metrics"]["corridor_proof_candidates_considered"]
                .as_u64()
                .unwrap()
                > 0
        );
        for key in [
            "corridor_proof_terminal_exits",
            "corridor_proof_terminal_root_candidates",
            "corridor_proof_terminal_root_winning_candidates",
            "corridor_proof_terminal_root_losing_candidates",
            "corridor_proof_terminal_root_overrides",
            "corridor_proof_terminal_root_move_changes",
            "corridor_proof_terminal_root_move_confirmations",
            "corridor_proof_wins",
            "corridor_proof_losses",
            "corridor_proof_unknown",
            "corridor_proof_move_changes",
            "corridor_proof_move_confirmations",
        ] {
            assert!(
                trace["metrics"][key].as_u64().is_some(),
                "missing corridor proof metric {key}"
            );
        }
        assert!(trace["corridor"]["search_nodes"].as_u64().unwrap() > 0);
    }

    #[test]
    fn corridor_proof_non_terminal_work_keeps_normal_search_move() {
        let recorded_leaf_loss = [
            112, 111, 127, 126, 97, 142, 113, 141, 82, 67, 96, 110, 94, 156, 171, 95, 128, 80, 65,
            140, 125, 139, 143, 138,
        ];
        let mut board = Board::new(RuleConfig::default());
        apply_cell_moves(&mut board, &recorded_leaf_loss[..4]);
        assert_eq!(board.current_player, Color::Black);

        let mut normal_bot = SearchBot::with_config(SearchBotConfig::custom_depth(3));
        let normal_move = normal_bot.choose_move(&board);

        let mut config = SearchBotConfig::custom_depth(3);
        config.corridor_proof = CorridorProofConfig {
            enabled: true,
            max_depth: 1,
            max_reply_width: 3,
            proof_candidate_limit: CorridorProofConfig::DEFAULT_PROOF_CANDIDATE_LIMIT,
        };
        let mut leaf_bot = SearchBot::with_config(config);
        let leaf_move = leaf_bot.choose_move(&board);
        let trace = leaf_bot.trace().expect("expected search trace");
        let metrics = &trace["metrics"];

        assert!(metrics["corridor_proof_active"].as_u64().unwrap() > 0);
        assert_eq!(
            metrics["corridor_proof_terminal_root_overrides"]
                .as_u64()
                .unwrap(),
            0
        );
        assert_eq!(
            leaf_move, normal_move,
            "non-terminal corridor proof work should not override normal move"
        );
    }

    #[test]
    fn corridor_proof_does_not_run_without_completed_normal_depth() {
        let mut board = Board::new(RuleConfig::default());
        apply_moves(&mut board, &["H8", "A1", "I8", "A2"]);

        let mut config = SearchBotConfig::custom_time_budget(0);
        config.corridor_proof = CorridorProofConfig {
            enabled: true,
            max_depth: 4,
            max_reply_width: 3,
            proof_candidate_limit: CorridorProofConfig::DEFAULT_PROOF_CANDIDATE_LIMIT,
        };
        let mut bot = SearchBot::with_config(config);

        let _ = bot.choose_move(&board);
        let trace = bot.trace().expect("expected search trace");

        assert_eq!(trace["depth"], 0);
        assert_eq!(trace["metrics"]["corridor_proof_passes"], 0);
        assert_eq!(trace["metrics"]["corridor_proof_candidates_considered"], 0);
    }

    #[test]
    fn corridor_proof_selects_normal_best_then_ranked_candidates() {
        let best = mv("H8");
        let close = mv("H9");
        let also_close = mv("H10");
        let too_far = mv("H11");
        let results = vec![
            RootCandidateResult {
                mv: close,
                score: 960_000,
            },
            RootCandidateResult {
                mv: best,
                score: 1_000_000,
            },
            RootCandidateResult {
                mv: too_far,
                score: 900_000,
            },
            RootCandidateResult {
                mv: also_close,
                score: 955_000,
            },
        ];

        let selected = select_corridor_proof_candidates(&results, best, 4)
            .into_iter()
            .map(|candidate| candidate.mv)
            .collect::<Vec<_>>();

        assert_eq!(selected, vec![best, close, also_close, too_far]);
    }

    #[test]
    fn corridor_proof_selects_top_candidates_without_score_margin() {
        let best = mv("H8");
        let second = mv("H9");
        let third = mv("H10");
        let fourth = mv("H11");
        let results = vec![
            RootCandidateResult {
                mv: fourth,
                score: -250_000,
            },
            RootCandidateResult {
                mv: best,
                score: 1_000_000,
            },
            RootCandidateResult {
                mv: third,
                score: 100_000,
            },
            RootCandidateResult {
                mv: second,
                score: 200_000,
            },
        ];

        let selected = select_corridor_proof_candidates(&results, best, 4);

        assert_eq!(
            selected
                .iter()
                .map(|candidate| candidate.mv)
                .collect::<Vec<_>>(),
            vec![best, second, third, fourth]
        );
        assert_eq!(
            selected
                .iter()
                .map(|candidate| candidate.rank)
                .collect::<Vec<_>>(),
            vec![1, 2, 3, 4]
        );
        assert_eq!(
            selected
                .iter()
                .map(|candidate| candidate.score_gap)
                .collect::<Vec<_>>(),
            vec![0, 800_000, 900_000, 1_250_000]
        );
    }

    #[test]
    fn corridor_proof_resolution_confirms_normal_best_win() {
        let best = mv("H8");
        let proof = CorridorProofCandidateResult {
            mv: best,
            outcome: CandidateProofOutcome::ProvenWin,
        };

        let decision = resolve_corridor_proof_candidates(best, &[proof]);

        assert_eq!(decision.best_move, best);
        assert_eq!(decision.reason, CorridorProofDecisionReason::ConfirmedWin);
    }

    #[test]
    fn corridor_proof_resolution_switches_to_proven_win() {
        let best = mv("H8");
        let proven = mv("J8");
        let proofs = vec![
            CorridorProofCandidateResult {
                mv: best,
                outcome: CandidateProofOutcome::Unknown,
            },
            CorridorProofCandidateResult {
                mv: proven,
                outcome: CandidateProofOutcome::ProvenWin,
            },
        ];

        let decision = resolve_corridor_proof_candidates(best, &proofs);

        assert_eq!(decision.best_move, proven);
        assert_eq!(decision.reason, CorridorProofDecisionReason::ChangedToWin);
    }

    #[test]
    fn corridor_proof_resolution_escapes_proven_loss_to_unknown() {
        let best = mv("H8");
        let fallback = mv("J8");
        let proofs = vec![
            CorridorProofCandidateResult {
                mv: best,
                outcome: CandidateProofOutcome::ProvenLoss,
            },
            CorridorProofCandidateResult {
                mv: fallback,
                outcome: CandidateProofOutcome::Unknown,
            },
        ];

        let decision = resolve_corridor_proof_candidates(best, &proofs);

        assert_eq!(decision.best_move, fallback);
        assert_eq!(decision.reason, CorridorProofDecisionReason::AvoidedLoss);
    }

    #[test]
    fn threat_view_shadow_mode_reports_tactical_ordering_parity_checks() {
        let mut board = Board::new(RuleConfig::default());
        apply_moves(&mut board, &["H8", "A1", "I8", "B1", "J8", "C1"]);

        let mut config = SearchBotConfig::custom_depth(1);
        config.safety_gate = SafetyGate::None;
        config.move_ordering = MoveOrdering::TacticalFull;
        config.threat_view_mode = ThreatViewMode::RollingShadow;
        let mut bot = SearchBot::with_config(config);

        let _ = bot.choose_move(&board);
        let trace = bot.trace().expect("expected search trace");

        assert_eq!(trace["config"]["threat_view_mode"], "rolling_shadow");
        assert!(
            trace["metrics"]["root_tactical_annotations"]
                .as_u64()
                .unwrap()
                > 0
        );
        assert!(
            trace["metrics"]["threat_view_shadow_checks"]
                .as_u64()
                .unwrap()
                >= trace["metrics"]["root_tactical_annotations"]
                    .as_u64()
                    .unwrap()
        );
        assert_eq!(trace["metrics"]["threat_view_shadow_mismatches"], 0);
        assert!(
            trace["metrics"]["threat_view_scan_queries"]
                .as_u64()
                .unwrap()
                > 0
        );
        assert!(
            trace["metrics"]["threat_view_frontier_queries"]
                .as_u64()
                .unwrap()
                > 0
        );
        assert_eq!(
            trace["metrics"]["threat_view_frontier_move_fact_updates"], 0,
            "tactical-only rolling should not maintain corridor move facts"
        );
        assert!(
            trace["metrics"]["threat_view_frontier_annotation_dirty_marks"]
                .as_u64()
                .unwrap()
                > 0
        );
    }

    #[test]
    fn rolling_threat_view_mode_can_drive_tactical_ordering() {
        let mut board = Board::new(RuleConfig::default());
        apply_moves(&mut board, &["H8", "A1", "I8", "B1", "J8", "C1"]);

        let mut config = SearchBotConfig::custom_depth(1);
        config.safety_gate = SafetyGate::None;
        config.move_ordering = MoveOrdering::TacticalFull;
        config.threat_view_mode = ThreatViewMode::Rolling;
        let mut bot = SearchBot::with_config(config);

        let _ = bot.choose_move(&board);
        let trace = bot.trace().expect("expected search trace");

        assert_eq!(trace["config"]["threat_view_mode"], "rolling");
        assert!(
            trace["metrics"]["root_tactical_annotations"]
                .as_u64()
                .unwrap()
                > 0
        );
        assert_eq!(trace["metrics"]["threat_view_shadow_checks"], 0);
        assert_eq!(trace["metrics"]["threat_view_shadow_mismatches"], 0);
        assert_eq!(trace["metrics"]["threat_view_scan_queries"], 0);
        assert!(
            trace["metrics"]["threat_view_frontier_queries"]
                .as_u64()
                .unwrap()
                > 0
        );
    }

    #[test]
    fn rolling_threat_view_mode_can_drive_current_obligation_safety() {
        let mut board = Board::new(RuleConfig::default());
        apply_moves(&mut board, &["H8", "A1", "I8", "B1", "J8"]);

        let mut config = SearchBotConfig::custom_depth(1);
        config.safety_gate = SafetyGate::CurrentObligation;
        config.threat_view_mode = ThreatViewMode::Rolling;
        let mut bot = SearchBot::with_config(config);

        let chosen = bot.choose_move(&board);
        let trace = bot.trace().expect("expected search trace");

        assert!(chosen == mv("G8") || chosen == mv("K8"));
        assert_eq!(trace["config"]["safety_gate"], "current_obligation");
        assert_eq!(trace["config"]["threat_view_mode"], "rolling");
        assert!(trace["safety_nodes"].as_u64().unwrap() > 0);
        assert_eq!(trace["metrics"]["threat_view_scan_queries"], 0);
        assert!(
            trace["metrics"]["threat_view_frontier_rebuilds"]
                .as_u64()
                .unwrap()
                > 0
        );
        assert!(
            trace["metrics"]["threat_view_frontier_queries"]
                .as_u64()
                .unwrap()
                > 0
        );
        assert!(
            trace["metrics"]["threat_view_frontier_immediate_win_queries"]
                .as_u64()
                .unwrap()
                > 0
        );
    }

    #[test]
    fn rolling_shadow_current_obligation_safety_preserves_scan_choice() {
        let mut board = Board::new(RuleConfig::default());
        apply_moves(&mut board, &["H8", "A1", "I8", "B1", "J8"]);

        let mut config = SearchBotConfig::custom_depth(1);
        config.safety_gate = SafetyGate::CurrentObligation;
        config.threat_view_mode = ThreatViewMode::RollingShadow;
        let mut bot = SearchBot::with_config(config);

        let chosen = bot.choose_move(&board);
        let trace = bot.trace().expect("expected search trace");

        assert!(chosen == mv("G8") || chosen == mv("K8"));
        assert_eq!(trace["metrics"]["threat_view_shadow_mismatches"], 0);
        assert!(
            trace["metrics"]["threat_view_shadow_checks"]
                .as_u64()
                .unwrap()
                > 0
        );
        assert!(
            trace["metrics"]["threat_view_scan_queries"]
                .as_u64()
                .unwrap()
                > 0
        );
        assert!(
            trace["metrics"]["threat_view_frontier_queries"]
                .as_u64()
                .unwrap()
                > 0
        );
    }

    #[test]
    fn rolling_threat_view_memoizes_dirty_ordering_summaries_per_state() {
        let mut board = Board::new(RuleConfig::default());
        apply_moves(&mut board, &["B8", "A1", "C8", "A2", "D8"]);
        let zobrist = ZobristTable::new(board.config.board_size);
        let mut state = SearchState::from_board_for_config(
            board,
            &zobrist,
            ThreatViewMode::Rolling,
            StaticEvaluation::LineShapeEval,
            CorridorProofConfig::DISABLED,
        );

        state.apply_trusted_legal_move(mv("E8"), &zobrist);

        let mut metrics = SearchMetrics::default();
        let player = state.board().current_player;
        let first = tactical_ordering_summary_for_threat_view_mode(
            &mut state,
            player,
            mv("A8"),
            ThreatViewMode::Rolling,
            &mut metrics,
        );
        let second = tactical_ordering_summary_for_threat_view_mode(
            &mut state,
            player,
            mv("A8"),
            ThreatViewMode::Rolling,
            &mut metrics,
        );

        assert_eq!(first, second);
        assert_eq!(metrics.threat_view_frontier_dirty_annotation_queries, 1);
        assert_eq!(metrics.threat_view_frontier_queries, 1);
    }

    #[test]
    fn rolling_immediate_win_query_records_dedicated_metrics() {
        let mut board = Board::new(RuleConfig::default());
        apply_moves(
            &mut board,
            &["H8", "A1", "I8", "A2", "J8", "A3", "K8", "A4"],
        );
        let zobrist = ZobristTable::new(board.config.board_size);
        let mut state = SearchState::from_board_for_config(
            board,
            &zobrist,
            ThreatViewMode::Rolling,
            StaticEvaluation::LineShapeEval,
            CorridorProofConfig::DISABLED,
        );
        let mut metrics = SearchMetrics::default();

        let wins = immediate_winning_moves_for_threat_view_mode(
            &mut state,
            Color::Black,
            ThreatViewMode::Rolling,
            &mut metrics,
        );

        assert_eq!(wins, vec![mv("G8"), mv("L8")]);
        assert_eq!(metrics.threat_view_frontier_queries, 1);
        assert_eq!(metrics.threat_view_frontier_immediate_win_queries, 1);
        assert!(metrics.threat_view_frontier_immediate_win_query_ns > 0);
    }

    #[test]
    fn rolling_attacker_corridor_materialization_uses_cached_candidate_potential() {
        let mut board = Board::new(RuleConfig::default());
        apply_moves(&mut board, &["H8", "A1", "I8", "A2", "J8", "A3"]);
        assert_eq!(board.current_player, Color::Black);

        let expected = scan_materialized_attacker_corridor_moves(&board, Color::Black);
        assert!(
            !expected.is_empty(),
            "fixture should expose attacker corridor candidates"
        );

        let zobrist = ZobristTable::new(board.config.board_size);
        let mut state = SearchState::from_board_for_config(
            board,
            &zobrist,
            ThreatViewMode::Rolling,
            StaticEvaluation::LineShapeEval,
            CorridorProofConfig {
                enabled: true,
                max_depth: 2,
                max_reply_width: 3,
                proof_candidate_limit: CorridorProofConfig::DEFAULT_PROOF_CANDIDATE_LIMIT,
            },
        );
        let mut metrics = SearchMetrics::default();

        let actual =
            rolling_materialized_attacker_corridor_moves(&mut state, Color::Black, &mut metrics);

        assert_eq!(actual, expected);
        assert_eq!(
            metrics.threat_view_frontier_move_fact_updates, 0,
            "candidate potential should avoid apply/undo frontier move-fact churn"
        );
    }

    #[test]
    fn rolling_attacker_corridor_materialization_matches_scan_on_benchmark_scenarios() {
        for scenario in scenarios::SCENARIOS {
            let board = scenario.board();
            if board.result != GameResult::Ongoing {
                continue;
            }

            let attacker = board.current_player;
            let expected = scan_materialized_attacker_corridor_moves(&board, attacker);
            let zobrist = ZobristTable::new(board.config.board_size);
            let mut state = SearchState::from_board_for_config(
                board,
                &zobrist,
                ThreatViewMode::Rolling,
                StaticEvaluation::LineShapeEval,
                CorridorProofConfig {
                    enabled: true,
                    max_depth: 2,
                    max_reply_width: 3,
                    proof_candidate_limit: CorridorProofConfig::DEFAULT_PROOF_CANDIDATE_LIMIT,
                },
            );
            let mut metrics = SearchMetrics::default();

            let actual =
                rolling_materialized_attacker_corridor_moves(&mut state, attacker, &mut metrics);

            assert_eq!(
                actual, expected,
                "scenario '{}' diverged for {:?}",
                scenario.id, attacker
            );
        }
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
        assert_eq!(
            trace["metrics"]["eval_calls"],
            trace["metrics"]["pattern_eval_calls"]
        );
        assert_eq!(trace["metrics"]["line_shape_eval_calls"], 0);
        assert!(
            trace["metrics"]["pattern_eval_ns"].as_u64().unwrap() > 0,
            "pattern eval timing should be recorded separately from generic eval calls"
        );
    }

    #[test]
    fn rolling_pattern_eval_uses_pattern_frame_cache() {
        let mut board = Board::new(RuleConfig::default());
        apply_moves(&mut board, &["H8", "G8", "H9", "G9"]);
        let mut config = SearchBotConfig::custom_depth(2);
        config.static_eval = StaticEvaluation::PatternEval;
        config.threat_view_mode = ThreatViewMode::Rolling;
        let mut bot = SearchBot::with_config(config);

        let _ = bot.choose_move(&board);
        let trace = bot.trace().expect("expected search trace");
        let metrics = &trace["metrics"];

        assert_eq!(trace["config"]["static_eval"], "pattern_eval");
        assert_eq!(trace["config"]["threat_view_mode"], "rolling");
        assert!(
            metrics["pattern_frame_queries"].as_u64().unwrap() > 0,
            "rolling pattern eval should query the cached pattern frame"
        );
        assert!(
            metrics["pattern_frame_query_ns"].as_u64().unwrap() > 0,
            "cached pattern frame query timing should be recorded"
        );
        assert!(
            metrics["pattern_frame_updates"].as_u64().unwrap() > 0,
            "search state move updates should keep the pattern frame in sync"
        );
        assert_eq!(
            metrics["pattern_frame_shadow_mismatches"], 0,
            "cached pattern eval should match scan eval in test/debug shadow checks"
        );
    }

    #[test]
    fn trace_records_aggregate_stage_timings() {
        let mut board = Board::new(RuleConfig::default());
        apply_moves(&mut board, &["H8", "G8", "H9", "G9"]);
        let mut config = SearchBotConfig::custom_depth(2);
        config.static_eval = StaticEvaluation::PatternEval;
        config.threat_view_mode = ThreatViewMode::Rolling;
        config.move_ordering = MoveOrdering::Tactical;
        config.child_limit = Some(8);
        let mut bot = SearchBot::with_config(config);

        let _ = bot.choose_move(&board);
        let trace = bot.trace().expect("expected search trace");
        let metrics = &trace["metrics"];

        assert!(metrics["stage_move_gen_ns"].as_u64().unwrap() > 0);
        assert!(metrics["stage_ordering_ns"].as_u64().unwrap() > 0);
        assert!(metrics["stage_eval_ns"].as_u64().unwrap() > 0);
        assert!(metrics["stage_threat_ns"].as_u64().unwrap() > 0);
        assert_eq!(metrics["stage_proof_ns"].as_u64().unwrap(), 0);
    }

    #[test]
    fn pattern_eval_scan_and_rolling_cache_choose_same_moves_on_benchmark_scenarios() {
        for scenario in scenarios::SCENARIOS {
            let board = scenario.board();
            if board.result != GameResult::Ongoing {
                continue;
            }

            let mut scan_config = SearchBotConfig::custom_depth(2);
            scan_config.static_eval = StaticEvaluation::PatternEval;
            scan_config.threat_view_mode = ThreatViewMode::Scan;

            let mut rolling_config = scan_config;
            rolling_config.threat_view_mode = ThreatViewMode::Rolling;

            let mut scan_bot = SearchBot::with_config(scan_config);
            let mut rolling_bot = SearchBot::with_config(rolling_config);

            let scan_move = scan_bot.choose_move(&board);
            let rolling_move = rolling_bot.choose_move(&board);

            assert_eq!(
                rolling_move, scan_move,
                "rolling cached pattern eval should preserve scan pattern eval choice on scenario '{}'",
                scenario.id
            );

            let scan_trace = scan_bot.trace().expect("expected scan search trace");
            assert_eq!(
                scan_trace["metrics"]["pattern_frame_queries"], 0,
                "scan mode should not use the rolling pattern frame on scenario '{}'",
                scenario.id
            );

            let rolling_trace = rolling_bot.trace().expect("expected rolling search trace");
            assert!(
                rolling_trace["metrics"]["pattern_frame_queries"]
                    .as_u64()
                    .unwrap()
                    > 0,
                "rolling mode should use the pattern frame on scenario '{}'",
                scenario.id
            );
            assert_eq!(
                rolling_trace["metrics"]["pattern_frame_shadow_mismatches"], 0,
                "rolling cached pattern eval should match scan eval on scenario '{}'",
                scenario.id
            );
        }
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
    fn null_cell_culling_filters_dead_root_candidates() {
        let mut board = Board::new(RuleConfig {
            variant: Variant::Freestyle,
            ..Default::default()
        });
        apply_moves(
            &mut board,
            &[
                "G8", "D8", "L8", "I8", "H7", "H4", "H12", "H9", "G7", "D4", "L12", "I9", "G9",
                "D12", "L4", "I7",
            ],
        );
        assert!(candidate_moves(&board, 2).contains(&mv("H8")));

        let mut config = SearchBotConfig::custom_depth(1);
        config.safety_gate = SafetyGate::None;
        config.null_cell_culling = NullCellCulling::Enabled;
        let mut bot = SearchBot::with_config(config);

        let _ = bot.choose_move(&board);
        let trace = bot.trace().expect("expected search trace");
        let metrics = &trace["metrics"];

        assert_eq!(trace["config"]["null_cell_culling"], "enabled");
        assert!(metrics["root_null_cell_cull_checks"].as_u64().unwrap() > 0);
        assert!(metrics["root_null_cells_culled"].as_u64().unwrap() > 0);

        let zobrist = ZobristTable::new(board.config.board_size);
        let mut state = SearchState::from_board_for_config(
            board,
            &zobrist,
            ThreatViewMode::Rolling,
            StaticEvaluation::LineShapeEval,
            CorridorProofConfig::DISABLED,
        );
        let mut tt = HashMap::new();
        let mut nodes = 0;
        let mut metrics = SearchMetrics::default();
        let _ = negamax(
            &mut state,
            1,
            i32::MIN + 1,
            i32::MAX,
            Color::Black,
            Color::Black,
            &mut tt,
            None,
            &zobrist,
            CandidateSource::NearAll { radius: 2 },
            NullCellCulling::Enabled,
            LegalityGate::ExactRules,
            MoveOrdering::TranspositionFirstBoardOrder,
            None,
            ThreatViewMode::Rolling,
            StaticEvaluation::LineShapeEval,
            &mut nodes,
            &mut metrics,
            SearchDeadline::new(Instant::now(), None, None, None),
        );
        assert!(metrics.search_null_cell_cull_checks > 0);
        assert!(metrics.search_null_cells_culled > 0);
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
        config.move_ordering = MoveOrdering::TacticalFull;
        let mut bot = SearchBot::with_config(config);

        let _ = bot.choose_move(&board);
        let trace = bot.trace().expect("expected search trace");
        let metrics = &trace["metrics"];

        assert_eq!(trace["config"]["move_ordering"], "tactical_full");
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
        config.move_ordering = MoveOrdering::TacticalFull;
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
            max_tt_entries: None,
            candidate_radius: 2,
            candidate_opponent_radius: None,
            safety_gate: SafetyGate::None,
            move_ordering: MoveOrdering::TranspositionFirstBoardOrder,
            child_limit: None,
            search_algorithm: SearchAlgorithm::AlphaBetaIterativeDeepening,
            static_eval: StaticEvaluation::LineShapeEval,
            corridor_proof: CorridorProofConfig::DISABLED,
            threat_view_mode: ThreatViewMode::Scan,
            null_cell_culling: NullCellCulling::Disabled,
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
        for case in SEARCH_BEHAVIOR_CASES {
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
}
