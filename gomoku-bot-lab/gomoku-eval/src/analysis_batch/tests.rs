use std::fs;

use gomoku_bot::tactical::LethalThreatKind;
use gomoku_core::{Board, Color, Move, Replay, RuleConfig, Variant};

use super::{
    add_actual_marker, add_loser_candidate_markers, add_reply_outcome_markers,
    defender_reply_candidates_for_frame, defender_reply_outcomes_for_frame,
    published_analysis_report_from_batch, run_analysis_batch, run_analysis_batch_replays,
    run_analysis_batch_replays_with_options, AnalysisBatchEntry, AnalysisBatchEntryStatus,
    AnalysisBatchModel, AnalysisBatchProofDetails, AnalysisBatchProofFrame,
    AnalysisBatchProofMarker, AnalysisBatchProofMarkerKind, AnalysisBatchReport,
    AnalysisBatchRunOptions, AnalysisBatchSummary, PublishedAnalysisMatchSummary,
    PublishedAnalysisSectionInput, ReplayAnalysisInput, PUBLISHED_ANALYSIS_REPORT_SCHEMA_VERSION,
};
use crate::analysis::{
    analyze_replay, replay_frame_annotations_for_analysis, AnalysisModel, AnalysisOptions,
    DefenderReplyAnalysis, DefenderReplyCandidate, DefenderReplyOutcome, DefenderReplyRole,
    FailureMode, ForcedInterval, GameAnalysis, ProofLimitCause, ProofStatus,
    ReplayFrameHighlightRole, ReplayFrameMarkerRole, ReplyClassification, RootCause,
    SearchDiagnostics, UnclearReason, ANALYSIS_SCHEMA_VERSION,
};

fn replay_from_moves(variant: Variant, moves: &[&str]) -> Replay {
    let rules = RuleConfig {
        variant,
        ..RuleConfig::default()
    };
    let mut board = Board::new(rules.clone());
    let mut replay = Replay::new(rules, "Black", "White");

    for notation in moves {
        let parsed = Move::from_notation(notation).expect("test move notation should parse");
        board
            .apply_move(parsed)
            .expect("test replay move should be legal");
        replay.push_move(parsed, 0, board.hash(), None);
    }
    replay.finish(&board.result, Some(0));
    replay
}

fn mv(notation: &str) -> Move {
    Move::from_notation(notation).expect("test move notation should parse")
}

fn board_from_moves(variant: Variant, moves: &[&str]) -> Board {
    let mut board = Board::new(RuleConfig {
        variant,
        ..RuleConfig::default()
    });
    for notation in moves {
        board
            .apply_move(mv(notation))
            .expect("test board move should be legal");
    }
    board
}

fn analysis_for_winner(winner: Color, rule_set: &str, max_depth: usize) -> GameAnalysis {
    GameAnalysis {
        schema_version: ANALYSIS_SCHEMA_VERSION,
        rule_set: rule_set.to_string(),
        winner: Some(winner),
        loser: Some(winner.opponent()),
        final_move: None,
        final_winning_line: Vec::new(),
        model: AnalysisModel {
            rule_set: rule_set.to_string(),
            max_depth,
            max_scan_plies: Some(64),
        },
        lethal_onset: None,
        setup_corridor: None,
        final_forced_interval_found: false,
        final_forced_interval: ForcedInterval {
            start_ply: 0,
            end_ply: 0,
        },
        proof_intervals: Vec::new(),
        unknown_gaps: Vec::new(),
        unclear_reason: None,
        unclear_context: None,
        last_chance_ply: None,
        decisive_attack_ply: None,
        critical_loser_ply: None,
        root_cause: RootCause::Unclear,
        failure: None,
        tactical_notes: Vec::new(),
        principal_line: Vec::new(),
        proof_summary: Vec::new(),
    }
}

fn reply_candidate(notation: &str, roles: Vec<DefenderReplyRole>) -> DefenderReplyCandidate {
    let mv = mv(notation);
    DefenderReplyCandidate {
        mv,
        notation: mv.to_notation(),
        roles,
    }
}

fn reply_analysis(
    notation: &str,
    roles: Vec<DefenderReplyRole>,
    outcome: DefenderReplyOutcome,
) -> DefenderReplyAnalysis {
    let mv = mv(notation);
    DefenderReplyAnalysis {
        mv,
        notation: mv.to_notation(),
        roles,
        outcome,
        principal_line: Vec::new(),
        principal_line_notation: Vec::new(),
        limit_causes: Vec::new(),
        diagnostics: SearchDiagnostics::default(),
    }
}

fn proof_frame_with_markers(
    side_to_move: Color,
    markers: Vec<AnalysisBatchProofMarker>,
    reply_outcomes: Vec<DefenderReplyAnalysis>,
) -> AnalysisBatchProofFrame {
    AnalysisBatchProofFrame {
        label: "test_frame".to_string(),
        ply: 0,
        side_to_move,
        status: ProofStatus::ForcedWin,
        move_played: None,
        move_played_notation: None,
        lethal_onset_reached: false,
        rows: Vec::new(),
        markers,
        reply_outcomes,
    }
}

fn temp_report_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "gomoku-analysis-batch-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("temp report dir should be created");
    dir
}

