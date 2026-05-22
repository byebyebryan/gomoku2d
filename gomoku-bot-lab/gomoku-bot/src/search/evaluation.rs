use crate::pattern::evaluate_pattern_scan;
use gomoku_core::{Board, Color, GameResult, DIRS};

use super::StaticEvaluation;

// --- Static evaluation ---

fn score_line(counts: &[i32; 6], open_ends: &[i32; 6]) -> i32 {
    let mut score = 0i32;
    for len in 2..=5usize {
        let c = counts[len];
        if c == 0 {
            continue;
        }
        let o = open_ends[len];
        let base: i32 = match len {
            5 => 1_000_000,
            4 => 10_000,
            3 => 1_000,
            2 => 100,
            _ => 0,
        };
        score += base * c * (o + 1);
    }
    score
}

pub(super) fn evaluate_static(board: &Board, color: Color, static_eval: StaticEvaluation) -> i32 {
    match static_eval {
        StaticEvaluation::LineShapeEval => evaluate(board, color),
        StaticEvaluation::PatternEval => evaluate_pattern_scan(board, color),
    }
}

pub(super) fn evaluate(board: &Board, color: Color) -> i32 {
    if let GameResult::Winner(w) = &board.result {
        return if *w == color { 2_000_000 } else { -2_000_000 };
    }
    if board.result == GameResult::Draw {
        return 0;
    }

    let size = board.config.board_size;
    let win_len = board.config.win_length as isize;

    let mut counts = [[0i32; 6]; 2];
    let mut open_ends = [[0i32; 6]; 2];
    let mut terminal_score = None;

    for &(dr, dc) in &DIRS {
        board.for_each_occupied(|row, col, player| {
            if terminal_score.is_some() {
                return;
            }

            let row = row as isize;
            let col = col as isize;

            // Only score a contiguous run once, from its back end.
            let pr = row - dr;
            let pc = col - dc;
            if pr >= 0
                && pr < size as isize
                && pc >= 0
                && pc < size as isize
                && board.has_color(pr as usize, pc as usize, player)
            {
                return;
            }

            let mut len = 0isize;
            let (mut r, mut c) = (row, col);
            while r >= 0
                && r < size as isize
                && c >= 0
                && c < size as isize
                && board.has_color(r as usize, c as usize, player)
            {
                len += 1;
                r += dr;
                c += dc;
            }

            if len >= win_len {
                terminal_score = Some(if player == color {
                    2_000_000
                } else {
                    -2_000_000
                });
                return;
            }
            if len < 2 {
                return;
            }

            let mut ends = 0i32;
            let (br, bc) = (row - dr, col - dc);
            if br >= 0
                && br < size as isize
                && bc >= 0
                && bc < size as isize
                && board.is_empty(br as usize, bc as usize)
            {
                ends += 1;
            }
            if r >= 0
                && r < size as isize
                && c >= 0
                && c < size as isize
                && board.is_empty(r as usize, c as usize)
            {
                ends += 1;
            }
            if ends > 0 {
                let score_idx = if player == color { 0 } else { 1 };
                let len_idx = len.min(5) as usize;
                counts[score_idx][len_idx] += 1;
                open_ends[score_idx][len_idx] += ends;
            }
        });

        if let Some(score) = terminal_score {
            return score;
        }
    }

    score_line(&counts[0], &open_ends[0]) - score_line(&counts[1], &open_ends[1])
}

#[cfg(test)]
pub(super) fn evaluate_reference(board: &Board, color: Color) -> i32 {
    if let GameResult::Winner(w) = &board.result {
        return if *w == color { 2_000_000 } else { -2_000_000 };
    }
    if board.result == GameResult::Draw {
        return 0;
    }

    let size = board.config.board_size;
    let win_len = board.config.win_length as isize;

    let mut my_score = 0i32;
    let mut opp_score = 0i32;
    let opp = color.opponent();

    for &player in &[color, opp] {
        let mut counts = [0i32; 6];
        let mut open_ends = [0i32; 6];

        for &(dr, dc) in &DIRS {
            for row in 0..size as isize {
                for col in 0..size as isize {
                    let pr = row - dr;
                    let pc = col - dc;
                    let back_in_bounds =
                        pr >= 0 && pr < size as isize && pc >= 0 && pc < size as isize;
                    if back_in_bounds && board.has_color(pr as usize, pc as usize, player) {
                        continue;
                    }
                    if !board.has_color(row as usize, col as usize, player) {
                        continue;
                    }

                    let mut len = 0isize;
                    let (mut r, mut c) = (row, col);
                    while r >= 0
                        && r < size as isize
                        && c >= 0
                        && c < size as isize
                        && board.has_color(r as usize, c as usize, player)
                    {
                        len += 1;
                        r += dr;
                        c += dc;
                    }
                    if len >= win_len {
                        if player == color {
                            return 2_000_000;
                        } else {
                            return -2_000_000;
                        }
                    }
                    if len < 2 {
                        continue;
                    }

                    let mut ends = 0i32;
                    let (br, bc) = (row - dr, col - dc);
                    if br >= 0
                        && br < size as isize
                        && bc >= 0
                        && bc < size as isize
                        && board.is_empty(br as usize, bc as usize)
                    {
                        ends += 1;
                    }
                    if r >= 0
                        && r < size as isize
                        && c >= 0
                        && c < size as isize
                        && board.is_empty(r as usize, c as usize)
                    {
                        ends += 1;
                    }
                    if ends > 0 {
                        let idx = len.min(5) as usize;
                        counts[idx] += 1;
                        open_ends[idx] += ends;
                    }
                }
            }
        }

        let s = score_line(&counts, &open_ends);
        if player == color {
            my_score += s;
        } else {
            opp_score += s;
        }
    }

    my_score - opp_score
}
