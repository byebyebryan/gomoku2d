use crate::board::{Board, Cell, Color, Move, DIRS};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ForbiddenReason {
    Overline,
    DoubleFour,
    DoubleThree,
}

const MAX_RECURSION_DEPTH: usize = 4;

pub(crate) fn forbidden_reason(board: &Board, mv: Move) -> Option<ForbiddenReason> {
    if mv.row >= board.config.board_size
        || mv.col >= board.config.board_size
        || board.cell(mv.row, mv.col).is_some()
    {
        return None;
    }

    let view = RenjuView::new(board).with_black(mv)?;
    forbidden_reason_in_view(&view, mv, MAX_RECURSION_DEPTH)
}

#[derive(Clone)]
struct RenjuView<'a> {
    board: &'a Board,
    black_overlays: Vec<Move>,
}

impl<'a> RenjuView<'a> {
    fn new(board: &'a Board) -> Self {
        Self {
            board,
            black_overlays: Vec::new(),
        }
    }

    fn with_black(&self, mv: Move) -> Option<Self> {
        if !self.is_empty(mv) {
            return None;
        }

        let mut next = self.clone();
        next.black_overlays.push(mv);
        Some(next)
    }

    fn size(&self) -> usize {
        self.board.config.board_size
    }

    fn win_len(&self) -> usize {
        self.board.config.win_length
    }

    fn cell(&self, row: isize, col: isize) -> Option<Cell> {
        let size = self.size() as isize;
        if row < 0 || row >= size || col < 0 || col >= size {
            return None;
        }

        let mv = Move {
            row: row as usize,
            col: col as usize,
        };
        if self.black_overlays.contains(&mv) {
            return Some(Some(Color::Black));
        }

        Some(self.board.cell(mv.row, mv.col))
    }

    fn is_empty(&self, mv: Move) -> bool {
        matches!(self.cell(mv.row as isize, mv.col as isize), Some(None))
    }

    fn has_black(&self, row: isize, col: isize) -> bool {
        matches!(self.cell(row, col), Some(Some(Color::Black)))
    }
}

fn forbidden_reason_in_view(
    view: &RenjuView<'_>,
    mv: Move,
    depth_remaining: usize,
) -> Option<ForbiddenReason> {
    if creates_exact_five(view, mv) {
        return None;
    }
    if creates_overline(view, mv) {
        return Some(ForbiddenReason::Overline);
    }
    if real_four_count(view, mv) >= 2 {
        return Some(ForbiddenReason::DoubleFour);
    }
    if real_three_direction_count(view, mv, depth_remaining) >= 2 {
        return Some(ForbiddenReason::DoubleThree);
    }

    None
}

fn creates_exact_five(view: &RenjuView<'_>, mv: Move) -> bool {
    DIRS.iter()
        .any(|&dir| contiguous_run_len(view, mv, dir) == view.win_len())
}

fn creates_overline(view: &RenjuView<'_>, mv: Move) -> bool {
    DIRS.iter()
        .any(|&dir| contiguous_run_len(view, mv, dir) > view.win_len())
}

fn contiguous_run_len(view: &RenjuView<'_>, mv: Move, (dr, dc): (isize, isize)) -> usize {
    1 + count_black(view, mv, (dr, dc)) + count_black(view, mv, (-dr, -dc))
}

fn count_black(view: &RenjuView<'_>, mv: Move, (dr, dc): (isize, isize)) -> usize {
    let mut count = 0;
    let (mut row, mut col) = (mv.row as isize + dr, mv.col as isize + dc);
    while view.has_black(row, col) {
        count += 1;
        row += dr;
        col += dc;
    }
    count
}

fn real_four_count(view: &RenjuView<'_>, origin: Move) -> usize {
    let mut keys = Vec::new();
    for (dir_index, &dir) in DIRS.iter().enumerate() {
        collect_real_four_keys(view, origin, dir_index, dir, &mut keys);
    }

    keys.sort();
    keys.dedup();
    keys.len()
}

