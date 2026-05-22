use crate::model::corridor_analysis_model;
use crate::replay::proof_has_limit_hit;
use crate::types::*;
use gomoku_bot::corridor as bot_corridor;
use gomoku_bot::tactical::corridor_active_threats;
use gomoku_core::{Board, Color, GameResult, Move};

#[derive(Debug, Clone, Copy)]
pub(crate) struct EvidenceAttribution {
    prefix_ply: Option<usize>,
    actual_reply: Option<Move>,
}

pub(crate) struct ThreatEvidenceInput {
    attribution: EvidenceAttribution,
    reply_classification: ReplyClassification,
    escape_replies: Vec<Move>,
    forced_replies: Vec<Move>,
    next_forcing_move: Option<Move>,
    proof_status: ProofStatus,
    limit_causes: Vec<ProofLimitCause>,
}
pub fn analyze_defender_reply_options(
    board: &Board,
    attacker: Color,
    actual_reply: Option<Move>,
    options: &AnalysisOptions,
) -> Vec<DefenderReplyAnalysis> {
    bot_corridor::analyze_defender_reply_options(
        board,
        attacker,
        actual_reply,
        &options.corridor_options(),
    )
}

pub fn analyze_alternate_defender_reply_options(
    board: &Board,
    attacker: Color,
    excluded_reply: Option<Move>,
    options: &AnalysisOptions,
) -> Vec<DefenderReplyAnalysis> {
    bot_corridor::analyze_alternate_defender_reply_options(
        board,
        attacker,
        excluded_reply,
        &options.corridor_options(),
    )
}

pub fn defender_reply_candidates(
    board: &Board,
    attacker: Color,
    actual_reply: Option<Move>,
) -> Vec<DefenderReplyCandidate> {
    visible_defender_reply_candidates(board, attacker, actual_reply)
}

pub fn visible_defender_reply_candidates(
    board: &Board,
    attacker: Color,
    actual_reply: Option<Move>,
) -> Vec<DefenderReplyCandidate> {
    bot_corridor::visible_defender_reply_candidates(board, attacker, actual_reply)
        .into_iter()
        .map(defender_reply_candidate_with_notation)
        .collect()
}

pub fn defender_reply_roles_for_move(
    board: &Board,
    attacker: Color,
    mv: Move,
) -> Vec<DefenderReplyRole> {
    bot_corridor::defender_reply_roles_for_move(board, attacker, mv)
}

fn defender_reply_candidate_with_notation(
    candidate: bot_corridor::DefenderReplyCandidate,
) -> DefenderReplyCandidate {
    DefenderReplyCandidate {
        mv: candidate.mv,
        notation: candidate.mv.to_notation(),
        roles: candidate.roles,
    }
}

type DefenderReplyProof = bot_corridor::DefenderReplyProof;

fn classify_defender_reply_for_report(
    board: &Board,
    attacker: Color,
    mv: Move,
    options: &AnalysisOptions,
) -> DefenderReplyProof {
    bot_corridor::classify_defender_reply(board, attacker, mv, &options.corridor_options())
}

pub(crate) struct ThreatReplySet {
    attacker: Color,
    defender: Color,
    winning_squares: Vec<Move>,
    raw_cost_squares: Vec<Move>,
    legal_cost_squares: Vec<Move>,
    illegal_cost_squares: Vec<Move>,
    defender_immediate_wins: Vec<Move>,
}

impl ThreatReplySet {
    pub(crate) fn new(board: &Board, attacker: Color) -> Self {
        let defender = attacker.opponent();
        let winning_squares = board.immediate_winning_moves_for(attacker);
        let raw_cost_squares = winning_squares.clone();
        let mut legal_cost_squares = Vec::new();
        let mut illegal_cost_squares = Vec::new();
        for mv in raw_cost_squares.iter().copied() {
            if board.is_legal_for_color(mv, defender) {
                legal_cost_squares.push(mv);
            } else {
                illegal_cost_squares.push(mv);
            }
        }
        let defender_immediate_wins = board.immediate_winning_moves_for(defender);
        Self {
            attacker,
            defender,
            winning_squares,
            raw_cost_squares,
            legal_cost_squares,
            illegal_cost_squares,
            defender_immediate_wins,
        }
    }

