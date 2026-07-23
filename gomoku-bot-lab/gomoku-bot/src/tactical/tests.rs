use super::{
    corridor_active_threats, corridor_defender_reply_moves, defender_hint_reply_candidates,
    defender_reply_candidates, has_forcing_local_threat, has_forcing_local_threat_at_move,
    legal_forcing_continuations_for_fact, lethal_threat, local_threat_evidence_stones,
    local_threat_facts_after_move, local_threat_facts_for_player, normalize_local_threat_facts,
    one_step_lethal_threat, one_step_lethal_threat_analysis, raw_local_threat_facts_after_move,
    raw_local_threat_facts_for_player, terminal_lethal_threat, terminal_lethal_threat_analysis,
    CorridorThreatPolicy, DefenderReplyCandidate, DefenderReplyRole, LethalThreatKind,
    LocalThreatFact, LocalThreatKind, LocalThreatOrigin, ScanThreatView, SearchThreatPolicy,
    ThreatObligationKind, ThreatView,
};
use gomoku_core::{Board, Color, Move, RuleConfig, Variant};

fn mv(notation: &str) -> Move {
    Move::from_notation(notation).expect("test move notation should parse")
}

fn apply_moves(board: &mut Board, moves: &[&str]) {
    for notation in moves {
        board.apply_move(mv(notation)).unwrap();
    }
}

fn fact(
    player: Color,
    kind: LocalThreatKind,
    origin: &str,
    defense_squares: &[&str],
    rest_squares: &[&str],
) -> LocalThreatFact {
    LocalThreatFact {
        player,
        kind,
        origin: LocalThreatOrigin::Existing(mv(origin)),
        defense_squares: defense_squares
            .iter()
            .map(|notation| mv(notation))
            .collect(),
        rest_squares: rest_squares.iter().map(|notation| mv(notation)).collect(),
    }
}

fn board_from_moves(variant: Variant, moves: &[&str]) -> Board {
    let mut board = Board::new(RuleConfig {
        variant,
        ..RuleConfig::default()
    });
    apply_moves(&mut board, moves);
    board
}

fn notation_list(moves: &[Move]) -> Vec<String> {
    moves.iter().map(|mv| mv.to_notation()).collect()
}

fn one_step_reply_entries(
    analysis: &super::OneStepLethalThreatAnalysis,
) -> Vec<(String, Vec<String>)> {
    analysis
        .defender_replies
        .iter()
        .map(|reply| {
            (
                reply.reply.to_notation(),
                reply
                    .lethal_entries
                    .iter()
                    .map(|entry| entry.mv.to_notation())
                    .collect(),
            )
        })
        .collect()
}

fn has_reply_role(
    candidates: &[DefenderReplyCandidate],
    notation: &str,
    role: DefenderReplyRole,
) -> bool {
    let mv = mv(notation);
    candidates
        .iter()
        .any(|candidate| candidate.mv == mv && candidate.roles.contains(&role))
}

#[test]
fn terminal_lethal_threat_detects_open_four_coverage() {
    let board = board_from_moves(
        Variant::Freestyle,
        &["H8", "A1", "I8", "A2", "J8", "A3", "K8"],
    );
    assert_eq!(board.current_player, Color::White);

    let analysis = terminal_lethal_threat_analysis(&board, Color::Black);

    assert_eq!(notation_list(&analysis.terminal_targets), vec!["G8", "L8"]);
    assert!(analysis.defender_immediate_wins.is_empty());
    assert!(analysis.covering_replies.is_empty());
    assert_eq!(
        terminal_lethal_threat(&board, Color::Black)
            .expect("open four should be terminal lethal")
            .terminal_targets,
        vec![mv("G8"), mv("L8")]
    );
}

#[test]
fn terminal_lethal_threat_rejects_single_blockable_four() {
    let board = board_from_moves(
        Variant::Freestyle,
        &["H8", "G8", "I8", "A1", "J8", "A2", "K8"],
    );
    assert_eq!(board.current_player, Color::White);

    let analysis = terminal_lethal_threat_analysis(&board, Color::Black);

    assert_eq!(notation_list(&analysis.terminal_targets), vec!["L8"]);
    assert_eq!(notation_list(&analysis.covering_replies), vec!["L8"]);
    assert!(terminal_lethal_threat(&board, Color::Black).is_none());
}

#[test]
fn terminal_lethal_threat_rejects_when_defender_can_win_now() {
    let board = board_from_moves(
        Variant::Freestyle,
        &["B1", "H8", "B2", "I8", "B3", "J8", "B4", "K8"],
    );
    assert_eq!(board.current_player, Color::Black);

    let analysis = terminal_lethal_threat_analysis(&board, Color::White);

    assert_eq!(notation_list(&analysis.terminal_targets), vec!["G8", "L8"]);
    assert_eq!(notation_list(&analysis.defender_immediate_wins), vec!["B5"]);
    assert!(analysis.covering_replies.is_empty());
    assert!(terminal_lethal_threat(&board, Color::White).is_none());
}

