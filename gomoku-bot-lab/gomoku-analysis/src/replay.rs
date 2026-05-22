use crate::annotations::{
    replay_frame_annotations_for_analysis_with_boards, replay_frame_annotations_from_proof,
};
use crate::failure::{failure_analysis, FailureAnalysisInput};
use crate::model::{corridor_analysis_model, rule_label};
use crate::onset::find_lethal_onset;
use crate::trace::replay_corridor_status_with_actual_child;
use crate::types::*;
use gomoku_core::{replay::ReplayResult, Board, Color, GameResult, Move, Replay};

fn replay_analysis_step_status(analysis: &GameAnalysis) -> ReplayAnalysisStepStatus {
    if analysis.winner.is_none() {
        return ReplayAnalysisStepStatus::Unsupported;
    }
    if analysis.root_cause == RootCause::Unclear || !analysis.final_forced_interval_found {
        return ReplayAnalysisStepStatus::Unclear;
    }
    ReplayAnalysisStepStatus::Resolved
}

fn replay_analysis_counters(proof_summary: &[ProofResult]) -> ReplayAnalysisCounters {
    ReplayAnalysisCounters {
        prefixes_analyzed: proof_summary.len(),
        branch_roots: proof_summary
            .iter()
            .map(|proof| proof.threat_evidence.len())
            .sum(),
        proof_nodes: proof_summary
            .iter()
            .map(|proof| proof.principal_line.len())
            .sum(),
    }
}

pub struct ReplayAnalysisSession {
    replay: Replay,
    options: AnalysisOptions,
    boards: Vec<Board>,
    actual_moves: Vec<Move>,
    winner: Option<Color>,
    model: AnalysisModel,
    lower_bound: usize,
    next_ply: Option<usize>,
    actual_child: Option<ProofResult>,
    scan_start: usize,
    proof_summary: Vec<ProofResult>,
    final_analysis: Option<GameAnalysis>,
    final_annotations_emitted: bool,
    emit_annotations: bool,
}

impl ReplayAnalysisSession {
    pub fn new(replay: Replay, options: AnalysisOptions) -> Result<Self, AnalysisError> {
        Self::new_with_annotation_mode(replay, options, true)
    }

    fn new_with_annotation_mode(
        replay: Replay,
        options: AnalysisOptions,
        emit_annotations: bool,
    ) -> Result<Self, AnalysisError> {
        let boards = replay_prefix_boards(&replay)?;
        let final_board = boards
            .last()
            .expect("replay prefixes include initial board");
        let winner = replay_winner(&replay, final_board);
        let model = corridor_analysis_model(final_board, &options);
        let final_analysis = winner
            .is_none()
            .then(|| no_winner_analysis(&replay, final_board, model.clone()));
        let actual_moves = if winner.is_some() {
            replay_moves(&replay)?
        } else {
            Vec::new()
        };
        let lower_bound = options
            .max_scan_plies
            .map(|max_scan_plies| boards.len().saturating_sub(max_scan_plies + 1))
            .unwrap_or(0);
        let next_ply = winner.map(|_| boards.len().saturating_sub(1));
        let scan_start = boards.len();

        Ok(Self {
            replay,
            options,
            boards,
            actual_moves,
            winner,
            model,
            lower_bound,
            next_ply,
            actual_child: None,
            scan_start,
            proof_summary: Vec::new(),
            final_analysis,
            final_annotations_emitted: false,
            emit_annotations,
        })
    }

