use instant::Instant;
use std::collections::HashMap;
#[cfg(test)]
use std::time::Duration;

use crate::corridor;
use crate::frontier::{FrontierAnnotationSource, RollingFrontierFeatures, RollingThreatFrontier};
#[cfg(any(test, debug_assertions))]
use crate::pattern::evaluate_pattern_scan;
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

mod timing;

mod corridor_proof;
use corridor_proof::{
    run_corridor_proof_pass, terminal_score_for_winner, CorridorProofDecisionReason,
    RootCandidateResult, TERMINAL_SCORE_THRESHOLD,
};
use timing::{thread_cpu_time, SearchDeadline};
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

mod candidates;

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

mod evaluation;
mod metrics;

#[cfg(test)]
use evaluation::evaluate_reference;
use evaluation::{evaluate, evaluate_static};

use candidates::candidate_moves_from_source;
pub use candidates::pipeline_bench_candidate_moves;
#[cfg(test)]
use candidates::{
    candidate_moves, candidate_moves_from_source_reference, candidate_moves_reference,
    default_candidate_masks, mask_contains, STACK_SEEN_WORDS,
};
pub use metrics::SearchMetrics;
use metrics::{RenjuForbiddenMetricSource, SearchMetricPhase};

mod state;

use state::SearchState;
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

mod engine;
mod ordering;
mod safety;
mod threat_view;

use engine::*;
use ordering::*;
use safety::*;
use threat_view::*;

mod config;

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
mod tests;
