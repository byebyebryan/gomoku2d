use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct OrderedMove {
    pub(super) mv: Move,
    pub(super) must_keep: bool,
}

pub(super) fn order_root_moves(
    state: &mut SearchState,
    moves: Vec<Move>,
    move_ordering: MoveOrdering,
    tt_move: Option<Move>,
    threat_view_mode: ThreatViewMode,
    metrics: &mut SearchMetrics,
) -> Vec<Move> {
    let stage_before = metrics.stage_snapshot();
    let start = Instant::now();
    let ordered = match move_ordering {
        MoveOrdering::TranspositionFirstBoardOrder => order_tt_first(moves, tt_move),
        MoveOrdering::TacticalFull | MoveOrdering::Tactical => order_moves_with_ordering(
            state,
            moves,
            tt_move,
            MoveOrderingOptions {
                move_ordering,
                child_limit: None,
                threat_view_mode,
                phase: SearchMetricPhase::Root,
            },
            metrics,
        )
        .into_iter()
        .map(|ordered| ordered.mv)
        .collect(),
    };
    metrics.record_ordering_scope(start.elapsed(), stage_before);
    ordered
}

pub(super) fn order_search_moves(
    state: &mut SearchState,
    moves: Vec<Move>,
    move_ordering: MoveOrdering,
    tt_move: Option<Move>,
    child_limit: Option<usize>,
    threat_view_mode: ThreatViewMode,
    metrics: &mut SearchMetrics,
) -> Vec<Move> {
    let stage_before = metrics.stage_snapshot();
    let start = Instant::now();
    let ordered = match move_ordering {
        MoveOrdering::TranspositionFirstBoardOrder => {
            let moves = order_tt_first(moves, tt_move);
            apply_plain_child_limit(moves, child_limit, metrics, SearchMetricPhase::Search)
        }
        MoveOrdering::TacticalFull => {
            let ordered = order_moves_tactical_full(
                state,
                moves,
                tt_move,
                threat_view_mode,
                metrics,
                SearchMetricPhase::Search,
            );
            apply_child_limit(ordered, child_limit, metrics, SearchMetricPhase::Search)
        }
        MoveOrdering::Tactical => {
            let ordered = order_moves_tactical(
                state,
                moves,
                tt_move,
                child_limit,
                threat_view_mode,
                metrics,
                SearchMetricPhase::Search,
            );
            apply_child_limit(ordered, child_limit, metrics, SearchMetricPhase::Search)
        }
    };
    metrics.record_ordering_scope(start.elapsed(), stage_before);
    ordered
}

pub(super) fn order_moves_with_ordering(
    state: &mut SearchState,
    moves: Vec<Move>,
    tt_move: Option<Move>,
    options: MoveOrderingOptions,
    metrics: &mut SearchMetrics,
) -> Vec<OrderedMove> {
    match options.move_ordering {
        MoveOrdering::TranspositionFirstBoardOrder => order_tt_first(moves, tt_move)
            .into_iter()
            .map(|mv| OrderedMove {
                mv,
                must_keep: false,
            })
            .collect(),
        MoveOrdering::TacticalFull => order_moves_tactical_full(
            state,
            moves,
            tt_move,
            options.threat_view_mode,
            metrics,
            options.phase,
        ),
        MoveOrdering::Tactical => order_moves_tactical(
            state,
            moves,
            tt_move,
            options.child_limit,
            options.threat_view_mode,
            metrics,
            options.phase,
        ),
    }
}

pub(super) fn order_tt_first(mut moves: Vec<Move>, tt_move: Option<Move>) -> Vec<Move> {
    let Some(tt_move) = tt_move else {
        return moves;
    };

    if let Some(index) = moves.iter().position(|&mv| mv == tt_move) {
        if index > 0 {
            let tt_move = moves.remove(index);
            moves.insert(0, tt_move);
        }
    }

    moves
}

pub(super) fn order_moves_tactical_full(
    state: &mut SearchState,
    moves: Vec<Move>,
    tt_move: Option<Move>,
    threat_view_mode: ThreatViewMode,
    metrics: &mut SearchMetrics,
    phase: SearchMetricPhase,
) -> Vec<OrderedMove> {
    if moves.is_empty() {
        return Vec::new();
    }

    let board_size = state.board().config.board_size;
    let opponent = state.board().current_player.opponent();
    let opponent_immediate_wins = immediate_winning_move_mask_for_threat_view_mode(
        state,
        opponent,
        threat_view_mode,
        metrics,
    );
    let mut scored = moves
        .into_iter()
        .enumerate()
        .map(|(index, mv)| {
            let summary =
                tactical_ordering_summary_counted(state, mv, threat_view_mode, metrics, phase);
            let immediate_block = move_mask_contains(&opponent_immediate_wins, board_size, mv);
            let (score, must_keep) = tactical_ordering_score_from_summary(summary, immediate_block);
            (index, mv, score, must_keep, Some(mv) == tt_move)
        })
        .collect::<Vec<_>>();

    scored.sort_by(|a, b| {
        b.2.cmp(&a.2)
            .then_with(|| b.4.cmp(&a.4))
            .then_with(|| a.0.cmp(&b.0))
    });
    scored
        .into_iter()
        .map(|(_, mv, _, must_keep, _)| OrderedMove { mv, must_keep })
        .collect()
}

