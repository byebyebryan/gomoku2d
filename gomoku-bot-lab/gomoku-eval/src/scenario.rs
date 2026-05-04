use gomoku_bot::{Bot, SearchBot, SearchBotConfig};
use gomoku_core::{Color, Move, Variant};
use serde::Serialize;
use std::time::Instant;

#[path = "../../benchmarks/scenarios.rs"]
mod benchmark_scenarios;

use benchmark_scenarios::{parse_move, BenchScenario, SCENARIOS};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TacticalScenarioRole {
    HardSafetyGate,
    Diagnostic,
}

impl TacticalScenarioRole {
    const fn as_str(self) -> &'static str {
        match self {
            TacticalScenarioRole::HardSafetyGate => "hard_safety_gate",
            TacticalScenarioRole::Diagnostic => "diagnostic",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TacticalScenarioLayer {
    Local,
    Priority,
    Combo,
}

impl TacticalScenarioLayer {
    const fn as_str(self) -> &'static str {
        match self {
            TacticalScenarioLayer::Local => "local",
            TacticalScenarioLayer::Priority => "priority",
            TacticalScenarioLayer::Combo => "combo",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TacticalScenarioIntent {
    Complete,
    Create,
    React,
    Prevent,
    Counter,
    DoubleThreat,
}

impl TacticalScenarioIntent {
    const fn as_str(self) -> &'static str {
        match self {
            TacticalScenarioIntent::Complete => "complete",
            TacticalScenarioIntent::Create => "create",
            TacticalScenarioIntent::React => "react",
            TacticalScenarioIntent::Prevent => "prevent",
            TacticalScenarioIntent::Counter => "counter",
            TacticalScenarioIntent::DoubleThreat => "double_threat",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TacticalScenarioShape {
    OpenFour,
    ClosedFour,
    BrokenFour,
    OpenThree,
    ClosedThree,
    BrokenThree,
}

impl TacticalScenarioShape {
    const fn as_str(self) -> &'static str {
        match self {
            TacticalScenarioShape::OpenFour => "open_four",
            TacticalScenarioShape::ClosedFour => "closed_four",
            TacticalScenarioShape::BrokenFour => "broken_four",
            TacticalScenarioShape::OpenThree => "open_three",
            TacticalScenarioShape::ClosedThree => "closed_three",
            TacticalScenarioShape::BrokenThree => "broken_three",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TacticalScenarioOutcome {
    PassedExpectedMove,
    FailedExpectedMove,
}

impl TacticalScenarioOutcome {
    const fn as_str(self) -> &'static str {
        match self {
            TacticalScenarioOutcome::PassedExpectedMove => "passed_expected_move",
            TacticalScenarioOutcome::FailedExpectedMove => "failed_expected_move",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TacticalScenarioCase {
    pub id: &'static str,
    pub scenario_id: &'static str,
    pub category: &'static str,
    pub role: TacticalScenarioRole,
    pub layer: TacticalScenarioLayer,
    pub intent: TacticalScenarioIntent,
    pub shape: Option<TacticalScenarioShape>,
    pub description: &'static str,
    pub expected_moves: &'static [&'static str],
}

impl TacticalScenarioCase {
    fn scenario(&self) -> &'static BenchScenario {
        SCENARIOS
            .iter()
            .find(|scenario| scenario.id == self.scenario_id)
            .unwrap_or_else(|| {
                panic!(
                    "tactical case '{}' references unknown scenario '{}'",
                    self.id, self.scenario_id
                )
            })
    }

    fn expected_move_set(&self) -> Vec<Move> {
        self.expected_moves
            .iter()
            .copied()
            .map(parse_move)
            .collect()
    }
}

pub static TACTICAL_SCENARIO_CASES: &[TacticalScenarioCase] = &[
    TacticalScenarioCase {
        id: "local_complete_open_four",
        scenario_id: "local_complete_open_four",
        category: "local_complete_open_four",
        role: TacticalScenarioRole::HardSafetyGate,
        layer: TacticalScenarioLayer::Local,
        intent: TacticalScenarioIntent::Complete,
        shape: Some(TacticalScenarioShape::OpenFour),
        description: "Current player should complete an existing open four.",
        expected_moves: &["G8", "L8"],
    },
    TacticalScenarioCase {
        id: "local_react_closed_four",
        scenario_id: "local_react_closed_four",
        category: "local_react_closed_four",
        role: TacticalScenarioRole::HardSafetyGate,
        layer: TacticalScenarioLayer::Local,
        intent: TacticalScenarioIntent::React,
        shape: Some(TacticalScenarioShape::ClosedFour),
        description: "Current player should answer the only completion square of the opponent's closed four.",
        expected_moves: &["E1"],
    },
    TacticalScenarioCase {
        id: "priority_complete_open_four_over_react_closed_four",
        scenario_id: "priority_complete_open_four_over_react_closed_four",
        category: "priority_complete_open_four_over_react_closed_four",
        role: TacticalScenarioRole::HardSafetyGate,
        layer: TacticalScenarioLayer::Priority,
        intent: TacticalScenarioIntent::Complete,
        shape: Some(TacticalScenarioShape::OpenFour),
        description: "Current player should complete its open four instead of reacting to the opponent's closed four.",
        expected_moves: &["G8", "L8"],
    },
    TacticalScenarioCase {
        id: "priority_prevent_open_four_over_extend_three",
        scenario_id: "priority_prevent_open_four_over_extend_three",
        category: "priority_prevent_open_four_over_extend_three",
        role: TacticalScenarioRole::HardSafetyGate,
        layer: TacticalScenarioLayer::Priority,
        intent: TacticalScenarioIntent::Prevent,
        shape: Some(TacticalScenarioShape::OpenThree),
        description: "Current player should prevent the opponent's open three from becoming an open four instead of extending its own weaker line.",
        expected_moves: &["G8", "K8"],
    },
    TacticalScenarioCase {
        id: "priority_create_open_four_over_prevent_open_three",
        scenario_id: "priority_create_open_four_over_prevent_open_three",
        category: "priority_create_open_four_over_prevent_open_three",
        role: TacticalScenarioRole::Diagnostic,
        layer: TacticalScenarioLayer::Priority,
        intent: TacticalScenarioIntent::Counter,
        shape: Some(TacticalScenarioShape::OpenFour),
        description: "Current player can create an open four, so it may counter-threat instead of blocking the opponent's open three.",
        expected_moves: &["B4", "F4"],
    },
    TacticalScenarioCase {
        id: "local_create_open_four",
        scenario_id: "local_create_open_four",
        category: "local_create_open_four",
        role: TacticalScenarioRole::Diagnostic,
        layer: TacticalScenarioLayer::Local,
        intent: TacticalScenarioIntent::Create,
        shape: Some(TacticalScenarioShape::OpenFour),
        description: "Current player should create an open four.",
        expected_moves: &["G8", "K8"],
    },
    TacticalScenarioCase {
        id: "local_create_closed_four",
        scenario_id: "local_create_closed_four",
        category: "local_create_closed_four",
        role: TacticalScenarioRole::Diagnostic,
        layer: TacticalScenarioLayer::Local,
        intent: TacticalScenarioIntent::Create,
        shape: Some(TacticalScenarioShape::ClosedFour),
        description: "Current player should create a closed four.",
        expected_moves: &["K8"],
    },
    TacticalScenarioCase {
        id: "local_create_broken_four",
        scenario_id: "local_create_broken_four",
        category: "local_create_broken_four",
        role: TacticalScenarioRole::Diagnostic,
        layer: TacticalScenarioLayer::Local,
        intent: TacticalScenarioIntent::Create,
        shape: Some(TacticalScenarioShape::BrokenFour),
        description: "Current player should create a broken four.",
        expected_moves: &["J8", "K8"],
    },
    TacticalScenarioCase {
        id: "local_react_broken_four",
        scenario_id: "local_react_broken_four",
        category: "local_react_broken_four",
        role: TacticalScenarioRole::Diagnostic,
        layer: TacticalScenarioLayer::Local,
        intent: TacticalScenarioIntent::React,
        shape: Some(TacticalScenarioShape::BrokenFour),
        description: "Current player should answer the internal completion square of the opponent's broken four.",
        expected_moves: &["K8"],
    },
    TacticalScenarioCase {
        id: "local_create_open_three",
        scenario_id: "local_create_open_three",
        category: "local_create_open_three",
        role: TacticalScenarioRole::Diagnostic,
        layer: TacticalScenarioLayer::Local,
        intent: TacticalScenarioIntent::Create,
        shape: Some(TacticalScenarioShape::OpenThree),
        description: "Current player should create an open three.",
        expected_moves: &["G8", "J8"],
    },
    TacticalScenarioCase {
        id: "local_prevent_open_four_from_open_three",
        scenario_id: "local_prevent_open_four_from_open_three",
        category: "local_prevent_open_four_from_open_three",
        role: TacticalScenarioRole::Diagnostic,
        layer: TacticalScenarioLayer::Local,
        intent: TacticalScenarioIntent::Prevent,
        shape: Some(TacticalScenarioShape::OpenThree),
        description: "Current player should prevent the opponent's open three from becoming an open four.",
        expected_moves: &["G8", "K8"],
    },
    TacticalScenarioCase {
        id: "local_create_closed_three",
        scenario_id: "local_create_closed_three",
        category: "local_create_closed_three",
        role: TacticalScenarioRole::Diagnostic,
        layer: TacticalScenarioLayer::Local,
        intent: TacticalScenarioIntent::Create,
        shape: Some(TacticalScenarioShape::ClosedThree),
        description: "Current player should create a closed three.",
        expected_moves: &["J8"],
    },
    TacticalScenarioCase {
        id: "local_prevent_closed_four_from_closed_three",
        scenario_id: "local_prevent_closed_four_from_closed_three",
        category: "local_prevent_closed_four_from_closed_three",
        role: TacticalScenarioRole::Diagnostic,
        layer: TacticalScenarioLayer::Local,
        intent: TacticalScenarioIntent::Prevent,
        shape: Some(TacticalScenarioShape::ClosedThree),
        description: "Current player should prevent the opponent's closed three from becoming a closed four.",
        expected_moves: &["K8"],
    },
    TacticalScenarioCase {
        id: "local_create_broken_three",
        scenario_id: "local_create_broken_three",
        category: "local_create_broken_three",
        role: TacticalScenarioRole::Diagnostic,
        layer: TacticalScenarioLayer::Local,
        intent: TacticalScenarioIntent::Create,
        shape: Some(TacticalScenarioShape::BrokenThree),
        description: "Current player should create a broken three.",
        expected_moves: &["I8", "J8"],
    },
    TacticalScenarioCase {
        id: "local_prevent_broken_four_from_broken_three",
        scenario_id: "local_prevent_broken_four_from_broken_three",
        category: "local_prevent_broken_four_from_broken_three",
        role: TacticalScenarioRole::Diagnostic,
        layer: TacticalScenarioLayer::Local,
        intent: TacticalScenarioIntent::Prevent,
        shape: Some(TacticalScenarioShape::BrokenThree),
        description: "Current player should prevent the opponent's broken three from becoming a broken four.",
        expected_moves: &["I8"],
    },
    TacticalScenarioCase {
        id: "combo_create_double_threat",
        scenario_id: "combo_create_double_threat",
        category: "combo_create_double_threat",
        role: TacticalScenarioRole::Diagnostic,
        layer: TacticalScenarioLayer::Combo,
        intent: TacticalScenarioIntent::DoubleThreat,
        shape: None,
        description: "Current player should create simultaneous immediate winning threats.",
        expected_moves: &["J8"],
    },
];

#[derive(Debug, Clone)]
pub struct ScenarioSearchConfig {
    pub id: String,
    pub config: SearchBotConfig,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ScenarioSearchMetrics {
    pub time_ms: u64,
    pub depth_reached: i32,
    pub nodes: u64,
    pub safety_nodes: u64,
    pub total_nodes: u64,
    pub eval_calls: u64,
    pub candidate_generations: u64,
    pub candidate_moves_total: u64,
    pub candidate_moves_max: u64,
    pub root_candidate_generations: u64,
    pub root_candidate_moves_total: u64,
    pub root_candidate_moves_max: u64,
    pub search_candidate_generations: u64,
    pub search_candidate_moves_total: u64,
    pub search_candidate_moves_max: u64,
    pub legality_checks: u64,
    pub illegal_moves_skipped: u64,
    pub root_legality_checks: u64,
    pub root_illegal_moves_skipped: u64,
    pub search_legality_checks: u64,
    pub search_illegal_moves_skipped: u64,
    pub tt_hits: u64,
    pub tt_cutoffs: u64,
    pub beta_cutoffs: u64,
    pub score: i32,
    pub budget_exhausted: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TacticalScenarioResult {
    pub case_id: &'static str,
    pub scenario_id: &'static str,
    pub category: &'static str,
    pub role: &'static str,
    pub layer: &'static str,
    pub intent: &'static str,
    pub shape: Option<&'static str>,
    pub variant: Variant,
    pub to_move: Color,
    pub config_id: String,
    pub description: &'static str,
    pub expected_moves: Vec<String>,
    pub actual_move: String,
    pub passed: bool,
    pub outcome: &'static str,
    pub metrics: ScenarioSearchMetrics,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TacticalScenarioGroupSummary {
    pub key: String,
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub budget_exhausted: usize,
    pub avg_time_ms: f64,
    pub avg_depth_reached: f64,
    pub avg_total_nodes: f64,
    pub avg_safety_nodes: f64,
    pub avg_candidate_moves_total: f64,
    pub avg_legality_checks: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TacticalScenarioReport {
    pub schema_version: u32,
    pub configs: Vec<String>,
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub role_summaries: Vec<TacticalScenarioGroupSummary>,
    pub layer_summaries: Vec<TacticalScenarioGroupSummary>,
    pub intent_summaries: Vec<TacticalScenarioGroupSummary>,
    pub results: Vec<TacticalScenarioResult>,
}

impl TacticalScenarioReport {
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn run_tactical_case(
    case: &TacticalScenarioCase,
    config_id: impl Into<String>,
    config: SearchBotConfig,
) -> TacticalScenarioResult {
    let board = case.scenario().board();
    let expected_moves = case.expected_move_set();
    let mut bot = SearchBot::with_config(config);

    let start = Instant::now();
    let actual_move = bot.choose_move(&board);
    let time_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
    let info = bot
        .last_info
        .as_ref()
        .expect("SearchBot should record search info after choose_move");
    let passed = expected_moves.contains(&actual_move);
    let outcome = if passed {
        TacticalScenarioOutcome::PassedExpectedMove
    } else {
        TacticalScenarioOutcome::FailedExpectedMove
    };

    TacticalScenarioResult {
        case_id: case.id,
        scenario_id: case.scenario_id,
        category: case.category,
        role: case.role.as_str(),
        layer: case.layer.as_str(),
        intent: case.intent.as_str(),
        shape: case.shape.map(TacticalScenarioShape::as_str),
        variant: board.config.variant.clone(),
        to_move: board.current_player,
        config_id: config_id.into(),
        description: case.description,
        expected_moves: expected_moves
            .iter()
            .copied()
            .map(Move::to_notation)
            .collect(),
        actual_move: actual_move.to_notation(),
        passed,
        outcome: outcome.as_str(),
        metrics: ScenarioSearchMetrics {
            time_ms,
            depth_reached: info.depth_reached,
            nodes: info.nodes,
            safety_nodes: info.safety_nodes,
            total_nodes: info.nodes + info.safety_nodes,
            eval_calls: info.metrics.eval_calls,
            candidate_generations: info.metrics.candidate_generations,
            candidate_moves_total: info.metrics.candidate_moves_total,
            candidate_moves_max: info.metrics.candidate_moves_max,
            root_candidate_generations: info.metrics.root_candidate_generations,
            root_candidate_moves_total: info.metrics.root_candidate_moves_total,
            root_candidate_moves_max: info.metrics.root_candidate_moves_max,
            search_candidate_generations: info.metrics.search_candidate_generations,
            search_candidate_moves_total: info.metrics.search_candidate_moves_total,
            search_candidate_moves_max: info.metrics.search_candidate_moves_max,
            legality_checks: info.metrics.legality_checks,
            illegal_moves_skipped: info.metrics.illegal_moves_skipped,
            root_legality_checks: info.metrics.root_legality_checks,
            root_illegal_moves_skipped: info.metrics.root_illegal_moves_skipped,
            search_legality_checks: info.metrics.search_legality_checks,
            search_illegal_moves_skipped: info.metrics.search_illegal_moves_skipped,
            tt_hits: info.metrics.tt_hits,
            tt_cutoffs: info.metrics.tt_cutoffs,
            beta_cutoffs: info.metrics.beta_cutoffs,
            score: info.score,
            budget_exhausted: info.budget_exhausted,
        },
    }
}

fn summarize_by(
    results: &[TacticalScenarioResult],
    key_for: impl Fn(&TacticalScenarioResult) -> &'static str,
) -> Vec<TacticalScenarioGroupSummary> {
    let mut grouped: Vec<(String, Vec<&TacticalScenarioResult>)> = Vec::new();
    for result in results {
        let key = key_for(result);
        if let Some((_, group)) = grouped
            .iter_mut()
            .find(|(existing_key, _)| existing_key == key)
        {
            group.push(result);
        } else {
            grouped.push((key.to_string(), vec![result]));
        }
    }

    grouped
        .into_iter()
        .map(|(key, group)| summarize_group(key, &group))
        .collect()
}

fn summarize_group(
    key: String,
    results: &[&TacticalScenarioResult],
) -> TacticalScenarioGroupSummary {
    let total = results.len();
    let total_f64 = total as f64;
    let passed = results.iter().filter(|result| result.passed).count();
    let budget_exhausted = results
        .iter()
        .filter(|result| result.metrics.budget_exhausted)
        .count();

    TacticalScenarioGroupSummary {
        key,
        total,
        passed,
        failed: total - passed,
        budget_exhausted,
        avg_time_ms: results
            .iter()
            .map(|result| result.metrics.time_ms as f64)
            .sum::<f64>()
            / total_f64,
        avg_depth_reached: results
            .iter()
            .map(|result| f64::from(result.metrics.depth_reached))
            .sum::<f64>()
            / total_f64,
        avg_total_nodes: results
            .iter()
            .map(|result| result.metrics.total_nodes as f64)
            .sum::<f64>()
            / total_f64,
        avg_safety_nodes: results
            .iter()
            .map(|result| result.metrics.safety_nodes as f64)
            .sum::<f64>()
            / total_f64,
        avg_candidate_moves_total: results
            .iter()
            .map(|result| result.metrics.candidate_moves_total as f64)
            .sum::<f64>()
            / total_f64,
        avg_legality_checks: results
            .iter()
            .map(|result| result.metrics.legality_checks as f64)
            .sum::<f64>()
            / total_f64,
    }
}

pub fn run_tactical_scenarios(
    configs: &[ScenarioSearchConfig],
    cases: &[TacticalScenarioCase],
) -> TacticalScenarioReport {
    let mut results = Vec::with_capacity(configs.len() * cases.len());
    for config in configs {
        for case in cases {
            results.push(run_tactical_case(case, &config.id, config.config));
        }
    }

    let passed = results.iter().filter(|result| result.passed).count();
    let total = results.len();

    TacticalScenarioReport {
        schema_version: 3,
        configs: configs.iter().map(|config| config.id.clone()).collect(),
        total,
        passed,
        failed: total - passed,
        role_summaries: summarize_by(&results, |result| result.role),
        layer_summaries: summarize_by(&results, |result| result.layer),
        intent_summaries: summarize_by(&results, |result| result.intent),
        results,
    }
}

#[cfg(test)]
mod tests {
    use gomoku_bot::SearchBotConfig;
    use gomoku_core::{Board, GameResult, Move, Variant};

    use super::{
        run_tactical_case, run_tactical_scenarios, ScenarioSearchConfig, TacticalScenarioIntent,
        TacticalScenarioLayer, TacticalScenarioRole, TacticalScenarioShape,
        TACTICAL_SCENARIO_CASES,
    };

    #[test]
    fn tactical_case_result_records_expected_move_and_search_metrics() {
        let case = TACTICAL_SCENARIO_CASES
            .iter()
            .find(|case| case.id == "local_complete_open_four")
            .expect("expected tactical case");

        let result = run_tactical_case(case, "search-d3", SearchBotConfig::custom_depth(3));

        assert!(result.passed);
        assert_eq!(result.case_id, "local_complete_open_four");
        assert_eq!(result.config_id, "search-d3");
        assert_eq!(result.role, "hard_safety_gate");
        assert_eq!(result.layer, "local");
        assert_eq!(result.intent, "complete");
        assert_eq!(result.shape, Some("open_four"));
        assert_eq!(result.outcome, "passed_expected_move");
        assert_eq!(result.variant, Variant::Freestyle);
        assert!(result.expected_moves.contains(&result.actual_move));
        assert!(result.metrics.nodes > 0);
        assert!(result.metrics.depth_reached >= 1);
        assert_eq!(
            result.metrics.candidate_generations,
            result.metrics.root_candidate_generations + result.metrics.search_candidate_generations
        );
        assert_eq!(
            result.metrics.legality_checks,
            result.metrics.root_legality_checks + result.metrics.search_legality_checks
        );
    }

    #[test]
    fn tactical_report_runs_each_case_for_each_config() {
        let configs = [
            ScenarioSearchConfig {
                id: "search-d2".to_string(),
                config: SearchBotConfig::custom_depth(2),
            },
            ScenarioSearchConfig {
                id: "search-d3".to_string(),
                config: SearchBotConfig::custom_depth(3),
            },
        ];
        let cases = &TACTICAL_SCENARIO_CASES[..2];

        let report = run_tactical_scenarios(&configs, cases);

        assert_eq!(report.schema_version, 3);
        assert_eq!(report.configs, vec!["search-d2", "search-d3"]);
        assert_eq!(report.results.len(), configs.len() * cases.len());
        assert_eq!(report.total, 4);
        assert_eq!(report.passed + report.failed, report.total);
        assert_eq!(report.role_summaries.len(), 1);
        assert_eq!(report.role_summaries[0].key, "hard_safety_gate");
        assert_eq!(report.role_summaries[0].total, 4);
        assert!(report.role_summaries[0].avg_candidate_moves_total > 0.0);
        assert_eq!(report.layer_summaries.len(), 1);
        assert_eq!(report.layer_summaries[0].key, "local");
        assert_eq!(report.intent_summaries.len(), 2);
        assert!(report
            .intent_summaries
            .iter()
            .any(|summary| summary.key == "complete" && summary.total == 2));
        assert!(report
            .intent_summaries
            .iter()
            .any(|summary| summary.key == "react" && summary.total == 2));
    }

    #[test]
    fn tactical_cases_keep_renju_legality_out_of_active_hard_gates() {
        assert!(
            TACTICAL_SCENARIO_CASES
                .iter()
                .filter(|case| case.role == TacticalScenarioRole::HardSafetyGate)
                .all(|case| case.scenario().variant == Variant::Freestyle),
            "active tactical hard gates should not use Renju legality-only cases as tactical gates"
        );
        assert!(
            TACTICAL_SCENARIO_CASES.iter().any(|case| {
                case.id == "priority_create_open_four_over_prevent_open_three"
                    && case.category == "priority_create_open_four_over_prevent_open_three"
                    && case.role == TacticalScenarioRole::Diagnostic
            }),
            "diagnostic corpus should include a counter-threat case where creating a four can defer blocking an open three"
        );
    }

    #[test]
    fn tactical_cases_declare_explicit_eval_metadata() {
        let expected = [
            (
                "local_complete_open_four",
                TacticalScenarioLayer::Local,
                TacticalScenarioIntent::Complete,
                Some(TacticalScenarioShape::OpenFour),
            ),
            (
                "local_react_closed_four",
                TacticalScenarioLayer::Local,
                TacticalScenarioIntent::React,
                Some(TacticalScenarioShape::ClosedFour),
            ),
            (
                "priority_complete_open_four_over_react_closed_four",
                TacticalScenarioLayer::Priority,
                TacticalScenarioIntent::Complete,
                Some(TacticalScenarioShape::OpenFour),
            ),
            (
                "priority_prevent_open_four_over_extend_three",
                TacticalScenarioLayer::Priority,
                TacticalScenarioIntent::Prevent,
                Some(TacticalScenarioShape::OpenThree),
            ),
            (
                "priority_create_open_four_over_prevent_open_three",
                TacticalScenarioLayer::Priority,
                TacticalScenarioIntent::Counter,
                Some(TacticalScenarioShape::OpenFour),
            ),
            (
                "local_create_open_four",
                TacticalScenarioLayer::Local,
                TacticalScenarioIntent::Create,
                Some(TacticalScenarioShape::OpenFour),
            ),
            (
                "local_create_closed_four",
                TacticalScenarioLayer::Local,
                TacticalScenarioIntent::Create,
                Some(TacticalScenarioShape::ClosedFour),
            ),
            (
                "local_create_broken_four",
                TacticalScenarioLayer::Local,
                TacticalScenarioIntent::Create,
                Some(TacticalScenarioShape::BrokenFour),
            ),
            (
                "local_react_broken_four",
                TacticalScenarioLayer::Local,
                TacticalScenarioIntent::React,
                Some(TacticalScenarioShape::BrokenFour),
            ),
            (
                "local_create_open_three",
                TacticalScenarioLayer::Local,
                TacticalScenarioIntent::Create,
                Some(TacticalScenarioShape::OpenThree),
            ),
            (
                "local_prevent_open_four_from_open_three",
                TacticalScenarioLayer::Local,
                TacticalScenarioIntent::Prevent,
                Some(TacticalScenarioShape::OpenThree),
            ),
            (
                "local_create_closed_three",
                TacticalScenarioLayer::Local,
                TacticalScenarioIntent::Create,
                Some(TacticalScenarioShape::ClosedThree),
            ),
            (
                "local_prevent_closed_four_from_closed_three",
                TacticalScenarioLayer::Local,
                TacticalScenarioIntent::Prevent,
                Some(TacticalScenarioShape::ClosedThree),
            ),
            (
                "local_create_broken_three",
                TacticalScenarioLayer::Local,
                TacticalScenarioIntent::Create,
                Some(TacticalScenarioShape::BrokenThree),
            ),
            (
                "local_prevent_broken_four_from_broken_three",
                TacticalScenarioLayer::Local,
                TacticalScenarioIntent::Prevent,
                Some(TacticalScenarioShape::BrokenThree),
            ),
            (
                "combo_create_double_threat",
                TacticalScenarioLayer::Combo,
                TacticalScenarioIntent::DoubleThreat,
                None,
            ),
        ];

        assert_eq!(TACTICAL_SCENARIO_CASES.len(), expected.len());
        for (id, layer, intent, shape) in expected {
            let case = TACTICAL_SCENARIO_CASES
                .iter()
                .find(|case| case.id == id)
                .expect("expected tactical case");
            assert_eq!(case.layer, layer, "case '{id}' layer");
            assert_eq!(case.intent, intent, "case '{id}' intent");
            assert_eq!(case.shape, shape, "case '{id}' shape");
            assert!(
                case.category.starts_with(case.layer.as_str()),
                "case '{id}' category '{}' should start with layer '{}'",
                case.category,
                case.layer.as_str()
            );
        }
    }

    #[test]
    fn tactical_cases_use_layered_local_priority_combo_taxonomy() {
        for category in [
            "local_complete_open_four",
            "local_create_open_four",
            "local_create_closed_four",
            "local_react_closed_four",
            "local_create_broken_four",
            "local_react_broken_four",
            "local_create_open_three",
            "local_prevent_open_four_from_open_three",
            "local_create_closed_three",
            "local_prevent_closed_four_from_closed_three",
            "local_create_broken_three",
            "local_prevent_broken_four_from_broken_three",
            "priority_complete_open_four_over_react_closed_four",
            "priority_prevent_open_four_over_extend_three",
            "priority_create_open_four_over_prevent_open_three",
            "combo_create_double_threat",
        ] {
            assert!(
                TACTICAL_SCENARIO_CASES
                    .iter()
                    .any(|case| case.category == category),
                "missing tactical case category for {category}"
            );
        }

        for case in TACTICAL_SCENARIO_CASES {
            assert!(
                case.category.starts_with("local_")
                    || case.category.starts_with("priority_")
                    || case.category.starts_with("combo_"),
                "case '{}' should use local_/priority_/combo_ category, got '{}'",
                case.id,
                case.category
            );
            assert!(
                !case.id.starts_with("shape_")
                    && !case.category.starts_with("shape_")
                    && !matches!(
                        case.id,
                        "take_immediate_win"
                            | "block_immediate_loss"
                            | "win_race_before_blocking"
                            | "prevent_open_three_reply"
                            | "create_open_four"
                            | "create_broken_three"
                            | "create_double_threat"
                    ),
                "case '{}' should be named by the layered tactical taxonomy",
                case.id
            );
        }
    }

    #[test]
    fn tactical_case_expected_moves_match_declared_semantics() {
        for case in TACTICAL_SCENARIO_CASES {
            let board = case.scenario().board();
            let expected_moves = case.expected_move_set();
            assert!(
                !expected_moves.is_empty(),
                "case '{}' must define at least one expected move",
                case.id
            );
            for &mv in &expected_moves {
                assert!(
                    board.is_legal(mv),
                    "case '{}' expected move {} must be legal",
                    case.id,
                    mv.to_notation()
                );
            }

            if let Some(shape) = case.category.strip_prefix("local_create_") {
                assert_local_create(case.id, shape, &board, &expected_moves);
                continue;
            }
            if let Some(shape) = case.category.strip_prefix("local_react_") {
                assert_local_react(case.id, shape, &board, &expected_moves);
                continue;
            }
            if case.category.starts_with("local_prevent_") {
                assert_local_prevent(case.id, case.category, &board, &expected_moves);
                continue;
            }

            match case.category {
                "local_complete_open_four" => {
                    let wins = board.immediate_winning_moves_for(board.current_player);
                    assert_contains_all(case.id, &wins, &expected_moves);
                }
                "priority_complete_open_four_over_react_closed_four" => {
                    let wins = board.immediate_winning_moves_for(board.current_player);
                    let opponent_wins =
                        board.immediate_winning_moves_for(board.current_player.opponent());
                    assert!(
                        !wins.is_empty() && !opponent_wins.is_empty(),
                        "case '{}' must contain wins for both sides",
                        case.id
                    );
                    assert_contains_all(case.id, &wins, &expected_moves);
                }
                "priority_prevent_open_four_over_extend_three" => {
                    let opponent_forcing_replies = opponent_forcing_replies_now(&board);
                    assert!(
                        !opponent_forcing_replies.is_empty(),
                        "case '{}' should start with an opponent open-three style forcing reply",
                        case.id
                    );
                    for &mv in &expected_moves {
                        assert!(
                            opponent_forcing_replies_after(&board, mv).is_empty(),
                            "case '{}' expected move {} should prevent opponent forcing replies",
                            case.id,
                            mv.to_notation()
                        );
                    }
                }
                "priority_create_open_four_over_prevent_open_three" => {
                    let opponent_forcing_replies = opponent_forcing_replies_now(&board);
                    assert!(
                        !opponent_forcing_replies.is_empty(),
                        "case '{}' should start with an opponent open-three style forcing reply",
                        case.id
                    );
                    for &mv in &expected_moves {
                        let own_replies = own_immediate_replies_after(&board, mv);
                        assert!(
                            own_replies.len() >= 2,
                            "case '{}' expected move {} should create an open four before blocking, got {:?}",
                            case.id,
                            mv.to_notation(),
                            own_replies
                        );
                    }
                }
                "combo_create_double_threat" => {
                    for &mv in &expected_moves {
                        let own_replies = own_immediate_replies_after(&board, mv);
                        assert!(
                            own_replies.len() >= 2,
                            "case '{}' expected move {} should create at least two immediate replies, got {:?}",
                            case.id,
                            mv.to_notation(),
                            own_replies
                        );
                    }
                }
                other => panic!("case '{}' has unvalidated category '{}'", case.id, other),
            }
        }
    }

    fn assert_local_create(case_id: &str, shape: &str, board: &Board, expected_moves: &[Move]) {
        match shape {
            "open_four" => {
                for &mv in expected_moves {
                    let own_replies = own_immediate_replies_after(board, mv);
                    assert!(
                        own_replies.len() >= 2,
                        "case '{case_id}' expected {} to create an open four, got {:?}",
                        mv.to_notation(),
                        own_replies
                    );
                }
            }
            "closed_four" | "broken_four" => {
                for &mv in expected_moves {
                    let own_replies = own_immediate_replies_after(board, mv);
                    assert_eq!(
                        own_replies.len(),
                        1,
                        "case '{case_id}' expected {} to create one immediate completion, got {:?}",
                        mv.to_notation(),
                        own_replies
                    );
                }
            }
            "open_three" => {
                for &mv in expected_moves {
                    let next_threats = threat_creating_replies_after(board, mv);
                    assert!(
                        next_threats.len() >= 2,
                        "case '{case_id}' expected {} to create an open three, got continuations {:?}",
                        mv.to_notation(),
                        next_threats
                    );
                }
            }
            "closed_three" | "broken_three" => {
                for &mv in expected_moves {
                    let next_threats = threat_creating_replies_after(board, mv);
                    assert!(
                        !next_threats.is_empty(),
                        "case '{case_id}' expected {} to create at least one continuation",
                        mv.to_notation()
                    );
                }
            }
            other => panic!("case '{case_id}' has unknown local create shape '{other}'"),
        }
    }

    fn assert_local_react(case_id: &str, shape: &str, board: &Board, expected_moves: &[Move]) {
        match shape {
            "closed_four" | "broken_four" => {
                let opponent_wins =
                    board.immediate_winning_moves_for(board.current_player.opponent());
                assert_contains_all(case_id, &opponent_wins, expected_moves);
            }
            other => panic!("case '{case_id}' has unknown local react shape '{other}'"),
        }
    }

    fn assert_local_prevent(case_id: &str, category: &str, board: &Board, expected_moves: &[Move]) {
        match category {
            "local_prevent_open_four_from_open_three"
            | "local_prevent_closed_four_from_closed_three"
            | "local_prevent_broken_four_from_broken_three" => {
                let opponent_continuations =
                    threat_creating_replies_for_player(board, board.current_player.opponent());
                assert_contains_all(case_id, &opponent_continuations, expected_moves);
            }
            other => panic!("case '{case_id}' has unknown local prevent category '{other}'"),
        }
    }

    fn assert_contains_all(case_id: &str, actual: &[Move], expected: &[Move]) {
        for &mv in expected {
            assert!(
                actual.contains(&mv),
                "case '{}' expected {} in {:?}",
                case_id,
                mv.to_notation(),
                actual
            );
        }
    }

    fn own_immediate_replies_after(board: &Board, mv: Move) -> Vec<Move> {
        let player = board.current_player;
        let mut next = board.clone();
        next.apply_move(mv).expect("expected legal move");
        next.immediate_winning_moves_for(player)
    }

    fn threat_creating_replies_after(board: &Board, mv: Move) -> Vec<Move> {
        let player = board.current_player;
        let mut next = board.clone();
        next.apply_move(mv).expect("expected legal move");
        threat_creating_replies_for_player(&next, player)
    }

    fn threat_creating_replies_for_player(board: &Board, player: gomoku_core::Color) -> Vec<Move> {
        let mut player_turn = board.clone();
        player_turn.current_player = player;

        let mut threats = Vec::new();
        for row in 0..player_turn.config.board_size {
            for col in 0..player_turn.config.board_size {
                let reply = Move { row, col };
                if !player_turn.is_legal(reply) {
                    continue;
                }

                let mut after_reply = player_turn.clone();
                let result = after_reply.apply_move(reply).expect("expected legal reply");
                if matches!(result, GameResult::Winner(winner) if winner == player)
                    || !after_reply.immediate_winning_moves_for(player).is_empty()
                {
                    threats.push(reply);
                }
            }
        }

        threats
    }

    fn opponent_forcing_replies_after(board: &Board, mv: Move) -> Vec<Move> {
        let mut next = board.clone();
        next.apply_move(mv).expect("expected legal move");

        forcing_replies_for_current_player(&next)
    }

    fn opponent_forcing_replies_now(board: &Board) -> Vec<Move> {
        let mut opponent_turn = board.clone();
        opponent_turn.current_player = board.current_player.opponent();
        forcing_replies_for_current_player(&opponent_turn)
    }

    fn forcing_replies_for_current_player(board: &Board) -> Vec<Move> {
        let player = board.current_player;
        let mut forcing = Vec::new();
        for row in 0..board.config.board_size {
            for col in 0..board.config.board_size {
                let reply = Move { row, col };
                if !board.is_legal(reply) {
                    continue;
                }

                let mut after_reply = board.clone();
                let result = after_reply.apply_move(reply).expect("expected legal reply");
                if matches!(result, GameResult::Winner(winner) if winner == player)
                    || after_reply.has_multiple_immediate_winning_moves_for(player)
                {
                    forcing.push(reply);
                }
            }
        }

        forcing
    }
}
