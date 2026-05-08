use std::collections::BTreeMap;
use std::path::Path;
use std::time::Instant;

use gomoku_core::{Board, Color, Move, Replay};
use rayon::prelude::*;
use serde::Serialize;

use crate::analysis::{
    analyze_alternate_defender_reply_options_with_retry, analyze_replay,
    defender_reply_roles_for_move, AnalysisBoardSnapshot, AnalysisOptions, DefenderReplyAnalysis,
    DefenderReplyOutcome, DefenderReplyRole, DefensePolicy, ForcedInterval, GameAnalysis,
    ProofLimitCause, ProofResult, ProofStatus, ReplyClassification, RootCause, TacticalNote,
    UnclearContext, UnclearReason, ANALYSIS_SCHEMA_VERSION,
};
use crate::report_board::{render_report_board, report_board_css, ReportBoardMarker};

const TACTICAL_ERROR_MIN_CORRIDOR_SPAN: usize = 5;
const STRATEGIC_LOSS_MIN_CORRIDOR_SPAN: usize = 9;

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
    pub defense_policy: DefensePolicy,
    pub max_depth: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deep_retry_depth: Option<usize>,
    pub deep_retry_limit: usize,
    pub max_backward_window: Option<usize>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct AnalysisBatchSummary {
    pub mistake: usize,
    pub tactical_error: usize,
    pub strategic_loss: usize,
    pub missed_defense: usize,
    pub missed_win: usize,
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
    pub loss_category: Option<AnalysisLossCategory>,
    pub root_cause: Option<RootCause>,
    pub unclear_reason: Option<UnclearReason>,
    pub final_move: Option<Move>,
    pub final_forced_interval_found: bool,
    pub final_forced_interval: Option<ForcedInterval>,
    pub proof_intervals: Vec<ForcedInterval>,
    pub last_chance_ply: Option<usize>,
    pub critical_mistake_ply: Option<usize>,
    pub tactical_notes: Vec<TacticalNote>,
    pub principal_line: Vec<Move>,
    pub unknown_gaps: Vec<usize>,
    pub unknown_gap_count: usize,
    pub unclear_context: Option<UnclearContext>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof_details: Option<AnalysisBatchProofDetails>,
    pub limit_causes: Vec<ProofLimitCause>,
    pub elapsed_ms: u64,
    pub prefixes_analyzed: usize,
    pub forced_prefix_count: usize,
    pub unknown_prefix_count: usize,
    pub escape_prefix_count: usize,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisLossCategory {
    Mistake,
    TacticalError,
    StrategicLoss,
    Unclear,
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
    pub rows: Vec<String>,
    pub markers: Vec<AnalysisBatchProofMarker>,
    pub reply_outcomes: Vec<DefenderReplyAnalysis>,
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
    Forbidden,
    ForcedLoss,
    Escape,
    UnprovedEscape,
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
    pub deep_retry_depth: Option<usize>,
    pub deep_retry_limit: usize,
}

impl From<AnalysisOptions> for AnalysisBatchRunOptions {
    fn from(analysis: AnalysisOptions) -> Self {
        Self {
            analysis,
            include_proof_details: false,
            deep_retry_depth: None,
            deep_retry_limit: 1,
        }
    }
}

pub fn run_analysis_batch(
    replay_dir: &Path,
    options: AnalysisOptions,
) -> Result<AnalysisBatchReport, String> {
    run_analysis_batch_with_options(replay_dir, options.into())
}

pub fn run_analysis_batch_with_options(
    replay_dir: &Path,
    options: AnalysisBatchRunOptions,
) -> Result<AnalysisBatchReport, String> {
    let batch_started = Instant::now();
    let model = model_from_options(&options);
    let mut paths = replay_paths(replay_dir)?;
    paths.sort();

    let mut summary = AnalysisBatchSummary::default();
    let mut analyzed = 0;
    let mut failed = 0;

    let entries = paths
        .par_iter()
        .map(|path| {
            let entry_started = Instant::now();
            let relative_path = path
                .strip_prefix(replay_dir)
                .unwrap_or(path)
                .display()
                .to_string();
            match analyze_replay_file(path, options.analysis.clone()) {
                Ok((replay, analysis)) => entry_from_analysis(
                    relative_path,
                    analysis,
                    replay.moves.len(),
                    elapsed_millis(entry_started.elapsed()),
                    options.include_proof_details.then_some(&replay),
                    options.deep_retry_depth,
                    options.deep_retry_limit,
                ),
                Err(error) => error_entry(
                    relative_path,
                    error,
                    elapsed_millis(entry_started.elapsed()),
                ),
            }
        })
        .collect::<Vec<_>>();

    for entry in &entries {
        if entry.status == AnalysisBatchEntryStatus::Analyzed {
            analyzed += 1;
            increment_summary_from_entry(&mut summary, entry);
        } else {
            failed += 1;
            summary.analysis_error += 1;
        }
    }
    let total_elapsed_ms = entries.iter().map(|entry| entry.elapsed_ms).sum();
    let limit_cause_counts = limit_cause_counts(&entries);

    Ok(AnalysisBatchReport {
        schema_version: ANALYSIS_SCHEMA_VERSION,
        source_kind: "replay_dir".to_string(),
        source: replay_dir.display().to_string(),
        replay_dir: replay_dir.display().to_string(),
        total: entries.len(),
        analyzed,
        failed,
        elapsed_ms: elapsed_millis(batch_started.elapsed()),
        total_elapsed_ms,
        model,
        summary,
        limit_cause_counts,
        entries,
    })
}

pub fn run_analysis_batch_replays(
    source: String,
    inputs: Vec<ReplayAnalysisInput>,
    options: AnalysisOptions,
) -> AnalysisBatchReport {
    run_analysis_batch_replays_with_options(source, inputs, options.into())
}

pub fn run_analysis_batch_replays_with_options(
    source: String,
    inputs: Vec<ReplayAnalysisInput>,
    options: AnalysisBatchRunOptions,
) -> AnalysisBatchReport {
    let batch_started = Instant::now();
    let model = model_from_options(&options);

    let entries = inputs
        .par_iter()
        .map(|input| {
            let entry_started = Instant::now();
            match analyze_replay(&input.replay, options.analysis.clone()) {
                Ok(analysis) => entry_from_analysis(
                    input.label.clone(),
                    analysis,
                    input.replay.moves.len(),
                    elapsed_millis(entry_started.elapsed()),
                    options.include_proof_details.then_some(&input.replay),
                    options.deep_retry_depth,
                    options.deep_retry_limit,
                ),
                Err(error) => error_entry(
                    input.label.clone(),
                    error.to_string(),
                    elapsed_millis(entry_started.elapsed()),
                ),
            }
        })
        .collect::<Vec<_>>();

    let mut summary = AnalysisBatchSummary::default();
    let mut analyzed = 0;
    let mut failed = 0;
    for entry in &entries {
        if entry.status == AnalysisBatchEntryStatus::Analyzed {
            analyzed += 1;
            increment_summary_from_entry(&mut summary, entry);
        } else {
            failed += 1;
            summary.analysis_error += 1;
        }
    }
    let total_elapsed_ms = entries.iter().map(|entry| entry.elapsed_ms).sum();
    let limit_cause_counts = limit_cause_counts(&entries);

    AnalysisBatchReport {
        schema_version: ANALYSIS_SCHEMA_VERSION,
        source_kind: "report_replays".to_string(),
        replay_dir: source.clone(),
        source,
        total: entries.len(),
        analyzed,
        failed,
        elapsed_ms: elapsed_millis(batch_started.elapsed()),
        total_elapsed_ms,
        model,
        summary,
        limit_cause_counts,
        entries,
    }
}

fn model_from_options(options: &AnalysisBatchRunOptions) -> AnalysisBatchModel {
    let deep_retry_depth = if options.include_proof_details && options.deep_retry_limit > 0 {
        options.deep_retry_depth
    } else {
        None
    };
    AnalysisBatchModel {
        defense_policy: options.analysis.defense_policy,
        max_depth: options.analysis.max_depth,
        deep_retry_depth,
        deep_retry_limit: if deep_retry_depth.is_some() {
            options.deep_retry_limit
        } else {
            0
        },
        max_backward_window: options.analysis.max_backward_window,
    }
}

fn elapsed_millis(duration: std::time::Duration) -> u64 {
    duration.as_millis().min(u128::from(u64::MAX)) as u64
}

fn error_entry(path: String, error: String, elapsed_ms: u64) -> AnalysisBatchEntry {
    AnalysisBatchEntry {
        path,
        status: AnalysisBatchEntryStatus::Error,
        winner: None,
        move_count: None,
        loss_category: None,
        root_cause: None,
        unclear_reason: None,
        final_move: None,
        final_forced_interval_found: false,
        final_forced_interval: None,
        proof_intervals: Vec::new(),
        last_chance_ply: None,
        critical_mistake_ply: None,
        tactical_notes: Vec::new(),
        principal_line: Vec::new(),
        unknown_gaps: Vec::new(),
        unknown_gap_count: 0,
        unclear_context: None,
        proof_details: None,
        limit_causes: Vec::new(),
        elapsed_ms,
        prefixes_analyzed: 0,
        forced_prefix_count: 0,
        unknown_prefix_count: 0,
        escape_prefix_count: 0,
        error: Some(error),
    }
}

pub fn render_analysis_batch_report_html(report: &AnalysisBatchReport) -> String {
    let limit_summary = limit_cause_counts_label(&report.limit_cause_counts);
    let entries = report
        .entries
        .iter()
        .map(analysis_entry_card_html)
        .collect::<String>();
    let model_label = "Corridor search";
    let model_config = corridor_search_config_label(report);
    let runtime_label = format!(
        "{} wall / {} entries",
        format_duration_ms(report.elapsed_ms),
        format_duration_ms(report.total_elapsed_ms)
    );

    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Gomoku2D Analysis Batch Report</title>
  <style>
    :root {{
      color-scheme: dark;
      --bg: #1e1e1e;
      --surface: #2a2a2a;
      --surface-strong: #333333;
      --card: #232323;
      --border: #575756;
      --line: var(--border);
      --text: #f5f5f5;
      --text-muted: #a6a6a0;
      --muted: var(--text-muted);
      --accent: #fccb57;
      --green: #5ad17a;
      --teal: #5fc7c2;
      --orange: #f08c4e;
      --cyan: #4ecdc4;
      --red: #ff5d5d;
      --pink: #ff7ab6;
      --blue: #58a6ff;
      --purple: #b877ff;
      --faint: #6f7a86;
    }}
    * {{
      box-sizing: border-box;
    }}
    body {{
      margin: 0;
      background: var(--bg);
      color: var(--text);
      font: 16px/1.4 ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
    }}
    main {{
      display: grid;
      gap: 24px;
      max-width: 1180px;
      margin: 0 auto;
      padding: 32px;
    }}
    h1, h2, h3, p {{
      margin: 0;
    }}
    a {{
      color: inherit;
      text-decoration: none;
    }}
    .hero {{
      background: var(--surface);
      border: 2px solid var(--border);
      display: grid;
      gap: 16px;
      overflow: auto;
      padding: 20px;
    }}
    .top-links {{
      display: flex;
      flex-wrap: wrap;
      gap: 8px;
    }}
    .top-links a {{
      background: var(--surface-strong);
      border: 2px solid var(--border);
      color: var(--text);
      display: inline-block;
      padding: 8px 12px;
      text-transform: uppercase;
    }}
    .top-links a:hover,
    .top-links a:focus {{
      border-color: var(--teal);
      outline: none;
    }}
    .eyebrow {{
      color: var(--accent);
      font-size: 12px;
      letter-spacing: .16em;
      text-transform: uppercase;
    }}
    h1 {{
      font-size: clamp(34px, 7vw, 64px);
      line-height: 1;
    }}
    .run-strip {{
      display: grid;
      gap: 8px;
      padding: 0;
    }}
    .run-group {{
      display: flex;
      flex-wrap: wrap;
      gap: 8px;
    }}
    .run-chip, .card {{
      background: var(--card);
      border: 1px solid var(--border);
      padding: 7px 10px;
    }}
    .run-chip {{
      display: inline-flex;
      gap: 8px;
      align-items: baseline;
      min-width: 0;
    }}
    .run-chip span, .card span, .detail span, .entry-metric span {{
      color: var(--muted);
      font-size: 11px;
      letter-spacing: 0.1em;
      text-transform: uppercase;
    }}
    .run-chip strong {{
      color: var(--green);
      font-size: 14px;
      line-height: 1.2;
      overflow-wrap: anywhere;
    }}
    .summary-grid {{
      background: var(--surface);
      border: 2px solid var(--border);
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
      gap: 12px;
      padding: 20px;
    }}
    .card strong {{
      display: block;
      margin-top: 4px;
      color: var(--accent);
      font-size: 20px;
    }}
    .card--mistake strong {{ color: var(--orange); }}
    .card--tactical strong {{ color: var(--accent); }}
    .card--strategic strong {{ color: var(--red); }}
    .card--unclear strong {{ color: var(--blue); }}
    .analysis-list {{
      background: var(--surface);
      border: 2px solid var(--border);
      display: grid;
      gap: 12px;
      padding: 20px;
    }}
    .analysis-entry {{
      border: 1px solid var(--border);
      border-left-width: 4px;
      background: var(--card);
    }}
    .analysis-entry:hover {{
      border-top-color: var(--teal);
      border-right-color: var(--teal);
      border-bottom-color: var(--teal);
    }}
    .analysis-entry--mistake {{ border-left-color: var(--orange); }}
    .analysis-entry--tactical-error {{ border-left-color: var(--accent); }}
    .analysis-entry--strategic-loss {{ border-left-color: var(--red); }}
    .analysis-entry--unclear {{ border-left-color: var(--blue); }}
    .analysis-entry--none {{ border-left-color: var(--faint); }}
    .analysis-entry[open] {{
      border-top-color: var(--accent);
      border-right-color: var(--accent);
      border-bottom-color: var(--accent);
    }}
    .analysis-entry summary {{
      cursor: pointer;
      display: grid;
      gap: 10px;
      grid-template-columns: minmax(68px, max-content) minmax(210px, 1.25fr) minmax(210px, 1.25fr) minmax(118px, max-content) repeat(2, minmax(86px, 1fr));
      align-items: center;
      padding: 12px 14px;
    }}
    .analysis-entry summary > * {{
      min-width: 0;
    }}
    .entry-title {{
      color: var(--text);
      overflow-wrap: anywhere;
    }}
    .entry-match {{
      display: block;
      color: var(--text);
      font-size: 14px;
    }}
    .entry-player {{
      display: grid;
      gap: 5px;
    }}
    .entry-player-head {{
      align-items: center;
      display: flex;
      flex-wrap: wrap;
      gap: 8px;
    }}
    .entry-player-color {{
      color: var(--muted);
      font-size: 11px;
      letter-spacing: .12em;
      text-transform: uppercase;
    }}
    .player-result {{
      border: 1px solid currentColor;
      font-size: 10px;
      letter-spacing: .08em;
      padding: 2px 5px;
      text-transform: uppercase;
    }}
    .player-result--win {{ color: var(--green); }}
    .player-result--lose {{ color: var(--red); }}
    .player-result--draw {{ color: var(--muted); }}
    .player-result--none {{ color: var(--faint); }}
    .entry-bots {{
      color: var(--muted);
    }}
    .bot-label {{
      display: inline-flex;
      flex-direction: column;
      gap: 2px;
      line-height: 1.2;
    }}
    .bot-label span:first-child {{
      color: var(--text);
      font-size: 12px;
    }}
    .bot-label span + span {{
      color: var(--muted);
      font-size: 11px;
      letter-spacing: .08em;
    }}
    .versus {{
      color: var(--faint);
      font-size: 11px;
      text-transform: uppercase;
    }}
    .entry-metric {{
      border-left: 1px solid var(--border);
      font-variant-numeric: tabular-nums;
      padding-left: 10px;
      text-align: right;
    }}
    .entry-metric strong {{
      color: var(--text);
      display: block;
      overflow-wrap: anywhere;
    }}
    .loss-chip {{
      border: 1px solid currentColor;
      display: inline-flex;
      justify-content: center;
      padding: 4px 8px;
      text-transform: uppercase;
      white-space: nowrap;
    }}
    .loss-chip--mistake {{ color: var(--orange); }}
    .loss-chip--tactical-error {{ color: var(--accent); }}
    .loss-chip--strategic-loss {{ color: var(--red); }}
    .loss-chip--unclear {{ color: var(--blue); }}
    .loss-chip--none {{ color: var(--muted); }}
    .entry-body {{
      border-top: 1px solid var(--border);
      display: grid;
      gap: 14px;
      padding: 14px;
    }}
    .detail-sections {{
      display: grid;
      gap: 10px;
      grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
    }}
    .detail-section {{
      display: grid;
      gap: 8px;
      align-content: start;
    }}
    .detail-section h3 {{
      color: var(--accent);
      font-size: 10px;
      letter-spacing: .12em;
      margin: 0;
      text-transform: uppercase;
    }}
    .detail-grid {{
      display: grid;
      gap: 8px;
      grid-template-columns: repeat(auto-fit, minmax(132px, 1fr));
    }}
    .detail {{
      background: var(--surface-strong);
      border: 1px solid var(--border);
      padding: 10px;
    }}
    .detail strong {{
      display: block;
      margin-top: 3px;
      overflow-wrap: anywhere;
    }}
    .entry-panels {{
      display: grid;
      gap: 12px;
    }}
    .entry-panel {{
      background: var(--surface-strong);
      border: 1px solid var(--border);
      padding: 12px;
    }}
    .entry-panel h2 {{
      color: var(--accent);
      font-size: 12px;
      letter-spacing: .1em;
      margin: 0 0 8px;
      text-transform: uppercase;
    }}
    .context {{
      color: var(--muted);
    }}
    .context strong {{
      color: var(--text);
    }}
    .context div + div {{
      margin-top: 4px;
    }}
    .context details {{
      margin-top: 8px;
    }}
    .context summary {{
      cursor: pointer;
      color: var(--accent);
    }}
    .context pre {{
      overflow: auto;
      margin: 8px 0 0;
      padding: 8px;
      border: 1px solid var(--border);
      background: var(--card);
      color: var(--text);
      line-height: 1.2;
    }}
    .proof-summary-strip {{
      display: grid;
      gap: 8px;
      grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
      margin-bottom: 12px;
    }}
    .proof-summary-strip div {{
      background: var(--surface-strong);
      border: 1px solid var(--line);
      padding: 8px;
    }}
    .proof-summary-strip span {{
      display: block;
      color: var(--faint);
      font-size: 9px;
      letter-spacing: 0.08em;
      text-transform: uppercase;
    }}
    .proof-summary-strip strong {{
      display: block;
      margin-top: 3px;
      overflow-wrap: anywhere;
    }}
    .proof-frames {{
      display: grid;
      gap: 12px;
    }}
    .proof-frames h3 {{
      margin: 0;
      color: var(--accent);
      font-size: 12px;
      letter-spacing: .1em;
      text-transform: uppercase;
    }}
    .proof-frame-list {{
      display: grid;
      gap: 12px;
      margin-top: 2px;
    }}
    .proof-frame {{
      border: 1px solid var(--line);
      background: var(--card);
      display: grid;
      gap: 12px;
      grid-template-columns: max-content minmax(220px, 1fr);
      padding: 10px;
    }}
    .proof-frame h3 {{
      margin: 0 0 6px;
      color: var(--text);
      font-size: 12px;
      letter-spacing: 0.05em;
    }}
    .proof-frame p {{
      margin: 0 0 8px;
      color: var(--muted);
      font-size: 11px;
    }}
    .proof-frame-copy {{
      min-width: 0;
    }}
    .proof-frame-lines {{
      display: grid;
      gap: 6px;
      margin: 8px 0 0;
    }}
    .proof-frame-line {{
      display: grid;
      gap: 8px;
      grid-template-columns: 92px minmax(0, 1fr);
      align-items: baseline;
      color: var(--muted);
      font-size: 11px;
    }}
    .proof-frame-line span {{
      color: var(--faint);
      font-size: 9px;
      letter-spacing: 0.08em;
      text-transform: uppercase;
    }}
    .proof-frame-line strong {{
      color: var(--text);
      overflow-wrap: anywhere;
    }}
    {board_css}
    .marker--winning,
    .marker--threat,
    .marker--imminent-defense,
    .marker--offensive-counter {{
      --proof-hint-color: transparent;
    }}
    .marker--winning::after,
    .marker--threat::after,
    .marker--imminent-defense::after,
    .marker--offensive-counter::after {{
      content: "";
      position: absolute;
      inset: 0;
      z-index: 3;
      pointer-events: none;
      box-shadow: inset 0 0 0 2px var(--proof-hint-color);
      transform: translateY(-1px);
    }}
    .marker--winning {{
      --proof-hint-color: var(--green);
    }}
    .marker--threat {{
      --proof-hint-color: var(--red);
    }}
    .marker--imminent-defense {{
      --proof-hint-color: var(--pink);
    }}
    .marker--offensive-counter {{
      --proof-hint-color: var(--purple);
    }}
    .marker--actual {{
    }}
    .marker--actual-black .proof-marker {{
      color: #07090c;
      text-shadow: 0 1px 0 rgba(255,255,255,0.36);
    }}
    .marker--actual-white .proof-marker {{
      color: #f6eed8;
      text-shadow: 0 1px 0 #000;
    }}
    .proof-cell--stone-black.marker--actual .proof-marker {{
      color: var(--text);
      text-shadow: 0 1px 0 #000;
    }}
    .marker--side-black .proof-marker {{
      color: #07090c;
      text-shadow: 0 1px 0 rgba(255,255,255,0.36);
    }}
    .marker--side-white .proof-marker {{
      color: #f6eed8;
      text-shadow: 0 1px 0 #000;
    }}
    .marker--escape .proof-marker {{
      color: var(--green);
      text-shadow: 0 1px 0 rgba(0,0,0,0.45);
    }}
    .marker--unproved-escape .proof-marker {{
      color: var(--cyan);
      text-shadow: 0 1px 0 rgba(0,0,0,0.45);
    }}
    .marker--unknown-outcome .proof-marker {{
      color: var(--muted);
    }}
    .proof-legend {{
      display: grid;
      gap: 6px;
      margin: 8px 0;
      color: var(--muted);
      font-size: 11px;
    }}
    .proof-legend-row {{
      display: flex;
      flex-wrap: wrap;
      gap: 8px 12px;
    }}
    .legend-role::before {{
      content: "";
      display: inline-block;
      width: 10px;
      height: 10px;
      margin-right: 5px;
      vertical-align: -1px;
      border: 1px solid currentColor;
    }}
    .legend-winning::before {{ color: var(--green); }}
    .legend-threat::before {{ color: var(--red); }}
    .legend-imminent::before {{ color: var(--pink); }}
    .legend-offensive::before {{ color: var(--purple); }}
    .legend-forbidden .legend-marker {{
      color: #f6eed8;
      text-shadow: 0 1px 0 #000;
    }}
    .legend-outcome {{
      display: inline-flex;
      align-items: baseline;
      gap: 5px;
    }}
    .legend-marker {{
      font-weight: 900;
      letter-spacing: 0.02em;
      text-shadow: 0 1px 0 rgba(0,0,0,0.45);
    }}
    .legend-marker--white {{
      color: #f6eed8;
      text-shadow: 0 1px 0 #000;
    }}
    .legend-escape .legend-marker {{
      color: var(--green);
    }}
    .legend-unproved .legend-marker {{
      color: var(--cyan);
    }}
    .legend-unknown .legend-marker {{
      color: var(--muted);
    }}
    .reply-outcomes {{
      margin-top: 10px;
      border-top: 1px solid var(--line);
      padding-top: 8px;
      display: grid;
      gap: 4px;
      font-size: 11px;
      color: var(--muted);
    }}
    .reply-outcome-row {{
      display: grid;
      grid-template-columns: minmax(38px, max-content) minmax(96px, 0.8fr) minmax(116px, 0.9fr) minmax(180px, 1.6fr);
      gap: 10px;
      align-items: baseline;
    }}
    .reply-outcome-row > * {{
      min-width: 0;
    }}
    .reply-outcome-row strong {{
      color: var(--text);
    }}
    .reply-outcome-row span {{
      overflow-wrap: anywhere;
    }}
    .reply-outcome-row span:last-child {{
      display: -webkit-box;
      overflow-wrap: normal;
      overflow: hidden;
      word-break: normal;
      -webkit-box-orient: vertical;
      -webkit-line-clamp: 2;
    }}
    .reply-outcome-row--header {{
      color: var(--faint);
      text-transform: uppercase;
      letter-spacing: 0.08em;
      font-size: 9px;
    }}
    @media (max-width: 920px) {{
      main {{
        padding: 16px;
      }}
      .hero, .summary-grid, .analysis-list {{
        padding: 16px;
      }}
      .run-chip {{
        justify-content: space-between;
        width: 100%;
      }}
      .analysis-entry summary {{
        grid-template-columns: minmax(0, 1fr) max-content;
      }}
      .entry-title, .entry-player {{
        grid-column: 1 / -1;
      }}
      .entry-metric {{
        border-left: 0;
        display: flex;
        grid-column: 1 / -1;
        justify-content: space-between;
        padding-left: 0;
      }}
      .entry-metric strong {{
        display: inline;
        text-align: right;
      }}
      .proof-frame {{
        grid-template-columns: minmax(0, 1fr);
      }}
    }}
  </style>
</head>
<body>
<main>
  <header class="hero">
    <nav class="top-links" aria-label="Project links">
      <a href="/">Game</a>
      <a href="/assets/">Assets</a>
      <a href="/bot-report/">Bots</a>
    </nav>
    <p class="eyebrow">Gomoku2D Bot Lab</p>
    <h1>Replay Analysis</h1>
    <div class="run-strip" aria-label="Run summary">
      <div class="run-group" aria-label="Analysis setup">
        <div class="run-chip"><span>Model</span><strong>{model}</strong></div>
        <div class="run-chip"><span>Config</span><strong>{model_config}</strong></div>
      </div>
      <div class="run-group" aria-label="Run stats">
        <div class="run-chip"><span>Runtime</span><strong>{runtime}</strong></div>
        <div class="run-chip"><span>Limit hits</span><strong>{limit_summary}</strong></div>
      </div>
      <div class="run-group" aria-label="Analysis provenance">
        <div class="run-chip"><span>Source</span><strong>{source}</strong></div>
      </div>
    </div>
  </header>
  <section class="summary-grid" aria-label="Analysis summary">
    <article class="card"><span>Analyzed</span><strong>{analyzed}</strong></article>
    <article class="card card--mistake"><span>Mistake</span><strong>{mistake}</strong></article>
    <article class="card card--tactical"><span>Tactical error</span><strong>{tactical_error}</strong></article>
    <article class="card card--strategic"><span>Strategic loss</span><strong>{strategic_loss}</strong></article>
    <article class="card card--unclear"><span>Unclear</span><strong>{unclear}</strong></article>
    <article class="card"><span>Errors</span><strong>{failed}</strong></article>
  </section>
  <section class="analysis-list" aria-label="Replay analysis entries">{entries}</section>
</main>
</body>
</html>
"#,
        analyzed = report.analyzed,
        mistake = report.summary.mistake,
        tactical_error = report.summary.tactical_error,
        strategic_loss = report.summary.strategic_loss,
        unclear = report.summary.unclear,
        failed = report.failed,
        model = html_escape(model_label),
        source = html_escape(&format!("{}: {}", report.source_kind, report.source)),
        runtime = html_escape(&runtime_label),
        limit_summary = html_escape(&limit_summary),
        model_config = html_escape(&model_config),
        board_css = report_board_css(),
        entries = entries,
    )
}

