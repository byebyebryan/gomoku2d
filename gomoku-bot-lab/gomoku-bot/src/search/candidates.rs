use std::sync::OnceLock;

use gomoku_core::{Board, GameResult, Move};

use super::CandidateSource;

pub(super) const STACK_SEEN_WORDS: usize = 4;
const STACK_SEEN_CELLS: usize = STACK_SEEN_WORDS * u64::BITS as usize;
const DEFAULT_BOARD_SIZE: usize = 15;

#[derive(Debug)]
pub(super) struct CandidateMaskSet {
    pub(super) size: usize,
    pub(super) words: usize,
    pub(super) masks: Vec<[u64; STACK_SEEN_WORDS]>,
}

static DEFAULT_CANDIDATE_MASKS_R1: OnceLock<CandidateMaskSet> = OnceLock::new();
static DEFAULT_CANDIDATE_MASKS_R2: OnceLock<CandidateMaskSet> = OnceLock::new();
static DEFAULT_CANDIDATE_MASKS_R3: OnceLock<CandidateMaskSet> = OnceLock::new();

pub(super) fn candidate_moves(board: &Board, radius: usize) -> Vec<Move> {
    if board.result != GameResult::Ongoing {
        return Vec::new();
    }

    let size = board.config.board_size;
    let cell_count = size * size;
    let mut moves = Vec::new();
    let has_stones = if let Some(masks) = candidate_masks(size, radius) {
        let mut seen = [0u64; STACK_SEEN_WORDS];
        let mut occupied = [0u64; STACK_SEEN_WORDS];
        let has_stones = mark_candidate_moves_from_masks(board, masks, &mut seen, &mut occupied);
        collect_marked_candidates(board, &seen, &mut moves);
        has_stones
    } else if cell_count <= STACK_SEEN_CELLS {
        let mut seen = [0u64; STACK_SEEN_WORDS];
        let has_stones = mark_candidate_moves(board, radius, &mut seen);
        collect_marked_candidates(board, &seen, &mut moves);
        has_stones
    } else {
        let mut seen = vec![0u64; cell_count.div_ceil(u64::BITS as usize)];
        let has_stones = mark_candidate_moves(board, radius, &mut seen);
        collect_marked_candidates(board, &seen, &mut moves);
        has_stones
    };

    if !has_stones {
        let center = size / 2;
        return vec![Move {
            row: center,
            col: center,
        }];
    }

    moves
}

pub(super) fn candidate_moves_from_source(
    board: &Board,
    candidate_source: CandidateSource,
) -> Vec<Move> {
    match candidate_source {
        CandidateSource::NearAll { radius } => candidate_moves(board, radius),
        CandidateSource::NearSelfOpponent {
            self_radius,
            opponent_radius,
        } => candidate_moves_from_current_and_opponent(board, self_radius, opponent_radius),
    }
}

fn candidate_moves_from_current_and_opponent(
    board: &Board,
    self_radius: usize,
    opponent_radius: usize,
) -> Vec<Move> {
    if self_radius == opponent_radius {
        return candidate_moves(board, self_radius);
    }
    if board.result != GameResult::Ongoing {
        return Vec::new();
    }

    let size = board.config.board_size;
    let cell_count = size * size;
    let mut moves = Vec::new();
    let has_stones = if cell_count <= STACK_SEEN_CELLS {
        let mut seen = [0u64; STACK_SEEN_WORDS];
        let mut occupied = [0u64; STACK_SEEN_WORDS];
        let has_stones = mark_candidate_moves_from_current_and_opponent(
            board,
            self_radius,
            opponent_radius,
            &mut seen,
            &mut occupied,
        );
        collect_marked_candidates(board, &seen, &mut moves);
        has_stones
    } else {
        let mut seen = vec![0u64; cell_count.div_ceil(u64::BITS as usize)];
        let mut occupied = vec![0u64; cell_count.div_ceil(u64::BITS as usize)];
        let has_stones = mark_candidate_moves_from_current_and_opponent(
            board,
            self_radius,
            opponent_radius,
            &mut seen,
            &mut occupied,
        );
        collect_marked_candidates(board, &seen, &mut moves);
        has_stones
    };

    if !has_stones {
        let center = size / 2;
        return vec![Move {
            row: center,
            col: center,
        }];
    }

    moves
}