#[test]
fn published_analysis_report_keeps_ui_frames_and_drops_debug_details() {
    let mut frame = proof_frame_with_markers(
        Color::White,
        vec![AnalysisBatchProofMarker {
            mv: mv("H8"),
            notation: "H8".to_string(),
            kinds: vec![AnalysisBatchProofMarkerKind::Threat],
        }],
        vec![reply_analysis(
            "H8",
            vec![DefenderReplyRole::ImmediateDefense],
            DefenderReplyOutcome::ForcedLoss,
        )],
    );
    frame.rows = vec!["debug board row".to_string()];
    let interval = ForcedInterval {
        start_ply: 1,
        end_ply: 2,
    };
    let entry = AnalysisBatchEntry {
        path: "match_0001__bot_a__vs__bot_b".to_string(),
        status: AnalysisBatchEntryStatus::Analyzed,
        winner: Some(Color::Black),
        move_count: Some(2),
        root_cause: Some(RootCause::CorridorEntry),
        unclear_reason: None,
        final_move: Some(mv("H8")),
        lethal_onset: None,
        setup_corridor: Some(interval.clone()),
        final_forced_interval_found: true,
        final_forced_interval: Some(interval.clone()),
        proof_intervals: vec![interval],
        last_chance_ply: Some(1),
        critical_loser_ply: Some(2),
        tactical_notes: Vec::new(),
        failure: None,
        principal_line: Vec::new(),
        unknown_gaps: Vec::new(),
        unknown_gap_count: 0,
        unclear_context: None,
        proof_details: Some(AnalysisBatchProofDetails {
            previous_prefix_ply: Some(1),
            final_forced_start_ply: 2,
            previous_proof: None,
            final_start_proof: None,
            snapshots: Vec::new(),
            proof_frames: vec![frame],
        }),
        proof_detail_diagnostics: Some(SearchDiagnostics {
            search_nodes: 99,
            branch_probes: 3,
            max_depth_reached: 4,
        }),
        limit_causes: Vec::new(),
        elapsed_ms: 7,
        prefixes_analyzed: 1,
        forced_prefix_count: 1,
        unknown_prefix_count: 0,
        escape_prefix_count: 0,
        error: None,
    };
    let batch = AnalysisBatchReport {
        schema_version: ANALYSIS_SCHEMA_VERSION,
        source_kind: "report_replays".to_string(),
        source: "outputs/full-report.json:Preset triangle".to_string(),
        replay_dir: "outputs/full-report.json:Preset triangle".to_string(),
        total: 1,
        analyzed: 1,
        failed: 0,
        elapsed_ms: 7,
        total_elapsed_ms: 7,
        model: AnalysisBatchModel {
            max_depth: 4,
            max_scan_plies: Some(64),
        },
        summary: AnalysisBatchSummary::default(),
        limit_cause_counts: Vec::new(),
        entries: vec![entry],
    };
    let published = published_analysis_report_from_batch(
        "outputs/full-report.json".to_string(),
        None,
        "Preset triangle".to_string(),
        &batch,
        &[PublishedAnalysisSectionInput {
            label: "Easy vs Normal".to_string(),
            entrant_a: "search-d1".to_string(),
            entrant_b: "search-d3+pattern-eval".to_string(),
            matches: vec![PublishedAnalysisMatchSummary {
                match_index: 1,
                black: "bot-a".to_string(),
                white: "bot-b".to_string(),
                result: "black_won".to_string(),
                winner: Some("bot-a".to_string()),
                end_reason: "win".to_string(),
                move_cells: vec![112, 113],
                move_count: 2,
            }],
        }],
    )
    .expect("published analysis report should build");
    let json = published
        .to_json()
        .expect("published analysis report should serialize");

    assert_eq!(published.report_kind, "published_analysis");
    assert_eq!(
        published.schema_version,
        PUBLISHED_ANALYSIS_REPORT_SCHEMA_VERSION
    );
    assert!(json.contains("\"provenance\""));
    assert_eq!(published.sections[0].entries[0].match_report.match_index, 1);
    assert!(json.contains("\"proof_frames\""));
    assert!(json.contains("\"markers\""));
    assert!(json.contains("\"reply_outcomes\""));
    assert!(!json.contains("proof_detail_diagnostics"));
    assert!(json.contains("\"search_details\""));
    assert!(json.contains("\"search_nodes\""));
    assert!(json.contains("\"branch_probes\""));
    assert!(json.contains("\"max_depth_reached\""));
    assert!(!json.contains("debug board row"));
}