fn analysis_entry_card_html(entry: &AnalysisBatchEntry) -> String {
    let loss_label = loss_category_label(entry.loss_category);
    let loss_class = loss_category_class(entry.loss_category);
    let entry_class = loss_entry_class(entry.loss_category);
    let forced = forced_interval_label(entry.final_forced_interval.as_ref());
    let detail_sections = analysis_entry_detail_sections_html(entry, &forced);
    let title = replay_entry_title(&entry.path);
    let (first_player, second_player) = ordered_player_columns_html(&title, entry.winner);
    let panels = analysis_entry_panels_html(entry);

    format!(
        r#"<details class="analysis-entry {entry_class}">
  <summary>
    <strong class="entry-title"><span class="entry-match">{match_label}</span></strong>
    {first_player}
    {second_player}
    <span class="loss-chip {loss_class}">{loss}</span>
    <span class="entry-metric"><span>Forced ply</span><strong>{forced}</strong></span>
    <span class="entry-metric"><span>Total ply</span><strong>{length}</strong></span>
  </summary>
  <div class="entry-body">
    {detail_sections}
    <div class="entry-panels">{panels}</div>
  </div>
</details>"#,
        entry_class = entry_class,
        loss_class = loss_class,
        match_label = html_escape(&title.match_label),
        first_player = first_player,
        second_player = second_player,
        loss = html_escape(&loss_label),
        forced = html_escape(&forced),
        length = html_escape(&ply_count_label(entry.move_count)),
        detail_sections = detail_sections,
        panels = panels,
    )
}