fn candidate_masks(size: usize, radius: usize) -> Option<&'static CandidateMaskSet> {
    (size == DEFAULT_BOARD_SIZE && (1..=3).contains(&radius))
        .then(|| default_candidate_masks(radius))
}

pub(super) fn default_candidate_masks(radius: usize) -> &'static CandidateMaskSet {
    match radius {
        1 => DEFAULT_CANDIDATE_MASKS_R1
            .get_or_init(|| build_candidate_masks(DEFAULT_BOARD_SIZE, radius)),
        2 => DEFAULT_CANDIDATE_MASKS_R2
            .get_or_init(|| build_candidate_masks(DEFAULT_BOARD_SIZE, radius)),
        3 => DEFAULT_CANDIDATE_MASKS_R3
            .get_or_init(|| build_candidate_masks(DEFAULT_BOARD_SIZE, radius)),
        _ => panic!("default candidate masks are only available for radius 1-3"),
    }
}

fn build_candidate_masks(size: usize, radius: usize) -> CandidateMaskSet {
    let words = (size * size).div_ceil(u64::BITS as usize);
    debug_assert!(words <= STACK_SEEN_WORDS);

    let mut masks = Vec::with_capacity(size * size);
    for row in 0..size {
        for col in 0..size {
            let mut mask = [0u64; STACK_SEEN_WORDS];
            let rmin = row.saturating_sub(radius);
            let rmax = (row + radius).min(size - 1);
            let cmin = col.saturating_sub(radius);
            let cmax = (col + radius).min(size - 1);
            for r in rmin..=rmax {
                for c in cmin..=cmax {
                    mark_seen(&mut mask, r * size + c);
                }
            }
            masks.push(mask);
        }
    }

    CandidateMaskSet { size, words, masks }
}

fn mark_candidate_moves_from_masks(
    board: &Board,
    masks: &CandidateMaskSet,
    seen: &mut [u64],
    occupied: &mut [u64],
) -> bool {
    let size = board.config.board_size;
    debug_assert_eq!(size, masks.size);
    let mut has_stones = false;

    board.for_each_occupied(|row, col, _| {
        has_stones = true;
        let idx = row * size + col;
        mark_seen(occupied, idx);
        for (seen_word, mask_word) in seen.iter_mut().zip(masks.masks[idx]).take(masks.words) {
            *seen_word |= mask_word;
        }
    });

    for (seen_word, occupied_word) in seen.iter_mut().zip(occupied.iter()).take(masks.words) {
        *seen_word &= !occupied_word;
    }

    has_stones
}

fn mark_candidate_moves(board: &Board, radius: usize, seen: &mut [u64]) -> bool {
    let size = board.config.board_size;
    let mut has_stones = false;

    board.for_each_occupied(|row, col, _| {
        has_stones = true;

        let rmin = row.saturating_sub(radius);
        let rmax = (row + radius).min(size - 1);
        let cmin = col.saturating_sub(radius);
        let cmax = (col + radius).min(size - 1);
        for r in rmin..=rmax {
            for c in cmin..=cmax {
                let idx = r * size + c;
                if board.is_empty(r, c) {
                    mark_seen(seen, idx);
                }
            }
        }
    });

    has_stones
}

fn mark_candidate_moves_from_current_and_opponent(
    board: &Board,
    self_radius: usize,
    opponent_radius: usize,
    seen: &mut [u64],
    occupied: &mut [u64],
) -> bool {
    let size = board.config.board_size;
    let current = board.current_player;
    let mut has_stones = false;

    board.for_each_occupied(|row, col, color| {
        has_stones = true;
        let idx = row * size + col;
        mark_seen(occupied, idx);
        let radius = if color == current {
            self_radius
        } else {
            opponent_radius
        };

        if let Some(masks) = candidate_masks(size, radius) {
            for (seen_word, mask_word) in seen.iter_mut().zip(masks.masks[idx]).take(masks.words) {
                *seen_word |= mask_word;
            }
        } else {
            let rmin = row.saturating_sub(radius);
            let rmax = (row + radius).min(size - 1);
            let cmin = col.saturating_sub(radius);
            let cmax = (col + radius).min(size - 1);
            for r in rmin..=rmax {
                for c in cmin..=cmax {
                    mark_seen(seen, r * size + c);
                }
            }
        }
    });

    for (seen_word, occupied_word) in seen.iter_mut().zip(occupied.iter()) {
        *seen_word &= !occupied_word;
    }

    has_stones
}