pub(super) fn order_moves_tactical(
    state: &mut SearchState,
    moves: Vec<Move>,
    tt_move: Option<Move>,
    child_limit: Option<usize>,
    threat_view_mode: ThreatViewMode,
    metrics: &mut SearchMetrics,
    phase: SearchMetricPhase,
) -> Vec<OrderedMove> {
    if moves.is_empty() {
        return Vec::new();
    }

    if child_limit.is_none() {
        return order_moves_tactical_full(state, moves, tt_move, threat_view_mode, metrics, phase);
    }

    let board_size = state.board().config.board_size;
    let player = state.board().current_player;
    let opponent = player.opponent();
    let own_immediate_wins =
        immediate_winning_move_mask_for_threat_view_mode(state, player, threat_view_mode, metrics);
    let opponent_immediate_wins = immediate_winning_move_mask_for_threat_view_mode(
        state,
        opponent,
        threat_view_mode,
        metrics,
    );

    let mut scored = moves
        .into_iter()
        .enumerate()
        .map(|(index, mv)| {
            let own_win = move_mask_contains(&own_immediate_wins, board_size, mv);
            let immediate_block = move_mask_contains(&opponent_immediate_wins, board_size, mv);
            let (hard_score, hard_keep) = hard_tactical_ordering_score(own_win, immediate_block);
            let should_annotate = hard_keep
                || has_tactical_annotation_potential_for_mode(
                    state,
                    player,
                    mv,
                    threat_view_mode,
                    metrics,
                );
            (
                index,
                mv,
                hard_score,
                hard_keep,
                immediate_block,
                Some(mv) == tt_move,
                should_annotate,
            )
        })
        .collect::<Vec<_>>();

    for scored_move in scored.iter_mut() {
        if !scored_move.6 {
            continue;
        }

        let summary = tactical_ordering_summary_counted(
            state,
            scored_move.1,
            threat_view_mode,
            metrics,
            phase,
        );
        let (tactical_score, tactical_keep) =
            tactical_ordering_score_from_summary(summary, scored_move.4);
        if tactical_score > 0 || tactical_keep {
            scored_move.2 = tactical_score;
        }
        scored_move.3 |= tactical_keep;
    }

    scored.sort_by(|a, b| {
        b.2.cmp(&a.2)
            .then_with(|| b.5.cmp(&a.5))
            .then_with(|| a.0.cmp(&b.0))
    });
    scored
        .into_iter()
        .map(|(_, mv, _, must_keep, _, _, _)| OrderedMove { mv, must_keep })
        .collect()
}

pub(super) fn hard_tactical_ordering_score(own_win: bool, immediate_block: bool) -> (i32, bool) {
    if own_win {
        (100_000, true)
    } else if immediate_block {
        (90_000, true)
    } else {
        (0, false)
    }
}

pub(super) fn has_tactical_annotation_potential(board: &Board, player: Color, mv: Move) -> bool {
    let size = board.config.board_size;
    if size == 0 || mv.row >= size || mv.col >= size || !board.is_empty(mv.row, mv.col) {
        return false;
    }

    DIRS.iter()
        .any(|&(dr, dc)| axis_has_tactical_annotation_potential(board, player, mv, dr, dc))
}

pub(super) fn has_tactical_annotation_potential_for_mode(
    state: &SearchState,
    player: Color,
    mv: Move,
    mode: ThreatViewMode,
    metrics: &mut SearchMetrics,
) -> bool {
    let board = state.board();
    match mode {
        ThreatViewMode::Scan => has_tactical_annotation_potential(board, player, mv),
        ThreatViewMode::Rolling => {
            let viability_mask = state
                .frontier
                .as_ref()
                .map(|frontier| frontier.viability_for(mv).mask_for(player))
                .unwrap_or_else(|| scan_cell_viability(board, mv).mask_for(player));
            has_tactical_annotation_potential_with_mask(board, player, mv, viability_mask)
        }
        ThreatViewMode::RollingShadow => {
            let scan = has_tactical_annotation_potential(board, player, mv);
            if let Some(frontier) = state.frontier.as_ref() {
                metrics.threat_view_shadow_checks += 1;
                let rolling = has_tactical_annotation_potential_with_mask(
                    board,
                    player,
                    mv,
                    frontier.viability_for(mv).mask_for(player),
                );
                if scan != rolling {
                    metrics.threat_view_shadow_mismatches += 1;
                }
            }
            scan
        }
    }
}