    pub fn step(&mut self, max_work_units: usize) -> ReplayAnalysisStep {
        let mut annotations = Vec::new();
        let work_units = max_work_units.max(1);

        if self.final_analysis.is_none() {
            for _ in 0..work_units {
                let Some(ply) = self.next_ply else {
                    self.finalize();
                    break;
                };
                let Some(winner) = self.winner else {
                    self.finalize();
                    break;
                };

                let actual_child = self.actual_child.clone();
                let proof = replay_corridor_status_with_actual_child(
                    &self.boards[ply],
                    &self.actual_moves,
                    winner,
                    &self.options,
                    ply,
                    actual_child.as_ref(),
                );
                self.actual_child = Some(proof.clone());
                self.proof_summary.insert(0, proof.clone());
                self.scan_start = ply;
                if self.emit_annotations {
                    annotations.push(replay_frame_annotations_from_proof(
                        ply,
                        &self.boards[ply],
                        winner,
                        &proof,
                        actual_child.as_ref(),
                        self.actual_moves.get(ply).copied(),
                        &self.options,
                    ));
                }

                let boundary_found = final_forced_interval_has_boundary(
                    &self.proof_summary,
                    self.scan_start,
                    self.actual_moves.len(),
                );
                let bounded_scan_reached_boundary =
                    self.options.max_scan_plies.is_some() && boundary_found;
                if bounded_scan_reached_boundary || ply == self.lower_bound {
                    self.finalize();
                    break;
                }

                self.next_ply = ply.checked_sub(1);
            }
        }

        let done = self.final_analysis.is_some();
        if self.emit_annotations && done && !self.final_annotations_emitted {
            if let Some(analysis) = &self.final_analysis {
                annotations.extend(replay_frame_annotations_for_analysis_with_boards(
                    &self.replay,
                    &self.boards,
                    analysis,
                ));
            }
            self.final_annotations_emitted = true;
        }
        ReplayAnalysisStep {
            status: self.step_status(done),
            done,
            current_ply: if done { None } else { self.next_ply },
            annotations,
            analysis: if done {
                self.final_analysis.clone()
            } else {
                None
            },
            counters: replay_analysis_counters(&self.proof_summary),
        }
    }

    fn finalize(&mut self) {
        if self.final_analysis.is_some() {
            return;
        }
        let Some(winner) = self.winner else {
            let final_board = self
                .boards
                .last()
                .expect("replay prefixes include initial board");
            self.final_analysis = Some(no_winner_analysis(
                &self.replay,
                final_board,
                self.model.clone(),
            ));
            return;
        };
        self.final_analysis = Some(finalize_replay_analysis(
            &self.replay,
            &self.boards,
            winner,
            self.model.clone(),
            self.scan_start,
            self.proof_summary.clone(),
        ));
        self.next_ply = None;
    }

    fn step_status(&self, done: bool) -> ReplayAnalysisStepStatus {
        if !done {
            return ReplayAnalysisStepStatus::Running;
        }
        let Some(analysis) = &self.final_analysis else {
            return ReplayAnalysisStepStatus::Running;
        };
        replay_analysis_step_status(analysis)
    }
}

pub fn analyze_replay(
    replay: &Replay,
    options: AnalysisOptions,
) -> Result<GameAnalysis, AnalysisError> {
    let mut session =
        ReplayAnalysisSession::new_with_annotation_mode(replay.clone(), options, false)?;
    loop {
        let step = session.step(usize::MAX);
        if step.done {
            return Ok(step
                .analysis
                .expect("completed replay analysis step includes final analysis"));
        }
    }
}

fn no_winner_analysis(replay: &Replay, final_board: &Board, model: AnalysisModel) -> GameAnalysis {
    let winner = replay_winner(replay, final_board);
    GameAnalysis {
        schema_version: ANALYSIS_SCHEMA_VERSION,
        rule_set: rule_label(&replay.rules.variant).to_string(),
        winner,
        loser: winner.map(Color::opponent),
        final_move: replay
            .moves
            .last()
            .and_then(|mv| Move::from_notation(&mv.mv).ok()),
        final_winning_line: Vec::new(),
        model,
        lethal_onset: None,
        setup_corridor: None,
        final_forced_interval_found: false,
        final_forced_interval: ForcedInterval {
            start_ply: 0,
            end_ply: 0,
        },
        proof_intervals: Vec::new(),
        unknown_gaps: Vec::new(),
        unclear_reason: Some(UnclearReason::DrawOrOngoing),
        unclear_context: None,
        last_chance_ply: None,
        decisive_attack_ply: None,
        critical_loser_ply: None,
        root_cause: RootCause::Unclear,
        failure: None,
        tactical_notes: Vec::new(),
        principal_line: Vec::new(),
        proof_summary: Vec::new(),
    }
}

