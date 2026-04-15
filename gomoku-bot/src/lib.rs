pub mod random;
pub mod search;

pub use random::RandomBot;
pub use search::SearchBot;

use gomoku_core::{Board, Move};

pub trait Bot {
    fn name(&self) -> &str;
    fn choose_move(&mut self, board: &Board) -> Move;
    /// Optional freeform trace output emitted after the last `choose_move` call.
    /// Bots that have nothing to say return `None` (the default).
    fn trace(&self) -> Option<serde_json::Value> { None }
}

// Re-export so callers don't need to depend on serde_json directly.
pub use serde_json::Value as TraceValue;
