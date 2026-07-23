use super::*;

pub(super) fn entry_from_analysis(
    path: String,
    analysis: GameAnalysis,
    move_count: usize,
    elapsed_ms: u64,
    replay: Option<&Replay>,
) -> AnalysisBatchEntry {
    let prefixes_analyzed = analysis.proof_summary.len();
    let forced_prefix_count = count_proof_status(&analysis, ProofStatus::ForcedWin);
    let unknown_prefix_count = count_proof_status(&analysis, ProofStatus::Unknown);
    let escape_prefix_count = count_proof_status(&analysis, ProofStatus::EscapeFound);
    let proof_details = replay.and_then(|replay| proof_details_from_analysis(replay, &analysis));
    let proof_detail_diagnostics = proof_details.as_ref().map(proof_details_diagnostics);
    let limit_causes = analysis
        .unclear_context
        .as_ref()
        .map(|context| context.previous_limit_causes.clone())
        .unwrap_or_default();

    AnalysisBatchEntry {
        path,
        status: AnalysisBatchEntryStatus::Analyzed,
        winner: analysis.winner,
        move_count: Some(move_count),
        root_cause: Some(analysis.root_cause),
        unclear_reason: analysis.unclear_reason,
        final_move: analysis.final_move,
        lethal_onset: analysis.lethal_onset,
        setup_corridor: analysis.setup_corridor,
        final_forced_interval_found: analysis.final_forced_interval_found,
        final_forced_interval: Some(analysis.final_forced_interval),
        proof_intervals: analysis.proof_intervals,
        last_chance_ply: analysis.last_chance_ply,
        critical_loser_ply: analysis.critical_loser_ply,
        tactical_notes: analysis.tactical_notes,
        failure: analysis.failure,
        principal_line: analysis.principal_line,
        unknown_gaps: analysis.unknown_gaps.clone(),
        unknown_gap_count: analysis.unknown_gaps.len(),
        unclear_context: analysis.unclear_context,
        proof_details,
        proof_detail_diagnostics,
        limit_causes,
        elapsed_ms,
        prefixes_analyzed,
        forced_prefix_count,
        unknown_prefix_count,
        escape_prefix_count,
        error: None,
    }
}

pub(super) fn proof_details_from_analysis(
    replay: &Replay,
    analysis: &GameAnalysis,
) -> Option<AnalysisBatchProofDetails> {
    analysis.winner?;
    if analysis.proof_summary.is_empty() {
        return None;
    }

    let boards = replay_prefix_boards(replay).ok()?;
    let scan_start = boards.len().checked_sub(analysis.proof_summary.len())?;
    let final_forced_start_ply = analysis.final_forced_interval.start_ply;
    let previous_prefix_ply = final_forced_start_ply.checked_sub(1);
    let previous_proof_result = previous_prefix_ply
        .and_then(|ply| proof_result_at(&analysis.proof_summary, scan_start, ply));
    let final_start_proof_result =
        proof_result_at(&analysis.proof_summary, scan_start, final_forced_start_ply);
    let previous_proof = previous_prefix_ply
        .zip(previous_proof_result)
        .map(|(ply, proof)| proof_snapshot(ply, proof));
    let final_start_proof =
        final_start_proof_result.map(|proof| proof_snapshot(final_forced_start_ply, proof));
    let mut snapshots = Vec::new();
    if let Some(previous_prefix_ply) = previous_prefix_ply {
        if let Some(board) = boards.get(previous_prefix_ply) {
            snapshots.push(board_snapshot(
                "escape_boundary",
                previous_prefix_ply,
                board,
            ));
        }
    }
    if snapshots
        .iter()
        .all(|snapshot| snapshot.ply != final_forced_start_ply)
    {
        if let Some(board) = boards.get(final_forced_start_ply) {
            snapshots.push(board_snapshot(
                "forced_entry",
                final_forced_start_ply,
                board,
            ));
        }
    }
    let proof_frames = proof_frames_for_actual_interval(replay, &boards, analysis, scan_start);

    Some(AnalysisBatchProofDetails {
        previous_prefix_ply,
        final_forced_start_ply,
        previous_proof,
        final_start_proof,
        snapshots,
        proof_frames,
    })
}

