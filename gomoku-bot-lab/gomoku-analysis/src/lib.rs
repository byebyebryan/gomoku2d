use gomoku_bot::corridor as bot_corridor;
use gomoku_bot::tactical::{
    corridor_active_threats, lethal_threat, LethalThreat, LethalThreatKind,
};
use gomoku_core::{replay::ReplayResult, Board, Color, GameResult, Move, Replay, Variant};
use serde::Serialize;

pub use gomoku_bot::corridor::{
    DefenderReplyAnalysis, DefenderReplyOutcome, DefenderReplyRole, ProofLimitCause,
    SearchDiagnostics,
};

pub const ANALYSIS_SCHEMA_VERSION: u32 = 19;
pub const DEFAULT_MAX_SCAN_PLIES: usize = 64;
const MAX_CORRIDOR_REPLY_WIDTH: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProofStatus {
    ForcedWin,
    EscapeFound,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplyPolicy {
    CorridorReplies,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RootCause {
    CorridorEntry,
    MissedDefense,
    MissedWin,
    Unclear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UnclearReason {
    PreviousPrefixUnknown,
    ScanWindowCutoff,
    ProofLimitHit,
    NoFinalForcedInterval,
    DrawOrOngoing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TacticalNote {
    ConversionError,
    MissedWin,
    StrongAttack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplyClassification {
    BlockedButForced,
    ConfirmedEscape,
    PossibleEscape,
    NoLegalBlock,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureMode {
    MissedImmediateWin,
    MissedImmediateResponse,
    MissedImminentResponse,
    MissedEscape,
    MissedLethalPrevention,
    Unclear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureConfidence {
    Confirmed,
    Possible,
    Unclear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MissedCandidateOutcome {
    ConfirmedEscape,
    PossibleEscape,
    PreventsLethalOnset,
    PreventsCorridorEntry,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MissedCandidate {
    pub mv: Move,
    pub notation: String,
    pub roles: Vec<DefenderReplyRole>,
    pub outcome: MissedCandidateOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FailureAnalysis {
    pub mode: FailureMode,
    pub side: Color,
    pub prefix_ply: Option<usize>,
    pub actual_move: Option<Move>,
    pub actual_notation: Option<String>,
    pub missed_candidates: Vec<MissedCandidate>,
    pub prevented_onset_ply: Option<usize>,
    pub confidence: FailureConfidence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DefenderReplyCandidate {
    pub mv: Move,
    pub notation: String,
    pub roles: Vec<DefenderReplyRole>,
}

#[derive(Debug, Clone)]
pub struct AnalysisOptions {
    pub reply_policy: ReplyPolicy,
    pub max_depth: usize,
    pub max_scan_plies: Option<usize>,
}

impl Default for AnalysisOptions {
    fn default() -> Self {
        Self {
            reply_policy: ReplyPolicy::CorridorReplies,
            max_depth: 4,
            max_scan_plies: Some(DEFAULT_MAX_SCAN_PLIES),
        }
    }
}

impl AnalysisOptions {
    fn corridor_options(&self) -> bot_corridor::CorridorOptions {
        bot_corridor::CorridorOptions {
            max_depth: self.max_depth,
            max_reply_width: MAX_CORRIDOR_REPLY_WIDTH,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AnalysisModel {
    pub reply_policy: ReplyPolicy,
    pub rule_set: String,
    pub max_depth: usize,
    pub max_scan_plies: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProofResult {
    pub status: ProofStatus,
    pub attacker: Color,
    pub side_to_move: Color,
    pub model: AnalysisModel,
    pub principal_line: Vec<Move>,
    pub escape_moves: Vec<Move>,
    pub threat_evidence: Vec<ThreatSequenceEvidence>,
    pub limit_hit: bool,
    pub limit_causes: Vec<ProofLimitCause>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ThreatSequenceEvidence {
    pub prefix_ply: Option<usize>,
    pub attacker: Color,
    pub defender: Color,
    pub winning_squares: Vec<Move>,
    pub raw_cost_squares: Vec<Move>,
    pub legal_cost_squares: Vec<Move>,
    pub illegal_cost_squares: Vec<Move>,
    pub defender_immediate_wins: Vec<Move>,
    pub actual_reply: Option<Move>,
    pub reply_classification: ReplyClassification,
    pub escape_replies: Vec<Move>,
    pub forced_replies: Vec<Move>,
    pub next_forcing_move: Option<Move>,
    pub proof_status: ProofStatus,
    pub limit_hit: bool,
    pub limit_causes: Vec<ProofLimitCause>,
}

#[derive(Debug, Clone, Copy)]
struct EvidenceAttribution {
    prefix_ply: Option<usize>,
    actual_reply: Option<Move>,
}

struct ThreatEvidenceInput {
    attribution: EvidenceAttribution,
    reply_classification: ReplyClassification,
    escape_replies: Vec<Move>,
    forced_replies: Vec<Move>,
    next_forcing_move: Option<Move>,
    proof_status: ProofStatus,
    limit_causes: Vec<ProofLimitCause>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ForcedInterval {
    pub start_ply: usize,
    pub end_ply: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LethalOnsetEntry {
    pub mv: Move,
    pub terminal_targets: Vec<Move>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LethalOnsetReply {
    pub reply: Move,
    pub lethal_entries: Vec<LethalOnsetEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LethalOnset {
    pub prefix_ply: usize,
    pub attacker: Color,
    pub defender: Color,
    pub kind: LethalThreatKind,
    pub terminal_targets: Vec<Move>,
    pub covering_replies: Vec<Move>,
    pub one_step_replies: Vec<LethalOnsetReply>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GameAnalysis {
    pub schema_version: u32,
    pub rule_set: String,
    pub winner: Option<Color>,
    pub loser: Option<Color>,
    pub final_move: Option<Move>,
    pub final_winning_line: Vec<Move>,
    pub model: AnalysisModel,
    pub lethal_onset: Option<LethalOnset>,
    pub setup_corridor: Option<ForcedInterval>,
    pub final_forced_interval_found: bool,
    pub final_forced_interval: ForcedInterval,
    pub proof_intervals: Vec<ForcedInterval>,
    pub unknown_gaps: Vec<usize>,
    pub unclear_reason: Option<UnclearReason>,
    pub unclear_context: Option<UnclearContext>,
    pub last_chance_ply: Option<usize>,
    pub decisive_attack_ply: Option<usize>,
    pub critical_loser_ply: Option<usize>,
    pub root_cause: RootCause,
    pub failure: Option<FailureAnalysis>,
    pub tactical_notes: Vec<TacticalNote>,
    pub principal_line: Vec<Move>,
    pub proof_summary: Vec<ProofResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplayAnalysisStepStatus {
    Running,
    Resolved,
    Unclear,
    Unsupported,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct ReplayAnalysisCounters {
    pub prefixes_analyzed: usize,
    pub branch_roots: usize,
    pub proof_nodes: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplayFrameHighlightRole {
    ImmediateWin,
    ImmediateThreat,
    ImminentThreat,
    CounterThreat,
    CorridorEntry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplayFrameMarkerRole {
    ConfirmedEscape,
    PossibleEscape,
    ForcedLoss,
    ImmediateLoss,
    Forbidden,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReplayFrameHighlight {
    pub role: ReplayFrameHighlightRole,
    pub mv: Move,
    pub notation: String,
    pub side: Color,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReplayFrameMarker {
    pub role: ReplayFrameMarkerRole,
    pub mv: Move,
    pub notation: String,
    pub side: Color,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReplayFrameAnnotations {
    pub ply: usize,
    pub side_to_move: Color,
    pub highlights: Vec<ReplayFrameHighlight>,
    pub markers: Vec<ReplayFrameMarker>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReplayAnalysisStep {
    pub status: ReplayAnalysisStepStatus,
    pub done: bool,
    pub current_ply: Option<usize>,
    pub annotations: Vec<ReplayFrameAnnotations>,
    pub analysis: Option<GameAnalysis>,
    pub counters: ReplayAnalysisCounters,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UnclearContext {
    pub reason: UnclearReason,
    pub previous_prefix_ply: Option<usize>,
    pub final_forced_interval: ForcedInterval,
    pub previous_proof_status: Option<ProofStatus>,
    pub previous_proof_limit_hit: Option<bool>,
    pub previous_limit_causes: Vec<ProofLimitCause>,
    pub previous_side_to_move: Option<Color>,
    pub winner: Color,
    pub principal_line: Vec<Move>,
    pub principal_line_notation: Vec<String>,
    pub scan_start_ply: usize,
    pub scan_end_ply: Option<usize>,
    pub move_count: usize,
    pub snapshots: Vec<AnalysisBoardSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AnalysisBoardSnapshot {
    pub label: String,
    pub ply: usize,
    pub side_to_move: Color,
    pub rows: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnalysisError {
    InvalidReplayMove { ply: usize, message: String },
}

impl std::fmt::Display for AnalysisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnalysisError::InvalidReplayMove { ply, message } => {
                write!(f, "invalid replay move at ply {ply}: {message}")
            }
        }
    }
}

impl std::error::Error for AnalysisError {}

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

fn replay_frame_annotations_from_proof(
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
        push_reply_outcome_annotations(&mut frame, winner, &replies);
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

fn replay_frame_annotations_for_analysis_with_boards(
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
            push_defender_reply_role_highlight(frame, *role, candidate.mv, winner);
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
    attacker: Color,
    replies: &[DefenderReplyAnalysis],
) {
    let defender = attacker.opponent();
    for reply in replies {
        for role in &reply.roles {
            push_defender_reply_role_highlight(frame, *role, reply.mv, attacker);
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
        push_defender_reply_role_highlight(frame, role, mv, attacker);
    }
}

fn push_defender_reply_role_highlight(
    frame: &mut ReplayFrameAnnotations,
    role: DefenderReplyRole,
    mv: Move,
    attacker: Color,
) {
    let Some((highlight_role, side)) = replay_highlight_for_defender_reply_role(role, attacker)
    else {
        return;
    };
    push_replay_highlight(&mut frame.highlights, highlight_role, mv, side);
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

struct ThreatReplySet {
    attacker: Color,
    defender: Color,
    winning_squares: Vec<Move>,
    raw_cost_squares: Vec<Move>,
    legal_cost_squares: Vec<Move>,
    illegal_cost_squares: Vec<Move>,
    defender_immediate_wins: Vec<Move>,
}

impl ThreatReplySet {
    fn new(board: &Board, attacker: Color) -> Self {
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

fn with_limit_causes(
    mut proof: ProofResult,
    causes: impl IntoIterator<Item = ProofLimitCause>,
) -> ProofResult {
    extend_limit_causes(&mut proof.limit_causes, causes);
    proof.limit_hit = !proof.limit_causes.is_empty();
    proof
}

fn corridor_proof_result(
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
enum CorridorReplyStatus {
    Forced,
    ConfirmedEscape,
    PossibleEscape,
    Unknown,
}

struct CorridorReplyOutcome {
    mv: Move,
    status: CorridorReplyStatus,
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

fn replay_corridor_status_with_actual_child(
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

fn classify_actual_corridor_reply(
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

fn corridor_defender_reply_moves(
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

fn replay_prefix_boards(replay: &Replay) -> Result<Vec<Board>, AnalysisError> {
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

fn replay_moves(replay: &Replay) -> Result<Vec<Move>, AnalysisError> {
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
fn replay_proof_summary(
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

fn proof_at(proofs: &[ProofResult], scan_start: usize, ply: usize) -> Option<&ProofResult> {
    proofs.get(ply.checked_sub(scan_start)?)
}

fn find_last_chance(
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

fn find_lethal_onset(
    boards: &[Board],
    attacker: Color,
    scan_start: usize,
    scan_end: usize,
) -> Option<LethalOnset> {
    let defender = attacker.opponent();
    let mut onset = None;
    let mut found_final_suffix = false;
    let scan_end = scan_end.min(boards.len().saturating_sub(1));

    for prefix_ply in (scan_start..=scan_end).rev() {
        let board = &boards[prefix_ply];
        if board.current_player != defender {
            continue;
        }

        if let Some(threat) = lethal_threat(board, attacker) {
            onset = Some(lethal_onset_from_threat(prefix_ply, threat));
            found_final_suffix = true;
        } else if found_final_suffix {
            break;
        }
    }

    onset
}

fn lethal_onset_from_threat(prefix_ply: usize, threat: LethalThreat) -> LethalOnset {
    LethalOnset {
        prefix_ply,
        attacker: threat.attacker,
        defender: threat.defender,
        kind: threat.kind,
        terminal_targets: threat.terminal_targets,
        covering_replies: threat.covering_replies,
        one_step_replies: threat
            .one_step_replies
            .into_iter()
            .map(|reply| LethalOnsetReply {
                reply: reply.reply,
                lethal_entries: reply
                    .lethal_entries
                    .into_iter()
                    .map(|entry| LethalOnsetEntry {
                        mv: entry.mv,
                        terminal_targets: entry.terminal_targets,
                    })
                    .collect(),
            })
            .collect(),
    }
}

struct FailureAnalysisInput<'a> {
    replay: &'a Replay,
    boards: &'a [Board],
    proof_summary: &'a [ProofResult],
    scan_start: usize,
    final_forced_interval_found: bool,
    final_forced_interval: &'a ForcedInterval,
    lethal_onset: Option<&'a LethalOnset>,
    root_cause: RootCause,
    winner: Color,
    loser: Color,
}

fn failure_analysis(input: FailureAnalysisInput<'_>) -> Option<FailureAnalysis> {
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

fn proof_has_limit_hit(proof: &ProofResult) -> bool {
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

pub fn corridor_analysis_model(board: &Board, options: &AnalysisOptions) -> AnalysisModel {
    AnalysisModel {
        reply_policy: options.reply_policy,
        rule_set: rule_label(&board.config.variant).to_string(),
        max_depth: options.max_depth,
        max_scan_plies: options.max_scan_plies,
    }
}

pub fn rule_label(variant: &Variant) -> &'static str {
    match variant {
        Variant::Freestyle => "freestyle",
        Variant::Renju => "renju",
    }
}

#[cfg(test)]
mod tests {
    use gomoku_bot::tactical::{
        has_forcing_local_threat, legal_forcing_continuations_for_fact,
        local_threat_facts_for_player as local_threat_facts, LethalThreatKind, LocalThreatFact,
        LocalThreatKind, LocalThreatOrigin,
    };
    use gomoku_core::{Board, Color, Move, Replay, RuleConfig, Variant};

    use super::{
        analyze_alternate_defender_reply_options, analyze_defender_reply_options, analyze_replay,
        corridor_analysis_model, failure_analysis, replay_frame_annotations_from_proof,
        replay_moves, replay_prefix_boards, replay_proof_summary, AnalysisOptions,
        DefenderReplyOutcome, DefenderReplyRole, FailureAnalysisInput, FailureMode, ForcedInterval,
        LethalOnset, MissedCandidateOutcome, ProofLimitCause, ProofResult, ProofStatus,
        ReplayAnalysisSession, ReplayFrameHighlightRole, ReplayFrameMarkerRole,
        ReplyClassification, ReplyPolicy, RootCause, TacticalNote, ThreatSequenceEvidence,
        UnclearReason,
    };

    fn mv(notation: &str) -> Move {
        Move::from_notation(notation).expect("test move notation should parse")
    }

    fn replay_from_moves(variant: Variant, moves: &[&str]) -> Replay {
        let rules = RuleConfig {
            variant,
            ..RuleConfig::default()
        };
        let mut board = Board::new(rules.clone());
        let mut replay = Replay::new(rules, "Black", "White");

        for notation in moves {
            let parsed = mv(notation);
            board.apply_move(parsed).expect("fixture move should apply");
            replay.push_move(parsed, 0, board.hash(), None);
        }
        replay.finish(&board.result, Some(0));
        replay
    }

    fn board_from_moves(variant: Variant, moves: &[&str]) -> Board {
        let mut board = Board::new(RuleConfig {
            variant,
            ..RuleConfig::default()
        });
        for notation in moves {
            board
                .apply_move(mv(notation))
                .expect("fixture move should apply");
        }
        board
    }

    fn proof_for_board(board: &Board, winner: Color, options: &AnalysisOptions) -> ProofResult {
        ProofResult {
            status: ProofStatus::ForcedWin,
            attacker: winner,
            side_to_move: board.current_player,
            model: corridor_analysis_model(board, options),
            principal_line: Vec::new(),
            escape_moves: Vec::new(),
            threat_evidence: Vec::new(),
            limit_hit: false,
            limit_causes: Vec::new(),
        }
    }

    fn test_analysis_options() -> AnalysisOptions {
        AnalysisOptions {
            reply_policy: ReplyPolicy::CorridorReplies,
            max_depth: 4,
            max_scan_plies: Some(64),
        }
    }

    fn proof_summary_with_escape(
        boards: &[Board],
        scan_start: usize,
        attacker: Color,
        prefix_ply: usize,
        escape_moves: Vec<Move>,
        reply_classification: ReplyClassification,
        limit_hit: bool,
    ) -> Vec<ProofResult> {
        let options = test_analysis_options();
        let mut proofs = (scan_start..boards.len())
            .map(|ply| proof_for_board(&boards[ply], attacker, &options))
            .collect::<Vec<_>>();
        proofs[prefix_ply - scan_start] = ProofResult {
            status: ProofStatus::EscapeFound,
            attacker,
            side_to_move: boards[prefix_ply].current_player,
            model: corridor_analysis_model(&boards[prefix_ply], &options),
            principal_line: Vec::new(),
            escape_moves: escape_moves.clone(),
            threat_evidence: vec![ThreatSequenceEvidence {
                prefix_ply: Some(prefix_ply),
                attacker,
                defender: attacker.opponent(),
                winning_squares: Vec::new(),
                raw_cost_squares: Vec::new(),
                legal_cost_squares: Vec::new(),
                illegal_cost_squares: Vec::new(),
                defender_immediate_wins: Vec::new(),
                actual_reply: None,
                reply_classification,
                escape_replies: escape_moves,
                forced_replies: Vec::new(),
                next_forcing_move: None,
                proof_status: ProofStatus::EscapeFound,
                limit_hit,
                limit_causes: if limit_hit {
                    vec![ProofLimitCause::DepthCutoff]
                } else {
                    Vec::new()
                },
            }],
            limit_hit,
            limit_causes: if limit_hit {
                vec![ProofLimitCause::DepthCutoff]
            } else {
                Vec::new()
            },
        };
        proofs
    }

    fn test_lethal_onset(prefix_ply: usize, attacker: Color) -> LethalOnset {
        LethalOnset {
            prefix_ply,
            attacker,
            defender: attacker.opponent(),
            kind: LethalThreatKind::OneStepCoverage,
            terminal_targets: Vec::new(),
            covering_replies: Vec::new(),
            one_step_replies: Vec::new(),
        }
    }

    #[test]
    fn defender_reply_options_distinguish_forced_loss_from_unproven_counterplay() {
        let board = board_from_moves(
            Variant::Renju,
            &[
                "H8", "I8", "H7", "I7", "H6", "H5", "I6", "I9", "G6", "J6", "G8", "J5", "G5",
            ],
        );
        let options = AnalysisOptions {
            reply_policy: ReplyPolicy::CorridorReplies,
            max_depth: 4,
            max_scan_plies: Some(8),
        };
        let replies =
            analyze_defender_reply_options(&board, Color::Black, Some(mv("G7")), &options);

        for notation in ["G4", "G7", "G9"] {
            let reply = reply_for(&replies, notation);
            assert!(reply.roles.contains(&DefenderReplyRole::ImminentDefense));
            assert_eq!(
                reply.outcome,
                DefenderReplyOutcome::ForcedLoss,
                "{notation}: line {:?} limits {:?}",
                reply.principal_line_notation,
                reply.limit_causes
            );
        }

        let i10 = reply_for(&replies, "I10");
        assert!(i10.roles.contains(&DefenderReplyRole::OffensiveCounter));
        assert_eq!(
            i10.outcome,
            DefenderReplyOutcome::PossibleEscape,
            "I10: line {:?} limits {:?}",
            i10.principal_line_notation,
            i10.limit_causes
        );
        assert!(i10.limit_causes.contains(&ProofLimitCause::DepthCutoff));

        let i11 = reply_for(&replies, "I11");
        assert!(i11.roles.contains(&DefenderReplyRole::OffensiveCounter));
        assert_eq!(
            i11.outcome,
            DefenderReplyOutcome::ForcedLoss,
            "I11: line {:?} limits {:?}",
            i11.principal_line_notation,
            i11.limit_causes
        );
    }

    #[test]
    fn imminent_open_three_defense_excludes_outer_cost_squares() {
        let board = board_from_moves(
            Variant::Renju,
            &[
                "H8", "I8", "H7", "I7", "H6", "H5", "I6", "I9", "G6", "J6", "G8",
            ],
        );
        let options = AnalysisOptions {
            reply_policy: ReplyPolicy::CorridorReplies,
            max_depth: 4,
            max_scan_plies: Some(8),
        };
        let replies =
            analyze_defender_reply_options(&board, Color::Black, Some(mv("J5")), &options);

        for notation in ["J5", "F9"] {
            let reply = reply_for(&replies, notation);
            assert!(
                reply.roles.contains(&DefenderReplyRole::ImminentDefense),
                "{notation}: roles {:?}",
                reply.roles
            );
        }
        for notation in ["E10", "K4"] {
            assert!(
                replies.iter().all(|reply| reply.notation != notation),
                "{notation} should not be a direct defense to the open three"
            );
        }
    }

    #[test]
    fn open_three_with_blocked_outer_side_includes_far_defense_square() {
        let board = board_from_moves(Variant::Renju, &["J9", "H9", "K9", "A1", "L9"]);
        assert!(
            local_threat_facts(&board, Color::Black).contains(&LocalThreatFact {
                player: Color::Black,
                kind: LocalThreatKind::OpenThree,
                origin: LocalThreatOrigin::Existing(mv("J9")),
                defense_squares: vec![mv("I9"), mv("M9"), mv("N9")],
                rest_squares: vec![],
            })
        );

        let options = AnalysisOptions {
            reply_policy: ReplyPolicy::CorridorReplies,
            max_depth: 4,
            max_scan_plies: Some(8),
        };
        let replies =
            analyze_defender_reply_options(&board, Color::Black, Some(mv("N9")), &options);
        let reply = reply_for(&replies, "N9");
        assert!(reply.roles.contains(&DefenderReplyRole::ImminentDefense));
    }

    #[test]
    fn boxed_three_is_not_a_forcing_open_three() {
        let board = board_from_moves(Variant::Renju, &["J9", "H9", "K9", "N9", "L9"]);
        assert!(
            local_threat_facts(&board, Color::Black)
                .iter()
                .all(|fact| fact.kind != LocalThreatKind::OpenThree),
            "{:?}",
            local_threat_facts(&board, Color::Black)
        );

        let options = AnalysisOptions {
            reply_policy: ReplyPolicy::CorridorReplies,
            max_depth: 4,
            max_scan_plies: Some(8),
        };
        let replies = analyze_defender_reply_options(&board, Color::Black, None, &options);
        for notation in ["I9", "M9"] {
            assert!(
                replies
                    .iter()
                    .filter(|reply| reply.notation == notation)
                    .all(|reply| !reply.roles.contains(&DefenderReplyRole::ImminentDefense)),
                "{notation}: {:?}",
                replies
            );
        }
    }

    #[test]
    fn renju_forbidden_only_black_local_threat_is_not_forcing() {
        let board = board_from_moves(
            Variant::Renju,
            &["H8", "G8", "I8", "A1", "J8", "A2", "L8", "A3", "M8"],
        );
        assert!(!board.is_legal_for_color(mv("K8"), Color::Black));
        assert!(!board
            .immediate_winning_moves_for(Color::Black)
            .contains(&mv("K8")));

        let facts = local_threat_facts(&board, Color::Black);
        let forbidden_gap_four = facts
            .iter()
            .find(|fact| {
                fact.kind == LocalThreatKind::BrokenFour && fact.defense_squares == vec![mv("K8")]
            })
            .unwrap_or_else(|| panic!("expected raw forbidden broken-four fact: {facts:?}"));
        assert!(
            legal_forcing_continuations_for_fact(&board, Color::Black, forbidden_gap_four)
                .is_empty()
        );
        assert!(
            !has_forcing_local_threat(&board, Color::Black),
            "unexpected forcing fact remains: {facts:?}"
        );
    }

    #[test]
    fn renju_mixed_black_local_threat_uses_only_legal_continuations() {
        let board = board_from_moves(
            Variant::Renju,
            &["H8", "A1", "I8", "A2", "J8", "A3", "K8", "A4", "M8"],
        );
        assert!(board.is_legal_for_color(mv("G8"), Color::Black));
        assert!(!board.is_legal_for_color(mv("L8"), Color::Black));

        let facts = local_threat_facts(&board, Color::Black);
        let mixed_open_four = facts
            .iter()
            .find(|fact| {
                fact.kind == LocalThreatKind::OpenFour
                    && fact.defense_squares == vec![mv("G8"), mv("L8")]
            })
            .unwrap_or_else(|| panic!("expected raw mixed open-four fact: {facts:?}"));
        let continuations =
            legal_forcing_continuations_for_fact(&board, Color::Black, mixed_open_four);
        assert_eq!(
            continuations
                .iter()
                .map(|continuation| continuation.mv)
                .collect::<Vec<_>>(),
            vec![mv("G8")]
        );
        assert!(has_forcing_local_threat(&board, Color::Black));
    }

    fn reply_for<'a>(
        replies: &'a [super::DefenderReplyAnalysis],
        notation: &str,
    ) -> &'a super::DefenderReplyAnalysis {
        replies
            .iter()
            .find(|reply| reply.notation == notation)
            .unwrap_or_else(|| panic!("expected reply {notation}"))
    }

    #[test]
    fn corridor_replies_finds_escape_for_single_closed_four() {
        let board = board_from_moves(
            Variant::Freestyle,
            &["H8", "G8", "I8", "A1", "J8", "A2", "K8"],
        );
        let replies = analyze_defender_reply_options(
            &board,
            Color::Black,
            None,
            &AnalysisOptions {
                reply_policy: ReplyPolicy::CorridorReplies,
                max_depth: 4,
                ..AnalysisOptions::default()
            },
        );

        assert_eq!(
            reply_for(&replies, "L8").outcome,
            DefenderReplyOutcome::ConfirmedEscape
        );
    }

    #[test]
    fn corridor_replies_proves_open_four_even_if_one_end_is_blocked() {
        let board = board_from_moves(
            Variant::Freestyle,
            &["H8", "A1", "I8", "A2", "J8", "A3", "K8"],
        );
        let replies = analyze_defender_reply_options(
            &board,
            Color::Black,
            None,
            &AnalysisOptions {
                reply_policy: ReplyPolicy::CorridorReplies,
                max_depth: 4,
                ..AnalysisOptions::default()
            },
        );

        assert_eq!(
            reply_for(&replies, "G8").outcome,
            DefenderReplyOutcome::ImmediateLoss
        );
        assert_eq!(
            reply_for(&replies, "L8").outcome,
            DefenderReplyOutcome::ImmediateLoss
        );
    }

    #[test]
    fn corridor_depth_proves_closed_four_block_into_open_four() {
        let replay = replay_from_moves(
            Variant::Freestyle,
            &[
                "H8", "G8", "I8", "A1", "J8", "C1", "K6", "E1", "K7", "G1", "K8", "L8", "K9", "K5",
                "K10",
            ],
        );

        let analysis = analyze_replay(
            &replay,
            AnalysisOptions {
                reply_policy: ReplyPolicy::CorridorReplies,
                max_depth: 4,
                max_scan_plies: Some(6),
            },
        )
        .expect("forced chain replay should analyze");

        assert_eq!(analysis.final_forced_interval.start_ply, 10);
        assert_eq!(analysis.root_cause, RootCause::MissedDefense);
        assert!(
            analysis
                .proof_summary
                .iter()
                .flat_map(|proof| proof.threat_evidence.iter())
                .any(|evidence| evidence.reply_classification
                    == ReplyClassification::BlockedButForced)
        );
    }

    #[test]
    fn corridor_depth_cutoff_is_possible_escape() {
        let board = board_from_moves(
            Variant::Freestyle,
            &[
                "H8", "G8", "I8", "A1", "J8", "C1", "K6", "E1", "K7", "G1", "K8",
            ],
        );
        let replies = analyze_defender_reply_options(
            &board,
            Color::Black,
            Some(mv("L8")),
            &AnalysisOptions {
                reply_policy: ReplyPolicy::CorridorReplies,
                max_depth: 0,
                ..AnalysisOptions::default()
            },
        );

        let reply = reply_for(&replies, "L8");
        assert_eq!(reply.outcome, DefenderReplyOutcome::PossibleEscape);
        assert!(reply.limit_causes.contains(&ProofLimitCause::DepthCutoff));
    }

    #[test]
    fn corridor_replies_allow_defender_immediate_win_escape() {
        let board = board_from_moves(
            Variant::Freestyle,
            &["A1", "H8", "A2", "I8", "A3", "J8", "A4", "K8"],
        );
        let replies = analyze_defender_reply_options(
            &board,
            Color::White,
            Some(mv("A5")),
            &AnalysisOptions {
                reply_policy: ReplyPolicy::CorridorReplies,
                max_depth: 4,
                ..AnalysisOptions::default()
            },
        );

        assert_eq!(
            reply_for(&replies, "A5").outcome,
            DefenderReplyOutcome::ConfirmedEscape
        );
    }

    #[test]
    fn corridor_replies_prove_renju_single_square_with_forbidden_block() {
        let replay = replay_from_moves(
            Variant::Renju,
            &[
                "C3", "D4", "H6", "E5", "H7", "F6", "F8", "G7", "G8", "A15", "A14", "H8",
            ],
        );

        let analysis = analyze_replay(
            &replay,
            AnalysisOptions {
                reply_policy: ReplyPolicy::CorridorReplies,
                max_depth: 4,
                max_scan_plies: Some(2),
            },
        )
        .expect("renju fixture should analyze");

        assert!(analysis
            .proof_summary
            .iter()
            .flat_map(|proof| proof.threat_evidence.iter())
            .any(|evidence| {
                evidence.reply_classification == ReplyClassification::NoLegalBlock
                    && evidence.raw_cost_squares == vec![mv("H8")]
                    && evidence.legal_cost_squares.is_empty()
                    && evidence.illegal_cost_squares == vec![mv("H8")]
            }));
    }

    #[test]
    fn replay_analysis_labels_missed_defense_without_overclaiming_previous_position() {
        let replay = replay_from_moves(
            Variant::Freestyle,
            &["H8", "G8", "I8", "A1", "J8", "A2", "K8", "B1", "L8"],
        );

        let analysis = analyze_replay(
            &replay,
            AnalysisOptions {
                reply_policy: ReplyPolicy::CorridorReplies,
                max_depth: 2,
                max_scan_plies: None,
            },
        )
        .expect("finished replay should analyze");

        assert_eq!(analysis.winner, Some(Color::Black));
        assert_eq!(analysis.final_forced_interval.start_ply, 8);
        assert_eq!(analysis.last_chance_ply, Some(7));
        assert_eq!(analysis.critical_loser_ply, Some(8));
        assert_eq!(analysis.root_cause, RootCause::MissedDefense);
        assert!(analysis.tactical_notes.is_empty());
    }

    #[test]
    fn replay_analysis_records_terminal_lethal_onset() {
        let replay = replay_from_moves(
            Variant::Freestyle,
            &["H8", "A1", "I8", "A2", "J8", "A3", "K8", "A4", "G8"],
        );

        let analysis = analyze_replay(&replay, AnalysisOptions::default())
            .expect("finished replay should analyze");

        let onset = analysis
            .lethal_onset
            .as_ref()
            .expect("open four should be a lethal onset");
        assert_eq!(onset.prefix_ply, 7);
        assert_eq!(onset.attacker, Color::Black);
        assert_eq!(onset.defender, Color::White);
        assert_eq!(onset.kind, LethalThreatKind::TerminalCoverage);
        assert_eq!(onset.terminal_targets, vec![mv("G8"), mv("L8")]);
        assert!(onset.covering_replies.is_empty());
        assert!(onset.one_step_replies.is_empty());
        assert_eq!(
            analysis
                .setup_corridor
                .as_ref()
                .map(|interval| interval.end_ply),
            Some(onset.prefix_ply)
        );
    }

    #[test]
    fn replay_analysis_records_one_step_lethal_onset_before_terminal_coverage() {
        let replay = replay_from_moves(
            Variant::Freestyle,
            &[
                "H8", "G8", "I8", "A1", "J8", "O1", "K8", "A15", "I7", "O15", "I9", "L8", "I6",
                "A14", "I10",
            ],
        );

        let analysis = analyze_replay(&replay, AnalysisOptions::default())
            .expect("finished replay should analyze");

        let onset = analysis
            .lethal_onset
            .as_ref()
            .expect("4+3 should be a one-step lethal onset");
        assert_eq!(onset.prefix_ply, 11);
        assert_eq!(onset.kind, LethalThreatKind::OneStepCoverage);
        assert_eq!(onset.terminal_targets, vec![mv("L8")]);
        assert_eq!(onset.one_step_replies.len(), 1);
        assert_eq!(onset.one_step_replies[0].reply, mv("L8"));
        assert_eq!(
            analysis
                .setup_corridor
                .as_ref()
                .map(|interval| interval.end_ply),
            Some(onset.prefix_ply)
        );
        assert_eq!(
            onset.one_step_replies[0]
                .lethal_entries
                .iter()
                .map(|entry| (entry.mv, entry.terminal_targets.clone()))
                .collect::<Vec<_>>(),
            vec![
                (mv("I6"), vec![mv("I5"), mv("I10")]),
                (mv("I10"), vec![mv("I6"), mv("I11")]),
            ]
        );
    }

    #[test]
    fn replay_analysis_stops_at_possible_escape_boundary() {
        let replay = replay_from_moves(
            Variant::Renju,
            &[
                "H8", "I8", "H7", "I7", "H6", "H5", "I6", "I9", "G6", "J6", "G8", "J5", "G5", "G7",
                "E6", "F6", "H9", "H10", "F7", "D5", "I10",
            ],
        );

        let analysis = analyze_replay(
            &replay,
            AnalysisOptions {
                reply_policy: ReplyPolicy::CorridorReplies,
                max_depth: 4,
                max_scan_plies: Some(8),
            },
        )
        .expect("finished replay should analyze");

        assert_eq!(analysis.winner, Some(Color::Black));
        assert_eq!(analysis.root_cause, RootCause::MissedDefense);
        assert_eq!(analysis.final_forced_interval.start_ply, 14);
        assert_eq!(analysis.last_chance_ply, Some(13));
        assert_eq!(analysis.critical_loser_ply, Some(14));
        let failure = analysis
            .failure
            .as_ref()
            .expect("bounded escape should produce failure detail");
        assert_eq!(failure.mode, FailureMode::MissedEscape);
        assert_eq!(failure.prefix_ply, Some(13));
        assert_eq!(failure.actual_notation.as_deref(), Some("G7"));
        assert!(failure
            .missed_candidates
            .iter()
            .any(|candidate| candidate.notation == "I10"));

        let scan_start = replay.moves.len() + 1 - analysis.proof_summary.len();
        assert_eq!(scan_start, 13);
        let boundary = analysis
            .proof_summary
            .get(13 - scan_start)
            .expect("escape boundary proof should be within the scan cap");
        assert_eq!(boundary.status, ProofStatus::EscapeFound);
        assert!(boundary.threat_evidence.iter().any(|evidence| {
            evidence.reply_classification == ReplyClassification::PossibleEscape
                && evidence.escape_replies.contains(&mv("I10"))
        }));
    }

    #[test]
    fn replay_analysis_session_matches_blocking_analysis() {
        let replay = replay_from_moves(
            Variant::Freestyle,
            &["H8", "G8", "I8", "A1", "J8", "A2", "K8", "B1", "L8"],
        );
        let options = AnalysisOptions {
            reply_policy: ReplyPolicy::CorridorReplies,
            max_depth: 2,
            max_scan_plies: Some(8),
        };
        let expected =
            analyze_replay(&replay, options.clone()).expect("blocking analysis should run");
        let mut session =
            ReplayAnalysisSession::new(replay, options).expect("session should initialize");
        let mut observed_plys = Vec::new();
        let mut final_analysis = None;

        while final_analysis.is_none() {
            let step = session.step(1);
            observed_plys.extend(step.annotations.iter().map(|frame| frame.ply));
            if step.done {
                final_analysis = step.analysis;
            }
        }

        assert_eq!(final_analysis, Some(expected));
        assert_eq!(&observed_plys[..3], [9, 8, 7]);
        assert_eq!(&observed_plys[3..], [8, 7]);
    }

    #[test]
    fn replay_analysis_session_clamps_zero_work_to_one_prefix() {
        let replay = replay_from_moves(
            Variant::Freestyle,
            &["H8", "G8", "I8", "A1", "J8", "A2", "K8", "B1", "L8"],
        );
        let mut session = ReplayAnalysisSession::new(
            replay,
            AnalysisOptions {
                reply_policy: ReplyPolicy::CorridorReplies,
                max_depth: 2,
                max_scan_plies: None,
            },
        )
        .expect("session should initialize");

        let step = session.step(0);

        assert!(!step.done);
        assert_eq!(step.counters.prefixes_analyzed, 1);
        assert_eq!(
            step.annotations
                .iter()
                .map(|frame| frame.ply)
                .collect::<Vec<_>>(),
            vec![9]
        );
    }

    #[test]
    fn replay_frame_annotations_emit_escape_boundary_candidates() {
        let board = board_from_moves(
            Variant::Renju,
            &[
                "H8", "I8", "H7", "I7", "H6", "H5", "I6", "I9", "G6", "J6", "G8", "J5", "G5",
            ],
        );
        let options = AnalysisOptions {
            reply_policy: ReplyPolicy::CorridorReplies,
            max_depth: 4,
            max_scan_plies: Some(8),
        };
        let proof = proof_for_board(&board, Color::Black, &options);
        let boundary = replay_frame_annotations_from_proof(
            13,
            &board,
            Color::Black,
            &proof,
            None,
            Some(mv("G7")),
            &options,
        );

        assert_eq!(boundary.side_to_move, Color::White);
        assert!(boundary.markers.iter().any(|marker| {
            marker.role == ReplayFrameMarkerRole::PossibleEscape
                && marker.mv == mv("I10")
                && marker.side == Color::White
        }));

        for notation in ["G4", "G7", "G9"] {
            let mv = mv(notation);
            assert!(
                boundary.highlights.iter().any(|highlight| {
                    highlight.role == ReplayFrameHighlightRole::ImminentThreat
                        && highlight.mv == mv
                        && highlight.side == Color::Black
                }),
                "{notation} should be highlighted as a current imminent-threat response: {:?}",
                boundary.highlights
            );
        }

        for notation in ["I10", "I11"] {
            let mv = mv(notation);
            assert!(
                boundary.highlights.iter().any(|highlight| {
                    highlight.role == ReplayFrameHighlightRole::CounterThreat
                        && highlight.mv == mv
                        && highlight.side == Color::White
                }),
                "{notation} should be highlighted as a current counter-threat response: {:?}",
                boundary.highlights
            );
        }
    }

    #[test]
    fn replay_analysis_session_marks_actual_counter_threat_hint() {
        let replay = replay_from_moves(
            Variant::Renju,
            &[
                "H8", "G7", "H6", "H7", "I7", "F7", "G5", "F4", "J8", "K9", "I8", "G8", "I6", "I9",
                "H9", "F6", "F5", "G9", "G10", "E7", "D7", "J7", "I5", "I4", "H5", "E5", "J5",
            ],
        );
        let mut session = ReplayAnalysisSession::new(
            replay,
            AnalysisOptions {
                reply_policy: ReplyPolicy::CorridorReplies,
                max_depth: 4,
                max_scan_plies: Some(64),
            },
        )
        .expect("session should initialize");
        let mut annotations = Vec::new();
        loop {
            let step = session.step(2);
            annotations.extend(step.annotations);
            if step.done {
                break;
            }
        }

        let frame = annotations
            .iter()
            .rev()
            .find(|frame| frame.ply == 19)
            .expect("move 19 frame should be annotated");

        assert_eq!(frame.side_to_move, Color::White);
        assert!(
            frame.highlights.iter().any(|highlight| {
                highlight.role == ReplayFrameHighlightRole::CounterThreat
                    && highlight.mv == mv("E7")
                    && highlight.side == Color::White
            }),
            "White's actual E7 reply should be highlighted as a counter threat: {:?}",
            frame.highlights
        );
    }

    #[test]
    fn replay_analysis_probes_all_imminent_combo_reply_outcomes() {
        let board = board_from_moves(
            Variant::Renju,
            &[
                "H8", "H7", "F8", "G9", "G8", "I8", "G6", "D9", "F9", "F10", "D7", "G10", "F7",
                "E10", "E8", "D8", "C6", "B5", "D10", "F11", "F6", "F5", "D6", "E6", "H5", "I4",
            ],
        );
        assert_eq!(board.current_player, Color::Black);

        let replies = analyze_alternate_defender_reply_options(
            &board,
            Color::White,
            Some(mv("C8")),
            &AnalysisOptions {
                reply_policy: ReplyPolicy::CorridorReplies,
                max_depth: 0,
                max_scan_plies: Some(64),
            },
        );

        for notation in ["J7", "H9", "E12", "G12"] {
            let reply = reply_for(&replies, notation);
            assert!(
                reply.roles.contains(&DefenderReplyRole::ImminentDefense),
                "{notation} should be probed as a response to the 3+3 corridor: {replies:?}"
            );
        }
        assert!(
            replies.iter().all(|reply| reply.notation != "C8"),
            "the actual replay move is inherited from replay context, not re-probed: {replies:?}"
        );
    }

    #[test]
    fn replay_analysis_does_not_treat_combo_imminent_actual_reply_as_miss() {
        let replay = replay_from_moves(
            Variant::Renju,
            &[
                "H8", "I9", "G8", "H6", "F8", "I8", "I7", "G9", "H9", "E6", "I10", "J11", "H10",
                "H11", "G10", "F10", "E8", "D8", "F11", "E9", "G11", "F6", "G6", "E7", "G5", "D6",
                "E12", "D13", "G4", "G7", "G3", "G2", "B6", "C9", "B10", "D9", "F9", "D7", "D5",
                "D10",
            ],
        );

        let analysis = analyze_replay(
            &replay,
            AnalysisOptions {
                reply_policy: ReplyPolicy::CorridorReplies,
                max_depth: 4,
                max_scan_plies: Some(64),
            },
        )
        .expect("finished replay should analyze");

        let failure = analysis.failure.as_ref().expect("failure should classify");
        assert_eq!(failure.mode, FailureMode::MissedEscape);
        assert_eq!(failure.prefix_ply, Some(36));
        assert!(failure.actual_notation.is_none());
    }

    #[test]
    fn failure_analysis_classifies_late_imminent_miss_before_lethal_prevention() {
        let replay = replay_from_moves(
            Variant::Renju,
            &[
                "H8", "H9", "J8", "I7", "I8", "G8", "I10", "L7", "K7", "I9", "J9", "G9", "J7",
                "J6", "K8", "L8", "K9", "K10", "J10", "J11", "G12", "H11", "L9", "F9", "E9", "I6",
                "N9", "M9", "H7", "G6", "K6", "K5", "G10", "I11", "H10", "F10", "G11", "F7", "F6",
                "L5", "N11", "M10", "G14", "G13", "N10", "L4", "M3", "L6",
            ],
        );

        let boards = replay_prefix_boards(&replay).expect("replay boards should build");
        let scan_start = 40;
        let proof_summary = proof_summary_with_escape(
            &boards,
            scan_start,
            Color::White,
            44,
            vec![mv("M3"), mv("L4"), mv("L6")],
            ReplyClassification::PossibleEscape,
            true,
        );
        let forced_interval = ForcedInterval {
            start_ply: 45,
            end_ply: replay.moves.len(),
        };
        let onset = test_lethal_onset(46, Color::White);

        let failure = failure_analysis(FailureAnalysisInput {
            replay: &replay,
            boards: &boards,
            proof_summary: &proof_summary,
            scan_start,
            final_forced_interval_found: true,
            final_forced_interval: &forced_interval,
            lethal_onset: Some(&onset),
            root_cause: RootCause::MissedDefense,
            winner: Color::White,
            loser: Color::Black,
        })
        .expect("failure should classify");
        assert_eq!(failure.mode, FailureMode::MissedImminentResponse);
        assert_eq!(failure.prefix_ply, Some(44));
        assert_eq!(failure.actual_notation.as_deref(), Some("N10"));
        for notation in ["M3", "L4", "L6"] {
            assert!(
                failure
                    .missed_candidates
                    .iter()
                    .any(|candidate| candidate.notation == notation),
                "{notation} should be listed as a missed imminent response: {failure:?}"
            );
        }
    }

    #[test]
    fn failure_analysis_classifies_early_prevention_as_missed_escape() {
        let replay = replay_from_moves(
            Variant::Renju,
            &[
                "H8", "H7", "F8", "G9", "G8", "I8", "G6", "D9", "F9", "F10", "D7", "G10", "F7",
                "E10", "E8", "D8", "C6", "B5", "F6", "F5", "E6", "D6", "D5", "C4", "H9",
            ],
        );

        let boards = replay_prefix_boards(&replay).expect("replay boards should build");
        let scan_start = 13;
        let proof_summary = proof_summary_with_escape(
            &boards,
            scan_start,
            Color::Black,
            13,
            vec![mv("E8")],
            ReplyClassification::PossibleEscape,
            true,
        );
        let forced_interval = ForcedInterval {
            start_ply: 14,
            end_ply: replay.moves.len(),
        };
        let onset = test_lethal_onset(21, Color::Black);

        let failure = failure_analysis(FailureAnalysisInput {
            replay: &replay,
            boards: &boards,
            proof_summary: &proof_summary,
            scan_start,
            final_forced_interval_found: true,
            final_forced_interval: &forced_interval,
            lethal_onset: Some(&onset),
            root_cause: RootCause::MissedDefense,
            winner: Color::Black,
            loser: Color::White,
        })
        .expect("failure should classify");
        assert_eq!(failure.mode, FailureMode::MissedEscape);
        assert_eq!(failure.prefix_ply, Some(13));
        assert_eq!(failure.actual_notation.as_deref(), Some("E10"));
        assert!(failure.missed_candidates.iter().any(|candidate| {
            candidate.notation == "E8"
                && candidate.outcome == MissedCandidateOutcome::PreventsLethalOnset
        }));
    }

    #[test]
    fn replay_annotations_do_not_show_lower_tier_forbidden_replies_during_immediate_threats() {
        let board = board_from_moves(
            Variant::Renju,
            &[
                "H8", "I8", "H10", "G9", "H9", "H7", "J9", "G12", "G10", "I10", "H11", "H12",
                "I12", "F9", "I6", "E10", "J11", "C12", "D11", "D9", "H13", "E9", "C9", "F11",
                "C8", "K10", "C7", "C10", "J10", "J8", "I9", "K11", "K9", "L9", "L8", "M7", "F6",
                "G7", "H6", "G6", "F7", "E12", "C5", "C6", "J12", "J13", "F15", "G14", "E8", "F12",
                "D12", "F10",
            ],
        );
        assert_eq!(board.current_player, Color::Black);
        let options = AnalysisOptions {
            reply_policy: ReplyPolicy::CorridorReplies,
            max_depth: 0,
            max_scan_plies: Some(64),
        };
        let proof = proof_for_board(&board, Color::White, &options);

        let frame = replay_frame_annotations_from_proof(
            52,
            &board,
            Color::White,
            &proof,
            None,
            Some(mv("F8")),
            &options,
        );

        assert!(
            frame
                .highlights
                .iter()
                .all(|highlight| highlight.mv != mv("D8")),
            "lower-tier forbidden imminent replies should not be highlighted while immediate threats are active: {:?}",
            frame.highlights
        );
        assert!(
            frame.markers.iter().all(|marker| marker.mv != mv("D8")),
            "lower-tier forbidden imminent replies should not be marked while immediate threats are active: {:?}",
            frame.markers
        );
        assert!(frame.highlights.iter().any(|highlight| {
            highlight.role == ReplayFrameHighlightRole::ImmediateThreat && highlight.mv == mv("F13")
        }));
    }

    #[test]
    fn replay_analysis_session_marks_next_corridor_entry_on_escape_boundary() {
        let replay = replay_from_moves(
            Variant::Renju,
            &[
                "H8", "H9", "J8", "I7", "I8", "G8", "I10", "L7", "K7", "I9", "J9", "H7", "J7",
                "J10", "H11", "F13", "I6", "F9", "J6", "J5", "L8", "K8", "I5", "H4", "M9",
            ],
        );
        let mut session = ReplayAnalysisSession::new(
            replay,
            AnalysisOptions {
                reply_policy: ReplyPolicy::CorridorReplies,
                max_depth: 4,
                max_scan_plies: Some(64),
            },
        )
        .expect("session should initialize");
        let mut annotations = Vec::new();
        loop {
            let step = session.step(2);
            annotations.extend(step.annotations);
            if step.done {
                break;
            }
        }

        let frame = annotations
            .iter()
            .rev()
            .find(|frame| frame.ply == 17)
            .expect("escape boundary frame should be annotated");

        assert_eq!(frame.side_to_move, Color::White);
        assert!(frame.highlights.iter().any(|highlight| {
            highlight.role == ReplayFrameHighlightRole::CorridorEntry
                && highlight.mv == mv("J6")
                && highlight.side == Color::Black
        }));
        assert!(frame.markers.iter().any(|marker| {
            marker.role == ReplayFrameMarkerRole::ConfirmedEscape
                && marker.mv == mv("J6")
                && marker.side == Color::White
        }));
        assert!(
            frame
                .highlights
                .iter()
                .all(|highlight| highlight.mv != mv("F9")),
            "the escape boundary should point at the next corridor-entry move, not the current actual reply: {:?}",
            frame.highlights
        );
    }

    #[test]
    fn replay_analysis_session_emits_corridor_entry_escape_boundary() {
        let replay = replay_from_moves(
            Variant::Renju,
            &[
                "H8", "H9", "I8", "I9", "G8", "J8", "J9", "K7", "H10", "H7", "G9", "L6", "M5",
                "I7", "G7", "G6", "F8", "E8", "E7", "D6", "I11",
            ],
        );
        let mut session = ReplayAnalysisSession::new(
            replay,
            AnalysisOptions {
                reply_policy: ReplyPolicy::CorridorReplies,
                max_depth: 4,
                max_scan_plies: Some(64),
            },
        )
        .expect("session should initialize");
        let mut annotations = Vec::new();

        loop {
            let step = session.step(2);
            annotations.extend(step.annotations);
            if step.done {
                break;
            }
        }

        let boundary = annotations
            .iter()
            .rev()
            .find(|frame| frame.ply == 13)
            .expect("attacker-started corridor boundary should be annotated");
        assert!(boundary.highlights.iter().any(|highlight| {
            highlight.role == ReplayFrameHighlightRole::CorridorEntry
                && highlight.mv == mv("G7")
                && highlight.side == Color::Black
        }));
        assert!(boundary.markers.iter().any(|marker| {
            marker.role == ReplayFrameMarkerRole::ConfirmedEscape
                && marker.mv == mv("G7")
                && marker.side == Color::White
        }));
    }

    #[test]
    fn replay_analysis_does_not_label_actual_immediate_block_as_escape() {
        let replay = replay_from_moves(
            Variant::Renju,
            &[
                "H8", "H7", "F8", "G9", "G8", "I8", "J9", "I7", "F7", "I10", "I9", "F6", "K9",
                "H9", "J7", "H10", "L9", "M9", "I6", "K8", "E8", "D8", "G6", "H5", "J6", "J10",
                "K10", "G5", "H4", "F11", "G10", "H6", "J8", "J5", "I5", "K7", "J11", "I12", "F9",
                "H11", "D7", "C6", "E7", "C7", "E9", "E6", "D9", "C10", "C9", "B9", "D10", "C11",
                "E11", "E10", "F12", "B8", "G13",
            ],
        );
        let prefix_ply = 49;
        let boards = replay_prefix_boards(&replay).expect("fixture replay should apply");
        let actual_moves = replay_moves(&replay).expect("fixture replay should parse");
        let proof_summary = replay_proof_summary(
            &boards,
            &actual_moves,
            Color::Black,
            &AnalysisOptions {
                reply_policy: ReplyPolicy::CorridorReplies,
                max_depth: 4,
                max_scan_plies: Some(8),
            },
            prefix_ply,
        );
        let proof = proof_summary
            .first()
            .expect("target prefix should be in proof summary");

        assert_eq!(proof.status, ProofStatus::ForcedWin);
        assert!(proof
            .threat_evidence
            .iter()
            .all(|evidence| evidence.actual_reply != Some(mv("B9"))
                || evidence.reply_classification != ReplyClassification::ConfirmedEscape));
        assert!(proof
            .threat_evidence
            .iter()
            .flat_map(|evidence| evidence.escape_replies.iter())
            .all(|&reply| reply != mv("B9")));
    }

    #[test]
    fn replay_analysis_extends_forced_corridor_through_forbidden_renju_defenses() {
        let replay = replay_from_moves(
            Variant::Renju,
            &[
                "H8", "G7", "H9", "J8", "H7", "H6", "I7", "F8", "E9", "H10", "I5", "G9", "E7",
                "I9", "K7", "G10", "G11", "I11", "J12", "G6", "G8", "F6", "L7", "J7", "J6", "E6",
                "D6", "I6",
            ],
        );

        let analysis = analyze_replay(
            &replay,
            AnalysisOptions {
                reply_policy: ReplyPolicy::CorridorReplies,
                max_depth: 4,
                max_scan_plies: Some(64),
            },
        )
        .expect("finished replay should analyze");

        assert_eq!(analysis.root_cause, RootCause::MissedDefense);
        assert_eq!(analysis.final_forced_interval.start_ply, 23);
        assert_eq!(analysis.critical_loser_ply, Some(23));
        assert_eq!(analysis.unclear_reason, None);
        assert!(analysis.unknown_gaps.is_empty());
    }

    #[test]
    fn corridor_reply_moves_exclude_non_corridor_actual_reply() {
        let board = board_from_moves(Variant::Freestyle, &["H8", "A1", "I8"]);
        let actual_moves = vec![mv("H8"), mv("A1"), mv("I8"), mv("A2"), mv("J8")];
        let threat = super::ThreatReplySet::new(&board, Color::Black);

        let replies = super::corridor_defender_reply_moves(
            &board,
            &actual_moves,
            3,
            &AnalysisOptions::default(),
            &threat,
        );

        assert!(
            !replies.contains(&mv("A2")),
            "non-corridor actual defender move must not become a probed reply: {replies:?}"
        );
        assert!(
            replies.contains(&mv("J8")),
            "the winner's next corridor-entry square should remain visible as the escape target"
        );
    }

    #[test]
    fn actual_corridor_reply_limit_hit_is_possible_escape() {
        let board = board_from_moves(Variant::Freestyle, &["H8"]);
        let options = AnalysisOptions::default();
        let unknown_child = super::with_limit_causes(
            super::corridor_proof_result(
                &board,
                Color::Black,
                &options,
                ProofStatus::Unknown,
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ),
            [ProofLimitCause::DepthCutoff],
        );

        let outcome = super::classify_actual_corridor_reply(
            &board,
            &[],
            Color::Black,
            &options,
            1,
            mv("A1"),
            Some(&unknown_child),
        );

        assert_eq!(outcome.status, super::CorridorReplyStatus::PossibleEscape);
    }

    #[test]
    fn replay_analysis_attaches_actual_reply_to_actual_line_evidence() {
        let replay = replay_from_moves(
            Variant::Freestyle,
            &["H8", "G8", "I8", "A1", "J8", "A2", "K8", "B1", "L8"],
        );

        let analysis = analyze_replay(
            &replay,
            AnalysisOptions {
                reply_policy: ReplyPolicy::CorridorReplies,
                max_depth: 2,
                max_scan_plies: None,
            },
        )
        .expect("finished replay should analyze");

        let proof = analysis
            .proof_summary
            .get(7)
            .expect("ply 7 proof should be scanned");
        let evidence = proof
            .threat_evidence
            .first()
            .expect("ply 7 immediate threat should be explained");
        assert_eq!(evidence.prefix_ply, Some(7));
        assert_eq!(evidence.actual_reply, Some(mv("B1")));
    }

    #[test]
    fn replay_analysis_tracks_conversion_error_before_final_interval() {
        let replay = replay_from_moves(
            Variant::Freestyle,
            &[
                "H8", "G8", "I8", "A1", "J8", "A2", "K8", "B1", "C1", "B2", "L8",
            ],
        );

        let analysis = analyze_replay(
            &replay,
            AnalysisOptions {
                reply_policy: ReplyPolicy::CorridorReplies,
                max_depth: 2,
                max_scan_plies: None,
            },
        )
        .expect("finished replay should analyze");

        assert_eq!(analysis.final_forced_interval.start_ply, 10);
        assert_eq!(analysis.root_cause, RootCause::MissedDefense);
        assert!(analysis
            .tactical_notes
            .contains(&TacticalNote::ConversionError));
        assert!(analysis.tactical_notes.contains(&TacticalNote::MissedWin));
    }

    #[test]
    fn replay_analysis_labels_losing_side_missed_win_as_root_cause() {
        let replay = replay_from_moves(
            Variant::Freestyle,
            &[
                "A1", "H8", "A2", "I8", "A3", "J8", "B1", "K8", "A4", "C1", "A5",
            ],
        );

        let analysis = analyze_replay(
            &replay,
            AnalysisOptions {
                reply_policy: ReplyPolicy::CorridorReplies,
                max_depth: 2,
                ..AnalysisOptions::default()
            },
        )
        .expect("finished replay should analyze");

        assert_eq!(analysis.winner, Some(Color::Black));
        assert_eq!(analysis.root_cause, RootCause::MissedWin);
        assert_eq!(analysis.critical_loser_ply, Some(10));
        assert!(analysis.tactical_notes.contains(&TacticalNote::MissedWin));
    }

    #[test]
    fn shallow_corridor_analysis_finds_open_four_point_of_no_return() {
        let replay = replay_from_moves(
            Variant::Freestyle,
            &["H8", "A1", "I8", "A2", "J8", "A3", "K8", "B1", "L8"],
        );

        let analysis = analyze_replay(
            &replay,
            AnalysisOptions {
                reply_policy: ReplyPolicy::CorridorReplies,
                max_depth: 1,
                max_scan_plies: Some(4),
            },
        )
        .expect("finished replay should analyze");

        assert_eq!(analysis.root_cause, RootCause::MissedDefense);
        assert!(analysis.final_forced_interval_found);
        assert_eq!(analysis.final_forced_interval.start_ply, 6);
        assert_eq!(analysis.last_chance_ply, Some(5));
    }

    #[test]
    fn ongoing_replay_has_no_winner_and_unknown_root_cause() {
        let replay = replay_from_moves(Variant::Freestyle, &["H8", "A1", "I8"]);

        let analysis = analyze_replay(&replay, AnalysisOptions::default())
            .expect("ongoing replay should still produce a bounded summary");

        assert_eq!(analysis.winner, None);
        assert_eq!(analysis.root_cause, RootCause::Unclear);
        assert_eq!(analysis.unclear_reason, Some(UnclearReason::DrawOrOngoing));
        assert!(!analysis.final_forced_interval_found);
        assert_eq!(analysis.final_forced_interval.start_ply, 0);
        assert_eq!(analysis.final_forced_interval.end_ply, 0);
    }

    #[test]
    fn renju_forbidden_defense_remains_model_visible() {
        let replay = replay_from_moves(Variant::Renju, &["H8", "A1"]);

        let analysis = analyze_replay(
            &replay,
            AnalysisOptions {
                reply_policy: ReplyPolicy::CorridorReplies,
                max_depth: 4,
                ..AnalysisOptions::default()
            },
        )
        .expect("ongoing renju replay should analyze");

        assert_eq!(analysis.model.reply_policy, ReplyPolicy::CorridorReplies);
        assert_eq!(analysis.root_cause, RootCause::Unclear);
    }
}
