use crate::replay::{find_last_chance, proof_at, proof_has_limit_hit};
use crate::trace::defender_reply_roles_for_move;
use crate::types::*;
use crate::util::push_unique_move;
use gomoku_core::{Board, Color, Move, Replay};

pub(crate) struct FailureAnalysisInput<'a> {
    pub(crate) replay: &'a Replay,
    pub(crate) boards: &'a [Board],
    pub(crate) proof_summary: &'a [ProofResult],
    pub(crate) scan_start: usize,
    pub(crate) final_forced_interval_found: bool,
    pub(crate) final_forced_interval: &'a ForcedInterval,
    pub(crate) lethal_onset: Option<&'a LethalOnset>,
    pub(crate) root_cause: RootCause,
    pub(crate) winner: Color,
    pub(crate) loser: Color,
}

pub(crate) fn failure_analysis(input: FailureAnalysisInput<'_>) -> Option<FailureAnalysis> {
    if let Some(failure) = missed_immediate_win_failure(&input) {
        return Some(failure);
    }
    if let Some(failure) = missed_response_failure(&input) {
        return Some(failure);
    }
    if let Some(failure) = missed_lethal_prevention_failure(&input) {
        return Some(failure);
    }
    if let Some(failure) = missed_escape_failure(&input) {
        return Some(failure);
    }

    let mode = FailureMode::Unclear;
    let confidence = match mode {
        FailureMode::Unclear => FailureConfidence::Unclear,
        _ => FailureConfidence::Confirmed,
    };
    Some(FailureAnalysis {
        mode,
        side: input.loser,
        prefix_ply: input
            .lethal_onset
            .map(|onset| onset.prefix_ply)
            .or(Some(input.final_forced_interval.start_ply)),
        actual_move: None,
        actual_notation: None,
        missed_candidates: Vec::new(),
        prevented_onset_ply: input.lethal_onset.map(|onset| onset.prefix_ply),
        confidence,
    })
}

fn missed_immediate_win_failure(input: &FailureAnalysisInput<'_>) -> Option<FailureAnalysis> {
    let prefix_ply = input.final_forced_interval.start_ply.checked_sub(1)?;
    let board = input.boards.get(prefix_ply)?;
    if board.current_player != input.loser {
        return None;
    }
    let mut immediate_wins = board.immediate_winning_moves_for(input.loser);
    if immediate_wins.is_empty() {
        return None;
    }
    normalize_moves(&mut immediate_wins);
    let actual_move = replay_move_at(input.replay, prefix_ply)?;
    if immediate_wins.contains(&actual_move) {
        return None;
    }

    Some(FailureAnalysis {
        mode: FailureMode::MissedImmediateWin,
        side: input.loser,
        prefix_ply: Some(prefix_ply),
        actual_move: Some(actual_move),
        actual_notation: Some(actual_move.to_notation()),
        missed_candidates: immediate_wins
            .into_iter()
            .map(|mv| {
                missed_candidate(
                    mv,
                    vec![DefenderReplyRole::OffensiveCounter],
                    MissedCandidateOutcome::ConfirmedEscape,
                )
            })
            .collect(),
        prevented_onset_ply: input.lethal_onset.map(|onset| onset.prefix_ply),
        confidence: FailureConfidence::Confirmed,
    })
}

fn missed_response_failure(input: &FailureAnalysisInput<'_>) -> Option<FailureAnalysis> {
    let max_prefix = failure_blame_cutoff(input)?.checked_sub(1)?;
    for prefix_ply in (input.scan_start..=max_prefix).rev() {
        let board = input.boards.get(prefix_ply)?;
        if board.current_player != input.loser {
            continue;
        }
        let proof = proof_at(input.proof_summary, input.scan_start, prefix_ply)?;
        if proof.status != ProofStatus::EscapeFound {
            continue;
        }
        let actual_move = replay_move_at(input.replay, prefix_ply)?;
        let mut candidates = escape_candidates_from_proof(board, input.winner, proof, prefix_ply);
        candidates.retain(|candidate| candidate.mv != actual_move);
        if candidates.is_empty() {
            continue;
        }

        let actual_roles = failure_candidate_roles(board, input.winner, actual_move);
        if is_response_role_set(&actual_roles) {
            continue;
        }

        let immediate = candidates
            .iter()
            .filter(|candidate| {
                candidate
                    .roles
                    .contains(&DefenderReplyRole::ImmediateDefense)
            })
            .cloned()
            .collect::<Vec<_>>();
        if !immediate.is_empty() {
            return Some(missed_candidate_failure(
                FailureMode::MissedImmediateResponse,
                input,
                prefix_ply,
                actual_move,
                immediate,
            ));
        }

        let imminent = candidates
            .iter()
            .filter(|candidate| {
                candidate.roles.iter().any(|role| {
                    matches!(
                        role,
                        DefenderReplyRole::ImminentDefense | DefenderReplyRole::OffensiveCounter
                    )
                })
            })
            .cloned()
            .collect::<Vec<_>>();
        if !imminent.is_empty() {
            return Some(missed_candidate_failure(
                FailureMode::MissedImminentResponse,
                input,
                prefix_ply,
                actual_move,
                imminent,
            ));
        }
    }
    None
}