    fn evidence(&self, input: ThreatEvidenceInput) -> ThreatSequenceEvidence {
        let limit_hit = !input.limit_causes.is_empty();
        ThreatSequenceEvidence {
            prefix_ply: input.attribution.prefix_ply,
            attacker: self.attacker,
            defender: self.defender,
            winning_squares: self.winning_squares.clone(),
            raw_cost_squares: self.raw_cost_squares.clone(),
            legal_cost_squares: self.legal_cost_squares.clone(),
            illegal_cost_squares: self.illegal_cost_squares.clone(),
            defender_immediate_wins: self.defender_immediate_wins.clone(),
            actual_reply: input.attribution.actual_reply,
            reply_classification: input.reply_classification,
            escape_replies: input.escape_replies,
            forced_replies: input.forced_replies,
            next_forcing_move: input.next_forcing_move,
            proof_status: input.proof_status,
            limit_hit,
            limit_causes: input.limit_causes,
        }
    }
}

fn next_attacker_move_after_defender_reply(principal_line: &[Move]) -> Option<Move> {
    principal_line.get(1).copied()
}

fn proof_limit_hit_from_evidence(threat_evidence: &[ThreatSequenceEvidence]) -> bool {
    !proof_limit_causes_from_evidence(threat_evidence).is_empty()
        || threat_evidence.iter().any(|evidence| evidence.limit_hit)
}

fn proof_limit_causes_from_evidence(
    threat_evidence: &[ThreatSequenceEvidence],
) -> Vec<ProofLimitCause> {
    let mut causes = Vec::new();
    for evidence in threat_evidence {
        extend_limit_causes(&mut causes, evidence.limit_causes.iter().copied());
    }
    causes
}

fn extend_limit_causes(
    causes: &mut Vec<ProofLimitCause>,
    new_causes: impl IntoIterator<Item = ProofLimitCause>,
) {
    for cause in new_causes {
        if !causes.contains(&cause) {
            causes.push(cause);
        }
    }
    causes.sort();
}

pub(crate) fn with_limit_causes(
    mut proof: ProofResult,
    causes: impl IntoIterator<Item = ProofLimitCause>,
) -> ProofResult {
    extend_limit_causes(&mut proof.limit_causes, causes);
    proof.limit_hit = !proof.limit_causes.is_empty();
    proof
}

