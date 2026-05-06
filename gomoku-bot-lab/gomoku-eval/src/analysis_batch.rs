use std::path::Path;

use gomoku_core::{Color, Replay};
use serde::Serialize;

use crate::analysis::{
    analyze_replay, AnalysisOptions, DefensePolicy, ForcedInterval, GameAnalysis, RootCause,
    ANALYSIS_SCHEMA_VERSION,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AnalysisBatchReport {
    pub schema_version: u32,
    pub replay_dir: String,
    pub total: usize,
    pub analyzed: usize,
    pub failed: usize,
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
    pub final_forced_interval: Option<ForcedInterval>,
    pub last_chance_ply: Option<usize>,
    pub critical_mistake_ply: Option<usize>,
    pub unknown_gap_count: usize,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisBatchEntryStatus {
    Analyzed,
    Error,
}

pub fn run_analysis_batch(
    replay_dir: &Path,
    options: AnalysisOptions,
) -> Result<AnalysisBatchReport, String> {
    let model = AnalysisBatchModel {
        defense_policy: options.defense_policy,
        max_depth: options.max_depth,
        max_forced_extensions: options.max_forced_extensions,
        max_backward_window: options.max_backward_window,
    };
    let mut paths = replay_paths(replay_dir)?;
    paths.sort();

    let mut summary = AnalysisBatchSummary::default();
    let mut entries = Vec::with_capacity(paths.len());
    let mut analyzed = 0;
    let mut failed = 0;

    for path in paths {
        let relative_path = path
            .strip_prefix(replay_dir)
            .unwrap_or(&path)
            .display()
            .to_string();
        match analyze_replay_file(&path, options.clone()) {
            Ok(analysis) => {
                analyzed += 1;
                increment_summary(&mut summary, &analysis);
                entries.push(entry_from_analysis(relative_path, analysis));
            }
            Err(error) => {
                failed += 1;
                summary.analysis_error += 1;
                entries.push(AnalysisBatchEntry {
                    path: relative_path,
                    status: AnalysisBatchEntryStatus::Error,
                    winner: None,
                    root_cause: None,
                    final_forced_interval: None,
                    last_chance_ply: None,
                    critical_mistake_ply: None,
                    unknown_gap_count: 0,
                    error: Some(error),
                });
            }
        }
    }

    Ok(AnalysisBatchReport {
        schema_version: ANALYSIS_SCHEMA_VERSION,
        replay_dir: replay_dir.display().to_string(),
        total: entries.len(),
        analyzed,
        failed,
        model,
        summary,
        entries,
    })
}

pub fn render_analysis_batch_report_html(report: &AnalysisBatchReport) -> String {
    let rows = report
        .entries
        .iter()
        .map(|entry| {
            format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                html_escape(&entry.path),
                html_escape(entry_status_label(entry.status)),
                html_escape(&option_debug(entry.winner)),
                html_escape(&root_cause_label(entry.root_cause)),
                html_escape(&interval_label(entry.final_forced_interval.as_ref())),
                html_escape(&entry.unknown_gap_count.to_string()),
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
      <tr><th>Replay</th><th>Status</th><th>Winner</th><th>Root</th><th>Forced</th><th>Unknowns</th><th>Error</th></tr>
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

fn entry_from_analysis(path: String, analysis: GameAnalysis) -> AnalysisBatchEntry {
    AnalysisBatchEntry {
        path,
        status: AnalysisBatchEntryStatus::Analyzed,
        winner: analysis.winner,
        root_cause: Some(analysis.root_cause),
        final_forced_interval: Some(analysis.final_forced_interval),
        last_chance_ply: analysis.last_chance_ply,
        critical_mistake_ply: analysis.critical_mistake_ply,
        unknown_gap_count: analysis.unknown_gaps.len(),
        error: None,
    }
}

fn increment_summary(summary: &mut AnalysisBatchSummary, analysis: &GameAnalysis) {
    if analysis.winner.is_none() {
        summary.ongoing_or_draw += 1;
        return;
    }

    match analysis.root_cause {
        RootCause::StrategicLoss => summary.strategic_loss += 1,
        RootCause::MissedDefense => summary.missed_defense += 1,
        RootCause::MissedWin => summary.missed_win += 1,
        RootCause::Unclear => summary.unclear += 1,
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

    use super::{render_analysis_batch_report_html, run_analysis_batch};
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
}