#[test]
fn analysis_batch_groups_replay_directory_by_root_cause() {
    let dir = temp_report_dir("root-cause");
    let missed_defense = replay_from_moves(
        Variant::Freestyle,
        &["H8", "G8", "I8", "A1", "J8", "A2", "K8", "B1", "L8"],
    );
    fs::write(
        dir.join("missed_defense.json"),
        missed_defense
            .to_json()
            .expect("test replay should serialize"),
    )
    .expect("test replay should write");

    let report =
        run_analysis_batch(&dir, AnalysisOptions::default()).expect("batch analysis should run");

    assert_eq!(report.total, 1);
    assert_eq!(report.analyzed, 1);
    assert_eq!(report.failed, 0);
    assert_eq!(report.model.max_depth, AnalysisOptions::default().max_depth);
    assert_eq!(report.summary.unclear, 0);
    assert_eq!(report.entries[0].root_cause, Some(RootCause::MissedDefense));
    assert_eq!(
        report.entries[0]
            .failure
            .as_ref()
            .map(|failure| failure.mode),
        Some(FailureMode::MissedImmediateResponse)
    );
    assert_eq!(report.entries[0].path, "missed_defense.json");

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn analysis_batch_replays_preserves_input_order_and_records_work_metrics() {
    let first = replay_from_moves(
        Variant::Freestyle,
        &["H8", "G8", "I8", "A1", "J8", "A2", "K8", "B1", "L8"],
    );
    let second = replay_from_moves(
        Variant::Freestyle,
        &["H8", "A1", "I8", "A2", "J8", "A3", "K8", "A4", "L8"],
    );

    let report = run_analysis_batch_replays(
        "report.json:bot-a vs bot-b".to_string(),
        vec![
            ReplayAnalysisInput {
                label: "match_0002".to_string(),
                replay: second,
            },
            ReplayAnalysisInput {
                label: "match_0001".to_string(),
                replay: first,
            },
        ],
        AnalysisOptions::default(),
    );

    assert_eq!(report.source_kind, "report_replays");
    assert_eq!(report.source, "report.json:bot-a vs bot-b");
    assert_eq!(report.entries[0].path, "match_0002");
    assert_eq!(report.entries[1].path, "match_0001");
    assert!(report.entries[0].proof_details.is_none());
    assert!(report.entries[0].final_forced_interval_found);
    assert!(
        report.entries[0].unclear_reason.is_some()
            || report.entries[0].root_cause != Some(RootCause::Unclear)
    );
    assert_eq!(
        report.entries[0].prefixes_analyzed,
        report.entries[0].forced_prefix_count
            + report.entries[0].unknown_prefix_count
            + report.entries[0].escape_prefix_count
    );
    assert_eq!(
        report.total_elapsed_ms,
        report
            .entries
            .iter()
            .map(|entry| entry.elapsed_ms)
            .sum::<u64>()
    );
}

#[test]
fn analysis_batch_replays_records_scan_cap_drilldown_context() {
    let replay = replay_from_moves(
        Variant::Freestyle,
        &["H8", "A1", "I8", "A2", "J8", "A3", "K8", "B1", "L8"],
    );

    let report = run_analysis_batch_replays(
        "report.json:bot-a vs bot-b".to_string(),
        vec![ReplayAnalysisInput {
            label: "proof_limit_case".to_string(),
            replay,
        }],
        AnalysisOptions {
            max_scan_plies: Some(0),
            ..AnalysisOptions::default()
        },
    );

    let entry = &report.entries[0];
    let context = entry
        .unclear_context
        .as_ref()
        .expect("scan-cap-limited entries should expose drilldown context");

    assert_eq!(entry.unclear_reason, Some(UnclearReason::ScanWindowCutoff));
    assert_eq!(context.previous_prefix_ply, Some(8));
    assert_eq!(context.previous_proof_status, None);
    assert_eq!(context.previous_proof_limit_hit, None);
    assert!(context
        .previous_limit_causes
        .contains(&ProofLimitCause::OutsideScanWindow));
    assert!(entry
        .limit_causes
        .contains(&ProofLimitCause::OutsideScanWindow));
    assert!(report
        .limit_cause_counts
        .iter()
        .any(|count| count.cause == ProofLimitCause::OutsideScanWindow && count.count == 1));
    assert_eq!(context.move_count, 9);
    assert!(context.principal_line.is_empty());
    assert!(context.principal_line_notation.is_empty());
    assert!(context
        .snapshots
        .iter()
        .any(|snapshot| snapshot.label == "previous_prefix" && snapshot.ply == 8));
}

#[test]
fn analysis_batch_replays_can_include_decisive_proof_details() {
    let replay = replay_from_moves(
        Variant::Freestyle,
        &["H8", "G8", "I8", "A1", "J8", "A2", "K8", "B1", "L8"],
    );

    let report = run_analysis_batch_replays_with_options(
        "report.json:bot-a vs bot-b".to_string(),
        vec![ReplayAnalysisInput {
            label: "missed_defense".to_string(),
            replay,
        }],
        AnalysisBatchRunOptions {
            analysis: AnalysisOptions::default(),
            include_proof_details: true,
        },
    );

    let entry = &report.entries[0];
    assert_eq!(entry.root_cause, Some(RootCause::MissedDefense));
    let details = entry
        .proof_details
        .as_ref()
        .expect("opt-in proof details should be recorded for decisive entries");

    assert_eq!(details.previous_prefix_ply, Some(7));
    assert_eq!(details.final_forced_start_ply, 8);

    let previous = details
        .previous_proof
        .as_ref()
        .expect("previous prefix proof should be available");
    assert_eq!(previous.prefix_ply, 7);
    assert_eq!(previous.status, ProofStatus::EscapeFound);
    assert_eq!(
        previous.reply_classification,
        Some(ReplyClassification::ConfirmedEscape)
    );
    assert_eq!(
        previous.escape_replies,
        vec![Move::from_notation("L8").unwrap()]
    );
    assert_eq!(
        previous.winning_squares,
        vec![Move::from_notation("L8").unwrap()]
    );

    let final_start = details
        .final_start_proof
        .as_ref()
        .expect("final forced start proof should be available");
    assert_eq!(final_start.prefix_ply, 8);
    assert_eq!(final_start.status, ProofStatus::ForcedWin);
    assert_eq!(
        final_start.principal_line,
        vec![Move::from_notation("L8").unwrap()]
    );
    assert_eq!(final_start.principal_line_notation, vec!["L8".to_string()]);

    assert!(details
        .snapshots
        .iter()
        .any(|snapshot| snapshot.label == "escape_boundary" && snapshot.ply == 7));
    assert!(details
        .snapshots
        .iter()
        .any(|snapshot| snapshot.label == "forced_entry" && snapshot.ply == 8));

    assert_eq!(
        details
            .proof_frames
            .iter()
            .map(|frame| (frame.label.as_str(), frame.ply))
            .collect::<Vec<_>>(),
        vec![("winning_ply", 9), ("actual_ply_8", 8), ("actual_ply_7", 7)]
    );
    assert!(details
        .proof_frames
        .iter()
        .all(|frame| frame
            .markers
            .iter()
            .all(|marker| marker.kinds.iter().all(|kind| matches!(
                kind,
                AnalysisBatchProofMarkerKind::Winning
                    | AnalysisBatchProofMarkerKind::Threat
                    | AnalysisBatchProofMarkerKind::ImminentDefense
                    | AnalysisBatchProofMarkerKind::OffensiveCounter
                    | AnalysisBatchProofMarkerKind::WinningEvidence
                    | AnalysisBatchProofMarkerKind::ThreatEvidence
                    | AnalysisBatchProofMarkerKind::ImminentEvidence
                    | AnalysisBatchProofMarkerKind::OffensiveEvidence
                    | AnalysisBatchProofMarkerKind::CorridorEntryBlack
                    | AnalysisBatchProofMarkerKind::CorridorEntryWhite
                    | AnalysisBatchProofMarkerKind::Forbidden
                    | AnalysisBatchProofMarkerKind::ForcedLoss
                    | AnalysisBatchProofMarkerKind::ConfirmedEscape
                    | AnalysisBatchProofMarkerKind::PossibleEscape
                    | AnalysisBatchProofMarkerKind::ImmediateLoss
                    | AnalysisBatchProofMarkerKind::UnknownOutcome
                    | AnalysisBatchProofMarkerKind::Actual
            )))));
    let winning_frame = details
        .proof_frames
        .first()
        .expect("winning-ply frame should be first");
    assert_eq!(winning_frame.side_to_move, Color::Black);
    let actual_l8 = winning_frame
        .markers
        .iter()
        .find(|marker| marker.notation == "L8")
        .expect("winning frame should mark the actual winning move");
    assert!(actual_l8
        .kinds
        .contains(&AnalysisBatchProofMarkerKind::Actual));
    assert_eq!(actual_l8.kinds, vec![AnalysisBatchProofMarkerKind::Actual]);

    let attacker_frame = details
        .proof_frames
        .iter()
        .find(|frame| frame.label == "actual_ply_7" && frame.ply == 7)
        .expect("winner-side setup frame should be recorded");
    assert_eq!(attacker_frame.side_to_move, Color::Black);
    assert!(attacker_frame.markers.iter().all(|marker| {
        marker
            .kinds
            .iter()
            .all(|kind| matches!(kind, AnalysisBatchProofMarkerKind::Actual))
    }));

    let final_frame = details
        .proof_frames
        .iter()
        .find(|frame| frame.label == "actual_ply_8" && frame.ply == 8)
        .expect("forced-interval decision frame should be recorded");
    assert_eq!(final_frame.side_to_move, Color::White);
    assert_eq!(final_frame.move_played_notation.as_deref(), Some("B1"));
    let final_actual = final_frame
        .markers
        .iter()
        .find(|marker| marker.notation == "B1")
        .expect("final frame should mark the actual replay move");
    assert!(final_actual
        .kinds
        .contains(&AnalysisBatchProofMarkerKind::Actual));
    assert_eq!(
        final_actual.kinds,
        vec![AnalysisBatchProofMarkerKind::Actual]
    );
    let final_l8 = final_frame
        .markers
        .iter()
        .find(|marker| marker.notation == "L8")
        .expect("final frame should mark the L8 losing square");
    assert!(final_l8
        .kinds
        .contains(&AnalysisBatchProofMarkerKind::Threat));
}

#[test]
fn analysis_batch_report_uses_perspective_status_after_lethal_onset() {
    let replay = replay_from_moves(
        Variant::Freestyle,
        &[
            "H8", "G8", "I8", "A1", "J8", "O1", "K8", "A15", "I7", "O15", "I9", "L8", "I6", "A14",
            "I10",
        ],
    );

    let report = run_analysis_batch_replays_with_options(
        "report.json:bot-a vs bot-b".to_string(),
        vec![ReplayAnalysisInput {
            label: "lethal_onset".to_string(),
            replay,
        }],
        AnalysisBatchRunOptions {
            analysis: AnalysisOptions::default(),
            include_proof_details: true,
        },
    );

    let entry = &report.entries[0];
    let onset = entry
        .lethal_onset
        .as_ref()
        .expect("analysis entry should carry lethal onset evidence");
    assert_eq!(onset.prefix_ply, 11);
    assert_eq!(onset.kind, LethalThreatKind::OneStepCoverage);
    assert_eq!(onset.shape.label, "4x3");
    let setup_corridor = entry
        .setup_corridor
        .as_ref()
        .expect("analysis entry should carry setup corridor evidence");
    assert_eq!(setup_corridor.end_ply, onset.prefix_ply);

    let details = entry
        .proof_details
        .as_ref()
        .expect("proof details should be recorded");
    let onset_frame = details
        .proof_frames
        .iter()
        .find(|frame| frame.ply == 12)
        .expect("onset frame should be recorded");
    assert_eq!(onset_frame.ply, 12);
    assert!(onset_frame.lethal_onset_reached);
    let l8 = marker_for(onset_frame, "L8");
    assert!(l8.kinds.contains(&AnalysisBatchProofMarkerKind::Threat));
    let i6 = marker_for(onset_frame, "I6");
    assert!(i6.kinds.contains(&AnalysisBatchProofMarkerKind::Threat));
    assert!(i6
        .kinds
        .contains(&AnalysisBatchProofMarkerKind::ImmediateLoss));
    let i10 = marker_for(onset_frame, "I10");
    assert!(i10.kinds.contains(&AnalysisBatchProofMarkerKind::Threat));
    assert!(i10
        .kinds
        .contains(&AnalysisBatchProofMarkerKind::ImmediateLoss));
    assert!(marker_for(onset_frame, "H8")
        .kinds
        .contains(&AnalysisBatchProofMarkerKind::ThreatEvidence));
    assert!(marker_for(onset_frame, "I7")
        .kinds
        .contains(&AnalysisBatchProofMarkerKind::ThreatEvidence));
}

#[test]
fn analysis_batch_marks_pre_corridor_entry_as_escape_target() {
    let replay = replay_from_moves(
        Variant::Renju,
        &[
            "H8", "H9", "I8", "I9", "G8", "J8", "G6", "H10", "K7", "G11", "F12", "F10", "J9", "H7",
            "E8", "F8", "F7", "H5", "C10", "D9", "G5", "G7", "H6", "E9", "D8", "G9", "F9", "E7",
            "D6", "I11",
        ],
    );

    let report = run_analysis_batch_replays_with_options(
        "report.json:bot-a vs bot-b".to_string(),
        vec![ReplayAnalysisInput {
            label: "pre_corridor_escape".to_string(),
            replay,
        }],
        AnalysisBatchRunOptions {
            analysis: AnalysisOptions {
                max_depth: 4,
                max_scan_plies: Some(8),
            },
            include_proof_details: true,
        },
    );

    let details = report.entries[0]
        .proof_details
        .as_ref()
        .expect("proof details should be recorded");
    assert_eq!(details.final_forced_start_ply, 23);
    let frame = details
        .proof_frames
        .iter()
        .find(|frame| frame.label == "actual_ply_23")
        .expect("pre-corridor escape frame should be present");
    assert_eq!(frame.side_to_move, Color::Black);
    assert_eq!(frame.status, ProofStatus::EscapeFound);
    assert!(frame.reply_outcomes.is_empty());

    let e9 = frame
        .markers
        .iter()
        .find(|marker| marker.notation == "E9")
        .expect("winner corridor entry should be shown as an escape target");
    assert!(e9
        .kinds
        .contains(&AnalysisBatchProofMarkerKind::CorridorEntryWhite));
    assert!(e9
        .kinds
        .contains(&AnalysisBatchProofMarkerKind::ConfirmedEscape));
}

#[test]
fn analysis_batch_marks_attacker_started_corridor_entry_as_escape_target() {
    let replay = replay_from_moves(
        Variant::Renju,
        &[
            "H8", "H9", "I8", "I9", "G8", "J8", "J9", "K7", "H10", "H7", "G9", "L6", "M5", "I7",
            "G7", "G6", "F8", "E8", "E7", "D6", "I11",
        ],
    );

    let report = run_analysis_batch_replays_with_options(
        "report.json:bot-a vs bot-b".to_string(),
        vec![ReplayAnalysisInput {
            label: "attacker_started_corridor_entry".to_string(),
            replay,
        }],
        AnalysisBatchRunOptions {
            analysis: AnalysisOptions {
                max_depth: 4,
                max_scan_plies: Some(64),
            },
            include_proof_details: true,
        },
    );

    let details = report.entries[0]
        .proof_details
        .as_ref()
        .expect("proof details should be recorded");
    assert_eq!(details.final_forced_start_ply, 13);
    let frame = details
        .proof_frames
        .iter()
        .find(|frame| frame.label == "actual_ply_14")
        .expect("defender frame after attacker corridor entry should be present");
    assert_eq!(frame.side_to_move, Color::White);
    assert_eq!(frame.status, ProofStatus::ForcedWin);
    assert!(frame.reply_outcomes.is_empty());

    let g7 = frame
        .markers
        .iter()
        .find(|marker| marker.notation == "G7")
        .expect("winner corridor entry should be shown as an escape target");
    assert!(g7
        .kinds
        .contains(&AnalysisBatchProofMarkerKind::CorridorEntryBlack));
    assert!(g7
        .kinds
        .contains(&AnalysisBatchProofMarkerKind::ConfirmedEscape));
}

#[test]
fn shared_replay_annotations_match_report_corridor_entry_boundary() {
    let replay = replay_from_moves(
        Variant::Renju,
        &[
            "H8", "H9", "I8", "I9", "G8", "J8", "J9", "K7", "H10", "H7", "G9", "L6", "M5", "I7",
            "G7", "G6", "F8", "E8", "E7", "D6", "I11",
        ],
    );
    let options = AnalysisOptions {
        max_depth: 4,
        max_scan_plies: Some(64),
    };
    let analysis = analyze_replay(&replay, options).expect("analysis should run");
    let annotations = replay_frame_annotations_for_analysis(&replay, &analysis)
        .expect("shared replay annotations should build");

    let boundary = annotations
        .iter()
        .find(|frame| frame.ply == 13)
        .expect("shared annotation should include the report corridor-entry boundary");
    assert!(boundary.highlights.iter().any(|highlight| {
        highlight.role == ReplayFrameHighlightRole::CorridorEntry
            && highlight.notation == "G7"
            && highlight.side == Color::Black
    }));
    assert!(boundary.markers.iter().any(|marker| {
        marker.role == ReplayFrameMarkerRole::ConfirmedEscape
            && marker.notation == "G7"
            && marker.side == Color::White
    }));
}

#[test]
fn analysis_batch_actual_marker_keeps_counter_hint() {
    let board = board_from_moves(
        Variant::Renju,
        &[
            "H8", "G7", "H6", "H7", "I7", "F7", "G5", "F4", "J8", "K9", "I8", "G8", "I6", "I9",
            "H9", "F6", "F5", "G9", "G10",
        ],
    );
    assert_eq!(board.current_player, Color::White);

    let mut markers = Vec::new();
    add_actual_marker(&mut markers, &board, Some(Color::Black), mv("E7"));

    let marker = proof_marker_for(&markers, "E7");
    assert!(marker
        .kinds
        .contains(&AnalysisBatchProofMarkerKind::OffensiveCounter));
    assert!(marker.kinds.contains(&AnalysisBatchProofMarkerKind::Actual));
}

#[test]
fn analysis_batch_reply_markers_combine_roles_and_outcomes() {
    let board = board_from_moves(Variant::Renju, &["H8"]);
    let mut markers = Vec::new();
    add_loser_candidate_markers(
        &mut markers,
        &board,
        Some(Color::Black),
        &[reply_candidate(
            "G7",
            vec![
                DefenderReplyRole::ImminentDefense,
                DefenderReplyRole::Actual,
            ],
        )],
    );
    add_actual_marker(&mut markers, &board, Some(Color::Black), mv("G7"));

    let replies = vec![
        reply_analysis(
            "G4",
            vec![DefenderReplyRole::ImminentDefense],
            DefenderReplyOutcome::ForcedLoss,
        ),
        reply_analysis(
            "G9",
            vec![DefenderReplyRole::ImminentDefense],
            DefenderReplyOutcome::ForcedLoss,
        ),
        reply_analysis(
            "I10",
            vec![DefenderReplyRole::OffensiveCounter],
            DefenderReplyOutcome::PossibleEscape,
        ),
        reply_analysis(
            "I11",
            vec![DefenderReplyRole::OffensiveCounter],
            DefenderReplyOutcome::ForcedLoss,
        ),
    ];
    add_reply_outcome_markers(&mut markers, &replies);
    let frame = proof_frame_with_markers(Color::White, markers, replies);
    let frame = &frame;

    let g4 = marker_for(frame, "G4");
    assert!(g4
        .kinds
        .contains(&AnalysisBatchProofMarkerKind::ImminentDefense));
    assert!(g4.kinds.contains(&AnalysisBatchProofMarkerKind::ForcedLoss));

    let g9 = marker_for(frame, "G9");
    assert!(g9
        .kinds
        .contains(&AnalysisBatchProofMarkerKind::ImminentDefense));
    assert!(g9.kinds.contains(&AnalysisBatchProofMarkerKind::ForcedLoss));

    let g7 = marker_for(frame, "G7");
    assert_eq!(
        g7.kinds,
        vec![
            AnalysisBatchProofMarkerKind::ImminentDefense,
            AnalysisBatchProofMarkerKind::Actual,
        ]
    );
    assert!(!frame
        .reply_outcomes
        .iter()
        .any(|reply| reply.notation == "G7"));

    let i10 = marker_for(frame, "I10");
    assert!(i10
        .kinds
        .contains(&AnalysisBatchProofMarkerKind::OffensiveCounter));
    assert!(i10
        .kinds
        .contains(&AnalysisBatchProofMarkerKind::PossibleEscape));

    let i11 = marker_for(frame, "I11");
    assert!(i11
        .kinds
        .contains(&AnalysisBatchProofMarkerKind::OffensiveCounter));
    assert!(i11
        .kinds
        .contains(&AnalysisBatchProofMarkerKind::ForcedLoss));
}

#[test]
fn analysis_batch_visual_frames_filter_forbidden_costs_to_current_prefix() {
    let replay = replay_from_moves(
        Variant::Renju,
        &[
            "H8", "G7", "H9", "J8", "H7", "H6", "I7", "F8", "E9", "H10", "I5", "G9", "E7", "I9",
            "K7", "G10", "G11", "I11", "J12", "G6", "G8", "F6", "L7", "J7", "J6", "E6", "D6", "I6",
        ],
    );

    let report = run_analysis_batch_replays_with_options(
        "report.json:bot-a vs bot-b".to_string(),
        vec![ReplayAnalysisInput {
            label: "renju_forbidden_cost_prefix_scope".to_string(),
            replay,
        }],
        AnalysisBatchRunOptions {
            analysis: AnalysisOptions {
                max_depth: 4,
                max_scan_plies: Some(64),
            },
            include_proof_details: true,
        },
    );

    let frames = &report.entries[0]
        .proof_details
        .as_ref()
        .expect("proof details should be present")
        .proof_frames;
    for label in ["actual_ply_24", "actual_ply_26"] {
        let frame = frames
            .iter()
            .find(|frame| frame.label == label)
            .unwrap_or_else(|| panic!("{label} frame should be present"));
        if let Some(i6) = frame.markers.iter().find(|marker| marker.notation == "I6") {
            assert!(
                !i6.kinds.contains(&AnalysisBatchProofMarkerKind::Forbidden),
                "{label} must not mark I6 using future forbidden-cost evidence: {:?}",
                i6.kinds
            );
        }
    }
}

#[test]
fn analysis_batch_actual_marker_keeps_imminent_hint() {
    let board = board_from_moves(
        Variant::Renju,
        &[
            "H8", "I8", "H6", "G7", "H7", "H9", "J7", "G4", "G5", "I7", "I5", "H5", "I6", "I9",
            "J6", "K6", "J4",
        ],
    );
    let actual = mv("J5");
    let analysis = analysis_for_winner(Color::Black, "renju", 0);
    let candidates = defender_reply_candidates_for_frame(&board, &analysis, Some(actual));
    let mut markers = Vec::new();
    add_loser_candidate_markers(&mut markers, &board, analysis.winner, &candidates);
    add_actual_marker(&mut markers, &board, analysis.winner, actual);

    let marker = proof_marker_for(&markers, "J5");
    assert_eq!(
        marker.kinds,
        vec![
            AnalysisBatchProofMarkerKind::ImminentDefense,
            AnalysisBatchProofMarkerKind::Actual,
        ]
    );
}

#[test]
fn analysis_batch_actual_marker_keeps_far_open_three_defense_hint() {
    let board = board_from_moves(
        Variant::Renju,
        &[
            "H8", "H9", "J8", "I7", "I8", "G8", "I10", "F7", "K8", "L8", "J9", "L7", "J7", "J10",
            "L9", "I6", "H7", "G6", "M10", "N11", "J6", "J5", "K9",
        ],
    );
    let actual = mv("N9");
    let mut markers = Vec::new();
    add_actual_marker(&mut markers, &board, Some(Color::Black), actual);

    let marker = proof_marker_for(&markers, "N9");
    assert_eq!(
        marker.kinds,
        vec![
            AnalysisBatchProofMarkerKind::ImminentDefense,
            AnalysisBatchProofMarkerKind::Actual,
        ]
    );
}

#[test]
fn analysis_batch_visual_frames_mark_both_forbidden_open_three_responses() {
    let replay = replay_from_moves(
        Variant::Renju,
        &[
            "H8", "G7", "H9", "J8", "H7", "H6", "I7", "F8", "E9", "H10", "I5", "G9", "E7", "I9",
            "K7", "G10", "G11", "I11", "J12", "G6", "G8", "F6", "L7", "J7", "J6", "E6", "D6", "I6",
        ],
    );

    let report = run_analysis_batch_replays_with_options(
        "report.json:bot-a vs bot-b".to_string(),
        vec![ReplayAnalysisInput {
            label: "renju_forbidden_open_three_responses".to_string(),
            replay,
        }],
        AnalysisBatchRunOptions {
            analysis: AnalysisOptions {
                max_depth: 4,
                max_scan_plies: Some(64),
            },
            include_proof_details: true,
        },
    );

    let frame = report.entries[0]
        .proof_details
        .as_ref()
        .expect("proof details should be present")
        .proof_frames
        .iter()
        .find(|frame| frame.label == "actual_ply_25")
        .expect("ply 25 decision frame should be present");

    for notation in ["E6", "I6"] {
        let marker = marker_for(frame, notation);
        assert!(
            marker
                .kinds
                .contains(&AnalysisBatchProofMarkerKind::ImminentDefense),
            "{notation} should be marked as an imminent open-three response: {:?}",
            marker.kinds
        );
        assert!(
            marker
                .kinds
                .contains(&AnalysisBatchProofMarkerKind::Forbidden),
            "{notation} should be marked forbidden for Black under Renju: {:?}",
            marker.kinds
        );
    }
}

#[test]
fn analysis_batch_candidate_markers_prioritize_immediate_threats_over_imminent_responses() {
    let board = board_from_moves(
        Variant::Renju,
        &[
            "H8", "I8", "H7", "I7", "H6", "H5", "I6", "I9", "G6", "J6", "G8", "J5", "E6", "F6",
            "H9",
        ],
    );
    let actual = mv("H10");
    let analysis = analysis_for_winner(Color::Black, "renju", 0);
    let candidates = defender_reply_candidates_for_frame(&board, &analysis, Some(actual));
    let mut markers = Vec::new();
    add_loser_candidate_markers(&mut markers, &board, analysis.winner, &candidates);
    add_actual_marker(&mut markers, &board, analysis.winner, actual);

    let h10 = proof_marker_for(&markers, "H10");
    assert!(h10.kinds.contains(&AnalysisBatchProofMarkerKind::Threat));
    assert!(h10.kinds.contains(&AnalysisBatchProofMarkerKind::Actual));
    assert!(
        !markers.iter().any(|marker| marker
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::ImminentDefense)),
        "imminent responses should be suppressed while an immediate threat response exists: {:?}",
        markers
    );
}

