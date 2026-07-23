use super::*;

#[test]
fn move_cells_match_saved_match_codec() {
    assert_eq!(encode_move_cell(Move { row: 0, col: 0 }, 15).unwrap(), 0);
    assert_eq!(encode_move_cell(Move { row: 7, col: 7 }, 15).unwrap(), 112);
    assert_eq!(
        encode_move_cell(Move { row: 14, col: 14 }, 15).unwrap(),
        224
    );
}

#[test]
fn report_sums_shadow_mismatches_from_standings() {
    let mut report = sample_report();
    let mut first = sample_standing_with_search_costs("search-d3+rolling-frontier-shadow");
    first.threat_view_shadow_mismatches = 2;
    let mut second = sample_standing_with_search_costs("search-d5+rolling-frontier-shadow");
    second.threat_view_shadow_mismatches = 3;
    report.standings = vec![first, second];

    assert_eq!(report.shadow_mismatch_count(), 5);
}

#[test]
fn side_stats_capture_child_caps_tactical_annotations_and_depth_distribution() {
    let mut stats = SideStatsAccumulator::default();
    let mut metrics = serde_json::json!({
        "eval_calls": 30,
        "line_shape_eval_calls": 10,
        "line_shape_eval_ns": 1000,
        "pattern_eval_calls": 20,
        "pattern_eval_ns": 8000,
        "tactical_annotations": 9,
        "root_tactical_annotations": 2,
        "search_tactical_annotations": 7,
        "child_limit_applications": 4,
        "root_child_limit_applications": 0,
        "search_child_limit_applications": 4,
        "child_cap_hits": 3,
        "root_child_cap_hits": 0,
        "search_child_cap_hits": 3,
        "child_moves_before_total": 48,
        "root_child_moves_before_total": 0,
        "search_child_moves_before_total": 48,
        "child_moves_before_max": 14,
        "root_child_moves_before_max": 0,
        "search_child_moves_before_max": 14,
        "child_moves_after_total": 32,
        "root_child_moves_after_total": 0,
        "search_child_moves_after_total": 32,
        "child_moves_after_max": 9,
        "root_child_moves_after_max": 0,
        "search_child_moves_after_max": 9,
        "corridor_width_exits": 5,
        "corridor_depth_exits": 6,
        "corridor_neutral_exits": 7,
        "corridor_terminal_exits": 8,
        "corridor_plies_followed": 9,
        "corridor_own_plies_followed": 6,
        "corridor_opponent_plies_followed": 3
    });
    metrics["stage_move_gen_ns"] = serde_json::json!(100);
    metrics["stage_ordering_ns"] = serde_json::json!(200);
    metrics["stage_eval_ns"] = serde_json::json!(300);
    metrics["stage_threat_ns"] = serde_json::json!(400);
    metrics["stage_proof_ns"] = serde_json::json!(500);
    metrics["corridor_proof_passes"] = serde_json::json!(10);
    metrics["corridor_proof_completed"] = serde_json::json!(11);
    metrics["corridor_proof_checks"] = serde_json::json!(12);
    metrics["corridor_proof_active"] = serde_json::json!(13);
    metrics["corridor_proof_quiet"] = serde_json::json!(14);
    metrics["corridor_proof_static_exits"] = serde_json::json!(15);
    metrics["corridor_proof_depth_exits"] = serde_json::json!(16);
    metrics["corridor_proof_deadline_exits"] = serde_json::json!(17);
    metrics["corridor_proof_terminal_exits"] = serde_json::json!(18);
    metrics["corridor_proof_terminal_root_candidates"] = serde_json::json!(19);
    metrics["corridor_proof_terminal_root_winning_candidates"] = serde_json::json!(20);
    metrics["corridor_proof_terminal_root_losing_candidates"] = serde_json::json!(21);
    metrics["corridor_proof_terminal_root_overrides"] = serde_json::json!(22);
    metrics["corridor_proof_terminal_root_move_changes"] = serde_json::json!(23);
    metrics["corridor_proof_terminal_root_move_confirmations"] = serde_json::json!(24);
    metrics["corridor_proof_candidates_considered"] = serde_json::json!(25);
    metrics["corridor_proof_wins"] = serde_json::json!(26);
    metrics["corridor_proof_losses"] = serde_json::json!(27);
    metrics["corridor_proof_unknown"] = serde_json::json!(28);
    metrics["corridor_proof_deadline_skips"] = serde_json::json!(29);
    metrics["corridor_proof_move_changes"] = serde_json::json!(30);
    metrics["corridor_proof_move_confirmations"] = serde_json::json!(31);
    metrics["corridor_proof_candidate_rank_total"] = serde_json::json!(32);
    metrics["corridor_proof_candidate_rank_max"] = serde_json::json!(6);
    metrics["corridor_proof_candidate_score_gap_total"] = serde_json::json!(123_456);
    metrics["corridor_proof_candidate_score_gap_max"] = serde_json::json!(50_000);
    metrics["corridor_proof_win_candidate_rank_total"] = serde_json::json!(7);
    metrics["corridor_proof_win_candidate_rank_max"] = serde_json::json!(2);
    metrics["pattern_frame_queries"] = serde_json::json!(15);
    metrics["pattern_frame_query_ns"] = serde_json::json!(150);
    metrics["pattern_frame_updates"] = serde_json::json!(8);
    metrics["pattern_frame_update_ns"] = serde_json::json!(800);
    metrics["pattern_frame_shadow_checks"] = serde_json::json!(15);
    metrics["pattern_frame_shadow_mismatches"] = serde_json::json!(0);
    let trace = serde_json::json!({
        "nodes": 100,
        "safety_nodes": 20,
        "total_nodes": 120,
        "depth": 5,
        "effective_depth": 8,
        "corridor": {
            "search_nodes": 7,
            "branch_probes": 3,
            "max_depth_reached": 2,
            "extra_plies": 3
        },
        "budget_pool": {
            "mode": "pooled_cpu",
            "base_ms": 1000,
            "move_budget_ms": 1750,
            "reserve_cap_ms": 4000,
            "max_move_ms": null,
            "reserve_before_ms": 750,
            "reserve_after_ms": 250,
            "consumed_ms": 1500,
            "budget_exhausted": false
        },
        "metrics": metrics
    });

    stats.record_move(11, Some(&trace));
    let report = stats.finish();

    assert_eq!(report.search_nodes, 100);
    assert_eq!(report.safety_nodes, 20);
    assert_eq!(report.corridor_nodes, 7);
    assert_eq!(report.corridor_branch_probes, 3);
    assert_eq!(report.corridor_max_depth, 2);
    assert_eq!(report.corridor_width_exits, 5);
    assert_eq!(report.corridor_depth_exits, 6);
    assert_eq!(report.corridor_neutral_exits, 7);
    assert_eq!(report.corridor_terminal_exits, 8);
    assert_eq!(report.corridor_plies_followed, 9);
    assert_eq!(report.corridor_own_plies_followed, 6);
    assert_eq!(report.corridor_opponent_plies_followed, 3);
    assert_eq!(report.corridor_proof_passes, 10);
    assert_eq!(report.corridor_proof_completed, 11);
    assert_eq!(report.corridor_proof_checks, 12);
    assert_eq!(report.corridor_proof_active, 13);
    assert_eq!(report.corridor_proof_quiet, 14);
    assert_eq!(report.corridor_proof_static_exits, 15);
    assert_eq!(report.corridor_proof_depth_exits, 16);
    assert_eq!(report.corridor_proof_deadline_exits, 17);
    assert_eq!(report.corridor_proof_terminal_exits, 18);
    assert_eq!(report.corridor_proof_terminal_root_candidates, 19);
    assert_eq!(report.corridor_proof_terminal_root_winning_candidates, 20);
    assert_eq!(report.corridor_proof_terminal_root_losing_candidates, 21);
    assert_eq!(report.corridor_proof_terminal_root_overrides, 22);
    assert_eq!(report.corridor_proof_terminal_root_move_changes, 23);
    assert_eq!(report.corridor_proof_terminal_root_move_confirmations, 24);
    assert_eq!(report.corridor_proof_candidates_considered, 25);
    assert_eq!(report.corridor_proof_wins, 26);
    assert_eq!(report.corridor_proof_losses, 27);
    assert_eq!(report.corridor_proof_unknown, 28);
    assert_eq!(report.corridor_proof_deadline_skips, 29);
    assert_eq!(report.corridor_proof_move_changes, 30);
    assert_eq!(report.corridor_proof_move_confirmations, 31);
    assert_eq!(report.corridor_proof_candidate_rank_total, 32);
    assert_eq!(report.corridor_proof_candidate_rank_max, 6);
    assert_eq!(report.corridor_proof_candidate_score_gap_total, 123_456);
    assert_eq!(report.corridor_proof_candidate_score_gap_max, 50_000);
    assert_eq!(report.corridor_proof_win_candidate_rank_total, 7);
    assert_eq!(report.corridor_proof_win_candidate_rank_max, 2);
    assert_eq!(report.eval_calls, 30);
    assert_eq!(report.avg_eval_calls, 30.0);
    assert_eq!(report.line_shape_eval_calls, 10);
    assert_eq!(report.line_shape_eval_ns, 1000);
    assert_eq!(report.avg_line_shape_eval_ns, 100.0);
    assert_eq!(report.pattern_eval_calls, 20);
    assert_eq!(report.pattern_eval_ns, 8000);
    assert_eq!(report.avg_pattern_eval_ns, 400.0);
    assert_eq!(report.pattern_frame_queries, 15);
    assert_eq!(report.pattern_frame_query_ns, 150);
    assert_eq!(report.avg_pattern_frame_query_ns, 10.0);
    assert_eq!(report.pattern_frame_updates, 8);
    assert_eq!(report.pattern_frame_update_ns, 800);
    assert_eq!(report.avg_pattern_frame_update_ns, 100.0);
    assert_eq!(report.pattern_frame_shadow_checks, 15);
    assert_eq!(report.pattern_frame_shadow_mismatches, 0);
    assert_eq!(report.stage_move_gen_ns, 100);
    assert_eq!(report.stage_ordering_ns, 200);
    assert_eq!(report.stage_eval_ns, 300);
    assert_eq!(report.stage_threat_ns, 400);
    assert_eq!(report.stage_proof_ns, 500);
    assert_eq!(report.effective_depth_sum, 8);
    assert_eq!(report.avg_effective_depth, 8.0);
    assert_eq!(report.max_effective_depth, 8);
    assert_eq!(report.tactical_annotations, 9);
    assert_eq!(report.root_tactical_annotations, 2);
    assert_eq!(report.search_tactical_annotations, 7);
    assert_eq!(report.child_limit_applications, 4);
    assert_eq!(report.search_child_limit_applications, 4);
    assert_eq!(report.child_cap_hits, 3);
    assert_eq!(report.search_child_cap_hits, 3);
    assert_eq!(report.child_moves_before_total, 48);
    assert_eq!(report.child_moves_after_total, 32);
    assert_eq!(report.avg_child_moves_before, 12.0);
    assert_eq!(report.avg_child_moves_after, 8.0);
    assert_eq!(
        report.depth_reached_counts,
        vec![DepthCountReport { depth: 5, count: 1 }]
    );
    assert_eq!(report.pooled_budget_moves, 1);
    assert_eq!(report.pooled_budget_over_base_count, 1);
    assert_eq!(report.pooled_budget_over_base_rate, 1.0);
    assert_eq!(report.pooled_budget_reserve_exhausted_count, 0);
    assert_eq!(report.pooled_budget_reserve_exhausted_rate, 0.0);
    assert_eq!(report.pooled_budget_avg_reserve_before_ms, 750.0);
    assert_eq!(report.pooled_budget_avg_reserve_after_ms, 250.0);
    assert_eq!(report.pooled_budget_min_reserve_after_ms, 250);
    assert_eq!(report.pooled_budget_max_move_budget_ms, 1750);
}