fn collect_marked_candidates(board: &Board, seen: &[u64], moves: &mut Vec<Move>) {
    let size = board.config.board_size;
    let cell_count = size * size;
    moves.reserve(size * size);

    for (word_idx, &word) in seen.iter().enumerate() {
        let mut bits = word;
        while bits != 0 {
            let bit_idx = bits.trailing_zeros() as usize;
            let idx = word_idx * u64::BITS as usize + bit_idx;
            if idx >= cell_count {
                return;
            }
            moves.push(Move {
                row: idx / size,
                col: idx % size,
            });
            bits &= bits - 1;
        }
    }
}

fn mark_seen(seen: &mut [u64], idx: usize) {
    let word = idx / u64::BITS as usize;
    let bit = 1u64 << (idx % u64::BITS as usize);
    seen[word] |= bit;
}

#[cfg(test)]
pub(super) fn mask_contains(mask: [u64; STACK_SEEN_WORDS], mv: Move, size: usize) -> bool {
    let idx = mv.row * size + mv.col;
    let word = idx / u64::BITS as usize;
    let bit = 1u64 << (idx % u64::BITS as usize);
    mask[word] & bit != 0
}

#[cfg(test)]
pub(super) fn candidate_moves_reference(board: &Board, radius: usize) -> Vec<Move> {
    if board.result != GameResult::Ongoing {
        return Vec::new();
    }

    let size = board.config.board_size;
    let mut seen = vec![false; size * size];
    let mut moves = Vec::new();
    let mut has_stones = false;

    for row in 0..size {
        for col in 0..size {
            if board.is_empty(row, col) {
                continue;
            }

            has_stones = true;

            let rmin = row.saturating_sub(radius);
            let rmax = (row + radius).min(size - 1);
            let cmin = col.saturating_sub(radius);
            let cmax = (col + radius).min(size - 1);
            for r in rmin..=rmax {
                for c in cmin..=cmax {
                    let idx = r * size + c;
                    if !seen[idx] && board.is_empty(r, c) {
                        seen[idx] = true;
                        moves.push(Move { row: r, col: c });
                    }
                }
            }
        }
    }

    if !has_stones {
        let center = size / 2;
        return vec![Move {
            row: center,
            col: center,
        }];
    }

    moves
}

#[cfg(test)]
pub(super) fn candidate_moves_from_source_reference(
    board: &Board,
    self_radius: usize,
    opponent_radius: usize,
) -> Vec<Move> {
    if board.result != GameResult::Ongoing {
        return Vec::new();
    }

    let size = board.config.board_size;
    let mut seen = vec![false; size * size];
    let mut moves = Vec::new();
    let mut has_stones = false;
    let current = board.current_player;

    for row in 0..size {
        for col in 0..size {
            let Some(color) = board.cell(row, col) else {
                continue;
            };
            has_stones = true;
            let radius = if color == current {
                self_radius
            } else {
                opponent_radius
            };

            let rmin = row.saturating_sub(radius);
            let rmax = (row + radius).min(size - 1);
            let cmin = col.saturating_sub(radius);
            let cmax = (col + radius).min(size - 1);
            for r in rmin..=rmax {
                for c in cmin..=cmax {
                    let idx = r * size + c;
                    if !seen[idx] && board.is_empty(r, c) {
                        seen[idx] = true;
                        moves.push(Move { row: r, col: c });
                    }
                }
            }
        }
    }

    if !has_stones {
        let center = size / 2;
        return vec![Move {
            row: center,
            col: center,
        }];
    }

    moves
}

#[doc(hidden)]
pub fn pipeline_bench_candidate_moves(board: &Board, radius: usize) -> Vec<Move> {
    candidate_moves(board, radius)
}
