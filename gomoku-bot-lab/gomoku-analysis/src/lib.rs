use gomoku_bot::corridor as bot_corridor;
use gomoku_bot::tactical::{corridor_active_threats, LocalThreatKind};
use gomoku_core::{replay::ReplayResult, Board, Color, GameResult, Move, Replay, Variant};
use serde::Serialize;

pub const ANALYSIS_SCHEMA_VERSION: u32 = 14;
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
    StrategicLoss,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProofLimitCause {
    DepthCutoff,
    ReplyWidthCutoff,
    AttackerChildUnknown,
    DefenderReplyUnknown,
    ModelScopeUnknown,
    OutsideScanWindow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TacticalNote {
    AccidentalBlunder,
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
pub enum DefenderReplyRole {
    Actual,
    ImmediateDefense,
    ImminentDefense,
    OffensiveCounter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DefenderReplyOutcome {
    ForcedLoss,
    ConfirmedEscape,
    PossibleEscape,
    ImmediateLoss,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DefenderReplyAnalysis {
    pub mv: Move,
    pub notation: String,
    pub roles: Vec<DefenderReplyRole>,
    pub outcome: DefenderReplyOutcome,
    pub principal_line: Vec<Move>,
    pub principal_line_notation: Vec<String>,
    pub limit_causes: Vec<ProofLimitCause>,
    pub diagnostics: SearchDiagnostics,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct SearchDiagnostics {
    pub search_nodes: usize,
    pub branch_probes: usize,
    pub max_depth_reached: usize,
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
pub struct GameAnalysis {
    pub schema_version: u32,
    pub rule_set: String,
    pub winner: Option<Color>,
    pub loser: Option<Color>,
    pub final_move: Option<Move>,
    pub final_winning_line: Vec<Move>,
    pub model: AnalysisModel,
    pub final_forced_interval_found: bool,
    pub final_forced_interval: ForcedInterval,
    pub proof_intervals: Vec<ForcedInterval>,
    pub unknown_gaps: Vec<usize>,
    pub unclear_reason: Option<UnclearReason>,
    pub unclear_context: Option<UnclearContext>,
    pub last_chance_ply: Option<usize>,
    pub decisive_attack_ply: Option<usize>,
    pub critical_mistake_ply: Option<usize>,
    pub root_cause: RootCause,
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
    actual_child: Option<&ProofResult>,
    actual_reply: Option<Move>,
    options: &AnalysisOptions,
) -> ReplayFrameAnnotations {
    let mut frame = ReplayFrameAnnotations {
        ply,
        side_to_move: proof.side_to_move,
        highlights: Vec::new(),
        markers: Vec::new(),
    };

    push_current_loser_tactical_annotations(&mut frame, board, winner);
    push_forbidden_cost_annotations(&mut frame, board, proof, ply, None);

    if board.current_player == winner.opponent() {
        if let Some(actual_child) = actual_child {
            push_forbidden_cost_annotations(
                &mut frame,
                board,
                actual_child,
                ply + 1,
                Some(ReplayFrameHighlightRole::ImminentThreat),
            );
        }
        let replies =
            analyze_alternate_defender_reply_options(board, winner, actual_reply, options);
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

fn push_current_loser_tactical_annotations(
    frame: &mut ReplayFrameAnnotations,
    board: &Board,
    winner: Color,
) {
    let defender = winner.opponent();
    if board.current_player != defender {
        return;
    }

    let defender_wins = board.immediate_winning_moves_for(defender);
    let attacker_wins = board.immediate_winning_moves_for(winner);

    for mv in defender_wins.iter().copied() {
        push_replay_highlight(
            &mut frame.highlights,
            ReplayFrameHighlightRole::ImmediateWin,
            mv,
            defender,
        );
    }
    for mv in attacker_wins.iter().copied() {
        push_replay_highlight(
            &mut frame.highlights,
            ReplayFrameHighlightRole::ImmediateThreat,
            mv,
            winner,
        );
        if !board.is_legal_for_color(mv, defender) {
            push_replay_marker(
                &mut frame.markers,
                ReplayFrameMarkerRole::Forbidden,
                mv,
                defender,
            );
        }
    }

    if !defender_wins.is_empty() || !attacker_wins.is_empty() {
        return;
    }

    for fact in corridor_active_threats(board, winner)
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
            push_replay_highlight(
                &mut frame.highlights,
                ReplayFrameHighlightRole::ImminentThreat,
                mv,
                winner,
            );
            if !board.is_legal_for_color(mv, defender) {
                push_replay_marker(
                    &mut frame.markers,
                    ReplayFrameMarkerRole::Forbidden,
                    mv,
                    defender,
                );
            }
        }
    }
}

fn push_forbidden_cost_annotations(
    frame: &mut ReplayFrameAnnotations,
    board: &Board,
    proof: &ProofResult,
    prefix_ply: usize,
    tactical_role: Option<ReplayFrameHighlightRole>,
) {
    for evidence in proof
        .threat_evidence
        .iter()
        .filter(|evidence| evidence.prefix_ply == Some(prefix_ply))
    {
        for mv in evidence.illegal_cost_squares.iter().copied().filter(|mv| {
            board.is_empty(mv.row, mv.col) && !board.is_legal_for_color(*mv, evidence.defender)
        }) {
            if let Some(tactical_role) = tactical_role {
                push_replay_highlight(&mut frame.highlights, tactical_role, mv, evidence.attacker);
            }
            push_replay_marker(
                &mut frame.markers,
                ReplayFrameMarkerRole::Forbidden,
                mv,
                evidence.defender,
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
            match role {
                DefenderReplyRole::Actual => {}
                DefenderReplyRole::ImmediateDefense => push_replay_highlight(
                    &mut frame.highlights,
                    ReplayFrameHighlightRole::ImmediateThreat,
                    reply.mv,
                    attacker,
                ),
                DefenderReplyRole::ImminentDefense => push_replay_highlight(
                    &mut frame.highlights,
                    ReplayFrameHighlightRole::ImminentThreat,
                    reply.mv,
                    attacker,
                ),
                DefenderReplyRole::OffensiveCounter => push_replay_highlight(
                    &mut frame.highlights,
                    ReplayFrameHighlightRole::CounterThreat,
                    reply.mv,
                    defender,
                ),
            }
        }

        let marker_role = match reply.outcome {
            DefenderReplyOutcome::ForcedLoss => ReplayFrameMarkerRole::ForcedLoss,
            DefenderReplyOutcome::ConfirmedEscape => ReplayFrameMarkerRole::ConfirmedEscape,
            DefenderReplyOutcome::PossibleEscape => ReplayFrameMarkerRole::PossibleEscape,
            DefenderReplyOutcome::ImmediateLoss => ReplayFrameMarkerRole::ImmediateLoss,
            DefenderReplyOutcome::Unknown => ReplayFrameMarkerRole::Unknown,
        };
        push_replay_marker(&mut frame.markers, marker_role, reply.mv, defender);
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
        match role {
            DefenderReplyRole::Actual => {}
            DefenderReplyRole::ImmediateDefense => push_replay_highlight(
                &mut frame.highlights,
                ReplayFrameHighlightRole::ImmediateThreat,
                mv,
                attacker,
            ),
            DefenderReplyRole::ImminentDefense => push_replay_highlight(
                &mut frame.highlights,
                ReplayFrameHighlightRole::ImminentThreat,
                mv,
                attacker,
            ),
            DefenderReplyRole::OffensiveCounter => push_replay_highlight(
                &mut frame.highlights,
                ReplayFrameHighlightRole::CounterThreat,
                mv,
                defender,
            ),
        }
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
        critical_mistake_ply: None,
        root_cause: RootCause::Unclear,
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
        losing_side_missed_immediate_win(replay, &boards, final_forced_interval.start_ply, loser)
    });
    let root_cause = classify_root_cause(previous_status, move_color, winner, missed_win_root);
    let last_chance_ply = find_last_chance(
        &boards,
        &proof_summary,
        scan_start,
        final_forced_interval.start_ply,
        loser,
    );
    let critical_mistake_ply = match root_cause {
        RootCause::MissedDefense | RootCause::MissedWin => Some(final_forced_interval.start_ply),
        _ => None,
    };
    let decisive_attack_ply =
        (move_color == Some(winner)).then_some(final_forced_interval.start_ply);
    let tactical_notes = tactical_notes(TacticalNoteInput {
        replay,
        boards: &boards,
        proofs: &proof_summary,
        scan_start,
        proof_intervals: &proof_intervals,
        final_forced_interval: &final_forced_interval,
        winner,
        root_cause,
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
        boards: &boards,
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
        final_forced_interval_found,
        final_forced_interval,
        proof_intervals,
        unknown_gaps,
        unclear_reason,
        unclear_context,
        last_chance_ply,
        decisive_attack_ply,
        critical_mistake_ply,
        root_cause,
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
    .into_iter()
    .map(map_bot_defender_reply_analysis)
    .collect()
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
    .into_iter()
    .map(map_bot_defender_reply_analysis)
    .collect()
}

pub fn defender_reply_roles_for_move(
    board: &Board,
    attacker: Color,
    mv: Move,
) -> Vec<DefenderReplyRole> {
    bot_corridor::defender_reply_roles_for_move(board, attacker, mv)
        .into_iter()
        .map(map_bot_defender_reply_role)
        .collect()
}

fn map_bot_defender_reply_analysis(
    reply: bot_corridor::DefenderReplyAnalysis,
) -> DefenderReplyAnalysis {
    DefenderReplyAnalysis {
        mv: reply.mv,
        notation: reply.notation,
        roles: reply
            .roles
            .into_iter()
            .map(map_bot_defender_reply_role)
            .collect(),
        outcome: map_bot_defender_reply_outcome(reply.outcome),
        principal_line: reply.principal_line,
        principal_line_notation: reply.principal_line_notation,
        limit_causes: reply
            .limit_causes
            .into_iter()
            .map(map_bot_proof_limit_cause)
            .collect(),
        diagnostics: map_bot_search_diagnostics(reply.diagnostics),
    }
}

fn map_bot_defender_reply_proof(proof: bot_corridor::DefenderReplyProof) -> DefenderReplyProof {
    DefenderReplyProof {
        outcome: map_bot_defender_reply_outcome(proof.outcome),
        principal_line: proof.principal_line,
        limit_causes: proof
            .limit_causes
            .into_iter()
            .map(map_bot_proof_limit_cause)
            .collect(),
    }
}

fn map_bot_defender_reply_role(role: bot_corridor::DefenderReplyRole) -> DefenderReplyRole {
    match role {
        bot_corridor::DefenderReplyRole::Actual => DefenderReplyRole::Actual,
        bot_corridor::DefenderReplyRole::ImmediateDefense => DefenderReplyRole::ImmediateDefense,
        bot_corridor::DefenderReplyRole::ImminentDefense => DefenderReplyRole::ImminentDefense,
        bot_corridor::DefenderReplyRole::OffensiveCounter => DefenderReplyRole::OffensiveCounter,
    }
}

fn map_bot_defender_reply_outcome(
    outcome: bot_corridor::DefenderReplyOutcome,
) -> DefenderReplyOutcome {
    match outcome {
        bot_corridor::DefenderReplyOutcome::ForcedLoss => DefenderReplyOutcome::ForcedLoss,
        bot_corridor::DefenderReplyOutcome::ConfirmedEscape => {
            DefenderReplyOutcome::ConfirmedEscape
        }
        bot_corridor::DefenderReplyOutcome::PossibleEscape => DefenderReplyOutcome::PossibleEscape,
        bot_corridor::DefenderReplyOutcome::ImmediateLoss => DefenderReplyOutcome::ImmediateLoss,
        bot_corridor::DefenderReplyOutcome::Unknown => DefenderReplyOutcome::Unknown,
    }
}

fn map_bot_proof_limit_cause(cause: bot_corridor::ProofLimitCause) -> ProofLimitCause {
    match cause {
        bot_corridor::ProofLimitCause::DepthCutoff => ProofLimitCause::DepthCutoff,
        bot_corridor::ProofLimitCause::ReplyWidthCutoff => ProofLimitCause::ReplyWidthCutoff,
        bot_corridor::ProofLimitCause::AttackerChildUnknown => {
            ProofLimitCause::AttackerChildUnknown
        }
        bot_corridor::ProofLimitCause::DefenderReplyUnknown => {
            ProofLimitCause::DefenderReplyUnknown
        }
        bot_corridor::ProofLimitCause::ModelScopeUnknown => ProofLimitCause::ModelScopeUnknown,
        bot_corridor::ProofLimitCause::OutsideScanWindow => ProofLimitCause::OutsideScanWindow,
    }
}

fn map_bot_search_diagnostics(diagnostics: bot_corridor::SearchDiagnostics) -> SearchDiagnostics {
    SearchDiagnostics {
        search_nodes: diagnostics.search_nodes,
        branch_probes: diagnostics.branch_probes,
        max_depth_reached: diagnostics.max_depth_reached,
    }
}

struct DefenderReplyProof {
    outcome: DefenderReplyOutcome,
    principal_line: Vec<Move>,
    limit_causes: Vec<ProofLimitCause>,
}

fn classify_defender_reply_for_report(
    board: &Board,
    attacker: Color,
    mv: Move,
    options: &AnalysisOptions,
) -> DefenderReplyProof {
    map_bot_defender_reply_proof(bot_corridor::classify_defender_reply(
        board,
        attacker,
        mv,
        &options.corridor_options(),
    ))
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
    for candidate in bot_corridor::defender_model_reply_candidates(
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
            RootCause::StrategicLoss
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
    if input.root_cause == RootCause::MissedDefense {
        push_note(&mut notes, TacticalNote::AccidentalBlunder);
    }
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
    if input.root_cause == RootCause::StrategicLoss {
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
        local_threat_facts_for_player as local_threat_facts, LocalThreatFact, LocalThreatKind,
        LocalThreatOrigin,
    };
    use gomoku_core::{Board, Color, Move, Replay, RuleConfig, Variant};

    use super::{
        analyze_defender_reply_options, analyze_replay, replay_moves, replay_prefix_boards,
        replay_proof_summary, AnalysisOptions, DefenderReplyOutcome, DefenderReplyRole,
        ProofLimitCause, ProofStatus, ReplayAnalysisSession, ReplayFrameHighlightRole,
        ReplayFrameMarkerRole, ReplyClassification, ReplyPolicy, RootCause, TacticalNote,
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
        assert_eq!(analysis.critical_mistake_ply, Some(8));
        assert_eq!(analysis.root_cause, RootCause::MissedDefense);
        assert!(analysis
            .tactical_notes
            .contains(&TacticalNote::AccidentalBlunder));
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
        assert_eq!(analysis.critical_mistake_ply, Some(14));

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
    fn replay_analysis_session_emits_possible_escape_marker_annotations() {
        let replay = replay_from_moves(
            Variant::Renju,
            &[
                "H8", "I8", "H7", "I7", "H6", "H5", "I6", "I9", "G6", "J6", "G8", "J5", "G5", "G7",
                "E6", "F6", "H9", "H10", "F7", "D5", "I10",
            ],
        );
        let mut session = ReplayAnalysisSession::new(
            replay,
            AnalysisOptions {
                reply_policy: ReplyPolicy::CorridorReplies,
                max_depth: 4,
                max_scan_plies: Some(8),
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
            .find(|frame| frame.ply == 13)
            .expect("escape boundary frame should be annotated");

        assert!(boundary.markers.iter().any(|marker| {
            marker.role == ReplayFrameMarkerRole::PossibleEscape
                && marker.mv == mv("I10")
                && marker.side == Color::White
        }));
    }

    #[test]
    fn replay_analysis_session_emits_current_imminent_and_counter_highlights() {
        let replay = replay_from_moves(
            Variant::Renju,
            &[
                "H8", "I8", "H7", "I7", "H6", "H5", "I6", "I9", "G6", "J6", "G8", "J5", "G5", "G7",
                "E6", "F6", "H9", "H10", "F7", "D5", "I10",
            ],
        );
        let mut session = ReplayAnalysisSession::new(
            replay,
            AnalysisOptions {
                reply_policy: ReplyPolicy::CorridorReplies,
                max_depth: 4,
                max_scan_plies: Some(8),
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
            .find(|frame| frame.ply == 13)
            .expect("escape boundary frame should be annotated");

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
            .any(|evidence| evidence.actual_reply == Some(mv("B9"))
                && evidence.reply_classification == ReplyClassification::BlockedButForced
                && evidence.escape_replies.is_empty()));
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
        assert_eq!(analysis.critical_mistake_ply, Some(23));
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
            "non-corridor actual defender move must not become a model reply: {replies:?}"
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
        assert_eq!(analysis.critical_mistake_ply, Some(10));
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