#[test]
fn side_stats_capture_threat_view_metrics() {
    let mut stats = SideStatsAccumulator::default();
    let trace = serde_json::json!({
        "metrics": {
            "threat_view_shadow_checks": 11,
            "threat_view_shadow_mismatches": 1,
            "threat_view_scan_queries": 13,
            "threat_view_scan_ns": 1700,
            "threat_view_frontier_rebuilds": 5,
            "threat_view_frontier_rebuild_ns": 2300,
            "threat_view_frontier_queries": 19,
            "threat_view_frontier_query_ns": 2900,
            "threat_view_frontier_immediate_win_queries": 20,
            "threat_view_frontier_immediate_win_query_ns": 3000,
            "threat_view_frontier_delta_captures": 7,
            "threat_view_frontier_delta_capture_ns": 3100,
            "threat_view_frontier_move_fact_updates": 8,
            "threat_view_frontier_move_fact_update_ns": 3200,
            "threat_view_frontier_annotation_dirty_marks": 9,
            "threat_view_frontier_annotation_dirty_mark_ns": 3300,
            "threat_view_frontier_clean_annotation_queries": 14,
            "threat_view_frontier_clean_annotation_query_ns": 3400,
            "threat_view_frontier_dirty_annotation_queries": 15,
            "threat_view_frontier_dirty_annotation_query_ns": 3500,
            "threat_view_frontier_fallback_annotation_queries": 16,
            "threat_view_frontier_fallback_annotation_query_ns": 3600,
            "threat_view_frontier_memo_annotation_queries": 17,
            "threat_view_frontier_memo_annotation_query_ns": 3700
        }
    });

    stats.record_move(11, Some(&trace));
    let report = stats.finish();

    assert_eq!(report.threat_view_shadow_checks, 11);
    assert_eq!(report.threat_view_shadow_mismatches, 1);
    assert_eq!(report.threat_view_scan_queries, 13);
    assert_eq!(report.threat_view_scan_ns, 1700);
    assert_eq!(report.threat_view_frontier_rebuilds, 5);
    assert_eq!(report.threat_view_frontier_rebuild_ns, 2300);
    assert_eq!(report.threat_view_frontier_queries, 19);
    assert_eq!(report.threat_view_frontier_query_ns, 2900);
    assert_eq!(report.threat_view_frontier_immediate_win_queries, 20);
    assert_eq!(report.threat_view_frontier_immediate_win_query_ns, 3000);
    assert_eq!(report.threat_view_frontier_delta_captures, 7);
    assert_eq!(report.threat_view_frontier_delta_capture_ns, 3100);
    assert_eq!(report.threat_view_frontier_move_fact_updates, 8);
    assert_eq!(report.threat_view_frontier_move_fact_update_ns, 3200);
    assert_eq!(report.threat_view_frontier_annotation_dirty_marks, 9);
    assert_eq!(report.threat_view_frontier_annotation_dirty_mark_ns, 3300);
    assert_eq!(report.threat_view_frontier_clean_annotation_queries, 14);
    assert_eq!(report.threat_view_frontier_clean_annotation_query_ns, 3400);
    assert_eq!(report.threat_view_frontier_dirty_annotation_queries, 15);
    assert_eq!(report.threat_view_frontier_dirty_annotation_query_ns, 3500);
    assert_eq!(report.threat_view_frontier_fallback_annotation_queries, 16);
    assert_eq!(
        report.threat_view_frontier_fallback_annotation_query_ns,
        3600
    );
    assert_eq!(report.threat_view_frontier_memo_annotation_queries, 17);
    assert_eq!(report.threat_view_frontier_memo_annotation_query_ns, 3700);
}