#[test]
fn analysis_batch_visual_frames_probe_all_imminent_combo_replies() {
    let board = board_from_moves(
        Variant::Renju,
        &[
            "H8", "H7", "F8", "G9", "G8", "I8", "G6", "D9", "F9", "F10", "D7", "G10", "F7", "E10",
            "E8", "D8", "C6", "B5", "D10", "F11", "F6", "F5", "D6", "E6", "H5", "I4",
        ],
    );
    assert_eq!(board.current_player, Color::Black);

    let analysis = analysis_for_winner(Color::White, "renju", 0);

    let reply_outcomes = defender_reply_outcomes_for_frame(&board, &analysis, Some(mv("C8")));
    for notation in ["J7", "H9", "E12", "G12"] {
        assert!(
            reply_outcomes
                .iter()
                .any(|reply| reply.notation == notation),
            "{notation} should be probed as a non-actual 3+3 reply: {:?}",
            reply_outcomes
        );
    }
    assert!(
        !reply_outcomes.iter().any(|reply| reply.notation == "C8"),
        "the actual replay move is inherited from replay context, not re-probed: {:?}",
        reply_outcomes
    );

    let mut markers = Vec::new();
    add_reply_outcome_markers(&mut markers, &reply_outcomes);
    for notation in ["J7", "H9", "E12"] {
        let marker = proof_marker_for(&markers, notation);
        assert!(
            marker
                .kinds
                .contains(&AnalysisBatchProofMarkerKind::ImminentDefense),
            "{notation} should keep its imminent-response hint box: {:?}",
            marker.kinds
        );
        assert!(
            marker.kinds.iter().any(|kind| matches!(
                kind,
                AnalysisBatchProofMarkerKind::ForcedLoss
                    | AnalysisBatchProofMarkerKind::ConfirmedEscape
                    | AnalysisBatchProofMarkerKind::PossibleEscape
                    | AnalysisBatchProofMarkerKind::ImmediateLoss
                    | AnalysisBatchProofMarkerKind::UnknownOutcome
            )),
            "{notation} should carry a proof outcome marker: {:?}",
            marker.kinds
        );
    }
    assert!(
        markers.iter().all(|marker| marker.notation != "C8"),
        "actual replay move should not be re-probed: {markers:?}"
    );
}