fn missed_lethal_prevention_failure(input: &FailureAnalysisInput<'_>) -> Option<FailureAnalysis> {
    let before_ply = failure_blame_cutoff(input)?;
    let prefix_ply =
        latest_loser_decision_before(input.boards, input.scan_start, before_ply, input.loser)?;
    let proof = proof_at(input.proof_summary, input.scan_start, prefix_ply)?;
    if proof.status != ProofStatus::EscapeFound {
        return None;
    }
    let actual_move = replay_move_at(input.replay, prefix_ply)?;
    let board = input.boards.get(prefix_ply)?;
    let missed_candidates =
        missed_prevention_candidates(board, input.winner, proof, prefix_ply, input.lethal_onset)?;
    if missed_candidates.is_empty()
        || missed_candidates
            .iter()
            .any(|candidate| candidate.mv == actual_move)
    {
        return None;
    }

    Some(FailureAnalysis {
        mode: FailureMode::MissedLethalPrevention,
        side: input.loser,
        prefix_ply: Some(prefix_ply),
        actual_move: Some(actual_move),
        actual_notation: Some(actual_move.to_notation()),
        confidence: if proof_has_limit_hit(proof) {
            FailureConfidence::Possible
        } else {
            FailureConfidence::Confirmed
        },
        missed_candidates,
        prevented_onset_ply: input.lethal_onset.map(|onset| onset.prefix_ply),
    })
}

fn missed_escape_failure(input: &FailureAnalysisInput<'_>) -> Option<FailureAnalysis> {
    if input.root_cause == RootCause::Unclear || !input.final_forced_interval_found {
        return None;
    }

    let before_ply = failure_blame_cutoff(input)?;
    if let Some(prefix_ply) = find_last_chance(
        input.boards,
        input.proof_summary,
        input.scan_start,
        before_ply,
        Some(input.loser),
    ) {
        let proof = proof_at(input.proof_summary, input.scan_start, prefix_ply)?;
        let actual_move = replay_move_at(input.replay, prefix_ply)?;
        let board = input.boards.get(prefix_ply)?;
        let missed_candidates = missed_prevention_candidates(
            board,
            input.winner,
            proof,
            prefix_ply,
            input.lethal_onset,
        )?;
        if missed_candidates.is_empty()
            || missed_candidates
                .iter()
                .any(|candidate| candidate.mv == actual_move)
        {
            return None;
        }

        return Some(FailureAnalysis {
            mode: FailureMode::MissedEscape,
            side: input.loser,
            prefix_ply: Some(prefix_ply),
            actual_move: Some(actual_move),
            actual_notation: Some(actual_move.to_notation()),
            confidence: if proof_has_limit_hit(proof) {
                FailureConfidence::Possible
            } else {
                FailureConfidence::Confirmed
            },
            missed_candidates,
            prevented_onset_ply: input.lethal_onset.map(|onset| onset.prefix_ply),
        });
    }

    Some(FailureAnalysis {
        mode: FailureMode::MissedEscape,
        side: input.loser,
        prefix_ply: input
            .lethal_onset
            .map(|onset| onset.prefix_ply)
            .or(Some(input.final_forced_interval.start_ply)),
        actual_move: None,
        actual_notation: None,
        missed_candidates: Vec::new(),
        prevented_onset_ply: input.lethal_onset.map(|onset| onset.prefix_ply),
        confidence: FailureConfidence::Confirmed,
    })
}

fn missed_prevention_candidates(
    board: &Board,
    attacker: Color,
    proof: &ProofResult,
    prefix_ply: usize,
    lethal_onset: Option<&LethalOnset>,
) -> Option<Vec<MissedCandidate>> {
    let outcome = if lethal_onset.is_some() {
        MissedCandidateOutcome::PreventsLethalOnset
    } else {
        MissedCandidateOutcome::PreventsCorridorEntry
    };
    let mut escape_moves = proof.escape_moves.clone();
    if escape_moves.is_empty() {
        for evidence in proof
            .threat_evidence
            .iter()
            .filter(|evidence| evidence.prefix_ply == Some(prefix_ply))
        {
            for mv in &evidence.escape_replies {
                push_unique_move(&mut escape_moves, *mv);
            }
        }
    }
    normalize_moves(&mut escape_moves);
    Some(
        escape_moves
            .into_iter()
            .map(|mv| missed_candidate(mv, failure_candidate_roles(board, attacker, mv), outcome))
            .collect::<Vec<_>>(),
    )
}