#[test]
fn standings_preserve_search_node_split_and_child_cap_metrics() {
    let mut report = sample_report();
    let mut first_match = sample_match(1, "search-d5+tactical-cap-8", "search-d3", None);
    first_match.black_stats = sample_side_stats_with_search_costs();
    first_match.black_stats.search_nodes = 900;
    first_match.black_stats.safety_nodes = 100;
    first_match.black_stats.corridor_nodes = 17;
    first_match.black_stats.corridor_branch_probes = 9;
    first_match.black_stats.corridor_max_depth = 2;
    first_match.black_stats.corridor_width_exits = 6;
    first_match.black_stats.corridor_depth_exits = 5;
    first_match.black_stats.corridor_neutral_exits = 4;
    first_match.black_stats.corridor_terminal_exits = 3;
    first_match.black_stats.corridor_plies_followed = 12;
    first_match.black_stats.corridor_own_plies_followed = 9;
    first_match.black_stats.corridor_opponent_plies_followed = 3;
    first_match.black_stats.corridor_proof_terminal_exits = 13;
    first_match
        .black_stats
        .corridor_proof_terminal_root_candidates = 7;
    first_match
        .black_stats
        .corridor_proof_terminal_root_winning_candidates = 5;
    first_match
        .black_stats
        .corridor_proof_terminal_root_losing_candidates = 2;
    first_match
        .black_stats
        .corridor_proof_terminal_root_overrides = 2;
    first_match
        .black_stats
        .corridor_proof_terminal_root_move_changes = 1;
    first_match
        .black_stats
        .corridor_proof_terminal_root_move_confirmations = 1;
    first_match.black_stats.corridor_proof_candidates_considered = 9;
    first_match.black_stats.corridor_proof_wins = 4;
    first_match.black_stats.corridor_proof_losses = 3;
    first_match.black_stats.corridor_proof_unknown = 2;
    first_match.black_stats.corridor_proof_deadline_skips = 1;
    first_match.black_stats.corridor_proof_move_changes = 1;
    first_match.black_stats.corridor_proof_move_confirmations = 1;
    first_match.black_stats.corridor_proof_candidate_rank_total = 12;
    first_match.black_stats.corridor_proof_candidate_rank_max = 4;
    first_match
        .black_stats
        .corridor_proof_candidate_score_gap_total = 75_000;
    first_match
        .black_stats
        .corridor_proof_candidate_score_gap_max = 50_000;
    first_match
        .black_stats
        .corridor_proof_win_candidate_rank_total = 3;
    first_match
        .black_stats
        .corridor_proof_win_candidate_rank_max = 2;
    first_match.black_stats.effective_depth_sum = 36;
    first_match.black_stats.avg_effective_depth = 7.2;
    first_match.black_stats.max_effective_depth = 9;
    first_match.black_stats.tactical_annotations = 20;
    first_match.black_stats.search_tactical_annotations = 20;
    first_match.black_stats.threat_view_shadow_checks = 30;
    first_match.black_stats.threat_view_shadow_mismatches = 2;
    first_match.black_stats.threat_view_scan_queries = 40;
    first_match.black_stats.threat_view_scan_ns = 5000;
    first_match.black_stats.threat_view_frontier_rebuilds = 6;
    first_match.black_stats.threat_view_frontier_rebuild_ns = 7000;
    first_match.black_stats.threat_view_frontier_queries = 80;
    first_match.black_stats.threat_view_frontier_query_ns = 9000;
    first_match.black_stats.child_limit_applications = 10;
    first_match.black_stats.search_child_limit_applications = 10;
    first_match.black_stats.child_cap_hits = 8;
    first_match.black_stats.search_child_cap_hits = 8;
    first_match.black_stats.child_moves_before_total = 120;
    first_match.black_stats.search_child_moves_before_total = 120;
    first_match.black_stats.child_moves_after_total = 80;
    first_match.black_stats.search_child_moves_after_total = 80;
    first_match.black_stats.avg_child_moves_before = 12.0;
    first_match.black_stats.avg_child_moves_after = 8.0;
    first_match.black_stats.depth_reached_counts = vec![DepthCountReport { depth: 5, count: 5 }];
    report.matches = vec![first_match];
    report.run.bots = vec![
        "search-d5+tactical-cap-8".to_string(),
        "search-d3".to_string(),
    ];
    let results = TournamentResults::new();

    let rows = standings(&report.run.bots, &results, &report.matches, &HashMap::new());
    let row = rows
        .iter()
        .find(|row| row.bot == "search-d5+tactical-cap-8")
        .expect("standing row should exist");

    assert_eq!(row.search_nodes, 900);
    assert_eq!(row.safety_nodes, 100);
    assert_eq!(row.corridor_nodes, 17);
    assert_eq!(row.corridor_branch_probes, 9);
    assert_eq!(row.corridor_max_depth, 2);
    assert_eq!(row.corridor_width_exits, 6);
    assert_eq!(row.corridor_depth_exits, 5);
    assert_eq!(row.corridor_neutral_exits, 4);
    assert_eq!(row.corridor_terminal_exits, 3);
    assert_eq!(row.corridor_plies_followed, 12);
    assert_eq!(row.corridor_own_plies_followed, 9);
    assert_eq!(row.corridor_opponent_plies_followed, 3);
    assert_eq!(row.corridor_proof_terminal_exits, 13);
    assert_eq!(row.corridor_proof_terminal_root_candidates, 7);
    assert_eq!(row.corridor_proof_terminal_root_winning_candidates, 5);
    assert_eq!(row.corridor_proof_terminal_root_losing_candidates, 2);
    assert_eq!(row.corridor_proof_terminal_root_overrides, 2);
    assert_eq!(row.corridor_proof_terminal_root_move_changes, 1);
    assert_eq!(row.corridor_proof_terminal_root_move_confirmations, 1);
    assert_eq!(row.corridor_proof_candidates_considered, 9);
    assert_eq!(row.corridor_proof_wins, 4);
    assert_eq!(row.corridor_proof_losses, 3);
    assert_eq!(row.corridor_proof_unknown, 2);
    assert_eq!(row.corridor_proof_deadline_skips, 1);
    assert_eq!(row.corridor_proof_move_changes, 1);
    assert_eq!(row.corridor_proof_move_confirmations, 1);
    assert_eq!(row.corridor_proof_candidate_rank_total, 12);
    assert_eq!(row.corridor_proof_candidate_rank_max, 4);
    assert_eq!(row.corridor_proof_candidate_score_gap_total, 75_000);
    assert_eq!(row.corridor_proof_candidate_score_gap_max, 50_000);
    assert_eq!(row.corridor_proof_win_candidate_rank_total, 3);
    assert_eq!(row.corridor_proof_win_candidate_rank_max, 2);
    assert_eq!(row.effective_depth_sum, 36);
    assert_eq!(row.avg_effective_depth, 7.2);
    assert_eq!(row.max_effective_depth, 9);
    assert_eq!(row.tactical_annotations, 20);
    assert_eq!(row.threat_view_shadow_checks, 30);
    assert_eq!(row.threat_view_shadow_mismatches, 2);
    assert_eq!(row.threat_view_scan_queries, 40);
    assert_eq!(row.threat_view_scan_ns, 5000);
    assert_eq!(row.threat_view_frontier_rebuilds, 6);
    assert_eq!(row.threat_view_frontier_rebuild_ns, 7000);
    assert_eq!(row.threat_view_frontier_queries, 80);
    assert_eq!(row.threat_view_frontier_query_ns, 9000);
    assert_eq!(row.child_limit_applications, 10);
    assert_eq!(row.child_cap_hits, 8);
    assert_eq!(row.child_moves_before_total, 120);
    assert_eq!(row.child_moves_after_total, 80);
    assert_eq!(row.avg_child_moves_before, 12.0);
    assert_eq!(row.avg_child_moves_after, 8.0);
    assert_eq!(
        row.depth_reached_counts,
        vec![DepthCountReport { depth: 5, count: 5 }]
    );
}