#[test]
fn terminal_lethal_threat_uses_renju_forbidden_replies_as_missing_coverage() {
    let board = board_from_moves(
        Variant::Renju,
        &[
            "H8", "H7", "J8", "I9", "I8", "G8", "F9", "F7", "H9", "G7", "I7", "E7", "D7", "G9",
            "G6", "G11",
        ],
    );
    assert_eq!(board.current_player, Color::Black);
    assert!(!board.is_legal_for_color(mv("G10"), Color::Black));

    let analysis = terminal_lethal_threat_analysis(&board, Color::White);

    assert_eq!(notation_list(&analysis.terminal_targets), vec!["G10"]);
    assert!(analysis.defender_immediate_wins.is_empty());
    assert!(
        analysis.covering_replies.is_empty(),
        "forbidden direct blocks should not count as legal coverage: {:?}",
        analysis.covering_replies
    );
    assert!(terminal_lethal_threat(&board, Color::White).is_some());
}

#[test]
fn one_step_lethal_threat_detects_four_three_coverage() {
    let board = board_from_moves(
        Variant::Freestyle,
        &[
            "H8", "G8", "I8", "A1", "J8", "O1", "K8", "A15", "I7", "O15", "I9",
        ],
    );
    assert_eq!(board.current_player, Color::White);

    let analysis = one_step_lethal_threat_analysis(&board, Color::Black);

    assert_eq!(
        notation_list(&analysis.terminal.terminal_targets),
        vec!["L8"]
    );
    assert_eq!(
        notation_list(&analysis.terminal.covering_replies),
        vec!["L8"]
    );
    assert!(
        analysis.escaping_replies.is_empty(),
        "unexpected escapes: {:?}; replies: {:?}",
        notation_list(&analysis.escaping_replies),
        one_step_reply_entries(&analysis)
    );
    assert_eq!(
        one_step_reply_entries(&analysis),
        vec![("L8".to_string(), vec!["I6".to_string(), "I10".to_string()])]
    );
    assert_eq!(
        one_step_lethal_threat(&board, Color::Black)
            .expect("4+3 should be one-step lethal")
            .kind,
        LethalThreatKind::OneStepCoverage
    );
    assert_eq!(
        lethal_threat(&board, Color::Black)
            .expect("general classifier should find 4+3 lethal")
            .kind,
        LethalThreatKind::OneStepCoverage
    );
}

#[test]
fn one_step_lethal_threat_detects_double_three_coverage() {
    let board = board_from_moves(
        Variant::Freestyle,
        &["H8", "A1", "I8", "O1", "J8", "A15", "I7", "O15", "I9"],
    );
    assert_eq!(board.current_player, Color::White);

    let analysis = one_step_lethal_threat_analysis(&board, Color::Black);

    assert!(analysis.terminal.terminal_targets.is_empty());
    assert!(
        analysis.escaping_replies.is_empty(),
        "unexpected escapes: {:?}; replies: {:?}",
        notation_list(&analysis.escaping_replies),
        one_step_reply_entries(&analysis)
    );
    assert_eq!(
        one_step_reply_entries(&analysis),
        vec![
            ("I6".to_string(), vec!["G8".to_string(), "K8".to_string()]),
            ("G8".to_string(), vec!["I6".to_string(), "I10".to_string()]),
            ("K8".to_string(), vec!["I6".to_string(), "I10".to_string()]),
            ("I10".to_string(), vec!["G8".to_string(), "K8".to_string()]),
        ]
    );
    assert_eq!(
        one_step_lethal_threat(&board, Color::Black)
            .expect("3+3 should be one-step lethal")
            .kind,
        LethalThreatKind::OneStepCoverage
    );
}

#[test]
fn one_step_lethal_threat_rejects_open_cross_shared_block() {
    let board = board_from_moves(
        Variant::Freestyle,
        &[
            "G8", "A1", "H8", "O1", "J8", "A15", "I6", "O15", "I7", "C3", "I9",
        ],
    );
    assert_eq!(board.current_player, Color::White);

    let analysis = one_step_lethal_threat_analysis(&board, Color::Black);

    assert!(
        notation_list(&analysis.escaping_replies).contains(&"I8".to_string()),
        "the open crossing point should be a shared escape: {:?}; replies: {:?}",
        notation_list(&analysis.escaping_replies),
        one_step_reply_entries(&analysis)
    );
    assert!(one_step_lethal_threat(&board, Color::Black).is_none());
    assert!(lethal_threat(&board, Color::Black).is_none());
}

#[test]
fn one_step_lethal_threat_rejects_single_open_three() {
    let board = board_from_moves(Variant::Freestyle, &["H8", "A1", "I8", "C1", "J8"]);
    assert_eq!(board.current_player, Color::White);

    let analysis = one_step_lethal_threat_analysis(&board, Color::Black);

    assert_eq!(notation_list(&analysis.escaping_replies), vec!["G8", "K8"]);
    assert!(one_step_lethal_threat(&board, Color::Black).is_none());
    assert!(lethal_threat(&board, Color::Black).is_none());
}

