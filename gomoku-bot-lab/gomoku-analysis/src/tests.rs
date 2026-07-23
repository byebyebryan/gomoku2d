use gomoku_bot::tactical::{
    has_forcing_local_threat, legal_forcing_continuations_for_fact, lethal_threat,
    local_threat_facts_for_player as local_threat_facts, LethalThreatKind, LocalThreatFact,
    LocalThreatKind, LocalThreatOrigin,
};
use gomoku_core::{Board, Color, Move, Replay, RuleConfig, Variant};

use super::{
    analyze_alternate_defender_reply_options, analyze_defender_reply_options, analyze_replay,
    corridor_analysis_model, defender_reply_candidates, failure_analysis,
    replay_frame_annotations_for_analysis, replay_frame_annotations_from_proof,
    replay_prefix_boards, AnalysisOptions, DefenderReplyOutcome, DefenderReplyRole,
    FailureAnalysisInput, FailureMode, ForcedInterval, LethalOnset, LethalOnsetComponentTier,
    LethalOnsetMechanism, LethalOnsetShape, MissedCandidateOutcome, ProofLimitCause, ProofResult,
    ProofStatus, ReplayAnalysisSession, ReplayFrameAnnotations, ReplayFrameHighlightRole,
    ReplayFrameMarkerRole, ReplyClassification, RootCause, TacticalNote, ThreatSequenceEvidence,
    UnclearReason,
};

fn mv(notation: &str) -> Move {
    Move::from_notation(notation).expect("test move notation should parse")
}

fn replay_from_moves(variant: Variant, moves: &[&str]) -> Replay {
    let rules = RuleConfig {
        variant,
        ..RuleConfig::default()
    };
    let mut board = Board::new(rules.clone());
    let mut replay = Replay::new(rules, "Black", "White");

    for notation in moves {
        let parsed = mv(notation);
        board.apply_move(parsed).expect("fixture move should apply");
        replay.push_move(parsed, 0, board.hash(), None);
    }
    replay.finish(&board.result, Some(0));
    replay
}

fn board_from_moves(variant: Variant, moves: &[&str]) -> Board {
    let mut board = Board::new(RuleConfig {
        variant,
        ..RuleConfig::default()
    });
    for notation in moves {
        board
            .apply_move(mv(notation))
            .expect("fixture move should apply");
    }
    board
}

fn proof_for_board(board: &Board, winner: Color, options: &AnalysisOptions) -> ProofResult {
    ProofResult {
        status: ProofStatus::ForcedWin,
        attacker: winner,
        side_to_move: board.current_player,
        model: corridor_analysis_model(board, options),
        principal_line: Vec::new(),
        escape_moves: Vec::new(),
        threat_evidence: Vec::new(),
        limit_hit: false,
        limit_causes: Vec::new(),
    }
}

fn test_analysis_options() -> AnalysisOptions {
    AnalysisOptions {
        max_depth: 4,
        max_scan_plies: Some(64),
    }
}

fn proof_summary_with_escape(
    boards: &[Board],
    scan_start: usize,
    attacker: Color,
    prefix_ply: usize,
    escape_moves: Vec<Move>,
    reply_classification: ReplyClassification,
    limit_hit: bool,
) -> Vec<ProofResult> {
    let options = test_analysis_options();
    let mut proofs = (scan_start..boards.len())
        .map(|ply| proof_for_board(&boards[ply], attacker, &options))
        .collect::<Vec<_>>();
    proofs[prefix_ply - scan_start] = ProofResult {
        status: ProofStatus::EscapeFound,
        attacker,
        side_to_move: boards[prefix_ply].current_player,
        model: corridor_analysis_model(&boards[prefix_ply], &options),
        principal_line: Vec::new(),
        escape_moves: escape_moves.clone(),
        threat_evidence: vec![ThreatSequenceEvidence {
            prefix_ply: Some(prefix_ply),
            attacker,
            defender: attacker.opponent(),
            winning_squares: Vec::new(),
            raw_cost_squares: Vec::new(),
            legal_cost_squares: Vec::new(),
            illegal_cost_squares: Vec::new(),
            defender_immediate_wins: Vec::new(),
            actual_reply: None,
            reply_classification,
            escape_replies: escape_moves,
            forced_replies: Vec::new(),
            next_forcing_move: None,
            proof_status: ProofStatus::EscapeFound,
            limit_hit,
            limit_causes: if limit_hit {
                vec![ProofLimitCause::DepthCutoff]
            } else {
                Vec::new()
            },
        }],
        limit_hit,
        limit_causes: if limit_hit {
            vec![ProofLimitCause::DepthCutoff]
        } else {
            Vec::new()
        },
    };
    proofs
}

fn test_lethal_onset(prefix_ply: usize, attacker: Color) -> LethalOnset {
    LethalOnset {
        prefix_ply,
        attacker,
        defender: attacker.opponent(),
        kind: LethalThreatKind::OneStepCoverage,
        shape: LethalOnsetShape {
            label: "3".to_string(),
            components: Vec::new(),
            mechanisms: vec![LethalOnsetMechanism::MultiRoute],
        },
        terminal_targets: Vec::new(),
        covering_replies: Vec::new(),
        one_step_replies: Vec::new(),
    }
}

#[test]
fn defender_reply_candidates_distinguish_defenses_from_counterplay() {
    let board = board_from_moves(
        Variant::Renju,
        &[
            "H8", "I8", "H7", "I7", "H6", "H5", "I6", "I9", "G6", "J6", "G8", "J5", "G5",
        ],
    );
    let candidates = defender_reply_candidates(&board, Color::Black, Some(mv("G7")));

    for notation in ["G4", "G7", "G9"] {
        assert!(
            candidates.iter().any(|candidate| {
                candidate.notation == notation
                    && candidate
                        .roles
                        .contains(&DefenderReplyRole::ImminentDefense)
            }),
            "{notation} should be visible as an imminent-defense candidate: {candidates:?}"
        );
    }

    for notation in ["I10", "I11"] {
        assert!(
            candidates.iter().any(|candidate| {
                candidate.notation == notation
                    && candidate
                        .roles
                        .contains(&DefenderReplyRole::OffensiveCounter)
            }),
            "{notation} should be visible as an offensive-counter candidate: {candidates:?}"
        );
    }
}