#[test]
fn schedule_summary_uses_played_pairs_for_sparse_schedules() {
    let mut report = sample_report();
    report.run.bots = vec![
        "candidate".to_string(),
        "anchor-a".to_string(),
        "anchor-b".to_string(),
    ];

    assert_eq!(schedule_summary(&report), "1 pair x 2 games = 2 matches");
}

#[test]
fn schedule_summary_describes_batch_gauntlet_shape() {
    let mut report = sample_report();
    report.run.schedule = "gauntlet".to_string();
    report.run.bots = vec![
        "candidate-a".to_string(),
        "candidate-b".to_string(),
        "anchor-a".to_string(),
        "anchor-b".to_string(),
    ];
    report.matches = (0..8)
        .map(|index| sample_match(index + 1, "candidate-a", "anchor-a", None))
        .collect();
    report.reference_anchors = Some(AnchorReferenceReport {
        source: AnchorReferenceSource {
            path: Some("../reports/lab/bot-report.json".to_string()),
            schedule: "round-robin".to_string(),
            git_commit: Some("abc123".to_string()),
            git_dirty: Some(false),
            rules: report.run.rules.clone(),
            games_per_pair: 64,
            opening_policy: "centered-suite".to_string(),
            opening_plies: 4,
            seed: 63,
            search_time_ms: None,
            search_cpu_time_ms: Some(1000),
            search_budget_mode: "strict".to_string(),
            search_cpu_reserve_ms: None,
            search_cpu_max_move_ms: None,
            max_moves: Some(120),
            max_game_ms: None,
        },
        anchors: vec![
            AnchorStandingReport {
                bot: "anchor-a".to_string(),
                sequential_elo: 1200.0,
                shuffled_elo_avg: 1200.0,
                shuffled_elo_stddev: 0.0,
                match_count: 64,
                score_percentage: 50.0,
            },
            AnchorStandingReport {
                bot: "anchor-b".to_string(),
                sequential_elo: 1200.0,
                shuffled_elo_avg: 1200.0,
                shuffled_elo_stddev: 0.0,
                match_count: 64,
                score_percentage: 50.0,
            },
        ],
        pairwise: vec![],
        pair_search: vec![],
    });

    assert_eq!(
        schedule_summary(&report),
        "2 candidates x 2 anchors x 2 games = 8 matches"
    );
}

#[test]
fn searchbot_labels_keep_report_variants_distinct() {
    let report = sample_report();

    assert_eq!(compact_bot_label(&report, "search-d5"), "SearchBot_D5");
    assert_eq!(
        compact_bot_label(&report, "search-d5+tactical-cap-8"),
        "SearchBot_D5+TCap8"
    );
    assert_eq!(
        compact_bot_label(&report, "search-d5+tactical-full-cap-8"),
        "SearchBot_D5+TFullCap8"
    );
    assert_eq!(
        compact_bot_label(&report, "search-d5+tactical-full"),
        "SearchBot_D5+TFull"
    );
    assert_eq!(
        compact_bot_label(&report, "search-d5+tactical-cap-8+pattern-eval"),
        "SearchBot_D5+TCap8+Pattern"
    );
    assert_eq!(
        compact_bot_label(&report, "search-d5+tactical-cap-8+near-self-r2-opponent-r1"),
        "SearchBot_D5+TCap8+SelfR2OppR1"
    );
    assert_eq!(
        compact_bot_label(
            &report,
            "search-d5+tactical-cap-8+pattern-eval+corridor-proof-c16-d8-w3"
        ),
        "SearchBot_D5+TCap8+Pattern+Corridor Proof"
    );
}

