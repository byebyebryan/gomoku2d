use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SearchOutcome {
    pub(super) score: i32,
    pub(super) best_move: Option<Move>,
    pub(super) timed_out: bool,
}

impl SearchOutcome {
    fn new(score: i32, best_move: Option<Move>, timed_out: bool) -> Self {
        Self {
            score,
            best_move,
            timed_out,
        }
    }
}

pub(super) fn store_tt_entry(
    tt: &mut HashMap<u64, TTEntry>,
    max_tt_entries: Option<usize>,
    metrics: &mut SearchMetrics,
    hash: u64,
    entry: TTEntry,
) {
    let can_insert = tt.contains_key(&hash) || max_tt_entries.is_none_or(|limit| tt.len() < limit);
    if can_insert {
        tt.insert(hash, entry);
    } else {
        metrics.tt_insert_skips += 1;
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn negamax(
    state: &mut SearchState,
    depth: i32,
    mut alpha: i32,
    beta: i32,
    color: Color,
    root_color: Color,
    tt: &mut HashMap<u64, TTEntry>,
    max_tt_entries: Option<usize>,
    zobrist: &ZobristTable,
    candidate_source: CandidateSource,
    null_cell_culling: NullCellCulling,
    legality_gate: LegalityGate,
    move_ordering: MoveOrdering,
    child_limit: Option<usize>,
    threat_view_mode: ThreatViewMode,
    static_eval: StaticEvaluation,
    nodes: &mut u64,
    metrics: &mut SearchMetrics,
    deadline: SearchDeadline,
) -> SearchOutcome {
    *nodes += 1;
    let hash = state.hash();

    if deadline.expired() {
        return SearchOutcome::new(
            evaluate_leaf_counted(state, color, root_color, static_eval, metrics),
            None,
            true,
        );
    }

    if let Some(entry) = tt.get(&hash) {
        metrics.tt_hits += 1;
        if entry.depth >= depth {
            match entry.flag {
                TTFlag::Exact => {
                    metrics.tt_cutoffs += 1;
                    return SearchOutcome::new(entry.score, entry.best_move, false);
                }
                TTFlag::LowerBound => {
                    if entry.score >= beta {
                        metrics.tt_cutoffs += 1;
                        return SearchOutcome::new(entry.score, entry.best_move, false);
                    }
                }
                TTFlag::UpperBound => {
                    if entry.score <= alpha {
                        metrics.tt_cutoffs += 1;
                        return SearchOutcome::new(entry.score, entry.best_move, false);
                    }
                }
            }
        }
    }

    if state.board().result != GameResult::Ongoing {
        return SearchOutcome::new(
            evaluate_leaf_counted(state, color, root_color, static_eval, metrics),
            None,
            false,
        );
    }

    if depth == 0 {
        return SearchOutcome::new(
            evaluate_leaf_counted(state, color, root_color, static_eval, metrics),
            None,
            false,
        );
    }

    let mut moves = candidate_moves_from_source_counted(
        state.board(),
        candidate_source,
        metrics,
        SearchMetricPhase::Search,
    );
    moves = cull_null_cells_counted(
        state.board(),
        state.frontier.as_ref(),
        moves,
        null_cell_culling,
        threat_view_mode,
        metrics,
        SearchMetricPhase::Search,
    );
    let mut needs_legality_check = needs_legality_gate(state.board(), color, legality_gate);
    if (matches!(
        move_ordering,
        MoveOrdering::TacticalFull | MoveOrdering::Tactical
    ) || child_limit.is_some())
        && needs_legality_check
    {
        moves.retain(|&mv| {
            legal_by_gate_counted(
                state.board(),
                mv,
                legality_gate,
                metrics,
                SearchMetricPhase::Search,
            )
        });
        needs_legality_check = false;
    }
    if moves.is_empty() {
        return SearchOutcome::new(
            evaluate_leaf_counted(state, color, root_color, static_eval, metrics),
            None,
            false,
        );
    }

    let orig_alpha = alpha;
    let mut best_score = i32::MIN + 1;
    let mut best_move: Option<Move> = None;

    let tt_move = tt.get(&hash).and_then(|e| e.best_move);
    let ordered = order_search_moves(
        state,
        moves,
        move_ordering,
        tt_move,
        child_limit,
        threat_view_mode,
        metrics,
    );

    let mut timed_out = false;
    for mv in ordered {
        if deadline.expired() {
            timed_out = true;
            break;
        }
        if needs_legality_check
            && !legal_by_gate_counted(
                state.board(),
                mv,
                legality_gate,
                metrics,
                SearchMetricPhase::Search,
            )
        {
            continue;
        }
        state.apply_trusted_legal_move_counted(mv, zobrist, metrics);
        let child_outcome = negamax(
            state,
            depth - 1,
            -beta,
            -alpha,
            color.opponent(),
            root_color,
            tt,
            max_tt_entries,
            zobrist,
            candidate_source,
            null_cell_culling,
            legality_gate,
            move_ordering,
            child_limit,
            threat_view_mode,
            static_eval,
            nodes,
            metrics,
            deadline,
        );
        let score = -child_outcome.score;
        state.undo_move_counted(mv, metrics);

        if child_outcome.timed_out {
            timed_out = true;
        }
        if score > best_score {
            best_score = score;
            best_move = Some(mv);
        }
        if score > alpha {
            alpha = score;
        }
        if alpha >= beta {
            metrics.beta_cutoffs += 1;
            break;
        }
        if timed_out {
            break;
        }
    }

    if best_move.is_none() {
        return SearchOutcome::new(
            evaluate_leaf_counted(state, color, root_color, static_eval, metrics),
            None,
            timed_out,
        );
    }

    if timed_out {
        return SearchOutcome {
            score: best_score,
            best_move,
            timed_out: true,
        };
    }

    let flag = if best_score <= orig_alpha {
        TTFlag::UpperBound
    } else if best_score >= beta {
        TTFlag::LowerBound
    } else {
        TTFlag::Exact
    };
    store_tt_entry(
        tt,
        max_tt_entries,
        metrics,
        hash,
        TTEntry {
            depth,
            score: best_score,
            flag,
            best_move,
        },
    );

    SearchOutcome {
        score: best_score,
        best_move,
        timed_out: false,
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn search_root(
    state: &mut SearchState,
    depth: i32,
    root_moves: &[Move],
    color: Color,
    tt: &mut HashMap<u64, TTEntry>,
    max_tt_entries: Option<usize>,
    zobrist: &ZobristTable,
    candidate_source: CandidateSource,
    null_cell_culling: NullCellCulling,
    legality_gate: LegalityGate,
    move_ordering: MoveOrdering,
    child_limit: Option<usize>,
    threat_view_mode: ThreatViewMode,
    static_eval: StaticEvaluation,
    nodes: &mut u64,
    metrics: &mut SearchMetrics,
    deadline: SearchDeadline,
    mut root_results: Option<&mut Vec<RootCandidateResult>>,
) -> SearchOutcome {
    *nodes += 1;
    let hash = state.hash();
    if deadline.expired() {
        return SearchOutcome::new(
            evaluate_state_counted(state, color, static_eval, metrics),
            None,
            true,
        );
    }

    let mut alpha = i32::MIN + 1;
    let beta = i32::MAX;
    let orig_alpha = alpha;
    let mut best_score = i32::MIN + 1;
    let mut best_move: Option<Move> = None;

    let tt_move = tt.get(&hash).and_then(|entry| entry.best_move);
    let ordered = order_root_moves(
        state,
        root_moves.to_vec(),
        move_ordering,
        tt_move,
        threat_view_mode,
        metrics,
    );

    let mut timed_out = false;
    for mv in ordered {
        if deadline.expired() {
            timed_out = true;
            break;
        }

        state.apply_trusted_legal_move_counted(mv, zobrist, metrics);
        let child_outcome = negamax(
            state,
            depth - 1,
            -beta,
            -alpha,
            color.opponent(),
            color,
            tt,
            max_tt_entries,
            zobrist,
            candidate_source,
            null_cell_culling,
            legality_gate,
            move_ordering,
            child_limit,
            threat_view_mode,
            static_eval,
            nodes,
            metrics,
            deadline,
        );
        let score = -child_outcome.score;
        state.undo_move_counted(mv, metrics);

        if let Some(results) = root_results.as_deref_mut() {
            results.push(RootCandidateResult { mv, score });
        }

        if child_outcome.timed_out {
            timed_out = true;
        }
        if score > best_score {
            best_score = score;
            best_move = Some(mv);
        }
        if score > alpha {
            alpha = score;
        }
        if alpha >= beta {
            metrics.beta_cutoffs += 1;
            break;
        }
        if timed_out {
            break;
        }
    }

    if best_move.is_none() {
        return SearchOutcome::new(
            evaluate_state_counted(state, color, static_eval, metrics),
            None,
            timed_out,
        );
    }

    if timed_out {
        return SearchOutcome {
            score: best_score,
            best_move,
            timed_out: true,
        };
    }

    let flag = if best_score <= orig_alpha {
        TTFlag::UpperBound
    } else if best_score >= beta {
        TTFlag::LowerBound
    } else {
        TTFlag::Exact
    };
    store_tt_entry(
        tt,
        max_tt_entries,
        metrics,
        hash,
        TTEntry {
            depth,
            score: best_score,
            flag,
            best_move,
        },
    );

    SearchOutcome {
        score: best_score,
        best_move,
        timed_out: false,
    }
}

// --- SearchBot ---
