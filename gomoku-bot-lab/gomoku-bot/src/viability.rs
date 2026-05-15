use gomoku_core::{Board, Color, Move, DIRS};

pub const ALL_DIRECTIONS_DEAD: u8 = 0;
pub const ALL_DIRECTIONS_MASK: u8 = (1u8 << DIRS.len()) - 1;

pub const fn direction_bit(direction_index: usize) -> u8 {
    1 << direction_index
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CellViability {
    pub black_direction_mask: u8,
    pub white_direction_mask: u8,
}

impl CellViability {
    pub const fn mask_for(self, player: Color) -> u8 {
        match player {
            Color::Black => self.black_direction_mask,
            Color::White => self.white_direction_mask,
        }
    }

    pub const fn has_any_for(self, player: Color) -> bool {
        self.mask_for(player) != ALL_DIRECTIONS_DEAD
    }

    pub const fn is_dead_for(self, player: Color) -> bool {
        self.mask_for(player) == ALL_DIRECTIONS_DEAD
    }

    pub const fn is_null(self) -> bool {
        self.black_direction_mask == ALL_DIRECTIONS_DEAD
            && self.white_direction_mask == ALL_DIRECTIONS_DEAD
    }
}

pub fn scan_cell_null(board: &Board, mv: Move) -> bool {
    scan_cell_viability(board, mv).is_null()
}

pub fn scan_cell_viability(board: &Board, mv: Move) -> CellViability {
    if !is_empty_board_cell(board, mv) {
        return CellViability {
            black_direction_mask: ALL_DIRECTIONS_MASK,
            white_direction_mask: ALL_DIRECTIONS_MASK,
        };
    }

    CellViability {
        black_direction_mask: scan_viable_direction_mask(board, Color::Black, mv),
        white_direction_mask: scan_viable_direction_mask(board, Color::White, mv),
    }
}

pub fn scan_viable_direction_mask(board: &Board, player: Color, mv: Move) -> u8 {
    if !is_empty_board_cell(board, mv) {
        return 0;
    }

    DIRS.iter()
        .enumerate()
        .fold(0u8, |mask, (direction_index, &direction)| {
            if scan_direction_viable(board, player, mv, direction) {
                mask | direction_bit(direction_index)
            } else {
                mask
            }
        })
}

pub fn scan_direction_viable(
    board: &Board,
    player: Color,
    mv: Move,
    direction: (isize, isize),
) -> bool {
    if !is_empty_board_cell(board, mv) {
        return false;
    }

    let size = board.config.board_size as isize;
    let win_length = board.config.win_length as isize;
    if win_length <= 0 {
        return false;
    }
    let row = mv.row as isize;
    let col = mv.col as isize;
    let opponent = player.opponent();
    let (dr, dc) = direction;

    (-(win_length - 1)..=0).any(|start| {
        (start..start + win_length).all(|offset| {
            let r = row + dr * offset;
            let c = col + dc * offset;
            if r < 0 || r >= size || c < 0 || c >= size {
                return false;
            }

            let r = r as usize;
            let c = c as usize;
            !board.has_color(r, c, opponent)
        })
    })
}

fn is_empty_board_cell(board: &Board, mv: Move) -> bool {
    mv.row < board.config.board_size
        && mv.col < board.config.board_size
        && board.is_empty(mv.row, mv.col)
}

#[cfg(test)]
mod tests {
    use gomoku_core::{Board, Color, Move, RuleConfig, Variant};

    fn mv(notation: &str) -> Move {
        Move::from_notation(notation).expect("test move notation should parse")
    }

    fn board_from_color_moves(black: &[&str], white: &[&str]) -> Board {
        assert_eq!(black.len(), white.len());
        let mut board = Board::new(RuleConfig {
            variant: Variant::Freestyle,
            ..RuleConfig::default()
        });
        for (black_move, white_move) in black.iter().zip(white.iter()) {
            board.apply_move(mv(black_move)).unwrap();
            board.apply_move(mv(white_move)).unwrap();
        }
        board
    }

    #[test]
    fn scan_direction_viability_is_color_specific() {
        let board = board_from_color_moves(&["H8", "M8"], &["A1", "A2"]);
        let probe = mv("J8");

        assert!(super::scan_direction_viable(
            &board,
            Color::Black,
            probe,
            (0, 1)
        ));
        assert!(!super::scan_direction_viable(
            &board,
            Color::White,
            probe,
            (0, 1)
        ));
        assert!(!super::scan_cell_null(&board, probe));
    }

    #[test]
    fn scan_cell_null_requires_both_colors_dead_in_every_direction() {
        let board = board_from_color_moves(
            &["G8", "L8", "H7", "H12", "G7", "L12", "G9", "L4"],
            &["D8", "I8", "H4", "H9", "D4", "I9", "D12", "I7"],
        );
        let probe = mv("H8");

        assert_eq!(
            super::scan_viable_direction_mask(&board, Color::Black, probe),
            0
        );
        assert_eq!(
            super::scan_viable_direction_mask(&board, Color::White, probe),
            0
        );
        assert!(super::scan_cell_null(&board, probe));
    }

    #[test]
    fn scan_cell_null_does_not_mark_occupied_cells_as_candidates() {
        let board = board_from_color_moves(&["H8"], &["A1"]);

        assert!(!super::scan_cell_null(&board, mv("H8")));
    }

    #[test]
    fn scan_cell_viability_exposes_per_side_masks() {
        let board = board_from_color_moves(&["H8", "M8"], &["A1", "A2"]);
        let probe = mv("J8");

        let viability = super::scan_cell_viability(&board, probe);

        assert_eq!(
            viability.mask_for(Color::Black),
            super::scan_viable_direction_mask(&board, Color::Black, probe)
        );
        assert_eq!(
            viability.mask_for(Color::White),
            super::scan_viable_direction_mask(&board, Color::White, probe)
        );
        assert!(viability.has_any_for(Color::Black));
        assert!(viability.has_any_for(Color::White));
        assert_ne!(
            viability.mask_for(Color::Black),
            viability.mask_for(Color::White)
        );
        assert!(!viability.is_null());
    }

    #[test]
    fn scan_cell_viability_derives_null_from_both_sides() {
        let board = board_from_color_moves(
            &["G8", "L8", "H7", "H12", "G7", "L12", "G9", "L4"],
            &["D8", "I8", "H4", "H9", "D4", "I9", "D12", "I7"],
        );

        let viability = super::scan_cell_viability(&board, mv("H8"));

        assert_eq!(viability.black_direction_mask, 0);
        assert_eq!(viability.white_direction_mask, 0);
        assert!(viability.is_dead_for(Color::Black));
        assert!(viability.is_dead_for(Color::White));
        assert!(viability.is_null());
    }
}
