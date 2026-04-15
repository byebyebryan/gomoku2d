use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand::rngs::StdRng;

use gomoku_core::{Board, Move};
use crate::Bot;

pub struct RandomBot {
    rng: StdRng,
}

impl RandomBot {
    pub fn new() -> Self {
        Self { rng: StdRng::from_entropy() }
    }

    pub fn seeded(seed: u64) -> Self {
        Self { rng: StdRng::seed_from_u64(seed) }
    }
}

impl Default for RandomBot {
    fn default() -> Self {
        Self::new()
    }
}

impl Bot for RandomBot {
    fn name(&self) -> &str {
        "random"
    }

    fn choose_move(&mut self, board: &Board) -> Move {
        let moves = board.legal_moves();
        assert!(!moves.is_empty(), "RandomBot called on a board with no legal moves");
        *moves.choose(&mut self.rng).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gomoku_core::RuleConfig;

    #[test]
    fn always_returns_legal_move() {
        let board = Board::new(RuleConfig::default());
        let mut bot = RandomBot::seeded(42);
        for _ in 0..20 {
            let mv = bot.choose_move(&board);
            assert!(board.is_legal(mv));
        }
    }
}