pub(super) fn proof_details_diagnostics(details: &AnalysisBatchProofDetails) -> SearchDiagnostics {
    let mut diagnostics = SearchDiagnostics::default();
    for reply in details
        .proof_frames
        .iter()
        .flat_map(|frame| frame.reply_outcomes.iter())
    {
        diagnostics.search_nodes += reply.diagnostics.search_nodes;
        diagnostics.branch_probes += reply.diagnostics.branch_probes;
        diagnostics.max_depth_reached = diagnostics
            .max_depth_reached
            .max(reply.diagnostics.max_depth_reached);
    }
    diagnostics
}

pub(super) fn proof_result_at(
    proofs: &[ProofResult],
    scan_start: usize,
    prefix_ply: usize,
) -> Option<&ProofResult> {
    proofs.get(prefix_ply.checked_sub(scan_start)?)
}

pub(super) fn proof_snapshot(prefix_ply: usize, proof: &ProofResult) -> AnalysisBatchProofSnapshot {
    let mut escape_replies = proof.escape_moves.clone();
    extend_unique_moves(
        &mut escape_replies,
        proof
            .threat_evidence
            .iter()
            .flat_map(|evidence| evidence.escape_replies.iter().copied()),
    );

    AnalysisBatchProofSnapshot {
        prefix_ply,
        attacker: proof.attacker,
        side_to_move: proof.side_to_move,
        status: proof.status,
        reply_classification: proof
            .threat_evidence
            .first()
            .map(|evidence| evidence.reply_classification),
        winning_squares: collect_evidence_moves(proof, |evidence| &evidence.winning_squares),
        legal_cost_squares: collect_evidence_moves(proof, |evidence| &evidence.legal_cost_squares),
        illegal_cost_squares: collect_evidence_moves(proof, |evidence| {
            &evidence.illegal_cost_squares
        }),
        defender_immediate_wins: collect_evidence_moves(proof, |evidence| {
            &evidence.defender_immediate_wins
        }),
        escape_replies,
        forced_replies: collect_evidence_moves(proof, |evidence| &evidence.forced_replies),
        principal_line: proof.principal_line.clone(),
        principal_line_notation: proof
            .principal_line
            .iter()
            .map(|mv| mv.to_notation())
            .collect(),
        limit_hit: proof.limit_hit,
        limit_causes: proof.limit_causes.clone(),
    }
}

pub(super) fn proof_frames_for_actual_interval(
    replay: &Replay,
    boards: &[Board],
    analysis: &GameAnalysis,
    scan_start: usize,
) -> Vec<AnalysisBatchProofFrame> {
    let first_ply = proof_frame_start_ply(boards, analysis);
    let replay_annotations = replay_frame_annotations_for_analysis(replay, analysis)
        .unwrap_or_default()
        .into_iter()
        .map(|frame| (frame.ply, frame))
        .collect::<BTreeMap<_, _>>();
    let plys = (first_ply..=analysis.final_forced_interval.end_ply)
        .rev()
        .collect::<Vec<_>>();

    plys.into_iter()
        .filter_map(|ply| {
            let board_ply = ply.checked_sub(1)?;
            let board = boards.get(board_ply)?;
            let proof = proof_result_at(&analysis.proof_summary, scan_start, board_ply);
            let label = actual_frame_label(ply, &analysis.final_forced_interval);
            let mut markers = Vec::new();
            let actual_move = actual_move_at_ply(replay, ply);
            let lethal_onset_reached = lethal_onset_reached_for_frame(analysis, board_ply);
            let reply_candidates =
                defender_reply_candidates_for_frame(board, analysis, actual_move);
            let reply_outcomes = defender_reply_outcomes_for_frame(board, analysis, actual_move);
            if reply_candidates.is_empty() {
                add_loser_tactical_hint_markers(&mut markers, board, analysis.winner);
            } else {
                add_loser_candidate_markers(
                    &mut markers,
                    board,
                    analysis.winner,
                    &reply_candidates,
                );
            }
            add_reply_outcome_markers(&mut markers, &reply_outcomes);
            add_pre_corridor_escape_marker(
                &mut markers,
                PreCorridorEscapeMarkerInput {
                    replay,
                    analysis,
                    ply,
                    board,
                    proof,
                    previous_proof: board_ply.checked_sub(1).and_then(|previous| {
                        proof_result_at(&analysis.proof_summary, scan_start, previous)
                    }),
                    reply_outcomes: &reply_outcomes,
                },
            );
            if let Some(actual_move) = actual_move {
                add_actual_marker(&mut markers, board, analysis.winner, actual_move);
            }
            if let Some(annotation) = replay_annotations.get(&board_ply) {
                add_replay_annotation_markers(&mut markers, annotation);
            }
            markers.sort_by_key(|marker| (marker.mv.row, marker.mv.col));
            Some(proof_frame(ProofFrameInput {
                label: &label,
                ply,
                board,
                status: proof
                    .map(|proof| proof.status)
                    .unwrap_or(ProofStatus::Unknown),
                move_played: actual_move,
                lethal_onset_reached,
                markers,
                reply_outcomes,
            }))
        })
        .collect::<Vec<_>>()
}

