use gomoku_core::{Board, Color, GameResult, Move, MoveError};

use crate::tactical::{
    raw_local_threat_facts_at_existing_move, raw_local_threat_facts_for_player,
    CorridorThreatPolicy, LocalThreatFact, ThreatView,
};

#[derive(Debug, Clone)]
pub struct RebuildThreatFrontier {
    board: Board,
    black_facts: Vec<LocalThreatFact>,
    white_facts: Vec<LocalThreatFact>,
    black_move_facts: Vec<LocalThreatFact>,
    white_move_facts: Vec<LocalThreatFact>,
}

#[derive(Debug, Clone)]
pub struct RollingThreatFrontier {
    board: Board,
    view: RebuildThreatFrontier,
    undo_stack: Vec<Board>,
}

impl RollingThreatFrontier {
    pub fn from_board(board: &Board) -> Self {
        Self {
            board: board.clone(),
            view: RebuildThreatFrontier::from_board(board),
            undo_stack: Vec::new(),
        }
    }

    pub fn apply_move(&mut self, mv: Move) -> Result<GameResult, MoveError> {
        self.undo_stack.push(self.board.clone());
        match self.board.apply_move(mv) {
            Ok(result) => {
                self.rebuild_view();
                Ok(result)
            }
            Err(err) => {
                self.board = self
                    .undo_stack
                    .pop()
                    .expect("apply failure should restore previous frontier board");
                Err(err)
            }
        }
    }

    pub fn apply_trusted_legal_move(&mut self, mv: Move) -> GameResult {
        self.undo_stack.push(self.board.clone());
        let result = self.board.apply_trusted_legal_move(mv);
        self.rebuild_view();
        result
    }

    pub fn undo_move(&mut self, mv: Move) {
        debug_assert_eq!(
            self.board.history.last(),
            Some(&mv),
            "frontier undo_move called with wrong move"
        );
        self.board = self
            .undo_stack
            .pop()
            .expect("frontier undo_move called without a matching apply");
        self.rebuild_view();
    }

    fn rebuild_view(&mut self) {
        self.view = RebuildThreatFrontier::from_board(&self.board);
    }
}

impl RebuildThreatFrontier {
    pub fn from_board(board: &Board) -> Self {
        Self {
            board: board.clone(),
            black_facts: raw_local_threat_facts_for_player(board, Color::Black),
            white_facts: raw_local_threat_facts_for_player(board, Color::White),
            black_move_facts: raw_local_threat_facts_for_player_by_origin(board, Color::Black),
            white_move_facts: raw_local_threat_facts_for_player_by_origin(board, Color::White),
        }
    }

    fn facts_for(&self, player: Color) -> &[LocalThreatFact] {
        match player {
            Color::Black => &self.black_facts,
            Color::White => &self.white_facts,
        }
    }

    fn move_facts_for(&self, player: Color) -> &[LocalThreatFact] {
        match player {
            Color::Black => &self.black_move_facts,
            Color::White => &self.white_move_facts,
        }
    }
}

impl ThreatView for RebuildThreatFrontier {
    fn active_corridor_threats(&self, attacker: Color) -> Vec<LocalThreatFact> {
        let policy = CorridorThreatPolicy;
        let mut facts = self
            .facts_for(attacker)
            .iter()
            .filter(|fact| policy.is_active_threat(&self.board, attacker, fact))
            .cloned()
            .collect::<Vec<_>>();
        facts.sort_by_key(|fact| std::cmp::Reverse(policy.rank(fact.kind)));
        facts
    }

    fn has_move_local_corridor_entry(&self, attacker: Color, mv: Move) -> bool {
        if !self.board.has_color(mv.row, mv.col, attacker) {
            return false;
        }

        let policy = CorridorThreatPolicy;
        self.move_facts_for(attacker)
            .iter()
            .filter(|fact| fact.origin.mv() == mv)
            .any(|fact| policy.is_active_threat(&self.board, attacker, fact))
    }

    fn defender_reply_moves(&self, attacker: Color, actual_reply: Option<Move>) -> Vec<Move> {
        // Keep reply selection delegated for the rebuild checkpoint. The cache
        // already owns the active facts; reply-set extraction becomes the first
        // target after fixture parity proves the fact index.
        CorridorThreatPolicy.defender_reply_moves(&self.board, attacker, actual_reply)
    }

    fn attacker_move_rank(&self, attacker: Color, mv: Move) -> u8 {
        // Ranking a hypothetical move still needs a post-move view. The rolling
        // frontier should optimize this once apply/undo deltas exist.
        CorridorThreatPolicy.attacker_move_rank(&self.board, attacker, mv)
    }
}

impl ThreatView for RollingThreatFrontier {
    fn active_corridor_threats(&self, attacker: Color) -> Vec<LocalThreatFact> {
        self.view.active_corridor_threats(attacker)
    }

    fn has_move_local_corridor_entry(&self, attacker: Color, mv: Move) -> bool {
        self.view.has_move_local_corridor_entry(attacker, mv)
    }

    fn defender_reply_moves(&self, attacker: Color, actual_reply: Option<Move>) -> Vec<Move> {
        self.view.defender_reply_moves(attacker, actual_reply)
    }

