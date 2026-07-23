use super::*;

pub(super) fn materialized_attacker_corridor_moves_for_threat_view_mode(
    state: &mut SearchState,
    attacker: Color,
    mode: ThreatViewMode,
    metrics: &mut SearchMetrics,
) -> Vec<Move> {
    match mode {
        ThreatViewMode::Scan => {
            scan_materialized_attacker_corridor_moves_timed(state.board(), attacker, metrics)
        }
        ThreatViewMode::Rolling => {
            rolling_materialized_attacker_corridor_moves(state, attacker, metrics)
        }
        ThreatViewMode::RollingShadow => {
            metrics.threat_view_shadow_checks += 1;
            let scan =
                scan_materialized_attacker_corridor_moves_timed(state.board(), attacker, metrics);
            let rolling = rolling_materialized_attacker_corridor_moves(state, attacker, metrics);
            if rolling != scan {
                metrics.threat_view_shadow_mismatches += 1;
            }
            scan
        }
    }
}

pub(super) fn scan_materialized_attacker_corridor_moves_timed(
    board: &Board,
    attacker: Color,
    metrics: &mut SearchMetrics,
) -> Vec<Move> {
    let start = Instant::now();
    let moves = scan_materialized_attacker_corridor_moves(board, attacker);
    metrics.record_threat_view_scan(start.elapsed());
    moves
}

pub(super) fn scan_materialized_attacker_corridor_moves(
    board: &Board,
    attacker: Color,
) -> Vec<Move> {
    if board.current_player != attacker || board.result != GameResult::Ongoing {
        return Vec::new();
    }

    let mut ranked = Vec::new();
    for mv in board.legal_moves() {
        let mut next = board.clone();
        if next.apply_move(mv).is_err() {
            continue;
        }
        let rank = match next.result {
            GameResult::Winner(winner) if winner == attacker => {
                CorridorThreatPolicy.rank(LocalThreatKind::Five)
            }
            GameResult::Winner(_) | GameResult::Draw => 0,
            GameResult::Ongoing => {
                ScanThreatView::new(&next).local_corridor_entry_rank(attacker, mv)
            }
        };

        if rank > 0 {
            ranked.push((mv, rank));
        }
    }

    highest_ranked_moves(ranked)
}

pub(super) fn rolling_materialized_attacker_corridor_moves(
    state: &mut SearchState,
    attacker: Color,
    metrics: &mut SearchMetrics,
) -> Vec<Move> {
    if state.board().current_player != attacker || state.board().result != GameResult::Ongoing {
        return Vec::new();
    }

    let mut ranked = Vec::new();
    for mv in state.board().legal_moves() {
        let start = Instant::now();
        let rank = state
            .threat_view()
            .candidate_corridor_entry_rank(attacker, mv);
        metrics.record_threat_view_frontier_query(start.elapsed());
        if rank > 0 {
            ranked.push((mv, rank));
        }
    }

    highest_ranked_moves(ranked)
}

pub(super) fn highest_ranked_moves(mut ranked: Vec<(Move, u8)>) -> Vec<Move> {
    let Some(best_rank) = ranked.iter().map(|(_, rank)| *rank).max() else {
        return Vec::new();
    };
    ranked.retain(|(_, rank)| *rank == best_rank);
    ranked.sort_by_key(|(mv, _)| (mv.row, mv.col));
    ranked.into_iter().map(|(mv, _)| mv).collect()
}

pub(super) fn narrow_corridor_reply_moves_for_threat_view_mode(
    state: &mut SearchState,
    attacker: Color,
    mode: ThreatViewMode,
    metrics: &mut SearchMetrics,
) -> Vec<Move> {
    match mode {
        ThreatViewMode::Scan => {
            scan_narrow_corridor_reply_moves_timed(state.board(), attacker, metrics)
        }
        ThreatViewMode::Rolling => rolling_narrow_corridor_reply_moves(state, attacker, metrics),
        ThreatViewMode::RollingShadow => {
            metrics.threat_view_shadow_checks += 1;
            let scan = scan_narrow_corridor_reply_moves_timed(state.board(), attacker, metrics);
            let rolling = rolling_narrow_corridor_reply_moves(state, attacker, metrics);
            if rolling != scan {
                metrics.threat_view_shadow_mismatches += 1;
            }
            scan
        }
    }
}

