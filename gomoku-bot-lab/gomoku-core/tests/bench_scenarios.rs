#[path = "../../benchmarks/scenarios.rs"]
mod scenarios;

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
        board.forbidden_moves_for_current_player()
            .contains(&scenarios::parse_move("H8")),
        "renju benchmark anchor should keep H8 forbidden"
    );
}
