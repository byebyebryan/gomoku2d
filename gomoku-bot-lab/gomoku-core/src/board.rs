use serde::{Deserialize, Serialize};

use crate::rules::{RuleConfig, Variant};
use crate::zobrist::ZobristTable;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    White = 1,
}

pub const DIRS: [(isize, isize); 4] = [(0, 1), (1, 0), (1, 1), (1, -1)];

impl Color {
    pub fn opponent(self) -> Color {
        match self {
            Color::Black => Color::White,
            Color::White => Color::Black,
        }
    }

    pub fn to_char(self) -> char {
        match self {
            Color::Black => 'B',
            Color::White => 'W',
        }
    }
}

pub type Cell = Option<Color>;

fn bit_word_count(board_size: usize) -> usize {
    (board_size * board_size).div_ceil(BITS_PER_WORD)
}

fn word_mask(idx: usize) -> (usize, u64) {
    (idx / BITS_PER_WORD, 1u64 << (idx % BITS_PER_WORD))
}

fn bit_is_set(bits: &[u64], idx: usize) -> bool {
    let (word, mask) = word_mask(idx);
    bits[word] & mask != 0
}

fn set_bit(bits: &mut [u64], idx: usize) {
    let (word, mask) = word_mask(idx);
    bits[word] |= mask;
}

fn clear_bit(bits: &mut [u64], idx: usize) {
    let (word, mask) = word_mask(idx);
    bits[word] &= !mask;
}