fn finalize_replay_analysis(
    replay: &Replay,
    boards: &[Board],
    winner: Color,
    model: AnalysisModel,
    scan_start: usize,
    proof_summary: Vec<ProofResult>,
) -> GameAnalysis {
    let final_board = boards
        .last()
        .expect("replay prefixes include initial board");
    let loser = Some(winner.opponent());
    let proof_intervals = proof_intervals(&proof_summary, scan_start);
    let (final_forced_interval_found, final_forced_interval) =
        find_final_forced_interval(&proof_intervals, replay.moves.len());
    let lethal_scan_start = if final_forced_interval_found {
        final_forced_interval.start_ply
    } else {
        scan_start
    };
    let lethal_onset = find_lethal_onset(
        boards,
        winner,
        lethal_scan_start,
        final_forced_interval.end_ply,
    );
    let setup_corridor = setup_corridor_interval(
        final_forced_interval_found,
        &final_forced_interval,
        lethal_onset.as_ref(),
    );
    let unknown_gaps = proof_summary
        .iter()
        .enumerate()
        .filter_map(|(idx, proof)| {
            (proof.status == ProofStatus::Unknown).then_some(scan_start + idx)
        })
        .collect::<Vec<_>>();

    let previous_status = final_forced_interval
        .start_ply
        .checked_sub(1)
        .and_then(|ply| proof_at(&proof_summary, scan_start, ply))
        .map(|proof| proof.status);
    let move_color = color_for_ply(final_forced_interval.start_ply);
    let missed_win_root = loser.is_some_and(|loser| {
        losing_side_missed_immediate_win(replay, boards, final_forced_interval.start_ply, loser)
    });
    let root_cause = classify_root_cause(previous_status, move_color, winner, missed_win_root);
    let last_chance_ply = find_last_chance(
        boards,
        &proof_summary,
        scan_start,
        final_forced_interval.start_ply,
        loser,
    );
    let critical_loser_ply = match root_cause {
        RootCause::MissedDefense | RootCause::MissedWin => Some(final_forced_interval.start_ply),
        _ => None,
    };
    let decisive_attack_ply =
        (move_color == Some(winner)).then_some(final_forced_interval.start_ply);
    let tactical_notes = tactical_notes(TacticalNoteInput {
        replay,
        boards,
        proofs: &proof_summary,
        scan_start,
        proof_intervals: &proof_intervals,
        final_forced_interval: &final_forced_interval,
        winner,
        root_cause,
    });
    let failure = failure_analysis(FailureAnalysisInput {
        replay,
        boards,
        proof_summary: &proof_summary,
        scan_start,
        final_forced_interval_found,
        final_forced_interval: &final_forced_interval,
        lethal_onset: lethal_onset.as_ref(),
        root_cause,
        winner,
        loser: winner.opponent(),
    });
    let principal_line = proof_at(&proof_summary, scan_start, final_forced_interval.start_ply)
        .map(|proof| proof.principal_line.clone())
        .unwrap_or_default();
    let unclear_reason = unclear_reason(UnclearReasonInput {
        root_cause,
        final_forced_interval_found,
        final_forced_interval: &final_forced_interval,
        previous_status,
        proof_summary: &proof_summary,
        scan_start,
    });
    let unclear_context = unclear_context(UnclearContextInput {
        root_cause,
        unclear_reason,
        final_forced_interval: &final_forced_interval,
        proof_summary: &proof_summary,
        scan_start,
        boards,
        winner,
        principal_line: &principal_line,
        move_count: replay.moves.len(),
    });

    GameAnalysis {
        schema_version: ANALYSIS_SCHEMA_VERSION,
        rule_set: rule_label(&replay.rules.variant).to_string(),
        winner: Some(winner),
        loser,
        final_move: replay
            .moves
            .last()
            .and_then(|mv| Move::from_notation(&mv.mv).ok()),
        final_winning_line: final_board.winning_line(),
        model,
        lethal_onset,
        setup_corridor,
        final_forced_interval_found,
        final_forced_interval,
        proof_intervals,
        unknown_gaps,
        unclear_reason,
        unclear_context,
        last_chance_ply,
        decisive_attack_ply,
        critical_loser_ply,
        root_cause,
        failure,
        tactical_notes,
        principal_line,
        proof_summary,
    }
}

pub(crate) fn replay_prefix_boards(replay: &Replay) -> Result<Vec<Board>, AnalysisError> {
    let mut board = Board::new(replay.rules.clone());
    let mut boards = vec![board.clone()];
    for (idx, replay_move) in replay.moves.iter().enumerate() {
        let ply = idx + 1;
        let mv = Move::from_notation(&replay_move.mv)
            .map_err(|message| AnalysisError::InvalidReplayMove { ply, message })?;
        board
            .apply_move(mv)
            .map_err(|err| AnalysisError::InvalidReplayMove {
                ply,
                message: err.to_string(),
            })?;
        boards.push(board.clone());
    }
    Ok(boards)
}

