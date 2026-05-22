use std::collections::BTreeMap;
use std::path::Path;
use std::time::Instant;

use gomoku_bot::tactical::{corridor_active_threats, LethalThreatKind, LocalThreatKind};
use gomoku_core::{Board, Color, Move, Replay};
use rayon::prelude::*;
use serde::Serialize;

use crate::analysis::{
    analyze_alternate_defender_reply_options, analyze_replay, defender_reply_roles_for_move,
    replay_frame_annotations_for_analysis, visible_defender_reply_candidates,
    AnalysisBoardSnapshot, AnalysisOptions, DefenderReplyAnalysis, DefenderReplyCandidate,
    DefenderReplyOutcome, DefenderReplyRole, FailureAnalysis, FailureMode, ForcedInterval,
    GameAnalysis, LethalOnset, LethalOnsetMechanism, MissedCandidateOutcome, ProofLimitCause,
    ProofResult, ProofStatus, ReplayFrameAnnotations, ReplayFrameHighlightRole,
    ReplayFrameMarkerRole, ReplyClassification, ReplyPolicy, RootCause, SearchDiagnostics,
    TacticalNote, UnclearContext, UnclearReason, ANALYSIS_SCHEMA_VERSION,
};
use crate::bot_label::compact_bot_label_parts;
use crate::report_board::{render_report_board, report_board_css, ReportBoardMarker};

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
    AnalysisBatchModel {
        reply_policy: options.analysis.reply_policy,
        max_depth: options.analysis.max_depth,
        max_scan_plies: options.analysis.max_scan_plies,
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
        root_cause: None,
        unclear_reason: None,
        final_move: None,
        lethal_onset: None,
        setup_corridor: None,
        final_forced_interval_found: false,
        final_forced_interval: None,
        proof_intervals: Vec::new(),
        last_chance_ply: None,
        critical_loser_ply: None,
        tactical_notes: Vec::new(),
        failure: None,
        principal_line: Vec::new(),
        unknown_gaps: Vec::new(),
        unknown_gap_count: 0,
        unclear_context: None,
        proof_details: None,
        proof_detail_diagnostics: None,
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
    let entries = report
        .entries
        .iter()
        .map(analysis_entry_card_html)
        .collect::<String>();
    let model_label = "Corridor search";
    let model_config = corridor_search_config_label(report);
    let runtime_label = format!(
        "{} wall / {} CPU",
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
    .run-chip span, .detail span, .entry-metric span {{
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
    .analysis-list {{
      background: var(--surface);
      border: 2px solid var(--border);
      display: grid;
      gap: 12px;
      padding: 20px;
    }}
    .analysis-entry {{
      border: 1px solid var(--border);
      background: var(--card);
    }}
    .analysis-entry:hover {{
      border-top-color: var(--teal);
      border-right-color: var(--teal);
      border-bottom-color: var(--teal);
    }}
    .analysis-entry[open] {{
      border-top-color: var(--accent);
      border-right-color: var(--accent);
      border-bottom-color: var(--accent);
    }}
    .analysis-entry summary {{
      cursor: pointer;
      display: grid;
      gap: 10px;
      grid-template-columns: minmax(68px, max-content) minmax(210px, 1fr) minmax(210px, 1fr);
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
    .entry-metrics {{
      border-top: 1px solid var(--border);
      display: grid;
      gap: 8px;
      grid-column: 1 / -1;
      grid-template-columns: repeat(6, minmax(92px, 1fr));
      padding-top: 10px;
    }}
    .entry-metric {{
      font-variant-numeric: tabular-nums;
      min-width: 0;
    }}
    .entry-metric span {{
      white-space: nowrap;
    }}
    .entry-metric strong {{
      color: var(--text);
      display: block;
      overflow-wrap: anywhere;
    }}
    .entry-body {{
      border-top: 1px solid var(--border);
      display: grid;
      gap: 14px;
      padding: 14px;
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
    .proof-frames {{
      display: grid;
      gap: 12px;
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
    .marker--offensive-counter,
    .marker--corridor-entry-black,
    .marker--corridor-entry-white,
    .marker--winning-evidence,
    .marker--threat-evidence,
    .marker--imminent-evidence,
    .marker--offensive-evidence {{
      --proof-hint-color: transparent;
    }}
    .marker--winning::after,
    .marker--threat::after,
    .marker--imminent-defense::after,
    .marker--offensive-counter::after,
    .marker--corridor-entry-black::after,
    .marker--corridor-entry-white::after,
    .marker--winning-evidence::after,
    .marker--threat-evidence::after,
    .marker--imminent-evidence::after,
    .marker--offensive-evidence::after {{
      content: "";
      position: absolute;
      inset: -1px;
      z-index: 3;
      pointer-events: none;
      border: 2px solid var(--proof-hint-color);
      box-sizing: border-box;
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
    .marker--winning-evidence {{
      --proof-hint-color: var(--green);
    }}
    .marker--threat-evidence {{
      --proof-hint-color: var(--red);
    }}
    .marker--imminent-evidence {{
      --proof-hint-color: var(--pink);
    }}
    .marker--offensive-evidence {{
      --proof-hint-color: var(--purple);
    }}
    .marker--corridor-entry-black {{
      --proof-hint-color: #050505;
    }}
    .marker--corridor-entry-white {{
      --proof-hint-color: #fff;
    }}
    .marker--actual {{
    }}
    .marker--actual-black .proof-marker {{
      color: #07090c;
    }}
    .marker--actual-white .proof-marker {{
      color: #f6eed8;
    }}
    .proof-cell--stone-black.marker--actual .proof-marker {{
      color: var(--text);
    }}
    .marker--side-black .proof-marker {{
      color: #07090c;
    }}
    .marker--side-white .proof-marker {{
      color: #f6eed8;
    }}
    .marker--confirmed-escape .proof-marker {{
      color: var(--green);
    }}
    .marker--possible-escape .proof-marker {{
      color: var(--cyan);
    }}
    .marker--forbidden .proof-marker,
    .marker--forced-loss .proof-marker,
    .marker--immediate-loss .proof-marker {{
      color: var(--red);
    }}
    .marker--immediate-loss .proof-marker {{
      font-size: 16px;
      font-weight: 1000;
      transform: translateY(-2px) scaleX(1.16);
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
    .legend-corridor-entry::before {{ color: #fff; }}
    .legend-outcome {{
      display: inline-flex;
      align-items: baseline;
      gap: 5px;
    }}
    .legend-marker {{
      font-weight: 900;
      letter-spacing: 0.02em;
    }}
    .legend-marker--white {{
      color: #f6eed8;
    }}
    .legend-immediate-loss .legend-marker,
    .legend-forced .legend-marker,
    .legend-forbidden .legend-marker {{
      color: var(--red);
    }}
    .legend-confirmed .legend-marker {{
      color: var(--green);
    }}
    .legend-possible .legend-marker {{
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
      .hero, .analysis-list {{
        padding: 16px;
      }}
      .run-chip {{
        justify-content: space-between;
        width: 100%;
      }}
      .analysis-entry summary {{
        grid-template-columns: minmax(0, 1fr);
      }}
      .entry-title, .entry-player {{
        grid-column: 1 / -1;
      }}
      .entry-metrics {{
        grid-template-columns: repeat(2, minmax(0, 1fr));
      }}
      .entry-metric {{
        border-left: 0;
      }}
      .entry-metric strong {{
        display: block;
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
        <div class="run-chip"><span>Total</span><strong>{total}</strong></div>
        <div class="run-chip"><span>Unclear</span><strong>{unclear}</strong></div>
        <div class="run-chip"><span>Errors</span><strong>{failed}</strong></div>
        <div class="run-chip"><span>Runtime</span><strong>{runtime}</strong></div>
      </div>
      <div class="run-group" aria-label="Analysis provenance">
        <div class="run-chip"><span>Source</span><strong>{source}</strong></div>
        <div class="run-chip"><span>Selector</span><strong>{selector}</strong></div>
      </div>
    </div>
  </header>
  <section class="analysis-list" aria-label="Replay analysis entries">{entries}</section>
</main>
</body>
</html>
"#,
        total = report.total,
        unclear = report.summary.unclear,
        failed = report.failed,
        model = html_escape(model_label),
        source = html_escape(&provenance_source_label(report)),
        selector = html_escape(&provenance_selector_label(report)),
        runtime = html_escape(&runtime_label),
        model_config = html_escape(&model_config),
        board_css = report_board_css(),
        entries = entries,
    )
}

fn analysis_entry_card_html(entry: &AnalysisBatchEntry) -> String {
    let lethal = lethal_onset_label(entry.lethal_onset.as_ref());
    let setup_range = forced_interval_range_label(entry.setup_corridor.as_ref());
    let setup_length = forced_interval_length_label(entry.setup_corridor.as_ref());
    let failure = failure_mode_label(entry.failure.as_ref(), entry.lethal_onset.as_ref());
    let critical = failure_critical_ply_label(entry.failure.as_ref());
    let detail_sections = analysis_entry_detail_sections_html(entry);
    let title = replay_entry_title(&entry.path);
    let (first_player, second_player) = ordered_player_columns_html(&title, entry.winner);
    let panels = analysis_entry_panels_html(entry);

    format!(
        r#"<details class="analysis-entry">
  <summary>
    <strong class="entry-title"><span class="entry-match">{match_label}</span></strong>
    {first_player}
    {second_player}
    <span class="entry-metrics" aria-label="Analysis metrics">
      <span class="entry-metric"><span>Failure</span><strong>{failure}</strong></span>
      <span class="entry-metric"><span>Critical ply</span><strong>{critical}</strong></span>
      <span class="entry-metric"><span>Lethal onset</span><strong>{lethal}</strong></span>
      <span class="entry-metric"><span>Setup corridor</span><strong>{setup_range}</strong></span>
      <span class="entry-metric"><span>Corridor len</span><strong>{setup_length}</strong></span>
      <span class="entry-metric"><span>Game len</span><strong>{length}</strong></span>
    </span>
  </summary>
  <div class="entry-body">
    {detail_sections}
    <div class="entry-panels">{panels}</div>
  </div>
</details>"#,
        match_label = html_escape(&title.match_label),
        first_player = first_player,
        second_player = second_player,
        failure = html_escape(&failure),
        critical = html_escape(&critical),
        lethal = html_escape(&lethal),
        setup_range = html_escape(&setup_range),
        setup_length = html_escape(&setup_length),
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

fn analysis_entry_detail_sections_html(entry: &AnalysisBatchEntry) -> String {
    let unclear_reason = unclear_reason_label(entry.unclear_reason);

    let mut details = Vec::new();
    if let Some(failure) = entry.failure.as_ref() {
        details.push(detail_html(
            "Failure step",
            &failure_step_label(failure, entry.lethal_onset.as_ref()),
        ));
        if let Some(candidates) = failure_candidates_label(failure) {
            details.push(detail_html("Missed candidates", &candidates));
        }
    }
    if unclear_reason != "-" {
        details.push(detail_html("Unclear", &unclear_reason));
    }
    if let Some(reply_probes) = proof_detail_reply_probe_count(entry) {
        details.push(detail_html("Reply probes", &reply_probes.to_string()));
    }
    if let Some(search_nodes) = proof_detail_search_node_count(entry) {
        details.push(detail_html("Search nodes", &search_nodes.to_string()));
    }
    details.push(detail_html(
        "Search time",
        &format_duration_ms(entry.elapsed_ms),
    ));

    format!(
        r#"<div class="detail-grid">{details}</div>"#,
        details = details.join("")
    )
}

fn proof_detail_reply_probe_count(entry: &AnalysisBatchEntry) -> Option<usize> {
    entry.proof_details.as_ref().map(|details| {
        details
            .proof_frames
            .iter()
            .map(|frame| frame.reply_outcomes.len())
            .sum::<usize>()
    })
}

fn proof_detail_search_node_count(entry: &AnalysisBatchEntry) -> Option<usize> {
    entry
        .proof_detail_diagnostics
        .as_ref()
        .map(|diagnostics| diagnostics.search_nodes)
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
    let (primary, modifiers) = compact_bot_label_parts(bot, false);
    let modifiers = modifiers
        .map(|modifiers| format!("<span>{}</span>", html_escape(&modifiers)))
        .unwrap_or_default();
    format!(
        "<span class=\"bot-label\"><span>{}</span>{}</span>",
        html_escape(&primary),
        modifiers
    )
}

fn detail_html(label: &str, value: &str) -> String {
    format!(
        "<div class=\"detail\"><span>{}</span><strong>{}</strong></div>",
        html_escape(label),
        html_escape(value)
    )
}

fn corridor_search_config_label(report: &AnalysisBatchReport) -> String {
    let scan_plies = report
        .model
        .max_scan_plies
        .map(|plies| plies.to_string())
        .unwrap_or_else(|| "unbounded".to_string());

    format!(
        "probe depth {} / traceback {}",
        report.model.max_depth, scan_plies
    )
}

fn provenance_source_label(report: &AnalysisBatchReport) -> String {
    if report.source_kind == "report_replays" {
        return report_source_and_selector(&report.source).0;
    }
    report.source.clone()
}

fn provenance_selector_label(report: &AnalysisBatchReport) -> String {
    if report.source_kind == "report_replays" {
        return report_source_and_selector(&report.source).1;
    }
    match report.source_kind.as_str() {
        "replay_dir" => "all replays".to_string(),
        _ => source_kind_label(&report.source_kind),
    }
}

fn report_source_and_selector(source: &str) -> (String, String) {
    let Some((report_path, selector)) = source.split_once(':') else {
        return (source.to_string(), "all report replays".to_string());
    };
    (report_path.trim().to_string(), selector.trim().to_string())
}

fn source_kind_label(source_kind: &str) -> String {
    source_kind.replace('_', " ")
}

fn forced_interval_range_label(interval: Option<&ForcedInterval>) -> String {
    let Some(interval) = interval else {
        return "-".to_string();
    };
    format!("{}-{}", interval.start_ply, interval.end_ply)
}

fn forced_interval_length_label(interval: Option<&ForcedInterval>) -> String {
    let Some(interval) = interval else {
        return "-".to_string();
    };
    ply_count_label(Some(
        interval
            .end_ply
            .saturating_sub(interval.start_ply)
            .saturating_add(1),
    ))
}

fn lethal_onset_label(onset: Option<&LethalOnset>) -> String {
    onset
        .map(|onset| {
            if let Some(shape) = lethal_onset_shape_label(onset) {
                format!("{} · {}", onset.prefix_ply, shape)
            } else {
                onset.prefix_ply.to_string()
            }
        })
        .unwrap_or_else(|| "-".to_string())
}

fn failure_mode_label(failure: Option<&FailureAnalysis>, onset: Option<&LethalOnset>) -> String {
    failure
        .map(|failure| failure_mode_text(failure.mode, onset))
        .unwrap_or_else(|| "-".to_string())
}

fn failure_critical_ply_label(failure: Option<&FailureAnalysis>) -> String {
    failure
        .and_then(|failure| failure.prefix_ply)
        .map(|ply| ply.to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn failure_step_label(failure: &FailureAnalysis, onset: Option<&LethalOnset>) -> String {
    let reason = failure_mode_text(failure.mode, onset);
    failure
        .actual_notation
        .as_deref()
        .map(|actual| format!("{actual}: {reason}"))
        .unwrap_or(reason)
}

fn failure_candidates_label(failure: &FailureAnalysis) -> Option<String> {
    (!failure.missed_candidates.is_empty()).then(|| {
        failure
            .missed_candidates
            .iter()
            .map(|candidate| {
                let outcome = missed_candidate_outcome_label(candidate.outcome);
                format!("{}: {}", candidate.notation, outcome)
            })
            .collect::<Vec<_>>()
            .join(", ")
    })
}

fn failure_mode_text(mode: FailureMode, onset: Option<&LethalOnset>) -> String {
    match mode {
        FailureMode::MissedImmediateWin => "missed win".to_string(),
        FailureMode::MissedImmediateResponse => "missed 4".to_string(),
        FailureMode::MissedImminentResponse => "missed 3".to_string(),
        FailureMode::MissedEscape => "missed escape".to_string(),
        FailureMode::MissedLethalPrevention => missed_lethal_onset_label(onset),
        FailureMode::Unclear => "unclear".to_string(),
    }
}

fn missed_lethal_onset_label(onset: Option<&LethalOnset>) -> String {
    let Some(onset) = onset else {
        return "missed fork".to_string();
    };
    lethal_onset_shape_label(onset)
        .map(|label| format!("missed {label}"))
        .unwrap_or_else(|| "missed fork".to_string())
}

fn lethal_onset_shape_label(onset: &LethalOnset) -> Option<String> {
    let label = onset.shape.label.trim();
    if label.is_empty() {
        return None;
    }
    let forbidden = onset
        .shape
        .mechanisms
        .contains(&LethalOnsetMechanism::ForbiddenCover);
    let multi_route = onset
        .shape
        .mechanisms
        .contains(&LethalOnsetMechanism::MultiRoute);
    if label == "4"
        && onset.kind == LethalThreatKind::TerminalCoverage
        && multi_route
        && onset.terminal_targets.len() >= 2
    {
        return Some("open four".to_string());
    }
    if forbidden && !multi_route {
        Some(format!("forbidden {label}"))
    } else {
        Some(label.to_string())
    }
}

fn missed_candidate_outcome_label(outcome: MissedCandidateOutcome) -> &'static str {
    match outcome {
        MissedCandidateOutcome::ConfirmedEscape => "confirmed escape",
        MissedCandidateOutcome::PossibleEscape => "possible escape",
        MissedCandidateOutcome::PreventsLethalOnset => "prevents onset",
        MissedCandidateOutcome::PreventsCorridorEntry => "prevents corridor",
    }
}

fn ply_count_label(value: Option<usize>) -> String {
    value
        .map(|value| format!("{value} ply"))
        .unwrap_or_else(|| "-".to_string())
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

fn proof_details_from_analysis(
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

fn proof_details_diagnostics(details: &AnalysisBatchProofDetails) -> SearchDiagnostics {
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
            reply_policy: analysis.model.reply_policy,
            max_depth: analysis.model.max_depth,
            max_scan_plies: analysis.model.max_scan_plies,
        },
    )
}

fn defender_reply_candidates_for_frame(
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

fn add_loser_candidate_markers(
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

fn has_immediate_tactical_hint(markers: &[AnalysisBatchProofMarker]) -> bool {
    markers.iter().any(|marker| {
        marker.kinds.iter().any(|kind| {
            matches!(
                kind,
                AnalysisBatchProofMarkerKind::Winning | AnalysisBatchProofMarkerKind::Threat
            )
        })
    })
}

fn add_current_imminent_response_markers(
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

fn add_replay_annotation_markers(
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

fn marker_kind_for_replay_highlight(
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

fn evidence_marker_kind_for_replay_highlight(
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

fn marker_kind_for_replay_marker(role: ReplayFrameMarkerRole) -> AnalysisBatchProofMarkerKind {
    match role {
        ReplayFrameMarkerRole::ConfirmedEscape => AnalysisBatchProofMarkerKind::ConfirmedEscape,
        ReplayFrameMarkerRole::PossibleEscape => AnalysisBatchProofMarkerKind::PossibleEscape,
        ReplayFrameMarkerRole::ForcedLoss => AnalysisBatchProofMarkerKind::ForcedLoss,
        ReplayFrameMarkerRole::ImmediateLoss => AnalysisBatchProofMarkerKind::ImmediateLoss,
        ReplayFrameMarkerRole::Forbidden => AnalysisBatchProofMarkerKind::Forbidden,
        ReplayFrameMarkerRole::Unknown => AnalysisBatchProofMarkerKind::UnknownOutcome,
    }
}

struct ProofFrameInput<'a> {
    label: &'a str,
    ply: usize,
    board: &'a Board,
    status: ProofStatus,
    move_played: Option<Move>,
    lethal_onset_reached: bool,
    markers: Vec<AnalysisBatchProofMarker>,
    reply_outcomes: Vec<DefenderReplyAnalysis>,
}

fn proof_frame(input: ProofFrameInput<'_>) -> AnalysisBatchProofFrame {
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

fn lethal_onset_reached_for_frame(analysis: &GameAnalysis, board_ply: usize) -> bool {
    analysis
        .lethal_onset
        .as_ref()
        .is_some_and(|onset| board_ply >= onset.prefix_ply)
}

fn add_reply_outcome_markers(
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

struct PreCorridorEscapeMarkerInput<'a> {
    replay: &'a Replay,
    analysis: &'a GameAnalysis,
    ply: usize,
    board: &'a Board,
    proof: Option<&'a ProofResult>,
    previous_proof: Option<&'a ProofResult>,
    reply_outcomes: &'a [DefenderReplyAnalysis],
}

fn add_pre_corridor_escape_marker(
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

fn pre_corridor_escape_entry_move(
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

fn corridor_entry_marker_kind(winner: Color) -> AnalysisBatchProofMarkerKind {
    match winner {
        Color::Black => AnalysisBatchProofMarkerKind::CorridorEntryBlack,
        Color::White => AnalysisBatchProofMarkerKind::CorridorEntryWhite,
    }
}

fn has_visible_tactical_hint(markers: &[AnalysisBatchProofMarker]) -> bool {
    markers
        .iter()
        .flat_map(|marker| marker.kinds.iter().copied())
        .any(is_hint_marker_kind)
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
        if let Some(kind) = marker_kind_for_defender_reply_role(role, None) {
            add_marker_kind(markers, [mv], kind);
        }
    }
}

fn marker_kind_for_defender_reply_role(
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

fn marker_kind_for_defender_reply_outcome(
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

fn is_hint_marker_kind(kind: AnalysisBatchProofMarkerKind) -> bool {
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

    if matches!(entry.root_cause, Some(RootCause::Unclear) | None) {
        summary.unclear += 1;
    }
}

fn unclear_reason_label(unclear_reason: Option<UnclearReason>) -> String {
    unclear_reason
        .map(|reason| match reason {
            UnclearReason::PreviousPrefixUnknown => "previous prefix unknown",
            UnclearReason::ScanWindowCutoff => "scan cap cutoff",
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
        _ => "Previous proof: outside scan cap".to_string(),
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

    let frames = proof_frames_html(&details.proof_frames);

    format!("<div class=\"context\">{frames}</div>", frames = frames)
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
        "<div class=\"proof-frames\"><div class=\"proof-legend\"><div class=\"proof-legend-row\"><span class=\"legend-role legend-winning\">immediate win</span><span class=\"legend-role legend-threat\">immediate threat</span><span class=\"legend-role legend-imminent\">imminent threat</span><span class=\"legend-role legend-offensive\">counter threat</span><span class=\"legend-role legend-corridor-entry\">corridor entry</span></div><div class=\"proof-legend-row\"><span class=\"legend-outcome legend-immediate-loss\"><strong class=\"legend-marker legend-marker--white\">!</strong> immediate loss</span><span class=\"legend-outcome legend-forced\"><strong class=\"legend-marker legend-marker--white\">L</strong> forced loss</span><span class=\"legend-outcome legend-forbidden\"><strong class=\"legend-marker\">F</strong> forbidden</span><span class=\"legend-outcome legend-confirmed\"><strong class=\"legend-marker\">E</strong> confirmed escape</span><span class=\"legend-outcome legend-possible\"><strong class=\"legend-marker\">P</strong> possible escape</span><span class=\"legend-outcome legend-unknown\"><strong class=\"legend-marker\">?</strong> unknown</span></div></div><div class=\"proof-frame-list\">{final_card}{turn_cards}</div></div>",
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
            ("Result", format!("{:?} won", frame.side_to_move)),
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
            proof_decision_turn_html(defender_frame, attacker_frame, winner)
        })
        .collect::<String>()
}

fn proof_decision_turn_html(
    defender_frame: &AnalysisBatchProofFrame,
    attacker_frame: Option<&AnalysisBatchProofFrame>,
    winner: Color,
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
    let lines = vec![
        ("Winner move", attacker_move),
        ("Loser reply", defender_move),
        (
            "Decision",
            format!(
                "{:?} to respond / {}",
                defender_frame.side_to_move,
                perspective_proof_status_label(defender_frame, winner)
            ),
        ),
    ];

    proof_frame_row_html(
        defender_frame.ply,
        &title,
        defender_frame,
        extra_actual,
        &lines,
        &replies,
    )
}

fn perspective_proof_status_label(
    frame: &AnalysisBatchProofFrame,
    attacker: Color,
) -> &'static str {
    match frame.status {
        ProofStatus::ForcedWin => {
            if frame.side_to_move == attacker {
                if frame.lethal_onset_reached {
                    "guaranteed win"
                } else {
                    "forced win"
                }
            } else if frame.lethal_onset_reached {
                "guaranteed loss"
            } else {
                "forced loss"
            }
        }
        ProofStatus::EscapeFound => {
            if frame.side_to_move == attacker {
                "win not forced"
            } else {
                "can escape"
            }
        }
        ProofStatus::Unknown => "unknown",
    }
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
            AnalysisBatchProofMarkerKind::WinningEvidence => "marker--winning-evidence",
            AnalysisBatchProofMarkerKind::ThreatEvidence => "marker--threat-evidence",
            AnalysisBatchProofMarkerKind::ImminentEvidence => "marker--imminent-evidence",
            AnalysisBatchProofMarkerKind::OffensiveEvidence => "marker--offensive-evidence",
            AnalysisBatchProofMarkerKind::CorridorEntryBlack => "marker--corridor-entry-black",
            AnalysisBatchProofMarkerKind::CorridorEntryWhite => "marker--corridor-entry-white",
            AnalysisBatchProofMarkerKind::Forbidden => "marker--forbidden",
            AnalysisBatchProofMarkerKind::ForcedLoss => "marker--forced-loss",
            AnalysisBatchProofMarkerKind::ConfirmedEscape => "marker--confirmed-escape",
            AnalysisBatchProofMarkerKind::PossibleEscape => "marker--possible-escape",
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
    if marker
        .kinds
        .contains(&AnalysisBatchProofMarkerKind::ConfirmedEscape)
    {
        return "E".to_string();
    }
    if marker
        .kinds
        .contains(&AnalysisBatchProofMarkerKind::PossibleEscape)
    {
        return "P".to_string();
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
        DefenderReplyOutcome::ConfirmedEscape => "confirmed escape",
        DefenderReplyOutcome::PossibleEscape => "possible escape",
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
        ProofLimitCause::OutsideScanWindow => "outside scan cap",
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

    use gomoku_bot::tactical::LethalThreatKind;
    use gomoku_core::{Board, Color, Move, Replay, RuleConfig, Variant};

    use super::{
        add_actual_marker, add_loser_candidate_markers, add_reply_outcome_markers, cell_classes,
        defender_reply_candidates_for_frame, defender_reply_detail_label,
        defender_reply_outcome_label, defender_reply_outcomes_for_frame, marker_label,
        ordered_player_columns_html, perspective_proof_status_label,
        render_analysis_batch_report_html, replay_entry_title, run_analysis_batch,
        run_analysis_batch_replays, run_analysis_batch_replays_with_options,
        AnalysisBatchProofFrame, AnalysisBatchProofMarker, AnalysisBatchProofMarkerKind,
        AnalysisBatchRunOptions, ReplayAnalysisInput,
    };
    use crate::analysis::{
        analyze_replay, replay_frame_annotations_for_analysis, AnalysisModel, AnalysisOptions,
        DefenderReplyAnalysis, DefenderReplyCandidate, DefenderReplyOutcome, DefenderReplyRole,
        FailureMode, ForcedInterval, GameAnalysis, ProofLimitCause, ProofStatus,
        ReplayFrameHighlightRole, ReplayFrameMarkerRole, ReplyClassification, ReplyPolicy,
        RootCause, SearchDiagnostics, UnclearReason, ANALYSIS_SCHEMA_VERSION,
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

    fn mv(notation: &str) -> Move {
        Move::from_notation(notation).expect("test move notation should parse")
    }

    fn board_from_moves(variant: Variant, moves: &[&str]) -> Board {
        let mut board = Board::new(RuleConfig {
            variant,
            ..RuleConfig::default()
        });
        for notation in moves {
            board
                .apply_move(mv(notation))
                .expect("test board move should be legal");
        }
        board
    }

    fn analysis_for_winner(winner: Color, rule_set: &str, max_depth: usize) -> GameAnalysis {
        GameAnalysis {
            schema_version: ANALYSIS_SCHEMA_VERSION,
            rule_set: rule_set.to_string(),
            winner: Some(winner),
            loser: Some(winner.opponent()),
            final_move: None,
            final_winning_line: Vec::new(),
            model: AnalysisModel {
                reply_policy: ReplyPolicy::CorridorReplies,
                rule_set: rule_set.to_string(),
                max_depth,
                max_scan_plies: Some(64),
            },
            lethal_onset: None,
            setup_corridor: None,
            final_forced_interval_found: false,
            final_forced_interval: ForcedInterval {
                start_ply: 0,
                end_ply: 0,
            },
            proof_intervals: Vec::new(),
            unknown_gaps: Vec::new(),
            unclear_reason: None,
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

    fn reply_candidate(notation: &str, roles: Vec<DefenderReplyRole>) -> DefenderReplyCandidate {
        let mv = mv(notation);
        DefenderReplyCandidate {
            mv,
            notation: mv.to_notation(),
            roles,
        }
    }

    fn reply_analysis(
        notation: &str,
        roles: Vec<DefenderReplyRole>,
        outcome: DefenderReplyOutcome,
    ) -> DefenderReplyAnalysis {
        let mv = mv(notation);
        DefenderReplyAnalysis {
            mv,
            notation: mv.to_notation(),
            roles,
            outcome,
            principal_line: Vec::new(),
            principal_line_notation: Vec::new(),
            limit_causes: Vec::new(),
            diagnostics: SearchDiagnostics::default(),
        }
    }

    fn proof_frame_with_markers(
        side_to_move: Color,
        markers: Vec<AnalysisBatchProofMarker>,
        reply_outcomes: Vec<DefenderReplyAnalysis>,
    ) -> AnalysisBatchProofFrame {
        AnalysisBatchProofFrame {
            label: "test_frame".to_string(),
            ply: 0,
            side_to_move,
            status: ProofStatus::ForcedWin,
            move_played: None,
            move_played_notation: None,
            lethal_onset_reached: false,
            rows: Vec::new(),
            markers,
            reply_outcomes,
        }
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
        assert_eq!(report.summary.unclear, 0);
        assert_eq!(report.entries[0].root_cause, Some(RootCause::MissedDefense));
        assert_eq!(
            report.entries[0]
                .failure
                .as_ref()
                .map(|failure| failure.mode),
            Some(FailureMode::MissedImmediateResponse)
        );
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
        assert!(html.contains("<span>Total</span><strong>1</strong>"));
        assert!(!html.contains("<span>Analyzed</span>"));
        assert!(html.contains("class=\"run-strip\" aria-label=\"Run summary\""));
        assert!(html.contains("class=\"run-group\" aria-label=\"Analysis setup\""));
        assert!(html.contains("class=\"run-group\" aria-label=\"Run stats\""));
        assert!(html.contains("class=\"run-group\" aria-label=\"Analysis provenance\""));
        assert!(html.contains(" CPU</strong>"));
        assert!(!html.contains(" entries</strong>"));
        assert!(!html.contains("<span>Limit hits</span>"));
        assert!(!html.contains("class=\"summary-grid\""));
        assert!(html.contains("class=\"analysis-list\" aria-label=\"Replay analysis entries\""));
        assert!(html.contains("class=\"analysis-entry\""));
        assert!(!html.contains("analysis-entry--missed-defense"));
        assert!(!html.contains("cause-chip"));
        assert!(!html.contains("Missed defense"));
        assert!(!html.contains("class=\"loss-chip"));
        assert!(!html.contains("Tactical error"));
        assert!(!html.contains("Strategic loss"));
        assert!(html.contains("<span>Model</span><strong>Corridor search</strong>"));
        assert!(html.contains(&format!(
            "<span>Source</span><strong>{}</strong>",
            dir.display()
        )));
        assert!(html.contains("<span>Selector</span><strong>all replays</strong>"));
        assert!(!html.contains("<span>Replays</span>"));
        assert!(!html.contains("Forced-corridor audit"));
        assert!(!html.contains("class=\"guide\""));
        assert!(!html.contains("<span>Cause</span>"));
        assert!(html.contains("<span>Search time</span>"));
        assert!(!html.contains("<span>Status</span>"));
        assert!(!html.contains("<span>Notes</span>"));
        assert!(!html.contains("<span>Winning move</span>"));
        assert!(!html.contains("<span>Prefixes</span>"));
        assert!(!html.contains("<span>Unknown gaps</span>"));
        assert!(!html.contains("Root detail"));
        assert!(html.contains("<span class=\"entry-match\">replay</span>"));
        assert!(html.contains("<span>Setup corridor</span><strong>"));
        assert!(html.contains("<span>Corridor len</span><strong>"));
        assert!(html.contains("<span>Game len</span><strong>9 ply</strong>"));
        assert!(html.contains("<span>Failure</span><strong>missed 4</strong>"));
        assert!(html.contains("<span>Critical ply</span><strong>7</strong>"));
        assert!(html.contains("<span>Failure step</span><strong>B1: missed 4</strong>"));
        assert!(html.contains("<span>Missed candidates</span><strong>L8:"));
        assert!(!html.contains("<span>Missed candidates</span><strong>L8: immediate"));
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
        let html = render_analysis_batch_report_html(&report);
        assert!(html.contains("<span>Source</span><strong>report.json</strong>"));
        assert!(html.contains("<span>Selector</span><strong>bot-a vs bot-b</strong>"));
        assert!(html.contains("probe depth 4 / traceback 64"));
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
    fn analysis_batch_replays_records_scan_cap_drilldown_context() {
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
                max_scan_plies: Some(0),
                ..AnalysisOptions::default()
            },
        );

        let entry = &report.entries[0];
        let context = entry
            .unclear_context
            .as_ref()
            .expect("scan-cap-limited entries should expose drilldown context");

        assert_eq!(entry.unclear_reason, Some(UnclearReason::ScanWindowCutoff));
        assert_eq!(context.previous_prefix_ply, Some(8));
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
        assert!(context.principal_line.is_empty());
        assert!(context.principal_line_notation.is_empty());
        assert!(context
            .snapshots
            .iter()
            .any(|snapshot| snapshot.label == "previous_prefix" && snapshot.ply == 8));

        let html = render_analysis_batch_report_html(&report);
        assert!(html.contains("<details"));
        assert!(html.contains("previous_prefix @ ply 8"));
        assert!(html.contains("outside scan cap"));
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
            Some(ReplyClassification::ConfirmedEscape)
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
                            | AnalysisBatchProofMarkerKind::WinningEvidence
                            | AnalysisBatchProofMarkerKind::ThreatEvidence
                            | AnalysisBatchProofMarkerKind::ImminentEvidence
                            | AnalysisBatchProofMarkerKind::OffensiveEvidence
                            | AnalysisBatchProofMarkerKind::CorridorEntryBlack
                            | AnalysisBatchProofMarkerKind::CorridorEntryWhite
                            | AnalysisBatchProofMarkerKind::Forbidden
                            | AnalysisBatchProofMarkerKind::ForcedLoss
                            | AnalysisBatchProofMarkerKind::ConfirmedEscape
                            | AnalysisBatchProofMarkerKind::PossibleEscape
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
    fn analysis_batch_report_uses_perspective_status_after_lethal_onset() {
        let replay = replay_from_moves(
            Variant::Freestyle,
            &[
                "H8", "G8", "I8", "A1", "J8", "O1", "K8", "A15", "I7", "O15", "I9", "L8", "I6",
                "A14", "I10",
            ],
        );

        let report = run_analysis_batch_replays_with_options(
            "report.json:bot-a vs bot-b".to_string(),
            vec![ReplayAnalysisInput {
                label: "lethal_onset".to_string(),
                replay,
            }],
            AnalysisBatchRunOptions {
                analysis: AnalysisOptions::default(),
                include_proof_details: true,
            },
        );

        let entry = &report.entries[0];
        let onset = entry
            .lethal_onset
            .as_ref()
            .expect("analysis entry should carry lethal onset evidence");
        assert_eq!(onset.prefix_ply, 11);
        assert_eq!(onset.kind, LethalThreatKind::OneStepCoverage);
        assert_eq!(onset.shape.label, "4x3");
        let setup_corridor = entry
            .setup_corridor
            .as_ref()
            .expect("analysis entry should carry setup corridor evidence");
        assert_eq!(setup_corridor.end_ply, onset.prefix_ply);

        let details = entry
            .proof_details
            .as_ref()
            .expect("proof details should be recorded");
        let onset_frame = details
            .proof_frames
            .iter()
            .find(|frame| frame.ply == 12)
            .expect("onset frame should be recorded");
        assert_eq!(onset_frame.ply, 12);
        assert!(onset_frame.lethal_onset_reached);
        let l8 = marker_for(onset_frame, "L8");
        assert!(l8.kinds.contains(&AnalysisBatchProofMarkerKind::Threat));
        let i6 = marker_for(onset_frame, "I6");
        assert!(i6.kinds.contains(&AnalysisBatchProofMarkerKind::Threat));
        assert!(i6
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::ImmediateLoss));
        let i10 = marker_for(onset_frame, "I10");
        assert!(i10.kinds.contains(&AnalysisBatchProofMarkerKind::Threat));
        assert!(i10
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::ImmediateLoss));
        assert!(marker_for(onset_frame, "H8")
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::ThreatEvidence));
        assert!(marker_for(onset_frame, "I7")
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::ThreatEvidence));

        let html = render_analysis_batch_report_html(&report);
        assert!(html.contains("<span>Lethal onset</span><strong>11 · 4x3</strong>"));
        assert!(html.contains("<span>Setup corridor</span><strong>"));
        assert!(html.contains("<span>Corridor len</span><strong>"));
        assert!(html.contains("marker--threat-evidence"));
        assert!(!html.contains("<span>Forced corridor</span>"));
        assert!(!html.contains("11 / one-step"));
        assert!(html.contains("White to respond / guaranteed loss"));
        assert!(!html.contains("<span>Lethal</span>"));
        assert!(!html.contains("legend-lethal-onset"));
        assert!(!html.contains("marker--lethal-onset"));
    }

    #[test]
    fn proof_status_copy_uses_side_to_move_perspective() {
        let mut frame = proof_frame_with_markers(Color::White, Vec::new(), Vec::new());
        frame.status = ProofStatus::ForcedWin;
        frame.lethal_onset_reached = false;
        assert_eq!(
            perspective_proof_status_label(&frame, Color::Black),
            "forced loss"
        );

        frame.lethal_onset_reached = true;
        assert_eq!(
            perspective_proof_status_label(&frame, Color::Black),
            "guaranteed loss"
        );

        frame.side_to_move = Color::Black;
        assert_eq!(
            perspective_proof_status_label(&frame, Color::Black),
            "guaranteed win"
        );
    }

    #[test]
    fn analysis_batch_marks_pre_corridor_entry_as_escape_target() {
        let replay = replay_from_moves(
            Variant::Renju,
            &[
                "H8", "H9", "I8", "I9", "G8", "J8", "G6", "H10", "K7", "G11", "F12", "F10", "J9",
                "H7", "E8", "F8", "F7", "H5", "C10", "D9", "G5", "G7", "H6", "E9", "D8", "G9",
                "F9", "E7", "D6", "I11",
            ],
        );

        let report = run_analysis_batch_replays_with_options(
            "report.json:bot-a vs bot-b".to_string(),
            vec![ReplayAnalysisInput {
                label: "pre_corridor_escape".to_string(),
                replay,
            }],
            AnalysisBatchRunOptions {
                analysis: AnalysisOptions {
                    reply_policy: ReplyPolicy::CorridorReplies,
                    max_depth: 4,
                    max_scan_plies: Some(8),
                },
                include_proof_details: true,
            },
        );

        let details = report.entries[0]
            .proof_details
            .as_ref()
            .expect("proof details should be recorded");
        assert_eq!(details.final_forced_start_ply, 23);
        let frame = details
            .proof_frames
            .iter()
            .find(|frame| frame.label == "actual_ply_23")
            .expect("pre-corridor escape frame should be present");
        assert_eq!(frame.side_to_move, Color::Black);
        assert_eq!(frame.status, ProofStatus::EscapeFound);
        assert!(frame.reply_outcomes.is_empty());

        let e9 = frame
            .markers
            .iter()
            .find(|marker| marker.notation == "E9")
            .expect("winner corridor entry should be shown as an escape target");
        assert!(e9
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::CorridorEntryWhite));
        assert!(e9
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::ConfirmedEscape));
        assert_eq!(marker_label(e9), "E");
        let classes = cell_classes(frame, '.', Some(e9));
        assert!(classes.contains("marker--corridor-entry-white"));
        assert!(classes.contains("marker--confirmed-escape"));
    }

    #[test]
    fn analysis_batch_marks_attacker_started_corridor_entry_as_escape_target() {
        let replay = replay_from_moves(
            Variant::Renju,
            &[
                "H8", "H9", "I8", "I9", "G8", "J8", "J9", "K7", "H10", "H7", "G9", "L6", "M5",
                "I7", "G7", "G6", "F8", "E8", "E7", "D6", "I11",
            ],
        );

        let report = run_analysis_batch_replays_with_options(
            "report.json:bot-a vs bot-b".to_string(),
            vec![ReplayAnalysisInput {
                label: "match_1731".to_string(),
                replay,
            }],
            AnalysisBatchRunOptions {
                analysis: AnalysisOptions {
                    reply_policy: ReplyPolicy::CorridorReplies,
                    max_depth: 4,
                    max_scan_plies: Some(64),
                },
                include_proof_details: true,
            },
        );

        let details = report.entries[0]
            .proof_details
            .as_ref()
            .expect("proof details should be recorded");
        assert_eq!(details.final_forced_start_ply, 13);
        let frame = details
            .proof_frames
            .iter()
            .find(|frame| frame.label == "actual_ply_14")
            .expect("defender frame after attacker corridor entry should be present");
        assert_eq!(frame.side_to_move, Color::White);
        assert_eq!(frame.status, ProofStatus::ForcedWin);
        assert!(frame.reply_outcomes.is_empty());

        let g7 = frame
            .markers
            .iter()
            .find(|marker| marker.notation == "G7")
            .expect("winner corridor entry should be shown as an escape target");
        assert!(g7
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::CorridorEntryBlack));
        assert!(g7
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::ConfirmedEscape));
        assert_eq!(marker_label(g7), "E");
        let classes = cell_classes(frame, '.', Some(g7));
        assert!(classes.contains("marker--corridor-entry-black"));
        assert!(classes.contains("marker--confirmed-escape"));
    }

    #[test]
    fn shared_replay_annotations_match_report_corridor_entry_boundary() {
        let replay = replay_from_moves(
            Variant::Renju,
            &[
                "H8", "H9", "I8", "I9", "G8", "J8", "J9", "K7", "H10", "H7", "G9", "L6", "M5",
                "I7", "G7", "G6", "F8", "E8", "E7", "D6", "I11",
            ],
        );
        let options = AnalysisOptions {
            reply_policy: ReplyPolicy::CorridorReplies,
            max_depth: 4,
            max_scan_plies: Some(64),
        };
        let analysis = analyze_replay(&replay, options).expect("analysis should run");
        let annotations = replay_frame_annotations_for_analysis(&replay, &analysis)
            .expect("shared replay annotations should build");

        let boundary = annotations
            .iter()
            .find(|frame| frame.ply == 13)
            .expect("shared annotation should include the report corridor-entry boundary");
        assert!(boundary.highlights.iter().any(|highlight| {
            highlight.role == ReplayFrameHighlightRole::CorridorEntry
                && highlight.notation == "G7"
                && highlight.side == Color::Black
        }));
        assert!(boundary.markers.iter().any(|marker| {
            marker.role == ReplayFrameMarkerRole::ConfirmedEscape
                && marker.notation == "G7"
                && marker.side == Color::White
        }));
    }

    #[test]
    fn analysis_batch_actual_marker_keeps_counter_hint() {
        let board = board_from_moves(
            Variant::Renju,
            &[
                "H8", "G7", "H6", "H7", "I7", "F7", "G5", "F4", "J8", "K9", "I8", "G8", "I6", "I9",
                "H9", "F6", "F5", "G9", "G10",
            ],
        );
        assert_eq!(board.current_player, Color::White);

        let mut markers = Vec::new();
        add_actual_marker(&mut markers, &board, Some(Color::Black), mv("E7"));

        let marker = proof_marker_for(&markers, "E7");
        assert!(marker
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::OffensiveCounter));
        assert!(marker.kinds.contains(&AnalysisBatchProofMarkerKind::Actual));
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
            },
        );

        let json =
            serde_json::to_string_pretty(&report).expect("analysis batch report should serialize");
        assert!(json.contains("\"proof_detail_diagnostics\""));
        assert!(json.contains("\"search_nodes\""));
        assert!(json.contains("\"branch_probes\""));
        assert!(json.contains("\"max_depth_reached\""));

        let html = render_analysis_batch_report_html(&report);
        assert!(html.contains("Proof details"));
        assert!(html.contains("<span>Reply probes</span>"));
        assert!(html.contains("<span>Search nodes</span>"));
        assert!(!html.contains("<span>Branch probes</span>"));
        assert!(!html.contains("Escape boundary"));
        assert!(!html.contains("Forced run entry"));
        assert!(!html.contains("Proof status"));
        assert!(!html.contains("Proof frames"));
        assert!(!html.contains("proof-summary-strip"));
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
        assert!(html.contains("Black won"));
        assert!(html.contains("legend-winning"));
        assert!(html.contains("marker--threat"));
        assert!(html.contains("marker--imminent-defense"));
        assert!(html.contains(".marker--offensive-counter"));
        assert!(html.contains("legend-offensive"));
        assert!(html.contains(".marker--side-white .proof-marker"));
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
        assert!(html.contains("legend-confirmed"));
        assert!(html.contains("legend-possible"));
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
            outcome: DefenderReplyOutcome::PossibleEscape,
            principal_line: vec![Move::from_notation("I11").unwrap()],
            principal_line_notation: vec!["I11".to_string()],
            limit_causes: vec![ProofLimitCause::DepthCutoff],
            diagnostics: SearchDiagnostics::default(),
        };

        assert_eq!(defender_reply_outcome_label(&reply), "possible escape");
        assert_eq!(defender_reply_detail_label(&reply), "I10 I11; depth cutoff");
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

        let immediate_loss = AnalysisBatchProofMarker {
            mv,
            notation: mv.to_notation(),
            kinds: vec![AnalysisBatchProofMarkerKind::ImmediateLoss],
        };
        assert_eq!(marker_label(&immediate_loss), "!");

        let possible_escape = AnalysisBatchProofMarker {
            mv,
            notation: mv.to_notation(),
            kinds: vec![AnalysisBatchProofMarkerKind::PossibleEscape],
        };
        assert_eq!(marker_label(&possible_escape), "P");
    }

    #[test]
    fn analysis_batch_reply_markers_combine_roles_and_outcomes() {
        let board = board_from_moves(Variant::Renju, &["H8"]);
        let mut markers = Vec::new();
        add_loser_candidate_markers(
            &mut markers,
            &board,
            Some(Color::Black),
            &[reply_candidate(
                "G7",
                vec![
                    DefenderReplyRole::ImminentDefense,
                    DefenderReplyRole::Actual,
                ],
            )],
        );
        add_actual_marker(&mut markers, &board, Some(Color::Black), mv("G7"));

        let replies = vec![
            reply_analysis(
                "G4",
                vec![DefenderReplyRole::ImminentDefense],
                DefenderReplyOutcome::ForcedLoss,
            ),
            reply_analysis(
                "G9",
                vec![DefenderReplyRole::ImminentDefense],
                DefenderReplyOutcome::ForcedLoss,
            ),
            reply_analysis(
                "I10",
                vec![DefenderReplyRole::OffensiveCounter],
                DefenderReplyOutcome::PossibleEscape,
            ),
            reply_analysis(
                "I11",
                vec![DefenderReplyRole::OffensiveCounter],
                DefenderReplyOutcome::ForcedLoss,
            ),
        ];
        add_reply_outcome_markers(&mut markers, &replies);
        let frame = proof_frame_with_markers(Color::White, markers, replies);
        let frame = &frame;

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
            .contains(&AnalysisBatchProofMarkerKind::PossibleEscape));
        assert!(cell_classes(frame, '.', Some(i10)).contains("marker--side-white"));

        let i11 = marker_for(frame, "I11");
        assert!(i11
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::OffensiveCounter));
        assert!(i11
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::ForcedLoss));
        assert!(cell_classes(frame, '.', Some(i11)).contains("marker--side-white"));
    }

    #[test]
    fn analysis_batch_candidate_markers_do_not_project_future_forbidden_costs_backwards() {
        let moves = [
            "H8", "I8", "H7", "I7", "H6", "H5", "I6", "G10", "J5", "G8", "G6", "J6", "F6", "E6",
            "G7", "I9", "K4", "L3", "E5", "D4", "H9", "H10", "I5", "J4", "F8", "E9", "F10", "F7",
            "F11", "F12", "G11", "H11", "E11", "I12", "F9", "D12", "I13", "H12", "G12", "K14",
            "J13", "H14", "C11", "H13",
        ];
        let analysis = analysis_for_winner(Color::White, "renju", 0);

        for (ply, actual) in [(39, "G12"), (41, "J13")] {
            let board = board_from_moves(Variant::Renju, &moves[..ply - 1]);
            assert_eq!(board.current_player, Color::Black);
            let candidates =
                defender_reply_candidates_for_frame(&board, &analysis, Some(mv(actual)));
            let mut markers = Vec::new();
            add_loser_candidate_markers(&mut markers, &board, analysis.winner, &candidates);
            add_actual_marker(&mut markers, &board, analysis.winner, mv(actual));
            let frame = proof_frame_with_markers(board.current_player, markers, Vec::new());
            if let Some(h13) = frame.markers.iter().find(|marker| marker.notation == "H13") {
                assert!(
                    !h13.kinds.contains(&AnalysisBatchProofMarkerKind::ImminentDefense)
                        && !h13.kinds.contains(&AnalysisBatchProofMarkerKind::Forbidden),
                    "ply {ply} must not mark future H13 proof evidence as a current forbidden/imminent reply: {:?}",
                    h13.kinds
                );
            }
        }
    }

    #[test]
    fn analysis_batch_visual_frames_filter_forbidden_costs_to_current_prefix() {
        let replay = replay_from_moves(
            Variant::Renju,
            &[
                "H8", "G7", "H9", "J8", "H7", "H6", "I7", "F8", "E9", "H10", "I5", "G9", "E7",
                "I9", "K7", "G10", "G11", "I11", "J12", "G6", "G8", "F6", "L7", "J7", "J6", "E6",
                "D6", "I6",
            ],
        );

        let report = run_analysis_batch_replays_with_options(
            "report.json:bot-a vs bot-b".to_string(),
            vec![ReplayAnalysisInput {
                label: "match_1584".to_string(),
                replay,
            }],
            AnalysisBatchRunOptions {
                analysis: AnalysisOptions {
                    reply_policy: ReplyPolicy::CorridorReplies,
                    max_depth: 4,
                    max_scan_plies: Some(64),
                },
                include_proof_details: true,
            },
        );

        let frames = &report.entries[0]
            .proof_details
            .as_ref()
            .expect("proof details should be present")
            .proof_frames;
        for label in ["actual_ply_24", "actual_ply_26"] {
            let frame = frames
                .iter()
                .find(|frame| frame.label == label)
                .unwrap_or_else(|| panic!("{label} frame should be present"));
            if let Some(i6) = frame.markers.iter().find(|marker| marker.notation == "I6") {
                assert!(
                    !i6.kinds.contains(&AnalysisBatchProofMarkerKind::Forbidden),
                    "{label} must not mark I6 using future forbidden-cost evidence: {:?}",
                    i6.kinds
                );
            }
        }
    }

    #[test]
    fn analysis_batch_actual_marker_keeps_imminent_hint() {
        let board = board_from_moves(
            Variant::Renju,
            &[
                "H8", "I8", "H6", "G7", "H7", "H9", "J7", "G4", "G5", "I7", "I5", "H5", "I6", "I9",
                "J6", "K6", "J4",
            ],
        );
        let actual = mv("J5");
        let analysis = analysis_for_winner(Color::Black, "renju", 0);
        let candidates = defender_reply_candidates_for_frame(&board, &analysis, Some(actual));
        let mut markers = Vec::new();
        add_loser_candidate_markers(&mut markers, &board, analysis.winner, &candidates);
        add_actual_marker(&mut markers, &board, analysis.winner, actual);

        let marker = proof_marker_for(&markers, "J5");
        assert_eq!(
            marker.kinds,
            vec![
                AnalysisBatchProofMarkerKind::ImminentDefense,
                AnalysisBatchProofMarkerKind::Actual,
            ]
        );
    }

    #[test]
    fn analysis_batch_actual_marker_keeps_far_open_three_defense_hint() {
        let board = board_from_moves(
            Variant::Renju,
            &[
                "H8", "H9", "J8", "I7", "I8", "G8", "I10", "F7", "K8", "L8", "J9", "L7", "J7",
                "J10", "L9", "I6", "H7", "G6", "M10", "N11", "J6", "J5", "K9",
            ],
        );
        let actual = mv("N9");
        let mut markers = Vec::new();
        add_actual_marker(&mut markers, &board, Some(Color::Black), actual);

        let marker = proof_marker_for(&markers, "N9");
        assert_eq!(
            marker.kinds,
            vec![
                AnalysisBatchProofMarkerKind::ImminentDefense,
                AnalysisBatchProofMarkerKind::Actual,
            ]
        );
    }

    #[test]
    fn analysis_batch_visual_frames_mark_both_forbidden_open_three_responses() {
        let replay = replay_from_moves(
            Variant::Renju,
            &[
                "H8", "G7", "H9", "J8", "H7", "H6", "I7", "F8", "E9", "H10", "I5", "G9", "E7",
                "I9", "K7", "G10", "G11", "I11", "J12", "G6", "G8", "F6", "L7", "J7", "J6", "E6",
                "D6", "I6",
            ],
        );

        let report = run_analysis_batch_replays_with_options(
            "report.json:bot-a vs bot-b".to_string(),
            vec![ReplayAnalysisInput {
                label: "renju_forbidden_open_three_responses".to_string(),
                replay,
            }],
            AnalysisBatchRunOptions {
                analysis: AnalysisOptions {
                    reply_policy: ReplyPolicy::CorridorReplies,
                    max_depth: 4,
                    max_scan_plies: Some(64),
                },
                include_proof_details: true,
            },
        );

        let frame = report.entries[0]
            .proof_details
            .as_ref()
            .expect("proof details should be present")
            .proof_frames
            .iter()
            .find(|frame| frame.label == "actual_ply_25")
            .expect("ply 25 decision frame should be present");

        for notation in ["E6", "I6"] {
            let marker = marker_for(frame, notation);
            assert!(
                marker
                    .kinds
                    .contains(&AnalysisBatchProofMarkerKind::ImminentDefense),
                "{notation} should be marked as an imminent open-three response: {:?}",
                marker.kinds
            );
            assert!(
                marker
                    .kinds
                    .contains(&AnalysisBatchProofMarkerKind::Forbidden),
                "{notation} should be marked forbidden for Black under Renju: {:?}",
                marker.kinds
            );
        }
    }

    #[test]
    fn analysis_batch_candidate_markers_prioritize_immediate_threats_over_imminent_responses() {
        let board = board_from_moves(
            Variant::Renju,
            &[
                "H8", "I8", "H7", "I7", "H6", "H5", "I6", "I9", "G6", "J6", "G8", "J5", "E6", "F6",
                "H9",
            ],
        );
        let actual = mv("H10");
        let analysis = analysis_for_winner(Color::Black, "renju", 0);
        let candidates = defender_reply_candidates_for_frame(&board, &analysis, Some(actual));
        let mut markers = Vec::new();
        add_loser_candidate_markers(&mut markers, &board, analysis.winner, &candidates);
        add_actual_marker(&mut markers, &board, analysis.winner, actual);

        let h10 = proof_marker_for(&markers, "H10");
        assert!(h10.kinds.contains(&AnalysisBatchProofMarkerKind::Threat));
        assert!(h10.kinds.contains(&AnalysisBatchProofMarkerKind::Actual));
        assert!(
            !markers.iter().any(|marker| marker
                .kinds
                .contains(&AnalysisBatchProofMarkerKind::ImminentDefense)),
            "imminent responses should be suppressed while an immediate threat response exists: {:?}",
            markers
        );
    }

    #[test]
    fn analysis_batch_visual_frames_probe_all_imminent_combo_replies() {
        let board = board_from_moves(
            Variant::Renju,
            &[
                "H8", "H7", "F8", "G9", "G8", "I8", "G6", "D9", "F9", "F10", "D7", "G10", "F7",
                "E10", "E8", "D8", "C6", "B5", "D10", "F11", "F6", "F5", "D6", "E6", "H5", "I4",
            ],
        );
        assert_eq!(board.current_player, Color::Black);

        let analysis = analysis_for_winner(Color::White, "renju", 0);

        let reply_outcomes = defender_reply_outcomes_for_frame(&board, &analysis, Some(mv("C8")));
        for notation in ["J7", "H9", "E12", "G12"] {
            assert!(
                reply_outcomes
                    .iter()
                    .any(|reply| reply.notation == notation),
                "{notation} should be probed as a non-actual 3+3 reply: {:?}",
                reply_outcomes
            );
        }
        assert!(
            !reply_outcomes.iter().any(|reply| reply.notation == "C8"),
            "the actual replay move is inherited from replay context, not re-probed: {:?}",
            reply_outcomes
        );

        let mut markers = Vec::new();
        add_reply_outcome_markers(&mut markers, &reply_outcomes);
        for notation in ["J7", "H9", "E12"] {
            let marker = proof_marker_for(&markers, notation);
            assert!(
                marker
                    .kinds
                    .contains(&AnalysisBatchProofMarkerKind::ImminentDefense),
                "{notation} should keep its imminent-response hint box: {:?}",
                marker.kinds
            );
            assert!(
                marker.kinds.iter().any(|kind| matches!(
                    kind,
                    AnalysisBatchProofMarkerKind::ForcedLoss
                        | AnalysisBatchProofMarkerKind::ConfirmedEscape
                        | AnalysisBatchProofMarkerKind::PossibleEscape
                        | AnalysisBatchProofMarkerKind::ImmediateLoss
                        | AnalysisBatchProofMarkerKind::UnknownOutcome
                )),
                "{notation} should carry a proof outcome marker: {:?}",
                marker.kinds
            );
        }
        assert!(
            markers.iter().all(|marker| marker.notation != "C8"),
            "actual replay move should not be re-probed: {markers:?}"
        );
    }

    #[test]
    fn analysis_batch_visual_frames_do_not_show_lower_tier_forbidden_replies_during_immediate_threats(
    ) {
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
        let analysis = analysis_for_winner(Color::White, "renju", 0);
        let reply_outcomes = defender_reply_outcomes_for_frame(&board, &analysis, Some(mv("F8")));
        assert!(
            reply_outcomes
                .iter()
                .any(|reply| reply.notation == "F13"
                    && reply.roles.contains(&DefenderReplyRole::ImmediateDefense)),
            "the active proof candidate should be the immediate-threat response: {reply_outcomes:?}"
        );
        assert!(
            reply_outcomes.iter().all(|reply| reply.notation != "D8"),
            "lower-tier forbidden imminent replies should not be probed: {reply_outcomes:?}"
        );

        let reply_candidates =
            defender_reply_candidates_for_frame(&board, &analysis, Some(mv("F8")));
        let mut markers = Vec::new();
        add_loser_candidate_markers(&mut markers, &board, analysis.winner, &reply_candidates);
        add_reply_outcome_markers(&mut markers, &reply_outcomes);

        assert!(
            markers.iter().all(|marker| marker.notation != "D8"),
            "lower-tier forbidden imminent replies should not be marked while immediate threats are active: {markers:?}"
        );
        let f13 = proof_marker_for(&markers, "F13");
        assert!(f13.kinds.contains(&AnalysisBatchProofMarkerKind::Threat));
        assert!(f13
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::ImmediateLoss));
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
                    reply_policy: ReplyPolicy::CorridorReplies,
                    max_depth: 4,
                    max_scan_plies: Some(8),
                },
                include_proof_details: true,
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

    fn proof_marker_for<'a>(
        markers: &'a [AnalysisBatchProofMarker],
        notation: &str,
    ) -> &'a AnalysisBatchProofMarker {
        markers
            .iter()
            .find(|marker| marker.notation == notation)
            .unwrap_or_else(|| panic!("expected marker {notation}"))
    }
}