pub(super) fn proof_frame_start_ply(boards: &[Board], analysis: &GameAnalysis) -> usize {
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

pub(super) fn defender_reply_outcomes_for_frame(
    board: &Board,
    analysis: &GameAnalysis,
    actual_move: Option<Move>,
) -> Vec<DefenderReplyAnalysis> {
    let Some(attacker) = analysis.winner else {
        return Vec::new();
    };
    if board.current_player != attacker.opponent() {
        return Vec::new();
    }

    analyze_alternate_defender_reply_options(
        board,
        attacker,
        actual_move,
        &AnalysisOptions {
            max_depth: analysis.model.max_depth,
            max_scan_plies: analysis.model.max_scan_plies,
        },
    )
}

pub(super) fn defender_reply_candidates_for_frame(
    board: &Board,
    analysis: &GameAnalysis,
    actual_move: Option<Move>,
) -> Vec<DefenderReplyCandidate> {
    let Some(attacker) = analysis.winner else {
        return Vec::new();
    };
    if board.current_player != attacker.opponent() {
        return Vec::new();
    }

    visible_defender_reply_candidates(board, attacker, actual_move)
}

pub(super) fn actual_frame_label(ply: usize, interval: &ForcedInterval) -> String {
    if ply == interval.end_ply {
        "winning_ply".to_string()
    } else {
        format!("actual_ply_{ply}")
    }
}

pub(super) fn actual_move_at_ply(replay: &Replay, ply: usize) -> Option<Move> {
    let replay_move = replay.moves.get(ply.checked_sub(1)?)?;
    Move::from_notation(&replay_move.mv).ok()
}

pub(super) fn add_loser_tactical_hint_markers(
    markers: &mut Vec<AnalysisBatchProofMarker>,
    board: &Board,
    winner: Option<Color>,
) {
    let Some(winner) = winner else {
        return;
    };
    if board.current_player != winner.opponent() {
        return;
    }

    add_marker_kind(
        markers,
        board.immediate_winning_moves_for(board.current_player),
        AnalysisBatchProofMarkerKind::Winning,
    );
    add_marker_kind(
        markers,
        board.immediate_winning_moves_for(board.current_player.opponent()),
        AnalysisBatchProofMarkerKind::Threat,
    );
    if has_immediate_tactical_hint(markers) {
        return;
    }
    add_current_imminent_response_markers(markers, board, winner);
}

pub(super) fn add_loser_candidate_markers(
    markers: &mut Vec<AnalysisBatchProofMarker>,
    board: &Board,
    winner: Option<Color>,
    candidates: &[DefenderReplyCandidate],
) {
    let Some(winner) = winner else {
        return;
    };
    if board.current_player != winner.opponent() {
        return;
    }

    let defender = winner.opponent();
    for candidate in candidates {
        for role in &candidate.roles {
            if let Some(kind) = marker_kind_for_defender_reply_role(*role, None) {
                add_marker_kind(markers, [candidate.mv], kind);
            }
        }
        if !board.is_legal_for_color(candidate.mv, defender) {
            add_marker_kind(
                markers,
                [candidate.mv],
                AnalysisBatchProofMarkerKind::Forbidden,
            );
        }
    }
}

pub(super) fn has_immediate_tactical_hint(markers: &[AnalysisBatchProofMarker]) -> bool {
    markers.iter().any(|marker| {
        marker.kinds.iter().any(|kind| {
            matches!(
                kind,
                AnalysisBatchProofMarkerKind::Winning | AnalysisBatchProofMarkerKind::Threat
            )
        })
    })
}

pub(super) fn add_current_imminent_response_markers(
    markers: &mut Vec<AnalysisBatchProofMarker>,
    board: &Board,
    attacker: Color,
) {
    let defender = attacker.opponent();
    for fact in corridor_active_threats(board, attacker)
        .into_iter()
        .filter(|fact| {
            matches!(
                fact.kind,
                LocalThreatKind::OpenThree | LocalThreatKind::BrokenThree
            )
        })
    {
        for mv in fact.defense_squares.iter().copied() {
            if !board.is_empty(mv.row, mv.col) {
                continue;
            }
            add_marker_kind(markers, [mv], AnalysisBatchProofMarkerKind::ImminentDefense);
            if !board.is_legal_for_color(mv, defender) {
                add_marker_kind(markers, [mv], AnalysisBatchProofMarkerKind::Forbidden);
            }
        }
    }
}

pub(super) fn add_replay_annotation_markers(
    markers: &mut Vec<AnalysisBatchProofMarker>,
    annotation: &ReplayFrameAnnotations,
) {
    for highlight in &annotation.evidence {
        if let Some(kind) = evidence_marker_kind_for_replay_highlight(highlight.role) {
            add_marker_kind(markers, [highlight.mv], kind);
        }
    }
    for highlight in &annotation.highlights {
        add_marker_kind(
            markers,
            [highlight.mv],
            marker_kind_for_replay_highlight(highlight.role, highlight.side),
        );
    }
    for marker in &annotation.markers {
        add_marker_kind(
            markers,
            [marker.mv],
            marker_kind_for_replay_marker(marker.role),
        );
    }
}

pub(super) fn marker_kind_for_replay_highlight(
    role: ReplayFrameHighlightRole,
    side: Color,
) -> AnalysisBatchProofMarkerKind {
    match role {
        ReplayFrameHighlightRole::ImmediateWin => AnalysisBatchProofMarkerKind::Winning,
        ReplayFrameHighlightRole::ImmediateThreat => AnalysisBatchProofMarkerKind::Threat,
        ReplayFrameHighlightRole::ImminentThreat => AnalysisBatchProofMarkerKind::ImminentDefense,
        ReplayFrameHighlightRole::CounterThreat => AnalysisBatchProofMarkerKind::OffensiveCounter,
        ReplayFrameHighlightRole::CorridorEntry => corridor_entry_marker_kind(side),
    }
}

pub(super) fn evidence_marker_kind_for_replay_highlight(
    role: ReplayFrameHighlightRole,
) -> Option<AnalysisBatchProofMarkerKind> {
    match role {
        ReplayFrameHighlightRole::ImmediateWin => {
            Some(AnalysisBatchProofMarkerKind::WinningEvidence)
        }
        ReplayFrameHighlightRole::ImmediateThreat => {
            Some(AnalysisBatchProofMarkerKind::ThreatEvidence)
        }
        ReplayFrameHighlightRole::ImminentThreat => {
            Some(AnalysisBatchProofMarkerKind::ImminentEvidence)
        }
        ReplayFrameHighlightRole::CounterThreat => {
            Some(AnalysisBatchProofMarkerKind::OffensiveEvidence)
        }
        ReplayFrameHighlightRole::CorridorEntry => None,
    }
}

pub(super) fn marker_kind_for_replay_marker(
    role: ReplayFrameMarkerRole,
) -> AnalysisBatchProofMarkerKind {
    match role {
        ReplayFrameMarkerRole::ConfirmedEscape => AnalysisBatchProofMarkerKind::ConfirmedEscape,
        ReplayFrameMarkerRole::PossibleEscape => AnalysisBatchProofMarkerKind::PossibleEscape,
        ReplayFrameMarkerRole::ForcedLoss => AnalysisBatchProofMarkerKind::ForcedLoss,
        ReplayFrameMarkerRole::ImmediateLoss => AnalysisBatchProofMarkerKind::ImmediateLoss,
        ReplayFrameMarkerRole::Forbidden => AnalysisBatchProofMarkerKind::Forbidden,
        ReplayFrameMarkerRole::Unknown => AnalysisBatchProofMarkerKind::UnknownOutcome,
    }
}

pub(super) struct ProofFrameInput<'a> {
    label: &'a str,
    ply: usize,
    board: &'a Board,
    status: ProofStatus,
    move_played: Option<Move>,
    lethal_onset_reached: bool,
    markers: Vec<AnalysisBatchProofMarker>,
    reply_outcomes: Vec<DefenderReplyAnalysis>,
}

