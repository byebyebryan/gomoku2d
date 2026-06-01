use serde::Serialize;
use serde_json::Value;

use crate::{
    AnalysisOptions, GameAnalysis, ReplayAnalysisCounters, ReplayAnalysisStep,
    ReplayAnalysisStepStatus, ReplayFrameAnnotations,
};

pub const REPLAY_ANALYZER_STEP_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplayAnalysisBridgeStatus {
    Running,
    Resolved,
    Unclear,
    Unsupported,
    Error,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReplayAnalysisStepEnvelope {
    pub schema_version: u32,
    pub status: ReplayAnalysisBridgeStatus,
    pub done: bool,
    pub current_ply: Option<usize>,
    pub annotations: Vec<ReplayFrameAnnotations>,
    pub analysis: Option<GameAnalysis>,
    pub error: Option<String>,
    pub counters: ReplayAnalysisCounters,
}

impl ReplayAnalysisStepEnvelope {
    pub fn from_step(step: ReplayAnalysisStep) -> Self {
        Self {
            schema_version: REPLAY_ANALYZER_STEP_SCHEMA_VERSION,
            status: replay_analysis_status(step.status),
            done: step.done,
            current_ply: step.current_ply,
            annotations: step.annotations,
            analysis: step.analysis,
            error: None,
            counters: step.counters,
        }
    }
}

pub fn replay_analysis_error(message: impl Into<String>) -> ReplayAnalysisStepEnvelope {
    ReplayAnalysisStepEnvelope {
        schema_version: REPLAY_ANALYZER_STEP_SCHEMA_VERSION,
        status: ReplayAnalysisBridgeStatus::Error,
        done: true,
        current_ply: None,
        annotations: Vec::new(),
        analysis: None,
        error: Some(message.into()),
        counters: ReplayAnalysisCounters::default(),
    }
}

pub fn analysis_options_from_json(options_json: &str) -> Result<AnalysisOptions, String> {
    let trimmed = options_json.trim();
    if trimmed.is_empty() {
        return Ok(AnalysisOptions::default());
    }

    let value = serde_json::from_str::<Value>(trimmed)
        .map_err(|err| format!("invalid options json: {err}"))?;
    let mut options = AnalysisOptions::default();

    let Some(object) = value.as_object() else {
        return Err("analysis options must be a JSON object".to_string());
    };

    if let Some(max_depth) = object.get("max_depth") {
        let value = max_depth
            .as_u64()
            .ok_or_else(|| "max_depth must be a non-negative integer".to_string())?;
        options.max_depth = value as usize;
    }

    if let Some(max_scan_plies) = object.get("max_scan_plies") {
        options.max_scan_plies = if max_scan_plies.is_null() {
            None
        } else {
            Some(max_scan_plies.as_u64().ok_or_else(|| {
                "max_scan_plies must be null or a non-negative integer".to_string()
            })? as usize)
        };
    }

    Ok(options)
}

fn replay_analysis_status(status: ReplayAnalysisStepStatus) -> ReplayAnalysisBridgeStatus {
    match status {
        ReplayAnalysisStepStatus::Running => ReplayAnalysisBridgeStatus::Running,
        ReplayAnalysisStepStatus::Resolved => ReplayAnalysisBridgeStatus::Resolved,
        ReplayAnalysisStepStatus::Unclear => ReplayAnalysisBridgeStatus::Unclear,
        ReplayAnalysisStepStatus::Unsupported => ReplayAnalysisBridgeStatus::Unsupported,
    }
}