pub(crate) fn replay_moves(replay: &Replay) -> Result<Vec<Move>, AnalysisError> {
    replay
        .moves
        .iter()
        .enumerate()
        .map(|(idx, replay_move)| {
            Move::from_notation(&replay_move.mv).map_err(|message| {
                AnalysisError::InvalidReplayMove {
                    ply: idx + 1,
                    message,
                }
            })
        })
        .collect()
}

fn replay_winner(replay: &Replay, final_board: &Board) -> Option<Color> {
    match final_board.result {
        GameResult::Winner(winner) => Some(winner),
        _ => match replay.result {
            ReplayResult::BlackWins => Some(Color::Black),
            ReplayResult::WhiteWins => Some(Color::White),
            ReplayResult::Draw | ReplayResult::Ongoing => None,
        },
    }
}

#[cfg(test)]
pub(crate) fn replay_proof_summary(
    boards: &[Board],
    actual_moves: &[Move],
    winner: Color,
    options: &AnalysisOptions,
    scan_start: usize,
) -> Vec<ProofResult> {
    let mut proof_summary = Vec::with_capacity(boards.len() - scan_start);
    let mut actual_child = None;
    for ply in (scan_start..boards.len()).rev() {
        let proof = replay_corridor_status_with_actual_child(
            &boards[ply],
            actual_moves,
            winner,
            options,
            ply,
            actual_child.as_ref(),
        );
        actual_child = Some(proof.clone());
        proof_summary.push(proof);
    }
    proof_summary.reverse();
    proof_summary
}

fn final_forced_interval_has_boundary(
    proof_summary: &[ProofResult],
    scan_start: usize,
    move_count: usize,
) -> bool {
    let proof_intervals = proof_intervals(proof_summary, scan_start);
    let (found, interval) = find_final_forced_interval(&proof_intervals, move_count);
    found && interval.start_ply > scan_start
}

fn find_final_forced_interval(
    proof_intervals: &[ForcedInterval],
    move_count: usize,
) -> (bool, ForcedInterval) {
    let found = proof_intervals
        .iter()
        .any(|interval| interval.end_ply == move_count);
    let interval = proof_intervals
        .iter()
        .rev()
        .find(|interval| interval.end_ply == move_count)
        .cloned()
        .unwrap_or(ForcedInterval {
            start_ply: move_count,
            end_ply: move_count,
        });
    (found, interval)
}

fn setup_corridor_interval(
    final_forced_interval_found: bool,
    final_forced_interval: &ForcedInterval,
    lethal_onset: Option<&LethalOnset>,
) -> Option<ForcedInterval> {
    if !final_forced_interval_found {
        return None;
    }

    let onset = lethal_onset?;
    if onset.prefix_ply < final_forced_interval.start_ply
        || onset.prefix_ply > final_forced_interval.end_ply
    {
        return None;
    }

    Some(ForcedInterval {
        start_ply: final_forced_interval.start_ply,
        end_ply: onset.prefix_ply,
    })
}

fn proof_intervals(proofs: &[ProofResult], scan_start: usize) -> Vec<ForcedInterval> {
    let mut intervals = Vec::new();
    let mut current_start = None;
    for (idx, proof) in proofs.iter().enumerate() {
        let ply = scan_start + idx;
        if proof.status == ProofStatus::ForcedWin {
            current_start.get_or_insert(ply);
        } else if let Some(start) = current_start.take() {
            intervals.push(ForcedInterval {
                start_ply: start,
                end_ply: ply - 1,
            });
        }
    }
    if let Some(start) = current_start {
        intervals.push(ForcedInterval {
            start_ply: start,
            end_ply: scan_start + proofs.len() - 1,
        });
    }
    intervals
}

pub(crate) fn proof_at(
    proofs: &[ProofResult],
    scan_start: usize,
    ply: usize,
) -> Option<&ProofResult> {
    proofs.get(ply.checked_sub(scan_start)?)
}

