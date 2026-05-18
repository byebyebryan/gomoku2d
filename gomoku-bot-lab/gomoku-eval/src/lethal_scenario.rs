use gomoku_bot::tactical::{terminal_lethal_threat_analysis, TerminalLethalThreatAnalysis};
use gomoku_core::{Board, Color, Move, RuleConfig, Variant};
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct LethalScenarioCase {
    pub id: &'static str,
    pub variant: Variant,
    pub attacker: Color,
    pub description: &'static str,
    pub moves: &'static [&'static str],
    pub expected_lethal: bool,
    pub expected_terminal_targets: &'static [&'static str],
    pub expected_covering_replies: &'static [&'static str],
    pub expected_defender_immediate_wins: &'static [&'static str],
}

impl LethalScenarioCase {
    fn board(&self) -> Board {
        let mut board = Board::new(RuleConfig {
            variant: self.variant.clone(),
            ..RuleConfig::default()
        });

        for &mv in self.moves {
            board.apply_move(parse_move(mv)).unwrap_or_else(|err| {
                panic!(
                    "lethal scenario '{}' failed to apply move {mv}: {err}",
                    self.id
                )
            });
        }

        assert_eq!(
            board.current_player,
            self.attacker.opponent(),
            "lethal scenario '{}' should leave defender to move",
            self.id
        );
        board
    }
}