pub(crate) fn corridor_proof_result(
    board: &Board,
    attacker: Color,
    options: &AnalysisOptions,
    status: ProofStatus,
    principal_line: Vec<Move>,
    escape_moves: Vec<Move>,
    threat_evidence: Vec<ThreatSequenceEvidence>,
) -> ProofResult {
    let limit_causes = proof_limit_causes_from_evidence(&threat_evidence);
    let limit_hit = !limit_causes.is_empty() || proof_limit_hit_from_evidence(&threat_evidence);
    ProofResult {
        status,
        attacker,
        side_to_move: board.current_player,
        model: corridor_analysis_model(board, options),
        principal_line,
        escape_moves,
        threat_evidence,
        limit_hit,
        limit_causes,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CorridorReplyStatus {
    Forced,
    ConfirmedEscape,
    PossibleEscape,
    Unknown,
}

pub(crate) struct CorridorReplyOutcome {
    mv: Move,
    pub(crate) status: CorridorReplyStatus,
    proof: ProofResult,
}

fn replay_corridor_status(
    board: &Board,
    actual_moves: &[Move],
    attacker: Color,
    options: &AnalysisOptions,
    prefix_ply: usize,
) -> ProofResult {
    replay_corridor_status_with_actual_child(
        board,
        actual_moves,
        attacker,
        options,
        prefix_ply,
        None,
    )
}

pub(crate) fn replay_corridor_status_with_actual_child(
    board: &Board,
    actual_moves: &[Move],
    attacker: Color,
    options: &AnalysisOptions,
    prefix_ply: usize,
    actual_child: Option<&ProofResult>,
) -> ProofResult {
    match board.result {
        GameResult::Winner(winner) if winner == attacker => {
            return corridor_proof_result(
                board,
                attacker,
                options,
                ProofStatus::ForcedWin,
                Vec::new(),
                Vec::new(),
                Vec::new(),
            );
        }
        GameResult::Winner(_) | GameResult::Draw => {
            return corridor_proof_result(
                board,
                attacker,
                options,
                ProofStatus::EscapeFound,
                Vec::new(),
                Vec::new(),
                Vec::new(),
            );
        }
        GameResult::Ongoing => {}
    }

    if board.current_player == attacker {
        replay_corridor_attacker_node(
            board,
            actual_moves,
            attacker,
            options,
            prefix_ply,
            actual_child,
        )
    } else {
        replay_corridor_defender_node(
            board,
            actual_moves,
            attacker,
            options,
            prefix_ply,
            actual_child,
        )
    }
}

fn replay_corridor_attacker_node(
    board: &Board,
    actual_moves: &[Move],
    attacker: Color,
    options: &AnalysisOptions,
    prefix_ply: usize,
    actual_child: Option<&ProofResult>,
) -> ProofResult {
    let immediate_wins = board.immediate_winning_moves_for(attacker);
    if let Some(&mv) = immediate_wins.first() {
        return corridor_proof_result(
            board,
            attacker,
            options,
            ProofStatus::ForcedWin,
            vec![mv],
            Vec::new(),
            Vec::new(),
        );
    }

    let Some(&actual_move) = actual_moves.get(prefix_ply) else {
        return corridor_proof_result(
            board,
            attacker,
            options,
            ProofStatus::EscapeFound,
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );
    };
    let actual_move_enters_corridor =
        is_corridor_attacker_move(board, attacker, actual_move, options);
    if !actual_move_enters_corridor && corridor_active_threats(board, attacker).is_empty() {
        return corridor_proof_result(
            board,
            attacker,
            options,
            ProofStatus::EscapeFound,
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );
    }

    let mut next = board.clone();
    if next.apply_move(actual_move).is_err() {
        return corridor_proof_result(
            board,
            attacker,
            options,
            ProofStatus::Unknown,
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );
    }

    let child = actual_child.cloned().unwrap_or_else(|| {
        replay_corridor_status(&next, actual_moves, attacker, options, prefix_ply + 1)
    });
    if !actual_move_enters_corridor && child.status != ProofStatus::ForcedWin {
        return corridor_proof_result(
            board,
            attacker,
            options,
            ProofStatus::EscapeFound,
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );
    }
    match child.status {
        ProofStatus::ForcedWin => {
            let mut principal_line = Vec::with_capacity(child.principal_line.len() + 1);
            principal_line.push(actual_move);
            principal_line.extend(child.principal_line);
            with_limit_causes(
                corridor_proof_result(
                    board,
                    attacker,
                    options,
                    ProofStatus::ForcedWin,
                    principal_line,
                    Vec::new(),
                    child.threat_evidence,
                ),
                child.limit_causes,
            )
        }
        ProofStatus::EscapeFound => with_limit_causes(
            corridor_proof_result(
                board,
                attacker,
                options,
                ProofStatus::EscapeFound,
                Vec::new(),
                child.escape_moves,
                child.threat_evidence,
            ),
            child.limit_causes,
        ),
        ProofStatus::Unknown => {
            let mut causes = child.limit_causes;
            extend_limit_causes(&mut causes, [ProofLimitCause::AttackerChildUnknown]);
            with_limit_causes(
                corridor_proof_result(
                    board,
                    attacker,
                    options,
                    ProofStatus::Unknown,
                    vec![actual_move],
                    Vec::new(),
                    child.threat_evidence,
                ),
                causes,
            )
        }
    }
}

fn replay_corridor_defender_node(
    board: &Board,
    actual_moves: &[Move],
    attacker: Color,
    options: &AnalysisOptions,
    prefix_ply: usize,
    actual_child: Option<&ProofResult>,
) -> ProofResult {
    let threat = ThreatReplySet::new(board, attacker);
    let attribution = EvidenceAttribution {
        prefix_ply: Some(prefix_ply),
        actual_reply: actual_moves.get(prefix_ply).copied(),
    };

    if !threat.winning_squares.is_empty()
        && threat.legal_cost_squares.is_empty()
        && threat.defender_immediate_wins.is_empty()
    {
        return corridor_proof_result(
            board,
            attacker,
            options,
            ProofStatus::ForcedWin,
            threat
                .winning_squares
                .first()
                .copied()
                .into_iter()
                .collect(),
            Vec::new(),
            vec![threat.evidence(ThreatEvidenceInput {
                attribution,
                reply_classification: ReplyClassification::NoLegalBlock,
                escape_replies: Vec::new(),
                forced_replies: Vec::new(),
                next_forcing_move: threat.winning_squares.first().copied(),
                proof_status: ProofStatus::ForcedWin,
                limit_causes: Vec::new(),
            })],
        );
    }

    let reply_moves =
        corridor_defender_reply_moves(board, actual_moves, prefix_ply, options, &threat);
    if reply_moves.is_empty() {
        if !corridor_active_threats(board, attacker).is_empty() {
            if let Some(child) = actual_child.filter(|proof| proof.status == ProofStatus::ForcedWin)
            {
                let mut principal_line = actual_moves
                    .get(prefix_ply)
                    .copied()
                    .into_iter()
                    .collect::<Vec<_>>();
                principal_line.extend(child.principal_line.clone());
                let mut evidence = vec![threat.evidence(ThreatEvidenceInput {
                    attribution,
                    reply_classification: ReplyClassification::NoLegalBlock,
                    escape_replies: Vec::new(),
                    forced_replies: actual_moves
                        .get(prefix_ply)
                        .copied()
                        .into_iter()
                        .collect::<Vec<_>>(),
                    next_forcing_move: next_attacker_move_after_defender_reply(&principal_line),
                    proof_status: ProofStatus::ForcedWin,
                    limit_causes: Vec::new(),
                })];
                evidence.extend(child.threat_evidence.clone());
                return with_limit_causes(
                    corridor_proof_result(
                        board,
                        attacker,
                        options,
                        ProofStatus::ForcedWin,
                        principal_line,
                        Vec::new(),
                        evidence,
                    ),
                    child.limit_causes.clone(),
                );
            }
        }
        return with_limit_causes(
            corridor_proof_result(
                board,
                attacker,
                options,
                ProofStatus::Unknown,
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ),
            [ProofLimitCause::ModelScopeUnknown],
        );
    }
    let mut outcomes = Vec::new();
    for mv in reply_moves {
        if Some(mv) == actual_moves.get(prefix_ply).copied() {
            // The actual replay reply inherits the already-computed next prefix proof.
            // Only alternate replies need fresh branch probes.
            outcomes.push(classify_actual_corridor_reply(
                board,
                actual_moves,
                attacker,
                options,
                prefix_ply,
                mv,
                actual_child,
            ));
        } else {
            outcomes.push(classify_corridor_reply(board, attacker, options, mv));
        }
    }

    let escape_replies = outcomes
        .iter()
        .filter_map(|outcome| {
            (outcome.status == CorridorReplyStatus::ConfirmedEscape).then_some(outcome.mv)
        })
        .collect::<Vec<_>>();
    if !escape_replies.is_empty() {
        let mut evidence = vec![threat.evidence(ThreatEvidenceInput {
            attribution,
            reply_classification: ReplyClassification::ConfirmedEscape,
            escape_replies: escape_replies.clone(),
            forced_replies: forced_corridor_replies(&outcomes),
            next_forcing_move: None,
            proof_status: ProofStatus::EscapeFound,
            limit_causes: Vec::new(),
        })];
        evidence.extend(first_corridor_branch_evidence(
            &outcomes,
            CorridorReplyStatus::ConfirmedEscape,
        ));
        return corridor_proof_result(
            board,
            attacker,
            options,
            ProofStatus::EscapeFound,
            Vec::new(),
            escape_replies.clone(),
            evidence,
        );
    }

    let possible_escape_replies = outcomes
        .iter()
        .filter_map(|outcome| {
            (outcome.status == CorridorReplyStatus::PossibleEscape).then_some(outcome.mv)
        })
        .collect::<Vec<_>>();
    if !possible_escape_replies.is_empty() {
        let mut limit_causes = Vec::new();
        for outcome in outcomes
            .iter()
            .filter(|outcome| outcome.status == CorridorReplyStatus::PossibleEscape)
        {
            extend_limit_causes(
                &mut limit_causes,
                outcome.proof.limit_causes.iter().copied(),
            );
        }
        extend_limit_causes(&mut limit_causes, [ProofLimitCause::DefenderReplyUnknown]);
        let mut evidence = vec![threat.evidence(ThreatEvidenceInput {
            attribution,
            reply_classification: ReplyClassification::PossibleEscape,
            escape_replies: possible_escape_replies.clone(),
            forced_replies: forced_corridor_replies(&outcomes),
            next_forcing_move: None,
            proof_status: ProofStatus::EscapeFound,
            limit_causes: limit_causes.clone(),
        })];
        evidence.extend(first_corridor_branch_evidence(
            &outcomes,
            CorridorReplyStatus::PossibleEscape,
        ));
        return with_limit_causes(
            corridor_proof_result(
                board,
                attacker,
                options,
                ProofStatus::EscapeFound,
                Vec::new(),
                possible_escape_replies,
                evidence,
            ),
            limit_causes,
        );
    }

    let mut limit_causes = Vec::new();
    for outcome in outcomes
        .iter()
        .filter(|outcome| outcome.status == CorridorReplyStatus::Unknown)
    {
        extend_limit_causes(
            &mut limit_causes,
            outcome.proof.limit_causes.iter().copied(),
        );
    }
    if !limit_causes.is_empty() {
        extend_limit_causes(&mut limit_causes, [ProofLimitCause::DefenderReplyUnknown]);
        let principal_line = first_forced_principal_line(&outcomes);
        let mut evidence = vec![threat.evidence(ThreatEvidenceInput {
            attribution,
            reply_classification: ReplyClassification::Unknown,
            escape_replies: Vec::new(),
            forced_replies: forced_corridor_replies(&outcomes),
            next_forcing_move: next_attacker_move_after_defender_reply(&principal_line),
            proof_status: ProofStatus::Unknown,
            limit_causes: vec![ProofLimitCause::DefenderReplyUnknown],
        })];
        evidence.extend(first_corridor_branch_evidence(
            &outcomes,
            CorridorReplyStatus::Unknown,
        ));
        return with_limit_causes(
            corridor_proof_result(
                board,
                attacker,
                options,
                ProofStatus::Unknown,
                principal_line,
                Vec::new(),
                evidence,
            ),
            limit_causes,
        );
    }

    let principal_line = first_forced_principal_line(&outcomes);
    let mut evidence = vec![threat.evidence(ThreatEvidenceInput {
        attribution,
        reply_classification: ReplyClassification::BlockedButForced,
        escape_replies: Vec::new(),
        forced_replies: forced_corridor_replies(&outcomes),
        next_forcing_move: next_attacker_move_after_defender_reply(&principal_line),
        proof_status: ProofStatus::ForcedWin,
        limit_causes: Vec::new(),
    })];
    evidence.extend(first_corridor_branch_evidence(
        &outcomes,
        CorridorReplyStatus::Forced,
    ));
    corridor_proof_result(
        board,
        attacker,
        options,
        ProofStatus::ForcedWin,
        principal_line.clone(),
        Vec::new(),
        evidence,
    )
}

fn classify_corridor_reply(
    board: &Board,
    attacker: Color,
    options: &AnalysisOptions,
    mv: Move,
) -> CorridorReplyOutcome {
    let mut next = board.clone();
    let applied = next.apply_move(mv).is_ok();
    let proof = if !applied {
        with_limit_causes(
            corridor_proof_result(
                board,
                attacker,
                options,
                ProofStatus::Unknown,
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ),
            [ProofLimitCause::ModelScopeUnknown],
        )
    } else {
        let reply_proof = classify_defender_reply_for_report(board, attacker, mv, options);
        let status = match reply_proof.outcome {
            DefenderReplyOutcome::ForcedLoss | DefenderReplyOutcome::ImmediateLoss => {
                ProofStatus::ForcedWin
            }
            DefenderReplyOutcome::ConfirmedEscape | DefenderReplyOutcome::PossibleEscape => {
                ProofStatus::EscapeFound
            }
            DefenderReplyOutcome::Unknown => ProofStatus::Unknown,
        };
        let escape_moves = match reply_proof.outcome {
            DefenderReplyOutcome::ConfirmedEscape | DefenderReplyOutcome::PossibleEscape => {
                vec![mv]
            }
            _ => Vec::new(),
        };
        with_limit_causes(
            corridor_proof_result(
                &next,
                attacker,
                options,
                status,
                reply_proof.principal_line,
                escape_moves,
                Vec::new(),
            ),
            reply_proof.limit_causes,
        )
    };
    let status = match proof.status {
        ProofStatus::ForcedWin => CorridorReplyStatus::Forced,
        ProofStatus::EscapeFound if applied && proof_has_limit_hit(&proof) => {
            CorridorReplyStatus::PossibleEscape
        }
        ProofStatus::EscapeFound => CorridorReplyStatus::ConfirmedEscape,
        ProofStatus::Unknown if applied => CorridorReplyStatus::PossibleEscape,
        ProofStatus::Unknown => CorridorReplyStatus::Unknown,
    };
    CorridorReplyOutcome { mv, status, proof }
}

pub(crate) fn classify_actual_corridor_reply(
    board: &Board,
    actual_moves: &[Move],
    attacker: Color,
    options: &AnalysisOptions,
    prefix_ply: usize,
    mv: Move,
    actual_child: Option<&ProofResult>,
) -> CorridorReplyOutcome {
    let mut next = board.clone();
    if next.apply_move(mv).is_err() {
        return CorridorReplyOutcome {
            mv,
            status: CorridorReplyStatus::Unknown,
            proof: with_limit_causes(
                corridor_proof_result(
                    board,
                    attacker,
                    options,
                    ProofStatus::Unknown,
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                ),
                [ProofLimitCause::ModelScopeUnknown],
            ),
        };
    }

    let proof = actual_child.cloned().unwrap_or_else(|| {
        replay_corridor_status(&next, actual_moves, attacker, options, prefix_ply + 1)
    });
    let status = match proof.status {
        ProofStatus::ForcedWin => CorridorReplyStatus::Forced,
        ProofStatus::EscapeFound => CorridorReplyStatus::ConfirmedEscape,
        ProofStatus::Unknown if proof_has_limit_hit(&proof) => CorridorReplyStatus::PossibleEscape,
        ProofStatus::Unknown => CorridorReplyStatus::Unknown,
    };
    CorridorReplyOutcome { mv, status, proof }
}

fn forced_corridor_replies(outcomes: &[CorridorReplyOutcome]) -> Vec<Move> {
    outcomes
        .iter()
        .filter_map(|outcome| (outcome.status == CorridorReplyStatus::Forced).then_some(outcome.mv))
        .collect()
}

fn first_forced_principal_line(outcomes: &[CorridorReplyOutcome]) -> Vec<Move> {
    outcomes
        .iter()
        .find(|outcome| outcome.status == CorridorReplyStatus::Forced)
        .map(|outcome| {
            let mut line = Vec::with_capacity(outcome.proof.principal_line.len() + 1);
            line.push(outcome.mv);
            line.extend(outcome.proof.principal_line.clone());
            line
        })
        .unwrap_or_default()
}

fn first_corridor_branch_evidence(
    outcomes: &[CorridorReplyOutcome],
    status: CorridorReplyStatus,
) -> Vec<ThreatSequenceEvidence> {
    outcomes
        .iter()
        .find(|outcome| outcome.status == status)
        .map(|outcome| outcome.proof.threat_evidence.clone())
        .unwrap_or_default()
}

pub(crate) fn corridor_defender_reply_moves(
    board: &Board,
    actual_moves: &[Move],
    prefix_ply: usize,
    options: &AnalysisOptions,
    threat: &ThreatReplySet,
) -> Vec<Move> {
    let mut replies = Vec::new();
    for candidate in bot_corridor::probed_defender_reply_candidates(
        board,
        threat.attacker,
        actual_moves.get(prefix_ply).copied(),
    ) {
        push_unique_move(&mut replies, candidate.mv);
    }

    if threat.winning_squares.is_empty() {
        if let Some(mv) = next_actual_attacker_corridor_move(
            board,
            actual_moves,
            prefix_ply,
            threat.attacker,
            options,
        ) {
            push_unique_move(&mut replies, mv);
        }
    }
    replies
}

fn next_actual_attacker_corridor_move(
    board: &Board,
    actual_moves: &[Move],
    prefix_ply: usize,
    attacker: Color,
    options: &AnalysisOptions,
) -> Option<Move> {
    let defender = attacker.opponent();
    let defender_reply = actual_moves.get(prefix_ply).copied()?;
    let attacker_move = actual_moves.get(prefix_ply + 1).copied()?;
    if !board.is_legal_for_color(attacker_move, defender) {
        return None;
    }

    let mut next = board.clone();
    next.apply_move(defender_reply).ok()?;
    if next.current_player != attacker {
        return None;
    }
    is_corridor_attacker_move(&next, attacker, attacker_move, options).then_some(attacker_move)
}

fn is_corridor_attacker_move(
    board: &Board,
    attacker: Color,
    mv: Move,
    _options: &AnalysisOptions,
) -> bool {
    bot_corridor::is_corridor_attacker_move(board, attacker, mv)
}

fn push_unique_move(moves: &mut Vec<Move>, mv: Move) {
    if !moves.contains(&mv) {
        moves.push(mv);
    }
}