#[test]
fn imminent_open_three_defense_excludes_outer_cost_squares() {
    let board = board_from_moves(
        Variant::Renju,
        &[
            "H8", "I8", "H7", "I7", "H6", "H5", "I6", "I9", "G6", "J6", "G8",
        ],
    );
    let options = AnalysisOptions {
        max_depth: 4,
        max_scan_plies: Some(8),
    };
    let replies = analyze_defender_reply_options(&board, Color::Black, Some(mv("J5")), &options);

    for notation in ["J5", "F9"] {
        let reply = reply_for(&replies, notation);
        assert!(
            reply.roles.contains(&DefenderReplyRole::ImminentDefense),
            "{notation}: roles {:?}",
            reply.roles
        );
    }
    for notation in ["E10", "K4"] {
        assert!(
            replies.iter().all(|reply| reply.notation != notation),
            "{notation} should not be a direct defense to the open three"
        );
    }
}

#[test]
fn open_three_with_blocked_outer_side_includes_far_defense_square() {
    let board = board_from_moves(Variant::Renju, &["J9", "H9", "K9", "A1", "L9"]);
    assert!(
        local_threat_facts(&board, Color::Black).contains(&LocalThreatFact {
            player: Color::Black,
            kind: LocalThreatKind::OpenThree,
            origin: LocalThreatOrigin::Existing(mv("J9")),
            defense_squares: vec![mv("I9"), mv("M9"), mv("N9")],
            rest_squares: vec![],
        })
    );

    let options = AnalysisOptions {
        max_depth: 4,
        max_scan_plies: Some(8),
    };
    let replies = analyze_defender_reply_options(&board, Color::Black, Some(mv("N9")), &options);
    let reply = reply_for(&replies, "N9");
    assert!(reply.roles.contains(&DefenderReplyRole::ImminentDefense));
}

#[test]
fn boxed_three_is_not_a_forcing_open_three() {
    let board = board_from_moves(Variant::Renju, &["J9", "H9", "K9", "N9", "L9"]);
    assert!(
        local_threat_facts(&board, Color::Black)
            .iter()
            .all(|fact| fact.kind != LocalThreatKind::OpenThree),
        "{:?}",
        local_threat_facts(&board, Color::Black)
    );

    let options = AnalysisOptions {
        max_depth: 4,
        max_scan_plies: Some(8),
    };
    let replies = analyze_defender_reply_options(&board, Color::Black, None, &options);
    for notation in ["I9", "M9"] {
        assert!(
            replies
                .iter()
                .filter(|reply| reply.notation == notation)
                .all(|reply| !reply.roles.contains(&DefenderReplyRole::ImminentDefense)),
            "{notation}: {:?}",
            replies
        );
    }
}

#[test]
fn renju_forbidden_only_black_local_threat_is_not_forcing() {
    let board = board_from_moves(
        Variant::Renju,
        &["H8", "G8", "I8", "A1", "J8", "A2", "L8", "A3", "M8"],
    );
    assert!(!board.is_legal_for_color(mv("K8"), Color::Black));
    assert!(!board
        .immediate_winning_moves_for(Color::Black)
        .contains(&mv("K8")));

    let facts = local_threat_facts(&board, Color::Black);
    let forbidden_gap_four = facts
        .iter()
        .find(|fact| {
            fact.kind == LocalThreatKind::BrokenFour && fact.defense_squares == vec![mv("K8")]
        })
        .unwrap_or_else(|| panic!("expected raw forbidden broken-four fact: {facts:?}"));
    assert!(
        legal_forcing_continuations_for_fact(&board, Color::Black, forbidden_gap_four).is_empty()
    );
    assert!(
        !has_forcing_local_threat(&board, Color::Black),
        "unexpected forcing fact remains: {facts:?}"
    );
}

#[test]
fn renju_mixed_black_local_threat_uses_only_legal_continuations() {
    let board = board_from_moves(
        Variant::Renju,
        &["H8", "A1", "I8", "A2", "J8", "A3", "K8", "A4", "M8"],
    );
    assert!(board.is_legal_for_color(mv("G8"), Color::Black));
    assert!(!board.is_legal_for_color(mv("L8"), Color::Black));

    let facts = local_threat_facts(&board, Color::Black);
    let mixed_open_four = facts
        .iter()
        .find(|fact| {
            fact.kind == LocalThreatKind::OpenFour
                && fact.defense_squares == vec![mv("G8"), mv("L8")]
        })
        .unwrap_or_else(|| panic!("expected raw mixed open-four fact: {facts:?}"));
    let continuations = legal_forcing_continuations_for_fact(&board, Color::Black, mixed_open_four);
    assert_eq!(
        continuations
            .iter()
            .map(|continuation| continuation.mv)
            .collect::<Vec<_>>(),
        vec![mv("G8")]
    );
    assert!(has_forcing_local_threat(&board, Color::Black));
}

fn reply_for<'a>(
    replies: &'a [super::DefenderReplyAnalysis],
    notation: &str,
) -> &'a super::DefenderReplyAnalysis {
    replies
        .iter()
        .find(|reply| reply.notation == notation)
        .unwrap_or_else(|| panic!("expected reply {notation}"))
}

#[test]
fn corridor_replies_finds_escape_for_single_closed_four() {
    let board = board_from_moves(
        Variant::Freestyle,
        &["H8", "G8", "I8", "A1", "J8", "A2", "K8"],
    );
    let replies = analyze_defender_reply_options(
        &board,
        Color::Black,
        None,
        &AnalysisOptions {
            max_depth: 4,
            ..AnalysisOptions::default()
        },
    );

    assert_eq!(
        reply_for(&replies, "L8").outcome,
        DefenderReplyOutcome::ConfirmedEscape
    );
}

#[test]
fn corridor_replies_proves_open_four_even_if_one_end_is_blocked() {
    let board = board_from_moves(
        Variant::Freestyle,
        &["H8", "A1", "I8", "A2", "J8", "A3", "K8"],
    );
    let replies = analyze_defender_reply_options(
        &board,
        Color::Black,
        None,
        &AnalysisOptions {
            max_depth: 4,
            ..AnalysisOptions::default()
        },
    );

    assert_eq!(
        reply_for(&replies, "G8").outcome,
        DefenderReplyOutcome::ImmediateLoss
    );
    assert_eq!(
        reply_for(&replies, "L8").outcome,
        DefenderReplyOutcome::ImmediateLoss
    );
}

