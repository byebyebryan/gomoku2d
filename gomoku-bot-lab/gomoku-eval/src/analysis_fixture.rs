use gomoku_core::{Board, Color, Move, Replay, RuleConfig, Variant};
use serde::Serialize;

use crate::analysis::{
    analysis_model, analyze_replay, rule_label, AnalysisError, AnalysisModel, AnalysisOptions,
    DefensePolicy, ForcedInterval, ProofStatus, RootCause, TacticalNote, ANALYSIS_SCHEMA_VERSION,
};

#[derive(Debug, Clone)]
pub struct AnalysisFixtureCase {
    pub case_id: &'static str,
    pub description: &'static str,
    pub variant: Variant,
    pub moves: &'static [&'static str],
    pub options: AnalysisFixtureOptions,
    pub expected: AnalysisFixtureExpected,
}

#[derive(Debug, Clone, Default)]
pub struct AnalysisFixtureOptions {
    pub max_depth: Option<usize>,
    pub max_forced_extensions: Option<usize>,
    pub defense_policy: Option<DefensePolicy>,
    pub max_backward_window: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct AnalysisFixtureExpected {
    pub winner: Option<Color>,
    pub root_cause: RootCause,
    pub final_forced_interval: ForcedInterval,
    pub last_chance_ply: Option<usize>,
    pub critical_mistake_ply: Option<usize>,
    pub tactical_notes: &'static [TacticalNote],
    pub required_unknown_gaps: &'static [usize],
}

pub const ANALYSIS_FIXTURE_CASES: &[AnalysisFixtureCase] = &[
    AnalysisFixtureCase {
        case_id: "missed_defense_closed_four",
        description: "White has a legal escape against Black's closed four, misses it, and Black wins next.",
        variant: Variant::Freestyle,
        moves: &["H8", "G8", "I8", "A1", "J8", "A2", "K8", "B1", "L8"],
        options: AnalysisFixtureOptions {
            max_depth: None,
            max_forced_extensions: None,
            defense_policy: None,
            max_backward_window: Some(3),
        },
        expected: AnalysisFixtureExpected {
            winner: Some(Color::Black),
            root_cause: RootCause::MissedDefense,
            final_forced_interval: ForcedInterval {
                start_ply: 8,
                end_ply: 9,
            },
            last_chance_ply: Some(7),
            critical_mistake_ply: Some(8),
            tactical_notes: &[TacticalNote::AccidentalBlunder],
            required_unknown_gaps: &[],
        },
    },
    AnalysisFixtureCase {
        case_id: "conversion_error_then_missed_defense",
        description: "Black has an immediate conversion, delays it, then White still misses the final block.",
        variant: Variant::Freestyle,
        moves: &[
            "H8", "G8", "I8", "A1", "J8", "A2", "K8", "B1", "C1", "B2", "L8",
        ],
        options: AnalysisFixtureOptions {
            max_depth: None,
            max_forced_extensions: None,
            defense_policy: None,
            max_backward_window: Some(4),
        },
        expected: AnalysisFixtureExpected {
            winner: Some(Color::Black),
            root_cause: RootCause::MissedDefense,
            final_forced_interval: ForcedInterval {
                start_ply: 10,
                end_ply: 11,
            },
            last_chance_ply: Some(9),
            critical_mistake_ply: Some(10),
            tactical_notes: &[
                TacticalNote::AccidentalBlunder,
                TacticalNote::ConversionError,
                TacticalNote::MissedWin,
            ],
            required_unknown_gaps: &[],
        },
    },
    AnalysisFixtureCase {
        case_id: "losing_side_missed_win",
        description: "The losing side has an immediate win available, ignores it, and loses next.",
        variant: Variant::Freestyle,
        moves: &[
            "A1", "H8", "A2", "I8", "A3", "J8", "B1", "K8", "A4", "C1", "A5",
        ],
        options: AnalysisFixtureOptions {
            max_depth: None,
            max_forced_extensions: None,
            defense_policy: None,
            max_backward_window: Some(3),
        },
        expected: AnalysisFixtureExpected {
            winner: Some(Color::Black),
            root_cause: RootCause::MissedWin,
            final_forced_interval: ForcedInterval {
                start_ply: 10,
                end_ply: 11,
            },
            last_chance_ply: Some(9),
            critical_mistake_ply: Some(10),
            tactical_notes: &[TacticalNote::MissedWin],
            required_unknown_gaps: &[],
        },
    },
    AnalysisFixtureCase {
        case_id: "unknown_guard_open_four_depth1",
        description: "A deliberately shallow model must keep an unknown prefix instead of over-labeling the loss.",
        variant: Variant::Freestyle,
        moves: &["H8", "A1", "I8", "A2", "J8", "A3", "K8", "B1", "L8"],
        options: AnalysisFixtureOptions {
            max_depth: Some(1),
            max_forced_extensions: None,
            defense_policy: None,
            max_backward_window: Some(3),
        },
        expected: AnalysisFixtureExpected {
            winner: Some(Color::Black),
            root_cause: RootCause::Unclear,
            final_forced_interval: ForcedInterval {
                start_ply: 7,
                end_ply: 9,
            },
            last_chance_ply: None,
            critical_mistake_ply: None,
            tactical_notes: &[],
            required_unknown_gaps: &[6],
        },
    },
    AnalysisFixtureCase {
        case_id: "forced_chain_closed_four_to_open_four",
        description: "Black creates a closed four, White blocks, then Black extends the new stone into an open-four win.",
        variant: Variant::Freestyle,
        moves: &[
            "H8", "G8", "I8", "A1", "J8", "C1", "K6", "E1", "K7", "G1", "K8",
            "L8", "K9", "K5", "K10",
        ],
        options: AnalysisFixtureOptions {
            max_depth: Some(2),
            max_forced_extensions: Some(4),
            defense_policy: None,
            max_backward_window: Some(6),
        },
        expected: AnalysisFixtureExpected {
            winner: Some(Color::Black),
            root_cause: RootCause::Unclear,
            final_forced_interval: ForcedInterval {
                start_ply: 10,
                end_ply: 15,
            },
            last_chance_ply: None,
            critical_mistake_ply: None,
            tactical_notes: &[],
            required_unknown_gaps: &[9],
        },
    },
    AnalysisFixtureCase {
        case_id: "ongoing_replay_unclear",
        description: "An unfinished replay should produce a bounded summary without forced-win claims.",
        variant: Variant::Freestyle,
        moves: &["H8", "A1", "I8"],
        options: AnalysisFixtureOptions {
            max_depth: None,
            max_forced_extensions: None,
            defense_policy: None,
            max_backward_window: None,
        },
        expected: AnalysisFixtureExpected {
            winner: None,
            root_cause: RootCause::Unclear,
            final_forced_interval: ForcedInterval {
                start_ply: 0,
                end_ply: 0,
            },
            last_chance_ply: None,
            critical_mistake_ply: None,
            tactical_notes: &[],
            required_unknown_gaps: &[],
        },
    },
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AnalysisFixtureReport {
    pub schema_version: u32,
    pub fixture_count: usize,
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub base_model: AnalysisModel,
    pub results: Vec<AnalysisFixtureResult>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AnalysisFixtureResult {
    pub case_id: String,
    pub description: String,
    pub passed: bool,
    pub variant: String,
    pub moves: Vec<String>,
    pub expected: AnalysisFixtureExpectationReport,
    pub actual: AnalysisFixtureActualReport,
    pub failures: Vec<String>,
    pub proof_rows: Vec<AnalysisFixtureProofRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AnalysisFixtureExpectationReport {
    pub winner: Option<Color>,
    pub root_cause: RootCause,
    pub final_forced_interval: ForcedInterval,
    pub last_chance_ply: Option<usize>,
    pub critical_mistake_ply: Option<usize>,
    pub tactical_notes: Vec<TacticalNote>,
    pub required_unknown_gaps: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AnalysisFixtureActualReport {
    pub winner: Option<Color>,
    pub root_cause: RootCause,
    pub final_forced_interval: ForcedInterval,
    pub last_chance_ply: Option<usize>,
    pub critical_mistake_ply: Option<usize>,
    pub tactical_notes: Vec<TacticalNote>,
    pub unknown_gaps: Vec<usize>,
    pub model: AnalysisModel,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AnalysisFixtureProofRow {
    pub ply: usize,
    pub side_to_move: Color,
    pub status: ProofStatus,
    pub principal_line: Vec<Move>,
    pub escape_moves: Vec<Move>,
}

pub fn run_analysis_fixtures(
    base_options: AnalysisOptions,
) -> Result<AnalysisFixtureReport, AnalysisError> {
    let mut results = Vec::with_capacity(ANALYSIS_FIXTURE_CASES.len());
    for case in ANALYSIS_FIXTURE_CASES {
        results.push(run_analysis_fixture(case, &base_options)?);
    }
    let passed = results.iter().filter(|result| result.passed).count();
    let failed = results.len() - passed;
    let base_model = analysis_model(
        &Board::new(RuleConfig {
            variant: Variant::Freestyle,
            ..RuleConfig::default()
        }),
        &base_options,
    );

    Ok(AnalysisFixtureReport {
        schema_version: ANALYSIS_SCHEMA_VERSION,
        fixture_count: ANALYSIS_FIXTURE_CASES.len(),
        total: results.len(),
        passed,
        failed,
        base_model,
        results,
    })
}

fn run_analysis_fixture(
    case: &AnalysisFixtureCase,
    base_options: &AnalysisOptions,
) -> Result<AnalysisFixtureResult, AnalysisError> {
    let options = fixture_options(base_options, &case.options);
    let replay = replay_from_notation_moves(case.variant.clone(), case.moves)?;
    let analysis = analyze_replay(&replay, options)?;
    let proof_scan_start = replay.moves.len() + 1 - analysis.proof_summary.len();
    let expected = expectation_report(&case.expected);
    let actual = AnalysisFixtureActualReport {
        winner: analysis.winner,
        root_cause: analysis.root_cause,
        final_forced_interval: analysis.final_forced_interval.clone(),
        last_chance_ply: analysis.last_chance_ply,
        critical_mistake_ply: analysis.critical_mistake_ply,
        tactical_notes: analysis.tactical_notes.clone(),
        unknown_gaps: analysis.unknown_gaps.clone(),
        model: analysis.model.clone(),
    };
    let failures = fixture_failures(&expected, &actual);

    Ok(AnalysisFixtureResult {
        case_id: case.case_id.to_string(),
        description: case.description.to_string(),
        passed: failures.is_empty(),
        variant: rule_label(&case.variant).to_string(),
        moves: case.moves.iter().map(|mv| (*mv).to_string()).collect(),
        expected,
        actual,
        failures,
        proof_rows: analysis
            .proof_summary
            .iter()
            .enumerate()
            .map(|(ply, proof)| AnalysisFixtureProofRow {
                ply: proof_scan_start + ply,
                side_to_move: proof.side_to_move,
                status: proof.status,
                principal_line: proof.principal_line.clone(),
                escape_moves: proof.escape_moves.clone(),
            })
            .collect(),
    })
}

fn fixture_options(
    base_options: &AnalysisOptions,
    fixture_options: &AnalysisFixtureOptions,
) -> AnalysisOptions {
    AnalysisOptions {
        defense_policy: fixture_options
            .defense_policy
            .unwrap_or(base_options.defense_policy),
        max_depth: fixture_options.max_depth.unwrap_or(base_options.max_depth),
        max_forced_extensions: fixture_options
            .max_forced_extensions
            .unwrap_or(base_options.max_forced_extensions),
        max_backward_window: fixture_options
            .max_backward_window
            .or(base_options.max_backward_window),
    }
}

fn replay_from_notation_moves(variant: Variant, moves: &[&str]) -> Result<Replay, AnalysisError> {
    let rules = RuleConfig {
        variant,
        ..RuleConfig::default()
    };
    let mut board = Board::new(rules.clone());
    let mut replay = Replay::new(rules, "Black", "White");

    for (idx, notation) in moves.iter().enumerate() {
        let ply = idx + 1;
        let mv = Move::from_notation(notation)
            .map_err(|message| AnalysisError::InvalidReplayMove { ply, message })?;
        board
            .apply_move(mv)
            .map_err(|err| AnalysisError::InvalidReplayMove {
                ply,
                message: err.to_string(),
            })?;
        replay.push_move(mv, 0, board.hash(), None);
    }
    replay.finish(&board.result, Some(0));
    Ok(replay)
}

fn expectation_report(expected: &AnalysisFixtureExpected) -> AnalysisFixtureExpectationReport {
    AnalysisFixtureExpectationReport {
        winner: expected.winner,
        root_cause: expected.root_cause,
        final_forced_interval: expected.final_forced_interval.clone(),
        last_chance_ply: expected.last_chance_ply,
        critical_mistake_ply: expected.critical_mistake_ply,
        tactical_notes: expected.tactical_notes.to_vec(),
        required_unknown_gaps: expected.required_unknown_gaps.to_vec(),
    }
}

fn fixture_failures(
    expected: &AnalysisFixtureExpectationReport,
    actual: &AnalysisFixtureActualReport,
) -> Vec<String> {
    let mut failures = Vec::new();
    if actual.winner != expected.winner {
        failures.push(format!(
            "winner expected {:?}, got {:?}",
            expected.winner, actual.winner
        ));
    }
    if actual.root_cause != expected.root_cause {
        failures.push(format!(
            "root_cause expected {:?}, got {:?}",
            expected.root_cause, actual.root_cause
        ));
    }
    if actual.final_forced_interval != expected.final_forced_interval {
        failures.push(format!(
            "final_forced_interval expected {:?}, got {:?}",
            expected.final_forced_interval, actual.final_forced_interval
        ));
    }
    if actual.last_chance_ply != expected.last_chance_ply {
        failures.push(format!(
            "last_chance_ply expected {:?}, got {:?}",
            expected.last_chance_ply, actual.last_chance_ply
        ));
    }
    if actual.critical_mistake_ply != expected.critical_mistake_ply {
        failures.push(format!(
            "critical_mistake_ply expected {:?}, got {:?}",
            expected.critical_mistake_ply, actual.critical_mistake_ply
        ));
    }
    if actual.tactical_notes != expected.tactical_notes {
        failures.push(format!(
            "tactical_notes expected {:?}, got {:?}",
            expected.tactical_notes, actual.tactical_notes
        ));
    }
    for gap in &expected.required_unknown_gaps {
        if !actual.unknown_gaps.contains(gap) {
            failures.push(format!(
                "unknown_gaps expected to contain {}, got {:?}",
                gap, actual.unknown_gaps
            ));
        }
    }
    failures
}

pub fn render_analysis_fixture_report_html(report: &AnalysisFixtureReport) -> String {
    let mut cases = String::new();
    for result in &report.results {
        cases.push_str(&render_analysis_fixture_case_html(result));
    }

    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Gomoku2D Analysis Fixture Report</title>
<style>
:root {{
  color-scheme: dark;
  --bg: #1e1e1e;
  --surface: #2a2a2a;
  --card: #232323;
  --border: #575756;
  --text: #f5f5f5;
  --muted: #a6a6a0;
  --accent: #fccb57;
  --green: #5ad17a;
  --red: #ff6b6b;
  --teal: #5fc7c2;
}}
* {{ box-sizing: border-box; }}
body {{
  margin: 0;
  background: var(--bg);
  color: var(--text);
  font: 16px/1.45 ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
}}
main {{
  display: grid;
  gap: 20px;
  margin: 0 auto;
  max-width: 1120px;
  padding: 28px;
}}
h1, h2, h3, p {{ margin: 0; }}
a {{ color: inherit; }}
.hero, .case, .summary-card {{
  background: var(--surface);
  border: 2px solid var(--border);
  display: grid;
  gap: 14px;
  padding: 18px;
}}
.top-links {{ display: flex; flex-wrap: wrap; gap: 8px; }}
.top-links a {{
  background: var(--card);
  border: 2px solid var(--border);
  padding: 8px 12px;
  text-decoration: none;
  text-transform: uppercase;
}}
.top-links a:hover {{ border-color: var(--teal); }}
.eyebrow {{
  color: var(--accent);
  font-size: 12px;
  letter-spacing: .16em;
  text-transform: uppercase;
}}
h1 {{ font-size: clamp(32px, 6vw, 56px); line-height: 1; }}
.lede {{ color: var(--muted); max-width: 76ch; }}
.summary-grid {{
  display: grid;
  gap: 12px;
  grid-template-columns: repeat(auto-fit, minmax(170px, 1fr));
}}
.summary-card span {{
  color: var(--muted);
  font-size: 12px;
  letter-spacing: .1em;
  text-transform: uppercase;
}}
.summary-card strong {{ color: var(--green); font-size: 1.35rem; }}
.case {{ border-color: var(--border); }}
.case.fail {{ border-color: var(--red); }}
.case-head {{
  align-items: start;
  display: grid;
  gap: 12px;
  grid-template-columns: auto 1fr auto;
}}
.badge {{
  border: 2px solid var(--border);
  color: var(--muted);
  padding: 4px 8px;
  text-transform: uppercase;
}}
.badge.pass {{ border-color: var(--green); color: var(--green); }}
.badge.fail {{ border-color: var(--red); color: var(--red); }}
.case-id {{ color: var(--accent); font-size: 1.1rem; }}
.desc {{ color: var(--muted); }}
.meta-grid, .compare-grid {{
  display: grid;
  gap: 10px;
  grid-template-columns: repeat(auto-fit, minmax(210px, 1fr));
}}
.kv {{
  background: var(--card);
  border: 1px solid var(--border);
  padding: 12px;
}}
.kv span {{
  color: var(--muted);
  display: block;
  font-size: 12px;
  letter-spacing: .08em;
  text-transform: uppercase;
}}
.kv strong {{ color: var(--text); word-break: break-word; }}
.compare {{ display: grid; gap: 8px; }}
.compare h3 {{ color: var(--accent); font-size: 1rem; }}
table {{ border-collapse: collapse; width: 100%; }}
th, td {{
  border-bottom: 1px solid var(--border);
  padding: 8px 10px;
  text-align: left;
  vertical-align: top;
}}
th {{
  color: var(--muted);
  font-size: 12px;
  letter-spacing: .08em;
  text-transform: uppercase;
}}
code, .moves {{ color: var(--accent); }}
.failures {{
  color: var(--red);
  display: grid;
  gap: 6px;
}}
@media (max-width: 720px) {{
  main {{ padding: 18px; }}
  .case-head {{ grid-template-columns: 1fr; }}
  table {{ display: block; overflow-x: auto; }}
}}
</style>
</head>
<body>
<main>
<header class="hero">
  <nav class="top-links"><a href="/">Outputs</a></nav>
  <p class="eyebrow">Gomoku2D Bot Lab</p>
  <h1>Analysis Fixture Report</h1>
  <p class="lede">Curated replay fixtures for validating bounded replay analysis and narrow forced-chain search.</p>
</header>
<section class="summary-grid">
  <article class="summary-card"><span>Result</span><strong>{passed} passed / {total} total</strong></article>
  <article class="summary-card"><span>Failed</span><strong>{failed}</strong></article>
  <article class="summary-card"><span>Defense</span><strong>{defense:?}</strong></article>
  <article class="summary-card"><span>Depth</span><strong>{depth}</strong></article>
  <article class="summary-card"><span>Forced Ext.</span><strong>{forced_extensions}</strong></article>
</section>
{cases}
</main>
</body>
</html>
"#,
        passed = report.passed,
        total = report.total,
        failed = report.failed,
        defense = report.base_model.defense_policy,
        depth = report.base_model.max_depth,
        forced_extensions = report.base_model.max_forced_extensions,
        cases = cases,
    )
}

fn render_analysis_fixture_case_html(result: &AnalysisFixtureResult) -> String {
    let pass_class = if result.passed { "pass" } else { "fail" };
    let failures = if result.failures.is_empty() {
        String::new()
    } else {
        format!(
            r#"<div class="failures">{}</div>"#,
            result
                .failures
                .iter()
                .map(|failure| format!("<p>{}</p>", html_escape(failure)))
                .collect::<String>()
        )
    };
    let proof_rows = result
        .proof_rows
        .iter()
        .map(|row| {
            format!(
                "<tr><td>{}</td><td>{:?}</td><td>{:?}</td><td>{}</td><td>{}</td></tr>",
                row.ply,
                row.side_to_move,
                row.status,
                html_escape(&moves_label(&row.principal_line)),
                html_escape(&moves_label(&row.escape_moves))
            )
        })
        .collect::<String>();

    format!(
        r#"<section class="case {pass_class}">
  <div class="case-head">
    <span class="badge {pass_class}">{status}</span>
    <div>
      <h2 class="case-id">{case_id}</h2>
      <p class="desc">{description}</p>
    </div>
    <span class="badge">{variant}</span>
  </div>
  {failures}
  <div class="meta-grid">
    <div class="kv"><span>Moves</span><strong class="moves">{moves}</strong></div>
    <div class="kv"><span>Model</span><strong>{model}</strong></div>
  </div>
  <div class="compare-grid">
    <div class="compare">
      <h3>Expected</h3>
      {expected}
    </div>
    <div class="compare">
      <h3>Actual</h3>
      {actual}
    </div>
  </div>
  <div>
    <h3>Proof Rows</h3>
    <table>
      <thead><tr><th>Ply</th><th>Side</th><th>Status</th><th>Principal</th><th>Escapes</th></tr></thead>
      <tbody>{proof_rows}</tbody>
    </table>
  </div>
</section>"#,
        pass_class = pass_class,
        status = if result.passed { "PASS" } else { "FAIL" },
        case_id = html_escape(&result.case_id),
        description = html_escape(&result.description),
        variant = html_escape(&result.variant),
        failures = failures,
        moves = html_escape(&result.moves.join(" ")),
        model = html_escape(&format!(
            "{:?}, depth {}, forced extensions {}, window {:?}",
            result.actual.model.defense_policy,
            result.actual.model.max_depth,
            result.actual.model.max_forced_extensions,
            result.actual.model.max_backward_window
        )),
        expected = expectation_table(&result.expected),
        actual = actual_table(&result.actual),
        proof_rows = proof_rows,
    )
}

fn expectation_table(expected: &AnalysisFixtureExpectationReport) -> String {
    key_value_table(&[
        ("Winner", option_debug(expected.winner)),
        ("Root", format!("{:?}", expected.root_cause)),
        ("Forced", interval_label(&expected.final_forced_interval)),
        ("Last Chance", option_usize(expected.last_chance_ply)),
        ("Critical", option_usize(expected.critical_mistake_ply)),
        ("Notes", notes_label(&expected.tactical_notes)),
        (
            "Unknown Gaps",
            usize_list_label(&expected.required_unknown_gaps),
        ),
    ])
}

fn actual_table(actual: &AnalysisFixtureActualReport) -> String {
    key_value_table(&[
        ("Winner", option_debug(actual.winner)),
        ("Root", format!("{:?}", actual.root_cause)),
        ("Forced", interval_label(&actual.final_forced_interval)),
        ("Last Chance", option_usize(actual.last_chance_ply)),
        ("Critical", option_usize(actual.critical_mistake_ply)),
        ("Notes", notes_label(&actual.tactical_notes)),
        ("Unknown Gaps", usize_list_label(&actual.unknown_gaps)),
    ])
}

fn key_value_table(rows: &[(&str, String)]) -> String {
    let rows = rows
        .iter()
        .map(|(key, value)| {
            format!(
                "<tr><th>{}</th><td>{}</td></tr>",
                html_escape(key),
                html_escape(value)
            )
        })
        .collect::<String>();
    format!("<table><tbody>{rows}</tbody></table>")
}

fn option_debug<T: std::fmt::Debug>(value: Option<T>) -> String {
    value
        .map(|value| format!("{value:?}"))
        .unwrap_or_else(|| "-".to_string())
}

fn option_usize(value: Option<usize>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn interval_label(interval: &ForcedInterval) -> String {
    format!("{}..{}", interval.start_ply, interval.end_ply)
}

fn notes_label(notes: &[TacticalNote]) -> String {
    if notes.is_empty() {
        "-".to_string()
    } else {
        notes
            .iter()
            .map(|note| format!("{note:?}"))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn usize_list_label(values: &[usize]) -> String {
    if values.is_empty() {
        "-".to_string()
    } else {
        values
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn moves_label(moves: &[Move]) -> String {
    if moves.is_empty() {
        "-".to_string()
    } else {
        moves
            .iter()
            .map(|mv| mv.to_notation())
            .collect::<Vec<_>>()
            .join(" ")
    }
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
    use gomoku_core::Move;

    use super::{
        render_analysis_fixture_report_html, run_analysis_fixtures, ANALYSIS_FIXTURE_CASES,
    };
    use crate::analysis::{AnalysisOptions, ForcedInterval, ProofStatus, RootCause};

    fn mv(notation: &str) -> Move {
        Move::from_notation(notation).expect("test move notation should parse")
    }

    #[test]
    fn analysis_fixtures_capture_curated_replay_expectations() {
        let report = run_analysis_fixtures(AnalysisOptions::default())
            .expect("curated analysis fixtures should run");

        assert_eq!(report.total, ANALYSIS_FIXTURE_CASES.len());
        assert_eq!(report.failed, 0);
        assert_eq!(report.passed, ANALYSIS_FIXTURE_CASES.len());

        let missed_defense = report
            .results
            .iter()
            .find(|result| result.case_id == "missed_defense_closed_four")
            .expect("missed defense fixture should be present");
        assert!(missed_defense.passed);
        assert_eq!(missed_defense.actual.root_cause, RootCause::MissedDefense);
        assert_eq!(
            missed_defense.actual.final_forced_interval,
            ForcedInterval {
                start_ply: 8,
                end_ply: 9,
            }
        );
        assert!(missed_defense.proof_rows.iter().any(|row| row.ply == 7
            && row.status == ProofStatus::EscapeFound
            && row.escape_moves == vec![mv("L8")]));
    }

    #[test]
    fn analysis_fixture_report_serializes_as_stable_json() {
        let report = run_analysis_fixtures(AnalysisOptions::default())
            .expect("curated analysis fixtures should run");
        let json = serde_json::to_string_pretty(&report)
            .expect("analysis fixture report should serialize");

        assert!(json.contains("\"schema_version\": 2"));
        assert!(json.contains("\"case_id\": \"missed_defense_closed_four\""));
        assert!(json.contains("\"expected\""));
        assert!(json.contains("\"actual\""));
        assert!(json.contains("\"proof_rows\""));
    }

    #[test]
    fn analysis_fixture_report_renders_standalone_html() {
        let report = run_analysis_fixtures(AnalysisOptions::default())
            .expect("curated analysis fixtures should run");
        let html = render_analysis_fixture_report_html(&report);

        assert!(html.contains("<title>Gomoku2D Analysis Fixture Report</title>"));
        assert!(html.contains("6 passed / 6 total"));
        assert!(html.contains("missed_defense_closed_four"));
        assert!(html.contains("Expected"));
        assert!(html.contains("Proof Rows"));
    }
}