#[test]
fn reference_anchors_copy_requested_standings_from_source_report() {
    let mut source = sample_report();
    source.run.schedule = "round-robin".to_string();
    source.provenance.git_commit = Some("abc123".to_string());
    source.provenance.git_dirty = Some(false);
    source.standings = vec![
        sample_standing_with_search_costs("candidate"),
        sample_standing_with_search_costs("anchor-a"),
        sample_standing_with_search_costs("anchor-b"),
    ];
    source.standings[1].shuffled_elo_avg = 1234.5;
    source.standings[1].shuffled_elo_stddev = 12.0;
    source.standings[2].shuffled_elo_avg = 1175.0;
    source.pairwise = vec![
        PairwiseReport {
            bot_a: "anchor-a".to_string(),
            bot_b: "anchor-b".to_string(),
            wins_a: 35,
            wins_b: 29,
            draws: 0,
            total: 64,
            score_a: 35.0,
            score_b: 29.0,
        },
        PairwiseReport {
            bot_a: "candidate".to_string(),
            bot_b: "anchor-a".to_string(),
            wins_a: 31,
            wins_b: 33,
            draws: 0,
            total: 64,
            score_a: 31.0,
            score_b: 33.0,
        },
    ];

    let reference = AnchorReferenceReport::from_report(
        Some("../reports/lab/bot-report.json".to_string()),
        &source,
        &["anchor-a".to_string(), "anchor-b".to_string()],
    )
    .expect("anchors should be copied");

    assert_eq!(
        reference.source.path.as_deref(),
        Some("../reports/lab/bot-report.json")
    );
    assert_eq!(reference.source.schedule, "round-robin");
    assert_eq!(reference.source.git_commit.as_deref(), Some("abc123"));
    assert_eq!(reference.anchors.len(), 2);
    assert_eq!(reference.anchors[0].bot, "anchor-a");
    assert_eq!(reference.anchors[0].shuffled_elo_avg, 1234.5);
    assert_eq!(reference.anchors[0].shuffled_elo_stddev, 12.0);
    assert_eq!(reference.anchors[1].bot, "anchor-b");
    assert_eq!(reference.anchors[1].shuffled_elo_avg, 1175.0);
    assert_eq!(reference.pairwise.len(), 1);
    assert_eq!(reference.pairwise[0].bot_a, "anchor-a");
    assert_eq!(reference.pairwise[0].bot_b, "anchor-b");
}

#[test]
fn reference_anchors_reject_missing_anchor_names() {
    let mut source = sample_report();
    source.standings = vec![sample_standing_with_search_costs("anchor-a")];

    let err = AnchorReferenceReport::from_report(
        None,
        &source,
        &["anchor-a".to_string(), "missing-anchor".to_string()],
    )
    .unwrap_err();

    assert!(err.contains("missing-anchor"));
}

#[test]
fn reference_anchors_require_round_robin_source_report() {
    let mut source = sample_report();
    source.run.schedule = "gauntlet".to_string();
    source.standings = vec![sample_standing_with_search_costs("anchor-a")];

    let err =
        AnchorReferenceReport::from_report(None, &source, &["anchor-a".to_string()]).unwrap_err();

    assert!(err.contains("round-robin"));
}

#[test]
fn reference_anchors_copy_max_limits_from_source_report() {
    let mut source = sample_report();
    source.run.max_moves = Some(120);
    source.run.max_game_ms = Some(10_000);
    source.standings = vec![sample_standing_with_search_costs("anchor-a")];

    let reference = AnchorReferenceReport::from_report(None, &source, &["anchor-a".to_string()])
        .expect("anchor reference should copy limits");

    assert_eq!(reference.source.max_moves, Some(120));
    assert_eq!(reference.source.max_game_ms, Some(10_000));
}

#[test]
fn reference_anchors_validate_matching_eval_context() {
    let mut source = sample_report();
    source.run.max_moves = Some(120);
    source.standings = vec![sample_standing_with_search_costs("anchor-a")];
    let reference = AnchorReferenceReport::from_report(None, &source, &["anchor-a".to_string()])
        .expect("anchor reference should parse");
    let mut run = source.run.clone();

    reference
        .validate_compatible_run(&run)
        .expect("same context should be compatible");

    run.search_cpu_time_ms = Some(500);
    let err = reference.validate_compatible_run(&run).unwrap_err();

    assert!(err.contains("search_cpu_time_ms"));

    let mut run = source.run.clone();
    run.search_budget_mode = "pooled".to_string();
    run.search_cpu_reserve_ms = Some(4_000);
    run.search_cpu_max_move_ms = Some(2_000);
    let err = reference.validate_compatible_run(&run).unwrap_err();

    assert!(err.contains("search_budget_mode"));
    assert!(err.contains("search_cpu_reserve_ms"));
    assert!(err.contains("search_cpu_max_move_ms"));
}

#[test]
fn published_report_keeps_replay_cells_and_drops_debug_metrics() {
    let report = sample_report();
    let published = PublishedTournamentReport::from_tournament_report(&report);
    let json = published
        .to_json()
        .expect("published report should serialize");

    assert_eq!(
        published.schema_version,
        PUBLISHED_TOURNAMENT_REPORT_SCHEMA_VERSION
    );
    assert_eq!(published.report_kind, "published_tournament");
    assert_eq!(
        published.matches[0].move_cells,
        report.matches[0].move_cells
    );
    assert!(json.contains("move_cells"));
    assert!(!json.contains("black_stats"));
    assert!(!json.contains("white_stats"));
    assert!(!json.contains("duration_ms"));
    assert!(!json.contains("\"opening\":"));
    assert!(!json.contains("renju_forbidden_prefilter_checks"));

    let parsed =
        PublishedTournamentReport::from_json(&json).expect("published report should parse");
    assert_eq!(parsed.matches.len(), report.matches.len());
}

#[test]
fn from_json_defaults_missing_search_metrics() {
    let input = r#"{
      "schema_version": 1,
      "report_kind": "tournament",
      "board_size": 15,
      "move_codec": "cell_index_v1",
      "shuffled_elo_samples": 256,
      "run": {
        "bots": ["search-d1", "search-d3"],
        "rules": {"board_size": 15, "win_length": 5, "variant": "renju"},
        "games_per_pair": 2,
        "seed": 42,
        "opening_plies": 4,
        "threads": 1,
        "search_time_ms": null,
        "search_cpu_time_ms": 1000,
        "max_moves": 120,
        "max_game_ms": null
      },
      "standings": [{
        "bot": "search-d1",
        "wins": 1,
        "draws": 0,
        "losses": 1,
        "sequential_elo": 1000.0,
        "shuffled_elo_avg": 1000.0,
        "shuffled_elo_stddev": 0.0,
        "match_count": 2,
        "move_count": 10,
        "search_move_count": 10,
        "total_time_ms": 100,
        "avg_search_time_ms": 10.0,
        "total_nodes": 1000,
        "avg_nodes": 100.0,
        "avg_depth": 3.0,
        "max_depth": 3,
        "budget_exhausted_count": 0,
        "budget_exhausted_rate": 0.0
      }],
      "pairwise": [],
      "color_splits": [],
      "end_reasons": [],
      "matches": [{
        "match_index": 1,
        "black": "search-d1",
        "white": "search-d3",
        "result": "black_won",
        "winner": "search-d1",
        "end_reason": "natural",
        "duration_ms": 100,
        "move_cells": [112, 113],
        "move_count": 2,
        "black_stats": {
          "move_count": 1,
          "search_move_count": 1,
          "total_time_ms": 10,
          "avg_search_time_ms": 10.0,
          "search_nodes": 100,
          "prefilter_nodes": 10,
          "total_nodes": 110,
          "avg_nodes": 110.0,
          "depth_sum": 3,
          "avg_depth": 3.0,
          "max_depth": 3,
          "budget_exhausted_count": 0,
          "budget_exhausted_rate": 0.0
        },
        "white_stats": {
          "move_count": 1,
          "search_move_count": 1,
          "total_time_ms": 10,
          "avg_search_time_ms": 10.0,
          "search_nodes": 100,
          "safety_nodes": 10,
          "total_nodes": 110,
          "avg_nodes": 110.0,
          "depth_sum": 3,
          "avg_depth": 3.0,
          "max_depth": 3,
          "budget_exhausted_count": 0,
          "budget_exhausted_rate": 0.0
        }
      }]
    }"#;

    let report = TournamentReport::from_json(input).expect("report should parse");

    assert_eq!(report.run.schedule, "round-robin");
    assert_eq!(report.run.opening_policy, "centered-suite");
    assert_eq!(report.run.search_budget_mode, "strict");
    assert_eq!(report.run.search_cpu_reserve_ms, None);
    assert_eq!(report.standings[0].eval_calls, 0);
    assert_eq!(report.standings[0].search_candidate_generations, 0);
    assert!(report.matches[0].opening.is_none());
    assert_eq!(report.matches[0].black_stats.safety_nodes, 10);
    assert_eq!(report.matches[0].white_stats.safety_nodes, 10);
    assert_eq!(report.matches[0].black_stats.root_legality_checks, 0);
    assert_eq!(report.matches[0].white_stats.search_legality_checks, 0);
}