#[test]
fn analysis_batch_visual_frames_do_not_show_lower_tier_forbidden_replies_during_immediate_threats()
{
    let board = board_from_moves(
        Variant::Renju,
        &[
            "H8", "I8", "H10", "G9", "H9", "H7", "J9", "G12", "G10", "I10", "H11", "H12", "I12",
            "F9", "I6", "E10", "J11", "C12", "D11", "D9", "H13", "E9", "C9", "F11", "C8", "K10",
            "C7", "C10", "J10", "J8", "I9", "K11", "K9", "L9", "L8", "M7", "F6", "G7", "H6", "G6",
            "F7", "E12", "C5", "C6", "J12", "J13", "F15", "G14", "E8", "F12", "D12", "F10",
        ],
    );
    assert_eq!(board.current_player, Color::Black);
    let analysis = analysis_for_winner(Color::White, "renju", 0);
    let reply_outcomes = defender_reply_outcomes_for_frame(&board, &analysis, Some(mv("F8")));
    assert!(
        reply_outcomes.iter().any(|reply| reply.notation == "F13"
            && reply.roles.contains(&DefenderReplyRole::ImmediateDefense)),
        "the active proof candidate should be the immediate-threat response: {reply_outcomes:?}"
    );
    assert!(
        reply_outcomes.iter().all(|reply| reply.notation != "D8"),
        "lower-tier forbidden imminent replies should not be probed: {reply_outcomes:?}"
    );

    let reply_candidates = defender_reply_candidates_for_frame(&board, &analysis, Some(mv("F8")));
    let mut markers = Vec::new();
    add_loser_candidate_markers(&mut markers, &board, analysis.winner, &reply_candidates);
    add_reply_outcome_markers(&mut markers, &reply_outcomes);

    assert!(
        markers.iter().all(|marker| marker.notation != "D8"),
        "lower-tier forbidden imminent replies should not be marked while immediate threats are active: {markers:?}"
    );
    let f13 = proof_marker_for(&markers, "F13");
    assert!(f13.kinds.contains(&AnalysisBatchProofMarkerKind::Threat));
    assert!(f13
        .kinds
        .contains(&AnalysisBatchProofMarkerKind::ImmediateLoss));
}

