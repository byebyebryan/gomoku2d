use gomoku_core::{Board, Color, Move, RuleConfig, Variant};
use serde::Serialize;

#[derive(Debug, Clone, Copy)]
pub struct RenjuRuleFixture {
    pub id: &'static str,
    pub description: &'static str,
    pub source: &'static str,
    pub moves: &'static [&'static str],
    pub candidate: &'static str,
    pub color: Color,
    pub expected_legal: bool,
}

#[derive(Debug, Serialize)]
pub struct RenjuRuleFixtureResult {
    pub id: String,
    pub description: String,
    pub source: String,
    pub candidate: String,
    pub color: Color,
    pub expected_legal: bool,
    pub actual_legal: bool,
    pub passed: bool,
    pub board: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct RenjuRuleReport {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub results: Vec<RenjuRuleFixtureResult>,
}

impl RenjuRuleReport {
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub const RENJU_RULE_FIXTURES: &[RenjuRuleFixture] = &[
    RenjuRuleFixture {
        id: "black_exact_five_legal",
        description: "Black exact five is a legal winning move",
        source: "rif",
        moves: &["E8", "A1", "F8", "A3", "G8", "A5", "H8", "A7"],
        candidate: "I8",
        color: Color::Black,
        expected_legal: true,
    },
    RenjuRuleFixture {
        id: "black_overline_forbidden",
        description: "Black overline without an exact five is forbidden",
        source: "rif",
        moves: &[
            "A1", "H15", "B1", "H13", "C1", "H11", "D1", "H9", "F1", "H7",
        ],
        candidate: "E1",
        color: Color::Black,
        expected_legal: false,
    },
    RenjuRuleFixture {
        id: "black_double_four_forbidden",
        description: "Black move creating two real fours is forbidden",
        source: "project_regression",
        moves: &[
            "D8", "A1", "E8", "A3", "F8", "A5", "H5", "A7", "H6", "A9", "H7", "A11",
        ],
        candidate: "H8",
        color: Color::Black,
        expected_legal: false,
    },
    RenjuRuleFixture {
        id: "black_double_three_forbidden",
        description: "Black move creating two real threes is forbidden",
        source: "project_regression",
        moves: &["H6", "A1", "H7", "A3", "F8", "A5", "G8", "A7"],
        candidate: "H8",
        color: Color::Black,
        expected_legal: false,
    },
    RenjuRuleFixture {
        id: "black_four_plus_three_legal",
        description: "Black move creating one four and one three is legal",
        source: "project_regression",
        moves: &["E8", "A1", "F8", "A3", "G8", "A5", "H6", "A7", "H7", "A9"],
        candidate: "H8",
        color: Color::Black,
        expected_legal: true,
    },
    RenjuRuleFixture {
        id: "white_double_three_unrestricted",
        description: "White can play the same double-three geometry legally",
        source: "rif",
        moves: &["A1", "H6", "A3", "H7", "A5", "F8", "A7", "G8"],
        candidate: "H8",
        color: Color::White,
        expected_legal: true,
    },
    RenjuRuleFixture {
        id: "match_1548_e6_legal",
        description: "Apparent double-three with a dead continuation is legal",
        source: "piskvork/project_regression",
        moves: &[
            "H8", "H7", "F8", "G9", "G8", "I8", "G6", "D9", "F9", "F10", "D7", "G10", "F7", "E10",
            "E8", "D8", "C6", "B5", "D10", "F11", "F6", "F5",
        ],
        candidate: "E6",
        color: Color::Black,
        expected_legal: true,
    },
];

pub fn run_renju_rule_fixtures(cases: &[RenjuRuleFixture]) -> Result<RenjuRuleReport, String> {
    let mut results = Vec::with_capacity(cases.len());

    for case in cases {
        let board = build_board(case)?;
        let candidate = Move::from_notation(case.candidate)?;
        let actual_legal = board.is_legal_for_color(candidate, case.color);
        let passed = actual_legal == case.expected_legal;

        results.push(RenjuRuleFixtureResult {
            id: case.id.to_string(),
            description: case.description.to_string(),
            source: case.source.to_string(),
            candidate: case.candidate.to_string(),
            color: case.color,
            expected_legal: case.expected_legal,
            actual_legal,
            passed,
            board: render_fixture_board(&board, candidate),
        });
    }

    let passed = results.iter().filter(|result| result.passed).count();
    Ok(RenjuRuleReport {
        total: results.len(),
        passed,
        failed: results.len() - passed,
        results,
    })
}

fn build_board(case: &RenjuRuleFixture) -> Result<Board, String> {
    let mut board = Board::new(RuleConfig {
        variant: Variant::Renju,
        ..Default::default()
    });

    for notation in case.moves {
        let mv = Move::from_notation(notation)?;
        board
            .apply_move(mv)
            .map_err(|err| format!("fixture '{}' failed on {notation}: {err}", case.id))?;
    }

    Ok(board)
}

fn render_fixture_board(board: &Board, candidate: Move) -> Vec<String> {
    let mut rows = Vec::with_capacity(board.config.board_size);
    for row in 0..board.config.board_size {
        let mut line = String::with_capacity(board.config.board_size);
        for col in 0..board.config.board_size {
            if candidate.row == row && candidate.col == col {
                line.push('*');
                continue;
            }
            line.push(match board.cell(row, col) {
                Some(Color::Black) => 'X',
                Some(Color::White) => 'O',
                None => '.',
            });
        }
        rows.push(line);
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::{run_renju_rule_fixtures, RENJU_RULE_FIXTURES};

    #[test]
    fn renju_rule_fixtures_pass() {
        let report = run_renju_rule_fixtures(RENJU_RULE_FIXTURES).unwrap();
        assert_eq!(report.failed, 0);
    }

    #[test]
    fn renju_rule_report_serializes() {
        let report = run_renju_rule_fixtures(&RENJU_RULE_FIXTURES[..1]).unwrap();
        let json = report.to_json().unwrap();
        assert!(json.contains("black_exact_five_legal"));
    }
}
