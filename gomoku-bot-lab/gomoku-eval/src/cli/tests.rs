use super::*;

#[test]
fn tournament_plan_builds_round_robin_pairs() {
    let plan = tournament_plan(
        CliTournamentSchedule::RoundRobin,
        Some("a,b,c"),
        None,
        None,
        None,
        None,
    )
    .expect("round robin plan should parse");

    assert_eq!(plan.bot_names, vec!["a", "b", "c"]);
    assert!(plan.anchor_names.is_empty());
    assert_eq!(plan.anchor_report, None);
    assert_eq!(
        plan.pairs,
        vec![
            TournamentPair {
                bot_a_idx: 0,
                bot_b_idx: 1,
            },
            TournamentPair {
                bot_a_idx: 0,
                bot_b_idx: 2,
            },
            TournamentPair {
                bot_a_idx: 1,
                bot_b_idx: 2,
            },
        ]
    );
}

#[test]
fn tournament_plan_requires_exactly_two_head_to_head_bots() {
    let plan = tournament_plan(
        CliTournamentSchedule::HeadToHead,
        Some("d5,d7"),
        None,
        None,
        None,
        None,
    )
    .expect("head-to-head plan should parse");

    assert_eq!(plan.bot_names, vec!["d5", "d7"]);
    assert_eq!(
        plan.pairs,
        vec![TournamentPair {
            bot_a_idx: 0,
            bot_b_idx: 1,
        }]
    );

    let err = tournament_plan(
        CliTournamentSchedule::HeadToHead,
        Some("d3,d5,d7"),
        None,
        None,
        None,
        None,
    )
    .unwrap_err();
    assert!(err.contains("exactly 2 bots"));
}

#[test]
fn make_bot_factory_rejects_retired_corridor_lab_aliases() {
    for spec in ["corridor-random", "corridor-d1"] {
        let err = match make_bot_factory(spec, None, None, CliSearchBudgetMode::Strict, 0, None) {
            Ok(_) => panic!("retired corridor bot alias should not parse: {spec}"),
            Err(err) => err,
        };
        assert!(err.contains("search-dN+suffixes"));
    }
}

#[test]
fn make_bot_factory_rejects_retired_corridor_quiescence_suffixes() {
    for spec in ["search-d1+corridor-q", "search-d1+corridor-qd4"] {
        let err =
            match make_bot_factory(spec, None, Some(123), CliSearchBudgetMode::Strict, 0, None) {
                Ok(_) => panic!("retired corridor quiescence suffix should not parse: {spec}"),
                Err(err) => err,
            };
        assert!(err.contains("Unknown bot"));
    }
}

#[test]
fn make_bot_factory_rejects_pooled_max_move_below_base_budget() {
    let err = match make_bot_factory(
        "search-d1",
        None,
        Some(2_000),
        CliSearchBudgetMode::Pooled,
        8_000,
        Some(1_000),
    ) {
        Ok(_) => panic!("pooled max move below base budget should not parse"),
        Err(err) => err,
    };

    assert!(err.contains("--search-cpu-max-move-ms"));
}

#[test]
fn tournament_plan_builds_candidate_vs_anchor_gauntlet() {
    let plan = tournament_plan(
        CliTournamentSchedule::Gauntlet,
        None,
        Some("candidate"),
        None,
        Some("anchor-a,anchor-b"),
        Some("../reports/lab/bot-report.json"),
    )
    .expect("gauntlet plan should parse");

    assert_eq!(plan.bot_names, vec!["candidate", "anchor-a", "anchor-b"]);
    assert_eq!(plan.anchor_names, vec!["anchor-a", "anchor-b"]);
    assert_eq!(
        plan.anchor_report.as_deref(),
        Some("../reports/lab/bot-report.json")
    );
    assert_eq!(
        plan.pairs,
        vec![
            TournamentPair {
                bot_a_idx: 0,
                bot_b_idx: 1,
            },
            TournamentPair {
                bot_a_idx: 0,
                bot_b_idx: 2,
            },
        ]
    );
}

