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

#[derive(Debug, Clone, Copy)]
pub struct TacticalScenarioCase {
    pub id: &'static str,
    pub scenario_id: &'static str,
    pub category: &'static str,
    pub role: TacticalScenarioRole,
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
        role: TacticalScenarioRole::HardSafetyGate,
        description: "Current player should finish a direct four-in-a-row.",
        expected_moves: &["G8", "L8"],
    },
    TacticalScenarioCase {
        id: "block_immediate_loss",
        scenario_id: "immediate_block",
        category: "forced_block",
        role: TacticalScenarioRole::HardSafetyGate,
        description: "Current player should block the opponent's direct win.",
        expected_moves: &["E1"],
    },
    TacticalScenarioCase {
        id: "win_race_before_blocking",
        scenario_id: "attack_wins_race",
        category: "attack_vs_defense",
        role: TacticalScenarioRole::HardSafetyGate,
        description: "Current player should win immediately instead of blocking.",
        expected_moves: &["G8", "L8"],
    },
    TacticalScenarioCase {
        id: "prevent_open_three_reply",
        scenario_id: "anti_blunder_open_three",
        category: "open_three",
        role: TacticalScenarioRole::HardSafetyGate,
        description: "Current player should prevent the opponent's open-three reply from becoming an open-four threat.",
        expected_moves: &["G8", "K8"],
    },
    TacticalScenarioCase {
        id: "counter_open_three_with_four",
        scenario_id: "counter_open_three_with_four",
        category: "counter_four",
        role: TacticalScenarioRole::Diagnostic,
        description: "Current player can create an open four, so it may counter-threat instead of blocking the opponent's open three.",
        expected_moves: &["B4", "F4"],
    },
    TacticalScenarioCase {
        id: "create_open_four",
        scenario_id: "create_open_four",
        category: "open_four",
        role: TacticalScenarioRole::Diagnostic,
        description: "Current player should create an open four when no direct win exists.",
        expected_moves: &["G8", "K8"],
    },
    TacticalScenarioCase {
        id: "shape_offense_open_four",
        scenario_id: "shape_offense_open_four",
        category: "shape_offense_open_four",
        role: TacticalScenarioRole::Diagnostic,
        description: "Current player should create an open four.",
        expected_moves: &["G8", "K8"],
    },
    TacticalScenarioCase {
        id: "shape_defense_open_four",
        scenario_id: "shape_defense_open_four",
        category: "shape_defense_open_four",
        role: TacticalScenarioRole::Diagnostic,
        description: "Current player should occupy one completion square of the opponent's open four.",
        expected_moves: &["G8", "L8"],
    },
    TacticalScenarioCase {
        id: "shape_offense_closed_four",
        scenario_id: "shape_offense_closed_four",
        category: "shape_offense_closed_four",
        role: TacticalScenarioRole::Diagnostic,
        description: "Current player should create a closed four.",
        expected_moves: &["K8"],
    },
    TacticalScenarioCase {
        id: "shape_defense_closed_four",
        scenario_id: "shape_defense_closed_four",
        category: "shape_defense_closed_four",
        role: TacticalScenarioRole::Diagnostic,
        description: "Current player should answer the only completion square of the opponent's closed four.",
        expected_moves: &["L8"],
    },
    TacticalScenarioCase {
        id: "shape_offense_broken_four",
        scenario_id: "shape_offense_broken_four",
        category: "shape_offense_broken_four",
        role: TacticalScenarioRole::Diagnostic,
        description: "Current player should create a broken four.",
        expected_moves: &["J8", "K8"],
    },
    TacticalScenarioCase {
        id: "shape_defense_broken_four",
        scenario_id: "shape_defense_broken_four",
        category: "shape_defense_broken_four",
        role: TacticalScenarioRole::Diagnostic,
        description: "Current player should answer the internal completion square of the opponent's broken four.",
        expected_moves: &["K8"],
    },
    TacticalScenarioCase {
        id: "shape_offense_open_three",
        scenario_id: "shape_offense_open_three",
        category: "shape_offense_open_three",
        role: TacticalScenarioRole::Diagnostic,
        description: "Current player should create an open three.",
        expected_moves: &["G8", "J8"],
    },
    TacticalScenarioCase {
        id: "shape_defense_open_three",
        scenario_id: "shape_defense_open_three",
        category: "shape_defense_open_three",
        role: TacticalScenarioRole::Diagnostic,
        description: "Current player should occupy one extension square of the opponent's open three.",
        expected_moves: &["G8", "K8"],
    },
    TacticalScenarioCase {
        id: "shape_offense_closed_three",
        scenario_id: "shape_offense_closed_three",
        category: "shape_offense_closed_three",
        role: TacticalScenarioRole::Diagnostic,
        description: "Current player should create a closed three.",
        expected_moves: &["J8"],
    },
    TacticalScenarioCase {
        id: "shape_defense_closed_three",
        scenario_id: "shape_defense_closed_three",
        category: "shape_defense_closed_three",
        role: TacticalScenarioRole::Diagnostic,
        description: "Current player should occupy the only extension square of the opponent's closed three.",
        expected_moves: &["K8"],
    },
    TacticalScenarioCase {
        id: "shape_offense_broken_three",
        scenario_id: "shape_offense_broken_three",
        category: "shape_offense_broken_three",
        role: TacticalScenarioRole::Diagnostic,
        description: "Current player should create a broken three.",
        expected_moves: &["I8", "J8"],
    },
    TacticalScenarioCase {
        id: "shape_defense_broken_three",
        scenario_id: "shape_defense_broken_three",
        category: "shape_defense_broken_three",
        role: TacticalScenarioRole::Diagnostic,
        description: "Current player should occupy the rest square of the opponent's broken three.",
        expected_moves: &["I8"],
    },
    TacticalScenarioCase {
        id: "create_broken_three",
        scenario_id: "create_broken_three",
        category: "broken_three",
        role: TacticalScenarioRole::Diagnostic,
        description: "Current player should create a broken three shape from a spaced pair.",
        expected_moves: &["I8", "J8"],
    },
    TacticalScenarioCase {
        id: "create_double_threat",
        scenario_id: "create_double_threat",
        category: "double_threat",
        role: TacticalScenarioRole::Diagnostic,
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
    pub variant: Variant,
    pub to_move: Color,
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
        role: case.role.as_str(),
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
        schema_version: 2,
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
    use gomoku_core::{Board, GameResult, Move, Variant};

    use super::{
        run_tactical_case, run_tactical_scenarios, ScenarioSearchConfig, TacticalScenarioRole,
        TACTICAL_SCENARIO_CASES,
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
        assert_eq!(result.role, "hard_safety_gate");
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

        assert_eq!(report.schema_version, 2);
        assert_eq!(report.configs, vec!["search-d2", "search-d3"]);
        assert_eq!(report.results.len(), configs.len() * cases.len());
        assert_eq!(report.total, 4);
        assert_eq!(report.passed + report.failed, report.total);
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
                case.id == "counter_open_three_with_four"
                    && case.category == "counter_four"
                    && case.role == TacticalScenarioRole::Diagnostic
            }),
            "diagnostic corpus should include a counter-threat case where creating a four can defer blocking an open three"
        );
    }

    #[test]
    fn tactical_cases_include_shape_offense_defense_pairs() {
        for shape in [
            "open_four",
            "closed_four",
            "broken_four",
            "open_three",
            "closed_three",
            "broken_three",
        ] {
            for stance in ["offense", "defense"] {
                let expected_category = format!("shape_{stance}_{shape}");
                assert!(
                    TACTICAL_SCENARIO_CASES.iter().any(|case| {
                        case.category == expected_category
                            && case.role == TacticalScenarioRole::Diagnostic
                    }),
                    "missing diagnostic tactical shape case for {expected_category}"
                );
            }
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

            if let Some(shape) = case.category.strip_prefix("shape_offense_") {
                assert_shape_offense(case.id, shape, &board, &expected_moves);
                continue;
            }
            if let Some(shape) = case.category.strip_prefix("shape_defense_") {
                assert_shape_defense(case.id, shape, &board, &expected_moves);
                continue;
            }

            match case.category {
                "immediate_win" => {
                    let wins = board.immediate_winning_moves_for(board.current_player);
                    assert_contains_all(case.id, &wins, &expected_moves);
                }
                "forced_block" => {
                    let opponent_wins =
                        board.immediate_winning_moves_for(board.current_player.opponent());
                    assert_contains_all(case.id, &opponent_wins, &expected_moves);
                }
                "attack_vs_defense" => {
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
                "open_three" => {
                    for &mv in &expected_moves {
                        assert!(
                            opponent_forcing_replies_after(&board, mv).is_empty(),
                            "case '{}' expected move {} should prevent opponent forcing replies",
                            case.id,
                            mv.to_notation()
                        );
                    }
                }
                "counter_four" => {
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
                "open_four" | "double_threat" => {
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
                "broken_three" => {
                    assert!(
                        board
                            .immediate_winning_moves_for(board.current_player)
                            .is_empty(),
                        "case '{}' should not already have an immediate win",
                        case.id
                    );
                    assert!(
                        board
                            .immediate_winning_moves_for(board.current_player.opponent())
                            .is_empty(),
                        "case '{}' should not already require an immediate block",
                        case.id
                    );
                }
                other => panic!("case '{}' has unvalidated category '{}'", case.id, other),
            }
        }
    }

    fn assert_shape_offense(case_id: &str, shape: &str, board: &Board, expected_moves: &[Move]) {
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
            other => panic!("case '{case_id}' has unknown offense shape '{other}'"),
        }
    }

    fn assert_shape_defense(case_id: &str, shape: &str, board: &Board, expected_moves: &[Move]) {
        match shape {
            "open_four" | "closed_four" | "broken_four" => {
                let opponent_wins =
                    board.immediate_winning_moves_for(board.current_player.opponent());
                assert_contains_all(case_id, &opponent_wins, expected_moves);
            }
            "open_three" | "closed_three" | "broken_three" => {
                let opponent_continuations =
                    threat_creating_replies_for_player(board, board.current_player.opponent());
                assert_contains_all(case_id, &opponent_continuations, expected_moves);
            }
            other => panic!("case '{case_id}' has unknown defense shape '{other}'"),
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