fn analysis_entry_panels_html(entry: &AnalysisBatchEntry) -> String {
    let mut panels = Vec::new();
    if let Some(context) = entry.unclear_context.as_ref() {
        panels.push(format!(
            "<section class=\"entry-panel\"><h2>Unclear context</h2>{}</section>",
            unclear_context_html(Some(context))
        ));
    }
    if let Some(details) = entry.proof_details.as_ref() {
        panels.push(format!(
            "<section class=\"entry-panel\"><h2>Proof details</h2>{}</section>",
            proof_details_html(Some(details))
        ));
    }
    if let Some(error) = entry.error.as_deref() {
        panels.push(format!(
            "<section class=\"entry-panel\"><h2>Error</h2><p>{}</p></section>",
            html_escape(error)
        ));
    }
    if panels.is_empty() {
        return "<p class=\"context\">No expanded proof details for this entry.</p>".to_string();
    }
    panels.join("")
}

fn analysis_entry_detail_sections_html(entry: &AnalysisBatchEntry, forced: &str) -> String {
    let cause = root_cause_label(entry.root_cause);
    let unclear_reason = unclear_reason_label(entry.unclear_reason);
    let tactical_notes = tactical_notes_label(&entry.tactical_notes);
    let limit_causes = proof_limit_cause_labels(&entry.limit_causes);
    let winning_move = entry
        .final_move
        .map(Move::to_notation)
        .unwrap_or_else(|| "-".to_string());
    let prefixes = format!(
        "{} checked / {} forced / {} escape",
        entry.prefixes_analyzed, entry.forced_prefix_count, entry.escape_prefix_count
    );

    let mut outcome_details = vec![
        detail_html("Status", entry_status_label(entry.status)),
        detail_html("Cause", &cause),
        detail_html("Notes", &tactical_notes),
    ];
    if unclear_reason != "-" {
        outcome_details.push(detail_html("Unclear", &unclear_reason));
    }

    let corridor_details = vec![
        detail_html("Forced ply", forced),
        detail_html(
            "Last escape",
            &before_ply_option_label(entry.last_chance_ply),
        ),
        detail_html(
            "Forced entry",
            &entry
                .final_forced_interval
                .as_ref()
                .map(|interval| before_ply_label(interval.start_ply))
                .unwrap_or_else(|| "-".to_string()),
        ),
        detail_html(
            "Critical reply",
            &ply_option_label(entry.critical_mistake_ply),
        ),
        detail_html("Winning move", &winning_move),
    ];

    let search_details = vec![
        detail_html("Prefixes", &prefixes),
        detail_html("Unknown gaps", &entry.unknown_gap_count.to_string()),
        detail_html("Limit hits", &limit_causes),
        detail_html("Runtime", &format_duration_ms(entry.elapsed_ms)),
    ];

    format!(
        r#"<div class="detail-sections">{outcome}{corridor}{search}</div>"#,
        outcome = detail_section_html("Outcome", &outcome_details),
        corridor = detail_section_html("Corridor", &corridor_details),
        search = detail_section_html("Search", &search_details),
    )
}

struct ReplayEntryTitle {
    match_label: String,
    black: Option<String>,
    white: Option<String>,
}

fn replay_entry_title(path: &str) -> ReplayEntryTitle {
    let label = path.rsplit('/').next().unwrap_or(path);
    let label = label.strip_suffix(".json").unwrap_or(label);
    let Some((left, bot_b)) = label.split_once("__vs__") else {
        return ReplayEntryTitle {
            match_label: match_label_for_display(label),
            black: None,
            white: None,
        };
    };
    let Some((match_label, bot_a)) = left.split_once("__") else {
        return ReplayEntryTitle {
            match_label: match_label_for_display(label),
            black: None,
            white: None,
        };
    };

    ReplayEntryTitle {
        match_label: match_label_for_display(match_label),
        black: Some(bot_name_from_report_label(bot_a)),
        white: Some(bot_name_from_report_label(bot_b)),
    }
}

fn match_label_for_display(label: &str) -> String {
    let Some(match_number) = label.strip_prefix("match_") else {
        return label.to_string();
    };
    match_number
        .parse::<usize>()
        .map(|number| format!("#{number}"))
        .unwrap_or_else(|_| format!("#{match_number}"))
}

fn ordered_player_columns_html(
    title: &ReplayEntryTitle,
    winner: Option<Color>,
) -> (String, String) {
    let black = || player_column_html("Black", title.black.as_deref(), winner, Color::Black);
    let white = || player_column_html("White", title.white.as_deref(), winner, Color::White);

    match winner {
        Some(Color::Black) | None => (black(), white()),
        Some(Color::White) => (white(), black()),
    }
}

fn player_column_html(
    color_label: &str,
    bot: Option<&str>,
    winner: Option<Color>,
    color: Color,
) -> String {
    let (result, result_class) = player_result_label(winner, color);
    let bot = bot
        .map(compact_bot_label_html)
        .unwrap_or_else(|| "<span class=\"bot-label\"><span>-</span></span>".to_string());
    format!(
        r#"<span class="entry-player"><span class="entry-player-head"><span class="entry-player-color">{color}</span><span class="player-result {result_class}">{result}</span></span><span class="entry-bots">{bot}</span></span>"#,
        color = html_escape(color_label),
        result = html_escape(result),
        result_class = result_class,
        bot = bot,
    )
}

fn player_result_label(winner: Option<Color>, color: Color) -> (&'static str, &'static str) {
    match winner {
        Some(winner) if winner == color => ("win", "player-result--win"),
        Some(_) => ("lose", "player-result--lose"),
        None => ("draw", "player-result--draw"),
    }
}

fn bot_name_from_report_label(label: &str) -> String {
    label.replace('_', "+")
}

fn compact_bot_label_html(bot: &str) -> String {
    let (primary, modifiers) = compact_bot_label_parts(bot);
    let modifiers = modifiers
        .map(|modifiers| format!("<span>{}</span>", html_escape(&modifiers)))
        .unwrap_or_default();
    format!(
        "<span class=\"bot-label\"><span>{}</span>{}</span>",
        html_escape(&primary),
        modifiers
    )
}

fn compact_bot_label_parts(bot: &str) -> (String, Option<String>) {
    let label = compact_bot_label(bot);
    let Some((primary, modifiers)) = label.split_once('+') else {
        return (label, None);
    };

    (
        primary.to_string(),
        Some(modifiers.split('+').collect::<Vec<_>>().join(" + ")),
    )
}

fn compact_bot_label(bot: &str) -> String {
    if bot == "random" {
        return "RandomBot".to_string();
    }

    let mut parts = bot.split('+');
    let Some(base) = parts.next() else {
        return bot.to_string();
    };
    let Some(depth) = searchbot_base_depth(base) else {
        return bot.to_string();
    };

    let mut label = format!("SearchBot_D{depth}");
    for feature in parts {
        label.push('+');
        label.push_str(&compact_searchbot_feature_label(feature));
    }
    label
}

fn searchbot_base_depth(bot: &str) -> Option<i32> {
    match bot {
        "fast" => Some(2),
        "balanced" => Some(3),
        "deep" => Some(5),
        "baseline" | "search" => Some(5),
        _ => bot
            .strip_prefix("baseline-")
            .or_else(|| bot.strip_prefix("search-"))
            .map(|depth| depth.strip_prefix('d').unwrap_or(depth))
            .and_then(|depth| depth.parse::<i32>().ok()),
    }
}

fn compact_searchbot_feature_label(feature: &str) -> String {
    if let Some(cap) = feature.strip_prefix("tactical-cap-") {
        return format!("TCap{cap}");
    }
    if let Some(cap) = feature.strip_prefix("child-cap-") {
        return format!("Cap{cap}");
    }
    if let Some(radius) = feature.strip_prefix("near-all-r") {
        return format!("NearR{radius}");
    }
    if let Some(rest) = feature.strip_prefix("near-self-r") {
        if let Some((self_radius, opponent_radius)) = rest.split_once("-opponent-r") {
            return format!("SelfR{self_radius}OppR{opponent_radius}");
        }
    }

    match feature {
        "pattern-eval" => "Pattern".to_string(),
        "tactical-first" => "Tactical".to_string(),
        "no-safety" => "NoSafety".to_string(),
        "opponent-reply-search-probe" => "SearchProbe".to_string(),
        "opponent-reply-local-threat-probe" => "LocalThreat".to_string(),
        _ => feature.to_string(),
    }
}

fn detail_html(label: &str, value: &str) -> String {
    format!(
        "<div class=\"detail\"><span>{}</span><strong>{}</strong></div>",
        html_escape(label),
        html_escape(value)
    )
}

fn detail_section_html(label: &str, details: &[String]) -> String {
    format!(
        r#"<section class="detail-section"><h3>{}</h3><div class="detail-grid">{}</div></section>"#,
        html_escape(label),
        details.join("")
    )
}

fn corridor_search_config_label(report: &AnalysisBatchReport) -> String {
    let retry = match (report.model.deep_retry_depth, report.model.deep_retry_limit) {
        (Some(depth), limit) if limit > 0 => format!("retry depth {depth} x{limit}"),
        _ => "retry off".to_string(),
    };
    let window = report
        .model
        .max_backward_window
        .map(|window| window.to_string())
        .unwrap_or_else(|| "unbounded".to_string());

    format!(
        "{} / depth {} / window {} / {}",
        defense_policy_label(report.model.defense_policy),
        report.model.max_depth,
        window,
        retry
    )
}

fn defense_policy_label(policy: DefensePolicy) -> &'static str {
    match policy {
        DefensePolicy::AllLegalDefense => "all legal defense",
        DefensePolicy::TacticalDefense => "tactical defense",
        DefensePolicy::HybridDefense => "hybrid defense",
    }
}

fn loss_category_class(loss_category: Option<AnalysisLossCategory>) -> &'static str {
    match loss_category {
        Some(AnalysisLossCategory::Mistake) => "loss-chip--mistake",
        Some(AnalysisLossCategory::TacticalError) => "loss-chip--tactical-error",
        Some(AnalysisLossCategory::StrategicLoss) => "loss-chip--strategic-loss",
        Some(AnalysisLossCategory::Unclear) => "loss-chip--unclear",
        None => "loss-chip--none",
    }
}

fn loss_entry_class(loss_category: Option<AnalysisLossCategory>) -> &'static str {
    match loss_category {
        Some(AnalysisLossCategory::Mistake) => "analysis-entry--mistake",
        Some(AnalysisLossCategory::TacticalError) => "analysis-entry--tactical-error",
        Some(AnalysisLossCategory::StrategicLoss) => "analysis-entry--strategic-loss",
        Some(AnalysisLossCategory::Unclear) => "analysis-entry--unclear",
        None => "analysis-entry--none",
    }
}

fn forced_interval_label(interval: Option<&ForcedInterval>) -> String {
    let Some(interval) = interval else {
        return "-".to_string();
    };
    let span = interval
        .end_ply
        .saturating_sub(interval.start_ply)
        .saturating_add(1);
    format!("{}-{} / {} ply", interval.start_ply, interval.end_ply, span)
}