fn collect_real_four_keys(
    view: &RenjuView<'_>,
    origin: Move,
    dir_index: usize,
    (dr, dc): (isize, isize),
    keys: &mut Vec<(usize, [(usize, usize); 4])>,
) {
    for origin_pos in 0..=4isize {
        let start = -origin_pos;
        let mut black = 0usize;
        let mut black_cells = [(0usize, 0usize); 4];
        let mut empty = None;
        let mut valid = true;

        for i in 0..5isize {
            let row = origin.row as isize + (start + i) * dr;
            let col = origin.col as isize + (start + i) * dc;
            match view.cell(row, col) {
                Some(Some(Color::Black)) => {
                    if black < black_cells.len() {
                        black_cells[black] = (row as usize, col as usize);
                    }
                    black += 1;
                }
                Some(None) => {
                    if empty.is_some() {
                        valid = false;
                        break;
                    }
                    empty = Some(Move {
                        row: row as usize,
                        col: col as usize,
                    });
                }
                _ => {
                    valid = false;
                    break;
                }
            }
        }

        if valid
            && black == 4
            && empty.is_some_and(|completion| legal_exact_five_completion(view, completion))
        {
            black_cells.sort();
            keys.push((dir_index, black_cells));
        }
    }
}

fn legal_exact_five_completion(view: &RenjuView<'_>, mv: Move) -> bool {
    view.with_black(mv)
        .is_some_and(|next| creates_exact_five(&next, mv))
}

fn real_three_direction_count(view: &RenjuView<'_>, origin: Move, depth_remaining: usize) -> usize {
    DIRS.iter()
        .filter(|&&dir| has_real_three_in_direction(view, origin, dir, depth_remaining))
        .count()
}

fn has_real_three_in_direction(
    view: &RenjuView<'_>,
    origin: Move,
    (dr, dc): (isize, isize),
    depth_remaining: usize,
) -> bool {
    if depth_remaining == 0 {
        return false;
    }

    for offset in -4..=4isize {
        if offset == 0 {
            continue;
        }

        let row = origin.row as isize + offset * dr;
        let col = origin.col as isize + offset * dc;
        let Some(None) = view.cell(row, col) else {
            continue;
        };

        let extension = Move {
            row: row as usize,
            col: col as usize,
        };
        let Some(next) = view.with_black(extension) else {
            continue;
        };

        if creates_exact_five(&next, extension)
            || !creates_straight_four_containing_origin(&next, origin, extension, (dr, dc))
        {
            continue;
        }

        if forbidden_reason_in_view(&next, extension, depth_remaining - 1).is_none() {
            return true;
        }
    }

    false
}

fn creates_straight_four_containing_origin(
    view: &RenjuView<'_>,
    origin: Move,
    extension: Move,
    (dr, dc): (isize, isize),
) -> bool {
    let before = count_black(view, extension, (-dr, -dc)) as isize;
    let after = count_black(view, extension, (dr, dc)) as isize;
    if 1 + before + after != 4 {
        return false;
    }

    let start_offset = -before;
    let end_offset = after;
    let origin_offset = if dr != 0 {
        (origin.row as isize - extension.row as isize) / dr
    } else {
        (origin.col as isize - extension.col as isize) / dc
    };
    if origin_offset < start_offset || origin_offset > end_offset {
        return false;
    }

    let left = Move {
        row: (extension.row as isize + (start_offset - 1) * dr) as usize,
        col: (extension.col as isize + (start_offset - 1) * dc) as usize,
    };
    let right = Move {
        row: (extension.row as isize + (end_offset + 1) * dr) as usize,
        col: (extension.col as isize + (end_offset + 1) * dc) as usize,
    };

    point_in_bounds(view, left)
        && point_in_bounds(view, right)
        && view.is_empty(left)
        && view.is_empty(right)
        && legal_exact_five_completion(view, left)
        && legal_exact_five_completion(view, right)
}

fn point_in_bounds(view: &RenjuView<'_>, mv: Move) -> bool {
    mv.row < view.size() && mv.col < view.size()
}
