use gomoku_bot::tactical::{
    lethal_threat, one_step_lethal_threat_analysis, LethalThreatKind, OneStepLethalThreatAnalysis,
    TerminalLethalThreatAnalysis,
};
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
    pub expected_kind: Option<LethalThreatKind>,
    pub expected_terminal_targets: &'static [&'static str],
    pub expected_covering_replies: &'static [&'static str],
    pub expected_defender_immediate_wins: &'static [&'static str],
    pub expected_one_step_replies: &'static [ExpectedOneStepReply],
    pub expected_escaping_replies: &'static [&'static str],
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

#[derive(Debug, Clone, Copy)]
pub struct ExpectedOneStepReply {
    pub defender_reply: &'static str,
    pub lethal_entries: &'static [ExpectedOneStepEntry],
}

#[derive(Debug, Clone, Copy)]
pub struct ExpectedOneStepEntry {
    pub mv: &'static str,
    pub terminal_targets: &'static [&'static str],
}

pub static LETHAL_SCENARIO_CASES: &[LethalScenarioCase] = &[
    LethalScenarioCase {
        id: "lethal_freestyle_open_four",
        variant: Variant::Freestyle,
        attacker: Color::Black,
        description: "Freestyle open four is terminal-coverage lethal because both endpoints win and no defender reply covers both.",
        moves: &["H8", "A1", "I8", "A2", "J8", "A3", "K8"],
        expected_lethal: true,
        expected_kind: Some(LethalThreatKind::TerminalCoverage),
        expected_terminal_targets: &["G8", "L8"],
        expected_covering_replies: &[],
        expected_defender_immediate_wins: &[],
        expected_one_step_replies: &[],
        expected_escaping_replies: &[],
    },
    LethalScenarioCase {
        id: "nonlethal_blockable_closed_four",
        variant: Variant::Freestyle,
        attacker: Color::Black,
        description: "A single closed four is not terminal-coverage lethal when the defender can legally block the only completion.",
        moves: &["H8", "G8", "I8", "A1", "J8", "A2", "K8"],
        expected_lethal: false,
        expected_kind: None,
        expected_terminal_targets: &["L8"],
        expected_covering_replies: &["L8"],
        expected_defender_immediate_wins: &[],
        expected_one_step_replies: &[ExpectedOneStepReply {
            defender_reply: "L8",
            lethal_entries: &[],
        }],
        expected_escaping_replies: &["L8"],
    },
    LethalScenarioCase {
        id: "nonlethal_defender_immediate_win",
        variant: Variant::Freestyle,
        attacker: Color::White,
        description: "Terminal coverage does not overclaim attacker lethal when the defender can win immediately.",
        moves: &["B1", "H8", "B2", "I8", "B3", "J8", "B4", "K8"],
        expected_lethal: false,
        expected_kind: None,
        expected_terminal_targets: &["G8", "L8"],
        expected_covering_replies: &[],
        expected_defender_immediate_wins: &["B5"],
        expected_one_step_replies: &[],
        expected_escaping_replies: &[],
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
        expected_kind: Some(LethalThreatKind::TerminalCoverage),
        expected_terminal_targets: &["G10"],
        expected_covering_replies: &[],
        expected_defender_immediate_wins: &[],
        expected_one_step_replies: &[],
        expected_escaping_replies: &[],
    },
    LethalScenarioCase {
        id: "nonlethal_renju_black_open_four_overline_completion",
        variant: Variant::Renju,
        attacker: Color::Black,
        description: "Renju Black apparent open four is not automatically lethal when one endpoint is an illegal overline and the other can be blocked.",
        moves: &["H8", "A1", "I8", "C1", "J8", "E1", "K8", "G1", "M8"],
        expected_lethal: false,
        expected_kind: None,
        expected_terminal_targets: &["G8"],
        expected_covering_replies: &["G8"],
        expected_defender_immediate_wins: &[],
        expected_one_step_replies: &[ExpectedOneStepReply {
            defender_reply: "G8",
            lethal_entries: &[],
        }],
        expected_escaping_replies: &["G8"],
    },
    LethalScenarioCase {
        id: "lethal_freestyle_four_three",
        variant: Variant::Freestyle,
        attacker: Color::Black,
        description: "A crossed 4+3 is one-step lethal when the only direct four block still lets the attacker create terminal open-four coverage on the crossing line.",
        moves: &[
            "H8", "G8", "I8", "A1", "J8", "O1", "K8", "A15", "I7", "O15", "I9",
        ],
        expected_lethal: true,
        expected_kind: Some(LethalThreatKind::OneStepCoverage),
        expected_terminal_targets: &["L8"],
        expected_covering_replies: &["L8"],
        expected_defender_immediate_wins: &[],
        expected_one_step_replies: &[ExpectedOneStepReply {
            defender_reply: "L8",
            lethal_entries: &[
                ExpectedOneStepEntry {
                    mv: "I6",
                    terminal_targets: &["I5", "I10"],
                },
                ExpectedOneStepEntry {
                    mv: "I10",
                    terminal_targets: &["I6", "I11"],
                },
            ],
        }],
        expected_escaping_replies: &[],
    },
    LethalScenarioCase {
        id: "lethal_freestyle_double_open_three",
        variant: Variant::Freestyle,
        attacker: Color::Black,
        description: "A crossed 3+3 is one-step lethal when every direct reply to either open three lets the attacker convert the other into terminal open-four coverage.",
        moves: &["H8", "A1", "I8", "O1", "J8", "A15", "I7", "O15", "I9"],
        expected_lethal: true,
        expected_kind: Some(LethalThreatKind::OneStepCoverage),
        expected_terminal_targets: &[],
        expected_covering_replies: &[],
        expected_defender_immediate_wins: &[],
        expected_one_step_replies: &[
            ExpectedOneStepReply {
                defender_reply: "I6",
                lethal_entries: &[
                    ExpectedOneStepEntry {
                        mv: "G8",
                        terminal_targets: &["F8", "K8"],
                    },
                    ExpectedOneStepEntry {
                        mv: "K8",
                        terminal_targets: &["G8", "L8"],
                    },
                ],
            },
            ExpectedOneStepReply {
                defender_reply: "G8",
                lethal_entries: &[
                    ExpectedOneStepEntry {
                        mv: "I6",
                        terminal_targets: &["I5", "I10"],
                    },
                    ExpectedOneStepEntry {
                        mv: "I10",
                        terminal_targets: &["I6", "I11"],
                    },
                ],
            },
            ExpectedOneStepReply {
                defender_reply: "K8",
                lethal_entries: &[
                    ExpectedOneStepEntry {
                        mv: "I6",
                        terminal_targets: &["I5", "I10"],
                    },
                    ExpectedOneStepEntry {
                        mv: "I10",
                        terminal_targets: &["I6", "I11"],
                    },
                ],
            },
            ExpectedOneStepReply {
                defender_reply: "I10",
                lethal_entries: &[
                    ExpectedOneStepEntry {
                        mv: "G8",
                        terminal_targets: &["F8", "K8"],
                    },
                    ExpectedOneStepEntry {
                        mv: "K8",
                        terminal_targets: &["G8", "L8"],
                    },
                ],
            },
        ],
        expected_escaping_replies: &[],
    },
    LethalScenarioCase {
        id: "nonlethal_crossed_broken_threes_shared_block",
        variant: Variant::Freestyle,
        attacker: Color::Black,
        description: "A crossed pair of broken threes is not lethal when the shared open crossing point blocks both threats.",
        moves: &[
            "G8", "A1", "H8", "O1", "J8", "A15", "I6", "O15", "I7", "C3", "I9",
        ],
        expected_lethal: false,
        expected_kind: None,
        expected_terminal_targets: &[],
        expected_covering_replies: &[],
        expected_defender_immediate_wins: &[],
        expected_one_step_replies: &[
            ExpectedOneStepReply {
                defender_reply: "I5",
                lethal_entries: &[ExpectedOneStepEntry {
                    mv: "I8",
                    terminal_targets: &["F8", "K8", "I10"],
                }],
            },
            ExpectedOneStepReply {
                defender_reply: "F8",
                lethal_entries: &[ExpectedOneStepEntry {
                    mv: "I8",
                    terminal_targets: &["I5", "K8", "I10"],
                }],
            },
            ExpectedOneStepReply {
                defender_reply: "I8",
                lethal_entries: &[],
            },
            ExpectedOneStepReply {
                defender_reply: "K8",
                lethal_entries: &[ExpectedOneStepEntry {
                    mv: "I8",
                    terminal_targets: &["I5", "F8", "I10"],
                }],
            },
            ExpectedOneStepReply {
                defender_reply: "I10",
                lethal_entries: &[ExpectedOneStepEntry {
                    mv: "I8",
                    terminal_targets: &["F8", "K8", "I5"],
                }],
            },
        ],
        expected_escaping_replies: &["I8"],
    },
    LethalScenarioCase {
        id: "nonlethal_single_open_three",
        variant: Variant::Freestyle,
        attacker: Color::Black,
        description: "A single open three is not lethal because either endpoint defense escapes terminal coverage.",
        moves: &["H8", "A1", "I8", "C1", "J8"],
        expected_lethal: false,
        expected_kind: None,
        expected_terminal_targets: &[],
        expected_covering_replies: &[],
        expected_defender_immediate_wins: &[],
        expected_one_step_replies: &[
            ExpectedOneStepReply {
                defender_reply: "G8",
                lethal_entries: &[],
            },
            ExpectedOneStepReply {
                defender_reply: "K8",
                lethal_entries: &[],
            },
        ],
        expected_escaping_replies: &["G8", "K8"],
    },
];

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LethalScenarioOneStepEntryResult {
    pub mv: String,
    pub terminal_targets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LethalScenarioOneStepReplyResult {
    pub defender_reply: String,
    pub lethal_entries: Vec<LethalScenarioOneStepEntryResult>,
}

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
    pub expected_kind: Option<LethalThreatKind>,
    pub actual_kind: Option<LethalThreatKind>,
    pub expected_terminal_targets: Vec<String>,
    pub actual_terminal_targets: Vec<String>,
    pub expected_covering_replies: Vec<String>,
    pub actual_covering_replies: Vec<String>,
    pub expected_defender_immediate_wins: Vec<String>,
    pub actual_defender_immediate_wins: Vec<String>,
    pub expected_one_step_replies: Vec<LethalScenarioOneStepReplyResult>,
    pub actual_one_step_replies: Vec<LethalScenarioOneStepReplyResult>,
    pub expected_escaping_replies: Vec<String>,
    pub actual_escaping_replies: Vec<String>,
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
    let terminal = gomoku_bot::tactical::terminal_lethal_threat_analysis(&board, case.attacker);
    let one_step = one_step_lethal_threat_analysis(&board, case.attacker);
    lethal_scenario_result(case, &board, terminal, one_step)
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
    terminal: TerminalLethalThreatAnalysis,
    one_step: OneStepLethalThreatAnalysis,
) -> LethalScenarioResult {
    let expected_terminal_targets = notation_list_from_strs(case.expected_terminal_targets);
    let actual_terminal_targets = notation_list(&terminal.terminal_targets);
    let expected_covering_replies = notation_list_from_strs(case.expected_covering_replies);
    let actual_covering_replies = notation_list(&terminal.covering_replies);
    let expected_defender_immediate_wins =
        notation_list_from_strs(case.expected_defender_immediate_wins);
    let mut defender_immediate_wins = terminal.defender_immediate_wins.clone();
    defender_immediate_wins.extend(one_step.defender_immediate_wins.iter().copied());
    normalize_moves(&mut defender_immediate_wins);
    let actual_defender_immediate_wins = notation_list(&defender_immediate_wins);
    let expected_one_step_replies =
        one_step_reply_list_from_expected(case.expected_one_step_replies);
    let actual_one_step_replies = one_step_reply_list(&one_step);
    let expected_escaping_replies = notation_list_from_strs(case.expected_escaping_replies);
    let actual_escaping_replies = notation_list(&one_step.escaping_replies);
    let threat = lethal_threat(board, case.attacker);
    let actual_kind = threat.as_ref().map(|threat| threat.kind);
    let actual_lethal = threat.is_some();
    let passed = actual_lethal == case.expected_lethal
        && actual_kind == case.expected_kind
        && actual_terminal_targets == expected_terminal_targets
        && actual_covering_replies == expected_covering_replies
        && actual_defender_immediate_wins == expected_defender_immediate_wins
        && actual_one_step_replies == expected_one_step_replies
        && actual_escaping_replies == expected_escaping_replies;

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
        expected_kind: case.expected_kind,
        actual_kind,
        expected_terminal_targets,
        actual_terminal_targets,
        expected_covering_replies,
        actual_covering_replies,
        expected_defender_immediate_wins,
        actual_defender_immediate_wins,
        expected_one_step_replies,
        actual_one_step_replies,
        expected_escaping_replies,
        actual_escaping_replies,
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
    normalize_moves(&mut moves);
    notation_list(&moves)
}