    fn attacker_move_rank(&self, attacker: Color, mv: Move) -> u8 {
        self.view.attacker_move_rank(attacker, mv)
    }
}

fn raw_local_threat_facts_for_player_by_origin(
    board: &Board,
    player: Color,
) -> Vec<LocalThreatFact> {
    let mut facts = Vec::new();
    board.for_each_occupied_color(player, |row, col| {
        facts.extend(raw_local_threat_facts_at_existing_move(
            board,
            player,
            Move { row, col },
        ));
    });
    normalize_local_threat_facts_by_origin(facts)
}

fn normalize_local_threat_facts_by_origin(
    facts: Vec<LocalThreatFact>,
) -> Vec<LocalThreatFact> {
    let mut facts = facts
        .into_iter()
        .map(crate::tactical::normalize_local_threat_fact)
        .collect::<Vec<_>>();
    facts.sort_by_key(|fact| {
        (
            fact.player as u8,
            fact.origin.mv().row,
            fact.origin.mv().col,
            fact.kind as u8,
        )
    });
    facts.dedup();
    facts
}

#[cfg(test)]
mod tests {
    use super::{RebuildThreatFrontier, RollingThreatFrontier};
    use crate::tactical::{ScanThreatView, ThreatView};
    use gomoku_core::{Board, Color, Move, RuleConfig, Variant};

    fn mv(notation: &str) -> Move {
        Move::from_notation(notation).expect("test move notation should parse")
    }

    fn board_from_moves(variant: Variant, moves: &[&str]) -> Board {
        let mut board = Board::new(RuleConfig {
            variant,
            ..RuleConfig::default()
        });
        for notation in moves {
            board.apply_move(mv(notation)).unwrap();
        }
        board
    }

    fn assert_frontier_matches_scan(board: &Board, attacker: Color, probe: Move) {
        let frontier = RebuildThreatFrontier::from_board(board);
        assert_view_matches_scan(board, &frontier, attacker, probe);
    }

    fn assert_view_matches_scan(
        board: &Board,
        view: &impl ThreatView,
        attacker: Color,
        probe: Move,
    ) {
        let scan = ScanThreatView::new(board);

        assert_eq!(
            view.active_corridor_threats(attacker),
            scan.active_corridor_threats(attacker)
        );
        assert_eq!(
            view.has_move_local_corridor_entry(attacker, probe),
            scan.has_move_local_corridor_entry(attacker, probe)
        );
        assert_eq!(
            view.defender_reply_moves(attacker, None),
            scan.defender_reply_moves(attacker, None)
        );
        assert_eq!(
            view.attacker_move_rank(attacker, probe),
            scan.attacker_move_rank(attacker, probe)
        );
    }

    #[test]
    fn rebuild_frontier_matches_scan_view_for_corridor_queries() {
        let board = board_from_moves(
            Variant::Renju,
            &["H8", "A1", "I8", "A2", "J8", "A3", "C3"],
        );

        assert_frontier_matches_scan(&board, Color::Black, mv("J8"));
        assert_frontier_matches_scan(&board, Color::Black, mv("K8"));
    }

    #[test]
    fn rolling_frontier_matches_scan_after_apply_and_undo() {
        let moves = ["H8", "A1", "I8", "A2", "J8", "A3", "K8", "B1", "L8"];
        let mut board = Board::new(RuleConfig {
            variant: Variant::Renju,
            ..RuleConfig::default()
        });
        let mut frontier = RollingThreatFrontier::from_board(&board);

        for notation in moves {
            let next = mv(notation);
            board.apply_move(next).unwrap();
            frontier.apply_move(next).unwrap();
            assert_view_matches_scan(&board, &frontier, Color::Black, next);
            assert_view_matches_scan(&board, &frontier, Color::White, next);
        }

        for notation in moves.into_iter().rev() {
            let previous = mv(notation);
            board.undo_move(previous);
            frontier.undo_move(previous);
            assert_view_matches_scan(&board, &frontier, Color::Black, previous);
            assert_view_matches_scan(&board, &frontier, Color::White, previous);
        }
    }

    #[test]
    fn rolling_frontier_matches_scan_through_deterministic_legal_sequence() {
        let mut board = Board::new(RuleConfig {
            variant: Variant::Renju,
            ..RuleConfig::default()
        });
        let mut frontier = RollingThreatFrontier::from_board(&board);
        let mut played = Vec::new();

        for step in 0..28 {
            let legal = board.legal_moves();
            if legal.is_empty() {
                break;
            }
            let next = legal[(step * 37 + 11) % legal.len()];
            board.apply_move(next).unwrap();
            frontier.apply_move(next).unwrap();
            played.push(next);

            assert_view_matches_scan(&board, &frontier, Color::Black, next);
            assert_view_matches_scan(&board, &frontier, Color::White, next);

            if board.result != gomoku_core::GameResult::Ongoing {
                break;
            }
        }

        for previous in played.into_iter().rev() {
            board.undo_move(previous);
            frontier.undo_move(previous);
            assert_view_matches_scan(&board, &frontier, Color::Black, previous);
            assert_view_matches_scan(&board, &frontier, Color::White, previous);
        }
    }
}
