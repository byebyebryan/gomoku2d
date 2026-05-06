use std::path::Path;
use std::time::Instant;

use gomoku_core::{Color, Move, Replay};
use rayon::prelude::*;
use serde::Serialize;

use crate::analysis::{
    analyze_replay, AnalysisOptions, DefensePolicy, ForcedInterval, GameAnalysis, ProofStatus,
    RootCause, TacticalNote, UnclearReason, ANALYSIS_SCHEMA_VERSION,
};

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
    pub entries: Vec<AnalysisBatchEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AnalysisBatchModel {
    pub defense_policy: DefensePolicy,
    pub max_depth: usize,
    pub max_forced_extensions: usize,
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
    pub elapsed_ms: u64,
    pub prefixes_analyzed: usize,
    pub forced_prefix_count: usize,
    pub unknown_prefix_count: usize,
    pub escape_prefix_count: usize,
    pub error: Option<String>,
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

pub fn run_analysis_batch(
    replay_dir: &Path,
    options: AnalysisOptions,
) -> Result<AnalysisBatchReport, String> {
    let batch_started = Instant::now();
    let model = AnalysisBatchModel {
        defense_policy: options.defense_policy,
        max_depth: options.max_depth,
        max_forced_extensions: options.max_forced_extensions,
        max_backward_window: options.max_backward_window,
    };
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
            match analyze_replay_file(path, options.clone()) {
                Ok(analysis) => entry_from_analysis(
                    relative_path,
                    analysis,
                    elapsed_millis(entry_started.elapsed()),
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
        entries,
    })
}

pub fn run_analysis_batch_replays(
    source: String,
    inputs: Vec<ReplayAnalysisInput>,
    options: AnalysisOptions,
) -> AnalysisBatchReport {
    let batch_started = Instant::now();
    let model = AnalysisBatchModel {
        defense_policy: options.defense_policy,
        max_depth: options.max_depth,
        max_forced_extensions: options.max_forced_extensions,
        max_backward_window: options.max_backward_window,
    };

    let entries = inputs
        .par_iter()
        .map(|input| {
            let entry_started = Instant::now();
            match analyze_replay(&input.replay, options.clone()) {
                Ok(analysis) => entry_from_analysis(
                    input.label.clone(),
                    analysis,
                    elapsed_millis(entry_started.elapsed()),
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
        entries,
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
        elapsed_ms,
        prefixes_analyzed: 0,
        forced_prefix_count: 0,
        unknown_prefix_count: 0,
        escape_prefix_count: 0,
        error: Some(error),
    }
}

pub fn render_analysis_batch_report_html(report: &AnalysisBatchReport) -> String {
    let rows = report
        .entries
        .iter()
        .map(|entry| {
            format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{} ms</td><td>{}</td></tr>",
                html_escape(&entry.path),
                html_escape(entry_status_label(entry.status)),
                html_escape(&option_debug(entry.winner)),
                html_escape(&root_cause_label(entry.root_cause)),
                html_escape(&unclear_reason_label(entry.unclear_reason)),
                html_escape(&interval_label(entry.final_forced_interval.as_ref())),
                html_escape(&entry.unknown_gap_count.to_string()),
                entry.elapsed_ms,
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
  </style>
</head>
<body>
<main>
  <h1>Replay Analysis Batch</h1>
  <p class="meta">{replay_dir}</p>
  <p class="meta">{model}</p>
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
      <tr><th>Replay</th><th>Status</th><th>Winner</th><th>Root</th><th>Why unclear</th><th>Forced</th><th>Unknowns</th><th>Time</th><th>Error</th></tr>
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
            "{:?}, depth {}, forced extensions {}, window {:?}",
            report.model.defense_policy,
            report.model.max_depth,
            report.model.max_forced_extensions,
            report.model.max_backward_window
        )),
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

fn analyze_replay_file(path: &Path, options: AnalysisOptions) -> Result<GameAnalysis, String> {
    let json = std::fs::read_to_string(path)
        .map_err(|err| format!("failed to read replay JSON: {err}"))?;
    let replay =
        Replay::from_json(&json).map_err(|err| format!("failed to parse replay: {err}"))?;
    analyze_replay(&replay, options).map_err(|err| format!("failed to analyze replay: {err}"))
}

fn entry_from_analysis(
    path: String,
    analysis: GameAnalysis,
    elapsed_ms: u64,
) -> AnalysisBatchEntry {
    let prefixes_analyzed = analysis.proof_summary.len();
    let forced_prefix_count = count_proof_status(&analysis, ProofStatus::ForcedWin);
    let unknown_prefix_count = count_proof_status(&analysis, ProofStatus::Unknown);
    let escape_prefix_count = count_proof_status(&analysis, ProofStatus::EscapeFound);

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
        elapsed_ms,
        prefixes_analyzed,
        forced_prefix_count,
        unknown_prefix_count,
        escape_prefix_count,
        error: None,
    }
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

    use gomoku_core::{Board, Move, Replay, RuleConfig, Variant};

    use super::{
        render_analysis_batch_report_html, run_analysis_batch, run_analysis_batch_replays,
        ReplayAnalysisInput,
    };
    use crate::analysis::{AnalysisOptions, RootCause};

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
}
