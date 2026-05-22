use crate::onset::{onset_line_key_between, onset_line_key_for_fact, onset_local_three_facts};
use crate::replay::replay_prefix_boards;
use crate::trace::{
    analyze_alternate_defender_reply_options, defender_reply_roles_for_move,
    visible_defender_reply_candidates,
};
use crate::types::*;
use crate::util::{normalize_moves, push_unique_move};
use gomoku_bot::tactical::{
    compound_imminent_evidence_stones, legal_forcing_continuations_for_fact,
    local_threat_evidence_stones, LocalThreatFact, LocalThreatKind, ScanThreatView,
    SearchThreatPolicy, ThreatObligationKind, ThreatView,
};
use gomoku_core::{Board, Color, Move, Replay};

pub(crate) fn replay_frame_annotations_from_proof(
    ply: usize,
    board: &Board,
    winner: Color,
    proof: &ProofResult,
    _actual_child: Option<&ProofResult>,
    actual_reply: Option<Move>,
    options: &AnalysisOptions,
) -> ReplayFrameAnnotations {
    let mut frame = ReplayFrameAnnotations {
        ply,
        side_to_move: proof.side_to_move,
        evidence: Vec::new(),
        highlights: Vec::new(),
        markers: Vec::new(),
    };

    let candidates = if board.current_player == winner.opponent() {
        visible_defender_reply_candidates(board, winner, actual_reply)
    } else {
        Vec::new()
    };
    let replies = if board.current_player == winner.opponent() {
        analyze_alternate_defender_reply_options(board, winner, actual_reply, options)
    } else {
        Vec::new()
    };
    push_current_loser_candidate_annotations(&mut frame, board, winner, &candidates);
    if candidates.is_empty() {
        push_current_loser_immediate_win_annotations(&mut frame, board, winner);
    }

    if board.current_player == winner.opponent() {
        if let Some(actual_reply) = actual_reply {
            push_actual_reply_hint_annotations(&mut frame, board, winner, actual_reply);
        }
        push_reply_outcome_annotations(&mut frame, board, winner, &replies);
    }

    frame
}

pub fn replay_frame_annotations_for_analysis(
    replay: &Replay,
    analysis: &GameAnalysis,
) -> Result<Vec<ReplayFrameAnnotations>, AnalysisError> {
    let boards = replay_prefix_boards(replay)?;
    Ok(replay_frame_annotations_for_analysis_with_boards(
        replay, &boards, analysis,
    ))
}

pub(crate) fn replay_frame_annotations_for_analysis_with_boards(
    replay: &Replay,
    boards: &[Board],
    analysis: &GameAnalysis,
) -> Vec<ReplayFrameAnnotations> {
    let Some(winner) = analysis.winner else {
        return Vec::new();
    };
    if analysis.proof_summary.is_empty() {
        return Vec::new();
    }

    let Some(scan_start) = boards.len().checked_sub(analysis.proof_summary.len()) else {
        return Vec::new();
    };
    let first_actual_ply = replay_annotation_start_actual_ply(boards, analysis);
    let mut frames = Vec::new();
    for actual_ply in (first_actual_ply..=analysis.final_forced_interval.end_ply).rev() {
        let Some(prefix_ply) = actual_ply.checked_sub(1) else {
            continue;
        };
        let Some(board) = boards.get(prefix_ply) else {
            continue;
        };
        let Some(proof) = proof_result_at(&analysis.proof_summary, scan_start, prefix_ply) else {
            continue;
        };
        let actual_child = proof_result_at(
            &analysis.proof_summary,
            scan_start,
            prefix_ply.saturating_add(1),
        );
        let previous_proof = prefix_ply
            .checked_sub(1)
            .and_then(|previous| proof_result_at(&analysis.proof_summary, scan_start, previous));
        let actual_reply = actual_move_at_prefix(replay, prefix_ply);
        let mut frame = replay_frame_annotations_from_proof(
            prefix_ply,
            board,
            winner,
            proof,
            actual_child,
            actual_reply,
            &AnalysisOptions {
                reply_policy: analysis.model.reply_policy,
                max_depth: analysis.model.max_depth,
                max_scan_plies: analysis.model.max_scan_plies,
            },
        );
        push_pre_corridor_escape_annotation(
            &mut frame,
            replay,
            analysis,
            actual_ply,
            board,
            proof,
            previous_proof,
        );
        push_lethal_onset_annotations(
            &mut frame,
            board,
            analysis.lethal_onset.as_ref(),
            actual_reply,
        );
        frames.push(frame);
    }
    frames
}