#[test]
fn tournament_plan_builds_batch_candidate_gauntlet() {
    let plan = tournament_plan(
        CliTournamentSchedule::Gauntlet,
        None,
        None,
        Some("candidate-a,candidate-b"),
        Some("anchor-a,anchor-b"),
        Some("../reports/lab/bot-report.json"),
    )
    .expect("batch gauntlet plan should parse");

    assert_eq!(
        plan.bot_names,
        vec!["candidate-a", "candidate-b", "anchor-a", "anchor-b"]
    );
    assert_eq!(plan.anchor_names, vec!["anchor-a", "anchor-b"]);
    assert_eq!(
        plan.anchor_report.as_deref(),
        Some("../reports/lab/bot-report.json")
    );
    assert_eq!(
        plan.pairs,
        vec![
            TournamentPair {
                bot_a_idx: 0,
                bot_b_idx: 2,
            },
            TournamentPair {
                bot_a_idx: 0,
                bot_b_idx: 3,
            },
            TournamentPair {
                bot_a_idx: 1,
                bot_b_idx: 2,
            },
            TournamentPair {
                bot_a_idx: 1,
                bot_b_idx: 3,
            },
        ]
    );
}

#[test]
fn tournament_plan_rejects_mixed_gauntlet_candidate_args() {
    let err = tournament_plan(
        CliTournamentSchedule::Gauntlet,
        None,
        Some("candidate-a"),
        Some("candidate-b"),
        Some("anchor-a"),
        None,
    )
    .unwrap_err();

    assert!(err.contains("either --candidate or --candidates"));
}

#[test]
fn tournament_plan_rejects_candidates_outside_gauntlet() {
    let err = tournament_plan(
        CliTournamentSchedule::RoundRobin,
        Some("a,b"),
        None,
        Some("candidate-a,candidate-b"),
        None,
        None,
    )
    .unwrap_err();

    assert!(err.contains("--candidate/--candidates/--anchors"));
}

#[test]
fn tournament_plan_rejects_anchor_report_outside_gauntlet() {
    let err = tournament_plan(
        CliTournamentSchedule::RoundRobin,
        Some("a,b"),
        None,
        None,
        None,
        Some("../reports/lab/bot-report.json"),
    )
    .unwrap_err();

    assert!(err.contains("--anchor-report"));
}

#[test]
fn tournament_command_parses_pooled_search_budget() {
    let cli = Cli::try_parse_from([
        "gomoku-eval",
        "tournament",
        "--bots",
        "search-d3,search-d5",
        "--search-cpu-time-ms",
        "1000",
        "--search-budget-mode",
        "pooled",
        "--search-cpu-reserve-ms",
        "8000",
        "--search-cpu-max-move-ms",
        "4000",
    ])
    .expect("tournament command should parse");

    let Commands::Tournament { options, .. } = cli.command else {
        panic!("expected tournament command");
    };

    assert_eq!(options.search_cpu_time_ms, Some(1000));
    assert_eq!(options.search_budget_mode, CliSearchBudgetMode::Pooled);
    assert_eq!(options.search_cpu_reserve_ms, 8000);
    assert_eq!(options.search_cpu_max_move_ms, Some(4000));
}

#[test]
fn tournament_command_parses_shadow_mismatch_guard() {
    let cli = Cli::try_parse_from([
        "gomoku-eval",
        "tournament",
        "--bots",
        "search-d3,search-d3+rolling-frontier-shadow",
        "--fail-on-shadow-mismatch",
    ])
    .expect("tournament command should parse");

    let Commands::Tournament {
        fail_on_shadow_mismatch,
        ..
    } = cli.command
    else {
        panic!("expected tournament command");
    };

    assert!(fail_on_shadow_mismatch);
}

#[test]
fn report_json_command_parses_input_and_output() {
    let cli = Cli::try_parse_from([
        "gomoku-eval",
        "report-json",
        "--input",
        "outputs/full-report.json",
        "--output",
        "outputs/report.json",
    ])
    .expect("report-json command should parse");

    let Commands::ReportJson { input, output } = cli.command else {
        panic!("expected report-json command");
    };

    assert_eq!(input, PathBuf::from("outputs/full-report.json"));
    assert_eq!(output, PathBuf::from("outputs/report.json"));
}

