use gomoku_core::{Color, Move, Replay};
use serde::Serialize;

use crate::analysis::{
    AnalysisBoardSnapshot, AnalysisOptions, DefenderReplyAnalysis, DefenderReplyOutcome,
    DefenderReplyRole, FailureAnalysis, ForcedInterval, LethalOnset, ProofLimitCause, ProofStatus,
    ReplyClassification, ReplyPolicy, RootCause, SearchDiagnostics, TacticalNote, UnclearContext,
    UnclearReason,
};
use crate::report::ReportProvenance;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AnalysisBatchReport {
    pub schema_version: u32,
    pub source_kind: String,
    pub source: String,
    pub replay_dir: String,
    pub total: usize,
    pub analyzed: usize,
    pub failed: usize,
    pub elapsed_ms: u64,
    pub total_elapsed_ms: u64,
    pub model: AnalysisBatchModel,
    pub summary: AnalysisBatchSummary,
    pub limit_cause_counts: Vec<ProofLimitCauseCount>,
    pub entries: Vec<AnalysisBatchEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AnalysisBatchModel {
    pub reply_policy: ReplyPolicy,
    pub max_depth: usize,
    pub max_scan_plies: Option<usize>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct AnalysisBatchSummary {
    pub unclear: usize,
    pub ongoing_or_draw: usize,
    pub analysis_error: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProofLimitCauseCount {
    pub cause: ProofLimitCause,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AnalysisBatchEntry {
    pub path: String,
    pub status: AnalysisBatchEntryStatus,
    pub winner: Option<Color>,
    pub move_count: Option<usize>,
    pub root_cause: Option<RootCause>,
    pub unclear_reason: Option<UnclearReason>,
    pub final_move: Option<Move>,
    pub lethal_onset: Option<LethalOnset>,
    pub setup_corridor: Option<ForcedInterval>,
    pub final_forced_interval_found: bool,
    pub final_forced_interval: Option<ForcedInterval>,
    pub proof_intervals: Vec<ForcedInterval>,
    pub last_chance_ply: Option<usize>,
    pub critical_loser_ply: Option<usize>,
    pub tactical_notes: Vec<TacticalNote>,
    pub failure: Option<FailureAnalysis>,
    pub principal_line: Vec<Move>,
    pub unknown_gaps: Vec<usize>,
    pub unknown_gap_count: usize,
    pub unclear_context: Option<UnclearContext>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof_details: Option<AnalysisBatchProofDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof_detail_diagnostics: Option<SearchDiagnostics>,
    pub limit_causes: Vec<ProofLimitCause>,
    pub elapsed_ms: u64,
    pub prefixes_analyzed: usize,
    pub forced_prefix_count: usize,
    pub unknown_prefix_count: usize,
    pub escape_prefix_count: usize,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AnalysisBatchProofDetails {
    pub previous_prefix_ply: Option<usize>,
    pub final_forced_start_ply: usize,
    pub previous_proof: Option<AnalysisBatchProofSnapshot>,
    pub final_start_proof: Option<AnalysisBatchProofSnapshot>,
    pub snapshots: Vec<AnalysisBoardSnapshot>,
    pub proof_frames: Vec<AnalysisBatchProofFrame>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AnalysisBatchProofSnapshot {
    pub prefix_ply: usize,
    pub attacker: Color,
    pub side_to_move: Color,
    pub status: ProofStatus,
    pub reply_classification: Option<ReplyClassification>,
    pub winning_squares: Vec<Move>,
    pub legal_cost_squares: Vec<Move>,
    pub illegal_cost_squares: Vec<Move>,
    pub defender_immediate_wins: Vec<Move>,
    pub escape_replies: Vec<Move>,
    pub forced_replies: Vec<Move>,
    pub principal_line: Vec<Move>,
    pub principal_line_notation: Vec<String>,
    pub limit_hit: bool,
    pub limit_causes: Vec<ProofLimitCause>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AnalysisBatchProofFrame {
    pub label: String,
    pub ply: usize,
    pub side_to_move: Color,
    pub status: ProofStatus,
    pub move_played: Option<Move>,
    pub move_played_notation: Option<String>,
    pub lethal_onset_reached: bool,
    pub rows: Vec<String>,
    pub markers: Vec<AnalysisBatchProofMarker>,
    pub reply_outcomes: Vec<DefenderReplyAnalysis>,
}

pub const PUBLISHED_ANALYSIS_REPORT_SCHEMA_VERSION: u32 = 3;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct PublishedAnalysisProvenance {
    pub generated_at_utc: Option<String>,
    pub generated_at_local: Option<String>,
    pub git_commit: Option<String>,
    pub git_dirty: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub command: Vec<String>,
}

impl From<&ReportProvenance> for PublishedAnalysisProvenance {
    fn from(value: &ReportProvenance) -> Self {
        Self {
            generated_at_utc: value.generated_at_utc.clone(),
            generated_at_local: value.generated_at_local.clone(),
            git_commit: value.git_commit.clone(),
            git_dirty: value.git_dirty,
            command: value.command.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PublishedAnalysisReport {
    pub schema_version: u32,
    pub report_kind: String,
    pub source_kind: String,
    pub source_report: String,
    pub provenance: PublishedAnalysisProvenance,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_report_provenance: Option<PublishedAnalysisProvenance>,
    pub selector: String,
    pub total: usize,
    pub analyzed: usize,
    pub failed: usize,
    pub elapsed_ms: u64,
    pub total_elapsed_ms: u64,
    pub model: AnalysisBatchModel,
    pub summary: AnalysisBatchSummary,
    pub sections: Vec<PublishedAnalysisSection>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PublishedAnalysisSection {
    pub label: String,
    pub entrant_a: String,
    pub entrant_b: String,
    pub total: usize,
    pub analyzed: usize,
    pub failed: usize,
    pub summary: AnalysisBatchSummary,
    pub entries: Vec<PublishedAnalysisEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PublishedAnalysisEntry {
    pub path: String,
    pub match_report: PublishedAnalysisMatchSummary,
    pub status: AnalysisBatchEntryStatus,
    pub root_cause: Option<RootCause>,
    pub unclear_reason: Option<UnclearReason>,
    pub lethal_onset: Option<LethalOnset>,
    pub setup_corridor: Option<ForcedInterval>,
    pub last_chance_ply: Option<usize>,
    pub critical_loser_ply: Option<usize>,
    pub failure: Option<FailureAnalysis>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof_details: Option<PublishedAnalysisProofDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_details: Option<PublishedAnalysisSearchDetails>,
    pub elapsed_ms: u64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PublishedAnalysisMatchSummary {
    pub match_index: usize,
    pub black: String,
    pub white: String,
    pub result: String,
    pub winner: Option<String>,
    pub end_reason: String,
    pub move_cells: Vec<usize>,
    pub move_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PublishedAnalysisProofDetails {
    pub proof_frames: Vec<PublishedAnalysisProofFrame>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PublishedAnalysisSearchDetails {
    pub search_nodes: usize,
    pub branch_probes: usize,
    pub max_depth_reached: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PublishedAnalysisProofFrame {
    pub label: String,
    pub ply: usize,
    pub side_to_move: Color,
    pub status: ProofStatus,
    pub move_played_notation: Option<String>,
    pub lethal_onset_reached: bool,
    pub markers: Vec<PublishedAnalysisProofMarker>,
    pub reply_outcomes: Vec<PublishedAnalysisReplyOutcome>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PublishedAnalysisProofMarker {
    pub notation: String,
    pub kinds: Vec<AnalysisBatchProofMarkerKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PublishedAnalysisReplyOutcome {
    pub notation: String,
    pub roles: Vec<DefenderReplyRole>,
    pub outcome: DefenderReplyOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishedAnalysisSectionInput {
    pub label: String,
    pub entrant_a: String,
    pub entrant_b: String,
    pub matches: Vec<PublishedAnalysisMatchSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AnalysisBatchProofMarker {
    pub mv: Move,
    pub notation: String,
    pub kinds: Vec<AnalysisBatchProofMarkerKind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisBatchProofMarkerKind {
    Winning,
    Threat,
    ImminentDefense,
    OffensiveCounter,
    WinningEvidence,
    ThreatEvidence,
    ImminentEvidence,
    OffensiveEvidence,
    CorridorEntryBlack,
    CorridorEntryWhite,
    Forbidden,
    ForcedLoss,
    ConfirmedEscape,
    PossibleEscape,
    ImmediateLoss,
    UnknownOutcome,
    Actual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisBatchEntryStatus {
    Analyzed,
    Error,
}

#[derive(Debug, Clone)]
pub struct ReplayAnalysisInput {
    pub label: String,
    pub replay: Replay,
}

#[derive(Debug, Clone)]
pub struct AnalysisBatchRunOptions {
    pub analysis: AnalysisOptions,
    pub include_proof_details: bool,
}

impl From<AnalysisOptions> for AnalysisBatchRunOptions {
    fn from(analysis: AnalysisOptions) -> Self {
        Self {
            analysis,
            include_proof_details: false,
        }
    }
}
