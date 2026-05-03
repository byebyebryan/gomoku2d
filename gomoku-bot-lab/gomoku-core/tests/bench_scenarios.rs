#[path = "../../benchmarks/scenarios.rs"]
mod scenarios;

use gomoku_core::{Board, Color, Move};
use gomoku_core::{GameResult, Variant};

#[test]
fn benchmark_scenarios_are_valid_and_ongoing() {
    for scenario in scenarios::SCENARIOS {
        let board = scenario.board();
        let probe = scenario.probe_move();

        assert_eq!(
            board.result,
            GameResult::Ongoing,
            "scenario '{}' unexpectedly ended the game",
            scenario.id
        );
        assert!(
            board.is_legal(probe),
            "scenario '{}' probe move {} must stay legal",
            scenario.id,
            scenario.probe_move
        );
    }
}

#[test]
fn renju_forbidden_anchor_stays_meaningful() {
    let scenario = scenarios::SCENARIOS
        .iter()
        .find(|scenario| scenario.id == "renju_forbidden_cross")
        .expect("expected renju anchor scenario");
    let board = scenario.board();

    assert_eq!(board.config.variant, Variant::Renju);
    assert!(
        board
            .forbidden_moves_for_current_player()
            .contains(&scenarios::parse_move("H8")),
        "renju benchmark anchor should keep H8 forbidden"
    );
}

#[test]
fn immediate_winning_moves_match_full_scan_on_benchmark_scenarios() {
    for scenario in scenarios::SCENARIOS {
        let board = scenario.board();
        for color in [Color::Black, Color::White] {
            assert_eq!(
                board.immediate_winning_moves_for(color),
                full_scan_immediate_winning_moves_for(&board, color),
                "scenario '{}' immediate wins drifted for {:?}",
                scenario.id,
                color
            );
        }
    }
}

fn full_scan_immediate_winning_moves_for(board: &Board, color: Color) -> Vec<Move> {
    if board.result != GameResult::Ongoing {
        return vec![];
    }

    let mut wins = Vec::new();
    for row in 0..board.config.board_size {
        for col in 0..board.config.board_size {
            if board.cell(row, col).is_some() {
                continue;
            }

            let mv = Move { row, col };
            let mut next = board.clone();
            next.current_player = color;
            if matches!(next.apply_move(mv), Ok(GameResult::Winner(winner)) if winner == color) {
                wins.push(mv);
            }
        }
    }
    wins
}