pub(super) fn proof_frame(input: ProofFrameInput<'_>) -> AnalysisBatchProofFrame {
    AnalysisBatchProofFrame {
        label: input.label.to_string(),
        ply: input.ply,
        side_to_move: input.board.current_player,
        status: input.status,
        move_played: input.move_played,
        move_played_notation: input.move_played.map(Move::to_notation),
        lethal_onset_reached: input.lethal_onset_reached,
        rows: board_rows(input.board),
        markers: input.markers,
        reply_outcomes: input.reply_outcomes,
    }
}

pub(super) fn lethal_onset_reached_for_frame(analysis: &GameAnalysis, board_ply: usize) -> bool {
    analysis
        .lethal_onset
        .as_ref()
        .is_some_and(|onset| board_ply >= onset.prefix_ply)
}

pub(super) fn add_reply_outcome_markers(
    markers: &mut Vec<AnalysisBatchProofMarker>,
    replies: &[DefenderReplyAnalysis],
) {
    for reply in replies {
        for role in &reply.roles {
            if let Some(kind) = marker_kind_for_defender_reply_role(
                *role,
                Some(AnalysisBatchProofMarkerKind::Actual),
            ) {
                add_marker_kind(markers, [reply.mv], kind);
            }
        }
        add_marker_kind(
            markers,
            [reply.mv],
            marker_kind_for_defender_reply_outcome(reply.outcome),
        );
    }
}