#[test]
fn normalize_local_threat_facts_sorts_inner_moves_and_dedups_shapes() {
    let facts = vec![
        fact(
            Color::Black,
            LocalThreatKind::OpenThree,
            "J8",
            &["L8", "H8"],
            &["K8", "I8"],
        ),
        fact(
            Color::Black,
            LocalThreatKind::OpenThree,
            "I8",
            &["H8", "L8"],
            &["I8", "K8"],
        ),
        fact(
            Color::White,
            LocalThreatKind::ClosedFour,
            "C3",
            &["B3"],
            &[],
        ),
    ];

    let normalized = normalize_local_threat_facts(facts);

    assert_eq!(
        normalized,
        vec![
            fact(
                Color::Black,
                LocalThreatKind::OpenThree,
                "J8",
                &["H8", "L8"],
                &["I8", "K8"],
            ),
            fact(
                Color::White,
                LocalThreatKind::ClosedFour,
                "C3",
                &["B3"],
                &[],
            ),
        ]
    );
}

fn assert_raw_fact_parity(
    before_moves: &[&str],
    gain: &str,
    player: Color,
    kind: LocalThreatKind,
    defense_squares: &[&str],
    rest_squares: &[&str],
) {
    let mut before = Board::new(RuleConfig::default());
    apply_moves(&mut before, before_moves);

    let mut expected_defense_squares = defense_squares
        .iter()
        .map(|notation| mv(notation))
        .collect::<Vec<_>>();
    let mut expected_rest_squares = rest_squares
        .iter()
        .map(|notation| mv(notation))
        .collect::<Vec<_>>();
    expected_defense_squares.sort_by_key(|mv| (mv.row, mv.col));
    expected_rest_squares.sort_by_key(|mv| (mv.row, mv.col));

    let after_move_fact = raw_local_threat_facts_after_move(&before, mv(gain))
        .into_iter()
        .find(|fact| {
            fact.player == player
                && fact.kind == kind
                && fact.defense_squares == expected_defense_squares
                && fact.rest_squares == expected_rest_squares
        })
        .unwrap_or_else(|| panic!("after-move detector should see {kind:?} with expected squares"));

    let mut existing = before.clone();
    existing.apply_move(mv(gain)).unwrap();
    assert!(
        raw_local_threat_facts_for_player(&existing, player)
            .iter()
            .any(|fact| {
                fact.kind == after_move_fact.kind
                    && fact.defense_squares == after_move_fact.defense_squares
                    && fact.rest_squares == after_move_fact.rest_squares
            }),
        "existing-board detector should produce the same raw shape as after-move detector"
    );
}

fn assert_no_raw_broken_three_after_move(before_moves: &[&str], gain: &str) {
    let mut before = Board::new(RuleConfig::default());
    apply_moves(&mut before, before_moves);

    let facts = raw_local_threat_facts_after_move(&before, mv(gain));
    assert!(
        facts
            .iter()
            .all(|fact| fact.kind != LocalThreatKind::BrokenThree),
        "shape should not be a forcing broken three: {facts:?}"
    );
}

#[test]
fn local_threat_facts_after_move_report_five_open_four_and_closed_four() {
    let mut five_board = Board::new(RuleConfig::default());
    apply_moves(
        &mut five_board,
        &["H8", "A1", "I8", "A2", "J8", "A3", "K8", "A4"],
    );
    assert_eq!(
        local_threat_facts_after_move(&five_board, mv("L8")),
        vec![LocalThreatFact {
            player: Color::Black,
            kind: LocalThreatKind::Five,
            origin: LocalThreatOrigin::AfterMove(mv("L8")),
            defense_squares: vec![],
            rest_squares: vec![],
        }]
    );

    let mut open_four_board = Board::new(RuleConfig::default());
    apply_moves(&mut open_four_board, &["H8", "A1", "I8", "A2", "J8", "A3"]);
    assert_eq!(
        local_threat_facts_after_move(&open_four_board, mv("K8")),
        vec![LocalThreatFact {
            player: Color::Black,
            kind: LocalThreatKind::OpenFour,
            origin: LocalThreatOrigin::AfterMove(mv("K8")),
            defense_squares: vec![mv("G8"), mv("L8")],
            rest_squares: vec![],
        }]
    );

    let mut closed_four_board = Board::new(RuleConfig::default());
    apply_moves(
        &mut closed_four_board,
        &["H8", "G8", "I8", "A1", "J8", "A2"],
    );
    assert_eq!(
        local_threat_facts_after_move(&closed_four_board, mv("K8")),
        vec![LocalThreatFact {
            player: Color::Black,
            kind: LocalThreatKind::ClosedFour,
            origin: LocalThreatOrigin::AfterMove(mv("K8")),
            defense_squares: vec![mv("L8")],
            rest_squares: vec![],
        }]
    );
}

