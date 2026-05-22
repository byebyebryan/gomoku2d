use crate::types::{AnalysisModel, AnalysisOptions};
use gomoku_core::{Board, Variant};

pub fn corridor_analysis_model(board: &Board, options: &AnalysisOptions) -> AnalysisModel {
    AnalysisModel {
        reply_policy: options.reply_policy,
        rule_set: rule_label(&board.config.variant).to_string(),
        max_depth: options.max_depth,
        max_scan_plies: options.max_scan_plies,
    }
}

pub fn rule_label(variant: &Variant) -> &'static str {
    match variant {
        Variant::Freestyle => "freestyle",
        Variant::Renju => "renju",
    }
}