#[test]
fn corridor_depth_proves_closed_four_block_into_open_four() {
    let replay = replay_from_moves(
        Variant::Freestyle,
        &[
            "H8", "G8", "I8", "A1", "J8", "C1", "K6", "E1", "K7", "G1", "K8", "L8", "K9", "K5",
            "K10",
        ],
    );

    let analysis = analyze_replay(
        &replay,
        AnalysisOptions {
            max_depth: 4,
            max_scan_plies: Some(6),
        },
    )
    .expect("forced chain replay should analyze");

    assert_eq!(analysis.final_forced_interval.start_ply, 10);
    assert_eq!(analysis.root_cause, RootCause::MissedDefense);
    assert!(analysis
        .proof_summary
        .iter()
        .flat_map(|proof| proof.threat_evidence.iter())
        .any(|evidence| evidence.reply_classification == ReplyClassification::BlockedButForced));
}

#[test]
fn corridor_depth_cutoff_is_possible_escape() {
    let board = board_from_moves(
        Variant::Freestyle,
        &[
            "H8", "G8", "I8", "A1", "J8", "C1", "K6", "E1", "K7", "G1", "K8",
        ],
    );
    let replies = analyze_defender_reply_options(
        &board,
        Color::Black,
        Some(mv("L8")),
        &AnalysisOptions {
            max_depth: 0,
            ..AnalysisOptions::default()
        },
    );

    let reply = reply_for(&replies, "L8");
    assert_eq!(reply.outcome, DefenderReplyOutcome::PossibleEscape);
    assert!(reply.limit_causes.contains(&ProofLimitCause::DepthCutoff));
}

#[test]
fn corridor_replies_allow_defender_immediate_win_escape() {
    let board = board_from_moves(
        Variant::Freestyle,
        &["A1", "H8", "A2", "I8", "A3", "J8", "A4", "K8"],
    );
    let replies = analyze_defender_reply_options(
        &board,
        Color::White,
        Some(mv("A5")),
        &AnalysisOptions {
            max_depth: 4,
            ..AnalysisOptions::default()
        },
    );

    assert_eq!(
        reply_for(&replies, "A5").outcome,
        DefenderReplyOutcome::ConfirmedEscape
    );
}

#[test]
fn corridor_replies_prove_renju_single_square_with_forbidden_block() {
    let replay = replay_from_moves(
        Variant::Renju,
        &[
            "C3", "D4", "H6", "E5", "H7", "F6", "F8", "G7", "G8", "A15", "A14", "H8",
        ],
    );

    let analysis = analyze_replay(
        &replay,
        AnalysisOptions {
            max_depth: 4,
            max_scan_plies: Some(2),
        },
    )
    .expect("renju fixture should analyze");

    assert!(analysis
        .proof_summary
        .iter()
        .flat_map(|proof| proof.threat_evidence.iter())
        .any(|evidence| {
            evidence.reply_classification == ReplyClassification::NoLegalBlock
                && evidence.raw_cost_squares == vec![mv("H8")]
                && evidence.legal_cost_squares.is_empty()
                && evidence.illegal_cost_squares == vec![mv("H8")]
        }));
}

#[test]
fn replay_analysis_labels_missed_defense_without_overclaiming_previous_position() {
    let replay = replay_from_moves(
        Variant::Freestyle,
        &["H8", "G8", "I8", "A1", "J8", "A2", "K8", "B1", "L8"],
    );

    let analysis = analyze_replay(
        &replay,
        AnalysisOptions {
            max_depth: 2,
            max_scan_plies: None,
        },
    )
    .expect("finished replay should analyze");

    assert_eq!(analysis.winner, Some(Color::Black));
    assert_eq!(analysis.final_forced_interval.start_ply, 8);
    assert_eq!(analysis.last_chance_ply, Some(7));
    assert_eq!(analysis.critical_loser_ply, Some(8));
    assert_eq!(analysis.root_cause, RootCause::MissedDefense);
    assert!(analysis.tactical_notes.is_empty());
}

#[test]
fn replay_analysis_records_terminal_lethal_onset() {
    let replay = replay_from_moves(
        Variant::Freestyle,
        &["H8", "A1", "I8", "A2", "J8", "A3", "K8", "A4", "G8"],
    );

    let analysis = analyze_replay(&replay, AnalysisOptions::default())
        .expect("finished replay should analyze");

    let onset = analysis
        .lethal_onset
        .as_ref()
        .expect("open four should be a lethal onset");
    assert_eq!(onset.prefix_ply, 7);
    assert_eq!(onset.attacker, Color::Black);
    assert_eq!(onset.defender, Color::White);
    assert_eq!(onset.kind, LethalThreatKind::TerminalCoverage);
    assert_eq!(onset.shape.label, "4");
    assert_eq!(
        onset
            .shape
            .components
            .iter()
            .map(|component| component.tier)
            .collect::<Vec<_>>(),
        vec![LethalOnsetComponentTier::Four]
    );
    assert_eq!(
        onset.shape.mechanisms,
        vec![LethalOnsetMechanism::MultiRoute]
    );
    assert_eq!(onset.terminal_targets, vec![mv("G8"), mv("L8")]);
    assert!(onset.covering_replies.is_empty());
    assert!(onset.one_step_replies.is_empty());
    assert_eq!(
        analysis
            .setup_corridor
            .as_ref()
            .map(|interval| interval.end_ply),
        Some(onset.prefix_ply)
    );
}

#[test]
fn lethal_onset_shape_collapses_diagonal_open_four() {
    let board = board_from_moves(
        Variant::Freestyle,
        &[
            "F9", "E8", "I8", "G8", "I10", "I7", "L11", "H8", "L5", "G9", "K3", "F10",
        ],
    );
    assert_eq!(board.current_player, Color::Black);

    let threat = lethal_threat(&board, Color::White).expect("diagonal open four should be lethal");
    let onset = super::lethal_onset_from_threat(12, &board, threat);

    assert_eq!(onset.kind, LethalThreatKind::TerminalCoverage);
    assert_eq!(onset.shape.label, "4");
    assert_eq!(
        onset
            .shape
            .components
            .iter()
            .map(|component| component.tier)
            .collect::<Vec<_>>(),
        vec![LethalOnsetComponentTier::Four]
    );
    assert_eq!(
        onset.shape.mechanisms,
        vec![LethalOnsetMechanism::MultiRoute]
    );
    assert_eq!(onset.terminal_targets, vec![mv("J6"), mv("E11")]);
}

