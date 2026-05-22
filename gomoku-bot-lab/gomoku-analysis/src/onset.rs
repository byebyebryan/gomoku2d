use crate::types::*;
use gomoku_bot::tactical::{
    corridor_active_threats, lethal_threat, normalize_local_threat_facts, LethalThreat,
    LethalThreatKind, LocalThreatFact, LocalThreatKind,
};
use gomoku_core::{Board, Color, GameResult, Move, DIRS};
use std::collections::BTreeSet;

pub(crate) fn find_lethal_onset(
    boards: &[Board],
    attacker: Color,
    scan_start: usize,
    scan_end: usize,
) -> Option<LethalOnset> {
    let defender = attacker.opponent();
    let mut onset = None;
    let mut found_final_suffix = false;
    let scan_end = scan_end.min(boards.len().saturating_sub(1));

    for prefix_ply in (scan_start..=scan_end).rev() {
        let board = &boards[prefix_ply];
        if board.current_player != defender {
            continue;
        }

        if let Some(threat) = lethal_threat(board, attacker) {
            onset = Some(lethal_onset_from_threat(prefix_ply, board, threat));
            found_final_suffix = true;
        } else if found_final_suffix {
            break;
        }
    }

    onset
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum OnsetLineKey {
    Horizontal(usize),
    Vertical(usize),
    DiagonalDown(isize),
    DiagonalUp(usize),
    Point(usize, usize),
}

pub(crate) fn lethal_onset_from_threat(
    prefix_ply: usize,
    board: &Board,
    threat: LethalThreat,
) -> LethalOnset {
    let shape = lethal_onset_shape(board, &threat);
    LethalOnset {
        prefix_ply,
        attacker: threat.attacker,
        defender: threat.defender,
        kind: threat.kind,
        shape,
        terminal_targets: threat.terminal_targets,
        covering_replies: threat.covering_replies,
        one_step_replies: threat
            .one_step_replies
            .into_iter()
            .map(|reply| LethalOnsetReply {
                reply: reply.reply,
                lethal_entries: reply
                    .lethal_entries
                    .into_iter()
                    .map(|entry| LethalOnsetEntry {
                        mv: entry.mv,
                        terminal_targets: entry.terminal_targets,
                    })
                    .collect(),
            })
            .collect(),
    }
}

fn lethal_onset_shape(board: &Board, threat: &LethalThreat) -> LethalOnsetShape {
    let mut components = Vec::new();
    let mut seen = BTreeSet::new();

    for &target in &threat.terminal_targets {
        push_onset_component(
            &mut components,
            &mut seen,
            LethalOnsetComponentTier::Four,
            onset_line_key_for_terminal_target(board, threat.attacker, target),
            target,
        );
    }

    let local_three_facts = onset_local_three_facts(board, threat.attacker);
    for fact in &local_three_facts {
        push_onset_component(
            &mut components,
            &mut seen,
            LethalOnsetComponentTier::Three,
            onset_line_key_for_fact(fact),
            fact.origin.mv(),
        );
    }

    let has_three_component = components
        .iter()
        .any(|component| component.tier == LethalOnsetComponentTier::Three);
    if threat.kind == LethalThreatKind::OneStepCoverage && !has_three_component {
        for reply in &threat.one_step_replies {
            for entry in &reply.lethal_entries {
                push_onset_component(
                    &mut components,
                    &mut seen,
                    LethalOnsetComponentTier::Three,
                    onset_line_key_for_entry(entry.mv, &entry.terminal_targets),
                    entry.mv,
                );
            }
        }
    }

    components.sort_by_key(|component| {
        (
            onset_component_tier_sort_key(component.tier),
            component.mv.row,
            component.mv.col,
        )
    });

    let mut mechanisms = Vec::new();
    if lethal_onset_has_multiple_routes(threat, &components) {
        mechanisms.push(LethalOnsetMechanism::MultiRoute);
    }
    if lethal_onset_has_forbidden_cover(board, threat) {
        mechanisms.push(LethalOnsetMechanism::ForbiddenCover);
    }

    LethalOnsetShape {
        label: onset_shape_label(&components, threat.kind),
        components,
        mechanisms,
    }
}

pub(crate) fn onset_local_three_facts(board: &Board, attacker: Color) -> Vec<LocalThreatFact> {
    normalize_local_threat_facts(
        corridor_active_threats(board, attacker)
            .into_iter()
            .filter(|fact| {
                matches!(
                    fact.kind,
                    LocalThreatKind::OpenThree | LocalThreatKind::BrokenThree
                )
            })
            .collect(),
    )
}

fn push_onset_component(
    components: &mut Vec<LethalOnsetComponent>,
    seen: &mut BTreeSet<(LethalOnsetComponentTierSort, OnsetLineKey)>,
    tier: LethalOnsetComponentTier,
    key: OnsetLineKey,
    mv: Move,
) {
    let sort_tier = onset_component_tier_sort_key(tier);
    if seen.insert((sort_tier, key)) {
        components.push(LethalOnsetComponent { tier, mv });
    }
}

type LethalOnsetComponentTierSort = u8;

fn onset_component_tier_sort_key(tier: LethalOnsetComponentTier) -> LethalOnsetComponentTierSort {
    match tier {
        LethalOnsetComponentTier::Four => 0,
        LethalOnsetComponentTier::Three => 1,
    }
}

fn onset_shape_label(components: &[LethalOnsetComponent], kind: LethalThreatKind) -> String {
    if components.is_empty() {
        return match kind {
            LethalThreatKind::TerminalCoverage => "4".to_string(),
            LethalThreatKind::OneStepCoverage => "3".to_string(),
        };
    }

    components
        .iter()
        .map(|component| match component.tier {
            LethalOnsetComponentTier::Four => "4",
            LethalOnsetComponentTier::Three => "3",
        })
        .collect::<Vec<_>>()
        .join("x")
}

pub(crate) fn onset_line_key_for_fact(fact: &LocalThreatFact) -> OnsetLineKey {
    let origin = fact.origin.mv();
    fact.rest_squares
        .iter()
        .chain(fact.defense_squares.iter())
        .find_map(|&mv| onset_line_key_between(origin, mv))
        .unwrap_or(OnsetLineKey::Point(origin.row, origin.col))
}

fn onset_line_key_for_entry(mv: Move, terminal_targets: &[Move]) -> OnsetLineKey {
    terminal_targets
        .iter()
        .copied()
        .filter(|&target| target != mv)
        .find_map(|target| onset_line_key_between(mv, target))
        .unwrap_or(OnsetLineKey::Point(mv.row, mv.col))
}

fn onset_line_key_for_terminal_target(
    board: &Board,
    attacker: Color,
    target: Move,
) -> OnsetLineKey {
    DIRS.iter()
        .copied()
        .find_map(|(dr, dc)| {
            let run_len = 1
                + count_attacker_stones(board, attacker, target, dr, dc)
                + count_attacker_stones(board, attacker, target, -dr, -dc);
            (run_len >= board.config.win_length)
                .then(|| onset_line_key_for_direction(target, dr, dc))
        })
        .unwrap_or(OnsetLineKey::Point(target.row, target.col))
}

fn count_attacker_stones(
    board: &Board,
    attacker: Color,
    target: Move,
    row_delta: isize,
    col_delta: isize,
) -> usize {
    let size = board.config.board_size as isize;
    let mut count = 0;
    let mut row = target.row as isize + row_delta;
    let mut col = target.col as isize + col_delta;
    while row >= 0
        && col >= 0
        && row < size
        && col < size
        && board.cell(row as usize, col as usize) == Some(attacker)
    {
        count += 1;
        row += row_delta;
        col += col_delta;
    }
    count
}

fn onset_line_key_for_direction(target: Move, row_delta: isize, col_delta: isize) -> OnsetLineKey {
    match (row_delta, col_delta) {
        (0, _) => OnsetLineKey::Horizontal(target.row),
        (_, 0) => OnsetLineKey::Vertical(target.col),
        (dr, dc) if dr == dc => {
            OnsetLineKey::DiagonalDown(target.row as isize - target.col as isize)
        }
        _ => OnsetLineKey::DiagonalUp(target.row + target.col),
    }
}

pub(crate) fn onset_line_key_between(a: Move, b: Move) -> Option<OnsetLineKey> {
    if a.row == b.row && a.col != b.col {
        Some(OnsetLineKey::Horizontal(a.row))
    } else if a.col == b.col && a.row != b.row {
        Some(OnsetLineKey::Vertical(a.col))
    } else {
        let row_delta = a.row.abs_diff(b.row);
        let col_delta = a.col.abs_diff(b.col);
        if row_delta == col_delta && row_delta != 0 {
            let a_row = a.row as isize;
            let a_col = a.col as isize;
            if a_row - a_col == b.row as isize - b.col as isize {
                Some(OnsetLineKey::DiagonalDown(a_row - a_col))
            } else {
                Some(OnsetLineKey::DiagonalUp(a.row + a.col))
            }
        } else {
            None
        }
    }
}

fn lethal_onset_has_multiple_routes(
    threat: &LethalThreat,
    components: &[LethalOnsetComponent],
) -> bool {
    threat.terminal_targets.len() > 1
        || components.len() > 1
        || threat.one_step_replies.iter().any(|reply| {
            reply.lethal_entries.len() > 1
                || reply
                    .lethal_entries
                    .iter()
                    .any(|entry| entry.terminal_targets.len() > 1)
        })
}

fn lethal_onset_has_forbidden_cover(board: &Board, threat: &LethalThreat) -> bool {
    terminal_targets_have_forbidden_cover(board, threat)
        || one_step_entries_have_forbidden_cover(board, threat)
}

fn terminal_targets_have_forbidden_cover(board: &Board, threat: &LethalThreat) -> bool {
    !threat.terminal_targets.is_empty()
        && threat
            .terminal_targets
            .iter()
            .any(|&target| !board.is_legal_for_color(target, threat.defender))
}

fn one_step_entries_have_forbidden_cover(board: &Board, threat: &LethalThreat) -> bool {
    for reply in &threat.one_step_replies {
        let mut after_reply = board.clone();
        if after_reply.apply_move(reply.reply).is_err() {
            continue;
        }

        for entry in &reply.lethal_entries {
            let mut after_entry = after_reply.clone();
            match after_entry.apply_move(entry.mv) {
                Ok(GameResult::Ongoing) => {
                    if entry
                        .terminal_targets
                        .iter()
                        .copied()
                        .filter(|&target| target != entry.mv)
                        .any(|target| !after_entry.is_legal_for_color(target, threat.defender))
                    {
                        return true;
                    }
                }
                Ok(GameResult::Winner(_)) | Ok(GameResult::Draw) | Err(_) => {}
            }
        }
    }

    false
}