fn failure_blame_cutoff(input: &FailureAnalysisInput<'_>) -> Option<usize> {
    Some(
        input
            .lethal_onset
            .map(|onset| onset.prefix_ply)
            .unwrap_or(input.final_forced_interval.start_ply)
            .min(input.replay.moves.len()),
    )
}

fn latest_loser_decision_before(
    boards: &[Board],
    scan_start: usize,
    before_ply: usize,
    loser: Color,
) -> Option<usize> {
    (scan_start..before_ply)
        .rev()
        .find(|&ply| boards[ply].current_player == loser)
}

fn missed_candidate_failure(
    mode: FailureMode,
    input: &FailureAnalysisInput<'_>,
    prefix_ply: usize,
    actual_move: Move,
    missed_candidates: Vec<MissedCandidate>,
) -> FailureAnalysis {
    FailureAnalysis {
        mode,
        side: input.loser,
        prefix_ply: Some(prefix_ply),
        actual_move: Some(actual_move),
        actual_notation: Some(actual_move.to_notation()),
        confidence: failure_confidence_for_candidates(&missed_candidates),
        missed_candidates,
        prevented_onset_ply: input.lethal_onset.map(|onset| onset.prefix_ply),
    }
}

fn escape_candidates_from_proof(
    board: &Board,
    attacker: Color,
    proof: &ProofResult,
    prefix_ply: usize,
) -> Vec<MissedCandidate> {
    let mut candidates = Vec::new();
    for evidence in proof
        .threat_evidence
        .iter()
        .filter(|evidence| evidence.prefix_ply == Some(prefix_ply))
    {
        let outcome = match evidence.reply_classification {
            ReplyClassification::ConfirmedEscape => MissedCandidateOutcome::ConfirmedEscape,
            ReplyClassification::PossibleEscape => MissedCandidateOutcome::PossibleEscape,
            _ => continue,
        };
        for mv in &evidence.escape_replies {
            push_missed_candidate(
                &mut candidates,
                missed_candidate(*mv, failure_candidate_roles(board, attacker, *mv), outcome),
            );
        }
    }
    candidates.sort_by_key(|candidate| (candidate.mv.row, candidate.mv.col));
    candidates
}

fn failure_candidate_roles(board: &Board, attacker: Color, mv: Move) -> Vec<DefenderReplyRole> {
    let mut roles = defender_reply_roles_for_move(board, attacker, mv);
    if roles.is_empty()
        && board
            .immediate_winning_moves_for(attacker.opponent())
            .contains(&mv)
    {
        roles.push(DefenderReplyRole::OffensiveCounter);
    }
    roles
}

fn is_response_role_set(roles: &[DefenderReplyRole]) -> bool {
    roles.iter().any(|role| {
        matches!(
            role,
            DefenderReplyRole::ImmediateDefense
                | DefenderReplyRole::ImminentDefense
                | DefenderReplyRole::OffensiveCounter
        )
    })
}

fn missed_candidate(
    mv: Move,
    roles: Vec<DefenderReplyRole>,
    outcome: MissedCandidateOutcome,
) -> MissedCandidate {
    MissedCandidate {
        mv,
        notation: mv.to_notation(),
        roles,
        outcome,
    }
}

fn push_missed_candidate(candidates: &mut Vec<MissedCandidate>, candidate: MissedCandidate) {
    if let Some(existing) = candidates
        .iter_mut()
        .find(|existing| existing.mv == candidate.mv)
    {
        for role in candidate.roles {
            if !existing.roles.contains(&role) {
                existing.roles.push(role);
            }
        }
        if candidate.outcome == MissedCandidateOutcome::ConfirmedEscape {
            existing.outcome = MissedCandidateOutcome::ConfirmedEscape;
        }
        return;
    }
    candidates.push(candidate);
}

fn failure_confidence_for_candidates(candidates: &[MissedCandidate]) -> FailureConfidence {
    if candidates
        .iter()
        .any(|candidate| candidate.outcome == MissedCandidateOutcome::PossibleEscape)
    {
        FailureConfidence::Possible
    } else {
        FailureConfidence::Confirmed
    }
}

fn replay_move_at(replay: &Replay, prefix_ply: usize) -> Option<Move> {
    replay
        .moves
        .get(prefix_ply)
        .and_then(|mv| Move::from_notation(&mv.mv).ok())
}

fn normalize_moves(moves: &mut Vec<Move>) {
    moves.sort_by_key(|mv| (mv.row, mv.col));
    moves.dedup();
}