#[test]
fn lethal_onset_shape_includes_active_threes_for_terminal_coverage() {
    let board = board_from_moves(
        Variant::Freestyle,
        &[
            "C3", "A15", "D3", "O15", "E3", "A14", "H8", "O14", "I8", "A13", "J8", "O13", "K8",
        ],
    );
    assert_eq!(board.current_player, Color::White);

    let threat = lethal_threat(&board, Color::Black)
        .expect("open four should be terminal lethal even with an active three");
    let onset = super::lethal_onset_from_threat(13, &board, threat);

    assert_eq!(onset.kind, LethalThreatKind::TerminalCoverage);
    assert_eq!(onset.shape.label, "4x3");
    assert_eq!(
        onset
            .shape
            .components
            .iter()
            .map(|component| component.tier)
            .collect::<Vec<_>>(),
        vec![
            LethalOnsetComponentTier::Four,
            LethalOnsetComponentTier::Three
        ]
    );
    assert_eq!(onset.terminal_targets, vec![mv("G8"), mv("L8")]);
}

#[test]
fn lethal_onset_shape_includes_active_threes_for_terminal_four_by_four() {
    let board = board_from_moves(
        Variant::Freestyle,
        &[
            "C3", "A15", "D3", "O15", "E3", "A14", "H8", "O14", "I8", "B15", "J8", "N15", "K8",
            "B14", "H10", "N14", "I10", "C15", "J10", "M15", "K10",
        ],
    );
    assert_eq!(board.current_player, Color::White);

    let threat =
        lethal_threat(&board, Color::Black).expect("parallel open fours should be terminal lethal");
    let onset = super::lethal_onset_from_threat(21, &board, threat);

    assert_eq!(onset.kind, LethalThreatKind::TerminalCoverage);
    assert_eq!(onset.shape.label, "4x4x3");
    assert_eq!(
        onset
            .shape
            .components
            .iter()
            .map(|component| component.tier)
            .collect::<Vec<_>>(),
        vec![
            LethalOnsetComponentTier::Four,
            LethalOnsetComponentTier::Four,
            LethalOnsetComponentTier::Three
        ]
    );
    assert_eq!(
        onset.terminal_targets,
        vec![mv("G8"), mv("L8"), mv("G10"), mv("L10")]
    );
}

#[test]
fn replay_analysis_records_one_step_lethal_onset_before_terminal_coverage() {
    let replay = replay_from_moves(
        Variant::Freestyle,
        &[
            "H8", "G8", "I8", "A1", "J8", "O1", "K8", "A15", "I7", "O15", "I9", "L8", "I6", "A14",
            "I10",
        ],
    );

    let analysis = analyze_replay(&replay, AnalysisOptions::default())
        .expect("finished replay should analyze");

    let onset = analysis
        .lethal_onset
        .as_ref()
        .expect("4+3 should be a one-step lethal onset");
    assert_eq!(onset.prefix_ply, 11);
    assert_eq!(onset.kind, LethalThreatKind::OneStepCoverage);
    assert_eq!(onset.shape.label, "4x3");
    assert_eq!(
        onset
            .shape
            .components
            .iter()
            .map(|component| component.tier)
            .collect::<Vec<_>>(),
        vec![
            LethalOnsetComponentTier::Four,
            LethalOnsetComponentTier::Three
        ]
    );
    assert_eq!(
        onset.shape.mechanisms,
        vec![LethalOnsetMechanism::MultiRoute]
    );
    assert_eq!(onset.terminal_targets, vec![mv("L8")]);
    assert_eq!(onset.one_step_replies.len(), 1);
    assert_eq!(onset.one_step_replies[0].reply, mv("L8"));
    assert_eq!(
        analysis
            .setup_corridor
            .as_ref()
            .map(|interval| interval.end_ply),
        Some(onset.prefix_ply)
    );
    assert_eq!(
        onset.one_step_replies[0]
            .lethal_entries
            .iter()
            .map(|entry| (entry.mv, entry.terminal_targets.clone()))
            .collect::<Vec<_>>(),
        vec![
            (mv("I6"), vec![mv("I5"), mv("I10")]),
            (mv("I10"), vec![mv("I6"), mv("I11")]),
        ]
    );
}

#[test]
fn replay_annotations_upgrade_all_one_step_lethal_components_to_immediate_threats() {
    let replay = replay_from_moves(
        Variant::Freestyle,
        &[
            "H8", "G8", "I8", "A1", "J8", "O1", "K8", "A15", "I7", "O15", "I9", "L8", "I6", "A14",
            "I10",
        ],
    );
    let analysis = analyze_replay(&replay, AnalysisOptions::default())
        .expect("finished replay should analyze");
    let onset = analysis
        .lethal_onset
        .as_ref()
        .expect("fixture should have lethal onset");
    assert_eq!(onset.prefix_ply, 11);
    assert_eq!(onset.shape.label, "4x3");

    let annotations = replay_frame_annotations_for_analysis(&replay, &analysis)
        .expect("annotations should build");
    let frame = annotations
        .iter()
        .find(|frame| frame.ply == onset.prefix_ply)
        .expect("onset frame should be annotated");
    let immediate_threats = frame
        .highlights
        .iter()
        .filter(|highlight| highlight.role == ReplayFrameHighlightRole::ImmediateThreat)
        .map(|highlight| highlight.mv)
        .collect::<Vec<_>>();
    let immediate_losses = frame
        .markers
        .iter()
        .filter(|marker| marker.role == ReplayFrameMarkerRole::ImmediateLoss)
        .map(|marker| marker.mv)
        .collect::<Vec<_>>();

    assert!(
        immediate_threats.contains(&mv("L8")),
        "four target should be highlighted as a lethal contributor: {:?}",
        frame.highlights
    );
    assert!(
        immediate_threats.contains(&mv("I6")) && immediate_threats.contains(&mv("I10")),
        "three response squares should be promoted to immediate-threat visuals at onset: {:?}",
        frame.highlights
    );
    assert!(
        frame.evidence.iter().any(|highlight| {
            highlight.role == ReplayFrameHighlightRole::ImmediateThreat
                && highlight.mv == mv("H8")
                && highlight.side == Color::Black
        }),
        "four source stones should be emitted as strong-red onset evidence: {:?}",
        frame.evidence
    );
    assert!(
        frame.evidence.iter().any(|highlight| {
            highlight.role == ReplayFrameHighlightRole::ImmediateThreat
                && highlight.mv == mv("I7")
                && highlight.side == Color::Black
        }) && frame.evidence.iter().any(|highlight| {
            highlight.role == ReplayFrameHighlightRole::ImmediateThreat
                && highlight.mv == mv("I9")
                && highlight.side == Color::Black
        }),
        "three source stones should be emitted as strong-red onset evidence: {:?}",
        frame.evidence
    );
    assert!(
        !immediate_losses.contains(&mv("L8"))
            && immediate_losses.contains(&mv("I6"))
            && immediate_losses.contains(&mv("I10")),
        "onset threat markers should skip the actual replay move: {:?}",
        frame.markers
    );
}