#[test]
fn analyze_replay_command_parses_input_output_and_model_limits() {
    let cli = Cli::try_parse_from([
        "gomoku-eval",
        "analyze-replay",
        "--input",
        "replays/match.json",
        "--output",
        "outputs/analysis.json",
        "--max-depth",
        "3",
        "--max-scan-plies",
        "12",
    ])
    .expect("analyze-replay command should parse");

    let Commands::AnalyzeReplay {
        input,
        output,
        max_depth,
        max_scan_plies,
    } = cli.command
    else {
        panic!("expected analyze-replay command");
    };

    assert_eq!(input, PathBuf::from("replays/match.json"));
    assert_eq!(output, Some(PathBuf::from("outputs/analysis.json")));
    assert_eq!(max_depth, 3);
    assert_eq!(max_scan_plies, 12);
}

#[test]
fn analyze_replay_command_defaults_to_bounded_scan_cap() {
    let cli = Cli::try_parse_from([
        "gomoku-eval",
        "analyze-replay",
        "--input",
        "replays/match.json",
    ])
    .expect("analyze-replay command should parse");

    let Commands::AnalyzeReplay { max_scan_plies, .. } = cli.command else {
        panic!("expected analyze-replay command");
    };

    assert_eq!(max_scan_plies, DEFAULT_MAX_SCAN_PLIES);
}

#[test]
fn renju_rules_command_parses_report_and_board_flag() {
    let cli = Cli::try_parse_from([
        "gomoku-eval",
        "renju-rules",
        "--report-json",
        "outputs/renju-rule-fixtures.json",
        "--show-boards",
    ])
    .expect("renju-rules command should parse");

    let Commands::RenjuRules {
        report_json,
        show_boards,
    } = cli.command
    else {
        panic!("expected renju-rules command");
    };

    assert_eq!(
        report_json,
        Some(PathBuf::from("outputs/renju-rule-fixtures.json"))
    );
    assert!(show_boards);
}

#[test]
fn analyze_replay_command_rejects_retired_reply_policy_flag() {
    let err = Cli::try_parse_from([
        "gomoku-eval",
        "analyze-replay",
        "--input",
        "replays/match.json",
        "--defense-policy",
        "all-legal-defense",
    ])
    .unwrap_err();

    assert!(err.to_string().contains("unexpected argument"));
}

#[test]
fn analysis_fixtures_command_parses_report_and_model_limits() {
    let cli = Cli::try_parse_from([
        "gomoku-eval",
        "analysis-fixtures",
        "--report-json",
        "outputs/analysis-fixtures.json",
        "--max-depth",
        "4",
        "--max-scan-plies",
        "16",
    ])
    .expect("analysis-fixtures command should parse");

    let Commands::AnalysisFixtures {
        report_json,
        max_depth,
        max_scan_plies,
    } = cli.command
    else {
        panic!("expected analysis-fixtures command");
    };

    assert_eq!(
        report_json,
        Some(PathBuf::from("outputs/analysis-fixtures.json"))
    );
    assert_eq!(max_depth, 4);
    assert_eq!(max_scan_plies, 16);
}

#[test]
fn analyze_replay_batch_command_parses_reports_and_model_limits() {
    let cli = Cli::try_parse_from([
        "gomoku-eval",
        "analyze-replay-batch",
        "--replay-dir",
        "outputs/replays",
        "--report-json",
        "outputs/analysis-batch.json",
        "--max-depth",
        "3",
        "--max-scan-plies",
        "12",
    ])
    .expect("analyze-replay-batch command should parse");

    let Commands::AnalyzeReplayBatch {
        replay_dir,
        report_json,
        max_depth,
        max_scan_plies,
        include_proof_details,
    } = cli.command
    else {
        panic!("expected analyze-replay-batch command");
    };

    assert_eq!(replay_dir, PathBuf::from("outputs/replays"));
    assert_eq!(
        report_json,
        Some(PathBuf::from("outputs/analysis-batch.json"))
    );
    assert_eq!(max_depth, 3);
    assert_eq!(max_scan_plies, 12);
    assert!(!include_proof_details);
}