pub(super) struct PreCorridorEscapeMarkerInput<'a> {
    pub(super) replay: &'a Replay,
    pub(super) analysis: &'a GameAnalysis,
    pub(super) ply: usize,
    pub(super) board: &'a Board,
    pub(super) proof: Option<&'a ProofResult>,
    pub(super) previous_proof: Option<&'a ProofResult>,
    pub(super) reply_outcomes: &'a [DefenderReplyAnalysis],
}

pub(super) fn add_pre_corridor_escape_marker(
    markers: &mut Vec<AnalysisBatchProofMarker>,
    input: PreCorridorEscapeMarkerInput<'_>,
) {
    let Some(winner) = input.analysis.winner else {
        return;
    };
    if input.board.current_player != winner.opponent()
        || !input.reply_outcomes.is_empty()
        || has_visible_tactical_hint(markers)
    {
        return;
    }

    let Some(entry_move) = pre_corridor_escape_entry_move(
        input.replay,
        input.analysis,
        input.ply,
        input.proof,
        input.previous_proof,
    ) else {
        return;
    };
    if !input.board.is_legal(entry_move) {
        return;
    }

    add_marker_kind(markers, [entry_move], corridor_entry_marker_kind(winner));
    add_marker_kind(
        markers,
        [entry_move],
        AnalysisBatchProofMarkerKind::ConfirmedEscape,
    );
}