fn replay_annotation_start_actual_ply(boards: &[Board], analysis: &GameAnalysis) -> usize {
    let start_ply = analysis.final_forced_interval.start_ply;
    let Some(winner) = analysis.winner else {
        return start_ply;
    };
    if start_ply <= 1 {
        return start_ply;
    }

    let start_board_ply = start_ply.saturating_sub(1);
    if boards
        .get(start_board_ply)
        .is_some_and(|board| board.current_player == winner.opponent())
    {
        start_ply - 1
    } else {
        start_ply
    }
}

fn proof_result_at(
    proofs: &[ProofResult],
    scan_start: usize,
    prefix_ply: usize,
) -> Option<&ProofResult> {
    proofs.get(prefix_ply.checked_sub(scan_start)?)
}

fn actual_move_at_prefix(replay: &Replay, prefix_ply: usize) -> Option<Move> {
    let replay_move = replay.moves.get(prefix_ply)?;
    Move::from_notation(&replay_move.mv).ok()
}

fn push_current_loser_immediate_win_annotations(
    frame: &mut ReplayFrameAnnotations,
    board: &Board,
    winner: Color,
) {
    let defender = winner.opponent();
    if board.current_player != defender {
        return;
    }

    let defender_wins = board.immediate_winning_moves_for(defender);
    for mv in defender_wins.iter().copied() {
        push_replay_highlight(
            &mut frame.highlights,
            ReplayFrameHighlightRole::ImmediateWin,
            mv,
            defender,
        );
        push_candidate_threat_evidence(
            frame,
            board,
            defender,
            mv,
            ReplayFrameHighlightRole::ImmediateWin,
            defender,
            |kind| kind == LocalThreatKind::Five,
        );
    }
}

fn push_current_loser_candidate_annotations(
    frame: &mut ReplayFrameAnnotations,
    board: &Board,
    winner: Color,
    candidates: &[DefenderReplyCandidate],
) {
    let defender = winner.opponent();
    if board.current_player != defender {
        return;
    }

    for candidate in candidates {
        for role in &candidate.roles {
            push_defender_reply_role_highlight(frame, board, *role, candidate.mv, winner);
        }
        if !board.is_legal_for_color(candidate.mv, defender) {
            push_replay_marker(
                &mut frame.markers,
                ReplayFrameMarkerRole::Forbidden,
                candidate.mv,
                defender,
            );
        }
    }
}

fn push_reply_outcome_annotations(
    frame: &mut ReplayFrameAnnotations,
    board: &Board,
    attacker: Color,
    replies: &[DefenderReplyAnalysis],
) {
    let defender = attacker.opponent();
    for reply in replies {
        for role in &reply.roles {
            push_defender_reply_role_highlight(frame, board, *role, reply.mv, attacker);
        }

        push_replay_marker(
            &mut frame.markers,
            replay_marker_role_for_defender_reply_outcome(reply.outcome),
            reply.mv,
            defender,
        );
    }
}

fn push_actual_reply_hint_annotations(
    frame: &mut ReplayFrameAnnotations,
    board: &Board,
    attacker: Color,
    mv: Move,
) {
    let defender = attacker.opponent();
    if board.current_player != defender {
        return;
    }

    for role in defender_reply_roles_for_move(board, attacker, mv) {
        push_defender_reply_role_highlight(frame, board, role, mv, attacker);
    }
}

fn push_defender_reply_role_highlight(
    frame: &mut ReplayFrameAnnotations,
    board: &Board,
    role: DefenderReplyRole,
    mv: Move,
    attacker: Color,
) {
    let Some((highlight_role, side)) = replay_highlight_for_defender_reply_role(role, attacker)
    else {
        return;
    };
    push_replay_highlight(&mut frame.highlights, highlight_role, mv, side);
    push_defender_reply_role_evidence(frame, board, role, mv, attacker, highlight_role, side);
}

fn replay_highlight_for_defender_reply_role(
    role: DefenderReplyRole,
    attacker: Color,
) -> Option<(ReplayFrameHighlightRole, Color)> {
    match role {
        DefenderReplyRole::Actual => None,
        DefenderReplyRole::ImmediateDefense => {
            Some((ReplayFrameHighlightRole::ImmediateThreat, attacker))
        }
        DefenderReplyRole::ImminentDefense => {
            Some((ReplayFrameHighlightRole::ImminentThreat, attacker))
        }
        DefenderReplyRole::OffensiveCounter => {
            Some((ReplayFrameHighlightRole::CounterThreat, attacker.opponent()))
        }
    }
}