#[test]
fn local_threat_facts_after_move_report_open_closed_and_broken_three() {
    let mut open_three_board = Board::new(RuleConfig::default());
    apply_moves(&mut open_three_board, &["H8", "A1", "I8", "A2"]);
    assert_eq!(
        local_threat_facts_after_move(&open_three_board, mv("J8")),
        vec![LocalThreatFact {
            player: Color::Black,
            kind: LocalThreatKind::OpenThree,
            origin: LocalThreatOrigin::AfterMove(mv("J8")),
            defense_squares: vec![mv("G8"), mv("K8")],
            rest_squares: vec![],
        }]
    );

    let mut closed_three_board = Board::new(RuleConfig::default());
    apply_moves(&mut closed_three_board, &["H8", "G8", "I8", "A1"]);
    assert_eq!(
        local_threat_facts_after_move(&closed_three_board, mv("J8")),
        vec![LocalThreatFact {
            player: Color::Black,
            kind: LocalThreatKind::ClosedThree,
            origin: LocalThreatOrigin::AfterMove(mv("J8")),
            defense_squares: vec![mv("K8")],
            rest_squares: vec![],
        }]
    );

    let mut broken_three_board = Board::new(RuleConfig::default());
    apply_moves(&mut broken_three_board, &["H8", "A1", "I8", "C1"]);
    assert_eq!(
        local_threat_facts_after_move(&broken_three_board, mv("K8")),
        vec![LocalThreatFact {
            player: Color::Black,
            kind: LocalThreatKind::BrokenThree,
            origin: LocalThreatOrigin::AfterMove(mv("K8")),
            defense_squares: vec![mv("G8"), mv("J8"), mv("L8")],
            rest_squares: vec![mv("J8")],
        }]
    );
}

#[test]
fn fixed_window_broken_threes_are_not_forcing() {
    assert_no_raw_broken_three_after_move(&["H8", "A1", "J8", "C1"], "L8"); // X_X_X
    assert_no_raw_broken_three_after_move(&["H8", "A1", "I8", "C1"], "L8"); // XX__X
    assert_no_raw_broken_three_after_move(&["H8", "A1", "K8", "C1"], "L8"); // X__XX
}

#[test]
fn one_side_blocked_sliding_broken_threes_are_not_forcing() {
    assert_no_raw_broken_three_after_move(&["H8", "G8", "I8", "A1"], "K8"); // OXX_X_
    assert_no_raw_broken_three_after_move(&["H8", "L8", "I8", "A1"], "K8"); // _XX_XO
    assert_no_raw_broken_three_after_move(&["H8", "G8", "J8", "A1"], "K8"); // OX_XX_
    assert_no_raw_broken_three_after_move(&["H8", "L8", "J8", "A1"], "K8"); // _X_XXO
}

#[test]
fn local_threat_facts_after_move_report_open_three_blocked_outer_variants() {
    let mut left_blocked_board = Board::new(RuleConfig::default());
    apply_moves(&mut left_blocked_board, &["J9", "H9", "K9", "A1"]);
    assert_eq!(
        local_threat_facts_after_move(&left_blocked_board, mv("L9")),
        vec![LocalThreatFact {
            player: Color::Black,
            kind: LocalThreatKind::OpenThree,
            origin: LocalThreatOrigin::AfterMove(mv("L9")),
            defense_squares: vec![mv("I9"), mv("M9"), mv("N9")],
            rest_squares: vec![],
        }]
    );

    let mut right_blocked_board = Board::new(RuleConfig::default());
    apply_moves(&mut right_blocked_board, &["J9", "N9", "K9", "A1"]);
    assert_eq!(
        local_threat_facts_after_move(&right_blocked_board, mv("L9")),
        vec![LocalThreatFact {
            player: Color::Black,
            kind: LocalThreatKind::OpenThree,
            origin: LocalThreatOrigin::AfterMove(mv("L9")),
            defense_squares: vec![mv("H9"), mv("I9"), mv("M9")],
            rest_squares: vec![],
        }]
    );
}

#[test]
fn boxed_three_is_not_an_active_open_three() {
    let board = board_from_moves(Variant::Freestyle, &["J9", "H9", "K9", "N9", "L9"]);
    let facts = local_threat_facts_for_player(&board, Color::Black);
    assert!(
        facts
            .iter()
            .all(|fact| fact.kind != LocalThreatKind::OpenThree),
        "{facts:?}"
    );
}

