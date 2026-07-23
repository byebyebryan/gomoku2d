use super::*;

pub(super) fn root_candidate_moves_with_metrics(
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

pub(super) fn apply_safety_gate_to_root_candidates(
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
pub(super) struct SafetyFilterOutcome {
    moves: Vec<Move>,
    work_units: u64,
}

pub(super) fn current_obligation_root_candidates(
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

pub(super) fn current_obligation_safety_policy(
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

pub(super) fn rolling_current_obligation_safety_policy(
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

pub(super) fn immediate_win_safety_outcome(
    moves: &[Move],
    winning_moves: Vec<Move>,
) -> Option<SafetyFilterOutcome> {
    let wins = moves_in_set(moves, &winning_moves);
    (!wins.is_empty()).then(|| SafetyFilterOutcome {
        moves: filtered_or_original(moves, wins),
        work_units: moves.len() as u64,
    })
}

pub(super) fn current_obligation_safety_policy_after_immediate(
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

pub(super) fn moves_in_set(moves: &[Move], set: &[Move]) -> Vec<Move> {
    moves
        .iter()
        .copied()
        .filter(|mv| set.contains(mv))
        .collect()
}

pub(super) fn filtered_or_original(original: &[Move], filtered: Vec<Move>) -> Vec<Move> {
    if filtered.is_empty() {
        original.to_vec()
    } else {
        filtered
    }
}

pub(super) fn creates_counter_four(annotation: TacticalMoveAnnotation) -> bool {
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