#[test]
fn lethal_onset_annotations_include_active_threes_for_terminal_coverage() {
    let board = board_from_moves(
        Variant::Freestyle,
        &[
            "C3", "A15", "D3", "O15", "E3", "A14", "H8", "O14", "I8", "A13", "J8", "O13", "K8",
        ],
    );
    let threat = lethal_threat(&board, Color::Black).expect("open four should be terminal lethal");
    let onset = super::lethal_onset_from_threat(13, &board, threat);
    let mut frame = ReplayFrameAnnotations {
        ply: onset.prefix_ply,
        side_to_move: board.current_player,
        evidence: Vec::new(),
        highlights: Vec::new(),
        markers: Vec::new(),
    };

    super::push_lethal_onset_annotations(&mut frame, &board, Some(&onset), None);

    let immediate_threats = frame
        .highlights
        .iter()
        .filter(|highlight| highlight.role == ReplayFrameHighlightRole::ImmediateThreat)
        .map(|highlight| highlight.mv)
        .collect::<Vec<_>>();
    let immediate_losses = frame
        .markers
        .iter()
        .filter(|marker| marker.role == ReplayFrameMarkerRole::ImmediateLoss)
        .map(|marker| marker.mv)
        .collect::<Vec<_>>();

    assert_eq!(
        immediate_threats,
        vec![mv("G8"), mv("L8"), mv("B3"), mv("F3")]
    );
    assert_eq!(
        immediate_losses,
        vec![mv("G8"), mv("L8"), mv("B3"), mv("F3")]
    );
    assert!(
        frame.evidence.iter().any(|highlight| {
            highlight.role == ReplayFrameHighlightRole::ImmediateThreat
                && highlight.mv == mv("C3")
                && highlight.side == Color::Black
        }),
        "terminal onset should emit active-three evidence as upgraded threat context: {:?}",
        frame.evidence
    );
}

#[test]
fn lethal_onset_shape_marks_single_renju_forbidden_cover() {
    let board = board_from_moves(
        Variant::Renju,
        &[
            "H8", "H7", "J8", "I9", "I8", "G8", "F9", "F7", "H9", "G7", "I7", "E7", "D7", "G9",
            "G6", "G11",
        ],
    );
    assert_eq!(board.current_player, Color::Black);
    assert!(!board.is_legal_for_color(mv("G10"), Color::Black));

    let threat = lethal_threat(&board, Color::White)
        .expect("forbidden direct block should make the four lethal");
    let onset = super::lethal_onset_from_threat(16, &board, threat);

    assert_eq!(onset.shape.label, "4");
    assert_eq!(
        onset
            .shape
            .components
            .iter()
            .map(|component| component.tier)
            .collect::<Vec<_>>(),
        vec![LethalOnsetComponentTier::Four]
    );
    assert_eq!(
        onset.shape.mechanisms,
        vec![LethalOnsetMechanism::ForbiddenCover]
    );
    assert_eq!(onset.terminal_targets, vec![mv("G10")]);
}

#[test]
fn replay_analysis_stops_at_possible_escape_boundary() {
    let replay = replay_from_moves(
        Variant::Renju,
        &[
            "H8", "I8", "H7", "I7", "H6", "H5", "I6", "I9", "G6", "J6", "G8", "J5", "G5", "G7",
            "E6", "F6", "H9", "H10", "F7", "D5", "I10",
        ],
    );

    let analysis = analyze_replay(
        &replay,
        AnalysisOptions {
            max_depth: 0,
            max_scan_plies: Some(8),
        },
    )
    .expect("finished replay should analyze");

    assert_eq!(analysis.winner, Some(Color::Black));
    assert_eq!(analysis.root_cause, RootCause::MissedDefense);
    assert_eq!(analysis.final_forced_interval.start_ply, 14);
    assert_eq!(analysis.last_chance_ply, Some(13));
    assert_eq!(analysis.critical_loser_ply, Some(14));
    let failure = analysis
        .failure
        .as_ref()
        .expect("bounded escape should produce failure detail");
    assert_eq!(failure.mode, FailureMode::MissedEscape);
    assert_eq!(failure.prefix_ply, Some(13));
    assert_eq!(failure.actual_notation.as_deref(), Some("G7"));
    assert!(failure
        .missed_candidates
        .iter()
        .any(|candidate| candidate.notation == "I10"));

    let scan_start = replay.moves.len() + 1 - analysis.proof_summary.len();
    assert_eq!(scan_start, 13);
    let boundary = analysis
        .proof_summary
        .get(13 - scan_start)
        .expect("escape boundary proof should be within the scan cap");
    assert_eq!(boundary.status, ProofStatus::EscapeFound);
    assert!(boundary.threat_evidence.iter().any(|evidence| {
        evidence.reply_classification == ReplyClassification::PossibleEscape
            && evidence.escape_replies.contains(&mv("I10"))
    }));
}

#[test]
fn replay_analysis_session_matches_blocking_analysis() {
    let replay = replay_from_moves(
        Variant::Freestyle,
        &["H8", "G8", "I8", "A1", "J8", "A2", "K8", "B1", "L8"],
    );
    let options = AnalysisOptions {
        max_depth: 2,
        max_scan_plies: Some(8),
    };
    let expected = analyze_replay(&replay, options.clone()).expect("blocking analysis should run");
    let mut session =
        ReplayAnalysisSession::new(replay, options).expect("session should initialize");
    let mut observed_plys = Vec::new();
    let mut final_analysis = None;

    while final_analysis.is_none() {
        let step = session.step(1);
        observed_plys.extend(step.annotations.iter().map(|frame| frame.ply));
        if step.done {
            final_analysis = step.analysis;
        }
    }

    assert_eq!(final_analysis, Some(expected));
    assert_eq!(&observed_plys[..3], [9, 8, 7]);
    assert_eq!(&observed_plys[3..], [8, 7]);
}

#[test]
fn replay_analysis_session_clamps_zero_work_to_one_prefix() {
    let replay = replay_from_moves(
        Variant::Freestyle,
        &["H8", "G8", "I8", "A1", "J8", "A2", "K8", "B1", "L8"],
    );
    let mut session = ReplayAnalysisSession::new(
        replay,
        AnalysisOptions {
            max_depth: 2,
            max_scan_plies: None,
        },
    )
    .expect("session should initialize");

    let step = session.step(0);

    assert!(!step.done);
    assert_eq!(step.counters.prefixes_analyzed, 1);
    assert_eq!(
        step.annotations
            .iter()
            .map(|frame| frame.ply)
            .collect::<Vec<_>>(),
        vec![9]
    );
}