#[test]
fn local_threat_facts_for_player_report_open_closed_and_broken_fours() {
    let open_four = board_from_moves(
        Variant::Freestyle,
        &["H8", "A1", "I8", "A2", "J8", "A3", "K8"],
    );
    assert!(
        local_threat_facts_for_player(&open_four, Color::Black).contains(&LocalThreatFact {
            player: Color::Black,
            kind: LocalThreatKind::OpenFour,
            origin: LocalThreatOrigin::Existing(mv("H8")),
            defense_squares: vec![mv("G8"), mv("L8")],
            rest_squares: vec![],
        })
    );

    let closed_four = board_from_moves(
        Variant::Freestyle,
        &["H8", "G8", "I8", "A1", "J8", "A2", "K8"],
    );
    assert!(
        local_threat_facts_for_player(&closed_four, Color::Black).contains(&LocalThreatFact {
            player: Color::Black,
            kind: LocalThreatKind::ClosedFour,
            origin: LocalThreatOrigin::Existing(mv("H8")),
            defense_squares: vec![mv("L8")],
            rest_squares: vec![],
        })
    );

    let broken_four = board_from_moves(
        Variant::Freestyle,
        &["H8", "A1", "I8", "A2", "K8", "A3", "L8"],
    );
    assert!(
        local_threat_facts_for_player(&broken_four, Color::Black).contains(&LocalThreatFact {
            player: Color::Black,
            kind: LocalThreatKind::BrokenFour,
            origin: LocalThreatOrigin::Existing(mv("H8")),
            defense_squares: vec![mv("J8")],
            rest_squares: vec![],
        })
    );
}

#[test]
fn local_threat_facts_for_player_report_open_three_outer_variants_and_broken_three() {
    let left_blocked = board_from_moves(Variant::Renju, &["J9", "H9", "K9", "A1", "L9"]);
    assert!(
        local_threat_facts_for_player(&left_blocked, Color::Black).contains(&LocalThreatFact {
            player: Color::Black,
            kind: LocalThreatKind::OpenThree,
            origin: LocalThreatOrigin::Existing(mv("J9")),
            defense_squares: vec![mv("I9"), mv("M9"), mv("N9")],
            rest_squares: vec![],
        })
    );

    let right_blocked = board_from_moves(Variant::Renju, &["J9", "N9", "K9", "A1", "L9"]);
    assert!(
        local_threat_facts_for_player(&right_blocked, Color::Black).contains(&LocalThreatFact {
            player: Color::Black,
            kind: LocalThreatKind::OpenThree,
            origin: LocalThreatOrigin::Existing(mv("J9")),
            defense_squares: vec![mv("H9"), mv("I9"), mv("M9")],
            rest_squares: vec![],
        })
    );

    let split_three = board_from_moves(Variant::Freestyle, &["H8", "A1", "I8", "C1", "K8"]);
    assert!(
        local_threat_facts_for_player(&split_three, Color::Black).contains(&LocalThreatFact {
            player: Color::Black,
            kind: LocalThreatKind::BrokenThree,
            origin: LocalThreatOrigin::Existing(mv("H8")),
            defense_squares: vec![mv("G8"), mv("J8"), mv("L8")],
            rest_squares: vec![mv("J8")],
        })
    );
}

#[test]
fn local_threat_evidence_stones_identify_existing_open_three_shape() {
    let board = board_from_moves(Variant::Freestyle, &["H8", "A1", "I8", "A2", "J8"]);
    let fact = fact(
        Color::Black,
        LocalThreatKind::OpenThree,
        "H8",
        &["G8", "K8"],
        &[],
    );

    assert_eq!(
        notation_list(&local_threat_evidence_stones(&board, &fact)),
        vec!["H8", "I8", "J8"],
    );
}

#[test]
fn local_threat_evidence_stones_exclude_virtual_candidate_move() {
    let board = board_from_moves(
        Variant::Freestyle,
        &["H8", "A1", "I8", "A2", "J8", "A3", "K8"],
    );
    let annotation = SearchThreatPolicy.annotation_for_player(&board, Color::Black, mv("L8"));
    let five = annotation
        .local_threats
        .iter()
        .find(|fact| fact.kind == LocalThreatKind::Five)
        .expect("candidate should complete a five");

    assert_eq!(
        notation_list(&local_threat_evidence_stones(&board, five)),
        vec!["H8", "I8", "J8", "K8"],
    );
}

#[test]
fn closed_three_endpoint_is_not_a_corridor_reply() {
    let left_blocked = board_from_moves(Variant::Renju, &["G8", "H8", "A1", "I8", "A2", "J8"]);
    assert_eq!(left_blocked.current_player, Color::Black);

    let replies = CorridorThreatPolicy.defender_reply_moves(&left_blocked, Color::White, None);
    assert!(
        !replies.contains(&mv("K8")),
        "the open endpoint of OXXX_ is a closed-three extension, not a forced reply: {replies:?}"
    );

    let right_blocked = board_from_moves(Variant::Renju, &["K8", "H8", "A1", "I8", "A2", "J8"]);
    assert_eq!(right_blocked.current_player, Color::Black);

    let replies = CorridorThreatPolicy.defender_reply_moves(&right_blocked, Color::White, None);
    assert!(
        !replies.contains(&mv("G8")),
        "the open endpoint of _XXXO is a closed-three extension, not a forced reply: {replies:?}"
    );
}

#[test]
fn defender_hint_candidates_require_imminent_threat_for_counter() {
    let board = board_from_moves(Variant::Freestyle, &["H8", "A1", "I8", "O1", "J8", "A15"]);
    assert_eq!(board.current_player, Color::Black);

    let candidates = defender_hint_reply_candidates(&board, Color::White);

    assert!(
        candidates.iter().all(|candidate| !candidate
            .roles
            .contains(&DefenderReplyRole::OffensiveCounter)),
        "quiet positions should not expose offensive counters as UI hints: {candidates:?}"
    );
}

