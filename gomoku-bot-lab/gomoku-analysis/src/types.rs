use gomoku_bot::corridor as bot_corridor;
pub use gomoku_bot::corridor::{
    DefenderReplyAnalysis, DefenderReplyOutcome, DefenderReplyRole, ProofLimitCause,
    SearchDiagnostics,
};
use gomoku_bot::tactical::LethalThreatKind;
use gomoku_core::{Color, Move};
use serde::Serialize;

pub const ANALYSIS_SCHEMA_VERSION: u32 = 20;
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
pub struct VisibleDefenderReplyCandidate {
    pub mv: Move,
    pub notation: String,
    pub roles: Vec<DefenderReplyRole>,
}

pub type DefenderReplyCandidate = VisibleDefenderReplyCandidate;

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
    pub(crate) fn corridor_options(&self) -> bot_corridor::CorridorOptions {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LethalOnsetComponentTier {
    Four,
    Three,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LethalOnsetComponent {
    pub tier: LethalOnsetComponentTier,
    pub mv: Move,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LethalOnsetMechanism {
    MultiRoute,
    ForbiddenCover,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LethalOnsetShape {
    pub label: String,
    pub components: Vec<LethalOnsetComponent>,
    pub mechanisms: Vec<LethalOnsetMechanism>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LethalOnset {
    pub prefix_ply: usize,
    pub attacker: Color,
    pub defender: Color,
    pub kind: LethalThreatKind,
    pub shape: LethalOnsetShape,
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
    pub evidence: Vec<ReplayFrameHighlight>,
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
