#![allow(dead_code)]

use gomoku_core::{Board, Color, Move, RuleConfig, Variant};

pub struct BenchScenario {
    pub id: &'static str,
    pub variant: Variant,
    pub to_move: Color,
    pub tags: &'static [&'static str],
    pub description: &'static str,
    pub probe_move: &'static str,
    pub moves: &'static [&'static str],
}

pub struct SearchBehaviorCase {
    pub id: &'static str,
    pub scenario_id: &'static str,
    pub config_id: &'static str,
    pub expected_moves: &'static [&'static str],
    pub description: &'static str,
}

#[allow(dead_code)]
impl BenchScenario {
    pub fn board(&self) -> Board {
        let mut board = Board::new(RuleConfig {
            variant: self.variant.clone(),
            ..RuleConfig::default()
        });

        for &mv in self.moves {
            board.apply_move(parse_move(mv)).unwrap_or_else(|err| {
                panic!("scenario '{}' failed to apply move {mv}: {err}", self.id)
            });
        }

        assert_eq!(
            board.current_player, self.to_move,
            "scenario '{}' current player drifted",
            self.id
        );
        board
    }

    pub fn probe_move(&self) -> Move {
        parse_move(self.probe_move)
    }
}

#[allow(dead_code)]
impl SearchBehaviorCase {
    pub fn scenario(&self) -> &'static BenchScenario {
        SCENARIOS
            .iter()
            .find(|scenario| scenario.id == self.scenario_id)
            .unwrap_or_else(|| {
                panic!(
                    "behavior case '{}' references unknown scenario '{}'",
                    self.id, self.scenario_id
                )
            })
    }

    pub fn expected_moves(&self) -> Vec<Move> {
        self.expected_moves
            .iter()
            .copied()
            .map(parse_move)
            .collect()
    }
}

pub fn parse_move(notation: &str) -> Move {
    Move::from_notation(notation)
        .unwrap_or_else(|err| panic!("invalid benchmark move '{notation}': {err}"))
}

