use std::collections::BTreeMap;
use std::path::Path;
use std::time::Instant;

use gomoku_core::{Board, Color, Move, Replay};
use rayon::prelude::*;
use serde::Serialize;

use crate::analysis::{
    analyze_defender_reply_options_with_retry, analyze_replay, AnalysisBoardSnapshot,
    AnalysisOptions, DefenderReplyAnalysis, DefenderReplyOutcome, DefenderReplyRole, DefensePolicy,
    ForcedInterval, GameAnalysis, ProofLimitCause, ProofResult, ProofStatus, ReplyClassification,
    RootCause, TacticalNote, UnclearContext, UnclearReason, ANALYSIS_SCHEMA_VERSION,
};
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
    pub defense_policy: DefensePolicy,
    pub max_depth: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deep_retry_depth: Option<usize>,
    pub deep_retry_limit: usize,
    pub max_backward_window: Option<usize>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct AnalysisBatchSummary {
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
    let rows = report
        .entries
        .iter()
        .map(|entry| {
            format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{} ms</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                html_escape(&entry.path),
                html_escape(entry_status_label(entry.status)),
                html_escape(&option_debug(entry.winner)),
                html_escape(&root_cause_label(entry.root_cause)),
                html_escape(&unclear_reason_label(entry.unclear_reason)),
                html_escape(&interval_label(entry.final_forced_interval.as_ref())),
                html_escape(&entry.unknown_gap_count.to_string()),
                entry.elapsed_ms,
                unclear_context_html(entry.unclear_context.as_ref()),
                proof_details_html(entry.proof_details.as_ref()),
                html_escape(entry.error.as_deref().unwrap_or("-")),
            )
        })
        .collect::<String>();

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
      --bg: #15191e;
      --panel: #202731;
      --line: #394452;
      --text: #f5f0dc;
      --muted: #9aa6b2;
      --accent: #f0c75e;
      --green: #58d68d;
      --orange: #f08c4e;
      --cyan: #4ecdc4;
      --red: #ff5d5d;
      --pink: #ff7ab6;
      --blue: #58a6ff;
    }}
    body {{
      margin: 0;
      background: radial-gradient(circle at top left, #29323d 0, var(--bg) 42rem);
      color: var(--text);
      font: 14px/1.5 ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
    }}
    main {{
      max-width: 1180px;
      margin: 0 auto;
      padding: 32px 20px 48px;
    }}
    h1 {{
      margin: 0 0 16px;
      font-size: clamp(24px, 4vw, 40px);
      letter-spacing: 0.03em;
    }}
    .summary {{
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
      gap: 12px;
      margin: 18px 0 24px;
    }}
    .card {{
      border: 1px solid var(--line);
      background: color-mix(in srgb, var(--panel) 88%, transparent);
      padding: 12px;
    }}
    .card span {{
      display: block;
      color: var(--muted);
      font-size: 11px;
      letter-spacing: 0.12em;
      text-transform: uppercase;
    }}
    .card strong {{
      display: block;
      margin-top: 4px;
      color: var(--accent);
      font-size: 20px;
    }}
    table {{
      width: 100%;
      border-collapse: collapse;
      background: color-mix(in srgb, var(--panel) 82%, transparent);
      border: 1px solid var(--line);
    }}
    th, td {{
      padding: 9px 10px;
      border-bottom: 1px solid var(--line);
      text-align: left;
      vertical-align: top;
    }}
    th {{
      color: var(--muted);
      font-size: 11px;
      letter-spacing: 0.12em;
      text-transform: uppercase;
    }}
    .meta {{
      color: var(--muted);
      margin: 0;
    }}
    .context {{
      color: var(--muted);
      min-width: 220px;
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
      border: 1px solid var(--line);
      background: #101419;
      color: var(--text);
      line-height: 1.2;
    }}
    .proof-frame-grid {{
      display: flex;
      flex-wrap: wrap;
      gap: 14px;
      margin-top: 10px;
    }}
    .proof-frame {{
      border: 1px solid var(--line);
      background: #111820;
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
    {board_css}
    .marker--winning {{
      box-shadow: inset 0 0 0 2px var(--green);
    }}
    .marker--threat {{
      box-shadow: inset 0 0 0 2px var(--red);
      background: color-mix(in srgb, var(--red) 22%, transparent);
    }}
    .marker--imminent-defense {{
      box-shadow: inset 0 0 0 2px var(--pink);
      background: color-mix(in srgb, var(--pink) 22%, transparent);
    }}
    .marker--offensive-counter {{
      box-shadow: inset 0 0 0 2px var(--blue);
      background: color-mix(in srgb, var(--blue) 20%, transparent);
    }}
    .marker--escape .proof-marker {{
      color: var(--green);
      text-shadow: 0 1px 0 rgba(0,0,0,0.45);
    }}
    .marker--unproved-escape .proof-marker {{
      color: color-mix(in srgb, var(--green) 70%, var(--muted));
      text-shadow: 0 1px 0 rgba(0,0,0,0.45);
    }}
    .marker--immediate-loss .proof-marker {{
      color: var(--red);
      text-shadow: 0 1px 0 rgba(0,0,0,0.45);
    }}
    .marker--unknown-outcome .proof-marker {{
      color: var(--muted);
    }}
    .marker--actual {{
    }}
    .marker--actual .proof-marker {{
      font-size: 10px;
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
    .proof-legend {{
      display: flex;
      flex-wrap: wrap;
      gap: 8px 12px;
      margin: 8px 0;
      color: var(--muted);
      font-size: 11px;
    }}
    .proof-legend span::before {{
      content: "";
      display: inline-block;
      width: 10px;
      height: 10px;
      margin-right: 5px;
      vertical-align: -1px;
      border: 1px solid currentColor;
    }}
    .legend-winning::before {{ color: var(--green); background: color-mix(in srgb, var(--green) 25%, transparent); }}
    .legend-threat::before {{ color: var(--red); background: color-mix(in srgb, var(--red) 25%, transparent); }}
    .legend-imminent::before {{ color: var(--pink); background: color-mix(in srgb, var(--pink) 25%, transparent); }}
    .legend-offensive::before {{ color: var(--blue); background: color-mix(in srgb, var(--blue) 25%, transparent); }}
    .legend-actual::before {{ color: var(--text); background: color-mix(in srgb, var(--text) 30%, transparent); }}
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
      grid-template-columns: 38px 104px 74px minmax(0, 1fr);
      gap: 8px;
      align-items: baseline;
    }}
    .reply-outcome-row strong {{
      color: var(--text);
    }}
    .reply-outcome-row--header {{
      color: var(--faint);
      text-transform: uppercase;
      letter-spacing: 0.08em;
      font-size: 9px;
    }}
  </style>
</head>
<body>
<main>
  <h1>Replay Analysis Batch</h1>
  <p class="meta">{replay_dir}</p>
  <p class="meta">{model}</p>
  <p class="meta">{limit_summary}</p>
  <section class="summary">
    <article class="card"><span>Total</span><strong>{total}</strong></article>
    <article class="card"><span>Analyzed</span><strong>{analyzed}</strong></article>
    <article class="card"><span>Missed defense</span><strong>{missed_defense}</strong></article>
    <article class="card"><span>Missed win</span><strong>{missed_win}</strong></article>
    <article class="card"><span>Unclear</span><strong>{unclear}</strong></article>
    <article class="card"><span>Errors</span><strong>{failed}</strong></article>
  </section>
  <table>
    <thead>
      <tr><th>Replay</th><th>Status</th><th>Winner</th><th>Root</th><th>Why unclear</th><th>Forced</th><th>Unknowns</th><th>Time</th><th>Context</th><th>Proof details</th><th>Error</th></tr>
    </thead>
    <tbody>{rows}</tbody>
  </table>
</main>
</body>
</html>
"#,
        replay_dir = html_escape(&report.replay_dir),
        total = report.total,
        analyzed = report.analyzed,
        missed_defense = report.summary.missed_defense,
        missed_win = report.summary.missed_win,
        unclear = report.summary.unclear,
        failed = report.failed,
        model = html_escape(&format!(
            "{:?}, corridor depth {}, deep retry {} (limit {}), window {:?}",
            report.model.defense_policy,
            report.model.max_depth,
            option_debug(report.model.deep_retry_depth),
            report.model.deep_retry_limit,
            report.model.max_backward_window
        )),
        limit_summary = html_escape(&limit_summary),
        board_css = report_board_css(),
        rows = rows,
    )
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
    let plys = (analysis.final_forced_interval.start_ply..=analysis.final_forced_interval.end_ply)
        .rev()
        .collect::<Vec<_>>();

    plys.into_iter()
        .filter_map(|ply| {
            let board_ply = ply.checked_sub(1)?;
            let board = boards.get(board_ply)?;
            let proof = proof_result_at(&analysis.proof_summary, scan_start, board_ply);
            let mut markers = Vec::new();
            add_decision_tactical_markers(&mut markers, board);
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
                add_marker_kind(
                    &mut markers,
                    [actual_move],
                    AnalysisBatchProofMarkerKind::Actual,
                );
            }
            markers.sort_by_key(|marker| (marker.mv.row, marker.mv.col));
            Some(proof_frame(
                &actual_frame_label(ply, &analysis.final_forced_interval),
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

    let replies = analyze_defender_reply_options_with_retry(
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

fn add_decision_tactical_markers(markers: &mut Vec<AnalysisBatchProofMarker>, board: &Board) {
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

    match entry.root_cause {
        Some(RootCause::StrategicLoss) => summary.strategic_loss += 1,
        Some(RootCause::MissedDefense) => summary.missed_defense += 1,
        Some(RootCause::MissedWin) => summary.missed_win += 1,
        Some(RootCause::Unclear) | None => summary.unclear += 1,
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

    let previous = details
        .previous_proof
        .as_ref()
        .map(proof_snapshot_html)
        .unwrap_or_else(|| "Previous proof: unavailable".to_string());
    let final_start = details
        .final_start_proof
        .as_ref()
        .map(proof_snapshot_html)
        .unwrap_or_else(|| "Final proof: unavailable".to_string());
    let snapshots = details
        .snapshots
        .iter()
        .map(|snapshot| {
            format!(
                "<div><strong>{}</strong><pre>{}</pre></div>",
                html_escape(&snapshot_title(snapshot)),
                html_escape(&snapshot.rows.join("\n"))
            )
        })
        .collect::<String>();
    let frames = proof_frames_html(&details.proof_frames);

    format!(
        "<div class=\"context\"><details><summary>root transition</summary><div><strong>Escape boundary</strong> {previous_ply}</div><div><strong>Forced run entry</strong> {final_ply}</div>{previous}{final_start}</details>{frames}<details><summary>ASCII board snapshots</summary>{snapshots}</details></div>",
        previous_ply = details
            .previous_prefix_ply
            .map(before_ply_label)
            .unwrap_or_else(|| "-".to_string()),
        final_ply = before_ply_label(details.final_forced_start_ply),
        previous = previous,
        final_start = final_start,
        frames = frames,
        snapshots = snapshots,
    )
}

fn proof_frames_html(frames: &[AnalysisBatchProofFrame]) -> String {
    if frames.is_empty() {
        return String::new();
    }
    let frame_cards = frames.iter().map(proof_frame_html).collect::<String>();
    format!(
        "<details><summary>Visual decision frames</summary><div class=\"proof-legend\"><span class=\"legend-winning\">win now</span><span class=\"legend-threat\">immediate threat</span><span class=\"legend-imminent\">defensive reply</span><span class=\"legend-offensive\">offensive reply</span><span class=\"legend-actual\">actual replay move</span><span>L forced loss</span><span>E escape</span><span>U unproved escape</span><span>! immediate loss</span><span>? unknown</span></div><div class=\"proof-frame-grid\">{frame_cards}</div></details>"
    )
}

fn proof_frame_html(frame: &AnalysisBatchProofFrame) -> String {
    let title = proof_frame_title(frame);
    let board = proof_board_html(frame);
    let replies = reply_outcomes_html(frame);
    format!(
        "<article class=\"proof-frame\" data-ply=\"{ply}\"><h3>{title}</h3><p>{side} to move / {status}</p>{board}{replies}</article>",
        ply = frame.ply,
        title = html_escape(&title),
        side = html_escape(&format!("{:?}", frame.side_to_move)),
        status = html_escape(proof_status_label(frame.status)),
        board = board,
        replies = replies,
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
            format!(
                "<div class=\"reply-outcome-row\"><strong>{mv}</strong><span>{roles}</span><span>{outcome}</span><span>{line}</span></div>",
                mv = html_escape(&reply.notation),
                roles = html_escape(&reply_roles_label(&reply.roles)),
                outcome = html_escape(&defender_reply_outcome_label(reply)),
                line = html_escape(&reply_line_label(reply)),
            )
        })
        .collect::<String>();
    format!(
        "<div class=\"reply-outcomes\"><div class=\"reply-outcome-row reply-outcome-row--header\"><span>Move</span><span>Role</span><span>Outcome</span><span>Sample</span></div>{rows}</div>",
    )
}

fn proof_frame_title(frame: &AnalysisBatchProofFrame) -> String {
    if frame.label == "escape_boundary" {
        return before_ply_label(frame.ply);
    }
    match (frame.label.as_str(), frame.move_played_notation.as_deref()) {
        ("winning_ply", Some(mv)) => {
            format!(
                "winning move / before ply {} / actual move {}",
                frame.ply, mv
            )
        }
        (_, Some(mv)) => format!("before ply {} / actual move {}", frame.ply, mv),
        (_, None) => format!("{} @ ply {}", frame.label, frame.ply),
    }
}

fn proof_board_html(frame: &AnalysisBatchProofFrame) -> String {
    let markers = frame
        .markers
        .iter()
        .map(|marker| {
            let mut report_marker = ReportBoardMarker::new(marker.mv)
                .with_classes(marker_classes(frame, marker))
                .with_label(marker_label(marker));
            if marker.kinds.contains(&AnalysisBatchProofMarkerKind::Actual) {
                report_marker = report_marker.with_actual_stone(frame.side_to_move);
            }
            report_marker
        })
        .collect::<Vec<_>>();
    render_report_board(&frame.rows, &markers)
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
        classes.push(match kind {
            AnalysisBatchProofMarkerKind::Winning => "marker--winning",
            AnalysisBatchProofMarkerKind::Threat => "marker--threat",
            AnalysisBatchProofMarkerKind::ImminentDefense => "marker--imminent-defense",
            AnalysisBatchProofMarkerKind::OffensiveCounter => "marker--offensive-counter",
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
        return "W".to_string();
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
    let outcome = match reply.outcome {
        DefenderReplyOutcome::ForcedLoss => "forced loss",
        DefenderReplyOutcome::Escape => "escape",
        DefenderReplyOutcome::UnprovedEscape => "unproved escape",
        DefenderReplyOutcome::ImmediateLoss => "immediate loss",
        DefenderReplyOutcome::Unknown => "unknown",
    };
    let mut parts = Vec::new();
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
        outcome.to_string()
    } else {
        format!("{outcome} ({})", parts.join("; "))
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

fn proof_snapshot_html(snapshot: &AnalysisBatchProofSnapshot) -> String {
    format!(
        "<div><strong>proof before ply {ply}</strong></div><div>Status {status}; attacker {attacker}; side to move {side}; reply {reply}</div><div>Aggregate proof evidence: win squares {winning}; defender cost/block {cost}; escape {escape}; forced replies {forced}</div><div>Sample proof branch {line}</div><div>Limits {limits}</div>",
        ply = snapshot.prefix_ply + 1,
        status = html_escape(proof_status_label(snapshot.status)),
        attacker = html_escape(&format!("{:?}", snapshot.attacker)),
        side = html_escape(&format!("{:?}", snapshot.side_to_move)),
        reply = html_escape(snapshot.reply_classification.map(reply_classification_label).unwrap_or("-")),
        winning = html_escape(&move_list_label(&snapshot.winning_squares)),
        cost = html_escape(&move_list_label(&snapshot.legal_cost_squares)),
        escape = html_escape(&move_list_label(&snapshot.escape_replies)),
        forced = html_escape(&move_list_label(&snapshot.forced_replies)),
        line = html_escape(&move_list_label(&snapshot.principal_line)),
        limits = html_escape(&proof_limit_cause_labels(&snapshot.limit_causes)),
    )
}

fn snapshot_title(snapshot: &AnalysisBoardSnapshot) -> String {
    match snapshot.label.as_str() {
        "escape_boundary" | "previous_prefix" => before_ply_label(snapshot.ply),
        "forced_entry" => before_ply_label(snapshot.ply),
        label => format!("{label} @ ply {}", snapshot.ply),
    }
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

fn reply_classification_label(classification: ReplyClassification) -> &'static str {
    match classification {
        ReplyClassification::IgnoredSingleWin => "ignored single win",
        ReplyClassification::BlockedButForced => "blocked but forced",
        ReplyClassification::Escaped => "escaped",
        ReplyClassification::UnprovedEscape => "unproved escape",
        ReplyClassification::NoLegalBlock => "no legal block",
        ReplyClassification::Unknown => "unknown",
    }
}

fn move_list_label(moves: &[Move]) -> String {
    if moves.is_empty() {
        return "-".to_string();
    }
    moves
        .iter()
        .map(|mv| mv.to_notation())
        .collect::<Vec<_>>()
        .join(" ")
}

fn limit_cause_counts_label(counts: &[ProofLimitCauseCount]) -> String {
    if counts.is_empty() {
        return "Limit causes: none".to_string();
    }
    let parts = counts
        .iter()
        .map(|count| format!("{} {}", proof_limit_cause_label(count.cause), count.count))
        .collect::<Vec<_>>()
        .join(", ");
    format!("Limit causes: {parts}")
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

fn interval_label(interval: Option<&ForcedInterval>) -> String {
    interval
        .map(|interval| format!("{}..{}", interval.start_ply, interval.end_ply))
        .unwrap_or_else(|| "-".to_string())
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
        cell_classes, marker_label, render_analysis_batch_report_html, run_analysis_batch,
        run_analysis_batch_replays, run_analysis_batch_replays_with_options,
        AnalysisBatchProofFrame, AnalysisBatchProofMarker, AnalysisBatchProofMarkerKind,
        AnalysisBatchRunOptions, ReplayAnalysisInput,
    };
    use crate::analysis::{
        AnalysisOptions, DefensePolicy, ProofLimitCause, ProofStatus, ReplyClassification,
        RootCause, UnclearReason,
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
        assert_eq!(report.summary.missed_defense, 1);
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
        assert!(html.contains("missed defense"));
        assert!(html.contains("replay.json"));

        let _ = fs::remove_dir_all(&dir);
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
            vec![("winning_ply", 9), ("actual_ply_8", 8)]
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
        assert!(actual_l8
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::Winning));

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
        assert!(html.contains("root transition"));
        assert!(html.contains("Escape boundary"));
        assert!(html.contains("Forced run entry"));
        assert!(html.contains("proof before ply 8"));
        assert!(html.contains("attacker Black; side to move White"));
        assert!(html.contains("Aggregate proof evidence: win squares L8"));
        assert!(html.contains("Sample proof branch L8"));
        assert!(html.contains("before ply 8 / actual move B1"));
        assert!(html.contains("Visual decision frames"));
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
        assert!(html.contains("winning move / before ply 9 / actual move L8"));
        assert!(html.contains("Black to move / forced win"));
        assert!(html.contains("marker--winning"));
        assert!(html.contains("marker--threat"));
        assert!(html.contains("marker--imminent-defense"));
        assert!(html.contains("reply-outcomes"));
        assert!(html.contains("forced loss"));
        assert!(!html.contains("marker--principal"));
        assert!(!html.contains("marker--cost"));
        assert!(html.contains("legend-threat"));
        assert!(html.contains("legend-imminent"));
        assert!(!html.contains("legend-principal"));
        assert!(!html.contains("legend-cost"));
        assert!(!html.contains("<span class=\"proof-marker\">1</span>"));
        assert!(!html.contains("<span class=\"proof-marker\">2</span>"));
        assert!(html.contains("escaped"));
        assert!(html.contains("L8"));
    }

    #[test]
    fn analysis_batch_proof_marker_labels_are_semantic_not_step_numbers() {
        let mv = Move::from_notation("H8").unwrap();
        let winning = AnalysisBatchProofMarker {
            mv,
            notation: mv.to_notation(),
            kinds: vec![AnalysisBatchProofMarkerKind::Winning],
        };
        assert_eq!(marker_label(&winning), "W");

        let actual_threat = AnalysisBatchProofMarker {
            mv,
            notation: mv.to_notation(),
            kinds: vec![
                AnalysisBatchProofMarkerKind::Threat,
                AnalysisBatchProofMarkerKind::Actual,
            ],
        };
        assert_eq!(marker_label(&actual_threat), "L");

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
        assert!(g7.kinds.contains(&AnalysisBatchProofMarkerKind::Actual));
        assert!(g7
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::ImminentDefense));
        assert!(g7.kinds.contains(&AnalysisBatchProofMarkerKind::ForcedLoss));
        assert!(cell_classes(frame, '.', Some(g7)).contains("marker--side-white"));

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
        assert!(html.contains(
            ".marker--imminent-defense {\n      box-shadow: inset 0 0 0 2px var(--pink);"
        ));
        assert!(html.contains(
            ".marker--offensive-counter {\n      box-shadow: inset 0 0 0 2px var(--blue);"
        ));
        assert!(html.contains("legend-offensive"));
        assert!(html.contains(".marker--side-white .proof-marker"));
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