#[test]
fn analyze_report_replays_command_parses_matchup_sample_and_model_limits() {
    let cli = Cli::try_parse_from([
        "gomoku-eval",
        "analyze-report-replays",
        "--report",
        "../reports/lab/bot-report.json",
        "--entrant-a",
        "search-d7+tactical-cap-8+pattern-eval",
        "--entrant-b",
        "search-d5+tactical-cap-8+pattern-eval",
        "--sample-size",
        "8",
        "--report-json",
        "outputs/analysis/top2-smoke.json",
        "--published-report-json",
        "../reports/lab/analysis-report.json",
        "--max-depth",
        "4",
        "--max-scan-plies",
        "8",
        "--include-proof-details",
    ])
    .expect("analyze-report-replays command should parse");

    let Commands::AnalyzeReportReplays {
        report,
        selector,
        entrant_a,
        entrant_b,
        sample_size,
        report_json,
        published_report_json,
        max_depth,
        max_scan_plies,
        include_proof_details,
    } = cli.command
    else {
        panic!("expected analyze-report-replays command");
    };

    assert_eq!(report, PathBuf::from("../reports/lab/bot-report.json"));
    assert_eq!(selector, CliReportReplaySelector::HeadToHead);
    assert_eq!(
        entrant_a.as_deref(),
        Some("search-d7+tactical-cap-8+pattern-eval")
    );
    assert_eq!(
        entrant_b.as_deref(),
        Some("search-d5+tactical-cap-8+pattern-eval")
    );
    assert_eq!(sample_size, 8);
    assert_eq!(
        report_json,
        Some(PathBuf::from("outputs/analysis/top2-smoke.json"))
    );
    assert_eq!(
        published_report_json,
        Some(PathBuf::from("../reports/lab/analysis-report.json"))
    );
    assert_eq!(max_depth, 4);
    assert_eq!(max_scan_plies, 8);
    assert!(include_proof_details);
}

#[test]
fn analyze_report_replays_command_parses_preset_triangle_selector() {
    let cli = Cli::try_parse_from([
        "gomoku-eval",
        "analyze-report-replays",
        "--report",
        "../reports/lab/bot-report.json",
        "--selector",
        "preset-triangle",
        "--published-report-json",
        "../reports/lab/analysis-report.json",
    ])
    .expect("analyze-report-replays command should parse");

    let Commands::AnalyzeReportReplays {
        selector,
        report,
        published_report_json,
        ..
    } = cli.command
    else {
        panic!("expected analyze-report-replays command");
    };

    assert_eq!(selector, CliReportReplaySelector::PresetTriangle);
    assert_eq!(report, PathBuf::from("../reports/lab/bot-report.json"));
    assert_eq!(
        published_report_json,
        Some(PathBuf::from("../reports/lab/analysis-report.json"))
    );
}

#[test]
fn report_replay_default_entrant_selection_avoids_self_match() {
    let standings = vec![
        "search-d7+tactical-cap-8+pattern-eval".to_string(),
        "search-d5+tactical-cap-8+pattern-eval".to_string(),
        "search-d3+pattern-eval".to_string(),
    ];

    let (entrant_a, entrant_b) = resolve_report_replay_entrants(
        &standings,
        Some("search-d5+tactical-cap-8+pattern-eval".to_string()),
        None,
    )
    .expect("missing entrant should default to the highest different standing");

    assert_eq!(entrant_a, "search-d5+tactical-cap-8+pattern-eval");
    assert_eq!(entrant_b, "search-d7+tactical-cap-8+pattern-eval");
}

#[test]
fn report_replay_source_label_keeps_default_selector_readable() {
    let report = PathBuf::from("../reports/lab/bot-report.json");

    assert_eq!(
        report_replay_source_label(&report, "search-d7", "search-d5", true),
        "../reports/lab/bot-report.json:Top 2 entrants"
    );
    assert_eq!(
        report_replay_source_label(&report, "search-d7", "search-d5", false),
        "../reports/lab/bot-report.json:search-d7 vs search-d5"
    );
}