fn for_each_set_bit(bits: &[u64], board_size: usize, mut f: impl FnMut(usize, usize)) {
    let cell_count = board_size * board_size;
    for (word_idx, &word) in bits.iter().enumerate() {
        let mut remaining = word;
        while remaining != 0 {
            let bit_idx = remaining.trailing_zeros() as usize;
            let idx = word_idx * BITS_PER_WORD + bit_idx;
            if idx >= cell_count {
                return;
            }
            f(idx / board_size, idx % board_size);
            remaining &= remaining - 1;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Move {
    pub row: usize,
    pub col: usize,
}

impl Move {
    /// Convert to display notation, e.g. `Move { row: 7, col: 7 }` → `"H8"`.
    pub fn to_notation(self) -> String {
        let col_char = (b'A' + self.col as u8) as char;
        format!("{}{}", col_char, self.row + 1)
    }

    /// Parse display notation, e.g. `"H8"` → `Move { row: 7, col: 7 }`.
    pub fn from_notation(s: &str) -> Result<Self, String> {
        let s = s.trim();
        if s.len() < 2 {
            return Err(format!("invalid notation: '{s}'"));
        }
        let col_char = s.chars().next().unwrap().to_ascii_uppercase();
        if !col_char.is_ascii_uppercase() {
            return Err(format!("invalid column in notation: '{s}'"));
        }
        let col = (col_char as u8 - b'A') as usize;
        let row: usize = s[1..]
            .parse()
            .map_err(|_| format!("invalid row in notation: '{s}'"))?;
        if row == 0 {
            return Err(format!("row in notation is 1-indexed, got 0: '{s}'"));
        }
        Ok(Move { row: row - 1, col })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameResult {
    Ongoing,
    Winner(Color),
    Draw,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoveError {
    OutOfBounds,
    Occupied,
    GameOver,
    /// Renju: Black's move violates a restriction (overline, double-four, or double-three).
    Forbidden,
}

impl std::fmt::Display for MoveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MoveError::OutOfBounds => write!(f, "move out of bounds"),
            MoveError::Occupied => write!(f, "cell already occupied"),
            MoveError::GameOver => write!(f, "game is already over"),
            MoveError::Forbidden => write!(f, "move forbidden by Renju rules"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Board {
    pub config: RuleConfig,
    black_bits: Vec<u64>,
    white_bits: Vec<u64>,
    pub history: Vec<Move>,
    pub current_player: Color,
    pub result: GameResult,
}

const BITS_PER_WORD: usize = u64::BITS as usize;

impl Board {
    pub fn new(config: RuleConfig) -> Self {
        let size = config.board_size;
        let words = bit_word_count(size);
        Self {
            black_bits: vec![0; words],
            white_bits: vec![0; words],
            history: Vec::new(),
            current_player: Color::Black,
            result: GameResult::Ongoing,
            config,
        }
    }

    pub fn cell(&self, row: usize, col: usize) -> Cell {
        let idx = self.index(row, col);
        if bit_is_set(&self.black_bits, idx) {
            Some(Color::Black)
        } else if bit_is_set(&self.white_bits, idx) {
            Some(Color::White)
        } else {
            None
        }
    }

    pub fn is_empty(&self, row: usize, col: usize) -> bool {
        self.is_empty_at(row, col)
    }

    pub fn has_color(&self, row: usize, col: usize, color: Color) -> bool {
        self.has_color_at(row, col, color)
    }

    pub fn for_each_occupied(&self, mut f: impl FnMut(usize, usize, Color)) {
        let size = self.config.board_size;
        for_each_set_bit(&self.black_bits, size, |row, col| {
            f(row, col, Color::Black);
        });
        for_each_set_bit(&self.white_bits, size, |row, col| {
            f(row, col, Color::White);
        });
    }

    pub fn for_each_occupied_color(&self, color: Color, f: impl FnMut(usize, usize)) {
        for_each_set_bit(self.bits_for_color(color), self.config.board_size, f);
    }

    fn index(&self, row: usize, col: usize) -> usize {
        debug_assert!(row < self.config.board_size);
        debug_assert!(col < self.config.board_size);
        row * self.config.board_size + col
    }

    fn is_empty_at(&self, row: usize, col: usize) -> bool {
        let idx = self.index(row, col);
        !bit_is_set(&self.black_bits, idx) && !bit_is_set(&self.white_bits, idx)
    }

    fn has_color_at(&self, row: usize, col: usize, color: Color) -> bool {
        let idx = self.index(row, col);
        bit_is_set(self.bits_for_color(color), idx)
    }

    fn bits_for_color(&self, color: Color) -> &[u64] {
        match color {
            Color::Black => &self.black_bits,
            Color::White => &self.white_bits,
        }
    }

    fn bits_for_color_mut(&mut self, color: Color) -> &mut [u64] {
        match color {
            Color::Black => &mut self.black_bits,
            Color::White => &mut self.white_bits,
        }
    }

    fn set_cell(&mut self, mv: Move, color: Color) {
        let idx = self.index(mv.row, mv.col);
        set_bit(self.bits_for_color_mut(color), idx);
    }

    fn clear_cell(&mut self, mv: Move) {
        let idx = self.index(mv.row, mv.col);
        clear_bit(&mut self.black_bits, idx);
        clear_bit(&mut self.white_bits, idx);
    }

    fn cell_at_index(&self, idx: usize) -> Cell {
        if bit_is_set(&self.black_bits, idx) {
            Some(Color::Black)
        } else if bit_is_set(&self.white_bits, idx) {
            Some(Color::White)
        } else {
            None
        }
    }

    pub fn is_legal(&self, mv: Move) -> bool {
        self.is_legal_for(mv, self.current_player)
    }

    pub fn legal_moves(&self) -> Vec<Move> {
        self.legal_moves_for(self.current_player)
    }

    pub fn immediate_winning_moves_for(&self, color: Color) -> Vec<Move> {
        if self.result != GameResult::Ongoing {
            return vec![];
        }

        let mut wins = Vec::new();

        for mv in self.nearby_empty_moves(self.config.win_length.saturating_sub(1)) {
            if self.probe_immediate_winning_move(mv, color) {
                wins.push(mv);
            }
        }
        wins
    }

    pub fn has_multiple_immediate_winning_moves_for(&self, color: Color) -> bool {
        if self.result != GameResult::Ongoing {
            return false;
        }

        let mut wins = 0;
        let radius = self.config.win_length.saturating_sub(1) as isize;
        let size = self.config.board_size;
        let mut seen = vec![false; size * size];

        for row in 0..size {
            for col in 0..size {
                if self.is_empty_at(row, col) {
                    continue;
                }

                for dr in -radius..=radius {
                    for dc in -radius..=radius {
                        let r = row as isize + dr;
                        let c = col as isize + dc;
                        if r < 0 || r >= size as isize || c < 0 || c >= size as isize {
                            continue;
                        }

                        let mv = Move {
                            row: r as usize,
                            col: c as usize,
                        };
                        let idx = mv.row * size + mv.col;
                        if seen[idx] || !self.is_empty_at(mv.row, mv.col) {
                            continue;
                        }
                        seen[idx] = true;

                        if self.probe_immediate_winning_move(mv, color) {
                            wins += 1;
                            if wins >= 2 {
                                return true;
                            }
                        }
                    }
                }
            }
        }

        false
    }

    fn mark_nearby_empty_moves(&self, row: usize, col: usize, radius: isize, seen: &mut [bool]) {
        let size = self.config.board_size;
        let row = row as isize;
        let col = col as isize;
        for dr in -radius..=radius {
            for dc in -radius..=radius {
                let r = row + dr;
                let c = col + dc;
                if r < 0 || r >= size as isize || c < 0 || c >= size as isize {
                    continue;
                }

                let mv = Move {
                    row: r as usize,
                    col: c as usize,
                };
                if !self.is_empty_at(mv.row, mv.col) {
                    continue;
                }
                seen[mv.row * size + mv.col] = true;
            }
        }
    }

    fn collect_seen_moves(&self, seen: &[bool]) -> Vec<Move> {
        let size = self.config.board_size;
        let mut moves = Vec::new();
        for row in 0..size {
            for col in 0..size {
                if seen[row * size + col] {
                    moves.push(Move { row, col });
                }
            }
        }
        moves
    }

    fn probe_immediate_winning_move(&self, mv: Move, color: Color) -> bool {
        if !self.is_legal_for(mv, color) {
            return false;
        }

        let row = mv.row as isize;
        let col = mv.col as isize;
        DIRS.iter().any(|&(dr, dc)| {
            let count = 1
                + self.count_direction(row, col, dr, dc, color)
                + self.count_direction(row, col, -dr, -dc, color);
            self.is_winning_run(count as usize, color)
        })
    }

    pub fn forbidden_moves_for_current_player(&self) -> Vec<Move> {
        if self.result != GameResult::Ongoing {
            return vec![];
        }
        if self.config.variant != Variant::Renju {
            return vec![];
        }
        if self.current_player != Color::Black {
            return vec![];
        }

        self.renju_forbidden_candidate_moves()
            .into_iter()
            .filter(|&mv| self.is_renju_forbidden_at(mv))
            .collect()
    }

    pub fn winning_line(&self) -> Vec<Move> {
        let GameResult::Winner(color) = self.result else {
            return vec![];
        };
        let Some(&last_move) = self.history.last() else {
            return vec![];
        };

        for (dr, dc) in DIRS {
            let line = self.line_through(last_move, dr, dc, color);
            if self.is_winning_run(line.len(), color) {
                return line;
            }
        }

        vec![]
    }

    fn line_through(&self, mv: Move, dr: isize, dc: isize, color: Color) -> Vec<Move> {
        let mut before = self.moves_in_direction(mv, -dr, -dc, color);
        before.reverse();
        before.push(mv);
        before.extend(self.moves_in_direction(mv, dr, dc, color));
        before
    }

    fn moves_in_direction(&self, mv: Move, dr: isize, dc: isize, color: Color) -> Vec<Move> {
        let size = self.config.board_size as isize;
        let mut moves = Vec::new();
        let (mut row, mut col) = (mv.row as isize + dr, mv.col as isize + dc);

        while row >= 0 && row < size && col >= 0 && col < size {
            let next = Move {
                row: row as usize,
                col: col as usize,
            };
            if !self.has_color_at(next.row, next.col, color) {
                break;
            }

            moves.push(next);
            row += dr;
            col += dc;
        }

        moves
    }

    fn is_winning_run(&self, count: usize, color: Color) -> bool {
        let win_len = self.config.win_length;
        if self.config.variant == Variant::Renju && color == Color::Black {
            count == win_len
        } else {
            count >= win_len
        }
    }

    fn nearby_empty_moves(&self, radius: usize) -> Vec<Move> {
        if self.result != GameResult::Ongoing {
            return vec![];
        }

        let size = self.config.board_size;
        let radius = radius as isize;
        let mut seen = vec![false; size * size];
        let mut has_stone = false;

        self.for_each_occupied(|row, col, _| {
            has_stone = true;
            self.mark_nearby_empty_moves(row, col, radius, &mut seen);
        });

        if !has_stone {
            return vec![];
        }

        self.collect_seen_moves(&seen)
    }

    fn nearby_empty_moves_for_color(&self, color: Color, radius: usize) -> Vec<Move> {
        if self.result != GameResult::Ongoing {
            return vec![];
        }

        let size = self.config.board_size;
        let radius = radius as isize;
        let mut seen = vec![false; size * size];
        let mut has_anchor = false;

        self.for_each_occupied_color(color, |row, col| {
            has_anchor = true;
            self.mark_nearby_empty_moves(row, col, radius, &mut seen);
        });

        if !has_anchor {
            return vec![];
        }

        self.collect_seen_moves(&seen)
    }

    fn renju_forbidden_candidate_moves(&self) -> Vec<Move> {
        self.nearby_empty_moves_for_color(Color::Black, 2)
    }

    fn legal_moves_for(&self, color: Color) -> Vec<Move> {
        if self.result != GameResult::Ongoing {
            return vec![];
        }
        let size = self.config.board_size;
        let mut moves = Vec::with_capacity(size * size);
        for row in 0..size {
            for col in 0..size {
                if self.is_empty_at(row, col) {
                    let mv = Move { row, col };
                    if !self.is_legal_for(mv, color) {
                        continue;
                    }
                    moves.push(mv);
                }
            }
        }
        moves
    }

    fn is_legal_for(&self, mv: Move, color: Color) -> bool {
        if self.result != GameResult::Ongoing {
            return false;
        }
        let size = self.config.board_size;
        if mv.row >= size || mv.col >= size {
            return false;
        }
        if !self.is_empty_at(mv.row, mv.col) {
            return false;
        }
        if self.config.variant == Variant::Renju
            && color == Color::Black
            && self.can_be_renju_forbidden_at(mv)
            && self.is_renju_forbidden_at(mv)
        {
            return false;
        }
        true
    }

    pub fn apply_move(&mut self, mv: Move) -> Result<GameResult, MoveError> {
        if self.result != GameResult::Ongoing {
            return Err(MoveError::GameOver);
        }
        let size = self.config.board_size;
        if mv.row >= size || mv.col >= size {
            return Err(MoveError::OutOfBounds);
        }
        if !self.is_empty_at(mv.row, mv.col) {
            return Err(MoveError::Occupied);
        }

        let color = self.current_player;
        if self.config.variant == Variant::Renju
            && color == Color::Black
            && self.can_be_renju_forbidden_at(mv)
            && self.is_renju_forbidden_at(mv)
        {
            return Err(MoveError::Forbidden);
        }

        Ok(self.apply_trusted_legal_move(mv))
    }

    /// Apply an already-validated legal move without repeating legality checks.
    ///
    /// This is intended for trusted in-process callers such as search, where
    /// candidate moves have already passed the relevant legality gate.
    pub fn apply_trusted_legal_move(&mut self, mv: Move) -> GameResult {
        debug_assert_eq!(self.result, GameResult::Ongoing);
        let size = self.config.board_size;
        debug_assert!(mv.row < size);
        debug_assert!(mv.col < size);
        debug_assert!(self.is_empty_at(mv.row, mv.col));

        let color = self.current_player;
        self.set_cell(mv, color);
        self.history.push(mv);

        if self.check_win(mv, color) {
            self.result = GameResult::Winner(color);
        } else if self.history.len() == size * size {
            self.result = GameResult::Draw;
        }

        self.current_player = color.opponent();
        self.result.clone()
    }

    fn check_win(&self, mv: Move, color: Color) -> bool {
        for (dr, dc) in DIRS {
            let count = 1
                + self.count_direction(mv.row as isize, mv.col as isize, dr, dc, color)
                + self.count_direction(mv.row as isize, mv.col as isize, -dr, -dc, color);
            if self.is_winning_run(count as usize, color) {
                return true;
            }
        }
        false
    }

    fn count_direction(&self, row: isize, col: isize, dr: isize, dc: isize, color: Color) -> isize {
        let size = self.config.board_size as isize;
        let mut count = 0;
        let (mut r, mut c) = (row + dr, col + dc);
        while r >= 0 && r < size && c >= 0 && c < size {
            if self.has_color_at(r as usize, c as usize, color) {
                count += 1;
                r += dr;
                c += dc;
            } else {
                break;
            }
        }
        count
    }

    // --- Renju restriction helpers ---
    //
    // All functions below are called BEFORE the stone is placed.
    // `cell_virtual` treats (vrow, vcol) as already containing `vcolor`.

    fn can_be_renju_forbidden_at(&self, mv: Move) -> bool {
        self.has_two_black_stones_on_any_axis(mv)
    }

    fn has_two_black_stones_on_any_axis(&self, mv: Move) -> bool {
        let size = self.config.board_size as isize;
        let row = mv.row as isize;
        let col = mv.col as isize;

        for (dr, dc) in DIRS {
            let mut black = 0;
            for step in [-4, -3, -2, -1, 1, 2, 3, 4] {
                let r = row + dr * step;
                let c = col + dc * step;
                if r < 0 || r >= size || c < 0 || c >= size {
                    continue;
                }
                if self.has_color_at(r as usize, c as usize, Color::Black) {
                    black += 1;
                    if black >= 2 {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn cell_virtual(
        &self,
        r: isize,
        c: isize,
        vrow: isize,
        vcol: isize,
        vcolor: Color,
    ) -> Option<Cell> {
        let size = self.config.board_size as isize;
        if r < 0 || r >= size || c < 0 || c >= size {
            return None;
        }
        if r == vrow && c == vcol {
            return Some(Some(vcolor));
        }
        let idx = self.index(r as usize, c as usize);
        if bit_is_set(&self.black_bits, idx) {
            Some(Some(Color::Black))
        } else if bit_is_set(&self.white_bits, idx) {
            Some(Some(Color::White))
        } else {
            Some(None)
        }
    }

    /// True if placing a Black stone at `mv` would create an overline, double-four, or
    /// double-three. Winning moves (exactly 5-in-a-row) are never forbidden.
    fn is_renju_forbidden_at(&self, mv: Move) -> bool {
        let row = mv.row as isize;
        let col = mv.col as isize;
        let win_len = self.config.win_length as isize;
        let color = Color::Black;

        let mut creates_win = false;

        for (dr, dc) in DIRS {
            let run = 1
                + self.count_direction(row, col, dr, dc, color)
                + self.count_direction(row, col, -dr, -dc, color);
            if run > 5 {
                return true;
            } // overline → always forbidden
            if run == win_len {
                creates_win = true;
            }
        }

        if creates_win {
            return false;
        } // winning move takes priority

        let mut four_dirs = 0u32;
        let mut three_dirs = 0u32;
        for (dr, dc) in DIRS {
            if self.has_four_at(row, col, dr, dc, color) {
                four_dirs += 1;
            }
            if self.has_open_three_at(row, col, dr, dc, color) {
                three_dirs += 1;
            }
        }

        four_dirs >= 2 || three_dirs >= 2
    }

    /// True if placing `color` at (row, col) creates a four in direction (dr, dc).
    /// A four = any window of 5 cells containing (row,col) with exactly 4 `color` stones
    /// and 1 empty cell (no opponent stones in the window).
    fn has_four_at(&self, row: isize, col: isize, dr: isize, dc: isize, color: Color) -> bool {
        for stone_pos in 0..=4isize {
            let start = -stone_pos;
            let mut black = 0u32;
            let mut empty = 0u32;
            let mut valid = true;
            for i in 0..5isize {
                let r = row + (start + i) * dr;
                let c = col + (start + i) * dc;
                match self.cell_virtual(r, c, row, col, color) {
                    None => {
                        valid = false;
                        break;
                    }
                    Some(Some(cl)) if cl == color => black += 1,
                    Some(None) => empty += 1,
                    Some(Some(_)) => {
                        valid = false;
                        break;
                    } // opponent
                }
            }
            if valid && black == 4 && empty == 1 {
                return true;
            }
        }
        false
    }

    /// True if placing `color` at (row, col) creates an open three in direction (dr, dc).
    /// An open three = any 6-cell window where (row,col) is at an inner position (1–4),
    /// both endpoints are in-bounds and empty, inner 4 cells have exactly 3 `color` + 1 empty,
    /// and no opponent stones appear anywhere in the window.
    fn has_open_three_at(
        &self,
        row: isize,
        col: isize,
        dr: isize,
        dc: isize,
        color: Color,
    ) -> bool {
        for stone_pos in 1..=4isize {
            let start = -stone_pos;
            let mut black = 0u32;
            let mut empty = 0u32;
            let mut valid = true;
            for i in 0..6isize {
                let r = row + (start + i) * dr;
                let c = col + (start + i) * dc;
                if i == 0 || i == 5 {
                    // Endpoints must be on-board and empty
                    match self.cell_virtual(r, c, row, col, color) {
                        Some(None) => {}
                        _ => {
                            valid = false;
                            break;
                        }
                    }
                } else {
                    match self.cell_virtual(r, c, row, col, color) {
                        None => {
                            valid = false;
                            break;
                        }
                        Some(Some(cl)) if cl == color => black += 1,
                        Some(None) => empty += 1,
                        Some(Some(_)) => {
                            valid = false;
                            break;
                        } // opponent
                    }
                }
            }
            if valid && black == 3 && empty == 1 {
                return true;
            }
        }
        false
    }

    /// Serialize board state to a compact string.
    /// Format: "<size>/<win_len>/<turn>/<cells...>"
    /// cells: '.' = empty, 'B' = black, 'W' = white
    pub fn to_fen(&self) -> String {
        let turn = match self.current_player {
            Color::Black => 'B',
            Color::White => 'W',
        };
        let size = self.config.board_size;
        let mut cells = String::with_capacity(size * size);
        for idx in 0..size * size {
            cells.push(self.cell_at_index(idx).map_or('.', Color::to_char));
        }
        format!(
            "{}/{}/{}/{}",
            self.config.board_size, self.config.win_length, turn, cells
        )
    }

    /// Undo the last move. Only valid if `mv` was the last move applied.
    /// Intended for use by search algorithms.
    pub fn undo_move(&mut self, mv: Move) {
        debug_assert_eq!(
            self.history.last(),
            Some(&mv),
            "undo_move called with wrong move"
        );
        self.clear_cell(mv);
        self.history.pop();
        self.current_player = self.current_player.opponent();
        self.result = GameResult::Ongoing;
    }

    /// Zobrist hash of the current position, recomputed from scratch.
    ///
    /// Intended for replay verification, debugging, and test assertions —
    /// not for hot search paths. Search code should maintain an incremental
    /// hash updated with each `apply_move`/`undo_move` call instead.
    pub fn hash(&self) -> u64 {
        let zt = ZobristTable::new(self.config.board_size);
        self.hash_with(&zt)
    }

    /// Zobrist hash of the current position using an existing table.
    pub fn hash_with(&self, zt: &ZobristTable) -> u64 {
        let mut h = 0u64;
        for_each_set_bit(&self.black_bits, self.config.board_size, |row, col| {
            h ^= zt.piece(row, col, Color::Black);
        });
        for_each_set_bit(&self.white_bits, self.config.board_size, |row, col| {
            h ^= zt.piece(row, col, Color::White);
        });
        if self.current_player == Color::White {
            h ^= zt.turn;
        }
        h
    }

    pub fn from_fen(fen: &str) -> Result<Self, String> {
        let parts: Vec<&str> = fen.splitn(4, '/').collect();
        if parts.len() != 4 {
            return Err("invalid FEN: expected 4 parts".into());
        }
        let board_size: usize = parts[0].parse().map_err(|_| "invalid board_size")?;
        let win_length: usize = parts[1].parse().map_err(|_| "invalid win_length")?;
        let turn = match parts[2] {
            "B" => Color::Black,
            "W" => Color::White,
            _ => return Err("invalid turn".into()),
        };
        let cell_str = parts[3];
        if cell_str.len() != board_size * board_size {
            return Err("cell string length mismatch".into());
        }

        let config = RuleConfig {
            board_size,
            win_length,
            ..Default::default()
        };
        let mut board = Board::new(config);
        board.current_player = turn;

        for (i, ch) in cell_str.chars().enumerate() {
            let row = i / board_size;
            let col = i % board_size;
            match ch {
                '.' => {}
                'B' => board.set_cell(Move { row, col }, Color::Black),
                'W' => board.set_cell(Move { row, col }, Color::White),
                _ => return Err(format!("invalid cell char '{ch}'")),
            }
        }
        Ok(board)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_board() -> Board {
        Board::new(RuleConfig::default())
    }

    #[test]
    fn fresh_board_is_empty() {
        let b = default_board();
        assert_eq!(b.legal_moves().len(), 225);
        assert_eq!(b.current_player, Color::Black);
        assert_eq!(b.result, GameResult::Ongoing);
    }

    #[test]
    fn color_and_cell_have_compact_representation() {
        assert_eq!(std::mem::size_of::<Color>(), 1);
        assert_eq!(std::mem::size_of::<Cell>(), 1);
    }

    #[test]
    fn occupied_cells_visit_each_stone_with_color() {
        let mut b = default_board();
        b.apply_move(Move { row: 7, col: 7 }).unwrap();
        b.apply_move(Move { row: 6, col: 8 }).unwrap();
        b.apply_move(Move { row: 5, col: 9 }).unwrap();

        let mut visited = Vec::new();
        b.for_each_occupied(|row, col, color| visited.push((row, col, color)));
        visited.sort_by_key(|&(row, col, color)| (row, col, color as u8));

        assert_eq!(
            visited,
            vec![
                (5, 9, Color::Black),
                (6, 8, Color::White),
                (7, 7, Color::Black),
            ]
        );
    }

    #[test]
    fn occupied_cells_for_color_visit_only_requested_stones() {
        let mut b = default_board();
        b.apply_move(Move { row: 7, col: 7 }).unwrap();
        b.apply_move(Move { row: 6, col: 8 }).unwrap();
        b.apply_move(Move { row: 5, col: 9 }).unwrap();

        let mut black = Vec::new();
        b.for_each_occupied_color(Color::Black, |row, col| black.push((row, col)));
        black.sort();

        assert_eq!(black, vec![(5, 9), (7, 7)]);
    }

    #[test]
    fn hash_with_reuses_existing_zobrist_table() {
        let mut b = default_board();
        b.apply_move(Move { row: 7, col: 7 }).unwrap();
        b.apply_move(Move { row: 6, col: 8 }).unwrap();
        b.apply_move(Move { row: 5, col: 9 }).unwrap();

        let zt = ZobristTable::new(b.config.board_size);

        assert_eq!(b.hash_with(&zt), b.hash());
    }

    #[test]
    fn apply_move_toggles_player() {
        let mut b = default_board();
        b.apply_move(Move { row: 7, col: 7 }).unwrap();
        assert_eq!(b.current_player, Color::White);
        b.apply_move(Move { row: 7, col: 8 }).unwrap();
        assert_eq!(b.current_player, Color::Black);
    }

    #[test]
    fn occupied_cell_rejected() {
        let mut b = default_board();
        b.apply_move(Move { row: 0, col: 0 }).unwrap();
        assert_eq!(
            b.apply_move(Move { row: 0, col: 0 }),
            Err(MoveError::Occupied)
        );
    }

    #[test]
    fn out_of_bounds_rejected() {
        let mut b = default_board();
        assert_eq!(
            b.apply_move(Move { row: 15, col: 0 }),
            Err(MoveError::OutOfBounds)
        );
    }

    #[test]
    fn win_horizontal() {
        let mut b = default_board();
        // Black: row 0, cols 0-4; White: row 1, cols 0-3
        for i in 0..4usize {
            b.apply_move(Move { row: 0, col: i }).unwrap();
            b.apply_move(Move { row: 1, col: i }).unwrap();
        }
        let result = b.apply_move(Move { row: 0, col: 4 }).unwrap();
        assert_eq!(result, GameResult::Winner(Color::Black));
    }

    #[test]
    fn win_vertical() {
        let mut b = default_board();
        for i in 0..4usize {
            b.apply_move(Move { row: i, col: 0 }).unwrap();
            b.apply_move(Move { row: i, col: 1 }).unwrap();
        }
        let result = b.apply_move(Move { row: 4, col: 0 }).unwrap();
        assert_eq!(result, GameResult::Winner(Color::Black));
    }

    #[test]
    fn win_diagonal() {
        let mut b = default_board();
        // Black diagonal (0,0),(1,1),(2,2),(3,3),(4,4); White fills safe spots
        for i in 0..4usize {
            b.apply_move(Move { row: i, col: i }).unwrap();
            b.apply_move(Move {
                row: i,
                col: (i + 5).min(14),
            })
            .unwrap();
        }
        let result = b.apply_move(Move { row: 4, col: 4 }).unwrap();
        assert_eq!(result, GameResult::Winner(Color::Black));
    }

    #[test]
    fn winning_line_empty_before_a_win() {
        let mut b = default_board();
        b.apply_move(Move { row: 7, col: 7 }).unwrap();
        assert!(b.winning_line().is_empty());
    }

    #[test]
    fn winning_line_returns_the_canonical_run() {
        let mut b = default_board();
        setup(
            &mut b,
            &[(7, 3), W[0], (7, 4), W[1], (7, 5), W[2], (7, 6), W[3]],
        );
        let result = b.apply_move(Move { row: 7, col: 7 }).unwrap();
        assert_eq!(result, GameResult::Winner(Color::Black));
        assert_eq!(
            b.winning_line(),
            vec![
                Move { row: 7, col: 3 },
                Move { row: 7, col: 4 },
                Move { row: 7, col: 5 },
                Move { row: 7, col: 6 },
                Move { row: 7, col: 7 },
            ]
        );
    }

    #[test]
    fn fen_round_trip() {
        let mut b = default_board();
        b.apply_move(Move { row: 7, col: 7 }).unwrap();
        b.apply_move(Move { row: 3, col: 3 }).unwrap();
        let fen = b.to_fen();
        let b2 = Board::from_fen(&fen).unwrap();
        assert_eq!(b2.to_fen(), fen);
        assert_eq!(b2.current_player, b.current_player);
    }

    #[test]
    fn immediate_winning_moves_for_current_player() {
        let mut b = default_board();
        setup(
            &mut b,
            &[(7, 3), W[0], (7, 4), W[1], (7, 5), W[2], (7, 6), W[3]],
        );
        assert_eq!(b.current_player, Color::Black);
        assert_eq!(
            b.immediate_winning_moves_for(Color::Black),
            vec![Move { row: 7, col: 2 }, Move { row: 7, col: 7 }]
        );
    }

    #[test]
    fn detects_multiple_immediate_winning_moves_for_player() {
        let mut fork = default_board();
        setup(
            &mut fork,
            &[(7, 3), W[0], (7, 4), W[1], (7, 5), W[2], (7, 6), W[3]],
        );
        assert!(fork.has_multiple_immediate_winning_moves_for(Color::Black));

        let mut single = default_board();
        setup(
            &mut single,
            &[(0, 0), (7, 3), (0, 1), (7, 4), (0, 2), (7, 5), (0, 3)],
        );
        assert!(!single.has_multiple_immediate_winning_moves_for(Color::Black));
    }

    #[test]
    fn immediate_winning_moves_for_non_current_player() {
        let mut b = default_board();
        setup(
            &mut b,
            &[(0, 0), (7, 3), (0, 1), (7, 4), (0, 2), (7, 5), (0, 3)],
        );
        assert_eq!(b.current_player, Color::White);
        assert_eq!(
            b.immediate_winning_moves_for(Color::Black),
            vec![Move { row: 0, col: 4 }]
        );
    }

    #[test]
    fn renju_forbidden_moves_for_current_player() {
        let mut b = renju_board();
        setup(
            &mut b,
            &[(5, 7), W[0], (6, 7), W[1], (7, 5), W[2], (7, 6), W[3]],
        );
        assert_eq!(
            b.forbidden_moves_for_current_player(),
            vec![Move { row: 7, col: 7 }]
        );
    }

    #[test]
    fn forbidden_moves_for_current_player_empty_outside_black_renju_turn() {
        let mut freestyle = default_board();
        setup(
            &mut freestyle,
            &[(5, 7), W[0], (6, 7), W[1], (7, 5), W[2], (7, 6), W[3]],
        );
        assert!(freestyle.forbidden_moves_for_current_player().is_empty());

        let mut renju = renju_board();
        renju.apply_move(Move { row: 7, col: 7 }).unwrap();
        assert_eq!(renju.current_player, Color::White);
        assert!(renju.forbidden_moves_for_current_player().is_empty());
    }

    fn renju_board() -> Board {
        Board::new(RuleConfig {
            variant: crate::rules::Variant::Renju,
            ..Default::default()
        })
    }

    fn full_scan_forbidden_moves_for_current_player(board: &Board) -> Vec<Move> {
        if board.result != GameResult::Ongoing
            || board.config.variant != Variant::Renju
            || board.current_player != Color::Black
        {
            return vec![];
        }

        let mut moves = Vec::new();
        for row in 0..board.config.board_size {
            for col in 0..board.config.board_size {
                if board.cell(row, col).is_none() {
                    let mv = Move { row, col };
                    if board.is_renju_forbidden_at(mv) {
                        moves.push(mv);
                    }
                }
            }
        }
        moves
    }

    #[test]
    fn renju_forbidden_candidate_moves_only_follow_black_stones() {
        let mut b = renju_board();
        setup(&mut b, &[(7, 7), (0, 0), (10, 10), (0, 2)]);

        let candidates = b.renju_forbidden_candidate_moves();

        assert!(candidates.contains(&Move { row: 7, col: 5 }));
        assert!(candidates.contains(&Move { row: 12, col: 12 }));
        assert!(!candidates.contains(&Move { row: 0, col: 1 }));
        assert!(!candidates.contains(&Move { row: 2, col: 2 }));
    }

    #[test]
    fn renju_forbidden_guard_rejects_single_nearby_black_stone() {
        let mut b = renju_board();
        setup(&mut b, &[(7, 7), (0, 0)]);

        assert!(!b.can_be_renju_forbidden_at(Move { row: 7, col: 9 }));
    }

    #[test]
    fn optimized_renju_forbidden_moves_match_full_scan() {
        let mut double_three = renju_board();
        setup(
            &mut double_three,
            &[(5, 7), W[0], (6, 7), W[1], (7, 5), W[2], (7, 6), W[3]],
        );

        let mut overline = renju_board();
        setup(
            &mut overline,
            &[
                (0, 0),
                W[0],
                (0, 2),
                W[1],
                (0, 3),
                W[2],
                (0, 4),
                W[3],
                (0, 5),
                W[4],
            ],
        );

        let mut white_noise = renju_board();
        setup(&mut white_noise, &[(7, 7), (0, 0), (10, 10), (0, 2)]);

        for board in [&double_three, &overline, &white_noise] {
            assert_eq!(
                board.forbidden_moves_for_current_player(),
                full_scan_forbidden_moves_for_current_player(board)
            );
        }
    }

    // Helper: make alternating moves (Black, White, Black, ...) from a list of (row, col) pairs.
    // Panics on any error.
    fn setup(board: &mut Board, moves: &[(usize, usize)]) {
        for &(row, col) in moves {
            board.apply_move(Move { row, col }).unwrap();
        }
    }

    // White stone placements that never form 5-in-a-row: row 14, every other column.
    const W: [(usize, usize); 8] = [
        (14, 0),
        (14, 2),
        (14, 4),
        (14, 6),
        (14, 8),
        (14, 10),
        (14, 12),
        (13, 1),
    ];

    #[test]
    fn renju_overline_forbidden() {
        let mut b = renju_board();
        // Black: cols 0,2,3,4,5 in row 0 (no five-in-a-row yet); White scattered.
        // Placing at col 1 would create a run of 6 (cols 0–5).
        setup(
            &mut b,
            &[
                (0, 0),
                W[0],
                (0, 2),
                W[1],
                (0, 3),
                W[2],
                (0, 4),
                W[3],
                (0, 5),
                W[4],
            ],
        );
        assert_eq!(
            b.apply_move(Move { row: 0, col: 1 }),
            Err(MoveError::Forbidden)
        );
    }

    #[test]
    fn renju_overline_not_a_win() {
        // Overline placement is forbidden, not treated as a win.
        // Black at cols 0,1,2,3 (four in a row) + col 5 (isolated). No five yet.
        // Placing at col 4 closes the gap and creates a run of 6 (cols 0–5).
        let mut b = renju_board();
        setup(
            &mut b,
            &[
                (0, 0),
                W[0],
                (0, 1),
                W[1],
                (0, 2),
                W[2],
                (0, 3),
                W[3],
                (0, 5),
                W[4],
            ],
        );
        assert_eq!(
            b.apply_move(Move { row: 0, col: 4 }),
            Err(MoveError::Forbidden)
        );
    }

    #[test]
    fn renju_double_four_forbidden() {
        let mut b = renju_board();
        // Black: (7,3),(7,4),(7,5) horizontal + (4,7),(5,7),(6,7) vertical; White scattered.
        // Placing at (7,7) creates a four in both directions simultaneously.
        setup(
            &mut b,
            &[
                (7, 3),
                W[0],
                (7, 4),
                W[1],
                (7, 5),
                W[2],
                (4, 7),
                W[3],
                (5, 7),
                W[4],
                (6, 7),
                W[5],
            ],
        );
        assert_eq!(
            b.apply_move(Move { row: 7, col: 7 }),
            Err(MoveError::Forbidden)
        );
    }

    #[test]
    fn renju_double_three_forbidden() {
        let mut b = renju_board();
        // Black: (5,7),(6,7) vertical + (7,5),(7,6) horizontal; White scattered.
        // Placing at (7,7) creates two open threes simultaneously.
        setup(
            &mut b,
            &[(5, 7), W[0], (6, 7), W[1], (7, 5), W[2], (7, 6), W[3]],
        );
        assert_eq!(
            b.apply_move(Move { row: 7, col: 7 }),
            Err(MoveError::Forbidden)
        );
    }

    #[test]
    fn renju_white_unrestricted() {
        let mut b = renju_board();
        // Same double-three shape but for White — Black moves first, White builds.
        setup(
            &mut b,
            &[
                (0, 0),
                (5, 7),
                (0, 2),
                (6, 7),
                (0, 4),
                (7, 5),
                (0, 6),
                (7, 6),
            ],
        );
        // White tries (7,7): double-three but White has no restrictions.
        assert!(b.apply_move(Move { row: 7, col: 7 }).is_ok());
    }

    #[test]
    fn renju_five_in_row_wins() {
        // Five-in-a-row is always a legal winning move for Black, even in Renju.
        let mut b = renju_board();
        setup(
            &mut b,
            &[(7, 3), W[0], (7, 4), W[1], (7, 5), W[2], (7, 6), W[3]],
        );
        let result = b.apply_move(Move { row: 7, col: 7 }).unwrap();
        assert_eq!(result, GameResult::Winner(Color::Black));
    }

    #[test]
    fn renju_forbidden_move_not_reported_as_immediate_win() {
        let mut b = renju_board();
        setup(
            &mut b,
            &[
                (0, 0),
                W[0],
                (0, 1),
                W[1],
                (0, 2),
                W[2],
                (0, 3),
                W[3],
                (0, 5),
                W[4],
            ],
        );
        assert!(!b
            .immediate_winning_moves_for(Color::Black)
            .contains(&Move { row: 0, col: 4 }));
    }

    #[test]
    fn renju_freestyle_allows_overline() {
        // In freestyle, 6-in-a-row is a win (not forbidden).
        let mut b = default_board();
        // Black: cols 0,1,2,3 + col 5 (no five); White scattered.
        setup(
            &mut b,
            &[
                (0, 0),
                W[0],
                (0, 1),
                W[1],
                (0, 2),
                W[2],
                (0, 3),
                W[3],
                (0, 5),
                W[4],
            ],
        );
        // Col 4 closes to 6-in-a-row → win in freestyle.
        let result = b.apply_move(Move { row: 0, col: 4 }).unwrap();
        assert_eq!(result, GameResult::Winner(Color::Black));
        assert_eq!(
            b.winning_line(),
            vec![
                Move { row: 0, col: 0 },
                Move { row: 0, col: 1 },
                Move { row: 0, col: 2 },
                Move { row: 0, col: 3 },
                Move { row: 0, col: 4 },
                Move { row: 0, col: 5 },
            ]
        );
    }

    #[test]
    fn game_over_blocks_moves() {
        let mut b = default_board();
        for i in 0..4usize {
            b.apply_move(Move { row: i, col: 0 }).unwrap();
            b.apply_move(Move { row: i, col: 1 }).unwrap();
        }
        b.apply_move(Move { row: 4, col: 0 }).unwrap(); // Black wins
        assert_eq!(
            b.apply_move(Move { row: 5, col: 0 }),
            Err(MoveError::GameOver)
        );
    }
}
