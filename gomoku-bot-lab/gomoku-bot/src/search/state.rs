use instant::Instant;
use std::collections::HashMap;

use crate::frontier::{RollingFrontierFeatures, RollingThreatFrontier};
use crate::pattern::PatternFrame;
use crate::tactical::TacticalOrderingSummary;
use gomoku_core::{renju_forbidden_metrics_snapshot, Board, Color, GameResult, Move, ZobristTable};

use super::{
    CorridorProofConfig, RenjuForbiddenMetricSource, SearchMetrics, StaticEvaluation,
    ThreatViewMode,
};

#[derive(Debug, Clone)]
pub(super) struct SearchState {
    pub(super) board: Board,
    pub(super) frontier: Option<RollingThreatFrontier>,
    pub(super) pattern_frame: Option<PatternFrame>,
    pub(super) frontier_ordering_summary_memo:
        HashMap<FrontierAnnotationMemoKey, TacticalOrderingSummary>,
    pub(super) hash: u64,
    pub(super) hash_stack: Vec<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct FrontierAnnotationMemoKey {
    pub(super) hash: u64,
    player: u8,
    row: usize,
    col: usize,
}

impl SearchState {
    #[cfg(test)]
    pub(super) fn from_board(board: Board, zobrist: &ZobristTable) -> Self {
        Self::from_board_with_frontier(board, zobrist, true)
    }

    pub(super) fn from_board_for_config(
        board: Board,
        zobrist: &ZobristTable,
        mode: ThreatViewMode,
        static_eval: StaticEvaluation,
        corridor_proof: CorridorProofConfig,
    ) -> Self {
        Self::from_board_with_frontier_features(
            board,
            zobrist,
            frontier_features_for_search(mode, corridor_proof),
            pattern_frame_for_search(mode, static_eval),
        )
    }

    #[cfg(test)]
    pub(super) fn from_board_with_frontier(
        board: Board,
        zobrist: &ZobristTable,
        enable_frontier: bool,
    ) -> Self {
        Self::from_board_with_frontier_features(
            board,
            zobrist,
            enable_frontier.then_some(RollingFrontierFeatures::Full),
            false,
        )
    }

    pub(super) fn from_board_with_frontier_features(
        board: Board,
        zobrist: &ZobristTable,
        frontier_features: Option<RollingFrontierFeatures>,
        enable_pattern_frame: bool,
    ) -> Self {
        let hash = board.hash_with(zobrist);
        let frontier = frontier_features
            .map(|features| RollingThreatFrontier::from_board_with_features(&board, features));
        let pattern_frame = enable_pattern_frame.then(|| PatternFrame::from_board(&board));
        Self {
            board,
            frontier,
            pattern_frame,
            frontier_ordering_summary_memo: HashMap::new(),
            hash,
            hash_stack: Vec::new(),
        }
    }

    pub(super) fn board(&self) -> &Board {
        &self.board
    }

    pub(super) fn threat_view(&self) -> &RollingThreatFrontier {
        self.frontier
            .as_ref()
            .expect("search state frontier requested when disabled")
    }

    pub(super) fn threat_view_mut(&mut self) -> &mut RollingThreatFrontier {
        self.frontier
            .as_mut()
            .expect("search state frontier requested when disabled")
    }

    pub(super) fn hash(&self) -> u64 {
        self.hash
    }

    pub(super) fn frontier_annotation_memo_key(
        &self,
        player: Color,
        mv: Move,
    ) -> FrontierAnnotationMemoKey {
        FrontierAnnotationMemoKey {
            hash: self.hash,
            player: player as u8,
            row: mv.row,
            col: mv.col,
        }
    }

    #[cfg(test)]
    pub(super) fn apply_trusted_legal_move(
        &mut self,
        mv: Move,
        zobrist: &ZobristTable,
    ) -> GameResult {
        self.apply_trusted_legal_move_inner(mv, zobrist, None)
    }