pub(super) fn pre_corridor_escape_entry_move(
    replay: &Replay,
    analysis: &GameAnalysis,
    ply: usize,
    proof: Option<&ProofResult>,
    previous_proof: Option<&ProofResult>,
) -> Option<Move> {
    if ply == analysis.final_forced_interval.start_ply
        && proof.map(|proof| proof.status) == Some(ProofStatus::EscapeFound)
    {
        return actual_move_at_ply(replay, ply + 1);
    }

    if ply == analysis.final_forced_interval.start_ply + 1
        && previous_proof.map(|proof| proof.status) == Some(ProofStatus::EscapeFound)
        && proof.map(|proof| proof.status) == Some(ProofStatus::ForcedWin)
    {
        return proof.and_then(|proof| proof.principal_line.first().copied());
    }

    None
}

pub(super) fn corridor_entry_marker_kind(winner: Color) -> AnalysisBatchProofMarkerKind {
    match winner {
        Color::Black => AnalysisBatchProofMarkerKind::CorridorEntryBlack,
        Color::White => AnalysisBatchProofMarkerKind::CorridorEntryWhite,
    }
}

pub(super) fn has_visible_tactical_hint(markers: &[AnalysisBatchProofMarker]) -> bool {
    markers
        .iter()
        .flat_map(|marker| marker.kinds.iter().copied())
        .any(is_hint_marker_kind)
}

pub(super) fn add_actual_marker(
    markers: &mut Vec<AnalysisBatchProofMarker>,
    board: &Board,
    winner: Option<Color>,
    mv: Move,
) {
    add_actual_hint_markers(markers, board, winner, mv);
    let marker = upsert_marker(markers, mv);
    marker.kinds.retain(|kind| is_hint_marker_kind(*kind));
    if !marker.kinds.contains(&AnalysisBatchProofMarkerKind::Actual) {
        marker.kinds.push(AnalysisBatchProofMarkerKind::Actual);
    }
}

pub(super) fn add_actual_hint_markers(
    markers: &mut Vec<AnalysisBatchProofMarker>,
    board: &Board,
    winner: Option<Color>,
    mv: Move,
) {
    let Some(winner) = winner else {
        return;
    };
    if board.current_player != winner.opponent() {
        return;
    }

    for role in defender_reply_roles_for_move(board, winner, mv) {
        if let Some(kind) = marker_kind_for_defender_reply_role(role, None) {
            add_marker_kind(markers, [mv], kind);
        }
    }
}

pub(super) fn marker_kind_for_defender_reply_role(
    role: DefenderReplyRole,
    actual_kind: Option<AnalysisBatchProofMarkerKind>,
) -> Option<AnalysisBatchProofMarkerKind> {
    match role {
        DefenderReplyRole::Actual => actual_kind,
        DefenderReplyRole::ImmediateDefense => Some(AnalysisBatchProofMarkerKind::Threat),
        DefenderReplyRole::ImminentDefense => Some(AnalysisBatchProofMarkerKind::ImminentDefense),
        DefenderReplyRole::OffensiveCounter => Some(AnalysisBatchProofMarkerKind::OffensiveCounter),
    }
}

pub(super) fn marker_kind_for_defender_reply_outcome(
    outcome: DefenderReplyOutcome,
) -> AnalysisBatchProofMarkerKind {
    match outcome {
        DefenderReplyOutcome::ForcedLoss => AnalysisBatchProofMarkerKind::ForcedLoss,
        DefenderReplyOutcome::ConfirmedEscape => AnalysisBatchProofMarkerKind::ConfirmedEscape,
        DefenderReplyOutcome::PossibleEscape => AnalysisBatchProofMarkerKind::PossibleEscape,
        DefenderReplyOutcome::ImmediateLoss => AnalysisBatchProofMarkerKind::ImmediateLoss,
        DefenderReplyOutcome::Unknown => AnalysisBatchProofMarkerKind::UnknownOutcome,
    }
}

pub(super) fn is_hint_marker_kind(kind: AnalysisBatchProofMarkerKind) -> bool {
    matches!(
        kind,
        AnalysisBatchProofMarkerKind::Winning
            | AnalysisBatchProofMarkerKind::Threat
            | AnalysisBatchProofMarkerKind::ImminentDefense
            | AnalysisBatchProofMarkerKind::OffensiveCounter
            | AnalysisBatchProofMarkerKind::CorridorEntryBlack
            | AnalysisBatchProofMarkerKind::CorridorEntryWhite
    )
}

