use crate::board::Color;

/// Fixed seed for the Zobrist table.
/// Stored in every replay so hashes are verifiable without the codebase.
pub const ZOBRIST_SEED: u64 = 0xdeadbeef_cafebabe;

/// Name of the PRNG algorithm used to generate the table.
/// Stored alongside the seed in replays.
pub const ZOBRIST_ALGORITHM: &str = "xorshift64";

/// Deterministic Zobrist hash table for a given board size.
///
/// Fully determined by `ZOBRIST_SEED` + `size`. To verify a hash stored in a
/// replay: reconstruct the table with `ZobristTable::new(board_size)` using
/// the seed and algorithm recorded in the replay's `hash_algo` field, then
/// re-hash the position by stepping through the move list.
pub struct ZobristTable {
    table: Vec<[u64; 2]>, // flat [row * size + col][color_index]
    pub turn: u64,
    size: usize,
}

impl ZobristTable {
    pub fn new(size: usize) -> Self {
        let mut rng = ZOBRIST_SEED;
        let mut next = || -> u64 {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            rng
        };
        let table = (0..size * size).map(|_| [next(), next()]).collect();
        Self { table, turn: next(), size }
    }

    #[inline(always)]
    pub fn piece(&self, row: usize, col: usize, color: Color) -> u64 {
        self.table[row * self.size + col][color as usize]
    }
}