#[test]
fn replay_frame_annotations_emit_escape_boundary_candidates() {
    let board = board_from_moves(
        Variant::Renju,
        &[
            "H8", "I8", "H7", "I7", "H6", "H5", "I6", "I9", "G6", "J6", "G8", "J5", "G5",
        ],
    );
    let options = AnalysisOptions {
        max_depth: 0,
        max_scan_plies: Some(8),
    };
    let proof = proof_for_board(&board, Color::Black, &options);
    let boundary = replay_frame_annotations_from_proof(
        13,
        &board,
        Color::Black,
        &proof,
        None,
        Some(mv("G7")),
        &options,
    );

    assert_eq!(boundary.side_to_move, Color::White);
    assert!(boundary.markers.iter().any(|marker| {
        marker.role == ReplayFrameMarkerRole::PossibleEscape
            && marker.mv == mv("I10")
            && marker.side == Color::White
    }));

    for notation in ["G4", "G7", "G9"] {
        let mv = mv(notation);
        assert!(
            boundary.highlights.iter().any(|highlight| {
                highlight.role == ReplayFrameHighlightRole::ImminentThreat
                    && highlight.mv == mv
                    && highlight.side == Color::Black
            }),
            "{notation} should be highlighted as a current imminent-threat response: {:?}",
            boundary.highlights
        );
    }

    for notation in ["I10", "I11"] {
        let mv = mv(notation);
        assert!(
            boundary.highlights.iter().any(|highlight| {
                highlight.role == ReplayFrameHighlightRole::CounterThreat
                    && highlight.mv == mv
                    && highlight.side == Color::White
            }),
            "{notation} should be highlighted as a current counter-threat response: {:?}",
            boundary.highlights
        );
    }

    assert!(
        boundary.evidence.iter().any(|highlight| {
            highlight.role == ReplayFrameHighlightRole::ImminentThreat
                && highlight.side == Color::Black
        }),
        "imminent-threat source stones should be annotated as replay evidence: {:?}",
        boundary.evidence
    );
    assert!(
        boundary.evidence.iter().any(|highlight| {
            highlight.role == ReplayFrameHighlightRole::CounterThreat
                && highlight.side == Color::White
        }),
        "counter-threat source stones should be annotated as replay evidence: {:?}",
        boundary.evidence
    );
}

#[test]
fn replay_frame_annotations_mark_actual_counter_threat_hint() {
    let board = board_from_moves(
        Variant::Renju,
        &[
            "H8", "G7", "H6", "H7", "I7", "F7", "G5", "F4", "J8", "K9", "I8", "G8", "I6", "I9",
            "H9", "F6", "F5", "G9", "G10",
        ],
    );
    let options = AnalysisOptions {
        max_depth: 0,
        max_scan_plies: Some(64),
    };
    let frame = replay_frame_annotations_from_proof(
        19,
        &board,
        Color::Black,
        &proof_for_board(&board, Color::Black, &options),
        None,
        Some(mv("E7")),
        &options,
    );

    assert_eq!(frame.side_to_move, Color::White);
    assert!(
        frame.highlights.iter().any(|highlight| {
            highlight.role == ReplayFrameHighlightRole::CounterThreat
                && highlight.mv == mv("E7")
                && highlight.side == Color::White
        }),
        "White's actual E7 reply should be highlighted as a counter threat: {:?}",
        frame.highlights
    );
}

#[test]
fn replay_analysis_probes_all_imminent_combo_reply_outcomes() {
    let board = board_from_moves(
        Variant::Renju,
        &[
            "H8", "H7", "F8", "G9", "G8", "I8", "G6", "D9", "F9", "F10", "D7", "G10", "F7", "E10",
            "E8", "D8", "C6", "B5", "D10", "F11", "F6", "F5", "D6", "E6", "H5", "I4",
        ],
    );
    assert_eq!(board.current_player, Color::Black);

    let replies = analyze_alternate_defender_reply_options(
        &board,
        Color::White,
        Some(mv("C8")),
        &AnalysisOptions {
            max_depth: 0,
            max_scan_plies: Some(64),
        },
    );

    for notation in ["J7", "H9", "E12", "G12"] {
        let reply = reply_for(&replies, notation);
        assert!(
            reply.roles.contains(&DefenderReplyRole::ImminentDefense),
            "{notation} should be probed as a response to the 3+3 corridor: {replies:?}"
        );
    }
    assert!(
        replies.iter().all(|reply| reply.notation != "C8"),
        "the actual replay move is inherited from replay context, not re-probed: {replies:?}"
    );
}

#[test]
fn replay_analysis_does_not_treat_combo_imminent_actual_reply_as_miss() {
    let replay = replay_from_moves(
        Variant::Renju,
        &[
            "H8", "I9", "G8", "H6", "F8", "I8", "I7", "G9", "H9", "E6", "I10", "J11", "H10", "H11",
            "G10", "F10", "E8", "D8", "F11", "E9", "G11", "F6", "G6", "E7", "G5", "D6", "E12",
            "D13", "G4", "G7", "G3", "G2", "B6", "C9", "B10", "D9", "F9", "D7", "D5", "D10",
        ],
    );

    let analysis = analyze_replay(
        &replay,
        AnalysisOptions {
            max_depth: 4,
            max_scan_plies: Some(64),
        },
    )
    .expect("finished replay should analyze");

    let failure = analysis.failure.as_ref().expect("failure should classify");
    assert_eq!(failure.mode, FailureMode::MissedEscape);
    assert_eq!(failure.prefix_ply, Some(36));
    assert!(failure.actual_notation.is_none());
}

