use gomoku_bot::{Bot, SearchBot, SearchBotConfig};
use gomoku_core::Move;
use serde::Serialize;
use std::time::Instant;

#[path = "../../benchmarks/scenarios.rs"]
mod benchmark_scenarios;

use benchmark_scenarios::{parse_move, BenchScenario, SCENARIOS};

#[derive(Debug, Clone, Copy)]
pub struct TacticalScenarioCase {
    pub id: &'static str,
    pub scenario_id: &'static str,
    pub category: &'static str,
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
        id: "take_immediate_win",
        scenario_id: "immediate_win",
        category: "immediate_win",
        description: "Current player should finish a direct four-in-a-row.",
        expected_moves: &["G8", "L8"],
    },
    TacticalScenarioCase {
        id: "block_immediate_loss",
        scenario_id: "immediate_block",
        category: "forced_block",
        description: "Current player should block the opponent's direct win.",
        expected_moves: &["E1"],
    },
    TacticalScenarioCase {
        id: "win_race_before_blocking",
        scenario_id: "attack_wins_race",
        category: "attack_vs_defense",
        description: "Current player should win immediately instead of blocking.",
        expected_moves: &["G8", "L8"],
    },
    TacticalScenarioCase {
        id: "block_open_three",
        scenario_id: "anti_blunder_open_three",
        category: "open_three",
        description: "Current player should block the forcing open three.",
        expected_moves: &["G8", "K8"],
    },
    TacticalScenarioCase {
        id: "create_open_four",
        scenario_id: "create_open_four",
        category: "open_four",
        description: "Current player should create an open four when no direct win exists.",
        expected_moves: &["G8", "K8"],
    },
    TacticalScenarioCase {
        id: "create_broken_three",
        scenario_id: "create_broken_three",
        category: "broken_three",
        description: "Current player should create a broken three shape from a spaced pair.",
        expected_moves: &["I8", "J8"],
    },
    TacticalScenarioCase {
        id: "create_double_threat",
        scenario_id: "create_double_threat",
        category: "double_threat",
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
    pub config_id: String,
    pub description: &'static str,
    pub expected_moves: Vec<String>,
    pub actual_move: String,
    pub passed: bool,
    pub metrics: ScenarioSearchMetrics,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TacticalScenarioReport {
    pub schema_version: u32,
    pub configs: Vec<String>,
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
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

    TacticalScenarioResult {
        case_id: case.id,
        scenario_id: case.scenario_id,
        category: case.category,
        config_id: config_id.into(),
        description: case.description,
        expected_moves: expected_moves
            .iter()
            .copied()
            .map(Move::to_notation)
            .collect(),
        actual_move: actual_move.to_notation(),
        passed: expected_moves.contains(&actual_move),
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
        schema_version: 1,
        configs: configs.iter().map(|config| config.id.clone()).collect(),
        total,
        passed,
        failed: total - passed,
        results,
    }
}

#[cfg(test)]
mod tests {
    use gomoku_bot::SearchBotConfig;

    use super::{
        run_tactical_case, run_tactical_scenarios, ScenarioSearchConfig, TACTICAL_SCENARIO_CASES,
    };

    #[test]
    fn tactical_case_result_records_expected_move_and_search_metrics() {
        let case = TACTICAL_SCENARIO_CASES
            .iter()
            .find(|case| case.id == "take_immediate_win")
            .expect("expected tactical case");

        let result = run_tactical_case(case, "search-d3", SearchBotConfig::custom_depth(3));

        assert!(result.passed);
        assert_eq!(result.case_id, "take_immediate_win");
        assert_eq!(result.config_id, "search-d3");
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

        assert_eq!(report.configs, vec!["search-d2", "search-d3"]);
        assert_eq!(report.results.len(), configs.len() * cases.len());
        assert_eq!(report.total, 4);
        assert_eq!(report.passed + report.failed, report.total);
    }
}
