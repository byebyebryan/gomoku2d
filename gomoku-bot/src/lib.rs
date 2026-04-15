pub mod random;
pub mod search;

pub use random::RandomBot;
pub use search::SearchBot;

use gomoku_core::{Board, Move};

pub trait Bot {
    fn name(&self) -> &str;
    fn choose_move(&mut self, board: &Board) -> Move;
}