pub(super) fn immediate_winning_moves_for_threat_view_mode(
    state: &mut SearchState,
    player: Color,
    mode: ThreatViewMode,
    metrics: &mut SearchMetrics,
) -> Vec<Move> {
    match mode {
        ThreatViewMode::Scan => scan_immediate_winning_moves_timed(state.board(), player, metrics),
        ThreatViewMode::Rolling => rolling_immediate_winning_moves_timed(state, player, metrics),
        ThreatViewMode::RollingShadow => {
            metrics.threat_view_shadow_checks += 1;
            let scan = scan_immediate_winning_moves_timed(state.board(), player, metrics);
            let rolling = rolling_immediate_winning_moves_timed(state, player, metrics);
            if rolling != scan {
                metrics.threat_view_shadow_mismatches += 1;
            }
            scan
        }
    }
}

pub(super) fn scan_immediate_winning_moves_timed(
    board: &Board,
    player: Color,
    metrics: &mut SearchMetrics,
) -> Vec<Move> {
    let renju_before = renju_forbidden_metrics_snapshot();
    let start = Instant::now();
    let moves = board.immediate_winning_moves_for(player);
    metrics.record_threat_view_scan(start.elapsed());
    metrics.record_renju_forbidden_source_delta(RenjuForbiddenMetricSource::Threat, renju_before);
    moves
}

pub(super) fn rolling_immediate_winning_moves_timed(
    state: &mut SearchState,
    player: Color,
    metrics: &mut SearchMetrics,
) -> Vec<Move> {
    let renju_before = renju_forbidden_metrics_snapshot();
    let start = Instant::now();
    let moves = state
        .threat_view_mut()
        .immediate_winning_moves_for_cached(player);
    metrics.record_threat_view_frontier_immediate_win_query(start.elapsed());
    metrics.record_renju_forbidden_source_delta(RenjuForbiddenMetricSource::Threat, renju_before);
    moves
}

pub(super) fn scan_narrow_corridor_reply_moves_timed(
    board: &Board,
    attacker: Color,
    metrics: &mut SearchMetrics,
) -> Vec<Move> {
    let renju_before = renju_forbidden_metrics_snapshot();
    let start = Instant::now();
    let moves = corridor::narrow_corridor_reply_moves(board, attacker);
    metrics.record_threat_view_scan(start.elapsed());
    metrics.record_renju_forbidden_source_delta(RenjuForbiddenMetricSource::Threat, renju_before);
    moves
}

pub(super) fn rolling_narrow_corridor_reply_moves(
    state: &mut SearchState,
    attacker: Color,
    metrics: &mut SearchMetrics,
) -> Vec<Move> {
    let defender = attacker.opponent();
    let winning_squares = rolling_immediate_winning_moves_timed(state, attacker, metrics);
    if !winning_squares.is_empty() {
        let mut replies = Vec::new();
        for mv in winning_squares {
            let renju_before = renju_forbidden_metrics_snapshot();
            if state.board().is_legal_for_color(mv, defender) {
                push_unique_move(&mut replies, mv);
            }
            metrics.record_renju_forbidden_source_delta(
                RenjuForbiddenMetricSource::Threat,
                renju_before,
            );
        }
        for mv in rolling_immediate_winning_moves_timed(state, defender, metrics) {
            push_unique_move(&mut replies, mv);
        }
        return replies;
    }

    let renju_before = renju_forbidden_metrics_snapshot();
    let start = Instant::now();
    let replies = state.threat_view().defender_reply_moves(attacker, None);
    metrics.record_threat_view_frontier_query(start.elapsed());
    metrics.record_renju_forbidden_source_delta(RenjuForbiddenMetricSource::Threat, renju_before);
    replies
}

pub(super) fn push_unique_move(moves: &mut Vec<Move>, mv: Move) {
    if !moves.contains(&mv) {
        moves.push(mv);
    }
}

// --- Negamax with alpha-beta (incremental Zobrist hash) ---