fn push_defender_reply_role_evidence(
    frame: &mut ReplayFrameAnnotations,
    board: &Board,
    role: DefenderReplyRole,
    mv: Move,
    attacker: Color,
    highlight_role: ReplayFrameHighlightRole,
    side: Color,
) {
    match role {
        DefenderReplyRole::Actual => {}
        DefenderReplyRole::ImmediateDefense => push_candidate_threat_evidence(
            frame,
            board,
            attacker,
            mv,
            highlight_role,
            side,
            |kind| kind == LocalThreatKind::Five,
        ),
        DefenderReplyRole::ImminentDefense => {
            let view = ScanThreatView::new(board);
            let Some(obligation) = view.threat_obligation(attacker) else {
                return;
            };
            if obligation.kind != ThreatObligationKind::Imminent {
                return;
            }
            for fact in &obligation.local_facts {
                for evidence in local_threat_evidence_stones(board, fact) {
                    push_replay_highlight(&mut frame.evidence, highlight_role, evidence, side);
                }
            }
            for evidence in
                compound_imminent_evidence_stones(board, attacker, &obligation.compound_entries)
            {
                push_replay_highlight(&mut frame.evidence, highlight_role, evidence, side);
            }
        }
        DefenderReplyRole::OffensiveCounter => push_candidate_threat_evidence(
            frame,
            board,
            attacker.opponent(),
            mv,
            highlight_role,
            side,
            |kind| {
                matches!(
                    kind,
                    LocalThreatKind::Five
                        | LocalThreatKind::OpenFour
                        | LocalThreatKind::ClosedFour
                        | LocalThreatKind::BrokenFour
                )
            },
        ),
    }
}

fn push_candidate_threat_evidence(
    frame: &mut ReplayFrameAnnotations,
    board: &Board,
    player: Color,
    mv: Move,
    role: ReplayFrameHighlightRole,
    side: Color,
    keep: impl Fn(LocalThreatKind) -> bool,
) {
    let annotation = SearchThreatPolicy.annotation_for_player(board, player, mv);
    for fact in annotation
        .local_threats
        .iter()
        .filter(|fact| keep(fact.kind))
    {
        for evidence in local_threat_evidence_stones(board, fact) {
            push_replay_highlight(&mut frame.evidence, role, evidence, side);
        }
    }
}

pub(crate) fn push_lethal_onset_annotations(
    frame: &mut ReplayFrameAnnotations,
    board: &Board,
    onset: Option<&LethalOnset>,
    actual_reply: Option<Move>,
) {
    let Some(onset) = onset else {
        return;
    };
    if frame.ply != onset.prefix_ply {
        return;
    }

    for &target in &onset.terminal_targets {
        push_replay_highlight(
            &mut frame.highlights,
            ReplayFrameHighlightRole::ImmediateThreat,
            target,
            onset.attacker,
        );
        push_onset_immediate_loss_marker(frame, board, onset.defender, target, actual_reply);
        push_candidate_threat_evidence(
            frame,
            board,
            onset.attacker,
            target,
            ReplayFrameHighlightRole::ImmediateThreat,
            onset.attacker,
            |kind| kind == LocalThreatKind::Five,
        );
    }

    for fact in onset_local_three_facts(board, onset.attacker)
        .iter()
        .filter(|fact| lethal_onset_includes_local_three_fact(onset, fact))
    {
        for mv in onset_defender_reply_moves_for_fact(board, onset.attacker, fact) {
            push_replay_highlight(
                &mut frame.highlights,
                ReplayFrameHighlightRole::ImmediateThreat,
                mv,
                onset.attacker,
            );
            push_onset_immediate_loss_marker(frame, board, onset.defender, mv, actual_reply);
        }
        for evidence in local_threat_evidence_stones(board, fact) {
            push_replay_highlight(
                &mut frame.evidence,
                ReplayFrameHighlightRole::ImmediateThreat,
                evidence,
                onset.attacker,
            );
        }
    }
}

fn push_onset_immediate_loss_marker(
    frame: &mut ReplayFrameAnnotations,
    board: &Board,
    defender: Color,
    mv: Move,
    actual_reply: Option<Move>,
) {
    if actual_reply == Some(mv) {
        return;
    }
    if board.is_empty(mv.row, mv.col) {
        push_replay_marker(
            &mut frame.markers,
            ReplayFrameMarkerRole::ImmediateLoss,
            mv,
            defender,
        );
    }
}

fn lethal_onset_includes_local_three_fact(onset: &LethalOnset, fact: &LocalThreatFact) -> bool {
    onset
        .shape
        .components
        .iter()
        .filter(|component| component.tier == LethalOnsetComponentTier::Three)
        .any(|component| lethal_onset_component_matches_fact(component, fact))
}