#[test]
fn defender_reply_candidates_require_imminent_threat_for_counter() {
    let board = board_from_moves(Variant::Freestyle, &["H8", "A1", "I8", "O1", "J8", "A15"]);
    assert_eq!(board.current_player, Color::Black);

    let candidates = defender_reply_candidates(&board, Color::White, None);

    assert!(
        candidates.iter().all(|candidate| !candidate
            .roles
            .contains(&DefenderReplyRole::OffensiveCounter)),
        "quiet positions should not expose offensive counters as corridor replies: {candidates:?}"
    );
}

#[test]
fn defender_hint_candidates_prioritize_immediate_replies_over_imminent_replies() {
    let board = board_from_moves(
        Variant::Freestyle,
        &[
            "A1", "H8", "C2", "I8", "E3", "J8", "G4", "K8", "I5", "F6", "K6", "G6", "M7", "H6",
        ],
    );
    assert_eq!(board.current_player, Color::Black);

    let candidates = defender_hint_reply_candidates(&board, Color::White);

    assert!(has_reply_role(
        &candidates,
        "G8",
        DefenderReplyRole::ImmediateDefense
    ));
    assert!(has_reply_role(
        &candidates,
        "L8",
        DefenderReplyRole::ImmediateDefense
    ));
    assert!(
        candidates.iter().all(|candidate| !candidate
            .roles
            .contains(&DefenderReplyRole::ImminentDefense)),
        "imminent replies should be suppressed while immediate replies exist: {candidates:?}"
    );
}

#[test]
fn defender_reply_candidates_prioritize_immediate_replies_over_imminent_replies() {
    let board = board_from_moves(
        Variant::Freestyle,
        &[
            "A1", "H8", "C2", "I8", "E3", "J8", "G4", "K8", "I5", "F6", "K6", "G6", "M7", "H6",
        ],
    );
    assert_eq!(board.current_player, Color::Black);

    let candidates = defender_reply_candidates(&board, Color::White, None);

    assert!(has_reply_role(
        &candidates,
        "G8",
        DefenderReplyRole::ImmediateDefense
    ));
    assert!(has_reply_role(
        &candidates,
        "L8",
        DefenderReplyRole::ImmediateDefense
    ));
    assert!(
        candidates.iter().all(|candidate| !candidate
            .roles
            .contains(&DefenderReplyRole::ImminentDefense)),
        "4+3 combos should probe the immediate 4 blocks, not lower-priority 3 replies: {candidates:?}"
    );
}

#[test]
fn defender_reply_candidates_cover_all_imminent_combo_threats() {
    let board = board_from_moves(
        Variant::Renju,
        &[
            "H8", "H7", "F8", "G9", "G8", "I8", "G6", "D9", "F9", "F10", "D7", "G10", "F7", "E10",
            "E8", "D8", "C6", "B5", "D10", "F11", "F6", "F5", "D6", "E6", "H5", "I4",
        ],
    );
    assert_eq!(board.current_player, Color::Black);

    let candidates = defender_reply_candidates(&board, Color::White, Some(mv("C8")));

    for notation in ["J7", "H9", "E12", "G12"] {
        assert!(
            has_reply_role(&candidates, notation, DefenderReplyRole::ImminentDefense),
            "{notation} should be probed as a response to the 3+3 corridor: {candidates:?}"
        );
    }
    assert!(has_reply_role(
        &candidates,
        "C8",
        DefenderReplyRole::ImminentDefense
    ));
    assert!(has_reply_role(&candidates, "C8", DefenderReplyRole::Actual));
}

#[test]
fn defender_reply_candidates_cover_combo_from_individually_nonforcing_threes() {
    let board = board_from_moves(
        Variant::Renju,
        &[
            "H8", "I9", "G8", "H6", "F8", "I8", "I7", "G9", "H9", "E6", "I10", "J11", "H10", "H11",
            "G10", "F10", "E8", "D8", "F11", "E9", "G11", "F6", "G6", "E7", "G5", "D6", "E12",
            "D13", "G4", "G7", "G3", "G2",
        ],
    );
    assert_eq!(board.current_player, Color::Black);

    let obligation = ScanThreatView::new(&board)
        .threat_obligation(Color::White)
        .expect("compound imminent threat should produce a position obligation");
    assert_eq!(obligation.kind, ThreatObligationKind::Imminent);
    assert!(
        obligation
            .compound_entries
            .iter()
            .any(|entry| entry.mv == mv("B6")),
        "B6 should be recognized as a one-step entry into lethal coverage: {obligation:?}"
    );

    let candidates = defender_reply_candidates(&board, Color::White, Some(mv("B6")));

    assert!(
        has_reply_role(&candidates, "B6", DefenderReplyRole::ImminentDefense),
        "B6 should be visible as a defensive reply to the combined imminent threat: {candidates:?}"
    );
    assert!(has_reply_role(&candidates, "B6", DefenderReplyRole::Actual));
    for notation in ["J10", "K10"] {
        assert!(
            has_reply_role(&candidates, notation, DefenderReplyRole::OffensiveCounter),
            "{notation} should remain visible as counter-threat escape: {candidates:?}"
        );
    }
}