pub static SCENARIOS: &[BenchScenario] = &[
    BenchScenario {
        id: "opening_sparse",
        variant: Variant::Freestyle,
        to_move: Color::Black,
        tags: &["opening", "sparse", "freestyle"],
        description: "Very early local opening around center; representative of the first few practice-bot turns.",
        probe_move: "H9",
        moves: &["H8", "H7", "G8", "I8"],
    },
    BenchScenario {
        id: "early_local_fight",
        variant: Variant::Freestyle,
        to_move: Color::Black,
        tags: &["opening", "local-fight", "freestyle"],
        description: "Compact early fight with short local tactical branches but no immediate forcing line.",
        probe_move: "H6",
        moves: &["H8", "I8", "H7", "G8", "I7", "G7", "H9", "I9"],
    },
    BenchScenario {
        id: "immediate_win",
        variant: Variant::Freestyle,
        to_move: Color::Black,
        tags: &["tactical", "immediate-win", "freestyle"],
        description: "Black has a direct winning move on the current turn.",
        probe_move: "G8",
        moves: &["H8", "A1", "I8", "C1", "J8", "E1", "K8", "G1"],
    },
    BenchScenario {
        id: "immediate_block",
        variant: Variant::Freestyle,
        to_move: Color::Black,
        tags: &["tactical", "immediate-block", "freestyle"],
        description: "Black must block White's direct horizontal win threat.",
        probe_move: "E1",
        moves: &["H8", "A1", "O1", "B1", "O2", "C1", "O3", "D1"],
    },
    BenchScenario {
        id: "attack_wins_race",
        variant: Variant::Freestyle,
        to_move: Color::Black,
        tags: &["tactical", "attack-vs-defense", "freestyle"],
        description: "Both players have an immediate win threat; Black should win now instead of blocking.",
        probe_move: "G8",
        moves: &["H8", "A1", "I8", "B1", "J8", "C1", "K8", "D1"],
    },
    BenchScenario {
        id: "anti_blunder_open_three",
        variant: Variant::Freestyle,
        to_move: Color::White,
        tags: &["tactical", "anti-blunder", "freestyle"],
        description: "White has a tempting diagonal extension, but the correct move is to block Black's open three.",
        probe_move: "G8",
        moves: &["H8", "D4", "I8", "F6", "J8"],
    },
    BenchScenario {
        id: "create_open_four",
        variant: Variant::Freestyle,
        to_move: Color::Black,
        tags: &["tactical", "open-four", "freestyle"],
        description: "Black can create an open four, forcing White to block one end.",
        probe_move: "K8",
        moves: &["H8", "A1", "I8", "C1", "J8", "E1"],
    },
    BenchScenario {
        id: "counter_open_three_with_four",
        variant: Variant::Freestyle,
        to_move: Color::White,
        tags: &["tactical", "counter-threat", "open-four", "freestyle"],
        description: "White can create an open four, so it can defer blocking Black's open three.",
        probe_move: "F4",
        moves: &["H8", "C4", "I8", "D4", "J8", "E4", "A15"],
    },
    BenchScenario {
        id: "create_broken_three",
        variant: Variant::Freestyle,
        to_move: Color::Black,
        tags: &["tactical", "broken-three", "freestyle"],
        description: "Black can connect a spaced pair into a broken three shape.",
        probe_move: "J8",
        moves: &["H8", "A1", "K8", "C1"],
    },
    BenchScenario {
        id: "create_double_threat",
        variant: Variant::Freestyle,
        to_move: Color::Black,
        tags: &["tactical", "double-threat", "freestyle"],
        description: "Black can create simultaneous horizontal and vertical immediate winning threats.",
        probe_move: "J8",
        moves: &[
            "G8", "A1", "H8", "C1", "I8", "E1", "J7", "A3", "J9", "C3", "J10", "E3",
        ],
    },
    BenchScenario {
        id: "renju_forbidden_cross",
        variant: Variant::Renju,
        to_move: Color::Black,
        tags: &["renju", "forbidden", "tactical"],
        description: "Black to move in Renju with a forbidden double-threat point at H8.",
        probe_move: "I8",
        moves: &["H6", "A15", "H7", "C15", "F8", "E15", "G8", "G15"],
    },
    BenchScenario {
        id: "midgame_medium",
        variant: Variant::Freestyle,
        to_move: Color::Black,
        tags: &["midgame", "medium-density", "freestyle"],
        description: "Representative clustered midgame without an immediate tactical finish.",
        probe_move: "F7",
        moves: &[
            "H8", "I8", "H7", "G8", "I7", "G7", "H9", "I9", "F8", "J8", "G9", "H6",
        ],
    },
    BenchScenario {
        id: "midgame_dense",
        variant: Variant::Freestyle,
        to_move: Color::Black,
        tags: &["midgame", "dense", "freestyle"],
        description: "Denser board with more frontier cells and a larger evaluation workload.",
        probe_move: "H10",
        moves: &[
            "H8", "I8", "H7", "G8", "I7", "G7", "H9", "I9", "F8", "J8", "G9", "H6", "J7",
            "F7", "G6", "J9", "F9", "I6", "E8", "K8",
        ],
    },
];

pub static SEARCH_BEHAVIOR_CASES: &[SearchBehaviorCase] = &[
    SearchBehaviorCase {
        id: "balanced_takes_immediate_win",
        scenario_id: "immediate_win",
        config_id: "balanced",
        expected_moves: &["G8", "L8"],
        description: "Balanced should finish its own open four.",
    },
    SearchBehaviorCase {
        id: "balanced_blocks_immediate_loss",
        scenario_id: "immediate_block",
        config_id: "balanced",
        expected_moves: &["E1"],
        description: "Balanced should block an opponent open four.",
    },
    SearchBehaviorCase {
        id: "balanced_blocks_open_three",
        scenario_id: "anti_blunder_open_three",
        config_id: "balanced",
        expected_moves: &["G8", "K8"],
        description: "Balanced should block the forcing open three instead of extending elsewhere.",
    },
    SearchBehaviorCase {
        id: "balanced_wins_race_before_blocking",
        scenario_id: "attack_wins_race",
        config_id: "balanced",
        expected_moves: &["G8", "L8"],
        description: "Balanced should take the immediate win when both sides threaten.",
    },
];
