use std::collections::BTreeMap;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use gomoku_bot::tactical::{corridor_active_threats, LocalThreatKind};
use gomoku_core::{Board, Color, Move, Replay};
use rayon::prelude::*;

use crate::analysis::{
    analyze_alternate_defender_reply_options, analyze_replay, defender_reply_roles_for_move,
    replay_frame_annotations_for_analysis, visible_defender_reply_candidates,
    AnalysisBoardSnapshot, AnalysisOptions, DefenderReplyAnalysis, DefenderReplyCandidate,
    DefenderReplyOutcome, DefenderReplyRole, ForcedInterval, GameAnalysis, ProofLimitCause,
    ProofResult, ProofStatus, ReplayFrameAnnotations, ReplayFrameHighlightRole,
    ReplayFrameMarkerRole, RootCause, SearchDiagnostics, ANALYSIS_SCHEMA_VERSION,
};
use crate::report::ReportProvenance;

mod types;

pub use types::{
    AnalysisBatchEntry, AnalysisBatchEntryStatus, AnalysisBatchModel, AnalysisBatchProofDetails,
    AnalysisBatchProofFrame, AnalysisBatchProofMarker, AnalysisBatchProofMarkerKind,
    AnalysisBatchProofSnapshot, AnalysisBatchReport, AnalysisBatchRunOptions, AnalysisBatchSummary,
    ProofLimitCauseCount, PublishedAnalysisEntry, PublishedAnalysisMatchSummary,
    PublishedAnalysisProofDetails, PublishedAnalysisProofFrame, PublishedAnalysisProofMarker,
    PublishedAnalysisProvenance, PublishedAnalysisReplyOutcome, PublishedAnalysisReport,
    PublishedAnalysisSearchDetails, PublishedAnalysisSection, PublishedAnalysisSectionInput,
    ReplayAnalysisInput, PUBLISHED_ANALYSIS_REPORT_SCHEMA_VERSION,
};

mod proof_frames;
mod published;
mod runner;

use proof_frames::*;
pub use published::published_analysis_report_from_batch;
pub use runner::{
    run_analysis_batch, run_analysis_batch_replays, run_analysis_batch_replays_with_options,
    run_analysis_batch_replays_with_progress, run_analysis_batch_with_options,
    run_analysis_batch_with_progress,
};

#[cfg(test)]
mod tests;