#[test]
fn raw_after_move_and_existing_board_facts_share_shape_logic() {
    assert_raw_fact_parity(
        &["H8", "A1", "I8", "A2"],
        "J8",
        Color::Black,
        LocalThreatKind::OpenThree,
        &["G8", "K8"],
        &[],
    );
    assert_raw_fact_parity(
        &["J9", "H9", "K9", "A1"],
        "L9",
        Color::Black,
        LocalThreatKind::OpenThree,
        &["I9", "M9", "N9"],
        &[],
    );
    assert_raw_fact_parity(
        &["J9", "N9", "K9", "A1"],
        "L9",
        Color::Black,
        LocalThreatKind::OpenThree,
        &["I9", "M9", "H9"],
        &[],
    );
    assert_raw_fact_parity(
        &["H8", "A1", "I8", "C1"],
        "K8",
        Color::Black,
        LocalThreatKind::BrokenThree,
        &["G8", "J8", "L8"],
        &["J8"],
    );
    assert_raw_fact_parity(
        &["I8", "A1", "K8", "C1"],
        "L8",
        Color::Black,
        LocalThreatKind::BrokenThree,
        &["H8", "J8", "M8"],
        &["J8"],
    );
}

#[test]
fn search_and_corridor_policies_treat_valid_broken_three_as_forcing() {
    let mut board = Board::new(RuleConfig::default());
    apply_moves(&mut board, &["H8", "A1", "K8", "C1"]);

    let annotation = SearchThreatPolicy.annotation_for_move(&board, mv("J8"));
    let broken_three = annotation
        .local_threats
        .iter()
        .find(|fact| fact.kind == LocalThreatKind::BrokenThree)
        .expect("search policy should retain broken-three material");
    assert!(SearchThreatPolicy.is_must_keep(broken_three));

    let mut existing = board.clone();
    existing.apply_move(mv("J8")).unwrap();
    let corridor_fact = raw_local_threat_facts_for_player(&existing, Color::Black)
        .into_iter()
        .find(|fact| fact.kind == LocalThreatKind::BrokenThree)
        .expect("corridor policy should see the existing broken three");
    assert!(CorridorThreatPolicy.is_active_threat(&existing, Color::Black, &corridor_fact));
    let continuations =
        legal_forcing_continuations_for_fact(&existing, Color::Black, &corridor_fact);
    assert_eq!(
        continuations
            .iter()
            .map(|continuation| continuation.mv)
            .collect::<Vec<_>>(),
        vec![mv("I8")]
    );
    assert_eq!(
        continuations[0].legal_cost_squares,
        vec![mv("G8"), mv("L8")]
    );
    assert_eq!(
        corridor_defender_reply_moves(&existing, Color::Black, None),
        vec![mv("G8"), mv("I8"), mv("L8")]
    );
}

#[test]
fn player_explicit_annotation_matches_current_player_annotation() {
    let board = board_from_moves(Variant::Renju, &["H8", "A1", "I8", "A2"]);
    let policy = SearchThreatPolicy;

    assert_eq!(
        policy.annotation_for_player(&board, Color::Black, mv("J8")),
        policy.annotation_for_move(&board, mv("J8"))
    );

    let mut white_turn = board.clone();
    white_turn.current_player = Color::White;
    assert_eq!(
        policy.annotation_for_player(&board, Color::White, mv("B2")),
        policy.annotation_for_move(&white_turn, mv("B2"))
    );
}

#[test]
fn known_legal_ordering_summary_matches_full_annotation_summary() {
    let policy = SearchThreatPolicy;

    let freestyle = board_from_moves(Variant::Freestyle, &["H8", "A1", "I8", "A2"]);
    let renju_white = board_from_moves(Variant::Renju, &["H8", "A1", "I8"]);
    let renju_forbidden_gap = board_from_moves(
        Variant::Renju,
        &["H8", "G8", "I8", "A1", "J8", "A2", "L8", "A3"],
    );

    let cases = [
        (&freestyle, Color::Black, mv("J8")),
        (&freestyle, Color::Black, mv("B2")),
        (&renju_white, Color::White, mv("B2")),
        (&renju_forbidden_gap, Color::Black, mv("M8")),
    ];

    for (board, player, probe) in cases {
        assert!(board.is_legal_for_color(probe, player));
        let annotation = policy.annotation_for_player(board, player, probe);
        assert_eq!(
            policy.ordering_summary_for_legal_player(board, player, probe),
            policy.ordering_summary(&annotation),
            "{player:?} {probe:?}"
        );
    }
}

