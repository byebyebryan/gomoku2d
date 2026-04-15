use serde::{Deserialize, Serialize};

use crate::rules::RuleConfig;
use crate::zobrist::ZobristTable;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(usize)]
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
        let row: usize = s[1..].parse().map_err(|_| format!("invalid row in notation: '{s}'"))?;
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
}

impl std::fmt::Display for MoveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MoveError::OutOfBounds => write!(f, "move out of bounds"),
            MoveError::Occupied => write!(f, "cell already occupied"),
            MoveError::GameOver => write!(f, "game is already over"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Board {
    pub config: RuleConfig,
    cells: Vec<Vec<Cell>>,
    pub history: Vec<Move>,
    pub current_player: Color,
    pub result: GameResult,
}

impl Board {
    pub fn new(config: RuleConfig) -> Self {
        let size = config.board_size;
        Self {
            cells: vec![vec![None; size]; size],
            history: Vec::new(),
            current_player: Color::Black,
            result: GameResult::Ongoing,
            config,
        }
    }

    pub fn cell(&self, row: usize, col: usize) -> Cell {
        self.cells[row][col]
    }

    pub fn is_legal(&self, mv: Move) -> bool {
        if self.result != GameResult::Ongoing {
            return false;
        }
        let size = self.config.board_size;
        mv.row < size && mv.col < size && self.cells[mv.row][mv.col].is_none()
    }

    pub fn legal_moves(&self) -> Vec<Move> {
        if self.result != GameResult::Ongoing {
            return vec![];
        }
        let size = self.config.board_size;
        let mut moves = Vec::with_capacity(size * size);
        for row in 0..size {
            for col in 0..size {
                if self.cells[row][col].is_none() {
                    moves.push(Move { row, col });
                }
            }
        }
        moves
    }

    pub fn apply_move(&mut self, mv: Move) -> Result<GameResult, MoveError> {
        if self.result != GameResult::Ongoing {
            return Err(MoveError::GameOver);
        }
        let size = self.config.board_size;
        if mv.row >= size || mv.col >= size {
            return Err(MoveError::OutOfBounds);
        }
        if self.cells[mv.row][mv.col].is_some() {
            return Err(MoveError::Occupied);
        }

        let color = self.current_player;
        self.cells[mv.row][mv.col] = Some(color);
        self.history.push(mv);

        if self.check_win(mv, color) {
            self.result = GameResult::Winner(color);
        } else if self.history.len() == size * size {
            self.result = GameResult::Draw;
        }

        self.current_player = color.opponent();
        Ok(self.result.clone())
    }

    fn check_win(&self, mv: Move, color: Color) -> bool {
        let win_len = self.config.win_length as isize;

        for (dr, dc) in DIRS {
            let count = 1
                + self.count_direction(mv.row as isize, mv.col as isize, dr, dc, color)
                + self.count_direction(mv.row as isize, mv.col as isize, -dr, -dc, color);
            if count >= win_len {
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
            if self.cells[r as usize][c as usize] == Some(color) {
                count += 1;
                r += dr;
                c += dc;
            } else {
                break;
            }
        }
        count
    }

    /// Serialize board state to a compact string.
    /// Format: "<size>/<win_len>/<turn>/<cells...>"
    /// cells: '.' = empty, 'B' = black, 'W' = white
    pub fn to_fen(&self) -> String {
        let turn = match self.current_player {
            Color::Black => 'B',
            Color::White => 'W',
        };
        let cells: String = self
            .cells
            .iter()
            .flatten()
            .map(|c| c.map_or('.', Color::to_char))
            .collect();
        format!("{}/{}/{}/{}", self.config.board_size, self.config.win_length, turn, cells)
    }

    /// Undo the last move. Only valid if `mv` was the last move applied.
    /// Intended for use by search algorithms.
    pub fn undo_move(&mut self, mv: Move) {
        debug_assert_eq!(self.history.last(), Some(&mv), "undo_move called with wrong move");
        self.cells[mv.row][mv.col] = None;
        self.history.pop();
        self.current_player = self.current_player.opponent();
        self.result = GameResult::Ongoing;
    }

    /// Zobrist hash of the current position. Stable across crates — uses the
    /// same seed as `ZobristTable::new(board_size)` in `gomoku-core`.
    pub fn hash(&self) -> u64 {
        let zt = ZobristTable::new(self.config.board_size);
        let mut h = 0u64;
        for row in 0..self.config.board_size {
            for col in 0..self.config.board_size {
                if let Some(color) = self.cells[row][col] {
                    h ^= zt.piece(row, col, color);
                }
            }
        }
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

        let config = RuleConfig { board_size, win_length };
        let mut board = Board::new(config);
        board.current_player = turn;

        for (i, ch) in cell_str.chars().enumerate() {
            let row = i / board_size;
            let col = i % board_size;
            board.cells[row][col] = match ch {
                '.' => None,
                'B' => Some(Color::Black),
                'W' => Some(Color::White),
                _ => return Err(format!("invalid cell char '{ch}'")),
            };
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
        assert_eq!(b.apply_move(Move { row: 0, col: 0 }), Err(MoveError::Occupied));
    }

    #[test]
    fn out_of_bounds_rejected() {
        let mut b = default_board();
        assert_eq!(b.apply_move(Move { row: 15, col: 0 }), Err(MoveError::OutOfBounds));
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
            b.apply_move(Move { row: i, col: (i + 5).min(14) }).unwrap();
        }
        let result = b.apply_move(Move { row: 4, col: 4 }).unwrap();
        assert_eq!(result, GameResult::Winner(Color::Black));
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
    fn game_over_blocks_moves() {
        let mut b = default_board();
        for i in 0..4usize {
            b.apply_move(Move { row: i, col: 0 }).unwrap();
            b.apply_move(Move { row: i, col: 1 }).unwrap();
        }
        b.apply_move(Move { row: 4, col: 0 }).unwrap(); // Black wins
        assert_eq!(b.apply_move(Move { row: 5, col: 0 }), Err(MoveError::GameOver));
    }
}
