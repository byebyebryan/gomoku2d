use super::*;

#[derive(Debug, Clone, Default)]
pub(super) struct SideStatsAccumulator {
    move_count: u32,
    search_move_count: u32,
    total_time_ms: u64,
    search_nodes: u64,
    safety_nodes: u64,
    corridor_nodes: u64,
    corridor_branch_probes: u64,
    corridor_max_depth: u32,
    corridor_width_exits: u64,
    corridor_depth_exits: u64,
    corridor_neutral_exits: u64,
    corridor_terminal_exits: u64,
    corridor_plies_followed: u64,
    corridor_own_plies_followed: u64,
    corridor_opponent_plies_followed: u64,
    corridor_proof_passes: u64,
    corridor_proof_completed: u64,
    corridor_proof_checks: u64,
    corridor_proof_active: u64,
    corridor_proof_quiet: u64,
    corridor_proof_static_exits: u64,
    corridor_proof_depth_exits: u64,
    corridor_proof_deadline_exits: u64,
    corridor_proof_terminal_exits: u64,
    corridor_proof_terminal_root_candidates: u64,
    corridor_proof_terminal_root_winning_candidates: u64,
    corridor_proof_terminal_root_losing_candidates: u64,
    corridor_proof_terminal_root_overrides: u64,
    corridor_proof_terminal_root_move_changes: u64,
    corridor_proof_terminal_root_move_confirmations: u64,
    corridor_proof_candidates_considered: u64,
    corridor_proof_wins: u64,
    corridor_proof_losses: u64,
    corridor_proof_unknown: u64,
    corridor_proof_deadline_skips: u64,
    corridor_proof_move_changes: u64,
    corridor_proof_move_confirmations: u64,
    corridor_proof_candidate_rank_total: u64,
    corridor_proof_candidate_rank_max: u64,
    corridor_proof_candidate_score_gap_total: u64,
    corridor_proof_candidate_score_gap_max: u64,
    corridor_proof_win_candidate_rank_total: u64,
    corridor_proof_win_candidate_rank_max: u64,
    total_nodes: u64,
    eval_calls: u64,
    line_shape_eval_calls: u64,
    line_shape_eval_ns: u64,
    pattern_eval_calls: u64,
    pattern_eval_ns: u64,
    pattern_frame_queries: u64,
    pattern_frame_query_ns: u64,
    pattern_frame_updates: u64,
    pattern_frame_update_ns: u64,
    pattern_frame_shadow_checks: u64,
    pattern_frame_shadow_mismatches: u64,
    candidate_generations: u64,
    candidate_moves_total: u64,
    candidate_moves_max: u64,
    root_candidate_generations: u64,
    root_candidate_moves_total: u64,
    root_candidate_moves_max: u64,
    search_candidate_generations: u64,
    search_candidate_moves_total: u64,
    search_candidate_moves_max: u64,
    legality_checks: u64,
    illegal_moves_skipped: u64,
    root_legality_checks: u64,
    root_illegal_moves_skipped: u64,
    search_legality_checks: u64,
    search_illegal_moves_skipped: u64,
    renju_forbidden_prefilter_checks: u64,
    renju_forbidden_prefilter_ns: u64,
    renju_forbidden_checks: u64,
    renju_forbidden_ns: u64,
    renju_forbidden_search_gate_checks: u64,
    renju_forbidden_search_gate_ns: u64,
    renju_forbidden_pattern_checks: u64,
    renju_forbidden_pattern_ns: u64,
    renju_forbidden_threat_checks: u64,
    renju_forbidden_threat_ns: u64,
    renju_forbidden_other_checks: u64,
    renju_forbidden_other_ns: u64,
    renju_effective_filter_calls: u64,
    renju_effective_filter_ns: u64,
    renju_effective_filter_continuation_checks: u64,
    renju_effective_filter_continuation_ns: u64,
    stage_move_gen_ns: u64,
    stage_ordering_ns: u64,
    stage_eval_ns: u64,
    stage_threat_ns: u64,
    stage_proof_ns: u64,
    tactical_annotations: u64,
    root_tactical_annotations: u64,
    search_tactical_annotations: u64,
    threat_view_shadow_checks: u64,
    threat_view_shadow_mismatches: u64,
    threat_view_scan_queries: u64,
    threat_view_scan_ns: u64,
    threat_view_frontier_rebuilds: u64,
    threat_view_frontier_rebuild_ns: u64,
    threat_view_frontier_queries: u64,
    threat_view_frontier_query_ns: u64,
    threat_view_frontier_immediate_win_queries: u64,
    threat_view_frontier_immediate_win_query_ns: u64,
    threat_view_frontier_delta_captures: u64,
    threat_view_frontier_delta_capture_ns: u64,
    threat_view_frontier_move_fact_updates: u64,
    threat_view_frontier_move_fact_update_ns: u64,
    threat_view_frontier_annotation_dirty_marks: u64,
    threat_view_frontier_annotation_dirty_mark_ns: u64,
    threat_view_frontier_clean_annotation_queries: u64,
    threat_view_frontier_clean_annotation_query_ns: u64,
    threat_view_frontier_dirty_annotation_queries: u64,
    threat_view_frontier_dirty_annotation_query_ns: u64,
    threat_view_frontier_fallback_annotation_queries: u64,
    threat_view_frontier_fallback_annotation_query_ns: u64,
    threat_view_frontier_memo_annotation_queries: u64,
    threat_view_frontier_memo_annotation_query_ns: u64,
    child_limit_applications: u64,
    root_child_limit_applications: u64,
    search_child_limit_applications: u64,
    child_cap_hits: u64,
    root_child_cap_hits: u64,
    search_child_cap_hits: u64,
    child_moves_before_total: u64,
    root_child_moves_before_total: u64,
    search_child_moves_before_total: u64,
    child_moves_before_max: u64,
    root_child_moves_before_max: u64,
    search_child_moves_before_max: u64,
    child_moves_after_total: u64,
    root_child_moves_after_total: u64,
    search_child_moves_after_total: u64,
    child_moves_after_max: u64,
    root_child_moves_after_max: u64,
    search_child_moves_after_max: u64,
    tt_hits: u64,
    tt_cutoffs: u64,
    beta_cutoffs: u64,
    depth_sum: u64,
    max_depth: u32,
    effective_depth_sum: u64,
    max_effective_depth: u32,
    depth_reached_counts: BTreeMap<u32, u32>,
    budget_exhausted_count: u32,
    pooled_budget_moves: u32,
    pooled_budget_over_base_count: u32,
    pooled_budget_reserve_exhausted_count: u32,
    pooled_budget_reserve_before_total_ms: u64,
    pooled_budget_reserve_after_total_ms: u64,
    pooled_budget_min_reserve_after_ms: Option<u64>,
    pooled_budget_max_move_budget_ms: u64,
}