fn normalize_moves(moves: &mut Vec<Move>) {
    moves.sort_by_key(|mv| (mv.row, mv.col));
    moves.dedup();
}

fn one_step_reply_list(
    analysis: &OneStepLethalThreatAnalysis,
) -> Vec<LethalScenarioOneStepReplyResult> {
    let mut replies = analysis
        .defender_replies
        .iter()
        .map(|reply| LethalScenarioOneStepReplyResult {
            defender_reply: reply.reply.to_notation(),
            lethal_entries: one_step_entry_list(&reply.lethal_entries),
        })
        .collect::<Vec<_>>();
    replies.sort_by_key(|reply| notation_sort_key(&reply.defender_reply));
    replies
}

fn one_step_reply_list_from_expected(
    replies: &[ExpectedOneStepReply],
) -> Vec<LethalScenarioOneStepReplyResult> {
    let mut replies = replies
        .iter()
        .map(|reply| LethalScenarioOneStepReplyResult {
            defender_reply: parse_move(reply.defender_reply).to_notation(),
            lethal_entries: one_step_entry_list_from_expected(reply.lethal_entries),
        })
        .collect::<Vec<_>>();
    replies.sort_by_key(|reply| notation_sort_key(&reply.defender_reply));
    replies
}

fn one_step_entry_list(
    entries: &[gomoku_bot::tactical::OneStepLethalEntry],
) -> Vec<LethalScenarioOneStepEntryResult> {
    let mut entries = entries
        .iter()
        .map(|entry| LethalScenarioOneStepEntryResult {
            mv: entry.mv.to_notation(),
            terminal_targets: notation_list(&entry.terminal_targets),
        })
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| notation_sort_key(&entry.mv));
    entries
}

fn one_step_entry_list_from_expected(
    entries: &[ExpectedOneStepEntry],
) -> Vec<LethalScenarioOneStepEntryResult> {
    let mut entries = entries
        .iter()
        .map(|entry| LethalScenarioOneStepEntryResult {
            mv: parse_move(entry.mv).to_notation(),
            terminal_targets: notation_list_from_strs(entry.terminal_targets),
        })
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| notation_sort_key(&entry.mv));
    entries
}

fn notation_sort_key(notation: &str) -> (usize, usize) {
    let mv = parse_move(notation);
    (mv.row, mv.col)
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
