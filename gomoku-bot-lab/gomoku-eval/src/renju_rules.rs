use gomoku_core::{Board, Color, Move, RuleConfig, Variant};
use serde::{Deserialize, Serialize};

const RENJUNET_ADVANCED_FIXTURES_JSON: &str =
    include_str!("../../external/renjunet-advanced-examples/fixtures.json");
const RENJU_RULE_BOARD_SIZE: usize = 15;

#[derive(Debug, Clone)]
pub struct RenjuRuleFixture {
    pub id: String,
    pub description: String,
    pub source: String,
    pub black: Vec<String>,
    pub white: Vec<String>,
    pub candidate: String,
    pub color: Color,
    pub expected_legal: bool,
}

#[derive(Debug, Deserialize)]
struct RenjuNetAdvancedFixture {
    id: String,
    source: String,
    window: String,
    probe: String,
    expected: String,
    black: Vec<String>,
    white: Vec<String>,
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

pub fn core_renju_rule_fixtures() -> Vec<RenjuRuleFixture> {
    vec![
        fixture_from_moves(
            "black_exact_five_legal",
            "Black exact five is a legal winning move",
            "rif",
            &["E8", "A1", "F8", "A3", "G8", "A5", "H8", "A7"],
            "I8",
            Color::Black,
            true,
        ),
        fixture_from_moves(
            "black_overline_forbidden",
            "Black overline without an exact five is forbidden",
            "rif",
            &[
                "A1", "H15", "B1", "H13", "C1", "H11", "D1", "H9", "F1", "H7",
            ],
            "E1",
            Color::Black,
            false,
        ),
        fixture_from_moves(
            "black_double_four_forbidden",
            "Black move creating two real fours is forbidden",
            "project_regression",
            &[
                "D8", "A1", "E8", "A3", "F8", "A5", "H5", "A7", "H6", "A9", "H7", "A11",
            ],
            "H8",
            Color::Black,
            false,
        ),
        fixture_from_moves(
            "black_double_three_forbidden",
            "Black move creating two real threes is forbidden",
            "project_regression",
            &["H6", "A1", "H7", "A3", "F8", "A5", "G8", "A7"],
            "H8",
            Color::Black,
            false,
        ),
        fixture_from_moves(
            "black_four_plus_three_legal",
            "Black move creating one four and one three is legal",
            "project_regression",
            &["E8", "A1", "F8", "A3", "G8", "A5", "H6", "A7", "H7", "A9"],
            "H8",
            Color::Black,
            true,
        ),
        fixture_from_moves(
            "white_double_three_unrestricted",
            "White can play the same double-three geometry legally",
            "rif",
            &["A1", "H6", "A3", "H7", "A5", "F8", "A7", "G8"],
            "H8",
            Color::White,
            true,
        ),
    ]
}

pub fn renjunet_advanced_rule_fixtures() -> Result<Vec<RenjuRuleFixture>, String> {
    let raw: Vec<RenjuNetAdvancedFixture> =
        serde_json::from_str(RENJUNET_ADVANCED_FIXTURES_JSON)
            .map_err(|err| format!("failed to parse RenjuNet advanced fixtures: {err}"))?;

    Ok(raw
        .into_iter()
        .map(|case| RenjuRuleFixture {
            id: format!("renjunet_advanced_{}", case.id),
            description: format!("RenjuNet advanced: {}", case.expected),
            source: format!("renjunet_advanced/{}; window {}", case.source, case.window),
            black: case.black,
            white: case.white,
            candidate: case.probe,
            color: Color::Black,
            expected_legal: case
                .expected
                .to_ascii_lowercase()
                .starts_with("not forbidden"),
        })
        .collect())
}

pub fn all_renju_rule_fixtures() -> Result<Vec<RenjuRuleFixture>, String> {
    let mut cases = core_renju_rule_fixtures();
    cases.extend(renjunet_advanced_rule_fixtures()?);
    Ok(cases)
}

fn fixture_from_moves(
    id: &str,
    description: &str,
    source: &str,
    moves: &[&str],
    candidate: &str,
    color: Color,
    expected_legal: bool,
) -> RenjuRuleFixture {
    let mut black = Vec::new();
    let mut white = Vec::new();
    for (index, mv) in moves.iter().enumerate() {
        if index % 2 == 0 {
            black.push((*mv).to_string());
        } else {
            white.push((*mv).to_string());
        }
    }

    RenjuRuleFixture {
        id: id.to_string(),
        description: description.to_string(),
        source: source.to_string(),
        black,
        white,
        candidate: candidate.to_string(),
        color,
        expected_legal,
    }
}

pub fn run_renju_rule_fixtures(cases: &[RenjuRuleFixture]) -> Result<RenjuRuleReport, String> {
    let mut results = Vec::with_capacity(cases.len());

    for case in cases {
        let board = build_board(case)?;
        let candidate = Move::from_notation(&case.candidate)?;
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
    let mut cells = vec!['.'; RENJU_RULE_BOARD_SIZE * RENJU_RULE_BOARD_SIZE];
    for notation in &case.black {
        place_stone(&mut cells, notation, Color::Black, &case.id)?;
    }
    for notation in &case.white {
        place_stone(&mut cells, notation, Color::White, &case.id)?;
    }

    let turn = match case.color {
        Color::Black => 'B',
        Color::White => 'W',
    };
    let fen = format!(
        "{RENJU_RULE_BOARD_SIZE}/5/{turn}/{}",
        cells.iter().collect::<String>()
    );
    let mut board = Board::from_fen(&fen)
        .map_err(|err| format!("fixture '{}' failed to build board: {err}", case.id))?;
    board.config = RuleConfig {
        variant: Variant::Renju,
        ..Default::default()
    };
    Ok(board)
}

fn place_stone(
    cells: &mut [char],
    notation: &str,
    color: Color,
    fixture_id: &str,
) -> Result<(), String> {
    let mv = Move::from_notation(notation)
        .map_err(|err| format!("fixture '{fixture_id}' has invalid move {notation}: {err}"))?;
    let index = mv.row * RENJU_RULE_BOARD_SIZE + mv.col;
    if cells[index] != '.' {
        return Err(format!(
            "fixture '{fixture_id}' has duplicate stone at {notation}"
        ));
    }
    cells[index] = color.to_char();
    Ok(())
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
    use super::{all_renju_rule_fixtures, core_renju_rule_fixtures, run_renju_rule_fixtures};

    #[test]
    fn renju_rule_fixtures_pass() {
        let cases = all_renju_rule_fixtures().unwrap();
        let report = run_renju_rule_fixtures(&cases).unwrap();
        assert_eq!(report.failed, 0);
        assert_eq!(report.total, 29);
    }

    #[test]
    fn renju_rule_report_serializes() {
        let cases = core_renju_rule_fixtures();
        let report = run_renju_rule_fixtures(&cases[..1]).unwrap();
        let json = report.to_json().unwrap();
        assert!(json.contains("black_exact_five_legal"));
    }
}
