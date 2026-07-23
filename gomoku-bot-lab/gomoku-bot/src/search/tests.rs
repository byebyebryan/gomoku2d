use super::*;
use crate::tactical::ScanThreatView;
use gomoku_core::RuleConfig;
use gomoku_lab_support::scenarios;

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
        id: "search_d3_completes_open_four",
        scenario_id: "local_complete_open_four",
        config_id: "search-d3",
        expected_moves: &["G8", "L8"],
        description: "search-d3 should finish its own open four.",
    },
    SearchBehaviorCase {
        id: "search_d3_reacts_closed_four",
        scenario_id: "local_react_closed_four",
        config_id: "search-d3",
        expected_moves: &["E1"],
        description: "search-d3 should answer the opponent's closed four.",
    },
    SearchBehaviorCase {
        id: "search_d3_prevents_open_four_over_extending_three",
        scenario_id: "priority_prevent_open_four_over_extend_three",
        config_id: "search-d3",
        expected_moves: &["G8", "K8"],
        description:
            "search-d3 should prevent the opponent's open three instead of extending elsewhere.",
    },
    SearchBehaviorCase {
        id: "search_d3_completes_four_before_reacting",
        scenario_id: "priority_complete_open_four_over_react_closed_four",
        config_id: "search-d3",
        expected_moves: &["G8", "L8"],
        description: "search-d3 should complete an open four when both sides threaten.",
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
        double_threat: immediate_wins_after >= 2 && immediate_wins_after > immediate_wins_before,
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
    let mut optimized =
        candidate_moves_from_source_counted(&board, source, &mut metrics, SearchMetricPhase::Root);
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
            let summary = SearchThreatPolicy.ordering_summary_for_legal_player(&board, player, mv);
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
        let mut scan_state = SearchState::from_board_with_frontier(board.clone(), &zobrist, false);
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
        112, 111, 127, 126, 97, 142, 113, 141, 82, 67, 96, 110, 94, 156, 171, 95, 128, 80, 65, 140,
        125, 139, 143, 138,
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
        112, 111, 127, 126, 97, 142, 113, 141, 82, 67, 96, 110, 94, 156, 171, 95, 128, 80, 65, 140,
        125, 139, 143, 138,
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
            "G8", "D8", "L8", "I8", "H7", "H4", "H12", "H9", "G7", "D4", "L12", "I9", "G9", "D12",
            "L4", "I7",
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
            "search-d3" => SearchBotConfig::custom_depth(3),
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
