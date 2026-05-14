use gomoku_core::{Board, Color, GameResult, Move, Variant, DIRS};

pub(crate) fn evaluate_pattern_scan(board: &Board, color: Color) -> i32 {
    if let GameResult::Winner(w) = &board.result {
        return if *w == color { 2_000_000 } else { -2_000_000 };
    }
    if board.result == GameResult::Draw {
        return 0;
    }

    let scores = pattern_scores_scan(board);
    match color {
        Color::Black => scores.black - scores.white,
        Color::White => scores.white - scores.black,
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PatternFrame {
    board: Board,
    windows: Vec<PatternWindow>,
    windows_by_cell: Vec<Vec<usize>>,
    black_legal: Vec<bool>,
    black_score: i32,
    white_score: i32,
    undo_stack: Vec<PatternDelta>,
}

impl PatternFrame {
    pub(crate) fn from_board(board: &Board) -> Self {
        let size = board.config.board_size;
        let black_legal = black_legal_moves(board);
        let mut windows = Vec::new();
        let mut windows_by_cell = vec![Vec::new(); size * size];

        for &(dr, dc) in &DIRS {
            for row in 0..size as isize {
                for col in 0..size as isize {
                    let end_row = row + dr * 4;
                    let end_col = col + dc * 4;
                    if !in_bounds(board, end_row, end_col) {
                        continue;
                    }

                    let mut cells = [Move { row: 0, col: 0 }; 5];
                    for offset in 0..5isize {
                        cells[offset as usize] = Move {
                            row: (row + dr * offset) as usize,
                            col: (col + dc * offset) as usize,
                        };
                    }

                    let window_index = windows.len();
                    for cell in cells {
                        windows_by_cell[cell_index(size, cell)].push(window_index);
                    }
                    windows.push(score_window(board, &black_legal, cells));
                }
            }
        }

        let mut frame = Self {
            board: board.clone(),
            windows,
            windows_by_cell,
            black_legal,
            black_score: 0,
            white_score: 0,
            undo_stack: Vec::with_capacity(size * size),
        };
        frame.recompute_totals();
        frame
    }

    pub(crate) fn score_for(&self, color: Color) -> i32 {
        if let GameResult::Winner(w) = &self.board.result {
            return if *w == color { 2_000_000 } else { -2_000_000 };
        }
        if self.board.result == GameResult::Draw {
            return 0;
        }

        match color {
            Color::Black => self.black_score - self.white_score,
            Color::White => self.white_score - self.black_score,
        }
    }

    pub(crate) fn apply_trusted_legal_move(&mut self, mv: Move) -> GameResult {
        let size = self.board.config.board_size;
        let result = self.board.apply_trusted_legal_move(mv);
        let previous_black_legal = self.refresh_black_legality_near_move(mv);
        let previous_windows =
            self.refresh_windows_for_move_and_legal_changes(mv, &previous_black_legal);
        self.undo_stack.push(PatternDelta {
            mv,
            previous_black_legal,
            previous_windows,
        });
        debug_assert!(!self.black_legal[cell_index(size, mv)]);
        result
    }

    pub(crate) fn undo_move(&mut self, mv: Move) {
        let delta = self
            .undo_stack
            .pop()
            .expect("pattern frame undo_move called without matching apply");
        debug_assert_eq!(delta.mv, mv, "pattern frame undo_move mismatch");

        self.board.undo_move(mv);
        for (index, previous) in delta.previous_black_legal {
            self.black_legal[index] = previous;
        }
        for (window_index, previous) in delta.previous_windows {
            let current = std::mem::replace(&mut self.windows[window_index], previous);
            self.black_score -= current.black_score;
            self.white_score -= current.white_score;
            self.black_score += self.windows[window_index].black_score;
            self.white_score += self.windows[window_index].white_score;
        }
    }

    fn recompute_totals(&mut self) {
        self.black_score = self.windows.iter().map(|window| window.black_score).sum();
        self.white_score = self.windows.iter().map(|window| window.white_score).sum();
    }

    fn refresh_black_legality_near_move(&mut self, mv: Move) -> Vec<(usize, bool)> {
        let size = self.board.config.board_size;
        let mut previous = Vec::new();
        for index in affected_legality_indices(&self.board, mv) {
            let cell = Move {
                row: index / size,
                col: index % size,
            };
            let is_legal = self.board.is_empty(cell.row, cell.col)
                && self.board.is_legal_for_color(cell, Color::Black);
            if self.black_legal[index] != is_legal {
                previous.push((index, self.black_legal[index]));
                self.black_legal[index] = is_legal;
            }
        }
        previous
    }

    fn refresh_windows_for_move_and_legal_changes(
        &mut self,
        mv: Move,
        previous_black_legal: &[(usize, bool)],
    ) -> Vec<(usize, PatternWindow)> {
        let size = self.board.config.board_size;
        let mut marked = vec![false; self.windows.len()];
        let mut affected = Vec::new();

        for &window_index in &self.windows_by_cell[cell_index(size, mv)] {
            if !marked[window_index] {
                marked[window_index] = true;
                affected.push(window_index);
            }
        }
        for &(index, _) in previous_black_legal {
            for &window_index in &self.windows_by_cell[index] {
                if !marked[window_index] {
                    marked[window_index] = true;
                    affected.push(window_index);
                }
            }
        }

        let mut previous = Vec::with_capacity(affected.len());
        for window_index in affected {
            let cells = self.windows[window_index].cells;
            let next = score_window(&self.board, &self.black_legal, cells);
            let current = std::mem::replace(&mut self.windows[window_index], next);
            self.black_score -= current.black_score;
            self.white_score -= current.white_score;
            self.black_score += self.windows[window_index].black_score;
            self.white_score += self.windows[window_index].white_score;
            previous.push((window_index, current));
        }
        previous
    }
}

#[derive(Debug, Clone)]
struct PatternDelta {
    mv: Move,
    previous_black_legal: Vec<(usize, bool)>,
    previous_windows: Vec<(usize, PatternWindow)>,
}

#[derive(Debug, Clone, Copy)]
struct PatternWindow {
    cells: [Move; 5],
    black_score: i32,
    white_score: i32,
}

#[derive(Default)]
struct PatternScores {
    black: i32,
    white: i32,
}

fn pattern_scores_scan(board: &Board) -> PatternScores {
    let size = board.config.board_size as isize;
    let mut scores = PatternScores::default();
    let mut legality_cache = PatternLegalityCache::new(board);

    for &(dr, dc) in &DIRS {
        for row in 0..size {
            for col in 0..size {
                let end_row = row + dr * 4;
                let end_col = col + dc * 4;
                if !in_bounds(board, end_row, end_col) {
                    continue;
                }

                let mut black_count = 0usize;
                let mut white_count = 0usize;
                let mut empty_moves = [Move { row: 0, col: 0 }; 5];
                let mut empty_count = 0usize;
                for offset in 0..5isize {
                    let r = (row + dr * offset) as usize;
                    let c = (col + dc * offset) as usize;
                    match board.cell(r, c) {
                        Some(Color::Black) => black_count += 1,
                        Some(Color::White) => white_count += 1,
                        None => {
                            empty_moves[empty_count] = Move { row: r, col: c };
                            empty_count += 1;
                        }
                    }
                }

                if black_count > 0 && white_count > 0 {
                    continue;
                }

                if black_count >= 2 {
                    scores.black += score_pattern_window(
                        black_count,
                        legality_cache.count_legal_black_moves(board, &empty_moves[..empty_count]),
                    );
                } else if white_count >= 2 {
                    scores.white += score_pattern_window(white_count, empty_count as i32);
                }
            }
        }
    }

    scores
}

fn black_legal_moves(board: &Board) -> Vec<bool> {
    let size = board.config.board_size;
    let mut legal = vec![false; size * size];
    for row in 0..size {
        for col in 0..size {
            let mv = Move { row, col };
            legal[cell_index(size, mv)] =
                board.is_empty(row, col) && board.is_legal_for_color(mv, Color::Black);
        }
    }
    legal
}

fn affected_legality_indices(board: &Board, mv: Move) -> Vec<usize> {
    let size = board.config.board_size;
    let radius = board.config.win_length;
    let mut seen = vec![false; size * size];
    let mut indices = Vec::with_capacity(1 + DIRS.len() * radius * 2);
    push_legality_index(size, mv, &mut seen, &mut indices);

    for &(dr, dc) in &DIRS {
        for distance in 1..=radius as isize {
            for sign in [-1isize, 1] {
                let row = mv.row as isize + dr * distance * sign;
                let col = mv.col as isize + dc * distance * sign;
                if in_bounds(board, row, col) {
                    push_legality_index(
                        size,
                        Move {
                            row: row as usize,
                            col: col as usize,
                        },
                        &mut seen,
                        &mut indices,
                    );
                }
            }
        }
    }

    indices
}

fn push_legality_index(size: usize, mv: Move, seen: &mut [bool], indices: &mut Vec<usize>) {
    let index = cell_index(size, mv);
    if !seen[index] {
        seen[index] = true;
        indices.push(index);
    }
}

fn score_window(board: &Board, black_legal: &[bool], cells: [Move; 5]) -> PatternWindow {
    let size = board.config.board_size;
    let mut black_count = 0usize;
    let mut white_count = 0usize;
    let mut empty_count = 0i32;
    let mut legal_black_empty_count = 0i32;

    for cell in cells {
        match board.cell(cell.row, cell.col) {
            Some(Color::Black) => black_count += 1,
            Some(Color::White) => white_count += 1,
            None => {
                empty_count += 1;
                if black_legal[cell_index(size, cell)] {
                    legal_black_empty_count += 1;
                }
            }
        }
    }

    let (black_score, white_score) = if black_count > 0 && white_count > 0 {
        (0, 0)
    } else if black_count >= 2 {
        (
            score_pattern_window(black_count, legal_black_empty_count),
            0,
        )
    } else if white_count >= 2 {
        (0, score_pattern_window(white_count, empty_count))
    } else {
        (0, 0)
    };

    PatternWindow {
        cells,
        black_score,
        white_score,
    }
}

fn in_bounds(board: &Board, row: isize, col: isize) -> bool {
    row >= 0
        && row < board.config.board_size as isize
        && col >= 0
        && col < board.config.board_size as isize
}

fn cell_index(size: usize, mv: Move) -> usize {
    mv.row * size + mv.col
}

fn score_pattern_window(player_count: usize, legal_empty_count: i32) -> i32 {
    if player_count >= 5 {
        return 1_000_000;
    }
    if legal_empty_count == 0 {
        return 0;
    }

    match player_count {
        4 => 12_000 * legal_empty_count,
        3 => 1_000 * legal_empty_count,
        2 => 80 * legal_empty_count,
        _ => 0,
    }
}

struct PatternLegalityCache {
    board_size: usize,
    renju_black: Option<Vec<Option<bool>>>,
}

impl PatternLegalityCache {
    fn new(board: &Board) -> Self {
        let needs_exact_renju_black = board.config.variant == Variant::Renju;
        let renju_black = needs_exact_renju_black
            .then(|| vec![None; board.config.board_size * board.config.board_size]);

        Self {
            board_size: board.config.board_size,
            renju_black,
        }
    }

    fn count_legal_black_moves(&mut self, board: &Board, moves: &[Move]) -> i32 {
        moves
            .iter()
            .filter(|&&mv| self.is_legal_black_move(board, mv))
            .count() as i32
    }

    fn is_legal_black_move(&mut self, board: &Board, mv: Move) -> bool {
        let Some(cache) = &mut self.renju_black else {
            return true;
        };

        let index = mv.row * self.board_size + mv.col;
        if let Some(is_legal) = cache[index] {
            return is_legal;
        }

        let is_legal = board.is_legal_for_color(mv, Color::Black);
        cache[index] = Some(is_legal);
        is_legal
    }
}

#[cfg(test)]
mod tests {
    use gomoku_core::{Board, Color, Move, RuleConfig, Variant};

    #[test]
    fn scan_eval_scores_terminal_winner() {
        let board = Board::new(RuleConfig::default());

        assert_eq!(super::evaluate_pattern_scan(&board, Color::Black), 0);
    }

    #[test]
    fn cached_frame_matches_scan_after_apply_and_undo() {
        let sequences = [
            vec!["H8", "G8", "H9", "G9", "H7", "G10"],
            vec![
                "A1", "G1", "C1", "A15", "D1", "C15", "E1", "E15", "F1", "G15",
            ],
            vec![
                "H8", "I8", "H9", "I9", "H10", "I10", "G10", "J10", "F10", "K10",
            ],
        ];

        for sequence in sequences {
            let mut board = Board::new(RuleConfig {
                variant: Variant::Renju,
                ..RuleConfig::default()
            });
            let mut frame = super::PatternFrame::from_board(&board);
            let moves = sequence.into_iter().map(mv).collect::<Vec<_>>();

            for &next in &moves {
                board.apply_move(next).unwrap();
                frame.apply_trusted_legal_move(next);
                assert_matches_scan(&frame, &board);
            }

            for &previous in moves.iter().rev() {
                board.undo_move(previous);
                frame.undo_move(previous);
                assert_matches_scan(&frame, &board);
            }
        }
    }

    fn assert_matches_scan(frame: &super::PatternFrame, board: &Board) {
        assert_eq!(
            frame.score_for(Color::Black),
            super::evaluate_pattern_scan(board, Color::Black),
            "black cached pattern eval diverged"
        );
        assert_eq!(
            frame.score_for(Color::White),
            super::evaluate_pattern_scan(board, Color::White),
            "white cached pattern eval diverged"
        );
    }

    fn mv(coord: &str) -> Move {
        let bytes = coord.as_bytes();
        let col = (bytes[0].to_ascii_uppercase() - b'A') as usize;
        let row: usize = coord[1..].parse().unwrap();
        Move { row: row - 1, col }
    }
}