fn ply_option_label(value: Option<usize>) -> String {
    value
        .map(|value| format!("ply {value}"))
        .unwrap_or_else(|| "-".to_string())
}

fn before_ply_option_label(value: Option<usize>) -> String {
    value
        .map(before_ply_label)
        .unwrap_or_else(|| "-".to_string())
}

fn ply_count_label(value: Option<usize>) -> String {
    value
        .map(|value| format!("{value} ply"))
        .unwrap_or_else(|| "-".to_string())
}

fn tactical_notes_label(notes: &[TacticalNote]) -> String {
    if notes.is_empty() {
        return "-".to_string();
    }
    notes
        .iter()
        .map(|note| match note {
            TacticalNote::AccidentalBlunder => "accidental blunder",
            TacticalNote::ConversionError => "conversion error",
            TacticalNote::MissedWin => "missed win",
            TacticalNote::StrongAttack => "strong attack",
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_duration_ms(ms: u64) -> String {
    if ms < 1_000 {
        format!("{ms} ms")
    } else if ms < 60_000 {
        format!("{:.2} s", ms as f64 / 1_000.0)
    } else {
        let minutes = ms / 60_000;
        let seconds = (ms % 60_000) as f64 / 1_000.0;
        format!("{minutes}m {seconds:.1}s")
    }
}

fn replay_paths(replay_dir: &Path) -> Result<Vec<std::path::PathBuf>, String> {
    let entries = std::fs::read_dir(replay_dir)
        .map_err(|err| format!("failed to read replay directory: {err}"))?;
    let mut paths = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|err| format!("failed to read replay directory entry: {err}"))?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) == Some("json") {
            paths.push(path);
        }
    }
    Ok(paths)
}

fn analyze_replay_file(
    path: &Path,
    options: AnalysisOptions,
) -> Result<(Replay, GameAnalysis), String> {
    let json = std::fs::read_to_string(path)
        .map_err(|err| format!("failed to read replay JSON: {err}"))?;
    let replay =
        Replay::from_json(&json).map_err(|err| format!("failed to parse replay: {err}"))?;
    let analysis = analyze_replay(&replay, options)
        .map_err(|err| format!("failed to analyze replay: {err}"))?;
    Ok((replay, analysis))
}

fn entry_from_analysis(
    path: String,
    analysis: GameAnalysis,
    move_count: usize,
    elapsed_ms: u64,
    replay: Option<&Replay>,
    deep_retry_depth: Option<usize>,
    deep_retry_limit: usize,
) -> AnalysisBatchEntry {
    let prefixes_analyzed = analysis.proof_summary.len();
    let forced_prefix_count = count_proof_status(&analysis, ProofStatus::ForcedWin);
    let unknown_prefix_count = count_proof_status(&analysis, ProofStatus::Unknown);
    let escape_prefix_count = count_proof_status(&analysis, ProofStatus::EscapeFound);
    let proof_details = replay.and_then(|replay| {
        proof_details_from_analysis(replay, &analysis, deep_retry_depth, deep_retry_limit)
    });
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
        loss_category: loss_category_for_analysis(&analysis),
        root_cause: Some(analysis.root_cause),
        unclear_reason: analysis.unclear_reason,
        final_move: analysis.final_move,
        final_forced_interval_found: analysis.final_forced_interval_found,
        final_forced_interval: Some(analysis.final_forced_interval),
        proof_intervals: analysis.proof_intervals,
        last_chance_ply: analysis.last_chance_ply,
        critical_mistake_ply: analysis.critical_mistake_ply,
        tactical_notes: analysis.tactical_notes,
        principal_line: analysis.principal_line,
        unknown_gaps: analysis.unknown_gaps.clone(),
        unknown_gap_count: analysis.unknown_gaps.len(),
        unclear_context: analysis.unclear_context,
        proof_details,
        limit_causes,
        elapsed_ms,
        prefixes_analyzed,
        forced_prefix_count,
        unknown_prefix_count,
        escape_prefix_count,
        error: None,
    }
}

fn proof_details_from_analysis(
    replay: &Replay,
    analysis: &GameAnalysis,
    deep_retry_depth: Option<usize>,
    deep_retry_limit: usize,
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
    let proof_frames = proof_frames_for_actual_interval(
        replay,
        &boards,
        analysis,
        scan_start,
        deep_retry_depth,
        deep_retry_limit,
    );

    Some(AnalysisBatchProofDetails {
        previous_prefix_ply,
        final_forced_start_ply,
        previous_proof,
        final_start_proof,
        snapshots,
        proof_frames,
    })
}

fn proof_result_at(
    proofs: &[ProofResult],
    scan_start: usize,
    prefix_ply: usize,
) -> Option<&ProofResult> {
    proofs.get(prefix_ply.checked_sub(scan_start)?)
}

fn proof_snapshot(prefix_ply: usize, proof: &ProofResult) -> AnalysisBatchProofSnapshot {
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

fn proof_frames_for_actual_interval(
    replay: &Replay,
    boards: &[Board],
    analysis: &GameAnalysis,
    scan_start: usize,
    deep_retry_depth: Option<usize>,
    deep_retry_limit: usize,
) -> Vec<AnalysisBatchProofFrame> {
    let mut deep_retries_remaining = if deep_retry_depth.is_some() {
        deep_retry_limit
    } else {
        0
    };
    let first_ply = proof_frame_start_ply(boards, analysis);
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
            add_loser_tactical_hint_markers(&mut markers, board, analysis.winner);
            add_forbidden_cost_markers(&mut markers, proof, None);
            if analysis
                .winner
                .is_some_and(|winner| board.current_player == winner.opponent())
            {
                let actual_child =
                    proof_result_at(&analysis.proof_summary, scan_start, board_ply + 1);
                add_forbidden_cost_markers(
                    &mut markers,
                    actual_child,
                    Some(AnalysisBatchProofMarkerKind::ImminentDefense),
                );
            }
            let actual_move = actual_move_at_ply(replay, ply);
            let reply_outcomes = defender_reply_outcomes_for_frame(
                board,
                analysis,
                actual_move,
                deep_retry_depth,
                &mut deep_retries_remaining,
            );
            add_reply_outcome_markers(&mut markers, &reply_outcomes);
            if let Some(actual_move) = actual_move {
                add_actual_marker(&mut markers, board, analysis.winner, actual_move);
            }
            markers.sort_by_key(|marker| (marker.mv.row, marker.mv.col));
            Some(proof_frame(
                &label,
                ply,
                board,
                proof
                    .map(|proof| proof.status)
                    .unwrap_or(ProofStatus::Unknown),
                actual_move,
                markers,
                reply_outcomes,
            ))
        })
        .collect::<Vec<_>>()
}