pub static LETHAL_SCENARIO_CASES: &[LethalScenarioCase] = &[
    LethalScenarioCase {
        id: "lethal_freestyle_open_four",
        variant: Variant::Freestyle,
        attacker: Color::Black,
        description: "Freestyle open four is terminal-coverage lethal because both endpoints win and no defender reply covers both.",
        moves: &["H8", "A1", "I8", "A2", "J8", "A3", "K8"],
        expected_lethal: true,
        expected_terminal_targets: &["G8", "L8"],
        expected_covering_replies: &[],
        expected_defender_immediate_wins: &[],
    },
    LethalScenarioCase {
        id: "nonlethal_blockable_closed_four",
        variant: Variant::Freestyle,
        attacker: Color::Black,
        description: "A single closed four is not terminal-coverage lethal when the defender can legally block the only completion.",
        moves: &["H8", "G8", "I8", "A1", "J8", "A2", "K8"],
        expected_lethal: false,
        expected_terminal_targets: &["L8"],
        expected_covering_replies: &["L8"],
        expected_defender_immediate_wins: &[],
    },
    LethalScenarioCase {
        id: "nonlethal_defender_immediate_win",
        variant: Variant::Freestyle,
        attacker: Color::White,
        description: "Terminal coverage does not overclaim attacker lethal when the defender can win immediately.",
        moves: &["B1", "H8", "B2", "I8", "B3", "J8", "B4", "K8"],
        expected_lethal: false,
        expected_terminal_targets: &["G8", "L8"],
        expected_covering_replies: &[],
        expected_defender_immediate_wins: &["B5"],
    },
    LethalScenarioCase {
        id: "lethal_renju_forbidden_block",
        variant: Variant::Renju,
        attacker: Color::White,
        description: "White single terminal target is lethal when Black's only natural block is forbidden.",
        moves: &[
            "H8", "H7", "J8", "I9", "I8", "G8", "F9", "F7", "H9", "G7", "I7", "E7",
            "D7", "G9", "G6", "G11",
        ],
        expected_lethal: true,
        expected_terminal_targets: &["G10"],
        expected_covering_replies: &[],
        expected_defender_immediate_wins: &[],
    },
    LethalScenarioCase {
        id: "nonlethal_renju_black_open_four_overline_completion",
        variant: Variant::Renju,
        attacker: Color::Black,
        description: "Renju Black apparent open four is not automatically lethal when one endpoint is an illegal overline and the other can be blocked.",
        moves: &["H8", "A1", "I8", "C1", "J8", "E1", "K8", "G1", "M8"],
        expected_lethal: false,
        expected_terminal_targets: &["G8"],
        expected_covering_replies: &["G8"],
        expected_defender_immediate_wins: &[],
    },
];

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LethalScenarioResult {
    pub case_id: &'static str,
    pub variant: Variant,
    pub attacker: Color,
    pub defender: Color,
    pub description: &'static str,
    pub moves: Vec<String>,
    pub board_ascii: String,
    pub expected_lethal: bool,
    pub actual_lethal: bool,
    pub expected_terminal_targets: Vec<String>,
    pub actual_terminal_targets: Vec<String>,
    pub expected_covering_replies: Vec<String>,
    pub actual_covering_replies: Vec<String>,
    pub expected_defender_immediate_wins: Vec<String>,
    pub actual_defender_immediate_wins: Vec<String>,
    pub passed: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LethalScenarioReport {
    pub schema_version: u32,
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub results: Vec<LethalScenarioResult>,
}

impl LethalScenarioReport {
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn run_lethal_scenario(case: &LethalScenarioCase) -> LethalScenarioResult {
    let board = case.board();
    let analysis = terminal_lethal_threat_analysis(&board, case.attacker);
    lethal_scenario_result(case, &board, analysis)
}

pub fn run_lethal_scenarios(cases: &[LethalScenarioCase]) -> LethalScenarioReport {
    let results = cases.iter().map(run_lethal_scenario).collect::<Vec<_>>();
    let total = results.len();
    let passed = results.iter().filter(|result| result.passed).count();
    LethalScenarioReport {
        schema_version: 1,
        total,
        passed,
        failed: total - passed,
        results,
    }
}

fn lethal_scenario_result(
    case: &LethalScenarioCase,
    board: &Board,
    analysis: TerminalLethalThreatAnalysis,
) -> LethalScenarioResult {
    let expected_terminal_targets = notation_list_from_strs(case.expected_terminal_targets);
    let actual_terminal_targets = notation_list(&analysis.terminal_targets);
    let expected_covering_replies = notation_list_from_strs(case.expected_covering_replies);
    let actual_covering_replies = notation_list(&analysis.covering_replies);
    let expected_defender_immediate_wins =
        notation_list_from_strs(case.expected_defender_immediate_wins);
    let actual_defender_immediate_wins = notation_list(&analysis.defender_immediate_wins);
    let actual_lethal = analysis.lethal_threat().is_some();
    let passed = actual_lethal == case.expected_lethal
        && actual_terminal_targets == expected_terminal_targets
        && actual_covering_replies == expected_covering_replies
        && actual_defender_immediate_wins == expected_defender_immediate_wins;

    LethalScenarioResult {
        case_id: case.id,
        variant: case.variant.clone(),
        attacker: case.attacker,
        defender: case.attacker.opponent(),
        description: case.description,
        moves: case.moves.iter().map(|mv| (*mv).to_string()).collect(),
        board_ascii: board_ascii(board),
        expected_lethal: case.expected_lethal,
        actual_lethal,
        expected_terminal_targets,
        actual_terminal_targets,
        expected_covering_replies,
        actual_covering_replies,
        expected_defender_immediate_wins,
        actual_defender_immediate_wins,
        passed,
    }
}

fn parse_move(notation: &str) -> Move {
    Move::from_notation(notation)
        .unwrap_or_else(|err| panic!("invalid lethal scenario move '{notation}': {err}"))
}

fn notation_list(moves: &[Move]) -> Vec<String> {
    moves.iter().copied().map(Move::to_notation).collect()
}

fn notation_list_from_strs(moves: &[&str]) -> Vec<String> {
    let mut moves = moves.iter().copied().map(parse_move).collect::<Vec<_>>();
    moves.sort_by_key(|mv| (mv.row, mv.col));
    notation_list(&moves)
}

fn board_ascii(board: &Board) -> String {
    let size = board.config.board_size;
    let mut output = String::new();

    output.push_str("    ");
    for col in 0..size {
        if col > 0 {
            output.push(' ');
        }
        output.push(column_label(col));
    }
    output.push('\n');

    for row in (0..size).rev() {
        output.push_str(&format!("{:>2}  ", row + 1));
        for col in 0..size {
            if col > 0 {
                output.push(' ');
            }
            output.push(board.cell(row, col).map_or('.', Color::to_char));
        }
        output.push_str(&format!("  {}", row + 1));
        output.push('\n');
    }

    output.push_str("    ");
    for col in 0..size {
        if col > 0 {
            output.push(' ');
        }
        output.push(column_label(col));
    }

    output
}

fn column_label(col: usize) -> char {
    let col = u8::try_from(col).expect("board column should fit in u8");
    char::from(b'A' + col)
}

#[cfg(test)]
mod tests {
    use super::{run_lethal_scenarios, LETHAL_SCENARIO_CASES};

    #[test]
    fn lethal_scenario_cases_match_terminal_classifier() {
        let report = run_lethal_scenarios(LETHAL_SCENARIO_CASES);
        assert_eq!(report.failed, 0, "{:#?}", report.results);
    }

    #[test]
    fn lethal_scenario_report_serializes() {
        let report = run_lethal_scenarios(LETHAL_SCENARIO_CASES);
        let json = report.to_json().expect("report should serialize");
        assert!(json.contains("\"schema_version\": 1"));
        assert!(json.contains("lethal_freestyle_open_four"));
        assert!(json.contains("\"board_ascii\""));
    }

    #[test]
    fn lethal_scenario_board_ascii_uses_printed_board_orientation() {
        let report = run_lethal_scenarios(&LETHAL_SCENARIO_CASES[..1]);
        let board = &report.results[0].board_ascii;

        assert!(board.starts_with("    A B C D E F G H I J K L M N O\n"));
        assert!(board.contains(" 8  . . . . . . . B B B B . . . .  8"));
        assert!(board.ends_with("    A B C D E F G H I J K L M N O"));
    }
}