fn lethal_onset_component_matches_fact(
    component: &LethalOnsetComponent,
    fact: &LocalThreatFact,
) -> bool {
    let origin = fact.origin.mv();
    if component.mv == origin {
        return true;
    }

    let fact_key = onset_line_key_for_fact(fact);
    onset_line_key_between(origin, component.mv).is_some_and(|key| key == fact_key)
}

fn onset_defender_reply_moves_for_fact(
    board: &Board,
    attacker: Color,
    fact: &LocalThreatFact,
) -> Vec<Move> {
    let continuations = legal_forcing_continuations_for_fact(board, attacker, fact);
    let mut replies = Vec::new();

    for continuation in &continuations {
        if board.is_empty(continuation.mv.row, continuation.mv.col) {
            push_unique_move(&mut replies, continuation.mv);
        }
    }

    let mut shared_cost_squares: Option<Vec<Move>> = None;
    for continuation in continuations {
        let costs = continuation
            .legal_cost_squares
            .into_iter()
            .filter(|&mv| board.is_empty(mv.row, mv.col))
            .collect::<Vec<_>>();

        shared_cost_squares = Some(match shared_cost_squares {
            Some(shared) => shared
                .into_iter()
                .filter(|mv| costs.contains(mv))
                .collect::<Vec<_>>(),
            None => costs,
        });
    }

    for mv in shared_cost_squares.unwrap_or_default() {
        push_unique_move(&mut replies, mv);
    }
    normalize_moves(&mut replies);
    replies
}

fn replay_marker_role_for_defender_reply_outcome(
    outcome: DefenderReplyOutcome,
) -> ReplayFrameMarkerRole {
    match outcome {
        DefenderReplyOutcome::ForcedLoss => ReplayFrameMarkerRole::ForcedLoss,
        DefenderReplyOutcome::ConfirmedEscape => ReplayFrameMarkerRole::ConfirmedEscape,
        DefenderReplyOutcome::PossibleEscape => ReplayFrameMarkerRole::PossibleEscape,
        DefenderReplyOutcome::ImmediateLoss => ReplayFrameMarkerRole::ImmediateLoss,
        DefenderReplyOutcome::Unknown => ReplayFrameMarkerRole::Unknown,
    }
}

fn push_pre_corridor_escape_annotation(
    frame: &mut ReplayFrameAnnotations,
    replay: &Replay,
    analysis: &GameAnalysis,
    actual_ply: usize,
    board: &Board,
    proof: &ProofResult,
    previous_proof: Option<&ProofResult>,
) {
    let Some(winner) = analysis.winner else {
        return;
    };
    if board.current_player != winner.opponent() || !frame.highlights.is_empty() {
        return;
    }

    let Some(entry_move) =
        pre_corridor_escape_entry_move(replay, analysis, actual_ply, proof, previous_proof)
    else {
        return;
    };
    if !board.is_legal(entry_move) {
        return;
    }

    push_replay_highlight(
        &mut frame.highlights,
        ReplayFrameHighlightRole::CorridorEntry,
        entry_move,
        winner,
    );
    push_replay_marker(
        &mut frame.markers,
        ReplayFrameMarkerRole::ConfirmedEscape,
        entry_move,
        winner.opponent(),
    );
}

fn pre_corridor_escape_entry_move(
    replay: &Replay,
    analysis: &GameAnalysis,
    actual_ply: usize,
    proof: &ProofResult,
    previous_proof: Option<&ProofResult>,
) -> Option<Move> {
    if actual_ply == analysis.final_forced_interval.start_ply
        && proof.status == ProofStatus::EscapeFound
    {
        return actual_move_at_prefix(replay, actual_ply);
    }

    if actual_ply == analysis.final_forced_interval.start_ply + 1
        && previous_proof.map(|proof| proof.status) == Some(ProofStatus::EscapeFound)
        && proof.status == ProofStatus::ForcedWin
    {
        return proof.principal_line.first().copied();
    }

    None
}

fn push_replay_highlight(
    highlights: &mut Vec<ReplayFrameHighlight>,
    role: ReplayFrameHighlightRole,
    mv: Move,
    side: Color,
) {
    let highlight = ReplayFrameHighlight {
        role,
        mv,
        notation: mv.to_notation(),
        side,
    };
    if !highlights.contains(&highlight) {
        highlights.push(highlight);
    }
}

fn push_replay_marker(
    markers: &mut Vec<ReplayFrameMarker>,
    role: ReplayFrameMarkerRole,
    mv: Move,
    side: Color,
) {
    let marker = ReplayFrameMarker {
        role,
        mv,
        notation: mv.to_notation(),
        side,
    };
    if !markers.contains(&marker) {
        markers.push(marker);
    }
}