pub(crate) fn find_last_chance(
    boards: &[Board],
    proofs: &[ProofResult],
    scan_start: usize,
    before_ply: usize,
    loser: Option<Color>,
) -> Option<usize> {
    let loser = loser?;
    (scan_start..before_ply).rev().find(|&ply| {
        boards[ply].current_player == loser
            && proof_at(proofs, scan_start, ply)
                .is_some_and(|proof| proof.status == ProofStatus::EscapeFound)
    })
}

fn classify_root_cause(
    previous_status: Option<ProofStatus>,
    move_color: Option<Color>,
    winner: Color,
    missed_win_root: bool,
) -> RootCause {
    if missed_win_root {
        return RootCause::MissedWin;
    }
    match (previous_status, move_color) {
        (Some(ProofStatus::EscapeFound), Some(color)) if color == winner.opponent() => {
            RootCause::MissedDefense
        }
        (Some(ProofStatus::EscapeFound), Some(color)) if color == winner => {
            RootCause::CorridorEntry
        }
        _ => RootCause::Unclear,
    }
}

struct UnclearReasonInput<'a> {
    root_cause: RootCause,
    final_forced_interval_found: bool,
    final_forced_interval: &'a ForcedInterval,
    previous_status: Option<ProofStatus>,
    proof_summary: &'a [ProofResult],
    scan_start: usize,
}

fn unclear_reason(input: UnclearReasonInput<'_>) -> Option<UnclearReason> {
    if input.root_cause != RootCause::Unclear {
        return None;
    }
    if !input.final_forced_interval_found {
        return Some(UnclearReason::NoFinalForcedInterval);
    }
    let previous_ply = input.final_forced_interval.start_ply.checked_sub(1);
    let Some(previous_ply) = previous_ply else {
        return Some(UnclearReason::ScanWindowCutoff);
    };
    let Some(previous_proof) = proof_at(input.proof_summary, input.scan_start, previous_ply) else {
        return Some(UnclearReason::ScanWindowCutoff);
    };
    match input.previous_status {
        Some(ProofStatus::Unknown) if proof_has_limit_hit(previous_proof) => {
            Some(UnclearReason::ProofLimitHit)
        }
        Some(ProofStatus::Unknown) => Some(UnclearReason::PreviousPrefixUnknown),
        None => Some(UnclearReason::ScanWindowCutoff),
        _ => Some(UnclearReason::PreviousPrefixUnknown),
    }
}

pub(crate) fn proof_has_limit_hit(proof: &ProofResult) -> bool {
    proof.limit_hit || !proof.limit_causes.is_empty()
}

struct UnclearContextInput<'a> {
    root_cause: RootCause,
    unclear_reason: Option<UnclearReason>,
    final_forced_interval: &'a ForcedInterval,
    proof_summary: &'a [ProofResult],
    scan_start: usize,
    boards: &'a [Board],
    winner: Color,
    principal_line: &'a [Move],
    move_count: usize,
}

fn unclear_context(input: UnclearContextInput<'_>) -> Option<UnclearContext> {
    if input.root_cause != RootCause::Unclear {
        return None;
    }
    let reason = input.unclear_reason?;
    if reason == UnclearReason::DrawOrOngoing {
        return None;
    }

    let previous_prefix_ply = input.final_forced_interval.start_ply.checked_sub(1);
    let previous_proof =
        previous_prefix_ply.and_then(|ply| proof_at(input.proof_summary, input.scan_start, ply));
    let previous_limit_causes = previous_proof
        .map(|proof| proof.limit_causes.clone())
        .unwrap_or_else(|| vec![ProofLimitCause::OutsideScanWindow]);
    let previous_board = previous_prefix_ply.and_then(|ply| input.boards.get(ply));
    let mut snapshots = Vec::new();
    if let (Some(ply), Some(board)) = (previous_prefix_ply, previous_board) {
        snapshots.push(board_snapshot("previous_prefix", ply, board));
    }
    if snapshots
        .iter()
        .all(|snapshot| snapshot.ply != input.final_forced_interval.start_ply)
    {
        if let Some(board) = input.boards.get(input.final_forced_interval.start_ply) {
            snapshots.push(board_snapshot(
                "final_forced_start",
                input.final_forced_interval.start_ply,
                board,
            ));
        }
    }

    Some(UnclearContext {
        reason,
        previous_prefix_ply,
        final_forced_interval: input.final_forced_interval.clone(),
        previous_proof_status: previous_proof.map(|proof| proof.status),
        previous_proof_limit_hit: previous_proof.map(proof_has_limit_hit),
        previous_limit_causes,
        previous_side_to_move: previous_board.map(|board| board.current_player),
        winner: input.winner,
        principal_line: input.principal_line.to_vec(),
        principal_line_notation: input
            .principal_line
            .iter()
            .map(|mv| mv.to_notation())
            .collect(),
        scan_start_ply: input.scan_start,
        scan_end_ply: if input.proof_summary.is_empty() {
            None
        } else {
            Some(input.scan_start + input.proof_summary.len() - 1)
        },
        move_count: input.move_count,
        snapshots,
    })
}