impl SideStatsAccumulator {
    pub(super) fn record_move(&mut self, time_ms: u64, trace: Option<&Value>) {
        self.move_count += 1;
        self.total_time_ms += time_ms;

        let Some(trace) = trace else {
            return;
        };

        self.search_move_count += 1;
        self.search_nodes += trace_value_u64(trace, "nodes");
        self.safety_nodes += trace_value_u64(trace, "safety_nodes");
        self.total_nodes += trace_value_u64(trace, "total_nodes");
        if let Some(corridor) = trace.get("corridor") {
            self.corridor_nodes += trace_value_u64(corridor, "search_nodes");
            self.corridor_branch_probes += trace_value_u64(corridor, "branch_probes");
            self.corridor_max_depth = self
                .corridor_max_depth
                .max(trace_value_u64(corridor, "max_depth_reached") as u32);
        }
        if let Some(metrics) = trace.get("metrics") {
            self.eval_calls += trace_value_u64(metrics, "eval_calls");
            self.line_shape_eval_calls += trace_value_u64(metrics, "line_shape_eval_calls");
            self.line_shape_eval_ns += trace_value_u64(metrics, "line_shape_eval_ns");
            self.pattern_eval_calls += trace_value_u64(metrics, "pattern_eval_calls");
            self.pattern_eval_ns += trace_value_u64(metrics, "pattern_eval_ns");
            self.pattern_frame_queries += trace_value_u64(metrics, "pattern_frame_queries");
            self.pattern_frame_query_ns += trace_value_u64(metrics, "pattern_frame_query_ns");
            self.pattern_frame_updates += trace_value_u64(metrics, "pattern_frame_updates");
            self.pattern_frame_update_ns += trace_value_u64(metrics, "pattern_frame_update_ns");
            self.pattern_frame_shadow_checks +=
                trace_value_u64(metrics, "pattern_frame_shadow_checks");
            self.pattern_frame_shadow_mismatches +=
                trace_value_u64(metrics, "pattern_frame_shadow_mismatches");
            self.candidate_generations += trace_value_u64(metrics, "candidate_generations");
            self.candidate_moves_total += trace_value_u64(metrics, "candidate_moves_total");
            self.candidate_moves_max = self
                .candidate_moves_max
                .max(trace_value_u64(metrics, "candidate_moves_max"));
            self.root_candidate_generations +=
                trace_value_u64(metrics, "root_candidate_generations");
            self.root_candidate_moves_total +=
                trace_value_u64(metrics, "root_candidate_moves_total");
            self.root_candidate_moves_max = self
                .root_candidate_moves_max
                .max(trace_value_u64(metrics, "root_candidate_moves_max"));
            self.search_candidate_generations +=
                trace_value_u64(metrics, "search_candidate_generations");
            self.search_candidate_moves_total +=
                trace_value_u64(metrics, "search_candidate_moves_total");
            self.search_candidate_moves_max = self
                .search_candidate_moves_max
                .max(trace_value_u64(metrics, "search_candidate_moves_max"));
            self.legality_checks += trace_value_u64(metrics, "legality_checks");
            self.illegal_moves_skipped += trace_value_u64(metrics, "illegal_moves_skipped");
            self.root_legality_checks += trace_value_u64(metrics, "root_legality_checks");
            self.root_illegal_moves_skipped +=
                trace_value_u64(metrics, "root_illegal_moves_skipped");
            self.search_legality_checks += trace_value_u64(metrics, "search_legality_checks");
            self.search_illegal_moves_skipped +=
                trace_value_u64(metrics, "search_illegal_moves_skipped");
            self.renju_forbidden_prefilter_checks +=
                trace_value_u64(metrics, "renju_forbidden_prefilter_checks");
            self.renju_forbidden_prefilter_ns +=
                trace_value_u64(metrics, "renju_forbidden_prefilter_ns");
            self.renju_forbidden_checks += trace_value_u64(metrics, "renju_forbidden_checks");
            self.renju_forbidden_ns += trace_value_u64(metrics, "renju_forbidden_ns");
            self.renju_forbidden_search_gate_checks +=
                trace_value_u64(metrics, "renju_forbidden_search_gate_checks");
            self.renju_forbidden_search_gate_ns +=
                trace_value_u64(metrics, "renju_forbidden_search_gate_ns");
            self.renju_forbidden_pattern_checks +=
                trace_value_u64(metrics, "renju_forbidden_pattern_checks");
            self.renju_forbidden_pattern_ns +=
                trace_value_u64(metrics, "renju_forbidden_pattern_ns");
            self.renju_forbidden_threat_checks +=
                trace_value_u64(metrics, "renju_forbidden_threat_checks");
            self.renju_forbidden_threat_ns += trace_value_u64(metrics, "renju_forbidden_threat_ns");
            self.renju_forbidden_other_checks +=
                trace_value_u64(metrics, "renju_forbidden_other_checks");
            self.renju_forbidden_other_ns += trace_value_u64(metrics, "renju_forbidden_other_ns");
            self.renju_effective_filter_calls +=
                trace_value_u64(metrics, "renju_effective_filter_calls");
            self.renju_effective_filter_ns += trace_value_u64(metrics, "renju_effective_filter_ns");
            self.renju_effective_filter_continuation_checks +=
                trace_value_u64(metrics, "renju_effective_filter_continuation_checks");
            self.renju_effective_filter_continuation_ns +=
                trace_value_u64(metrics, "renju_effective_filter_continuation_ns");
            self.stage_move_gen_ns += trace_value_u64(metrics, "stage_move_gen_ns");
            self.stage_ordering_ns += trace_value_u64(metrics, "stage_ordering_ns");
            self.stage_eval_ns += trace_value_u64(metrics, "stage_eval_ns");
            self.stage_threat_ns += trace_value_u64(metrics, "stage_threat_ns");
            self.stage_proof_ns += trace_value_u64(metrics, "stage_proof_ns");
            self.tactical_annotations += trace_value_u64(metrics, "tactical_annotations");
            self.root_tactical_annotations += trace_value_u64(metrics, "root_tactical_annotations");
            self.search_tactical_annotations +=
                trace_value_u64(metrics, "search_tactical_annotations");
            self.threat_view_shadow_checks += trace_value_u64(metrics, "threat_view_shadow_checks");
            self.threat_view_shadow_mismatches +=
                trace_value_u64(metrics, "threat_view_shadow_mismatches");
            self.threat_view_scan_queries += trace_value_u64(metrics, "threat_view_scan_queries");
            self.threat_view_scan_ns += trace_value_u64(metrics, "threat_view_scan_ns");
            self.threat_view_frontier_rebuilds +=
                trace_value_u64(metrics, "threat_view_frontier_rebuilds");
            self.threat_view_frontier_rebuild_ns +=
                trace_value_u64(metrics, "threat_view_frontier_rebuild_ns");
            self.threat_view_frontier_queries +=
                trace_value_u64(metrics, "threat_view_frontier_queries");
            self.threat_view_frontier_query_ns +=
                trace_value_u64(metrics, "threat_view_frontier_query_ns");
            self.threat_view_frontier_immediate_win_queries +=
                trace_value_u64(metrics, "threat_view_frontier_immediate_win_queries");
            self.threat_view_frontier_immediate_win_query_ns +=
                trace_value_u64(metrics, "threat_view_frontier_immediate_win_query_ns");
            self.threat_view_frontier_delta_captures +=
                trace_value_u64(metrics, "threat_view_frontier_delta_captures");
            self.threat_view_frontier_delta_capture_ns +=
                trace_value_u64(metrics, "threat_view_frontier_delta_capture_ns");
            self.threat_view_frontier_move_fact_updates +=
                trace_value_u64(metrics, "threat_view_frontier_move_fact_updates");
            self.threat_view_frontier_move_fact_update_ns +=
                trace_value_u64(metrics, "threat_view_frontier_move_fact_update_ns");
            self.threat_view_frontier_annotation_dirty_marks +=
                trace_value_u64(metrics, "threat_view_frontier_annotation_dirty_marks");
            self.threat_view_frontier_annotation_dirty_mark_ns +=
                trace_value_u64(metrics, "threat_view_frontier_annotation_dirty_mark_ns");
            self.threat_view_frontier_clean_annotation_queries +=
                trace_value_u64(metrics, "threat_view_frontier_clean_annotation_queries");
            self.threat_view_frontier_clean_annotation_query_ns +=
                trace_value_u64(metrics, "threat_view_frontier_clean_annotation_query_ns");
            self.threat_view_frontier_dirty_annotation_queries +=
                trace_value_u64(metrics, "threat_view_frontier_dirty_annotation_queries");
            self.threat_view_frontier_dirty_annotation_query_ns +=
                trace_value_u64(metrics, "threat_view_frontier_dirty_annotation_query_ns");
            self.threat_view_frontier_fallback_annotation_queries +=
                trace_value_u64(metrics, "threat_view_frontier_fallback_annotation_queries");
            self.threat_view_frontier_fallback_annotation_query_ns +=
                trace_value_u64(metrics, "threat_view_frontier_fallback_annotation_query_ns");
            self.threat_view_frontier_memo_annotation_queries +=
                trace_value_u64(metrics, "threat_view_frontier_memo_annotation_queries");
            self.threat_view_frontier_memo_annotation_query_ns +=
                trace_value_u64(metrics, "threat_view_frontier_memo_annotation_query_ns");
            self.child_limit_applications += trace_value_u64(metrics, "child_limit_applications");
            self.root_child_limit_applications +=
                trace_value_u64(metrics, "root_child_limit_applications");
            self.search_child_limit_applications +=
                trace_value_u64(metrics, "search_child_limit_applications");
            self.child_cap_hits += trace_value_u64(metrics, "child_cap_hits");
            self.root_child_cap_hits += trace_value_u64(metrics, "root_child_cap_hits");
            self.search_child_cap_hits += trace_value_u64(metrics, "search_child_cap_hits");
            self.child_moves_before_total += trace_value_u64(metrics, "child_moves_before_total");
            self.root_child_moves_before_total +=
                trace_value_u64(metrics, "root_child_moves_before_total");
            self.search_child_moves_before_total +=
                trace_value_u64(metrics, "search_child_moves_before_total");
            self.child_moves_before_max = self
                .child_moves_before_max
                .max(trace_value_u64(metrics, "child_moves_before_max"));
            self.root_child_moves_before_max = self
                .root_child_moves_before_max
                .max(trace_value_u64(metrics, "root_child_moves_before_max"));
            self.search_child_moves_before_max = self
                .search_child_moves_before_max
                .max(trace_value_u64(metrics, "search_child_moves_before_max"));
            self.child_moves_after_total += trace_value_u64(metrics, "child_moves_after_total");
            self.root_child_moves_after_total +=
                trace_value_u64(metrics, "root_child_moves_after_total");
            self.search_child_moves_after_total +=
                trace_value_u64(metrics, "search_child_moves_after_total");
            self.child_moves_after_max = self
                .child_moves_after_max
                .max(trace_value_u64(metrics, "child_moves_after_max"));
            self.root_child_moves_after_max = self
                .root_child_moves_after_max
                .max(trace_value_u64(metrics, "root_child_moves_after_max"));
            self.search_child_moves_after_max = self
                .search_child_moves_after_max
                .max(trace_value_u64(metrics, "search_child_moves_after_max"));
            self.tt_hits += trace_value_u64(metrics, "tt_hits");
            self.tt_cutoffs += trace_value_u64(metrics, "tt_cutoffs");
            self.beta_cutoffs += trace_value_u64(metrics, "beta_cutoffs");
            self.corridor_width_exits += trace_value_u64(metrics, "corridor_width_exits");
            self.corridor_depth_exits += trace_value_u64(metrics, "corridor_depth_exits");
            self.corridor_neutral_exits += trace_value_u64(metrics, "corridor_neutral_exits");
            self.corridor_terminal_exits += trace_value_u64(metrics, "corridor_terminal_exits");
            self.corridor_plies_followed += trace_value_u64(metrics, "corridor_plies_followed");
            self.corridor_own_plies_followed +=
                trace_value_u64(metrics, "corridor_own_plies_followed");
            self.corridor_opponent_plies_followed +=
                trace_value_u64(metrics, "corridor_opponent_plies_followed");
            self.corridor_proof_passes += trace_value_u64(metrics, "corridor_proof_passes");
            self.corridor_proof_completed += trace_value_u64(metrics, "corridor_proof_completed");
            self.corridor_proof_checks += trace_value_u64(metrics, "corridor_proof_checks");
            self.corridor_proof_active += trace_value_u64(metrics, "corridor_proof_active");
            self.corridor_proof_quiet += trace_value_u64(metrics, "corridor_proof_quiet");
            self.corridor_proof_static_exits +=
                trace_value_u64(metrics, "corridor_proof_static_exits");
            self.corridor_proof_depth_exits +=
                trace_value_u64(metrics, "corridor_proof_depth_exits");
            self.corridor_proof_deadline_exits +=
                trace_value_u64(metrics, "corridor_proof_deadline_exits");
            self.corridor_proof_terminal_exits +=
                trace_value_u64(metrics, "corridor_proof_terminal_exits");
            self.corridor_proof_terminal_root_candidates +=
                trace_value_u64(metrics, "corridor_proof_terminal_root_candidates");
            self.corridor_proof_terminal_root_winning_candidates +=
                trace_value_u64(metrics, "corridor_proof_terminal_root_winning_candidates");
            self.corridor_proof_terminal_root_losing_candidates +=
                trace_value_u64(metrics, "corridor_proof_terminal_root_losing_candidates");
            self.corridor_proof_terminal_root_overrides +=
                trace_value_u64(metrics, "corridor_proof_terminal_root_overrides");
            self.corridor_proof_terminal_root_move_changes +=
                trace_value_u64(metrics, "corridor_proof_terminal_root_move_changes");
            self.corridor_proof_terminal_root_move_confirmations +=
                trace_value_u64(metrics, "corridor_proof_terminal_root_move_confirmations");
            self.corridor_proof_candidates_considered +=
                trace_value_u64(metrics, "corridor_proof_candidates_considered");
            self.corridor_proof_wins += trace_value_u64(metrics, "corridor_proof_wins");
            self.corridor_proof_losses += trace_value_u64(metrics, "corridor_proof_losses");
            self.corridor_proof_unknown += trace_value_u64(metrics, "corridor_proof_unknown");
            self.corridor_proof_deadline_skips +=
                trace_value_u64(metrics, "corridor_proof_deadline_skips");
            self.corridor_proof_move_changes +=
                trace_value_u64(metrics, "corridor_proof_move_changes");
            self.corridor_proof_move_confirmations +=
                trace_value_u64(metrics, "corridor_proof_move_confirmations");
            self.corridor_proof_candidate_rank_total +=
                trace_value_u64(metrics, "corridor_proof_candidate_rank_total");
            self.corridor_proof_candidate_rank_max = self.corridor_proof_candidate_rank_max.max(
                trace_value_u64(metrics, "corridor_proof_candidate_rank_max"),
            );
            self.corridor_proof_candidate_score_gap_total +=
                trace_value_u64(metrics, "corridor_proof_candidate_score_gap_total");
            self.corridor_proof_candidate_score_gap_max = self
                .corridor_proof_candidate_score_gap_max
                .max(trace_value_u64(
                    metrics,
                    "corridor_proof_candidate_score_gap_max",
                ));
            self.corridor_proof_win_candidate_rank_total +=
                trace_value_u64(metrics, "corridor_proof_win_candidate_rank_total");
            self.corridor_proof_win_candidate_rank_max = self
                .corridor_proof_win_candidate_rank_max
                .max(trace_value_u64(
                    metrics,
                    "corridor_proof_win_candidate_rank_max",
                ));
        }
        if let Some(depth) = trace.get("depth").and_then(Value::as_u64) {
            self.depth_sum += depth;
            self.max_depth = self.max_depth.max(depth as u32);
            *self.depth_reached_counts.entry(depth as u32).or_insert(0) += 1;
            let effective_depth = trace
                .get("effective_depth")
                .and_then(Value::as_u64)
                .unwrap_or(depth);
            self.effective_depth_sum += effective_depth;
            self.max_effective_depth = self.max_effective_depth.max(effective_depth as u32);
        }
        if trace
            .get("budget_exhausted")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            self.budget_exhausted_count += 1;
        }
        if let Some(pool) = trace.get("budget_pool") {
            self.pooled_budget_moves += 1;
            let reserve_before_ms = trace_value_u64(pool, "reserve_before_ms");
            let reserve_after_ms = trace_value_u64(pool, "reserve_after_ms");
            self.pooled_budget_reserve_before_total_ms += reserve_before_ms;
            self.pooled_budget_reserve_after_total_ms += reserve_after_ms;
            self.pooled_budget_min_reserve_after_ms = Some(
                self.pooled_budget_min_reserve_after_ms
                    .map_or(reserve_after_ms, |current| current.min(reserve_after_ms)),
            );
            self.pooled_budget_max_move_budget_ms = self
                .pooled_budget_max_move_budget_ms
                .max(trace_value_u64(pool, "move_budget_ms"));
            if trace_value_u64(pool, "consumed_ms") > trace_value_u64(pool, "base_ms")
                || reserve_after_ms < reserve_before_ms
            {
                self.pooled_budget_over_base_count += 1;
            }
            if pool
                .get("budget_exhausted")
                .or_else(|| pool.get("reserve_exhausted"))
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                self.pooled_budget_reserve_exhausted_count += 1;
            }
        }
    }

    pub(super) fn add_report(&mut self, stats: &SideStatsReport) {
        self.move_count += stats.move_count;
        self.search_move_count += stats.search_move_count;
        self.total_time_ms += stats.total_time_ms;
        self.search_nodes += stats.search_nodes;
        self.safety_nodes += stats.safety_nodes;
        self.corridor_nodes += stats.corridor_nodes;
        self.corridor_branch_probes += stats.corridor_branch_probes;
        self.corridor_max_depth = self.corridor_max_depth.max(stats.corridor_max_depth);
        self.corridor_width_exits += stats.corridor_width_exits;
        self.corridor_depth_exits += stats.corridor_depth_exits;
        self.corridor_neutral_exits += stats.corridor_neutral_exits;
        self.corridor_terminal_exits += stats.corridor_terminal_exits;
        self.corridor_plies_followed += stats.corridor_plies_followed;
        self.corridor_own_plies_followed += stats.corridor_own_plies_followed;
        self.corridor_opponent_plies_followed += stats.corridor_opponent_plies_followed;
        self.corridor_proof_passes += stats.corridor_proof_passes;
        self.corridor_proof_completed += stats.corridor_proof_completed;
        self.corridor_proof_checks += stats.corridor_proof_checks;
        self.corridor_proof_active += stats.corridor_proof_active;
        self.corridor_proof_quiet += stats.corridor_proof_quiet;
        self.corridor_proof_static_exits += stats.corridor_proof_static_exits;
        self.corridor_proof_depth_exits += stats.corridor_proof_depth_exits;
        self.corridor_proof_deadline_exits += stats.corridor_proof_deadline_exits;
        self.corridor_proof_terminal_exits += stats.corridor_proof_terminal_exits;
        self.corridor_proof_terminal_root_candidates +=
            stats.corridor_proof_terminal_root_candidates;
        self.corridor_proof_terminal_root_winning_candidates +=
            stats.corridor_proof_terminal_root_winning_candidates;
        self.corridor_proof_terminal_root_losing_candidates +=
            stats.corridor_proof_terminal_root_losing_candidates;
        self.corridor_proof_terminal_root_overrides += stats.corridor_proof_terminal_root_overrides;
        self.corridor_proof_terminal_root_move_changes +=
            stats.corridor_proof_terminal_root_move_changes;
        self.corridor_proof_terminal_root_move_confirmations +=
            stats.corridor_proof_terminal_root_move_confirmations;
        self.corridor_proof_candidates_considered += stats.corridor_proof_candidates_considered;
        self.corridor_proof_wins += stats.corridor_proof_wins;
        self.corridor_proof_losses += stats.corridor_proof_losses;
        self.corridor_proof_unknown += stats.corridor_proof_unknown;
        self.corridor_proof_deadline_skips += stats.corridor_proof_deadline_skips;
        self.corridor_proof_move_changes += stats.corridor_proof_move_changes;
        self.corridor_proof_move_confirmations += stats.corridor_proof_move_confirmations;
        self.corridor_proof_candidate_rank_total += stats.corridor_proof_candidate_rank_total;
        self.corridor_proof_candidate_rank_max = self
            .corridor_proof_candidate_rank_max
            .max(stats.corridor_proof_candidate_rank_max);
        self.corridor_proof_candidate_score_gap_total +=
            stats.corridor_proof_candidate_score_gap_total;
        self.corridor_proof_candidate_score_gap_max = self
            .corridor_proof_candidate_score_gap_max
            .max(stats.corridor_proof_candidate_score_gap_max);
        self.corridor_proof_win_candidate_rank_total +=
            stats.corridor_proof_win_candidate_rank_total;
        self.corridor_proof_win_candidate_rank_max = self
            .corridor_proof_win_candidate_rank_max
            .max(stats.corridor_proof_win_candidate_rank_max);
        self.total_nodes += stats.total_nodes;
        self.eval_calls += stats.eval_calls;
        self.line_shape_eval_calls += stats.line_shape_eval_calls;
        self.line_shape_eval_ns += stats.line_shape_eval_ns;
        self.pattern_eval_calls += stats.pattern_eval_calls;
        self.pattern_eval_ns += stats.pattern_eval_ns;
        self.pattern_frame_queries += stats.pattern_frame_queries;
        self.pattern_frame_query_ns += stats.pattern_frame_query_ns;
        self.pattern_frame_updates += stats.pattern_frame_updates;
        self.pattern_frame_update_ns += stats.pattern_frame_update_ns;
        self.pattern_frame_shadow_checks += stats.pattern_frame_shadow_checks;
        self.pattern_frame_shadow_mismatches += stats.pattern_frame_shadow_mismatches;
        self.candidate_generations += stats.candidate_generations;
        self.candidate_moves_total += stats.candidate_moves_total;
        self.candidate_moves_max = self.candidate_moves_max.max(stats.candidate_moves_max);
        self.root_candidate_generations += stats.root_candidate_generations;
        self.root_candidate_moves_total += stats.root_candidate_moves_total;
        self.root_candidate_moves_max = self
            .root_candidate_moves_max
            .max(stats.root_candidate_moves_max);
        self.search_candidate_generations += stats.search_candidate_generations;
        self.search_candidate_moves_total += stats.search_candidate_moves_total;
        self.search_candidate_moves_max = self
            .search_candidate_moves_max
            .max(stats.search_candidate_moves_max);
        self.legality_checks += stats.legality_checks;
        self.illegal_moves_skipped += stats.illegal_moves_skipped;
        self.root_legality_checks += stats.root_legality_checks;
        self.root_illegal_moves_skipped += stats.root_illegal_moves_skipped;
        self.search_legality_checks += stats.search_legality_checks;
        self.search_illegal_moves_skipped += stats.search_illegal_moves_skipped;
        self.renju_forbidden_prefilter_checks += stats.renju_forbidden_prefilter_checks;
        self.renju_forbidden_prefilter_ns += stats.renju_forbidden_prefilter_ns;
        self.renju_forbidden_checks += stats.renju_forbidden_checks;
        self.renju_forbidden_ns += stats.renju_forbidden_ns;
        self.renju_forbidden_search_gate_checks += stats.renju_forbidden_search_gate_checks;
        self.renju_forbidden_search_gate_ns += stats.renju_forbidden_search_gate_ns;
        self.renju_forbidden_pattern_checks += stats.renju_forbidden_pattern_checks;
        self.renju_forbidden_pattern_ns += stats.renju_forbidden_pattern_ns;
        self.renju_forbidden_threat_checks += stats.renju_forbidden_threat_checks;
        self.renju_forbidden_threat_ns += stats.renju_forbidden_threat_ns;
        self.renju_forbidden_other_checks += stats.renju_forbidden_other_checks;
        self.renju_forbidden_other_ns += stats.renju_forbidden_other_ns;
        self.renju_effective_filter_calls += stats.renju_effective_filter_calls;
        self.renju_effective_filter_ns += stats.renju_effective_filter_ns;
        self.renju_effective_filter_continuation_checks +=
            stats.renju_effective_filter_continuation_checks;
        self.renju_effective_filter_continuation_ns += stats.renju_effective_filter_continuation_ns;
        self.stage_move_gen_ns += stats.stage_move_gen_ns;
        self.stage_ordering_ns += stats.stage_ordering_ns;
        self.stage_eval_ns += stats.stage_eval_ns;
        self.stage_threat_ns += stats.stage_threat_ns;
        self.stage_proof_ns += stats.stage_proof_ns;
        self.tactical_annotations += stats.tactical_annotations;
        self.root_tactical_annotations += stats.root_tactical_annotations;
        self.search_tactical_annotations += stats.search_tactical_annotations;
        self.threat_view_shadow_checks += stats.threat_view_shadow_checks;
        self.threat_view_shadow_mismatches += stats.threat_view_shadow_mismatches;
        self.threat_view_scan_queries += stats.threat_view_scan_queries;
        self.threat_view_scan_ns += stats.threat_view_scan_ns;
        self.threat_view_frontier_rebuilds += stats.threat_view_frontier_rebuilds;
        self.threat_view_frontier_rebuild_ns += stats.threat_view_frontier_rebuild_ns;
        self.threat_view_frontier_queries += stats.threat_view_frontier_queries;
        self.threat_view_frontier_query_ns += stats.threat_view_frontier_query_ns;
        self.threat_view_frontier_immediate_win_queries +=
            stats.threat_view_frontier_immediate_win_queries;
        self.threat_view_frontier_immediate_win_query_ns +=
            stats.threat_view_frontier_immediate_win_query_ns;
        self.threat_view_frontier_delta_captures += stats.threat_view_frontier_delta_captures;
        self.threat_view_frontier_delta_capture_ns += stats.threat_view_frontier_delta_capture_ns;
        self.threat_view_frontier_move_fact_updates += stats.threat_view_frontier_move_fact_updates;
        self.threat_view_frontier_move_fact_update_ns +=
            stats.threat_view_frontier_move_fact_update_ns;
        self.threat_view_frontier_annotation_dirty_marks +=
            stats.threat_view_frontier_annotation_dirty_marks;
        self.threat_view_frontier_annotation_dirty_mark_ns +=
            stats.threat_view_frontier_annotation_dirty_mark_ns;
        self.threat_view_frontier_clean_annotation_queries +=
            stats.threat_view_frontier_clean_annotation_queries;
        self.threat_view_frontier_clean_annotation_query_ns +=
            stats.threat_view_frontier_clean_annotation_query_ns;
        self.threat_view_frontier_dirty_annotation_queries +=
            stats.threat_view_frontier_dirty_annotation_queries;
        self.threat_view_frontier_dirty_annotation_query_ns +=
            stats.threat_view_frontier_dirty_annotation_query_ns;
        self.threat_view_frontier_fallback_annotation_queries +=
            stats.threat_view_frontier_fallback_annotation_queries;
        self.threat_view_frontier_fallback_annotation_query_ns +=
            stats.threat_view_frontier_fallback_annotation_query_ns;
        self.threat_view_frontier_memo_annotation_queries +=
            stats.threat_view_frontier_memo_annotation_queries;
        self.threat_view_frontier_memo_annotation_query_ns +=
            stats.threat_view_frontier_memo_annotation_query_ns;
        self.child_limit_applications += stats.child_limit_applications;
        self.root_child_limit_applications += stats.root_child_limit_applications;
        self.search_child_limit_applications += stats.search_child_limit_applications;
        self.child_cap_hits += stats.child_cap_hits;
        self.root_child_cap_hits += stats.root_child_cap_hits;
        self.search_child_cap_hits += stats.search_child_cap_hits;
        self.child_moves_before_total += stats.child_moves_before_total;
        self.root_child_moves_before_total += stats.root_child_moves_before_total;
        self.search_child_moves_before_total += stats.search_child_moves_before_total;
        self.child_moves_before_max = self
            .child_moves_before_max
            .max(stats.child_moves_before_max);
        self.root_child_moves_before_max = self
            .root_child_moves_before_max
            .max(stats.root_child_moves_before_max);
        self.search_child_moves_before_max = self
            .search_child_moves_before_max
            .max(stats.search_child_moves_before_max);
        self.child_moves_after_total += stats.child_moves_after_total;
        self.root_child_moves_after_total += stats.root_child_moves_after_total;
        self.search_child_moves_after_total += stats.search_child_moves_after_total;
        self.child_moves_after_max = self.child_moves_after_max.max(stats.child_moves_after_max);
        self.root_child_moves_after_max = self
            .root_child_moves_after_max
            .max(stats.root_child_moves_after_max);
        self.search_child_moves_after_max = self
            .search_child_moves_after_max
            .max(stats.search_child_moves_after_max);
        self.tt_hits += stats.tt_hits;
        self.tt_cutoffs += stats.tt_cutoffs;
        self.beta_cutoffs += stats.beta_cutoffs;
        self.depth_sum += stats.depth_sum;
        self.max_depth = self.max_depth.max(stats.max_depth);
        self.effective_depth_sum += stats.effective_depth_sum;
        self.max_effective_depth = self.max_effective_depth.max(stats.max_effective_depth);
        for count in &stats.depth_reached_counts {
            *self.depth_reached_counts.entry(count.depth).or_insert(0) += count.count;
        }
        self.budget_exhausted_count += stats.budget_exhausted_count;
        self.pooled_budget_moves += stats.pooled_budget_moves;
        self.pooled_budget_over_base_count += stats.pooled_budget_over_base_count;
        self.pooled_budget_reserve_exhausted_count += stats.pooled_budget_reserve_exhausted_count;
        self.pooled_budget_reserve_before_total_ms += (stats.pooled_budget_avg_reserve_before_ms
            * stats.pooled_budget_moves as f64)
            .round() as u64;
        self.pooled_budget_reserve_after_total_ms += (stats.pooled_budget_avg_reserve_after_ms
            * stats.pooled_budget_moves as f64)
            .round() as u64;
        if stats.pooled_budget_moves > 0 {
            self.pooled_budget_min_reserve_after_ms = Some(
                self.pooled_budget_min_reserve_after_ms
                    .map_or(stats.pooled_budget_min_reserve_after_ms, |current| {
                        current.min(stats.pooled_budget_min_reserve_after_ms)
                    }),
            );
        }
        self.pooled_budget_max_move_budget_ms = self
            .pooled_budget_max_move_budget_ms
            .max(stats.pooled_budget_max_move_budget_ms);
    }

    pub(super) fn finish(self) -> SideStatsReport {
        let avg_search_time_ms = avg(self.total_time_ms as f64, self.search_move_count);
        let avg_nodes = avg(self.total_nodes as f64, self.search_move_count);
        let avg_eval_calls = avg(self.eval_calls as f64, self.search_move_count);
        let avg_line_shape_eval_ns = avg(
            self.line_shape_eval_ns as f64,
            self.line_shape_eval_calls as u32,
        );
        let avg_pattern_eval_ns = avg(self.pattern_eval_ns as f64, self.pattern_eval_calls as u32);
        let avg_pattern_frame_query_ns = avg(
            self.pattern_frame_query_ns as f64,
            self.pattern_frame_queries as u32,
        );
        let avg_pattern_frame_update_ns = avg(
            self.pattern_frame_update_ns as f64,
            self.pattern_frame_updates as u32,
        );
        let avg_candidate_generations =
            avg(self.candidate_generations as f64, self.search_move_count);
        let avg_candidate_moves = avg(
            self.candidate_moves_total as f64,
            self.candidate_generations as u32,
        );
        let avg_child_moves_before = avg(
            self.child_moves_before_total as f64,
            self.child_limit_applications as u32,
        );
        let avg_child_moves_after = avg(
            self.child_moves_after_total as f64,
            self.child_limit_applications as u32,
        );
        let avg_legality_checks = avg(self.legality_checks as f64, self.search_move_count);
        let avg_renju_forbidden_prefilter_checks = avg(
            self.renju_forbidden_prefilter_checks as f64,
            self.search_move_count,
        );
        let avg_renju_forbidden_prefilter_ns = avg(
            self.renju_forbidden_prefilter_ns as f64,
            self.renju_forbidden_prefilter_checks as u32,
        );
        let avg_renju_forbidden_checks =
            avg(self.renju_forbidden_checks as f64, self.search_move_count);
        let avg_renju_forbidden_ns = avg(
            self.renju_forbidden_ns as f64,
            self.renju_forbidden_checks as u32,
        );
        let avg_renju_effective_filter_calls = avg(
            self.renju_effective_filter_calls as f64,
            self.search_move_count,
        );
        let avg_renju_effective_filter_ns = avg(
            self.renju_effective_filter_ns as f64,
            self.renju_effective_filter_calls as u32,
        );
        let avg_renju_effective_filter_continuation_checks = avg(
            self.renju_effective_filter_continuation_checks as f64,
            self.search_move_count,
        );
        let avg_renju_effective_filter_continuation_ns = avg(
            self.renju_effective_filter_continuation_ns as f64,
            self.renju_effective_filter_continuation_checks as u32,
        );
        let avg_depth = avg(self.depth_sum as f64, self.search_move_count);
        let avg_effective_depth = avg(self.effective_depth_sum as f64, self.search_move_count);
        let budget_exhausted_rate = avg(self.budget_exhausted_count as f64, self.search_move_count);
        let pooled_budget_over_base_rate = avg(
            self.pooled_budget_over_base_count as f64,
            self.pooled_budget_moves,
        );
        let pooled_budget_reserve_exhausted_rate = avg(
            self.pooled_budget_reserve_exhausted_count as f64,
            self.pooled_budget_moves,
        );
        let pooled_budget_avg_reserve_before_ms = avg(
            self.pooled_budget_reserve_before_total_ms as f64,
            self.pooled_budget_moves,
        );
        let pooled_budget_avg_reserve_after_ms = avg(
            self.pooled_budget_reserve_after_total_ms as f64,
            self.pooled_budget_moves,
        );
        let depth_reached_counts = self
            .depth_reached_counts
            .into_iter()
            .map(|(depth, count)| DepthCountReport { depth, count })
            .collect();

        SideStatsReport {
            move_count: self.move_count,
            search_move_count: self.search_move_count,
            total_time_ms: self.total_time_ms,
            avg_search_time_ms,
            search_nodes: self.search_nodes,
            safety_nodes: self.safety_nodes,
            corridor_nodes: self.corridor_nodes,
            corridor_branch_probes: self.corridor_branch_probes,
            corridor_max_depth: self.corridor_max_depth,
            corridor_width_exits: self.corridor_width_exits,
            corridor_depth_exits: self.corridor_depth_exits,
            corridor_neutral_exits: self.corridor_neutral_exits,
            corridor_terminal_exits: self.corridor_terminal_exits,
            corridor_plies_followed: self.corridor_plies_followed,
            corridor_own_plies_followed: self.corridor_own_plies_followed,
            corridor_opponent_plies_followed: self.corridor_opponent_plies_followed,
            corridor_proof_passes: self.corridor_proof_passes,
            corridor_proof_completed: self.corridor_proof_completed,
            corridor_proof_checks: self.corridor_proof_checks,
            corridor_proof_active: self.corridor_proof_active,
            corridor_proof_quiet: self.corridor_proof_quiet,
            corridor_proof_static_exits: self.corridor_proof_static_exits,
            corridor_proof_depth_exits: self.corridor_proof_depth_exits,
            corridor_proof_deadline_exits: self.corridor_proof_deadline_exits,
            corridor_proof_terminal_exits: self.corridor_proof_terminal_exits,
            corridor_proof_terminal_root_candidates: self.corridor_proof_terminal_root_candidates,
            corridor_proof_terminal_root_winning_candidates: self
                .corridor_proof_terminal_root_winning_candidates,
            corridor_proof_terminal_root_losing_candidates: self
                .corridor_proof_terminal_root_losing_candidates,
            corridor_proof_terminal_root_overrides: self.corridor_proof_terminal_root_overrides,
            corridor_proof_terminal_root_move_changes: self
                .corridor_proof_terminal_root_move_changes,
            corridor_proof_terminal_root_move_confirmations: self
                .corridor_proof_terminal_root_move_confirmations,
            corridor_proof_candidates_considered: self.corridor_proof_candidates_considered,
            corridor_proof_wins: self.corridor_proof_wins,
            corridor_proof_losses: self.corridor_proof_losses,
            corridor_proof_unknown: self.corridor_proof_unknown,
            corridor_proof_deadline_skips: self.corridor_proof_deadline_skips,
            corridor_proof_move_changes: self.corridor_proof_move_changes,
            corridor_proof_move_confirmations: self.corridor_proof_move_confirmations,
            corridor_proof_candidate_rank_total: self.corridor_proof_candidate_rank_total,
            corridor_proof_candidate_rank_max: self.corridor_proof_candidate_rank_max,
            corridor_proof_candidate_score_gap_total: self.corridor_proof_candidate_score_gap_total,
            corridor_proof_candidate_score_gap_max: self.corridor_proof_candidate_score_gap_max,
            corridor_proof_win_candidate_rank_total: self.corridor_proof_win_candidate_rank_total,
            corridor_proof_win_candidate_rank_max: self.corridor_proof_win_candidate_rank_max,
            total_nodes: self.total_nodes,
            avg_nodes,
            eval_calls: self.eval_calls,
            avg_eval_calls,
            line_shape_eval_calls: self.line_shape_eval_calls,
            line_shape_eval_ns: self.line_shape_eval_ns,
            avg_line_shape_eval_ns,
            pattern_eval_calls: self.pattern_eval_calls,
            pattern_eval_ns: self.pattern_eval_ns,
            avg_pattern_eval_ns,
            pattern_frame_queries: self.pattern_frame_queries,
            pattern_frame_query_ns: self.pattern_frame_query_ns,
            avg_pattern_frame_query_ns,
            pattern_frame_updates: self.pattern_frame_updates,
            pattern_frame_update_ns: self.pattern_frame_update_ns,
            avg_pattern_frame_update_ns,
            pattern_frame_shadow_checks: self.pattern_frame_shadow_checks,
            pattern_frame_shadow_mismatches: self.pattern_frame_shadow_mismatches,
            candidate_generations: self.candidate_generations,
            avg_candidate_generations,
            candidate_moves_total: self.candidate_moves_total,
            avg_candidate_moves,
            candidate_moves_max: self.candidate_moves_max,
            root_candidate_generations: self.root_candidate_generations,
            root_candidate_moves_total: self.root_candidate_moves_total,
            root_candidate_moves_max: self.root_candidate_moves_max,
            search_candidate_generations: self.search_candidate_generations,
            search_candidate_moves_total: self.search_candidate_moves_total,
            search_candidate_moves_max: self.search_candidate_moves_max,
            legality_checks: self.legality_checks,
            avg_legality_checks,
            illegal_moves_skipped: self.illegal_moves_skipped,
            root_legality_checks: self.root_legality_checks,
            root_illegal_moves_skipped: self.root_illegal_moves_skipped,
            search_legality_checks: self.search_legality_checks,
            search_illegal_moves_skipped: self.search_illegal_moves_skipped,
            renju_forbidden_prefilter_checks: self.renju_forbidden_prefilter_checks,
            avg_renju_forbidden_prefilter_checks,
            renju_forbidden_prefilter_ns: self.renju_forbidden_prefilter_ns,
            avg_renju_forbidden_prefilter_ns,
            renju_forbidden_checks: self.renju_forbidden_checks,
            avg_renju_forbidden_checks,
            renju_forbidden_ns: self.renju_forbidden_ns,
            avg_renju_forbidden_ns,
            renju_forbidden_search_gate_checks: self.renju_forbidden_search_gate_checks,
            renju_forbidden_search_gate_ns: self.renju_forbidden_search_gate_ns,
            renju_forbidden_pattern_checks: self.renju_forbidden_pattern_checks,
            renju_forbidden_pattern_ns: self.renju_forbidden_pattern_ns,
            renju_forbidden_threat_checks: self.renju_forbidden_threat_checks,
            renju_forbidden_threat_ns: self.renju_forbidden_threat_ns,
            renju_forbidden_other_checks: self.renju_forbidden_other_checks,
            renju_forbidden_other_ns: self.renju_forbidden_other_ns,
            renju_effective_filter_calls: self.renju_effective_filter_calls,
            avg_renju_effective_filter_calls,
            renju_effective_filter_ns: self.renju_effective_filter_ns,
            avg_renju_effective_filter_ns,
            renju_effective_filter_continuation_checks: self
                .renju_effective_filter_continuation_checks,
            avg_renju_effective_filter_continuation_checks,
            renju_effective_filter_continuation_ns: self.renju_effective_filter_continuation_ns,
            avg_renju_effective_filter_continuation_ns,
            stage_move_gen_ns: self.stage_move_gen_ns,
            stage_ordering_ns: self.stage_ordering_ns,
            stage_eval_ns: self.stage_eval_ns,
            stage_threat_ns: self.stage_threat_ns,
            stage_proof_ns: self.stage_proof_ns,
            tactical_annotations: self.tactical_annotations,
            root_tactical_annotations: self.root_tactical_annotations,
            search_tactical_annotations: self.search_tactical_annotations,
            threat_view_shadow_checks: self.threat_view_shadow_checks,
            threat_view_shadow_mismatches: self.threat_view_shadow_mismatches,
            threat_view_scan_queries: self.threat_view_scan_queries,
            threat_view_scan_ns: self.threat_view_scan_ns,
            threat_view_frontier_rebuilds: self.threat_view_frontier_rebuilds,
            threat_view_frontier_rebuild_ns: self.threat_view_frontier_rebuild_ns,
            threat_view_frontier_queries: self.threat_view_frontier_queries,
            threat_view_frontier_query_ns: self.threat_view_frontier_query_ns,
            threat_view_frontier_immediate_win_queries: self
                .threat_view_frontier_immediate_win_queries,
            threat_view_frontier_immediate_win_query_ns: self
                .threat_view_frontier_immediate_win_query_ns,
            threat_view_frontier_delta_captures: self.threat_view_frontier_delta_captures,
            threat_view_frontier_delta_capture_ns: self.threat_view_frontier_delta_capture_ns,
            threat_view_frontier_move_fact_updates: self.threat_view_frontier_move_fact_updates,
            threat_view_frontier_move_fact_update_ns: self.threat_view_frontier_move_fact_update_ns,
            threat_view_frontier_annotation_dirty_marks: self
                .threat_view_frontier_annotation_dirty_marks,
            threat_view_frontier_annotation_dirty_mark_ns: self
                .threat_view_frontier_annotation_dirty_mark_ns,
            threat_view_frontier_clean_annotation_queries: self
                .threat_view_frontier_clean_annotation_queries,
            threat_view_frontier_clean_annotation_query_ns: self
                .threat_view_frontier_clean_annotation_query_ns,
            threat_view_frontier_dirty_annotation_queries: self
                .threat_view_frontier_dirty_annotation_queries,
            threat_view_frontier_dirty_annotation_query_ns: self
                .threat_view_frontier_dirty_annotation_query_ns,
            threat_view_frontier_fallback_annotation_queries: self
                .threat_view_frontier_fallback_annotation_queries,
            threat_view_frontier_fallback_annotation_query_ns: self
                .threat_view_frontier_fallback_annotation_query_ns,
            threat_view_frontier_memo_annotation_queries: self
                .threat_view_frontier_memo_annotation_queries,
            threat_view_frontier_memo_annotation_query_ns: self
                .threat_view_frontier_memo_annotation_query_ns,
            child_limit_applications: self.child_limit_applications,
            root_child_limit_applications: self.root_child_limit_applications,
            search_child_limit_applications: self.search_child_limit_applications,
            child_cap_hits: self.child_cap_hits,
            root_child_cap_hits: self.root_child_cap_hits,
            search_child_cap_hits: self.search_child_cap_hits,
            child_moves_before_total: self.child_moves_before_total,
            root_child_moves_before_total: self.root_child_moves_before_total,
            search_child_moves_before_total: self.search_child_moves_before_total,
            child_moves_before_max: self.child_moves_before_max,
            root_child_moves_before_max: self.root_child_moves_before_max,
            search_child_moves_before_max: self.search_child_moves_before_max,
            child_moves_after_total: self.child_moves_after_total,
            root_child_moves_after_total: self.root_child_moves_after_total,
            search_child_moves_after_total: self.search_child_moves_after_total,
            child_moves_after_max: self.child_moves_after_max,
            root_child_moves_after_max: self.root_child_moves_after_max,
            search_child_moves_after_max: self.search_child_moves_after_max,
            avg_child_moves_before,
            avg_child_moves_after,
            tt_hits: self.tt_hits,
            tt_cutoffs: self.tt_cutoffs,
            beta_cutoffs: self.beta_cutoffs,
            depth_sum: self.depth_sum,
            avg_depth,
            max_depth: self.max_depth,
            effective_depth_sum: self.effective_depth_sum,
            avg_effective_depth,
            max_effective_depth: self.max_effective_depth,
            depth_reached_counts,
            budget_exhausted_count: self.budget_exhausted_count,
            budget_exhausted_rate,
            pooled_budget_moves: self.pooled_budget_moves,
            pooled_budget_over_base_count: self.pooled_budget_over_base_count,
            pooled_budget_over_base_rate,
            pooled_budget_reserve_exhausted_count: self.pooled_budget_reserve_exhausted_count,
            pooled_budget_reserve_exhausted_rate,
            pooled_budget_avg_reserve_before_ms,
            pooled_budget_avg_reserve_after_ms,
            pooled_budget_min_reserve_after_ms: self
                .pooled_budget_min_reserve_after_ms
                .unwrap_or(0),
            pooled_budget_max_move_budget_ms: self.pooled_budget_max_move_budget_ms,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub(super) struct EloAggregate {
    sum: f64,
    sum_sq: f64,
}

impl EloAggregate {
    pub(super) fn add(&mut self, rating: f64) {
        self.sum += rating;
        self.sum_sq += rating * rating;
    }

    pub(super) fn finish(&self, samples: usize) -> (f64, f64) {
        if samples == 0 {
            return (DEFAULT_INITIAL_RATING, 0.0);
        }
        let mean = self.sum / samples as f64;
        let variance = (self.sum_sq / samples as f64) - mean * mean;
        (mean, variance.max(0.0).sqrt())
    }
}

pub(super) fn standings(
    bots: &[String],
    results: &TournamentResults,
    matches: &[MatchReport],
    shuffled_elo: &HashMap<String, (f64, f64)>,
) -> Vec<StandingReport> {
    let mut stats: HashMap<String, SideStatsAccumulator> = bots
        .iter()
        .map(|bot| (bot.clone(), SideStatsAccumulator::default()))
        .collect();

    for report_match in matches {
        stats
            .entry(report_match.black.clone())
            .or_default()
            .add_report(&report_match.black_stats);
        stats
            .entry(report_match.white.clone())
            .or_default()
            .add_report(&report_match.white_stats);
    }

    let mut standings = bots
        .iter()
        .map(|bot| {
            let side_stats = stats.remove(bot).unwrap_or_default().finish();
            let wins = *results.wins.get(bot).unwrap_or(&0);
            let draws = *results.draws.get(bot).unwrap_or(&0);
            let losses = *results.losses.get(bot).unwrap_or(&0);
            let (shuffled_elo_avg, shuffled_elo_stddev) = shuffled_elo
                .get(bot)
                .copied()
                .unwrap_or((DEFAULT_INITIAL_RATING, 0.0));

            StandingReport {
                bot: bot.clone(),
                wins,
                draws,
                losses,
                sequential_elo: results.elo_tracker.get_rating(bot),
                shuffled_elo_avg,
                shuffled_elo_stddev,
                match_count: wins + draws + losses,
                move_count: side_stats.move_count,
                search_move_count: side_stats.search_move_count,
                total_time_ms: side_stats.total_time_ms,
                avg_search_time_ms: side_stats.avg_search_time_ms,
                search_nodes: side_stats.search_nodes,
                safety_nodes: side_stats.safety_nodes,
                corridor_nodes: side_stats.corridor_nodes,
                corridor_branch_probes: side_stats.corridor_branch_probes,
                corridor_max_depth: side_stats.corridor_max_depth,
                corridor_width_exits: side_stats.corridor_width_exits,
                corridor_depth_exits: side_stats.corridor_depth_exits,
                corridor_neutral_exits: side_stats.corridor_neutral_exits,
                corridor_terminal_exits: side_stats.corridor_terminal_exits,
                corridor_plies_followed: side_stats.corridor_plies_followed,
                corridor_own_plies_followed: side_stats.corridor_own_plies_followed,
                corridor_opponent_plies_followed: side_stats.corridor_opponent_plies_followed,
                corridor_proof_passes: side_stats.corridor_proof_passes,
                corridor_proof_completed: side_stats.corridor_proof_completed,
                corridor_proof_checks: side_stats.corridor_proof_checks,
                corridor_proof_active: side_stats.corridor_proof_active,
                corridor_proof_quiet: side_stats.corridor_proof_quiet,
                corridor_proof_static_exits: side_stats.corridor_proof_static_exits,
                corridor_proof_depth_exits: side_stats.corridor_proof_depth_exits,
                corridor_proof_deadline_exits: side_stats.corridor_proof_deadline_exits,
                corridor_proof_terminal_exits: side_stats.corridor_proof_terminal_exits,
                corridor_proof_terminal_root_candidates: side_stats
                    .corridor_proof_terminal_root_candidates,
                corridor_proof_terminal_root_winning_candidates: side_stats
                    .corridor_proof_terminal_root_winning_candidates,
                corridor_proof_terminal_root_losing_candidates: side_stats
                    .corridor_proof_terminal_root_losing_candidates,
                corridor_proof_terminal_root_overrides: side_stats
                    .corridor_proof_terminal_root_overrides,
                corridor_proof_terminal_root_move_changes: side_stats
                    .corridor_proof_terminal_root_move_changes,
                corridor_proof_terminal_root_move_confirmations: side_stats
                    .corridor_proof_terminal_root_move_confirmations,
                corridor_proof_candidates_considered: side_stats
                    .corridor_proof_candidates_considered,
                corridor_proof_wins: side_stats.corridor_proof_wins,
                corridor_proof_losses: side_stats.corridor_proof_losses,
                corridor_proof_unknown: side_stats.corridor_proof_unknown,
                corridor_proof_deadline_skips: side_stats.corridor_proof_deadline_skips,
                corridor_proof_move_changes: side_stats.corridor_proof_move_changes,
                corridor_proof_move_confirmations: side_stats.corridor_proof_move_confirmations,
                corridor_proof_candidate_rank_total: side_stats.corridor_proof_candidate_rank_total,
                corridor_proof_candidate_rank_max: side_stats.corridor_proof_candidate_rank_max,
                corridor_proof_candidate_score_gap_total: side_stats
                    .corridor_proof_candidate_score_gap_total,
                corridor_proof_candidate_score_gap_max: side_stats
                    .corridor_proof_candidate_score_gap_max,
                corridor_proof_win_candidate_rank_total: side_stats
                    .corridor_proof_win_candidate_rank_total,
                corridor_proof_win_candidate_rank_max: side_stats
                    .corridor_proof_win_candidate_rank_max,
                total_nodes: side_stats.total_nodes,
                avg_nodes: side_stats.avg_nodes,
                eval_calls: side_stats.eval_calls,
                avg_eval_calls: side_stats.avg_eval_calls,
                line_shape_eval_calls: side_stats.line_shape_eval_calls,
                line_shape_eval_ns: side_stats.line_shape_eval_ns,
                avg_line_shape_eval_ns: side_stats.avg_line_shape_eval_ns,
                pattern_eval_calls: side_stats.pattern_eval_calls,
                pattern_eval_ns: side_stats.pattern_eval_ns,
                avg_pattern_eval_ns: side_stats.avg_pattern_eval_ns,
                pattern_frame_queries: side_stats.pattern_frame_queries,
                pattern_frame_query_ns: side_stats.pattern_frame_query_ns,
                avg_pattern_frame_query_ns: side_stats.avg_pattern_frame_query_ns,
                pattern_frame_updates: side_stats.pattern_frame_updates,
                pattern_frame_update_ns: side_stats.pattern_frame_update_ns,
                avg_pattern_frame_update_ns: side_stats.avg_pattern_frame_update_ns,
                pattern_frame_shadow_checks: side_stats.pattern_frame_shadow_checks,
                pattern_frame_shadow_mismatches: side_stats.pattern_frame_shadow_mismatches,
                candidate_generations: side_stats.candidate_generations,
                avg_candidate_generations: side_stats.avg_candidate_generations,
                candidate_moves_total: side_stats.candidate_moves_total,
                avg_candidate_moves: side_stats.avg_candidate_moves,
                candidate_moves_max: side_stats.candidate_moves_max,
                root_candidate_generations: side_stats.root_candidate_generations,
                root_candidate_moves_total: side_stats.root_candidate_moves_total,
                root_candidate_moves_max: side_stats.root_candidate_moves_max,
                search_candidate_generations: side_stats.search_candidate_generations,
                search_candidate_moves_total: side_stats.search_candidate_moves_total,
                search_candidate_moves_max: side_stats.search_candidate_moves_max,
                legality_checks: side_stats.legality_checks,
                avg_legality_checks: side_stats.avg_legality_checks,
                illegal_moves_skipped: side_stats.illegal_moves_skipped,
                root_legality_checks: side_stats.root_legality_checks,
                root_illegal_moves_skipped: side_stats.root_illegal_moves_skipped,
                search_legality_checks: side_stats.search_legality_checks,
                search_illegal_moves_skipped: side_stats.search_illegal_moves_skipped,
                renju_forbidden_prefilter_checks: side_stats.renju_forbidden_prefilter_checks,
                avg_renju_forbidden_prefilter_checks: side_stats
                    .avg_renju_forbidden_prefilter_checks,
                renju_forbidden_prefilter_ns: side_stats.renju_forbidden_prefilter_ns,
                avg_renju_forbidden_prefilter_ns: side_stats.avg_renju_forbidden_prefilter_ns,
                renju_forbidden_checks: side_stats.renju_forbidden_checks,
                avg_renju_forbidden_checks: side_stats.avg_renju_forbidden_checks,
                renju_forbidden_ns: side_stats.renju_forbidden_ns,
                avg_renju_forbidden_ns: side_stats.avg_renju_forbidden_ns,
                renju_forbidden_search_gate_checks: side_stats.renju_forbidden_search_gate_checks,
                renju_forbidden_search_gate_ns: side_stats.renju_forbidden_search_gate_ns,
                renju_forbidden_pattern_checks: side_stats.renju_forbidden_pattern_checks,
                renju_forbidden_pattern_ns: side_stats.renju_forbidden_pattern_ns,
                renju_forbidden_threat_checks: side_stats.renju_forbidden_threat_checks,
                renju_forbidden_threat_ns: side_stats.renju_forbidden_threat_ns,
                renju_forbidden_other_checks: side_stats.renju_forbidden_other_checks,
                renju_forbidden_other_ns: side_stats.renju_forbidden_other_ns,
                renju_effective_filter_calls: side_stats.renju_effective_filter_calls,
                avg_renju_effective_filter_calls: side_stats.avg_renju_effective_filter_calls,
                renju_effective_filter_ns: side_stats.renju_effective_filter_ns,
                avg_renju_effective_filter_ns: side_stats.avg_renju_effective_filter_ns,
                renju_effective_filter_continuation_checks: side_stats
                    .renju_effective_filter_continuation_checks,
                avg_renju_effective_filter_continuation_checks: side_stats
                    .avg_renju_effective_filter_continuation_checks,
                renju_effective_filter_continuation_ns: side_stats
                    .renju_effective_filter_continuation_ns,
                avg_renju_effective_filter_continuation_ns: side_stats
                    .avg_renju_effective_filter_continuation_ns,
                stage_move_gen_ns: side_stats.stage_move_gen_ns,
                stage_ordering_ns: side_stats.stage_ordering_ns,
                stage_eval_ns: side_stats.stage_eval_ns,
                stage_threat_ns: side_stats.stage_threat_ns,
                stage_proof_ns: side_stats.stage_proof_ns,
                tactical_annotations: side_stats.tactical_annotations,
                root_tactical_annotations: side_stats.root_tactical_annotations,
                search_tactical_annotations: side_stats.search_tactical_annotations,
                threat_view_shadow_checks: side_stats.threat_view_shadow_checks,
                threat_view_shadow_mismatches: side_stats.threat_view_shadow_mismatches,
                threat_view_scan_queries: side_stats.threat_view_scan_queries,
                threat_view_scan_ns: side_stats.threat_view_scan_ns,
                threat_view_frontier_rebuilds: side_stats.threat_view_frontier_rebuilds,
                threat_view_frontier_rebuild_ns: side_stats.threat_view_frontier_rebuild_ns,
                threat_view_frontier_queries: side_stats.threat_view_frontier_queries,
                threat_view_frontier_query_ns: side_stats.threat_view_frontier_query_ns,
                threat_view_frontier_immediate_win_queries: side_stats
                    .threat_view_frontier_immediate_win_queries,
                threat_view_frontier_immediate_win_query_ns: side_stats
                    .threat_view_frontier_immediate_win_query_ns,
                threat_view_frontier_delta_captures: side_stats.threat_view_frontier_delta_captures,
                threat_view_frontier_delta_capture_ns: side_stats
                    .threat_view_frontier_delta_capture_ns,
                threat_view_frontier_move_fact_updates: side_stats
                    .threat_view_frontier_move_fact_updates,
                threat_view_frontier_move_fact_update_ns: side_stats
                    .threat_view_frontier_move_fact_update_ns,
                threat_view_frontier_annotation_dirty_marks: side_stats
                    .threat_view_frontier_annotation_dirty_marks,
                threat_view_frontier_annotation_dirty_mark_ns: side_stats
                    .threat_view_frontier_annotation_dirty_mark_ns,
                threat_view_frontier_clean_annotation_queries: side_stats
                    .threat_view_frontier_clean_annotation_queries,
                threat_view_frontier_clean_annotation_query_ns: side_stats
                    .threat_view_frontier_clean_annotation_query_ns,
                threat_view_frontier_dirty_annotation_queries: side_stats
                    .threat_view_frontier_dirty_annotation_queries,
                threat_view_frontier_dirty_annotation_query_ns: side_stats
                    .threat_view_frontier_dirty_annotation_query_ns,
                threat_view_frontier_fallback_annotation_queries: side_stats
                    .threat_view_frontier_fallback_annotation_queries,
                threat_view_frontier_fallback_annotation_query_ns: side_stats
                    .threat_view_frontier_fallback_annotation_query_ns,
                threat_view_frontier_memo_annotation_queries: side_stats
                    .threat_view_frontier_memo_annotation_queries,
                threat_view_frontier_memo_annotation_query_ns: side_stats
                    .threat_view_frontier_memo_annotation_query_ns,
                child_limit_applications: side_stats.child_limit_applications,
                root_child_limit_applications: side_stats.root_child_limit_applications,
                search_child_limit_applications: side_stats.search_child_limit_applications,
                child_cap_hits: side_stats.child_cap_hits,
                root_child_cap_hits: side_stats.root_child_cap_hits,
                search_child_cap_hits: side_stats.search_child_cap_hits,
                child_moves_before_total: side_stats.child_moves_before_total,
                root_child_moves_before_total: side_stats.root_child_moves_before_total,
                search_child_moves_before_total: side_stats.search_child_moves_before_total,
                child_moves_before_max: side_stats.child_moves_before_max,
                root_child_moves_before_max: side_stats.root_child_moves_before_max,
                search_child_moves_before_max: side_stats.search_child_moves_before_max,
                child_moves_after_total: side_stats.child_moves_after_total,
                root_child_moves_after_total: side_stats.root_child_moves_after_total,
                search_child_moves_after_total: side_stats.search_child_moves_after_total,
                child_moves_after_max: side_stats.child_moves_after_max,
                root_child_moves_after_max: side_stats.root_child_moves_after_max,
                search_child_moves_after_max: side_stats.search_child_moves_after_max,
                avg_child_moves_before: side_stats.avg_child_moves_before,
                avg_child_moves_after: side_stats.avg_child_moves_after,
                tt_hits: side_stats.tt_hits,
                tt_cutoffs: side_stats.tt_cutoffs,
                beta_cutoffs: side_stats.beta_cutoffs,
                avg_depth: side_stats.avg_depth,
                max_depth: side_stats.max_depth,
                effective_depth_sum: side_stats.effective_depth_sum,
                avg_effective_depth: side_stats.avg_effective_depth,
                max_effective_depth: side_stats.max_effective_depth,
                depth_reached_counts: side_stats.depth_reached_counts,
                budget_exhausted_count: side_stats.budget_exhausted_count,
                budget_exhausted_rate: side_stats.budget_exhausted_rate,
                pooled_budget_moves: side_stats.pooled_budget_moves,
                pooled_budget_over_base_count: side_stats.pooled_budget_over_base_count,
                pooled_budget_over_base_rate: side_stats.pooled_budget_over_base_rate,
                pooled_budget_reserve_exhausted_count: side_stats
                    .pooled_budget_reserve_exhausted_count,
                pooled_budget_reserve_exhausted_rate: side_stats
                    .pooled_budget_reserve_exhausted_rate,
                pooled_budget_avg_reserve_before_ms: side_stats.pooled_budget_avg_reserve_before_ms,
                pooled_budget_avg_reserve_after_ms: side_stats.pooled_budget_avg_reserve_after_ms,
                pooled_budget_min_reserve_after_ms: side_stats.pooled_budget_min_reserve_after_ms,
                pooled_budget_max_move_budget_ms: side_stats.pooled_budget_max_move_budget_ms,
            }
        })
        .collect::<Vec<_>>();

    standings.sort_by(|a, b| {
        b.shuffled_elo_avg
            .partial_cmp(&a.shuffled_elo_avg)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    standings
}

pub(super) fn pairwise(bots: &[String], matches: &[MatchReport]) -> Vec<PairwiseReport> {
    let order = bot_order(bots);
    let mut map: HashMap<(String, String), PairwiseReport> = HashMap::new();

    for report_match in matches {
        let (bot_a, bot_b) = ordered_pair(&report_match.black, &report_match.white, &order);
        let entry = map
            .entry((bot_a.clone(), bot_b.clone()))
            .or_insert(PairwiseReport {
                bot_a,
                bot_b,
                wins_a: 0,
                wins_b: 0,
                draws: 0,
                total: 0,
                score_a: 0.0,
                score_b: 0.0,
            });
        entry.total += 1;

        match report_match.winner.as_deref() {
            Some(winner) if winner == entry.bot_a => {
                entry.wins_a += 1;
                entry.score_a += 1.0;
            }
            Some(winner) if winner == entry.bot_b => {
                entry.wins_b += 1;
                entry.score_b += 1.0;
            }
            None => {
                entry.draws += 1;
                entry.score_a += 0.5;
                entry.score_b += 0.5;
            }
            _ => {}
        }
    }

    let mut values = map.into_values().collect::<Vec<_>>();
    values.sort_by_key(|entry| {
        (
            order.get(&entry.bot_a).copied().unwrap_or(usize::MAX),
            order.get(&entry.bot_b).copied().unwrap_or(usize::MAX),
        )
    });
    values
}

pub(super) fn color_splits(matches: &[MatchReport]) -> Vec<ColorSplitReport> {
    let mut map: HashMap<(String, String), ColorSplitReport> = HashMap::new();

    for report_match in matches {
        let entry = map
            .entry((report_match.black.clone(), report_match.white.clone()))
            .or_insert(ColorSplitReport {
                black: report_match.black.clone(),
                white: report_match.white.clone(),
                black_wins: 0,
                white_wins: 0,
                draws: 0,
                total: 0,
            });
        entry.total += 1;

        match report_match.winner.as_deref() {
            Some(winner) if winner == entry.black => entry.black_wins += 1,
            Some(winner) if winner == entry.white => entry.white_wins += 1,
            None => entry.draws += 1,
            _ => {}
        }
    }

    let mut values = map.into_values().collect::<Vec<_>>();
    values.sort_by(|a, b| a.black.cmp(&b.black).then(a.white.cmp(&b.white)));
    values
}

pub(super) fn end_reasons(results: &TournamentResults) -> Vec<CountReport> {
    let mut values = results
        .end_reasons
        .iter()
        .map(|(reason, count)| CountReport {
            key: end_reason_code(*reason).to_string(),
            count: *count,
        })
        .collect::<Vec<_>>();
    values.sort_by(|a, b| a.key.cmp(&b.key));
    values
}

pub(super) fn shuffled_elo_stats(
    bots: &[String],
    matches: &[MatchReport],
    samples: usize,
) -> HashMap<String, (f64, f64)> {
    let mut aggregate: HashMap<String, EloAggregate> = bots
        .iter()
        .map(|bot| (bot.clone(), EloAggregate::default()))
        .collect();

    for sample in 0..samples {
        let mut indices = (0..matches.len()).collect::<Vec<_>>();
        shuffle_indices(&mut indices, sample as u64);
        let ratings = elo_for_order(bots, matches, &indices);
        for (bot, rating) in ratings {
            aggregate.entry(bot).or_default().add(rating);
        }
    }

    aggregate
        .into_iter()
        .map(|(bot, aggregate)| (bot, aggregate.finish(samples)))
        .collect()
}

pub(super) fn elo_for_order(
    bots: &[String],
    matches: &[MatchReport],
    indices: &[usize],
) -> HashMap<String, f64> {
    let mut ratings: HashMap<String, f64> = bots
        .iter()
        .map(|bot| (bot.clone(), DEFAULT_INITIAL_RATING))
        .collect();

    for &idx in indices {
        let report_match = &matches[idx];
        let black_rating = *ratings
            .get(&report_match.black)
            .unwrap_or(&DEFAULT_INITIAL_RATING);
        let white_rating = *ratings
            .get(&report_match.white)
            .unwrap_or(&DEFAULT_INITIAL_RATING);
        let expected_black = expected_score(black_rating, white_rating);
        let expected_white = expected_score(white_rating, black_rating);
        let (score_black, score_white) = match report_match.winner.as_deref() {
            Some(winner) if winner == report_match.black => (1.0, 0.0),
            Some(winner) if winner == report_match.white => (0.0, 1.0),
            None => (0.5, 0.5),
            _ => (0.5, 0.5),
        };

        ratings.insert(
            report_match.black.clone(),
            black_rating + DEFAULT_K_FACTOR * (score_black - expected_black),
        );
        ratings.insert(
            report_match.white.clone(),
            white_rating + DEFAULT_K_FACTOR * (score_white - expected_white),
        );
    }

    ratings
}

pub(super) fn shuffle_indices(indices: &mut [usize], sample: u64) {
    let mut state = 0x9e37_79b9_7f4a_7c15_u64 ^ sample.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    for i in (1..indices.len()).rev() {
        state = xorshift64(state);
        let j = (state as usize) % (i + 1);
        indices.swap(i, j);
    }
}

pub(super) fn xorshift64(mut value: u64) -> u64 {
    value ^= value << 13;
    value ^= value >> 7;
    value ^= value << 17;
    value
}

pub(super) fn bot_order(bots: &[String]) -> HashMap<String, usize> {
    bots.iter()
        .enumerate()
        .map(|(idx, bot)| (bot.clone(), idx))
        .collect()
}

pub(super) fn ordered_pair(
    first: &str,
    second: &str,
    order: &HashMap<String, usize>,
) -> (String, String) {
    let first_order = order.get(first).copied().unwrap_or(usize::MAX);
    let second_order = order.get(second).copied().unwrap_or(usize::MAX);
    if first_order < second_order || (first_order == second_order && first <= second) {
        (first.to_string(), second.to_string())
    } else {
        (second.to_string(), first.to_string())
    }
}

pub(super) fn encode_move_cell(mv: Move, board_size: usize) -> Result<usize, String> {
    if mv.row >= board_size || mv.col >= board_size {
        return Err(format!(
            "move outside board: {} for board size {}",
            mv.to_notation(),
            board_size
        ));
    }
    Ok(mv.row * board_size + mv.col)
}

pub(super) fn result_code(result: &GameResult) -> &'static str {
    match result {
        GameResult::Winner(Color::Black) => "black_won",
        GameResult::Winner(Color::White) => "white_won",
        GameResult::Draw => "draw",
        GameResult::Ongoing => "ongoing",
    }
}

pub(super) fn winner_name(result: &GameResult, black: &str, white: &str) -> Option<String> {
    match result {
        GameResult::Winner(Color::Black) => Some(black.to_string()),
        GameResult::Winner(Color::White) => Some(white.to_string()),
        GameResult::Draw | GameResult::Ongoing => None,
    }
}

pub(super) fn end_reason_code(reason: MatchEndReason) -> &'static str {
    match reason {
        MatchEndReason::Natural => "natural",
        MatchEndReason::MaxMoves => "max_moves",
        MatchEndReason::MaxGameTime => "max_game_time",
    }
}

pub(super) fn trace_value_u64(trace: &Value, key: &str) -> u64 {
    trace.get(key).and_then(Value::as_u64).unwrap_or(0)
}

pub(super) fn avg(total: f64, count: u32) -> f64 {
    if count == 0 {
        0.0
    } else {
        total / count as f64
    }
}

pub(super) fn default_opening_policy() -> String {
    "centered-suite".to_string()
}

pub(super) fn default_search_budget_mode() -> String {
    "strict".to_string()
}

pub(super) fn default_schedule() -> String {
    "round-robin".to_string()
}

pub(super) fn score_rate(wins: u32, draws: u32, total: u32) -> f64 {
    avg(wins as f64 + draws as f64 * 0.5, total)
}

#[cfg(test)]
pub(super) fn schedule_summary(report: &TournamentReport) -> String {
    if report.run.schedule == "gauntlet" {
        if let Some(reference) = &report.reference_anchors {
            let anchor_count = reference.anchors.len();
            if anchor_count > 0 && report.run.bots.len() > anchor_count {
                let candidate_count = report.run.bots.len() - anchor_count;
                let candidate_word = if candidate_count == 1 {
                    "candidate"
                } else {
                    "candidates"
                };
                let anchor_word = if anchor_count == 1 {
                    "anchor"
                } else {
                    "anchors"
                };
                return format!(
                    "{} {} x {} {} x {} games = {} matches",
                    candidate_count,
                    candidate_word,
                    anchor_count,
                    anchor_word,
                    report.run.games_per_pair,
                    report.matches.len()
                );
            }
        }
    }

    let pair_count = report.pairwise.len();
    let pair_word = if pair_count == 1 { "pair" } else { "pairs" };
    format!(
        "{} {} x {} games = {} matches",
        pair_count,
        pair_word,
        report.run.games_per_pair,
        report.matches.len()
    )
}

#[cfg(test)]
pub(super) fn compact_bot_label(report: &TournamentReport, bot: &str) -> String {
    shared_compact_bot_label(bot, report_uses_budgeted_unqualified_search(&report.run))
}

#[cfg(test)]
pub(super) fn report_uses_budgeted_unqualified_search(run: &TournamentRunReport) -> bool {
    run.search_time_ms.is_some() || run.search_cpu_time_ms.is_some()
}

#[derive(Default)]
pub(super) struct PairSearchStats {
    search_move_count: u32,
    total_time_ms: u64,
    total_nodes: u64,
}

impl PairSearchStats {
    pub(super) fn record(&mut self, stats: &SideStatsReport) {
        self.search_move_count += stats.search_move_count;
        self.total_time_ms += stats.total_time_ms;
        self.total_nodes += stats.total_nodes;
    }
}

impl ReferencePairSearchReport {
    pub(super) fn from_pair_and_stats(
        pair: &PairwiseReport,
        bot_a_stats: &PairSearchStats,
        bot_b_stats: &PairSearchStats,
    ) -> Self {
        Self {
            bot_a: pair.bot_a.clone(),
            bot_b: pair.bot_b.clone(),
            bot_a_search_move_count: bot_a_stats.search_move_count,
            bot_a_total_time_ms: bot_a_stats.total_time_ms,
            bot_a_total_nodes: bot_a_stats.total_nodes,
            bot_b_search_move_count: bot_b_stats.search_move_count,
            bot_b_total_time_ms: bot_b_stats.total_time_ms,
            bot_b_total_nodes: bot_b_stats.total_nodes,
        }
    }
}

pub(super) fn reference_pair_search_reports(
    source_report: &TournamentReport,
    pairwise: &[PairwiseReport],
) -> Vec<ReferencePairSearchReport> {
    pairwise
        .iter()
        .map(|pair| {
            let bot_a_stats =
                pair_search_stats_for_matches(&source_report.matches, pair, &pair.bot_a);
            let bot_b_stats =
                pair_search_stats_for_matches(&source_report.matches, pair, &pair.bot_b);
            ReferencePairSearchReport::from_pair_and_stats(pair, &bot_a_stats, &bot_b_stats)
        })
        .collect()
}

pub(super) fn pair_search_stats_for_matches(
    matches: &[MatchReport],
    pair: &PairwiseReport,
    bot: &str,
) -> PairSearchStats {
    let mut stats = PairSearchStats::default();
    for report_match in matches {
        if !same_pair(report_match, &pair.bot_a, &pair.bot_b) {
            continue;
        }

        if report_match.black == bot {
            stats.record(&report_match.black_stats);
        } else if report_match.white == bot {
            stats.record(&report_match.white_stats);
        }
    }
    stats
}

pub(super) fn same_pair(report_match: &MatchReport, bot_a: &str, bot_b: &str) -> bool {
    (report_match.black == bot_a && report_match.white == bot_b)
        || (report_match.black == bot_b && report_match.white == bot_a)
}