#[test]
fn from_json_rejects_unsupported_schema() {
    let input = r#"{
      "schema_version": 999,
      "report_kind": "tournament",
      "board_size": 15,
      "move_codec": "cell_index_v1",
      "shuffled_elo_samples": 256,
      "run": {
        "bots": [],
        "rules": {"board_size": 15, "win_length": 5, "variant": "renju"},
        "games_per_pair": 0,
        "seed": 0,
        "opening_plies": 0,
        "threads": 1,
        "search_time_ms": null,
        "search_cpu_time_ms": null,
        "max_moves": null,
        "max_game_ms": null
      },
      "standings": [],
      "pairwise": [],
      "color_splits": [],
      "end_reasons": [],
      "matches": []
    }"#;

    let err = TournamentReport::from_json(input).unwrap_err();
    assert!(err.contains("unsupported tournament report schema version"));
}

fn sample_report() -> TournamentReport {
    TournamentReport {
        schema_version: TOURNAMENT_REPORT_SCHEMA_VERSION,
        report_kind: "tournament".to_string(),
        board_size: 15,
        move_codec: MOVE_CODEC.to_string(),
        shuffled_elo_samples: SHUFFLED_ELO_SAMPLES,
        provenance: ReportProvenance::default(),
        reference_anchors: None,
        run: TournamentRunReport {
            bots: vec!["search-d1".to_string(), "search-d3".to_string()],
            schedule: "round-robin".to_string(),
            rules: RuleConfig {
                board_size: 15,
                win_length: 5,
                variant: gomoku_core::Variant::Renju,
            },
            games_per_pair: 2,
            seed: 42,
            opening_plies: 4,
            opening_policy: "centered-suite".to_string(),
            threads: 1,
            search_time_ms: None,
            search_cpu_time_ms: Some(1000),
            search_budget_mode: "strict".to_string(),
            search_cpu_reserve_ms: None,
            search_cpu_max_move_ms: None,
            max_moves: Some(120),
            max_game_ms: None,
            total_wall_time_ms: Some(100),
        },
        standings: Vec::new(),
        pairwise: vec![PairwiseReport {
            bot_a: "search-d1".to_string(),
            bot_b: "search-d3".to_string(),
            wins_a: 0,
            wins_b: 2,
            draws: 0,
            total: 2,
            score_a: 0.0,
            score_b: 2.0,
        }],
        color_splits: vec![
            ColorSplitReport {
                black: "search-d1".to_string(),
                white: "search-d3".to_string(),
                black_wins: 0,
                white_wins: 1,
                draws: 0,
                total: 1,
            },
            ColorSplitReport {
                black: "search-d3".to_string(),
                white: "search-d1".to_string(),
                black_wins: 1,
                white_wins: 0,
                draws: 0,
                total: 1,
            },
        ],
        end_reasons: Vec::new(),
        matches: vec![
            sample_match(1, "search-d1", "search-d3", Some("search-d3")),
            sample_match(2, "search-d3", "search-d1", Some("search-d3")),
        ],
    }
}

fn sample_match(index: usize, black: &str, white: &str, winner: Option<&str>) -> MatchReport {
    MatchReport {
        match_index: index,
        black: black.to_string(),
        white: white.to_string(),
        result: if winner.is_some() { "win" } else { "draw" }.to_string(),
        winner: winner.map(str::to_string),
        end_reason: "natural".to_string(),
        duration_ms: Some(100),
        opening: Some(MatchOpeningReport {
            policy: "centered-suite".to_string(),
            index: 0,
            ply_count: 4,
            suite_index: Some(3),
            template_index: Some(0),
            transform_index: Some(3),
        }),
        move_cells: vec![112, 113, 127, 128, 142],
        move_count: 5,
        black_stats: SideStatsReport::default(),
        white_stats: SideStatsReport::default(),
    }
}