pub(super) fn has_tactical_annotation_potential_with_mask(
    board: &Board,
    player: Color,
    mv: Move,
    viability_mask: u8,
) -> bool {
    let size = board.config.board_size;
    if viability_mask == 0
        || size == 0
        || mv.row >= size
        || mv.col >= size
        || !board.is_empty(mv.row, mv.col)
    {
        return false;
    }

    DIRS.iter()
        .enumerate()
        .filter(|(direction_index, _)| viability_mask & direction_bit(*direction_index) != 0)
        .any(|(_, &(dr, dc))| axis_has_tactical_annotation_potential(board, player, mv, dr, dc))
}

pub(super) fn axis_has_tactical_annotation_potential(
    board: &Board,
    player: Color,
    mv: Move,
    dr: isize,
    dc: isize,
) -> bool {
    let size = board.config.board_size as isize;
    let row = mv.row as isize;
    let col = mv.col as isize;
    let opponent = player.opponent();

    for start in -4..=0 {
        let mut own_count = 1;
        let mut clean_window = true;

        for offset in start..start + 5 {
            let r = row + dr * offset;
            let c = col + dc * offset;
            if r < 0 || r >= size || c < 0 || c >= size {
                clean_window = false;
                break;
            }

            let r = r as usize;
            let c = c as usize;
            if r == mv.row && c == mv.col {
                continue;
            }
            if board.has_color(r, c, opponent) {
                clean_window = false;
                break;
            }
            if board.has_color(r, c, player) {
                own_count += 1;
            }
        }

        if clean_window && own_count >= 3 {
            return true;
        }
    }

    false
}

pub(super) fn immediate_winning_move_mask_for_threat_view_mode(
    state: &mut SearchState,
    player: Color,
    mode: ThreatViewMode,
    metrics: &mut SearchMetrics,
) -> Vec<bool> {
    let size = state.board().config.board_size;
    let mut mask = vec![false; size * size];
    for mv in immediate_winning_moves_for_threat_view_mode(state, player, mode, metrics) {
        mask[mv.row * size + mv.col] = true;
    }
    mask
}

pub(super) fn move_mask_contains(mask: &[bool], board_size: usize, mv: Move) -> bool {
    mv.row < board_size && mv.col < board_size && mask[mv.row * board_size + mv.col]
}

#[cfg(test)]
pub(super) fn tactical_ordering_score(
    annotation: &TacticalMoveAnnotation,
    immediate_block: bool,
) -> (i32, bool) {
    let summary = SearchThreatPolicy.ordering_summary(annotation);
    tactical_ordering_score_from_summary(summary, immediate_block)
}

pub(super) fn tactical_ordering_score_from_summary(
    summary: TacticalOrderingSummary,
    immediate_block: bool,
) -> (i32, bool) {
    let score = if immediate_block {
        summary.score.max(90_000)
    } else {
        summary.score
    };
    let must_keep = summary.must_keep || immediate_block;

    (score, must_keep)
}

pub(super) fn apply_child_limit(
    ordered: Vec<OrderedMove>,
    child_limit: Option<usize>,
    metrics: &mut SearchMetrics,
    phase: SearchMetricPhase,
) -> Vec<Move> {
    let Some(limit) = child_limit else {
        return ordered.into_iter().map(|ordered| ordered.mv).collect();
    };
    let limit = limit.max(1);
    let before = ordered.len();
    let moves = ordered
        .into_iter()
        .enumerate()
        .filter_map(|(index, ordered)| {
            if index < limit || ordered.must_keep {
                Some(ordered.mv)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    metrics.record_child_limit(before, moves.len(), phase);
    moves
}

pub(super) fn apply_plain_child_limit(
    mut moves: Vec<Move>,
    child_limit: Option<usize>,
    metrics: &mut SearchMetrics,
    phase: SearchMetricPhase,
) -> Vec<Move> {
    let Some(limit) = child_limit else {
        return moves;
    };
    let limit = limit.max(1);
    let before = moves.len();
    moves.truncate(limit);
    metrics.record_child_limit(before, moves.len(), phase);
    moves
}
