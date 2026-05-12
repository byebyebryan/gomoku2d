use gomoku_core::{Board, Color, Move};

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
    use super::RebuildThreatFrontier;
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
        let scan = ScanThreatView::new(board);
        let frontier = RebuildThreatFrontier::from_board(board);

        assert_eq!(
            frontier.active_corridor_threats(attacker),
            scan.active_corridor_threats(attacker)
        );
        assert_eq!(
            frontier.has_move_local_corridor_entry(attacker, probe),
            scan.has_move_local_corridor_entry(attacker, probe)
        );
        assert_eq!(
            frontier.defender_reply_moves(attacker, None),
            scan.defender_reply_moves(attacker, None)
        );
        assert_eq!(
            frontier.attacker_move_rank(attacker, probe),
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
}
