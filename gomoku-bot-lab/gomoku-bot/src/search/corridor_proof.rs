use std::cmp::Reverse;

use gomoku_core::{Board, Color, GameResult, Move, ZobristTable};

use super::config::CorridorSide;
use super::{
    immediate_winning_moves_for_threat_view_mode,
    materialized_attacker_corridor_moves_for_threat_view_mode,
    narrow_corridor_reply_moves_for_threat_view_mode, CorridorProofConfig, SearchDeadline,
    SearchMetrics, SearchState, StaticEvaluation, ThreatViewMode,
};

pub(super) const TERMINAL_SCORE_THRESHOLD: i32 = 1_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct RootCandidateResult {
    pub(super) mv: Move,
    pub(super) score: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CorridorProofCandidate {
    mv: Move,
    rank: usize,
    score_gap: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateProofOutcome {
    ProvenWin,
    ProvenLoss,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CorridorProofCandidateResult {
    mv: Move,
    outcome: CandidateProofOutcome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CorridorProofDecisionReason {
    NoChange,
    ConfirmedWin,
    ChangedToWin,
    AvoidedLoss,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CorridorProofDecision {
    pub(super) best_move: Move,
    pub(super) reason: CorridorProofDecisionReason,
}

fn select_corridor_proof_candidates(
    root_results: &[RootCandidateResult],
    best_move: Move,
    max_candidates: usize,
) -> Vec<CorridorProofCandidate> {
    if max_candidates == 0 {
        return Vec::new();
    }

    let mut ranked = root_results.to_vec();
    ranked.sort_by_key(|result| Reverse(result.score));

    let Some(best_score) = ranked
        .iter()
        .find(|result| result.mv == best_move)
        .map(|result| result.score)
    else {
        return Vec::new();
    };

    let to_candidate = |rank: usize, result: RootCandidateResult| CorridorProofCandidate {
        mv: result.mv,
        rank,
        score_gap: best_score.saturating_sub(result.score).max(0) as u64,
    };

    let Some(best_candidate) = ranked
        .iter()
        .copied()
        .enumerate()
        .find(|(_, result)| result.mv == best_move)
        .map(|(index, result)| to_candidate(index + 1, result))
    else {
        return Vec::new();
    };

    let mut selected = Vec::with_capacity(max_candidates.min(root_results.len()));
    selected.push(best_candidate);
    for (index, result) in ranked.into_iter().enumerate() {
        if result.mv == best_move {
            continue;
        }
        let candidate = to_candidate(index + 1, result);
        if selected.len() >= max_candidates {
            break;
        }
        selected.push(candidate);
    }
    selected
}

fn resolve_corridor_proof_candidates(
    normal_best: Move,
    proofs: &[CorridorProofCandidateResult],
) -> CorridorProofDecision {
    if proofs
        .iter()
        .any(|proof| proof.mv == normal_best && proof.outcome == CandidateProofOutcome::ProvenWin)
    {
        return CorridorProofDecision {
            best_move: normal_best,
            reason: CorridorProofDecisionReason::ConfirmedWin,
        };
    }

    if let Some(proof) = proofs
        .iter()
        .find(|proof| proof.outcome == CandidateProofOutcome::ProvenWin)
    {
        return CorridorProofDecision {
            best_move: proof.mv,
            reason: CorridorProofDecisionReason::ChangedToWin,
        };
    }

    let normal_best_is_loss = proofs
        .iter()
        .any(|proof| proof.mv == normal_best && proof.outcome == CandidateProofOutcome::ProvenLoss);
    if normal_best_is_loss {
        if let Some(proof) = proofs.iter().find(|proof| {
            proof.mv != normal_best && proof.outcome != CandidateProofOutcome::ProvenLoss
        }) {
            return CorridorProofDecision {
                best_move: proof.mv,
                reason: CorridorProofDecisionReason::AvoidedLoss,
            };
        }
    }

    CorridorProofDecision {
        best_move: normal_best,
        reason: CorridorProofDecisionReason::NoChange,
    }
}

pub(super) fn terminal_score_for_winner(winner: Color, color: Color, root_color: Color) -> i32 {
    let root_score = if winner == root_color {
        2_000_000
    } else {
        -2_000_000
    };
    if color == root_color {
        root_score
    } else {
        -root_score
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn run_corridor_proof_pass(
    board: &Board,
    root_color: Color,
    normal_best: Move,
    root_results: &[RootCandidateResult],
    corridor_proof: CorridorProofConfig,
    threat_view_mode: ThreatViewMode,
    zobrist: &ZobristTable,
    metrics: &mut SearchMetrics,
    deadline: SearchDeadline,
) -> CorridorProofDecision {
    let candidates = select_corridor_proof_candidates(
        root_results,
        normal_best,
        corridor_proof.proof_candidate_limit,
    );
    let mut proofs = Vec::with_capacity(candidates.len());

    for candidate in candidates {
        if deadline.expired() {
            metrics.corridor_proof_deadline_skips += 1;
            metrics.corridor_proof_deadline_exits += 1;
            break;
        }

        let mv = candidate.mv;
        metrics.corridor_proof_candidates_considered += 1;
        metrics.corridor_proof_candidate_rank_total += candidate.rank as u64;
        metrics.corridor_proof_candidate_rank_max = metrics
            .corridor_proof_candidate_rank_max
            .max(candidate.rank as u64);
        metrics.corridor_proof_candidate_score_gap_total += candidate.score_gap;
        metrics.corridor_proof_candidate_score_gap_max = metrics
            .corridor_proof_candidate_score_gap_max
            .max(candidate.score_gap);
        let outcome = prove_corridor_candidate(
            board,
            root_color,
            mv,
            corridor_proof,
            threat_view_mode,
            zobrist,
            metrics,
            deadline,
        );
        match outcome {
            CandidateProofOutcome::ProvenWin => {
                metrics.corridor_proof_wins += 1;
                metrics.corridor_proof_win_candidate_rank_total += candidate.rank as u64;
                metrics.corridor_proof_win_candidate_rank_max = metrics
                    .corridor_proof_win_candidate_rank_max
                    .max(candidate.rank as u64);
                metrics.corridor_proof_terminal_root_candidates += 1;
                metrics.corridor_proof_terminal_root_winning_candidates += 1;
            }
            CandidateProofOutcome::ProvenLoss => {
                metrics.corridor_proof_losses += 1;
                metrics.corridor_proof_terminal_root_candidates += 1;
                metrics.corridor_proof_terminal_root_losing_candidates += 1;
            }
            CandidateProofOutcome::Unknown => {
                metrics.corridor_proof_unknown += 1;
            }
        }

        proofs.push(CorridorProofCandidateResult { mv, outcome });
        if outcome == CandidateProofOutcome::ProvenWin {
            break;
        }
    }

    resolve_corridor_proof_candidates(normal_best, &proofs)
}

#[allow(clippy::too_many_arguments)]
fn prove_corridor_candidate(
    board: &Board,
    root_color: Color,
    mv: Move,
    corridor_proof: CorridorProofConfig,
    threat_view_mode: ThreatViewMode,
    zobrist: &ZobristTable,
    metrics: &mut SearchMetrics,
    deadline: SearchDeadline,
) -> CandidateProofOutcome {
    let mut state = SearchState::from_board_for_config(
        board.clone(),
        zobrist,
        threat_view_mode,
        StaticEvaluation::LineShapeEval,
        corridor_proof,
    );
    let result = state.apply_trusted_legal_move_counted(mv, zobrist, metrics);
    let outcome = match result {
        GameResult::Winner(winner) if winner == root_color => CandidateProofOutcome::ProvenWin,
        GameResult::Winner(_) => CandidateProofOutcome::ProvenLoss,
        GameResult::Draw => CandidateProofOutcome::Unknown,
        GameResult::Ongoing => {
            let color = state.board().current_player;
            if let Some(attacker) =
                corridor_proof_attacker(&mut state, color, threat_view_mode, metrics)
            {
                metrics.record_corridor_proof_check(true);
                let winner = prove_corridor_for_attacker(
                    &mut state,
                    color,
                    attacker,
                    CorridorSide::for_player(attacker, root_color),
                    corridor_proof,
                    0,
                    threat_view_mode,
                    zobrist,
                    metrics,
                    deadline,
                );
                match winner {
                    Some(winner) if winner == root_color => CandidateProofOutcome::ProvenWin,
                    Some(_) => CandidateProofOutcome::ProvenLoss,
                    None => CandidateProofOutcome::Unknown,
                }
            } else {
                metrics.record_corridor_proof_check(false);
                CandidateProofOutcome::Unknown
            }
        }
    };
    state.undo_move_counted(mv, metrics);
    outcome
}

fn corridor_proof_attacker(
    state: &mut SearchState,
    color: Color,
    threat_view_mode: ThreatViewMode,
    metrics: &mut SearchMetrics,
) -> Option<Color> {
    let opponent = color.opponent();
    if !immediate_winning_moves_for_threat_view_mode(state, color, threat_view_mode, metrics)
        .is_empty()
    {
        return Some(color);
    }
    if !immediate_winning_moves_for_threat_view_mode(state, opponent, threat_view_mode, metrics)
        .is_empty()
    {
        return Some(opponent);
    }
    if !narrow_corridor_reply_moves_for_threat_view_mode(state, opponent, threat_view_mode, metrics)
        .is_empty()
    {
        return Some(opponent);
    }
    if !narrow_corridor_reply_moves_for_threat_view_mode(state, color, threat_view_mode, metrics)
        .is_empty()
    {
        return Some(color);
    }
    None
}

#[allow(clippy::too_many_arguments)]
fn prove_corridor_for_attacker(
    state: &mut SearchState,
    color: Color,
    attacker: Color,
    side: CorridorSide,
    corridor_proof: CorridorProofConfig,
    depth_used: usize,
    threat_view_mode: ThreatViewMode,
    zobrist: &ZobristTable,
    metrics: &mut SearchMetrics,
    deadline: SearchDeadline,
) -> Option<Color> {
    metrics.record_corridor_node(depth_used as u32);

    if deadline.expired() {
        metrics.corridor_proof_deadline_exits += 1;
        return None;
    }

    if let GameResult::Winner(winner) = state.board().result {
        metrics.corridor_terminal_exits += 1;
        metrics.corridor_proof_terminal_exits += 1;
        return Some(winner);
    }
    if state.board().result == GameResult::Draw {
        metrics.corridor_neutral_exits += 1;
        metrics.corridor_proof_static_exits += 1;
        return None;
    }

    if depth_used >= corridor_proof.max_depth {
        metrics.corridor_depth_exits += 1;
        metrics.corridor_proof_depth_exits += 1;
        return None;
    }

    let moves = if color == attacker {
        materialized_attacker_corridor_moves_for_threat_view_mode(
            state,
            attacker,
            threat_view_mode,
            metrics,
        )
    } else {
        let replies = narrow_corridor_reply_moves_for_threat_view_mode(
            state,
            attacker,
            threat_view_mode,
            metrics,
        );
        if replies.len() > corridor_proof.max_reply_width {
            metrics.corridor_width_exits += 1;
            metrics.corridor_proof_static_exits += 1;
            return None;
        }
        if replies.is_empty()
            && !immediate_winning_moves_for_threat_view_mode(
                state,
                attacker,
                threat_view_mode,
                metrics,
            )
            .is_empty()
        {
            metrics.corridor_terminal_exits += 1;
            metrics.corridor_proof_terminal_exits += 1;
            return Some(attacker);
        }
        replies
    };

    if moves.is_empty() {
        metrics.corridor_neutral_exits += 1;
        metrics.corridor_proof_static_exits += 1;
        return None;
    }

    metrics.corridor_branch_probes += moves.len() as u64;
    if color == attacker {
        for mv in moves {
            if deadline.expired() {
                metrics.corridor_proof_deadline_exits += 1;
                return None;
            }
            state.apply_trusted_legal_move_counted(mv, zobrist, metrics);
            metrics.record_corridor_ply(side);
            let winner = prove_corridor_for_attacker(
                state,
                color.opponent(),
                attacker,
                side,
                corridor_proof,
                depth_used + 1,
                threat_view_mode,
                zobrist,
                metrics,
                deadline,
            );
            state.undo_move_counted(mv, metrics);
            if winner == Some(attacker) {
                return Some(attacker);
            }
        }
        None
    } else {
        for mv in moves {
            if deadline.expired() {
                metrics.corridor_proof_deadline_exits += 1;
                return None;
            }
            state.apply_trusted_legal_move_counted(mv, zobrist, metrics);
            metrics.record_corridor_ply(side);
            let winner = prove_corridor_for_attacker(
                state,
                color.opponent(),
                attacker,
                side,
                corridor_proof,
                depth_used + 1,
                threat_view_mode,
                zobrist,
                metrics,
                deadline,
            );
            state.undo_move_counted(mv, metrics);
            if winner != Some(attacker) {
                return None;
            }
        }
        Some(attacker)
    }
}

#[cfg(test)]
mod tests {
    use gomoku_core::Move;

    use super::*;

    fn mv(notation: &str) -> Move {
        Move::from_notation(notation).unwrap()
    }

    #[test]
    fn selects_normal_best_then_ranked_candidates() {
        let best = mv("H8");
        let close = mv("H9");
        let also_close = mv("H10");
        let too_far = mv("H11");
        let results = vec![
            RootCandidateResult {
                mv: close,
                score: 960_000,
            },
            RootCandidateResult {
                mv: best,
                score: 1_000_000,
            },
            RootCandidateResult {
                mv: too_far,
                score: 900_000,
            },
            RootCandidateResult {
                mv: also_close,
                score: 955_000,
            },
        ];

        let selected = select_corridor_proof_candidates(&results, best, 4)
            .into_iter()
            .map(|candidate| candidate.mv)
            .collect::<Vec<_>>();

        assert_eq!(selected, vec![best, close, also_close, too_far]);
    }

    #[test]
    fn selects_top_candidates_without_score_margin() {
        let best = mv("H8");
        let second = mv("H9");
        let third = mv("H10");
        let fourth = mv("H11");
        let results = vec![
            RootCandidateResult {
                mv: fourth,
                score: -250_000,
            },
            RootCandidateResult {
                mv: best,
                score: 1_000_000,
            },
            RootCandidateResult {
                mv: third,
                score: 100_000,
            },
            RootCandidateResult {
                mv: second,
                score: 200_000,
            },
        ];

        let selected = select_corridor_proof_candidates(&results, best, 4);

        assert_eq!(
            selected
                .iter()
                .map(|candidate| candidate.mv)
                .collect::<Vec<_>>(),
            vec![best, second, third, fourth]
        );
        assert_eq!(
            selected
                .iter()
                .map(|candidate| candidate.rank)
                .collect::<Vec<_>>(),
            vec![1, 2, 3, 4]
        );
        assert_eq!(
            selected
                .iter()
                .map(|candidate| candidate.score_gap)
                .collect::<Vec<_>>(),
            vec![0, 800_000, 900_000, 1_250_000]
        );
    }

    #[test]
    fn resolution_confirms_normal_best_win() {
        let best = mv("H8");
        let proof = CorridorProofCandidateResult {
            mv: best,
            outcome: CandidateProofOutcome::ProvenWin,
        };

        let decision = resolve_corridor_proof_candidates(best, &[proof]);

        assert_eq!(decision.best_move, best);
        assert_eq!(decision.reason, CorridorProofDecisionReason::ConfirmedWin);
    }

    #[test]
    fn resolution_switches_to_proven_win() {
        let best = mv("H8");
        let proven = mv("J8");
        let proofs = vec![
            CorridorProofCandidateResult {
                mv: best,
                outcome: CandidateProofOutcome::Unknown,
            },
            CorridorProofCandidateResult {
                mv: proven,
                outcome: CandidateProofOutcome::ProvenWin,
            },
        ];

        let decision = resolve_corridor_proof_candidates(best, &proofs);

        assert_eq!(decision.best_move, proven);
        assert_eq!(decision.reason, CorridorProofDecisionReason::ChangedToWin);
    }

    #[test]
    fn resolution_escapes_proven_loss_to_unknown() {
        let best = mv("H8");
        let fallback = mv("J8");
        let proofs = vec![
            CorridorProofCandidateResult {
                mv: best,
                outcome: CandidateProofOutcome::ProvenLoss,
            },
            CorridorProofCandidateResult {
                mv: fallback,
                outcome: CandidateProofOutcome::Unknown,
            },
        ];

        let decision = resolve_corridor_proof_candidates(best, &proofs);

        assert_eq!(decision.best_move, fallback);
        assert_eq!(decision.reason, CorridorProofDecisionReason::AvoidedLoss);
    }
}