fn board_snapshot(label: &str, ply: usize, board: &Board) -> AnalysisBoardSnapshot {
    AnalysisBoardSnapshot {
        label: label.to_string(),
        ply,
        side_to_move: board.current_player,
        rows: board_rows(board),
    }
}

fn board_rows(board: &Board) -> Vec<String> {
    let size = board.config.board_size;
    (0..size)
        .map(|row| {
            (0..size)
                .map(|col| board.cell(row, col).map_or('.', Color::to_char))
                .collect()
        })
        .collect()
}

struct TacticalNoteInput<'a> {
    replay: &'a Replay,
    boards: &'a [Board],
    proofs: &'a [ProofResult],
    scan_start: usize,
    proof_intervals: &'a [ForcedInterval],
    final_forced_interval: &'a ForcedInterval,
    winner: Color,
    root_cause: RootCause,
}

fn tactical_notes(input: TacticalNoteInput<'_>) -> Vec<TacticalNote> {
    let mut notes = Vec::new();
    if input.root_cause == RootCause::MissedWin {
        push_note(&mut notes, TacticalNote::MissedWin);
    }
    if input
        .proof_intervals
        .iter()
        .any(|interval| interval.end_ply < input.final_forced_interval.start_ply)
    {
        push_note(&mut notes, TacticalNote::ConversionError);
    }
    if missed_forced_win(
        input.replay,
        input.boards,
        input.proofs,
        input.scan_start,
        input.winner,
    ) {
        push_note(&mut notes, TacticalNote::MissedWin);
    }
    if input.root_cause == RootCause::CorridorEntry {
        push_note(&mut notes, TacticalNote::StrongAttack);
    }
    notes
}

fn losing_side_missed_immediate_win(
    replay: &Replay,
    boards: &[Board],
    forced_start_ply: usize,
    loser: Color,
) -> bool {
    let Some(prefix_ply) = forced_start_ply.checked_sub(1) else {
        return false;
    };
    let Some(board) = boards.get(prefix_ply) else {
        return false;
    };
    if board.current_player != loser {
        return false;
    }
    let immediate_wins = board.immediate_winning_moves_for(loser);
    if immediate_wins.is_empty() {
        return false;
    }
    let Some(actual) = replay.moves.get(prefix_ply) else {
        return false;
    };
    let Ok(actual_move) = Move::from_notation(&actual.mv) else {
        return false;
    };
    !immediate_wins.contains(&actual_move)
}

fn missed_forced_win(
    replay: &Replay,
    boards: &[Board],
    proofs: &[ProofResult],
    scan_start: usize,
    winner: Color,
) -> bool {
    for (ply, board) in boards
        .iter()
        .enumerate()
        .take(replay.moves.len())
        .skip(scan_start)
    {
        if board.current_player != winner {
            continue;
        }
        if !proof_at(proofs, scan_start, ply)
            .is_some_and(|proof| proof.status == ProofStatus::ForcedWin)
        {
            continue;
        }
        let immediate_wins = board.immediate_winning_moves_for(winner);
        if immediate_wins.is_empty() {
            continue;
        }
        let Ok(actual_move) = Move::from_notation(&replay.moves[ply].mv) else {
            continue;
        };
        if !immediate_wins.contains(&actual_move) {
            return true;
        }
    }
    false
}

fn push_note(notes: &mut Vec<TacticalNote>, note: TacticalNote) {
    if !notes.contains(&note) {
        notes.push(note);
    }
}

fn color_for_ply(ply: usize) -> Option<Color> {
    if ply == 0 {
        None
    } else if ply % 2 == 1 {
        Some(Color::Black)
    } else {
        Some(Color::White)
    }
}
