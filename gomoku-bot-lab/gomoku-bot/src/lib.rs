pub mod random;
pub mod search;

pub use random::RandomBot;
pub use search::{SearchBot, SearchBotConfig};

use gomoku_core::{Board, Move};

/// Internal trusted-bot interface.
///
/// Bots receive full `&Board` access and return a `Move` directly. This is the
/// right contract for in-process, trusted bots (search, random, eval arena).
///
/// It is intentionally not the external bot interface — remote engines, sandboxed
/// bots, or protocol adapters should use a separate adapter layer rather than
/// implementing this trait directly.
pub trait Bot: Send {
    fn name(&self) -> &str;
    fn choose_move(&mut self, board: &Board) -> Move;
    /// Optional freeform trace output emitted after the last `choose_move` call.
    /// Bots that have nothing to say return `None` (the default).
    fn trace(&self) -> Option<serde_json::Value> {
        None
    }
}

// Re-export so callers don't need to depend on serde_json directly.
pub use serde_json::Value as TraceValue;
