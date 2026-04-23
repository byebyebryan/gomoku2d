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
        moves: &["H8", "A1", "I8", "B1", "J8", "C1", "K8", "D1"],
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
        id: "anti_blunder_open_three",
        variant: Variant::Freestyle,
        to_move: Color::White,
        tags: &["tactical", "anti-blunder", "freestyle"],
        description: "White has a tempting diagonal extension, but the correct move is to block Black's open three.",
        probe_move: "G8",
        moves: &["H8", "D4", "I8", "F6", "J8"],
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