    pub(super) fn apply_trusted_legal_move_counted(
        &mut self,
        mv: Move,
        zobrist: &ZobristTable,
        metrics: &mut SearchMetrics,
    ) -> GameResult {
        self.apply_trusted_legal_move_inner(mv, zobrist, Some(metrics))
    }

    pub(super) fn apply_trusted_legal_move_inner(
        &mut self,
        mv: Move,
        zobrist: &ZobristTable,
        metrics: Option<&mut SearchMetrics>,
    ) -> GameResult {
        let color = self.board.current_player;
        self.hash_stack.push(self.hash);
        self.hash ^= zobrist.piece(mv.row, mv.col, color) ^ zobrist.turn;
        let board_result = self.board.apply_trusted_legal_move(mv);
        let mut metrics = metrics;
        if let Some(frontier) = &mut self.frontier {
            let start = Instant::now();
            let (frontier_result, timings) = frontier.apply_trusted_legal_move_profiled(mv);
            if let Some(metrics) = metrics.as_mut() {
                metrics.record_threat_view_frontier_rebuild(start.elapsed());
                metrics.record_threat_view_frontier_update_parts(timings);
            }
            debug_assert_eq!(
                board_result, frontier_result,
                "search state board/frontier result diverged after apply"
            );
        }
        if let Some(pattern_frame) = &mut self.pattern_frame {
            let renju_before = renju_forbidden_metrics_snapshot();
            let start = Instant::now();
            let pattern_result = pattern_frame.apply_trusted_legal_move(mv);
            if let Some(metrics) = metrics.as_mut() {
                metrics.record_pattern_frame_update(start.elapsed());
                metrics.record_renju_forbidden_source_delta(
                    RenjuForbiddenMetricSource::Pattern,
                    renju_before,
                );
            }
            debug_assert_eq!(
                board_result, pattern_result,
                "search state board/pattern-frame result diverged after apply"
            );
        }
        board_result
    }

    #[cfg(test)]
    pub(super) fn undo_move(&mut self, mv: Move) {
        self.undo_move_inner(mv, None);
    }

    pub(super) fn undo_move_counted(&mut self, mv: Move, metrics: &mut SearchMetrics) {
        self.undo_move_inner(mv, Some(metrics));
    }

    pub(super) fn undo_move_inner(&mut self, mv: Move, metrics: Option<&mut SearchMetrics>) {
        self.board.undo_move(mv);
        let mut metrics = metrics;
        if let Some(frontier) = &mut self.frontier {
            let start = Instant::now();
            let timings = frontier.undo_move_profiled(mv);
            if let Some(metrics) = metrics.as_mut() {
                metrics.record_threat_view_frontier_rebuild(start.elapsed());
                metrics.record_threat_view_frontier_update_parts(timings);
            }
        }
        if let Some(pattern_frame) = &mut self.pattern_frame {
            let renju_before = renju_forbidden_metrics_snapshot();
            let start = Instant::now();
            pattern_frame.undo_move(mv);
            if let Some(metrics) = metrics.as_mut() {
                metrics.record_pattern_frame_update(start.elapsed());
                metrics.record_renju_forbidden_source_delta(
                    RenjuForbiddenMetricSource::Pattern,
                    renju_before,
                );
            }
        }
        self.hash = self
            .hash_stack
            .pop()
            .expect("search state undo_move called without matching apply");
    }
}

fn pattern_frame_for_search(mode: ThreatViewMode, static_eval: StaticEvaluation) -> bool {
    mode.uses_frontier() && static_eval == StaticEvaluation::PatternEval
}

fn frontier_features_for_search(
    mode: ThreatViewMode,
    corridor_proof: CorridorProofConfig,
) -> Option<RollingFrontierFeatures> {
    if !mode.uses_frontier() {
        return None;
    }
    if !corridor_proof.enabled {
        Some(RollingFrontierFeatures::TacticalOnly)
    } else {
        Some(RollingFrontierFeatures::Full)
    }
}