#[test]
fn raw_known_legal_ordering_summary_matches_raw_annotation_summary() {
    let policy = SearchThreatPolicy;

    let freestyle = board_from_moves(Variant::Freestyle, &["H8", "A1", "I8", "A2"]);
    let renju_black_raw = board_from_moves(
        Variant::Renju,
        &["H8", "G8", "I8", "A1", "J8", "A2", "L8", "A3"],
    );

    for (board, player, probe) in [
        (&freestyle, Color::Black, mv("J8")),
        (&freestyle, Color::Black, mv("B2")),
        (&renju_black_raw, Color::Black, mv("M8")),
    ] {
        assert!(board.is_legal_for_color(probe, player));
        let raw_annotation = policy.raw_annotation_for_legal_player(board, player, probe);
        assert_eq!(
            policy.raw_ordering_summary_for_legal_player(board, player, probe),
            policy.ordering_summary(&raw_annotation),
            "{player:?} {probe:?}"
        );
    }
}

#[test]
fn scan_threat_view_matches_existing_corridor_queries() {
    let board = board_from_moves(Variant::Renju, &["H8", "A1", "I8", "A2", "J8", "A3", "C3"]);
    let view = ScanThreatView::new(&board);

    assert_eq!(
        view.active_corridor_threats(Color::Black),
        corridor_active_threats(&board, Color::Black)
    );
    assert_eq!(
        view.defender_reply_moves(Color::Black, None),
        corridor_defender_reply_moves(&board, Color::Black, None)
    );
    assert_eq!(
        view.has_move_local_corridor_entry(Color::Black, mv("J8")),
        has_forcing_local_threat_at_move(&board, Color::Black, mv("J8"))
    );
    assert_eq!(
        view.local_corridor_entry_rank(Color::Black, mv("J8")) > 0,
        has_forcing_local_threat_at_move(&board, Color::Black, mv("J8"))
    );
    assert!(
        !view.has_move_local_corridor_entry(Color::Black, mv("C3")),
        "quiet existing stones should not become corridor entries"
    );
}

#[test]
fn renju_black_forbidden_only_local_threat_gets_no_tactical_credit() {
    let board = board_from_moves(
        Variant::Renju,
        &["H8", "G8", "I8", "A1", "J8", "A2", "L8", "A3"],
    );
    assert!(board.is_legal_for_color(mv("M8"), Color::Black));

    let raw_facts = raw_local_threat_facts_after_move(&board, mv("M8"));
    assert!(
        raw_facts
            .iter()
            .any(|fact| fact.kind == LocalThreatKind::BrokenFour),
        "raw detector should preserve the forbidden-gap shape: {raw_facts:?}"
    );

    let raw_annotation =
        SearchThreatPolicy.raw_annotation_for_player(&board, Color::Black, mv("M8"));
    assert!(
        raw_annotation
            .local_threats
            .iter()
            .any(|fact| fact.kind == LocalThreatKind::BrokenFour),
        "raw annotation should preserve the forbidden-gap shape: {raw_annotation:?}"
    );

    let effective_annotation =
        SearchThreatPolicy.effective_annotation_from_raw(&board, raw_annotation);
    assert!(
        effective_annotation
            .local_threats
            .iter()
            .all(|fact| !SearchThreatPolicy.is_must_keep(fact)),
        "effective annotation should remove forbidden-only forcing threats: {effective_annotation:?}"
    );

    let facts = local_threat_facts_after_move(&board, mv("M8"));
    let search_policy = SearchThreatPolicy;
    assert!(
        facts.iter().all(|fact| !search_policy.is_must_keep(fact)),
        "forbidden-only local threat should not be forcing: {facts:?}"
    );
}

#[test]
fn renju_forbidden_only_existing_local_threat_is_not_forcing() {
    let board = board_from_moves(
        Variant::Renju,
        &["H8", "G8", "I8", "A1", "J8", "A2", "L8", "A3", "M8"],
    );
    assert!(!board.is_legal_for_color(mv("K8"), Color::Black));

    let facts = local_threat_facts_for_player(&board, Color::Black);
    let forbidden_gap_four = facts
        .iter()
        .find(|fact| {
            fact.kind == LocalThreatKind::BrokenFour && fact.defense_squares == vec![mv("K8")]
        })
        .unwrap_or_else(|| panic!("expected raw forbidden broken-four fact: {facts:?}"));
    assert!(
        legal_forcing_continuations_for_fact(&board, Color::Black, forbidden_gap_four).is_empty()
    );
    assert!(!has_forcing_local_threat(&board, Color::Black));
}

#[test]
fn localized_forcing_threat_gate_checks_only_requested_move() {
    let board = board_from_moves(
        Variant::Freestyle,
        &["H8", "A1", "I8", "A2", "J8", "A3", "C3"],
    );

    assert!(has_forcing_local_threat_at_move(
        &board,
        Color::Black,
        mv("J8")
    ));
    assert!(!has_forcing_local_threat_at_move(
        &board,
        Color::Black,
        mv("C3")
    ));
    assert!(!has_forcing_local_threat_at_move(
        &board,
        Color::White,
        mv("J8")
    ));
}
