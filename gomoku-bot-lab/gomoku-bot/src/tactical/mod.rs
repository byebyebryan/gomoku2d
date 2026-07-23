use gomoku_core::{Board, Color, GameResult, Move, Variant, DIRS};
use serde::Serialize;

mod metrics;

use metrics::{
    record_compound_imminent_query, record_renju_effective_filter,
    record_renju_effective_filter_continuation,
};
pub use metrics::{tactical_metrics_snapshot, TacticalMetrics};

mod policies;
mod scan;
mod shapes;
mod threats;

pub use policies::*;
pub use scan::*;
use shapes::*;
pub use threats::*;

#[cfg(test)]
mod tests;