fn sample_standing_with_search_costs(bot: &str) -> StandingReport {
    StandingReport {
        bot: bot.to_string(),
        wins: 1,
        draws: 0,
        losses: 1,
        sequential_elo: 1000.0,
        shuffled_elo_avg: 1000.0,
        shuffled_elo_stddev: 0.0,
        match_count: 2,
        move_count: 10,
        search_move_count: 5,
        total_time_ms: 50,
        avg_search_time_ms: 10.0,
        search_nodes: 900,
        safety_nodes: 100,
        corridor_nodes: 0,
        corridor_branch_probes: 0,
        corridor_max_depth: 0,
        corridor_width_exits: 0,
        corridor_depth_exits: 0,
        corridor_neutral_exits: 0,
        corridor_terminal_exits: 0,
        corridor_plies_followed: 0,
        corridor_own_plies_followed: 0,
        corridor_opponent_plies_followed: 0,
        corridor_proof_passes: 0,
        corridor_proof_completed: 0,
        corridor_proof_checks: 0,
        corridor_proof_active: 0,
        corridor_proof_quiet: 0,
        corridor_proof_static_exits: 0,
        corridor_proof_depth_exits: 0,
        corridor_proof_deadline_exits: 0,
        corridor_proof_terminal_exits: 0,
        corridor_proof_terminal_root_candidates: 0,
        corridor_proof_terminal_root_winning_candidates: 0,
        corridor_proof_terminal_root_losing_candidates: 0,
        corridor_proof_terminal_root_overrides: 0,
        corridor_proof_terminal_root_move_changes: 0,
        corridor_proof_terminal_root_move_confirmations: 0,
        corridor_proof_candidates_considered: 0,
        corridor_proof_wins: 0,
        corridor_proof_losses: 0,
        corridor_proof_unknown: 0,
        corridor_proof_deadline_skips: 0,
        corridor_proof_move_changes: 0,
        corridor_proof_move_confirmations: 0,
        corridor_proof_candidate_rank_total: 0,
        corridor_proof_candidate_rank_max: 0,
        corridor_proof_candidate_score_gap_total: 0,
        corridor_proof_candidate_score_gap_max: 0,
        corridor_proof_win_candidate_rank_total: 0,
        corridor_proof_win_candidate_rank_max: 0,
        total_nodes: 1000,
        avg_nodes: 200.0,
        eval_calls: 500,
        avg_eval_calls: 100.0,
        line_shape_eval_calls: 0,
        line_shape_eval_ns: 0,
        avg_line_shape_eval_ns: 0.0,
        pattern_eval_calls: 500,
        pattern_eval_ns: 1_000_000,
        avg_pattern_eval_ns: 2000.0,
        pattern_frame_queries: 0,
        pattern_frame_query_ns: 0,
        avg_pattern_frame_query_ns: 0.0,
        pattern_frame_updates: 0,
        pattern_frame_update_ns: 0,
        avg_pattern_frame_update_ns: 0.0,
        pattern_frame_shadow_checks: 0,
        pattern_frame_shadow_mismatches: 0,
        candidate_generations: 25,
        avg_candidate_generations: 5.0,
        candidate_moves_total: 2500,
        avg_candidate_moves: 100.0,
        candidate_moves_max: 120,
        root_candidate_generations: 5,
        root_candidate_moves_total: 400,
        root_candidate_moves_max: 90,
        search_candidate_generations: 20,
        search_candidate_moves_total: 2100,
        search_candidate_moves_max: 120,
        legality_checks: 30,
        avg_legality_checks: 6.0,
        illegal_moves_skipped: 2,
        root_legality_checks: 10,
        root_illegal_moves_skipped: 1,
        search_legality_checks: 20,
        search_illegal_moves_skipped: 1,
        renju_forbidden_prefilter_checks: 30,
        avg_renju_forbidden_prefilter_checks: 6.0,
        renju_forbidden_prefilter_ns: 500_000,
        avg_renju_forbidden_prefilter_ns: 16_666.7,
        renju_forbidden_checks: 12,
        avg_renju_forbidden_checks: 2.4,
        renju_forbidden_ns: 1_000_000,
        avg_renju_forbidden_ns: 83_333.3,
        renju_forbidden_search_gate_checks: 2,
        renju_forbidden_search_gate_ns: 100_000,
        renju_forbidden_pattern_checks: 6,
        renju_forbidden_pattern_ns: 600_000,
        renju_forbidden_threat_checks: 3,
        renju_forbidden_threat_ns: 250_000,
        renju_forbidden_other_checks: 1,
        renju_forbidden_other_ns: 50_000,
        renju_effective_filter_calls: 8,
        avg_renju_effective_filter_calls: 1.6,
        renju_effective_filter_ns: 2_000_000,
        avg_renju_effective_filter_ns: 250_000.0,
        renju_effective_filter_continuation_checks: 16,
        avg_renju_effective_filter_continuation_checks: 3.2,
        renju_effective_filter_continuation_ns: 1_200_000,
        avg_renju_effective_filter_continuation_ns: 75_000.0,
        stage_move_gen_ns: 5_000_000,
        stage_ordering_ns: 10_000_000,
        stage_eval_ns: 15_000_000,
        stage_threat_ns: 2_500_000,
        stage_proof_ns: 0,
        tactical_annotations: 8,
        root_tactical_annotations: 2,
        search_tactical_annotations: 6,
        threat_view_shadow_checks: 0,
        threat_view_shadow_mismatches: 0,
        threat_view_scan_queries: 0,
        threat_view_scan_ns: 0,
        threat_view_frontier_rebuilds: 0,
        threat_view_frontier_rebuild_ns: 0,
        threat_view_frontier_queries: 0,
        threat_view_frontier_query_ns: 0,
        threat_view_frontier_immediate_win_queries: 0,
        threat_view_frontier_immediate_win_query_ns: 0,
        threat_view_frontier_delta_captures: 0,
        threat_view_frontier_delta_capture_ns: 0,
        threat_view_frontier_move_fact_updates: 0,
        threat_view_frontier_move_fact_update_ns: 0,
        threat_view_frontier_annotation_dirty_marks: 0,
        threat_view_frontier_annotation_dirty_mark_ns: 0,
        threat_view_frontier_clean_annotation_queries: 0,
        threat_view_frontier_clean_annotation_query_ns: 0,
        threat_view_frontier_dirty_annotation_queries: 0,
        threat_view_frontier_dirty_annotation_query_ns: 0,
        threat_view_frontier_fallback_annotation_queries: 0,
        threat_view_frontier_fallback_annotation_query_ns: 0,
        threat_view_frontier_memo_annotation_queries: 0,
        threat_view_frontier_memo_annotation_query_ns: 0,
        child_limit_applications: 4,
        root_child_limit_applications: 0,
        search_child_limit_applications: 4,
        child_cap_hits: 3,
        root_child_cap_hits: 0,
        search_child_cap_hits: 3,
        child_moves_before_total: 48,
        root_child_moves_before_total: 0,
        search_child_moves_before_total: 48,
        child_moves_before_max: 14,
        root_child_moves_before_max: 0,
        search_child_moves_before_max: 14,
        child_moves_after_total: 32,
        root_child_moves_after_total: 0,
        search_child_moves_after_total: 32,
        child_moves_after_max: 9,
        root_child_moves_after_max: 0,
        search_child_moves_after_max: 9,
        avg_child_moves_before: 12.0,
        avg_child_moves_after: 8.0,
        tt_hits: 7,
        tt_cutoffs: 3,
        beta_cutoffs: 9,
        avg_depth: 3.0,
        max_depth: 3,
        effective_depth_sum: 15,
        avg_effective_depth: 3.0,
        max_effective_depth: 3,
        depth_reached_counts: vec![DepthCountReport { depth: 3, count: 5 }],
        budget_exhausted_count: 1,
        budget_exhausted_rate: 0.2,
        pooled_budget_moves: 0,
        pooled_budget_over_base_count: 0,
        pooled_budget_over_base_rate: 0.0,
        pooled_budget_reserve_exhausted_count: 0,
        pooled_budget_reserve_exhausted_rate: 0.0,
        pooled_budget_avg_reserve_before_ms: 0.0,
        pooled_budget_avg_reserve_after_ms: 0.0,
        pooled_budget_min_reserve_after_ms: 0,
        pooled_budget_max_move_budget_ms: 0,
    }
}