#[test]
fn failure_analysis_classifies_late_imminent_miss_before_lethal_prevention() {
    let replay = replay_from_moves(
        Variant::Renju,
        &[
            "H8", "H9", "J8", "I7", "I8", "G8", "I10", "L7", "K7", "I9", "J9", "G9", "J7", "J6",
            "K8", "L8", "K9", "K10", "J10", "J11", "G12", "H11", "L9", "F9", "E9", "I6", "N9",
            "M9", "H7", "G6", "K6", "K5", "G10", "I11", "H10", "F10", "G11", "F7", "F6", "L5",
            "N11", "M10", "G14", "G13", "N10", "L4", "M3", "L6",
        ],
    );

    let boards = replay_prefix_boards(&replay).expect("replay boards should build");
    let scan_start = 40;
    let proof_summary = proof_summary_with_escape(
        &boards,
        scan_start,
        Color::White,
        44,
        vec![mv("M3"), mv("L4"), mv("L6")],
        ReplyClassification::PossibleEscape,
        true,
    );
    let forced_interval = ForcedInterval {
        start_ply: 45,
        end_ply: replay.moves.len(),
    };
    let onset = test_lethal_onset(46, Color::White);

    let failure = failure_analysis(FailureAnalysisInput {
        replay: &replay,
        boards: &boards,
        proof_summary: &proof_summary,
        scan_start,
        final_forced_interval_found: true,
        final_forced_interval: &forced_interval,
        lethal_onset: Some(&onset),
        root_cause: RootCause::MissedDefense,
        winner: Color::White,
        loser: Color::Black,
    })
    .expect("failure should classify");
    assert_eq!(failure.mode, FailureMode::MissedImminentResponse);
    assert_eq!(failure.prefix_ply, Some(44));
    assert_eq!(failure.actual_notation.as_deref(), Some("N10"));
    for notation in ["M3", "L4", "L6"] {
        assert!(
            failure
                .missed_candidates
                .iter()
                .any(|candidate| candidate.notation == notation),
            "{notation} should be listed as a missed imminent response: {failure:?}"
        );
    }
}

#[test]
fn failure_analysis_classifies_early_prevention_as_missed_escape() {
    let replay = replay_from_moves(
        Variant::Renju,
        &[
            "H8", "H7", "F8", "G9", "G8", "I8", "G6", "D9", "F9", "F10", "D7", "G10", "F7", "E10",
            "E8", "D8", "C6", "B5", "F6", "F5", "E6", "D6", "D5", "C4", "H9",
        ],
    );

    let boards = replay_prefix_boards(&replay).expect("replay boards should build");
    let scan_start = 13;
    let proof_summary = proof_summary_with_escape(
        &boards,
        scan_start,
        Color::Black,
        13,
        vec![mv("E8")],
        ReplyClassification::PossibleEscape,
        true,
    );
    let forced_interval = ForcedInterval {
        start_ply: 14,
        end_ply: replay.moves.len(),
    };
    let onset = test_lethal_onset(21, Color::Black);

    let failure = failure_analysis(FailureAnalysisInput {
        replay: &replay,
        boards: &boards,
        proof_summary: &proof_summary,
        scan_start,
        final_forced_interval_found: true,
        final_forced_interval: &forced_interval,
        lethal_onset: Some(&onset),
        root_cause: RootCause::MissedDefense,
        winner: Color::Black,
        loser: Color::White,
    })
    .expect("failure should classify");
    assert_eq!(failure.mode, FailureMode::MissedEscape);
    assert_eq!(failure.prefix_ply, Some(13));
    assert_eq!(failure.actual_notation.as_deref(), Some("E10"));
    assert!(failure.missed_candidates.iter().any(|candidate| {
        candidate.notation == "E8"
            && candidate.outcome == MissedCandidateOutcome::PreventsLethalOnset
    }));
}

#[test]
fn replay_annotations_do_not_show_lower_tier_forbidden_replies_during_immediate_threats() {
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
    let options = AnalysisOptions {
        max_depth: 0,
        max_scan_plies: Some(64),
    };
    let proof = proof_for_board(&board, Color::White, &options);

    let frame = replay_frame_annotations_from_proof(
        52,
        &board,
        Color::White,
        &proof,
        None,
        Some(mv("F8")),
        &options,
    );

    assert!(
            frame
                .highlights
                .iter()
                .all(|highlight| highlight.mv != mv("D8")),
            "lower-tier forbidden imminent replies should not be highlighted while immediate threats are active: {:?}",
            frame.highlights
        );
    assert!(
            frame.markers.iter().all(|marker| marker.mv != mv("D8")),
            "lower-tier forbidden imminent replies should not be marked while immediate threats are active: {:?}",
            frame.markers
        );
    assert!(frame.highlights.iter().any(|highlight| {
        highlight.role == ReplayFrameHighlightRole::ImmediateThreat && highlight.mv == mv("F13")
    }));
}

#[test]
fn replay_analysis_session_marks_next_corridor_entry_on_escape_boundary() {
    let replay = replay_from_moves(
        Variant::Renju,
        &[
            "H8", "H9", "J8", "I7", "I8", "G8", "I10", "L7", "K7", "I9", "J9", "H7", "J7", "J10",
            "H11", "F13", "I6", "F9", "J6", "J5", "L8", "K8", "I5", "H4", "M9",
        ],
    );
    let mut session = ReplayAnalysisSession::new(
        replay,
        AnalysisOptions {
            max_depth: 4,
            max_scan_plies: Some(64),
        },
    )
    .expect("session should initialize");
    let mut annotations = Vec::new();
    loop {
        let step = session.step(2);
        annotations.extend(step.annotations);
        if step.done {
            break;
        }
    }

    let frame = annotations
        .iter()
        .rev()
        .find(|frame| frame.ply == 17)
        .expect("escape boundary frame should be annotated");

    assert_eq!(frame.side_to_move, Color::White);
    assert!(frame.highlights.iter().any(|highlight| {
        highlight.role == ReplayFrameHighlightRole::CorridorEntry
            && highlight.mv == mv("J6")
            && highlight.side == Color::Black
    }));
    assert!(frame.markers.iter().any(|marker| {
        marker.role == ReplayFrameMarkerRole::ConfirmedEscape
            && marker.mv == mv("J6")
            && marker.side == Color::White
    }));
    assert!(
            frame
                .highlights
                .iter()
                .all(|highlight| highlight.mv != mv("F9")),
            "the escape boundary should point at the next corridor-entry move, not the current actual reply: {:?}",
            frame.highlights
        );
}

#[test]
fn replay_analysis_extends_forced_corridor_through_forbidden_renju_defenses() {
    let replay = replay_from_moves(
        Variant::Renju,
        &[
            "H8", "G7", "H9", "J8", "H7", "H6", "I7", "F8", "E9", "H10", "I5", "G9", "E7", "I9",
            "K7", "G10", "G11", "I11", "J12", "G6", "G8", "F6", "L7", "J7", "J6", "E6", "D6", "I6",
        ],
    );

    let analysis = analyze_replay(
        &replay,
        AnalysisOptions {
            max_depth: 4,
            max_scan_plies: Some(64),
        },
    )
    .expect("finished replay should analyze");

    assert_eq!(analysis.root_cause, RootCause::MissedDefense);
    assert_eq!(analysis.final_forced_interval.start_ply, 23);
    assert_eq!(analysis.critical_loser_ply, Some(23));
    assert_eq!(analysis.unclear_reason, None);
    assert!(analysis.unknown_gaps.is_empty());
}

