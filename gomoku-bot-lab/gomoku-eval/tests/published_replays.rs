use gomoku_core::{Board, Move, RuleConfig};

#[test]
fn published_tournament_replays_remain_legal_under_current_rules() {
    let report: serde_json::Value =
        serde_json::from_str(include_str!("../../../reports/lab/bot-report.json")).unwrap();
    let rules: RuleConfig = serde_json::from_value(report["run"]["rules"].clone()).unwrap();
    let matches = report["matches"].as_array().unwrap();
    assert!(!matches.is_empty());
    for game in matches {
        let mut board = Board::new(rules.clone());
        for (ply, cell) in game["move_cells"].as_array().unwrap().iter().enumerate() {
            let cell = cell.as_u64().unwrap() as usize;
            let mv = Move {
                row: cell / rules.board_size,
                col: cell % rules.board_size,
            };
            assert!(
                board.apply_move(mv).is_ok(),
                "published match {} ply {} ({})",
                game["match_index"],
                ply + 1,
                mv.to_notation()
            );
        }
    }
}