fn sample_side_stats_with_search_costs() -> SideStatsReport {
    SideStatsReport {
        move_count: 5,
        search_move_count: 5,
        total_time_ms: 50,
        avg_search_time_ms: 10.0,
        search_nodes: 900,
        safety_nodes: 100,
        corridor_nodes: 0,
        corridor_branch_probes: 0,
        corridor_max_depth: 0,
        corridor_width_exits: 0,
        corridor_depth_exits: 0,
        corridor_neutral_exits: 0,
        corridor_terminal_exits: 0,
        corridor_plies_followed: 0,
        corridor_own_plies_followed: 0,
        corridor_opponent_plies_followed: 0,
        corridor_proof_passes: 0,
        corridor_proof_completed: 0,
        corridor_proof_checks: 0,
        corridor_proof_active: 0,
        corridor_proof_quiet: 0,
        corridor_proof_static_exits: 0,
        corridor_proof_depth_exits: 0,
        corridor_proof_deadline_exits: 0,
        corridor_proof_terminal_exits: 0,
        corridor_proof_terminal_root_candidates: 0,
        corridor_proof_terminal_root_winning_candidates: 0,
        corridor_proof_terminal_root_losing_candidates: 0,
        corridor_proof_terminal_root_overrides: 0,
        corridor_proof_terminal_root_move_changes: 0,
        corridor_proof_terminal_root_move_confirmations: 0,
        corridor_proof_candidates_considered: 0,
        corridor_proof_wins: 0,
        corridor_proof_losses: 0,
        corridor_proof_unknown: 0,
        corridor_proof_deadline_skips: 0,
        corridor_proof_move_changes: 0,
        corridor_proof_move_confirmations: 0,
        corridor_proof_candidate_rank_total: 0,
        corridor_proof_candidate_rank_max: 0,
        corridor_proof_candidate_score_gap_total: 0,
        corridor_proof_candidate_score_gap_max: 0,
        corridor_proof_win_candidate_rank_total: 0,
        corridor_proof_win_candidate_rank_max: 0,
        total_nodes: 1000,
        avg_nodes: 200.0,
        eval_calls: 500,
        avg_eval_calls: 100.0,
        line_shape_eval_calls: 0,
        line_shape_eval_ns: 0,
        avg_line_shape_eval_ns: 0.0,
        pattern_eval_calls: 500,
        pattern_eval_ns: 1_000_000,
        avg_pattern_eval_ns: 2000.0,
        pattern_frame_queries: 0,
        pattern_frame_query_ns: 0,
        avg_pattern_frame_query_ns: 0.0,
        pattern_frame_updates: 0,
        pattern_frame_update_ns: 0,
        avg_pattern_frame_update_ns: 0.0,
        pattern_frame_shadow_checks: 0,
        pattern_frame_shadow_mismatches: 0,
        candidate_generations: 25,
        avg_candidate_generations: 5.0,
        candidate_moves_total: 2500,
        avg_candidate_moves: 100.0,
        candidate_moves_max: 120,
        root_candidate_generations: 5,
        root_candidate_moves_total: 400,
        root_candidate_moves_max: 90,
        search_candidate_generations: 20,
        search_candidate_moves_total: 2100,
        search_candidate_moves_max: 120,
        legality_checks: 30,
        avg_legality_checks: 6.0,
        illegal_moves_skipped: 2,
        root_legality_checks: 10,
        root_illegal_moves_skipped: 1,
        search_legality_checks: 20,
        search_illegal_moves_skipped: 1,
        renju_forbidden_prefilter_checks: 30,
        avg_renju_forbidden_prefilter_checks: 6.0,
        renju_forbidden_prefilter_ns: 500_000,
        avg_renju_forbidden_prefilter_ns: 16_666.7,
        renju_forbidden_checks: 12,
        avg_renju_forbidden_checks: 2.4,
        renju_forbidden_ns: 1_000_000,
        avg_renju_forbidden_ns: 83_333.3,
        renju_forbidden_search_gate_checks: 2,
        renju_forbidden_search_gate_ns: 100_000,
        renju_forbidden_pattern_checks: 6,
        renju_forbidden_pattern_ns: 600_000,
        renju_forbidden_threat_checks: 3,
        renju_forbidden_threat_ns: 250_000,
        renju_forbidden_other_checks: 1,
        renju_forbidden_other_ns: 50_000,
        renju_effective_filter_calls: 8,
        avg_renju_effective_filter_calls: 1.6,
        renju_effective_filter_ns: 2_000_000,
        avg_renju_effective_filter_ns: 250_000.0,
        renju_effective_filter_continuation_checks: 16,
        avg_renju_effective_filter_continuation_checks: 3.2,
        renju_effective_filter_continuation_ns: 1_200_000,
        avg_renju_effective_filter_continuation_ns: 75_000.0,
        stage_move_gen_ns: 5_000_000,
        stage_ordering_ns: 10_000_000,
        stage_eval_ns: 15_000_000,
        stage_threat_ns: 2_500_000,
        stage_proof_ns: 0,
        tactical_annotations: 8,
        root_tactical_annotations: 2,
        search_tactical_annotations: 6,
        threat_view_shadow_checks: 0,
        threat_view_shadow_mismatches: 0,
        threat_view_scan_queries: 0,
        threat_view_scan_ns: 0,
        threat_view_frontier_rebuilds: 0,
        threat_view_frontier_rebuild_ns: 0,
        threat_view_frontier_queries: 0,
        threat_view_frontier_query_ns: 0,
        threat_view_frontier_immediate_win_queries: 0,
        threat_view_frontier_immediate_win_query_ns: 0,
        threat_view_frontier_delta_captures: 0,
        threat_view_frontier_delta_capture_ns: 0,
        threat_view_frontier_move_fact_updates: 0,
        threat_view_frontier_move_fact_update_ns: 0,
        threat_view_frontier_annotation_dirty_marks: 0,
        threat_view_frontier_annotation_dirty_mark_ns: 0,
        threat_view_frontier_clean_annotation_queries: 0,
        threat_view_frontier_clean_annotation_query_ns: 0,
        threat_view_frontier_dirty_annotation_queries: 0,
        threat_view_frontier_dirty_annotation_query_ns: 0,
        threat_view_frontier_fallback_annotation_queries: 0,
        threat_view_frontier_fallback_annotation_query_ns: 0,
        threat_view_frontier_memo_annotation_queries: 0,
        threat_view_frontier_memo_annotation_query_ns: 0,
        child_limit_applications: 4,
        root_child_limit_applications: 0,
        search_child_limit_applications: 4,
        child_cap_hits: 3,
        root_child_cap_hits: 0,
        search_child_cap_hits: 3,
        child_moves_before_total: 48,
        root_child_moves_before_total: 0,
        search_child_moves_before_total: 48,
        child_moves_before_max: 14,
        root_child_moves_before_max: 0,
        search_child_moves_before_max: 14,
        child_moves_after_total: 32,
        root_child_moves_after_total: 0,
        search_child_moves_after_total: 32,
        child_moves_after_max: 9,
        root_child_moves_after_max: 0,
        search_child_moves_after_max: 9,
        avg_child_moves_before: 12.0,
        avg_child_moves_after: 8.0,
        tt_hits: 7,
        tt_cutoffs: 3,
        beta_cutoffs: 9,
        depth_sum: 15,
        avg_depth: 3.0,
        max_depth: 3,
        effective_depth_sum: 15,
        avg_effective_depth: 3.0,
        max_effective_depth: 3,
        depth_reached_counts: vec![DepthCountReport { depth: 3, count: 5 }],
        budget_exhausted_count: 1,
        budget_exhausted_rate: 0.2,
        pooled_budget_moves: 0,
        pooled_budget_over_base_count: 0,
        pooled_budget_over_base_rate: 0.0,
        pooled_budget_reserve_exhausted_count: 0,
        pooled_budget_reserve_exhausted_rate: 0.0,
        pooled_budget_avg_reserve_before_ms: 0.0,
        pooled_budget_avg_reserve_after_ms: 0.0,
        pooled_budget_min_reserve_after_ms: 0,
        pooled_budget_max_move_budget_ms: 0,
    }
}