fn proof_frame_start_ply(boards: &[Board], analysis: &GameAnalysis) -> usize {
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

fn defender_reply_outcomes_for_frame(
    board: &Board,
    analysis: &GameAnalysis,
    actual_move: Option<Move>,
    deep_retry_depth: Option<usize>,
    deep_retries_remaining: &mut usize,
) -> Vec<DefenderReplyAnalysis> {
    let Some(attacker) = analysis.winner else {
        return Vec::new();
    };
    if board.current_player != attacker.opponent() {
        return Vec::new();
    }

    let replies = analyze_alternate_defender_reply_options_with_retry(
        board,
        attacker,
        actual_move,
        &AnalysisOptions {
            defense_policy: analysis.model.defense_policy,
            max_depth: analysis.model.max_depth,
            max_backward_window: analysis.model.max_backward_window,
        },
        deep_retry_depth,
        *deep_retries_remaining,
    );
    let used = replies
        .iter()
        .filter(|reply| reply.deep_retry_depth.is_some())
        .count();
    *deep_retries_remaining = (*deep_retries_remaining).saturating_sub(used);
    replies
}

fn actual_frame_label(ply: usize, interval: &ForcedInterval) -> String {
    if ply == interval.end_ply {
        "winning_ply".to_string()
    } else {
        format!("actual_ply_{ply}")
    }
}

fn actual_move_at_ply(replay: &Replay, ply: usize) -> Option<Move> {
    let replay_move = replay.moves.get(ply.checked_sub(1)?)?;
    Move::from_notation(&replay_move.mv).ok()
}

fn add_loser_tactical_hint_markers(
    markers: &mut Vec<AnalysisBatchProofMarker>,
    board: &Board,
    winner: Option<Color>,
) {
    if !winner.is_some_and(|winner| board.current_player == winner.opponent()) {
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
}

fn add_forbidden_cost_markers(
    markers: &mut Vec<AnalysisBatchProofMarker>,
    proof: Option<&ProofResult>,
    tactical_role: Option<AnalysisBatchProofMarkerKind>,
) {
    let Some(proof) = proof else {
        return;
    };
    let moves = proof
        .threat_evidence
        .iter()
        .flat_map(|evidence| evidence.illegal_cost_squares.iter().copied())
        .collect::<Vec<_>>();
    if let Some(tactical_role) = tactical_role {
        add_marker_kind(markers, moves.iter().copied(), tactical_role);
    }
    add_marker_kind(markers, moves, AnalysisBatchProofMarkerKind::Forbidden);
}

fn proof_frame(
    label: &str,
    ply: usize,
    board: &Board,
    status: ProofStatus,
    move_played: Option<Move>,
    markers: Vec<AnalysisBatchProofMarker>,
    reply_outcomes: Vec<DefenderReplyAnalysis>,
) -> AnalysisBatchProofFrame {
    AnalysisBatchProofFrame {
        label: label.to_string(),
        ply,
        side_to_move: board.current_player,
        status,
        move_played,
        move_played_notation: move_played.map(Move::to_notation),
        rows: board_rows(board),
        markers,
        reply_outcomes,
    }
}

fn add_reply_outcome_markers(
    markers: &mut Vec<AnalysisBatchProofMarker>,
    replies: &[DefenderReplyAnalysis],
) {
    for reply in replies {
        for role in &reply.roles {
            let kind = match role {
                DefenderReplyRole::Actual => AnalysisBatchProofMarkerKind::Actual,
                DefenderReplyRole::ImmediateDefense => AnalysisBatchProofMarkerKind::Threat,
                DefenderReplyRole::ImminentDefense => AnalysisBatchProofMarkerKind::ImminentDefense,
                DefenderReplyRole::OffensiveCounter => {
                    AnalysisBatchProofMarkerKind::OffensiveCounter
                }
            };
            add_marker_kind(markers, [reply.mv], kind);
        }
        let outcome_kind = match reply.outcome {
            DefenderReplyOutcome::ForcedLoss => AnalysisBatchProofMarkerKind::ForcedLoss,
            DefenderReplyOutcome::Escape => AnalysisBatchProofMarkerKind::Escape,
            DefenderReplyOutcome::UnprovedEscape => AnalysisBatchProofMarkerKind::UnprovedEscape,
            DefenderReplyOutcome::ImmediateLoss => AnalysisBatchProofMarkerKind::ImmediateLoss,
            DefenderReplyOutcome::Unknown => AnalysisBatchProofMarkerKind::UnknownOutcome,
        };
        add_marker_kind(markers, [reply.mv], outcome_kind);
    }
}

fn add_actual_marker(
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

fn add_actual_hint_markers(
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
        match role {
            DefenderReplyRole::Actual => {}
            DefenderReplyRole::ImmediateDefense => {
                add_marker_kind(markers, [mv], AnalysisBatchProofMarkerKind::Threat);
            }
            DefenderReplyRole::ImminentDefense => {
                add_marker_kind(markers, [mv], AnalysisBatchProofMarkerKind::ImminentDefense);
            }
            DefenderReplyRole::OffensiveCounter => {
                add_marker_kind(
                    markers,
                    [mv],
                    AnalysisBatchProofMarkerKind::OffensiveCounter,
                );
            }
        }
    }
}

fn is_hint_marker_kind(kind: AnalysisBatchProofMarkerKind) -> bool {
    matches!(
        kind,
        AnalysisBatchProofMarkerKind::Winning
            | AnalysisBatchProofMarkerKind::Threat
            | AnalysisBatchProofMarkerKind::ImminentDefense
            | AnalysisBatchProofMarkerKind::OffensiveCounter
    )
}

fn add_marker_kind(
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

fn upsert_marker(
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

fn collect_evidence_moves(
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

fn extend_unique_moves(target: &mut Vec<Move>, moves: impl IntoIterator<Item = Move>) {
    for mv in moves {
        if !target.contains(&mv) {
            target.push(mv);
        }
    }
}

fn replay_prefix_boards(replay: &Replay) -> Result<Vec<Board>, String> {
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

fn limit_cause_counts(entries: &[AnalysisBatchEntry]) -> Vec<ProofLimitCauseCount> {
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

fn count_proof_status(analysis: &GameAnalysis, status: ProofStatus) -> usize {
    analysis
        .proof_summary
        .iter()
        .filter(|proof| proof.status == status)
        .count()
}

fn increment_summary_from_entry(summary: &mut AnalysisBatchSummary, entry: &AnalysisBatchEntry) {
    if entry.winner.is_none() {
        summary.ongoing_or_draw += 1;
        return;
    }

    match entry.loss_category {
        Some(AnalysisLossCategory::Mistake) => summary.mistake += 1,
        Some(AnalysisLossCategory::TacticalError) => summary.tactical_error += 1,
        Some(AnalysisLossCategory::StrategicLoss) => summary.strategic_loss += 1,
        Some(AnalysisLossCategory::Unclear) | None => summary.unclear += 1,
    }

    match entry.root_cause {
        Some(RootCause::MissedDefense) => summary.missed_defense += 1,
        Some(RootCause::MissedWin) => summary.missed_win += 1,
        Some(RootCause::StrategicLoss) | Some(RootCause::Unclear) | None => {}
    }
}

fn loss_category_for_analysis(analysis: &GameAnalysis) -> Option<AnalysisLossCategory> {
    analysis.winner?;
    if analysis.root_cause == RootCause::Unclear || !analysis.final_forced_interval_found {
        return Some(AnalysisLossCategory::Unclear);
    }
    if analysis.root_cause == RootCause::MissedWin {
        return Some(AnalysisLossCategory::Mistake);
    }

    let corridor_span = analysis
        .final_forced_interval
        .end_ply
        .saturating_sub(analysis.final_forced_interval.start_ply)
        + 1;
    Some(loss_category_for_corridor_span(corridor_span))
}

fn loss_category_for_corridor_span(corridor_span: usize) -> AnalysisLossCategory {
    if corridor_span < TACTICAL_ERROR_MIN_CORRIDOR_SPAN {
        AnalysisLossCategory::Mistake
    } else if corridor_span < STRATEGIC_LOSS_MIN_CORRIDOR_SPAN {
        AnalysisLossCategory::TacticalError
    } else {
        AnalysisLossCategory::StrategicLoss
    }
}

fn entry_status_label(status: AnalysisBatchEntryStatus) -> &'static str {
    match status {
        AnalysisBatchEntryStatus::Analyzed => "analyzed",
        AnalysisBatchEntryStatus::Error => "error",
    }
}

fn root_cause_label(root_cause: Option<RootCause>) -> String {
    root_cause
        .map(|root_cause| match root_cause {
            RootCause::StrategicLoss => "strategic loss",
            RootCause::MissedDefense => "missed defense",
            RootCause::MissedWin => "missed win",
            RootCause::Unclear => "unclear",
        })
        .unwrap_or("-")
        .to_string()
}

fn loss_category_label(loss_category: Option<AnalysisLossCategory>) -> String {
    loss_category
        .map(|category| match category {
            AnalysisLossCategory::Mistake => "mistake",
            AnalysisLossCategory::TacticalError => "tactical error",
            AnalysisLossCategory::StrategicLoss => "strategic loss",
            AnalysisLossCategory::Unclear => "unclear",
        })
        .unwrap_or("-")
        .to_string()
}

fn unclear_reason_label(unclear_reason: Option<UnclearReason>) -> String {
    unclear_reason
        .map(|reason| match reason {
            UnclearReason::PreviousPrefixUnknown => "previous prefix unknown",
            UnclearReason::ScanWindowCutoff => "scan window cutoff",
            UnclearReason::ProofLimitHit => "proof limit hit",
            UnclearReason::NoFinalForcedInterval => "no final forced interval",
            UnclearReason::DrawOrOngoing => "draw or ongoing",
        })
        .unwrap_or("-")
        .to_string()
}

fn unclear_context_html(context: Option<&UnclearContext>) -> String {
    let Some(context) = context else {
        return "-".to_string();
    };

    let previous_proof = match (
        context.previous_proof_status,
        context.previous_proof_limit_hit,
    ) {
        (Some(status), Some(true)) => format!("Previous proof: {status:?} (limit hit)"),
        (Some(status), Some(false)) => format!("Previous proof: {status:?}"),
        _ => "Previous proof: outside scan window".to_string(),
    };
    let principal_line = if context.principal_line_notation.is_empty() {
        "-".to_string()
    } else {
        context.principal_line_notation.join(" ")
    };
    let limit_causes = proof_limit_cause_labels(&context.previous_limit_causes);
    let snapshots = context
        .snapshots
        .iter()
        .map(|snapshot| {
            format!(
                "<div><strong>{} @ ply {}</strong><pre>{}</pre></div>",
                html_escape(&snapshot.label),
                snapshot.ply,
                html_escape(&snapshot.rows.join("\n"))
            )
        })
        .collect::<String>();

    format!(
        "<div class=\"context\"><div><strong>Prev ply</strong> {previous}</div><div>{previous_proof}; side {side}</div><div><strong>Limit causes</strong> {limit_causes}</div><div><strong>Line</strong> {line}</div><details><summary>Board snapshots</summary>{snapshots}</details></div>",
        previous = context
            .previous_prefix_ply
            .map(|ply| ply.to_string())
            .unwrap_or_else(|| "-".to_string()),
        previous_proof = html_escape(&previous_proof),
        side = html_escape(&option_debug(context.previous_side_to_move)),
        limit_causes = html_escape(&limit_causes),
        line = html_escape(&principal_line),
        snapshots = snapshots,
    )
}

fn proof_details_html(details: Option<&AnalysisBatchProofDetails>) -> String {
    let Some(details) = details else {
        return "-".to_string();
    };

    let previous_status = details
        .previous_proof
        .as_ref()
        .map(|proof| proof_status_label(proof.status))
        .unwrap_or("-");
    let final_status = details
        .final_start_proof
        .as_ref()
        .map(|proof| proof_status_label(proof.status))
        .unwrap_or("-");
    let frames = proof_frames_html(&details.proof_frames);

    format!(
        "<div class=\"context\"><div class=\"proof-summary-strip\"><div><span>Escape boundary</span><strong>{previous_ply}</strong></div><div><span>Forced run entry</span><strong>{final_ply}</strong></div><div><span>Proof status</span><strong>{previous_status} -> {final_status}</strong></div></div>{frames}</div>",
        previous_ply = details
            .previous_prefix_ply
            .map(before_ply_label)
            .unwrap_or_else(|| "-".to_string()),
        final_ply = before_ply_label(details.final_forced_start_ply),
        previous_status = html_escape(previous_status),
        final_status = html_escape(final_status),
        frames = frames,
    )
}

fn proof_frames_html(frames: &[AnalysisBatchProofFrame]) -> String {
    if frames.is_empty() {
        return String::new();
    }

    let winning_frame = frames
        .iter()
        .find(|frame| frame.label == "winning_ply")
        .or_else(|| frames.first());
    let Some(winning_frame) = winning_frame else {
        return String::new();
    };

    let winner = winning_frame.side_to_move;
    let final_card = proof_winning_frame_html(winning_frame);
    let turn_cards = proof_decision_turns_html(frames, winner, winning_frame.ply);

    format!(
        "<div class=\"proof-frames\"><h3>Proof frames</h3><div class=\"proof-legend\"><div class=\"proof-legend-row\"><span class=\"legend-role legend-winning\">immediate win</span><span class=\"legend-role legend-threat\">immediate threat</span><span class=\"legend-role legend-imminent\">defensive reply</span><span class=\"legend-role legend-offensive\">offensive reply</span></div><div class=\"proof-legend-row\"><span class=\"legend-outcome legend-immediate-loss\"><strong class=\"legend-marker legend-marker--white\">!</strong> immediate loss</span><span class=\"legend-outcome legend-forced\"><strong class=\"legend-marker legend-marker--white\">L</strong> forced loss</span><span class=\"legend-outcome legend-forbidden\"><strong class=\"legend-marker\">F</strong> forbidden</span><span class=\"legend-outcome legend-escape\"><strong class=\"legend-marker\">E</strong> escape</span><span class=\"legend-outcome legend-unproved\"><strong class=\"legend-marker\">U</strong> unproved escape</span><span class=\"legend-outcome legend-unknown\"><strong class=\"legend-marker\">?</strong> unknown</span></div></div><div class=\"proof-frame-list\">{final_card}{turn_cards}</div></div>",
        final_card = final_card,
        turn_cards = turn_cards,
    )
}

fn proof_winning_frame_html(frame: &AnalysisBatchProofFrame) -> String {
    let move_label = frame.move_played_notation.as_deref().unwrap_or("-");
    proof_frame_row_html(
        frame.ply,
        &format!("Final ply {}", frame.ply),
        frame,
        None,
        &[
            ("Winner move", format!("{}: {move_label}", frame.ply)),
            (
                "Side",
                format!(
                    "{:?} to move / {}",
                    frame.side_to_move,
                    proof_status_label(frame.status)
                ),
            ),
        ],
        "",
    )
}

fn proof_decision_turns_html(
    frames: &[AnalysisBatchProofFrame],
    winner: Color,
    winning_ply: usize,
) -> String {
    frames
        .iter()
        .filter(|frame| frame.ply != winning_ply && frame.side_to_move == winner.opponent())
        .map(|defender_frame| {
            let attacker_frame = defender_frame.ply.checked_sub(1).and_then(|attacker_ply| {
                frames
                    .iter()
                    .find(|frame| frame.ply == attacker_ply && frame.side_to_move == winner)
            });
            proof_decision_turn_html(defender_frame, attacker_frame)
        })
        .collect::<String>()
}

fn proof_decision_turn_html(
    defender_frame: &AnalysisBatchProofFrame,
    attacker_frame: Option<&AnalysisBatchProofFrame>,
) -> String {
    let title = attacker_frame
        .map(|attacker| format!("Turn {}-{}", attacker.ply, defender_frame.ply))
        .unwrap_or_else(|| format!("Before ply {}", defender_frame.ply));
    let attacker_move = attacker_frame
        .and_then(|frame| {
            frame
                .move_played_notation
                .as_deref()
                .map(|mv| format!("{}: {mv}", frame.ply))
        })
        .unwrap_or_else(|| "-".to_string());
    let defender_move = defender_frame
        .move_played_notation
        .as_deref()
        .map(|mv| format!("{}: {mv}", defender_frame.ply))
        .unwrap_or_else(|| "-".to_string());
    let extra_actual =
        attacker_frame.and_then(|frame| frame.move_played.map(|mv| (mv, frame.side_to_move)));
    let replies = reply_outcomes_html(defender_frame);

    proof_frame_row_html(
        defender_frame.ply,
        &title,
        defender_frame,
        extra_actual,
        &[
            ("Winner move", attacker_move),
            ("Loser reply", defender_move),
            (
                "Decision",
                format!(
                    "{:?} to respond / {}",
                    defender_frame.side_to_move,
                    proof_status_label(defender_frame.status)
                ),
            ),
        ],
        &replies,
    )
}

fn proof_frame_row_html(
    ply: usize,
    title: &str,
    board_frame: &AnalysisBatchProofFrame,
    extra_actual: Option<(Move, Color)>,
    lines: &[(&str, String)],
    extra_html: &str,
) -> String {
    let board = proof_board_with_extra_actual_html(board_frame, extra_actual);
    let lines = lines
        .iter()
        .map(|(label, value)| {
            format!(
                "<div class=\"proof-frame-line\"><span>{label}</span><strong>{value}</strong></div>",
                label = html_escape(label),
                value = html_escape(value),
            )
        })
        .collect::<String>();
    format!(
        "<article class=\"proof-frame\" data-ply=\"{ply}\">{board}<div class=\"proof-frame-copy\"><h3>{title}</h3><div class=\"proof-frame-lines\">{lines}</div>{extra_html}</div></article>",
        ply = ply,
        title = html_escape(title),
        board = board,
        lines = lines,
        extra_html = extra_html,
    )
}

fn reply_outcomes_html(frame: &AnalysisBatchProofFrame) -> String {
    if frame.reply_outcomes.is_empty() {
        return String::new();
    }

    let rows = frame
        .reply_outcomes
        .iter()
        .map(|reply| {
            let details = defender_reply_detail_label(reply);
            format!(
                "<div class=\"reply-outcome-row\"><strong>{mv}</strong><span>{roles}</span><span>{outcome}</span><span title=\"{details}\">{details}</span></div>",
                mv = html_escape(&reply.notation),
                roles = html_escape(&reply_roles_label(&reply.roles)),
                outcome = html_escape(&defender_reply_outcome_label(reply)),
                details = html_escape(&details),
            )
        })
        .collect::<String>();
    format!(
        "<div class=\"reply-outcomes\"><div class=\"reply-outcome-row reply-outcome-row--header\"><span>Move</span><span>Role</span><span>Outcome</span><span>Details</span></div>{rows}</div>",
    )
}

fn proof_board_with_extra_actual_html(
    frame: &AnalysisBatchProofFrame,
    extra_actual: Option<(Move, Color)>,
) -> String {
    let markers = frame
        .markers
        .iter()
        .map(|marker| {
            let label = marker_label(marker);
            let mut report_marker =
                ReportBoardMarker::new(marker.mv).with_classes(marker_classes(frame, marker));
            if !label.is_empty() {
                report_marker = report_marker.with_label(label);
            }
            if marker.kinds.contains(&AnalysisBatchProofMarkerKind::Actual) {
                report_marker = report_marker.with_actual_stone(frame.side_to_move);
            }
            report_marker
        })
        .collect::<Vec<_>>();
    let markers = with_extra_actual_marker(markers, extra_actual);
    render_report_board(&frame.rows, &markers)
}

fn with_extra_actual_marker(
    mut markers: Vec<ReportBoardMarker>,
    extra_actual: Option<(Move, Color)>,
) -> Vec<ReportBoardMarker> {
    let Some((mv, color)) = extra_actual else {
        return markers;
    };

    if let Some(marker) = markers
        .iter_mut()
        .find(|marker| marker.mv.row == mv.row && marker.mv.col == mv.col)
    {
        marker.actual_stone = Some(color);
        marker.hide_stone = true;
        if !marker.classes.iter().any(|class| class == "marker--actual") {
            marker.classes.push("marker--actual".to_string());
        }
        let actual_class = actual_marker_class(color);
        if !marker.classes.iter().any(|class| class == actual_class) {
            marker.classes.push(actual_class.to_string());
        }
        return markers;
    }

    markers.push(
        ReportBoardMarker::new(mv)
            .with_classes(["marker--actual", actual_marker_class(color)])
            .with_actual_stone(color)
            .without_underlying_stone(),
    );
    markers
}

fn actual_marker_class(color: Color) -> &'static str {
    match color {
        Color::Black => "marker--actual-black",
        Color::White => "marker--actual-white",
    }
}

#[cfg(test)]
fn cell_classes(
    frame: &AnalysisBatchProofFrame,
    stone: char,
    marker: Option<&AnalysisBatchProofMarker>,
) -> String {
    let mut classes = vec!["proof-cell"];
    match stone {
        'B' => classes.push("proof-cell--stone-black"),
        'W' => classes.push("proof-cell--stone-white"),
        _ => {}
    }
    if let Some(marker) = marker {
        classes.extend(marker_classes(frame, marker));
    }
    classes.join(" ")
}

fn marker_classes(
    frame: &AnalysisBatchProofFrame,
    marker: &AnalysisBatchProofMarker,
) -> Vec<&'static str> {
    let mut classes = Vec::new();
    for kind in &marker.kinds {
        if *kind == AnalysisBatchProofMarkerKind::Winning
            && frame.label == "winning_ply"
            && marker.kinds.contains(&AnalysisBatchProofMarkerKind::Actual)
        {
            continue;
        }
        classes.push(match kind {
            AnalysisBatchProofMarkerKind::Winning => "marker--winning",
            AnalysisBatchProofMarkerKind::Threat => "marker--threat",
            AnalysisBatchProofMarkerKind::ImminentDefense => "marker--imminent-defense",
            AnalysisBatchProofMarkerKind::OffensiveCounter => "marker--offensive-counter",
            AnalysisBatchProofMarkerKind::Forbidden => "marker--forbidden",
            AnalysisBatchProofMarkerKind::ForcedLoss => "marker--forced-loss",
            AnalysisBatchProofMarkerKind::Escape => "marker--escape",
            AnalysisBatchProofMarkerKind::UnprovedEscape => "marker--unproved-escape",
            AnalysisBatchProofMarkerKind::ImmediateLoss => "marker--immediate-loss",
            AnalysisBatchProofMarkerKind::UnknownOutcome => "marker--unknown-outcome",
            AnalysisBatchProofMarkerKind::Actual => "marker--actual",
        });
    }
    classes.push(match frame.side_to_move {
        Color::Black => "marker--side-black",
        Color::White => "marker--side-white",
    });
    if marker.kinds.contains(&AnalysisBatchProofMarkerKind::Actual) {
        classes.push(match frame.side_to_move {
            Color::Black => "marker--actual-black",
            Color::White => "marker--actual-white",
        });
    }
    classes
}

fn marker_label(marker: &AnalysisBatchProofMarker) -> String {
    if marker.kinds.contains(&AnalysisBatchProofMarkerKind::Actual) {
        return String::new();
    }
    if marker
        .kinds
        .contains(&AnalysisBatchProofMarkerKind::Forbidden)
    {
        return "F".to_string();
    }
    if marker
        .kinds
        .contains(&AnalysisBatchProofMarkerKind::ImmediateLoss)
    {
        return "!".to_string();
    }
    if marker
        .kinds
        .contains(&AnalysisBatchProofMarkerKind::ForcedLoss)
    {
        return "L".to_string();
    }
    if marker.kinds.contains(&AnalysisBatchProofMarkerKind::Escape) {
        return "E".to_string();
    }
    if marker
        .kinds
        .contains(&AnalysisBatchProofMarkerKind::UnprovedEscape)
    {
        return "U".to_string();
    }
    if marker
        .kinds
        .contains(&AnalysisBatchProofMarkerKind::UnknownOutcome)
    {
        return "?".to_string();
    }
    if marker.kinds.contains(&AnalysisBatchProofMarkerKind::Threat) {
        return "L".to_string();
    }
    if marker
        .kinds
        .contains(&AnalysisBatchProofMarkerKind::Winning)
    {
        return String::new();
    }
    String::new()
}

fn reply_roles_label(roles: &[DefenderReplyRole]) -> String {
    roles
        .iter()
        .map(|role| match role {
            DefenderReplyRole::Actual => "actual",
            DefenderReplyRole::ImmediateDefense => "immediate",
            DefenderReplyRole::ImminentDefense => "imminent",
            DefenderReplyRole::OffensiveCounter => "offensive",
        })
        .collect::<Vec<_>>()
        .join(" + ")
}

fn defender_reply_outcome_label(reply: &DefenderReplyAnalysis) -> String {
    match reply.outcome {
        DefenderReplyOutcome::ForcedLoss => "forced loss",
        DefenderReplyOutcome::Escape => "escape",
        DefenderReplyOutcome::UnprovedEscape => "unproved escape",
        DefenderReplyOutcome::ImmediateLoss => "immediate loss",
        DefenderReplyOutcome::Unknown => "unknown",
    }
    .to_string()
}

fn defender_reply_detail_label(reply: &DefenderReplyAnalysis) -> String {
    let mut parts = Vec::new();
    let line = reply_line_label(reply);
    if line != "-" {
        parts.push(line);
    }
    if let Some(depth) = reply.deep_retry_depth {
        if matches!(
            reply.outcome,
            DefenderReplyOutcome::Unknown | DefenderReplyOutcome::UnprovedEscape
        ) {
            parts.push(format!("deep {depth} tried"));
        } else {
            parts.push(format!("deep {depth}"));
        }
    }
    if !reply.limit_causes.is_empty() {
        parts.push(proof_limit_cause_labels(&reply.limit_causes));
    }
    if parts.is_empty() {
        "-".to_string()
    } else {
        parts.join("; ")
    }
}

fn reply_line_label(reply: &DefenderReplyAnalysis) -> String {
    if reply.principal_line_notation.is_empty() {
        return "-".to_string();
    }

    let mut line = Vec::with_capacity(reply.principal_line_notation.len() + 1);
    line.push(reply.notation.clone());
    line.extend(reply.principal_line_notation.clone());
    line.join(" ")
}

fn before_ply_label(prefix_ply: usize) -> String {
    format!("before ply {}", prefix_ply + 1)
}

fn proof_status_label(status: ProofStatus) -> &'static str {
    match status {
        ProofStatus::ForcedWin => "forced win",
        ProofStatus::EscapeFound => "escape found",
        ProofStatus::Unknown => "unknown",
    }
}

fn limit_cause_counts_label(counts: &[ProofLimitCauseCount]) -> String {
    if counts.is_empty() {
        return "none".to_string();
    }
    counts
        .iter()
        .map(|count| format!("{} {}", proof_limit_cause_label(count.cause), count.count))
        .collect::<Vec<_>>()
        .join(", ")
}

fn proof_limit_cause_labels(causes: &[ProofLimitCause]) -> String {
    if causes.is_empty() {
        return "-".to_string();
    }
    causes
        .iter()
        .map(|cause| proof_limit_cause_label(*cause))
        .collect::<Vec<_>>()
        .join(", ")
}

fn proof_limit_cause_label(cause: ProofLimitCause) -> &'static str {
    match cause {
        ProofLimitCause::DepthCutoff => "depth cutoff",
        ProofLimitCause::ReplyWidthCutoff => "reply-width cutoff",
        ProofLimitCause::AttackerChildUnknown => "attacker child unknown",
        ProofLimitCause::DefenderReplyUnknown => "defender reply unknown",
        ProofLimitCause::ModelScopeUnknown => "model-scope unknown",
        ProofLimitCause::OutsideScanWindow => "outside scan window",
    }
}

fn option_debug<T: std::fmt::Debug>(value: Option<T>) -> String {
    value
        .map(|value| format!("{value:?}"))
        .unwrap_or_else(|| "-".to_string())
}

fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use std::fs;

    use gomoku_core::{Board, Color, Move, Replay, RuleConfig, Variant};

    use super::{
        cell_classes, defender_reply_detail_label, defender_reply_outcome_label,
        loss_category_for_corridor_span, marker_label, ordered_player_columns_html,
        render_analysis_batch_report_html, replay_entry_title, run_analysis_batch,
        run_analysis_batch_replays, run_analysis_batch_replays_with_options,
        AnalysisBatchProofFrame, AnalysisBatchProofMarker, AnalysisBatchProofMarkerKind,
        AnalysisBatchRunOptions, AnalysisLossCategory, ReplayAnalysisInput,
    };
    use crate::analysis::{
        AnalysisOptions, DefenderReplyAnalysis, DefenderReplyOutcome, DefenderReplyRole,
        DefensePolicy, ProofLimitCause, ProofStatus, ReplyClassification, RootCause, UnclearReason,
    };

    fn replay_from_moves(variant: Variant, moves: &[&str]) -> Replay {
        let rules = RuleConfig {
            variant,
            ..RuleConfig::default()
        };
        let mut board = Board::new(rules.clone());
        let mut replay = Replay::new(rules, "Black", "White");

        for notation in moves {
            let parsed = Move::from_notation(notation).expect("test move notation should parse");
            board
                .apply_move(parsed)
                .expect("test replay move should be legal");
            replay.push_move(parsed, 0, board.hash(), None);
        }
        replay.finish(&board.result, Some(0));
        replay
    }

    fn temp_report_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "gomoku-analysis-batch-{name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("temp report dir should be created");
        dir
    }

    #[test]
    fn analysis_loss_category_uses_inclusive_corridor_span_cutoffs() {
        assert_eq!(
            loss_category_for_corridor_span(4),
            AnalysisLossCategory::Mistake
        );
        assert_eq!(
            loss_category_for_corridor_span(5),
            AnalysisLossCategory::TacticalError
        );
        assert_eq!(
            loss_category_for_corridor_span(8),
            AnalysisLossCategory::TacticalError
        );
        assert_eq!(
            loss_category_for_corridor_span(9),
            AnalysisLossCategory::StrategicLoss
        );
    }

    #[test]
    fn analysis_batch_groups_replay_directory_by_root_cause() {
        let dir = temp_report_dir("root-cause");
        let missed_defense = replay_from_moves(
            Variant::Freestyle,
            &["H8", "G8", "I8", "A1", "J8", "A2", "K8", "B1", "L8"],
        );
        fs::write(
            dir.join("missed_defense.json"),
            missed_defense
                .to_json()
                .expect("test replay should serialize"),
        )
        .expect("test replay should write");

        let report = run_analysis_batch(&dir, AnalysisOptions::default())
            .expect("batch analysis should run");

        assert_eq!(report.total, 1);
        assert_eq!(report.analyzed, 1);
        assert_eq!(report.failed, 0);
        assert_eq!(report.model.max_depth, AnalysisOptions::default().max_depth);
        assert_eq!(report.summary.mistake, 1);
        assert_eq!(report.summary.missed_defense, 1);
        assert_eq!(
            report.entries[0].loss_category,
            Some(AnalysisLossCategory::Mistake)
        );
        assert_eq!(report.entries[0].root_cause, Some(RootCause::MissedDefense));
        assert_eq!(report.entries[0].path, "missed_defense.json");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn analysis_batch_report_renders_standalone_html() {
        let dir = temp_report_dir("html");
        let replay = replay_from_moves(
            Variant::Freestyle,
            &["H8", "G8", "I8", "A1", "J8", "A2", "K8", "B1", "L8"],
        );
        fs::write(
            dir.join("replay.json"),
            replay.to_json().expect("test replay should serialize"),
        )
        .expect("test replay should write");

        let report = run_analysis_batch(&dir, AnalysisOptions::default())
            .expect("batch analysis should run");
        let html = render_analysis_batch_report_html(&report);

        assert!(html.contains("<title>Gomoku2D Analysis Batch Report</title>"));
        assert!(html.contains("<nav class=\"top-links\" aria-label=\"Project links\">"));
        assert!(html.contains("<a href=\"/bot-report/\">Bots</a>"));
        assert!(html.contains("--bg: #1e1e1e"));
        assert!(!html.contains("<span>Total</span>"));
        assert!(html.contains("<span>Analyzed</span>"));
        assert!(html.contains("class=\"run-strip\" aria-label=\"Run summary\""));
        assert!(html.contains("class=\"run-group\" aria-label=\"Analysis setup\""));
        assert!(html.contains("class=\"run-group\" aria-label=\"Run stats\""));
        assert!(html.contains("class=\"run-group\" aria-label=\"Analysis provenance\""));
        assert!(html.contains("class=\"analysis-list\" aria-label=\"Replay analysis entries\""));
        assert!(html.contains("class=\"analysis-entry analysis-entry--mistake\""));
        assert!(html.contains("class=\"loss-chip loss-chip--mistake\""));
        assert!(html.contains("Mistake"));
        assert!(html.contains("<span>Model</span><strong>Corridor search</strong>"));
        assert!(html.contains("<span>Source</span>"));
        assert!(!html.contains("<span>Replays</span>"));
        assert!(!html.contains("Forced-corridor audit"));
        assert!(!html.contains("class=\"guide\""));
        assert!(html.contains("missed defense"));
        assert!(html.contains("<span>Cause</span><strong>missed defense</strong>"));
        assert!(!html.contains("Root detail"));
        assert!(html.contains("<span class=\"entry-match\">replay</span>"));
        assert!(html.contains("<span>Total ply</span><strong>9 ply</strong>"));
        assert!(!html.contains("<span>Time</span>"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn analysis_batch_report_splits_matchup_entry_titles() {
        let title = replay_entry_title(
            "match_1729__search-d5_tactical-cap-8_pattern-eval__vs__search-d7_tactical-cap-8_pattern-eval",
        );

        assert_eq!(title.match_label, "#1729");
        assert_eq!(
            title.black.as_deref(),
            Some("search-d5+tactical-cap-8+pattern-eval")
        );
        assert_eq!(
            title.white.as_deref(),
            Some("search-d7+tactical-cap-8+pattern-eval")
        );
    }

    #[test]
    fn analysis_batch_report_orders_players_by_result() {
        let title = replay_entry_title(
            "match_1731__search-d5_tactical-cap-8_pattern-eval__vs__search-d7_tactical-cap-8_pattern-eval",
        );

        let (winner, loser) = ordered_player_columns_html(&title, Some(Color::White));
        assert!(winner.contains("White"));
        assert!(winner.contains("player-result--win"));
        assert!(winner.contains("SearchBot_D7"));
        assert!(loser.contains("Black"));
        assert!(loser.contains("player-result--lose"));
        assert!(loser.contains("SearchBot_D5"));

        let (draw_black, draw_white) = ordered_player_columns_html(&title, None);
        assert!(draw_black.contains("Black"));
        assert!(draw_white.contains("White"));
    }

    #[test]
    fn analysis_batch_replays_preserves_input_order_and_records_work_metrics() {
        let first = replay_from_moves(
            Variant::Freestyle,
            &["H8", "G8", "I8", "A1", "J8", "A2", "K8", "B1", "L8"],
        );
        let second = replay_from_moves(
            Variant::Freestyle,
            &["H8", "A1", "I8", "A2", "J8", "A3", "K8", "A4", "L8"],
        );

        let report = run_analysis_batch_replays(
            "report.json:bot-a vs bot-b".to_string(),
            vec![
                ReplayAnalysisInput {
                    label: "match_0002".to_string(),
                    replay: second,
                },
                ReplayAnalysisInput {
                    label: "match_0001".to_string(),
                    replay: first,
                },
            ],
            AnalysisOptions::default(),
        );

        assert_eq!(report.source_kind, "report_replays");
        assert_eq!(report.source, "report.json:bot-a vs bot-b");
        assert_eq!(report.entries[0].path, "match_0002");
        assert_eq!(report.entries[1].path, "match_0001");
        assert!(report.entries[0].proof_details.is_none());
        assert!(report.entries[0].final_forced_interval_found);
        assert!(
            report.entries[0].unclear_reason.is_some()
                || report.entries[0].root_cause != Some(RootCause::Unclear)
        );
        assert_eq!(
            report.entries[0].prefixes_analyzed,
            report.entries[0].forced_prefix_count
                + report.entries[0].unknown_prefix_count
                + report.entries[0].escape_prefix_count
        );
        assert_eq!(
            report.total_elapsed_ms,
            report
                .entries
                .iter()
                .map(|entry| entry.elapsed_ms)
                .sum::<u64>()
        );
    }

    #[test]
    fn analysis_batch_replays_records_scan_window_drilldown_context() {
        let replay = replay_from_moves(
            Variant::Freestyle,
            &["H8", "A1", "I8", "A2", "J8", "A3", "K8", "B1", "L8"],
        );

        let report = run_analysis_batch_replays(
            "report.json:bot-a vs bot-b".to_string(),
            vec![ReplayAnalysisInput {
                label: "proof_limit_case".to_string(),
                replay,
            }],
            AnalysisOptions {
                max_backward_window: Some(0),
                ..AnalysisOptions::default()
            },
        );

        let entry = &report.entries[0];
        let context = entry
            .unclear_context
            .as_ref()
            .expect("scan-window-limited entries should expose drilldown context");

        assert_eq!(entry.unclear_reason, Some(UnclearReason::ScanWindowCutoff));
        assert_eq!(context.previous_prefix_ply, Some(7));
        assert_eq!(context.previous_proof_status, None);
        assert_eq!(context.previous_proof_limit_hit, None);
        assert!(context
            .previous_limit_causes
            .contains(&ProofLimitCause::OutsideScanWindow));
        assert!(entry
            .limit_causes
            .contains(&ProofLimitCause::OutsideScanWindow));
        assert!(report
            .limit_cause_counts
            .iter()
            .any(|count| count.cause == ProofLimitCause::OutsideScanWindow && count.count == 1));
        assert_eq!(context.move_count, 9);
        assert!(!context.principal_line.is_empty());
        assert!(!context.principal_line_notation.is_empty());
        assert!(context
            .snapshots
            .iter()
            .any(|snapshot| snapshot.label == "previous_prefix" && snapshot.ply == 7));

        let html = render_analysis_batch_report_html(&report);
        assert!(html.contains("<details"));
        assert!(html.contains("previous_prefix @ ply 7"));
        assert!(html.contains("outside scan window"));
    }

    #[test]
    fn analysis_batch_replays_can_include_decisive_proof_details() {
        let replay = replay_from_moves(
            Variant::Freestyle,
            &["H8", "G8", "I8", "A1", "J8", "A2", "K8", "B1", "L8"],
        );

        let report = run_analysis_batch_replays_with_options(
            "report.json:bot-a vs bot-b".to_string(),
            vec![ReplayAnalysisInput {
                label: "missed_defense".to_string(),
                replay,
            }],
            AnalysisBatchRunOptions {
                analysis: AnalysisOptions::default(),
                include_proof_details: true,
                deep_retry_depth: None,
                deep_retry_limit: 1,
            },
        );

        let entry = &report.entries[0];
        assert_eq!(entry.root_cause, Some(RootCause::MissedDefense));
        let details = entry
            .proof_details
            .as_ref()
            .expect("opt-in proof details should be recorded for decisive entries");

        assert_eq!(details.previous_prefix_ply, Some(7));
        assert_eq!(details.final_forced_start_ply, 8);

        let previous = details
            .previous_proof
            .as_ref()
            .expect("previous prefix proof should be available");
        assert_eq!(previous.prefix_ply, 7);
        assert_eq!(previous.status, ProofStatus::EscapeFound);
        assert_eq!(
            previous.reply_classification,
            Some(ReplyClassification::Escaped)
        );
        assert_eq!(
            previous.escape_replies,
            vec![Move::from_notation("L8").unwrap()]
        );
        assert_eq!(
            previous.winning_squares,
            vec![Move::from_notation("L8").unwrap()]
        );

        let final_start = details
            .final_start_proof
            .as_ref()
            .expect("final forced start proof should be available");
        assert_eq!(final_start.prefix_ply, 8);
        assert_eq!(final_start.status, ProofStatus::ForcedWin);
        assert_eq!(
            final_start.principal_line,
            vec![Move::from_notation("L8").unwrap()]
        );
        assert_eq!(final_start.principal_line_notation, vec!["L8".to_string()]);

        assert!(details
            .snapshots
            .iter()
            .any(|snapshot| snapshot.label == "escape_boundary" && snapshot.ply == 7));
        assert!(details
            .snapshots
            .iter()
            .any(|snapshot| snapshot.label == "forced_entry" && snapshot.ply == 8));

        assert_eq!(
            details
                .proof_frames
                .iter()
                .map(|frame| (frame.label.as_str(), frame.ply))
                .collect::<Vec<_>>(),
            vec![("winning_ply", 9), ("actual_ply_8", 8), ("actual_ply_7", 7)]
        );
        assert!(details
            .proof_frames
            .iter()
            .all(
                |frame| frame.markers.iter().all(|marker| marker.kinds.iter().all(
                    |kind| matches!(
                        kind,
                        AnalysisBatchProofMarkerKind::Winning
                            | AnalysisBatchProofMarkerKind::Threat
                            | AnalysisBatchProofMarkerKind::ImminentDefense
                            | AnalysisBatchProofMarkerKind::OffensiveCounter
                            | AnalysisBatchProofMarkerKind::Forbidden
                            | AnalysisBatchProofMarkerKind::ForcedLoss
                            | AnalysisBatchProofMarkerKind::Escape
                            | AnalysisBatchProofMarkerKind::UnprovedEscape
                            | AnalysisBatchProofMarkerKind::ImmediateLoss
                            | AnalysisBatchProofMarkerKind::UnknownOutcome
                            | AnalysisBatchProofMarkerKind::Actual
                    )
                ))
            ));
        let winning_frame = details
            .proof_frames
            .first()
            .expect("winning-ply frame should be first");
        assert_eq!(winning_frame.side_to_move, Color::Black);
        let actual_l8 = winning_frame
            .markers
            .iter()
            .find(|marker| marker.notation == "L8")
            .expect("winning frame should mark the actual winning move");
        assert!(actual_l8
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::Actual));
        assert_eq!(actual_l8.kinds, vec![AnalysisBatchProofMarkerKind::Actual]);
        let actual_l8_classes = cell_classes(winning_frame, '.', Some(actual_l8));
        assert!(actual_l8_classes.contains("marker--actual"));
        assert!(!actual_l8_classes.contains("marker--winning"));

        let attacker_frame = details
            .proof_frames
            .iter()
            .find(|frame| frame.label == "actual_ply_7" && frame.ply == 7)
            .expect("winner-side setup frame should be recorded");
        assert_eq!(attacker_frame.side_to_move, Color::Black);
        assert!(attacker_frame.markers.iter().all(|marker| {
            marker
                .kinds
                .iter()
                .all(|kind| matches!(kind, AnalysisBatchProofMarkerKind::Actual))
        }));

        let final_frame = details
            .proof_frames
            .iter()
            .find(|frame| frame.label == "actual_ply_8" && frame.ply == 8)
            .expect("forced-interval decision frame should be recorded");
        assert_eq!(final_frame.side_to_move, Color::White);
        assert_eq!(final_frame.move_played_notation.as_deref(), Some("B1"));
        let final_actual = final_frame
            .markers
            .iter()
            .find(|marker| marker.notation == "B1")
            .expect("final frame should mark the actual replay move");
        assert!(final_actual
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::Actual));
        assert_eq!(
            final_actual.kinds,
            vec![AnalysisBatchProofMarkerKind::Actual]
        );
        let final_l8 = final_frame
            .markers
            .iter()
            .find(|marker| marker.notation == "L8")
            .expect("final frame should mark the L8 losing square");
        assert!(final_l8
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::Threat));
    }

    #[test]
    fn analysis_batch_report_renders_opt_in_proof_details() {
        let replay = replay_from_moves(
            Variant::Freestyle,
            &["H8", "G8", "I8", "A1", "J8", "A2", "K8", "B1", "L8"],
        );

        let report = run_analysis_batch_replays_with_options(
            "report.json:bot-a vs bot-b".to_string(),
            vec![ReplayAnalysisInput {
                label: "missed_defense".to_string(),
                replay,
            }],
            AnalysisBatchRunOptions {
                analysis: AnalysisOptions::default(),
                include_proof_details: true,
                deep_retry_depth: None,
                deep_retry_limit: 1,
            },
        );

        let html = render_analysis_batch_report_html(&report);
        assert!(html.contains("Proof details"));
        assert!(html.contains("Escape boundary"));
        assert!(html.contains("Forced run entry"));
        assert!(html.contains("Proof status"));
        assert!(html.contains("Proof frames"));
        assert!(html.contains("Final ply 9"));
        assert!(html.contains("Winner move"));
        assert!(html.contains("9: L8"));
        assert!(html.contains("Turn 7-8"));
        assert!(html.contains("Loser reply"));
        assert!(html.contains("7: K8"));
        assert!(html.contains("8: B1"));
        assert!(!html.contains("root transition"));
        assert!(!html.contains("ASCII board snapshots"));
        assert!(!html.contains("Visual decision frames"));
        assert!(!html.contains("Aggregate proof evidence"));
        assert!(html.contains("class=\"proof-board\""));
        assert!(html.contains("class=\"proof-stone proof-stone--black\""));
        assert!(html.contains("--proof-cell-size: 20px"));
        assert!(html.contains("--proof-grid-span: 281px"));
        assert!(html.contains("padding: 0"));
        assert!(html.contains(".proof-board::before"));
        assert!(html.contains("left: calc(var(--proof-cell-size) / 2)"));
        assert!(html.contains("width: var(--proof-grid-span)"));
        assert!(html.contains("grid-template-rows: repeat(15, var(--proof-cell-size))"));
        assert!(html.contains("background: #d7ad63"));
        assert!(html.contains("top: 2px"));
        assert!(html.contains("bottom: 4px"));
        assert!(html.contains(
            "background: radial-gradient(circle at 35% 30%, #4d4f55 0%, #15171b 58%, #050608 100%)"
        ));
        assert!(html.contains(".proof-cell--stone-black.marker--actual .proof-marker"));
        assert!(html.contains(".marker--actual-black .proof-marker"));
        assert!(html.contains(".marker--actual-white .proof-marker"));
        assert!(html.contains("class=\"proof-actual-stone proof-actual-stone--black\""));
        assert!(html.contains("class=\"proof-actual-stone proof-actual-stone--white\""));
        assert!(html.contains("marker--actual-black"));
        assert!(html.contains("marker--actual-white"));
        assert!(html.contains("marker--actual"));
        assert!(html.contains("data-move=\"L8\""));
        assert!(html.contains("Black to move / forced win"));
        assert!(html.contains("legend-winning"));
        assert!(html.contains("marker--threat"));
        assert!(html.contains("marker--imminent-defense"));
        assert!(html.contains("reply-outcomes"));
        assert!(html.contains("<span>Details</span>"));
        assert!(!html.contains("<span>Sample</span>"));
        assert!(html.contains("forced loss"));
        assert!(!html.contains("marker--principal"));
        assert!(!html.contains("marker--cost"));
        assert!(html.contains("legend-threat"));
        assert!(html.contains("legend-imminent"));
        assert!(html.contains("legend-forbidden"));
        assert!(html.contains(">F</strong> forbidden"));
        assert!(html.contains("proof-legend-row"));
        assert!(html.contains("legend-escape"));
        assert!(html.contains("legend-unproved"));
        assert!(!html.contains("legend-won"));
        assert!(!html.contains(">W</strong>"));
        assert!(!html.contains("actual replay move"));
        assert!(!html.contains("legend-principal"));
        assert!(!html.contains("legend-cost"));
        assert!(!html.contains("<span class=\"proof-marker\">1</span>"));
        assert!(!html.contains("<span class=\"proof-marker\">2</span>"));
        assert!(html.contains("escape"));
        assert!(html.contains("L8"));
    }

    #[test]
    fn defender_reply_labels_split_outcome_from_details() {
        let mv = Move::from_notation("I10").unwrap();
        let reply = DefenderReplyAnalysis {
            mv,
            notation: mv.to_notation(),
            roles: vec![
                DefenderReplyRole::ImminentDefense,
                DefenderReplyRole::OffensiveCounter,
            ],
            outcome: DefenderReplyOutcome::UnprovedEscape,
            deep_retry_depth: Some(8),
            principal_line: vec![Move::from_notation("I11").unwrap()],
            principal_line_notation: vec!["I11".to_string()],
            limit_causes: vec![ProofLimitCause::DepthCutoff],
        };

        assert_eq!(defender_reply_outcome_label(&reply), "unproved escape");
        assert_eq!(
            defender_reply_detail_label(&reply),
            "I10 I11; deep 8 tried; depth cutoff"
        );
    }

    #[test]
    fn analysis_batch_proof_marker_labels_are_semantic_not_step_numbers() {
        let mv = Move::from_notation("H8").unwrap();
        let winning = AnalysisBatchProofMarker {
            mv,
            notation: mv.to_notation(),
            kinds: vec![AnalysisBatchProofMarkerKind::Winning],
        };
        assert_eq!(marker_label(&winning), "");

        let actual_threat = AnalysisBatchProofMarker {
            mv,
            notation: mv.to_notation(),
            kinds: vec![
                AnalysisBatchProofMarkerKind::Threat,
                AnalysisBatchProofMarkerKind::Actual,
            ],
        };
        assert_eq!(marker_label(&actual_threat), "");

        let actual_only = AnalysisBatchProofMarker {
            mv,
            notation: mv.to_notation(),
            kinds: vec![AnalysisBatchProofMarkerKind::Actual],
        };
        assert_eq!(marker_label(&actual_only), "");

        let threat = AnalysisBatchProofMarker {
            mv,
            notation: mv.to_notation(),
            kinds: vec![AnalysisBatchProofMarkerKind::Threat],
        };
        assert_eq!(marker_label(&threat), "L");

        let forbidden_threat = AnalysisBatchProofMarker {
            mv,
            notation: mv.to_notation(),
            kinds: vec![
                AnalysisBatchProofMarkerKind::Threat,
                AnalysisBatchProofMarkerKind::Forbidden,
            ],
        };
        assert_eq!(marker_label(&forbidden_threat), "F");

        let unproved_escape = AnalysisBatchProofMarker {
            mv,
            notation: mv.to_notation(),
            kinds: vec![AnalysisBatchProofMarkerKind::UnprovedEscape],
        };
        assert_eq!(marker_label(&unproved_escape), "U");
    }

    #[test]
    fn analysis_batch_visual_frames_mark_defender_reply_roles_and_outcomes() {
        let replay = replay_from_moves(
            Variant::Renju,
            &[
                "H8", "I8", "H7", "I7", "H6", "H5", "I6", "I9", "G6", "J6", "G8", "J5", "G5", "G7",
                "E6", "F6", "H9", "H10", "F7", "D5", "I10",
            ],
        );

        let report = run_analysis_batch_replays_with_options(
            "report.json:bot-a vs bot-b".to_string(),
            vec![ReplayAnalysisInput {
                label: "forced_reply_options".to_string(),
                replay,
            }],
            AnalysisBatchRunOptions {
                analysis: AnalysisOptions {
                    defense_policy: DefensePolicy::AllLegalDefense,
                    max_depth: 4,
                    max_backward_window: Some(8),
                },
                include_proof_details: true,
                deep_retry_depth: None,
                deep_retry_limit: 1,
            },
        );

        let frame = report.entries[0]
            .proof_details
            .as_ref()
            .expect("proof details should be present")
            .proof_frames
            .iter()
            .find(|frame| frame.label == "actual_ply_14")
            .expect("ply 14 decision frame should be present");

        let g4 = marker_for(frame, "G4");
        assert!(g4
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::ImminentDefense));
        assert!(g4.kinds.contains(&AnalysisBatchProofMarkerKind::ForcedLoss));
        assert!(cell_classes(frame, '.', Some(g4)).contains("marker--side-white"));

        let g9 = marker_for(frame, "G9");
        assert!(g9
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::ImminentDefense));
        assert!(g9.kinds.contains(&AnalysisBatchProofMarkerKind::ForcedLoss));
        assert!(cell_classes(frame, '.', Some(g9)).contains("marker--side-white"));

        let g7 = marker_for(frame, "G7");
        assert_eq!(
            g7.kinds,
            vec![
                AnalysisBatchProofMarkerKind::ImminentDefense,
                AnalysisBatchProofMarkerKind::Actual,
            ]
        );
        assert!(!frame
            .reply_outcomes
            .iter()
            .any(|reply| reply.notation == "G7"));

        let i10 = marker_for(frame, "I10");
        assert!(i10
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::OffensiveCounter));
        assert!(i10
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::UnprovedEscape));
        assert!(cell_classes(frame, '.', Some(i10)).contains("marker--side-white"));

        let i11 = marker_for(frame, "I11");
        assert!(i11
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::OffensiveCounter));
        assert!(i11
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::ForcedLoss));
        assert!(cell_classes(frame, '.', Some(i11)).contains("marker--side-white"));

        let html = render_analysis_batch_report_html(&report);
        assert!(
            html.contains(".marker--imminent-defense {\n      --proof-hint-color: var(--pink);")
        );
        assert!(
            html.contains(".marker--offensive-counter {\n      --proof-hint-color: var(--purple);")
        );
        assert!(html.contains("transform: translateY(-1px);"));
        assert!(html.contains("legend-offensive"));
        assert!(html.contains(".marker--side-white .proof-marker"));
    }

    #[test]
    fn analysis_batch_visual_frames_exclude_actual_reply_from_branch_probes() {
        let replay = replay_from_moves(
            Variant::Renju,
            &[
                "H8", "I8", "H6", "G7", "H7", "H9", "J7", "G4", "G5", "I7", "I5", "H5", "I6", "I9",
                "J6", "K6", "J4", "J5", "H4", "K7", "F6", "G6", "I3", "J2", "E7",
            ],
        );

        let report = run_analysis_batch_replays_with_options(
            "report.json:bot-a vs bot-b".to_string(),
            vec![ReplayAnalysisInput {
                label: "actual_unproved_entered_forced_interval".to_string(),
                replay,
            }],
            AnalysisBatchRunOptions {
                analysis: AnalysisOptions {
                    defense_policy: DefensePolicy::AllLegalDefense,
                    max_depth: 4,
                    max_backward_window: Some(8),
                },
                include_proof_details: true,
                deep_retry_depth: None,
                deep_retry_limit: 1,
            },
        );

        let entry = &report.entries[0];
        assert_eq!(entry.final_forced_interval.as_ref().unwrap().start_ply, 18);
        assert_eq!(entry.critical_mistake_ply, Some(18));
        let frame = entry
            .proof_details
            .as_ref()
            .expect("proof details should be present")
            .proof_frames
            .iter()
            .find(|frame| frame.label == "actual_ply_18")
            .expect("ply 18 decision frame should be present");

        assert!(!frame
            .reply_outcomes
            .iter()
            .any(|reply| reply.notation == "J5"));

        let marker = marker_for(frame, "J5");
        assert_eq!(
            marker.kinds,
            vec![
                AnalysisBatchProofMarkerKind::ImminentDefense,
                AnalysisBatchProofMarkerKind::Actual,
            ]
        );
    }

    #[test]
    fn analysis_batch_visual_frames_mark_actual_far_open_three_defense_hint() {
        let replay = replay_from_moves(
            Variant::Renju,
            &[
                "H8", "H9", "J8", "I7", "I8", "G8", "I10", "F7", "K8", "L8", "J9", "L7", "J7",
                "J10", "L9", "I6", "H7", "G6", "M10", "N11", "J6", "J5", "K9", "N9", "K10", "L11",
                "K11", "K7", "K12",
            ],
        );

        let report = run_analysis_batch_replays_with_options(
            "report.json:bot-a vs bot-b".to_string(),
            vec![ReplayAnalysisInput {
                label: "actual_far_open_three_defense".to_string(),
                replay,
            }],
            AnalysisBatchRunOptions {
                analysis: AnalysisOptions {
                    defense_policy: DefensePolicy::AllLegalDefense,
                    max_depth: 4,
                    max_backward_window: Some(8),
                },
                include_proof_details: true,
                deep_retry_depth: None,
                deep_retry_limit: 1,
            },
        );

        let frame = report.entries[0]
            .proof_details
            .as_ref()
            .expect("proof details should be present")
            .proof_frames
            .iter()
            .find(|frame| frame.label == "actual_ply_24")
            .expect("ply 24 decision frame should be present");

        assert!(!frame
            .reply_outcomes
            .iter()
            .any(|reply| reply.notation == "N9"));
        let marker = marker_for(frame, "N9");
        assert_eq!(
            marker.kinds,
            vec![
                AnalysisBatchProofMarkerKind::ImminentDefense,
                AnalysisBatchProofMarkerKind::Actual,
            ]
        );
    }

    #[test]
    fn analysis_batch_visual_frames_mark_renju_forbidden_blocks() {
        let replay = replay_from_moves(
            Variant::Renju,
            &[
                "H8", "H7", "J8", "I9", "I8", "G8", "F9", "F7", "H9", "G7", "I7", "E7", "D7", "G9",
                "G6", "G11", "K8", "G10",
            ],
        );

        let report = run_analysis_batch_replays_with_options(
            "report.json:bot-a vs bot-b".to_string(),
            vec![ReplayAnalysisInput {
                label: "renju_forbidden_block".to_string(),
                replay,
            }],
            AnalysisBatchRunOptions {
                analysis: AnalysisOptions {
                    defense_policy: DefensePolicy::AllLegalDefense,
                    max_depth: 4,
                    max_backward_window: Some(8),
                },
                include_proof_details: true,
                deep_retry_depth: None,
                deep_retry_limit: 0,
            },
        );

        let frames = &report.entries[0]
            .proof_details
            .as_ref()
            .expect("proof details should be present")
            .proof_frames;
        let turn_16_17 = frames
            .iter()
            .find(|frame| frame.label == "actual_ply_17")
            .expect("ply 17 decision frame should be present");
        let g10 = marker_for(turn_16_17, "G10");
        assert!(g10.kinds.contains(&AnalysisBatchProofMarkerKind::Threat));
        assert!(g10.kinds.contains(&AnalysisBatchProofMarkerKind::Forbidden));
        assert_eq!(marker_label(g10), "F");
        assert!(cell_classes(turn_16_17, '.', Some(g10)).contains("marker--forbidden"));

        let turn_14_15 = frames
            .iter()
            .find(|frame| frame.label == "actual_ply_15")
            .expect("ply 15 decision frame should be present");
        let future_g10 = marker_for(turn_14_15, "G10");
        assert!(future_g10
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::ImminentDefense));
        assert!(future_g10
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::Forbidden));
        assert_eq!(marker_label(future_g10), "F");
    }

    #[test]
    fn analysis_batch_model_only_reports_deep_retry_when_proof_details_run() {
        let replay = replay_from_moves(
            Variant::Freestyle,
            &["H8", "G8", "I8", "A1", "J8", "A2", "K8", "B1", "L8"],
        );

        let report = run_analysis_batch_replays_with_options(
            "deep-retry-off".to_string(),
            vec![ReplayAnalysisInput {
                label: "missed_defense".to_string(),
                replay,
            }],
            AnalysisBatchRunOptions {
                analysis: AnalysisOptions::default(),
                include_proof_details: false,
                deep_retry_depth: Some(10),
                deep_retry_limit: 1,
            },
        );

        assert_eq!(report.model.deep_retry_depth, None);
        assert_eq!(report.model.deep_retry_limit, 0);
        assert!(report.entries[0].proof_details.is_none());
    }

    fn marker_for<'a>(
        frame: &'a AnalysisBatchProofFrame,
        notation: &str,
    ) -> &'a AnalysisBatchProofMarker {
        frame
            .markers
            .iter()
            .find(|marker| marker.notation == notation)
            .unwrap_or_else(|| panic!("expected marker {notation}"))
    }
}