pub(super) fn add_marker_kind(
    markers: &mut Vec<AnalysisBatchProofMarker>,
    moves: impl IntoIterator<Item = Move>,
    kind: AnalysisBatchProofMarkerKind,
) {
    for mv in moves {
        let marker = upsert_marker(markers, mv);
        if !marker.kinds.contains(&kind) {
            marker.kinds.push(kind);
        }
    }
}

pub(super) fn upsert_marker(
    markers: &mut Vec<AnalysisBatchProofMarker>,
    mv: Move,
) -> &mut AnalysisBatchProofMarker {
    if let Some(index) = markers.iter().position(|marker| marker.mv == mv) {
        return &mut markers[index];
    }
    markers.push(AnalysisBatchProofMarker {
        mv,
        notation: mv.to_notation(),
        kinds: Vec::new(),
    });
    markers
        .last_mut()
        .expect("marker was just pushed and must exist")
}

pub(super) fn collect_evidence_moves(
    proof: &ProofResult,
    selector: fn(&crate::analysis::ThreatSequenceEvidence) -> &[Move],
) -> Vec<Move> {
    let mut moves = Vec::new();
    extend_unique_moves(
        &mut moves,
        proof
            .threat_evidence
            .iter()
            .flat_map(|evidence| selector(evidence).iter().copied()),
    );
    moves
}

pub(super) fn extend_unique_moves(target: &mut Vec<Move>, moves: impl IntoIterator<Item = Move>) {
    for mv in moves {
        if !target.contains(&mv) {
            target.push(mv);
        }
    }
}

pub(super) fn replay_prefix_boards(replay: &Replay) -> Result<Vec<Board>, String> {
    let mut board = Board::new(replay.rules.clone());
    let mut boards = vec![board.clone()];
    for (idx, replay_move) in replay.moves.iter().enumerate() {
        let ply = idx + 1;
        let mv = Move::from_notation(&replay_move.mv)
            .map_err(|message| format!("invalid replay move at ply {ply}: {message}"))?;
        board
            .apply_move(mv)
            .map_err(|err| format!("invalid replay move at ply {ply}: {err}"))?;
        boards.push(board.clone());
    }
    Ok(boards)
}

pub(super) fn board_snapshot(label: &str, ply: usize, board: &Board) -> AnalysisBoardSnapshot {
    AnalysisBoardSnapshot {
        label: label.to_string(),
        ply,
        side_to_move: board.current_player,
        rows: board_rows(board),
    }
}

pub(super) fn board_rows(board: &Board) -> Vec<String> {
    let size = board.config.board_size;
    (0..size)
        .map(|row| {
            (0..size)
                .map(|col| board.cell(row, col).map_or('.', Color::to_char))
                .collect()
        })
        .collect()
}

pub(super) fn limit_cause_counts(entries: &[AnalysisBatchEntry]) -> Vec<ProofLimitCauseCount> {
    let mut counts = BTreeMap::<ProofLimitCause, usize>::new();
    for entry in entries {
        for cause in &entry.limit_causes {
            *counts.entry(*cause).or_default() += 1;
        }
    }
    counts
        .into_iter()
        .map(|(cause, count)| ProofLimitCauseCount { cause, count })
        .collect()
}

pub(super) fn count_proof_status(analysis: &GameAnalysis, status: ProofStatus) -> usize {
    analysis
        .proof_summary
        .iter()
        .filter(|proof| proof.status == status)
        .count()
}

pub(super) fn increment_summary_from_entry(
    summary: &mut AnalysisBatchSummary,
    entry: &AnalysisBatchEntry,
) {
    if entry.winner.is_none() {
        summary.ongoing_or_draw += 1;
        return;
    }

    if matches!(entry.root_cause, Some(RootCause::Unclear) | None) {
        summary.unclear += 1;
    }
}
