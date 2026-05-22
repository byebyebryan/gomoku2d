use std::time::Duration;

use serde::Serialize;

use crate::frontier::{FrontierAnnotationSource, FrontierUpdateTimings};
use crate::tactical::{tactical_metrics_snapshot, TacticalMetrics};
use gomoku_core::{renju_forbidden_metrics_snapshot, RenjuForbiddenMetrics};

use super::config::{CorridorSide, StaticEvaluation};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct SearchMetrics {
    pub eval_calls: u64,
    pub line_shape_eval_calls: u64,
    pub line_shape_eval_ns: u64,
    pub pattern_eval_calls: u64,
    pub pattern_eval_ns: u64,
    pub pattern_frame_queries: u64,
    pub pattern_frame_query_ns: u64,
    pub pattern_frame_updates: u64,
    pub pattern_frame_update_ns: u64,
    pub pattern_frame_shadow_checks: u64,
    pub pattern_frame_shadow_mismatches: u64,
    pub candidate_generations: u64,
    pub candidate_moves_total: u64,
    pub candidate_moves_max: u64,
    pub root_candidate_generations: u64,
    pub root_candidate_moves_total: u64,
    pub root_candidate_moves_max: u64,
    pub search_candidate_generations: u64,
    pub search_candidate_moves_total: u64,
    pub search_candidate_moves_max: u64,
    pub null_cell_cull_checks: u64,
    pub null_cell_cull_ns: u64,
    pub null_cells_culled: u64,
    pub root_null_cell_cull_checks: u64,
    pub root_null_cell_cull_ns: u64,
    pub root_null_cells_culled: u64,
    pub search_null_cell_cull_checks: u64,
    pub search_null_cell_cull_ns: u64,
    pub search_null_cells_culled: u64,
    pub legality_checks: u64,
    pub illegal_moves_skipped: u64,
    pub root_legality_checks: u64,
    pub root_illegal_moves_skipped: u64,
    pub search_legality_checks: u64,
    pub search_illegal_moves_skipped: u64,
    pub renju_forbidden_prefilter_checks: u64,
    pub renju_forbidden_prefilter_ns: u64,
    pub renju_forbidden_checks: u64,
    pub renju_forbidden_ns: u64,
    pub renju_forbidden_search_gate_checks: u64,
    pub renju_forbidden_search_gate_ns: u64,
    pub renju_forbidden_pattern_checks: u64,
    pub renju_forbidden_pattern_ns: u64,
    pub renju_forbidden_threat_checks: u64,
    pub renju_forbidden_threat_ns: u64,
    pub renju_forbidden_other_checks: u64,
    pub renju_forbidden_other_ns: u64,
    pub renju_effective_filter_calls: u64,
    pub renju_effective_filter_ns: u64,
    pub renju_effective_filter_continuation_checks: u64,
    pub renju_effective_filter_continuation_ns: u64,
    pub compound_imminent_queries: u64,
    pub compound_imminent_ns: u64,
    pub compound_imminent_prefilter_candidates: u64,
    pub compound_imminent_confirmed_entries: u64,
    pub compound_imminent_hits: u64,
    pub stage_move_gen_ns: u64,
    pub stage_ordering_ns: u64,
    pub stage_eval_ns: u64,
    pub stage_threat_ns: u64,
    pub stage_proof_ns: u64,
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
    pub tt_insert_skips: u64,
    pub beta_cutoffs: u64,
    pub corridor_nodes: u64,
    pub corridor_branch_probes: u64,
    pub corridor_width_exits: u64,
    pub corridor_depth_exits: u64,
    pub corridor_neutral_exits: u64,
    pub corridor_terminal_exits: u64,
    pub corridor_plies_followed: u64,
    pub corridor_own_plies_followed: u64,
    pub corridor_opponent_plies_followed: u64,
    pub corridor_max_depth: u32,
    pub corridor_proof_passes: u64,
    pub corridor_proof_completed: u64,
    pub corridor_proof_checks: u64,
    pub corridor_proof_active: u64,
    pub corridor_proof_quiet: u64,
    pub corridor_proof_static_exits: u64,
    pub corridor_proof_depth_exits: u64,
    pub corridor_proof_deadline_exits: u64,
    pub corridor_proof_terminal_exits: u64,
    pub corridor_proof_terminal_root_candidates: u64,
    pub corridor_proof_terminal_root_winning_candidates: u64,
    pub corridor_proof_terminal_root_losing_candidates: u64,
    pub corridor_proof_terminal_root_overrides: u64,
    pub corridor_proof_terminal_root_move_changes: u64,
    pub corridor_proof_terminal_root_move_confirmations: u64,
    pub corridor_proof_candidates_considered: u64,
    pub corridor_proof_wins: u64,
    pub corridor_proof_losses: u64,
    pub corridor_proof_unknown: u64,
    pub corridor_proof_deadline_skips: u64,
    pub corridor_proof_move_changes: u64,
    pub corridor_proof_move_confirmations: u64,
    pub corridor_proof_candidate_rank_total: u64,
    pub corridor_proof_candidate_rank_max: u64,
    pub corridor_proof_candidate_score_gap_total: u64,
    pub corridor_proof_candidate_score_gap_max: u64,
    pub corridor_proof_win_candidate_rank_total: u64,
    pub corridor_proof_win_candidate_rank_max: u64,
    pub threat_view_shadow_checks: u64,
    pub threat_view_shadow_mismatches: u64,
    pub threat_view_scan_queries: u64,
    pub threat_view_scan_ns: u64,
    pub threat_view_frontier_rebuilds: u64,
    pub threat_view_frontier_rebuild_ns: u64,
    pub threat_view_frontier_queries: u64,
    pub threat_view_frontier_query_ns: u64,
    pub threat_view_frontier_immediate_win_queries: u64,
    pub threat_view_frontier_immediate_win_query_ns: u64,
    pub threat_view_frontier_delta_captures: u64,
    pub threat_view_frontier_delta_capture_ns: u64,
    pub threat_view_frontier_move_fact_updates: u64,
    pub threat_view_frontier_move_fact_update_ns: u64,
    pub threat_view_frontier_annotation_dirty_marks: u64,
    pub threat_view_frontier_annotation_dirty_mark_ns: u64,
    pub threat_view_frontier_clean_annotation_queries: u64,
    pub threat_view_frontier_clean_annotation_query_ns: u64,
    pub threat_view_frontier_dirty_annotation_queries: u64,
    pub threat_view_frontier_dirty_annotation_query_ns: u64,
    pub threat_view_frontier_fallback_annotation_queries: u64,
    pub threat_view_frontier_fallback_annotation_query_ns: u64,
    pub threat_view_frontier_memo_annotation_queries: u64,
    pub threat_view_frontier_memo_annotation_query_ns: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SearchMetricPhase {
    Root,
    Search,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RenjuForbiddenMetricSource {
    SearchGate,
    Pattern,
    Threat,
}

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct SearchStageSnapshot {
    move_gen_ns: u64,
    ordering_ns: u64,
    eval_ns: u64,
    threat_ns: u64,
    proof_ns: u64,
}

impl SearchMetrics {
    pub(super) fn stage_snapshot(&self) -> SearchStageSnapshot {
        SearchStageSnapshot {
            move_gen_ns: self.stage_move_gen_ns,
            ordering_ns: self.stage_ordering_ns,
            eval_ns: self.stage_eval_ns,
            threat_ns: self.stage_threat_ns,
            proof_ns: self.stage_proof_ns,
        }
    }

    pub(super) fn stage_delta_since(&self, before: SearchStageSnapshot) -> u64 {
        self.stage_move_gen_ns
            .saturating_sub(before.move_gen_ns)
            .saturating_add(self.stage_ordering_ns.saturating_sub(before.ordering_ns))
            .saturating_add(self.stage_eval_ns.saturating_sub(before.eval_ns))
            .saturating_add(self.stage_threat_ns.saturating_sub(before.threat_ns))
            .saturating_add(self.stage_proof_ns.saturating_sub(before.proof_ns))
    }

    pub(super) fn record_ordering_scope(&mut self, elapsed: Duration, before: SearchStageSnapshot) {
        let elapsed_ns = duration_ns(elapsed);
        let nested_ns = self.stage_delta_since(before);
        self.stage_ordering_ns = self
            .stage_ordering_ns
            .saturating_add(elapsed_ns.saturating_sub(nested_ns));
    }

    pub(super) fn record_proof_scope(&mut self, elapsed: Duration, before: SearchStageSnapshot) {
        let elapsed_ns = duration_ns(elapsed);
        let nested_ns = self.stage_delta_since(before);
        self.stage_proof_ns = self
            .stage_proof_ns
            .saturating_add(elapsed_ns.saturating_sub(nested_ns));
    }

    pub(super) fn record_static_eval(&mut self, static_eval: StaticEvaluation, elapsed: Duration) {
        self.eval_calls += 1;
        let ns = duration_ns(elapsed);
        self.stage_eval_ns = self.stage_eval_ns.saturating_add(ns);
        match static_eval {
            StaticEvaluation::LineShapeEval => {
                self.line_shape_eval_calls += 1;
                self.line_shape_eval_ns = self.line_shape_eval_ns.saturating_add(ns);
            }
            StaticEvaluation::PatternEval => {
                self.pattern_eval_calls += 1;
                self.pattern_eval_ns = self.pattern_eval_ns.saturating_add(ns);
            }
        }
    }

    pub(super) fn record_pattern_frame_query(&mut self, elapsed: Duration) {
        self.pattern_frame_queries += 1;
        self.pattern_frame_query_ns = self
            .pattern_frame_query_ns
            .saturating_add(duration_ns(elapsed));
    }

    pub(super) fn record_pattern_frame_update(&mut self, elapsed: Duration) {
        let ns = duration_ns(elapsed);
        self.pattern_frame_updates += 1;
        self.pattern_frame_update_ns = self.pattern_frame_update_ns.saturating_add(ns);
        self.stage_eval_ns = self.stage_eval_ns.saturating_add(ns);
    }

    pub(super) fn record_candidates(
        &mut self,
        count: usize,
        elapsed: Duration,
        phase: SearchMetricPhase,
    ) {
        let count = count as u64;
        self.stage_move_gen_ns = self.stage_move_gen_ns.saturating_add(duration_ns(elapsed));
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

    pub(super) fn record_null_cell_cull(
        &mut self,
        checks: usize,
        culled: usize,
        elapsed: Duration,
        phase: SearchMetricPhase,
    ) {
        let checks = checks as u64;
        let culled = culled as u64;
        let ns = duration_ns(elapsed);
        self.stage_move_gen_ns = self.stage_move_gen_ns.saturating_add(ns);
        self.null_cell_cull_checks += checks;
        self.null_cell_cull_ns = self.null_cell_cull_ns.saturating_add(ns);
        self.null_cells_culled += culled;

        match phase {
            SearchMetricPhase::Root => {
                self.root_null_cell_cull_checks += checks;
                self.root_null_cell_cull_ns = self.root_null_cell_cull_ns.saturating_add(ns);
                self.root_null_cells_culled += culled;
            }
            SearchMetricPhase::Search => {
                self.search_null_cell_cull_checks += checks;
                self.search_null_cell_cull_ns = self.search_null_cell_cull_ns.saturating_add(ns);
                self.search_null_cells_culled += culled;
            }
        }
    }

    pub(super) fn record_legality(
        &mut self,
        legal: bool,
        elapsed: Duration,
        phase: SearchMetricPhase,
    ) -> bool {
        self.stage_move_gen_ns = self.stage_move_gen_ns.saturating_add(duration_ns(elapsed));
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

    pub(super) fn renju_forbidden_delta_since(
        before: RenjuForbiddenMetrics,
    ) -> RenjuForbiddenMetrics {
        let after = renju_forbidden_metrics_snapshot();
        RenjuForbiddenMetrics {
            prefilter_checks: after
                .prefilter_checks
                .saturating_sub(before.prefilter_checks),
            prefilter_ns: after.prefilter_ns.saturating_sub(before.prefilter_ns),
            checks: after.checks.saturating_sub(before.checks),
            ns: after.ns.saturating_sub(before.ns),
        }
    }

    pub(super) fn tactical_metrics_delta_since(before: TacticalMetrics) -> TacticalMetrics {
        let after = tactical_metrics_snapshot();
        TacticalMetrics {
            renju_effective_filter_calls: after
                .renju_effective_filter_calls
                .saturating_sub(before.renju_effective_filter_calls),
            renju_effective_filter_ns: after
                .renju_effective_filter_ns
                .saturating_sub(before.renju_effective_filter_ns),
            renju_effective_filter_continuation_checks: after
                .renju_effective_filter_continuation_checks
                .saturating_sub(before.renju_effective_filter_continuation_checks),
            renju_effective_filter_continuation_ns: after
                .renju_effective_filter_continuation_ns
                .saturating_sub(before.renju_effective_filter_continuation_ns),
            compound_imminent_queries: after
                .compound_imminent_queries
                .saturating_sub(before.compound_imminent_queries),
            compound_imminent_ns: after
                .compound_imminent_ns
                .saturating_sub(before.compound_imminent_ns),
            compound_imminent_prefilter_candidates: after
                .compound_imminent_prefilter_candidates
                .saturating_sub(before.compound_imminent_prefilter_candidates),
            compound_imminent_confirmed_entries: after
                .compound_imminent_confirmed_entries
                .saturating_sub(before.compound_imminent_confirmed_entries),
            compound_imminent_hits: after
                .compound_imminent_hits
                .saturating_sub(before.compound_imminent_hits),
        }
    }

    pub(super) fn record_renju_forbidden_source_delta(
        &mut self,
        source: RenjuForbiddenMetricSource,
        before: RenjuForbiddenMetrics,
    ) {
        let delta = Self::renju_forbidden_delta_since(before);
        match source {
            RenjuForbiddenMetricSource::SearchGate => {
                self.renju_forbidden_search_gate_checks = self
                    .renju_forbidden_search_gate_checks
                    .saturating_add(delta.checks);
                self.renju_forbidden_search_gate_ns =
                    self.renju_forbidden_search_gate_ns.saturating_add(delta.ns);
            }
            RenjuForbiddenMetricSource::Pattern => {
                self.renju_forbidden_pattern_checks = self
                    .renju_forbidden_pattern_checks
                    .saturating_add(delta.checks);
                self.renju_forbidden_pattern_ns =
                    self.renju_forbidden_pattern_ns.saturating_add(delta.ns);
            }
            RenjuForbiddenMetricSource::Threat => {
                self.renju_forbidden_threat_checks = self
                    .renju_forbidden_threat_checks
                    .saturating_add(delta.checks);
                self.renju_forbidden_threat_ns =
                    self.renju_forbidden_threat_ns.saturating_add(delta.ns);
            }
        }
    }

    pub(super) fn record_renju_forbidden_total_delta(&mut self, before: RenjuForbiddenMetrics) {
        let delta = Self::renju_forbidden_delta_since(before);
        self.renju_forbidden_prefilter_checks = self
            .renju_forbidden_prefilter_checks
            .saturating_add(delta.prefilter_checks);
        self.renju_forbidden_prefilter_ns = self
            .renju_forbidden_prefilter_ns
            .saturating_add(delta.prefilter_ns);
        self.renju_forbidden_checks = self.renju_forbidden_checks.saturating_add(delta.checks);
        self.renju_forbidden_ns = self.renju_forbidden_ns.saturating_add(delta.ns);

        let known_checks = self
            .renju_forbidden_search_gate_checks
            .saturating_add(self.renju_forbidden_pattern_checks)
            .saturating_add(self.renju_forbidden_threat_checks);
        let known_ns = self
            .renju_forbidden_search_gate_ns
            .saturating_add(self.renju_forbidden_pattern_ns)
            .saturating_add(self.renju_forbidden_threat_ns);
        self.renju_forbidden_other_checks = self
            .renju_forbidden_other_checks
            .saturating_add(delta.checks.saturating_sub(known_checks));
        self.renju_forbidden_other_ns = self
            .renju_forbidden_other_ns
            .saturating_add(delta.ns.saturating_sub(known_ns));
    }

    pub(super) fn record_tactical_metric_total_delta(&mut self, before: TacticalMetrics) {
        let delta = Self::tactical_metrics_delta_since(before);
        self.renju_effective_filter_calls = self
            .renju_effective_filter_calls
            .saturating_add(delta.renju_effective_filter_calls);
        self.renju_effective_filter_ns = self
            .renju_effective_filter_ns
            .saturating_add(delta.renju_effective_filter_ns);
        self.renju_effective_filter_continuation_checks = self
            .renju_effective_filter_continuation_checks
            .saturating_add(delta.renju_effective_filter_continuation_checks);
        self.renju_effective_filter_continuation_ns = self
            .renju_effective_filter_continuation_ns
            .saturating_add(delta.renju_effective_filter_continuation_ns);
        self.compound_imminent_queries = self
            .compound_imminent_queries
            .saturating_add(delta.compound_imminent_queries);
        self.compound_imminent_ns = self
            .compound_imminent_ns
            .saturating_add(delta.compound_imminent_ns);
        self.compound_imminent_prefilter_candidates = self
            .compound_imminent_prefilter_candidates
            .saturating_add(delta.compound_imminent_prefilter_candidates);
        self.compound_imminent_confirmed_entries = self
            .compound_imminent_confirmed_entries
            .saturating_add(delta.compound_imminent_confirmed_entries);
        self.compound_imminent_hits = self
            .compound_imminent_hits
            .saturating_add(delta.compound_imminent_hits);
    }

    pub(super) fn record_tactical_annotation(&mut self, phase: SearchMetricPhase) {
        self.tactical_annotations += 1;
        match phase {
            SearchMetricPhase::Root => self.root_tactical_annotations += 1,
            SearchMetricPhase::Search => self.search_tactical_annotations += 1,
        }
    }

    pub(super) fn record_child_limit(
        &mut self,
        before: usize,
        after: usize,
        phase: SearchMetricPhase,
    ) {
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

    pub(super) fn record_corridor_ply(&mut self, side: CorridorSide) {
        self.corridor_plies_followed += 1;
        match side {
            CorridorSide::Own => self.corridor_own_plies_followed += 1,
            CorridorSide::Opponent => self.corridor_opponent_plies_followed += 1,
        }
    }

    pub(super) fn record_corridor_node(&mut self, depth_reached: u32) {
        self.corridor_nodes += 1;
        self.corridor_max_depth = self.corridor_max_depth.max(depth_reached);
    }

    pub(super) fn record_corridor_proof_check(&mut self, active: bool) {
        self.corridor_proof_checks += 1;
        if active {
            self.corridor_proof_active += 1;
        } else {
            self.corridor_proof_quiet += 1;
        }
    }

    pub(super) fn record_threat_view_scan(&mut self, elapsed: Duration) {
        let ns = duration_ns(elapsed);
        self.threat_view_scan_queries += 1;
        self.threat_view_scan_ns = self.threat_view_scan_ns.saturating_add(ns);
        self.stage_threat_ns = self.stage_threat_ns.saturating_add(ns);
    }

    pub(super) fn record_threat_view_frontier_rebuild(&mut self, elapsed: Duration) {
        let ns = duration_ns(elapsed);
        self.threat_view_frontier_rebuilds += 1;
        self.threat_view_frontier_rebuild_ns =
            self.threat_view_frontier_rebuild_ns.saturating_add(ns);
        self.stage_threat_ns = self.stage_threat_ns.saturating_add(ns);
    }

    pub(super) fn record_threat_view_frontier_query(&mut self, elapsed: Duration) {
        let ns = duration_ns(elapsed);
        self.threat_view_frontier_queries += 1;
        self.threat_view_frontier_query_ns = self.threat_view_frontier_query_ns.saturating_add(ns);
        self.stage_threat_ns = self.stage_threat_ns.saturating_add(ns);
    }

    pub(super) fn record_threat_view_frontier_immediate_win_query(&mut self, elapsed: Duration) {
        self.record_threat_view_frontier_query(elapsed);
        self.threat_view_frontier_immediate_win_queries += 1;
        self.threat_view_frontier_immediate_win_query_ns = self
            .threat_view_frontier_immediate_win_query_ns
            .saturating_add(duration_ns(elapsed));
    }

    pub(super) fn record_threat_view_frontier_update_parts(
        &mut self,
        timings: FrontierUpdateTimings,
    ) {
        if let Some(elapsed) = timings.delta_capture {
            let ns = duration_ns(elapsed);
            self.threat_view_frontier_delta_captures += 1;
            self.threat_view_frontier_delta_capture_ns = self
                .threat_view_frontier_delta_capture_ns
                .saturating_add(ns);
            self.stage_threat_ns = self.stage_threat_ns.saturating_add(ns);
        }
        if let Some(elapsed) = timings.move_fact_update {
            let ns = duration_ns(elapsed);
            self.threat_view_frontier_move_fact_updates += 1;
            self.threat_view_frontier_move_fact_update_ns = self
                .threat_view_frontier_move_fact_update_ns
                .saturating_add(ns);
            self.stage_threat_ns = self.stage_threat_ns.saturating_add(ns);
        }
        if let Some(elapsed) = timings.annotation_dirty_mark {
            let ns = duration_ns(elapsed);
            self.threat_view_frontier_annotation_dirty_marks += 1;
            self.threat_view_frontier_annotation_dirty_mark_ns = self
                .threat_view_frontier_annotation_dirty_mark_ns
                .saturating_add(ns);
            self.stage_threat_ns = self.stage_threat_ns.saturating_add(ns);
        }
    }

    pub(super) fn record_threat_view_frontier_annotation_query(
        &mut self,
        elapsed: Duration,
        source: FrontierAnnotationSource,
    ) {
        self.record_threat_view_frontier_query(elapsed);
        match source {
            FrontierAnnotationSource::CleanCache => {
                self.threat_view_frontier_clean_annotation_queries += 1;
                self.threat_view_frontier_clean_annotation_query_ns = self
                    .threat_view_frontier_clean_annotation_query_ns
                    .saturating_add(duration_ns(elapsed));
            }
            FrontierAnnotationSource::DirtyRecompute => {
                self.threat_view_frontier_dirty_annotation_queries += 1;
                self.threat_view_frontier_dirty_annotation_query_ns = self
                    .threat_view_frontier_dirty_annotation_query_ns
                    .saturating_add(duration_ns(elapsed));
            }
            FrontierAnnotationSource::Fallback => {
                self.threat_view_frontier_fallback_annotation_queries += 1;
                self.threat_view_frontier_fallback_annotation_query_ns = self
                    .threat_view_frontier_fallback_annotation_query_ns
                    .saturating_add(duration_ns(elapsed));
            }
        }
    }

    pub(super) fn record_threat_view_frontier_memo_annotation_query(&mut self, elapsed: Duration) {
        let ns = duration_ns(elapsed);
        self.threat_view_frontier_memo_annotation_queries += 1;
        self.threat_view_frontier_memo_annotation_query_ns = self
            .threat_view_frontier_memo_annotation_query_ns
            .saturating_add(ns);
        self.stage_threat_ns = self.stage_threat_ns.saturating_add(ns);
    }

    pub(super) fn trace(self) -> serde_json::Value {
        serde_json::to_value(self).expect("search metrics should serialize")
    }
}

pub(super) fn duration_ns(elapsed: Duration) -> u64 {
    u64::try_from(elapsed.as_nanos()).unwrap_or(u64::MAX).max(1)
}