#[test]
fn corridor_reply_moves_exclude_non_corridor_actual_reply() {
    let board = board_from_moves(Variant::Freestyle, &["H8", "A1", "I8"]);
    let actual_moves = vec![mv("H8"), mv("A1"), mv("I8"), mv("A2"), mv("J8")];
    let threat = super::ThreatReplySet::new(&board, Color::Black);

    let replies = super::corridor_defender_reply_moves(
        &board,
        &actual_moves,
        3,
        &AnalysisOptions::default(),
        &threat,
    );

    assert!(
        !replies.contains(&mv("A2")),
        "non-corridor actual defender move must not become a probed reply: {replies:?}"
    );
    assert!(
        replies.contains(&mv("J8")),
        "the winner's next corridor-entry square should remain visible as the escape target"
    );
}

#[test]
fn actual_corridor_reply_forced_child_is_not_escape() {
    let board = board_from_moves(Variant::Freestyle, &["H8"]);
    let options = AnalysisOptions::default();
    let forced_child = proof_for_board(&board, Color::Black, &options);

    let outcome = super::classify_actual_corridor_reply(
        &board,
        &[],
        Color::Black,
        &options,
        1,
        mv("A1"),
        Some(&forced_child),
    );

    assert_eq!(outcome.status, super::CorridorReplyStatus::Forced);
}

#[test]
fn actual_corridor_reply_limit_hit_is_possible_escape() {
    let board = board_from_moves(Variant::Freestyle, &["H8"]);
    let options = AnalysisOptions::default();
    let unknown_child = super::with_limit_causes(
        super::corridor_proof_result(
            &board,
            Color::Black,
            &options,
            ProofStatus::Unknown,
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ),
        [ProofLimitCause::DepthCutoff],
    );

    let outcome = super::classify_actual_corridor_reply(
        &board,
        &[],
        Color::Black,
        &options,
        1,
        mv("A1"),
        Some(&unknown_child),
    );

    assert_eq!(outcome.status, super::CorridorReplyStatus::PossibleEscape);
}

#[test]
fn replay_analysis_attaches_actual_reply_to_actual_line_evidence() {
    let replay = replay_from_moves(
        Variant::Freestyle,
        &["H8", "G8", "I8", "A1", "J8", "A2", "K8", "B1", "L8"],
    );

    let analysis = analyze_replay(
        &replay,
        AnalysisOptions {
            max_depth: 2,
            max_scan_plies: None,
        },
    )
    .expect("finished replay should analyze");

    let proof = analysis
        .proof_summary
        .get(7)
        .expect("ply 7 proof should be scanned");
    let evidence = proof
        .threat_evidence
        .first()
        .expect("ply 7 immediate threat should be explained");
    assert_eq!(evidence.prefix_ply, Some(7));
    assert_eq!(evidence.actual_reply, Some(mv("B1")));
}

#[test]
fn replay_analysis_tracks_conversion_error_before_final_interval() {
    let replay = replay_from_moves(
        Variant::Freestyle,
        &[
            "H8", "G8", "I8", "A1", "J8", "A2", "K8", "B1", "C1", "B2", "L8",
        ],
    );

    let analysis = analyze_replay(
        &replay,
        AnalysisOptions {
            max_depth: 2,
            max_scan_plies: None,
        },
    )
    .expect("finished replay should analyze");

    assert_eq!(analysis.final_forced_interval.start_ply, 10);
    assert_eq!(analysis.root_cause, RootCause::MissedDefense);
    assert!(analysis
        .tactical_notes
        .contains(&TacticalNote::ConversionError));
    assert!(analysis.tactical_notes.contains(&TacticalNote::MissedWin));
}

#[test]
fn replay_analysis_labels_losing_side_missed_win_as_root_cause() {
    let replay = replay_from_moves(
        Variant::Freestyle,
        &[
            "A1", "H8", "A2", "I8", "A3", "J8", "B1", "K8", "A4", "C1", "A5",
        ],
    );

    let analysis = analyze_replay(
        &replay,
        AnalysisOptions {
            max_depth: 2,
            ..AnalysisOptions::default()
        },
    )
    .expect("finished replay should analyze");

    assert_eq!(analysis.winner, Some(Color::Black));
    assert_eq!(analysis.root_cause, RootCause::MissedWin);
    assert_eq!(analysis.critical_loser_ply, Some(10));
    assert!(analysis.tactical_notes.contains(&TacticalNote::MissedWin));
}

#[test]
fn shallow_corridor_analysis_finds_open_four_point_of_no_return() {
    let replay = replay_from_moves(
        Variant::Freestyle,
        &["H8", "A1", "I8", "A2", "J8", "A3", "K8", "B1", "L8"],
    );

    let analysis = analyze_replay(
        &replay,
        AnalysisOptions {
            max_depth: 1,
            max_scan_plies: Some(4),
        },
    )
    .expect("finished replay should analyze");

    assert_eq!(analysis.root_cause, RootCause::MissedDefense);
    assert!(analysis.final_forced_interval_found);
    assert_eq!(analysis.final_forced_interval.start_ply, 6);
    assert_eq!(analysis.last_chance_ply, Some(5));
}

#[test]
fn ongoing_replay_has_no_winner_and_unknown_root_cause() {
    let replay = replay_from_moves(Variant::Freestyle, &["H8", "A1", "I8"]);

    let analysis = analyze_replay(&replay, AnalysisOptions::default())
        .expect("ongoing replay should still produce a bounded summary");

    assert_eq!(analysis.winner, None);
    assert_eq!(analysis.root_cause, RootCause::Unclear);
    assert_eq!(analysis.unclear_reason, Some(UnclearReason::DrawOrOngoing));
    assert!(!analysis.final_forced_interval_found);
    assert_eq!(analysis.final_forced_interval.start_ply, 0);
    assert_eq!(analysis.final_forced_interval.end_ply, 0);
}

#[test]
fn renju_forbidden_defense_remains_model_visible() {
    let replay = replay_from_moves(Variant::Renju, &["H8", "A1"]);

    let analysis = analyze_replay(
        &replay,
        AnalysisOptions {
            max_depth: 4,
            ..AnalysisOptions::default()
        },
    )
    .expect("ongoing renju replay should analyze");

    assert_eq!(analysis.root_cause, RootCause::Unclear);
}