#[test]
fn analysis_batch_visual_frames_mark_renju_forbidden_blocks() {
    let replay = replay_from_moves(
        Variant::Renju,
        &[
            "H8", "H7", "J8", "I9", "I8", "G8", "F9", "F7", "H9", "G7", "I7", "E7", "D7", "G9",
            "G6", "G11", "K8", "G10",
        ],
    );

    let report = run_analysis_batch_replays_with_options(
        "report.json:bot-a vs bot-b".to_string(),
        vec![ReplayAnalysisInput {
            label: "renju_forbidden_block".to_string(),
            replay,
        }],
        AnalysisBatchRunOptions {
            analysis: AnalysisOptions {
                max_depth: 4,
                max_scan_plies: Some(8),
            },
            include_proof_details: true,
        },
    );

    let frames = &report.entries[0]
        .proof_details
        .as_ref()
        .expect("proof details should be present")
        .proof_frames;
    let turn_16_17 = frames
        .iter()
        .find(|frame| frame.label == "actual_ply_17")
        .expect("ply 17 decision frame should be present");
    let g10 = marker_for(turn_16_17, "G10");
    assert!(g10.kinds.contains(&AnalysisBatchProofMarkerKind::Threat));
    assert!(g10.kinds.contains(&AnalysisBatchProofMarkerKind::Forbidden));

    let turn_14_15 = frames
        .iter()
        .find(|frame| frame.label == "actual_ply_15")
        .expect("ply 15 decision frame should be present");
    let future_g10 = marker_for(turn_14_15, "G10");
    assert!(future_g10
        .kinds
        .contains(&AnalysisBatchProofMarkerKind::ImminentDefense));
    assert!(future_g10
        .kinds
        .contains(&AnalysisBatchProofMarkerKind::Forbidden));
}

fn marker_for<'a>(
    frame: &'a AnalysisBatchProofFrame,
    notation: &str,
) -> &'a AnalysisBatchProofMarker {
    frame
        .markers
        .iter()
        .find(|marker| marker.notation == notation)
        .unwrap_or_else(|| panic!("expected marker {notation}"))
}

fn proof_marker_for<'a>(
    markers: &'a [AnalysisBatchProofMarker],
    notation: &str,
) -> &'a AnalysisBatchProofMarker {
    markers
        .iter()
        .find(|marker| marker.notation == notation)
        .unwrap_or_else(|| panic!("expected marker {notation}"))
}
