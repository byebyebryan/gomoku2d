use gomoku_core::{Board, Color, GameResult, Move, MoveError, Variant, DIRS};

use crate::tactical::{
    raw_local_threat_facts_at_existing_move, raw_local_threat_facts_for_player,
    CorridorThreatPolicy, LocalThreatFact, SearchThreatPolicy, TacticalMoveAnnotation, ThreatView,
};

#[derive(Debug, Clone)]
pub struct RebuildThreatFrontier {
    board: Board,
    black_facts: Vec<LocalThreatFact>,
    white_facts: Vec<LocalThreatFact>,
    black_move_facts: Vec<LocalThreatFact>,
    white_move_facts: Vec<LocalThreatFact>,
    search_annotations: Vec<TacticalMoveAnnotation>,
}

#[derive(Debug, Clone)]
pub struct RollingThreatFrontier {
    board: Board,
    black_move_facts_by_origin: Vec<Vec<LocalThreatFact>>,
    white_move_facts_by_origin: Vec<Vec<LocalThreatFact>>,
    black_search_annotations: Vec<TacticalMoveAnnotation>,
    white_search_annotations: Vec<TacticalMoveAnnotation>,
    undo_stack: Vec<FrontierDelta>,
}

#[derive(Debug, Clone)]
struct FrontierDelta {
    mv: Move,
    previous_move_facts: Vec<(Color, usize, Vec<LocalThreatFact>)>,
    previous_annotations: Vec<(Color, usize, TacticalMoveAnnotation)>,
}

impl RollingThreatFrontier {
    pub fn from_board(board: &Board) -> Self {
        let size = board.config.board_size;
        Self {
            board: board.clone(),
            black_move_facts_by_origin: move_facts_by_origin(board, Color::Black),
            white_move_facts_by_origin: move_facts_by_origin(board, Color::White),
            black_search_annotations: search_annotations_for_player(board, Color::Black),
            white_search_annotations: search_annotations_for_player(board, Color::White),
            undo_stack: Vec::with_capacity(size * size),
        }
    }

    fn search_annotations_for(&self, player: Color) -> &[TacticalMoveAnnotation] {
        match player {
            Color::Black => &self.black_search_annotations,
            Color::White => &self.white_search_annotations,
        }
    }

    fn search_annotations_for_mut(&mut self, player: Color) -> &mut [TacticalMoveAnnotation] {
        match player {
            Color::Black => &mut self.black_search_annotations,
            Color::White => &mut self.white_search_annotations,
        }
    }

    fn move_facts_for(&self, player: Color) -> &[Vec<LocalThreatFact>] {
        match player {
            Color::Black => &self.black_move_facts_by_origin,
            Color::White => &self.white_move_facts_by_origin,
        }
    }

    fn move_facts_for_mut(&mut self, player: Color) -> &mut [Vec<LocalThreatFact>] {
        match player {
            Color::Black => &mut self.black_move_facts_by_origin,
            Color::White => &mut self.white_move_facts_by_origin,
        }
    }

    fn capture_delta(&self, mv: Move) -> FrontierDelta {
        let affected_fact_cells = affected_axis_cells(&self.board, mv);
        let mut previous_move_facts = Vec::with_capacity(affected_fact_cells.len() * 2);
        let mut previous_annotations = Vec::new();

        for affected in affected_fact_cells {
            let index = cell_index(self.board.config.board_size, affected);
            for player in [Color::Black, Color::White] {
                previous_move_facts.push((
                    player,
                    index,
                    self.move_facts_for(player)[index].clone(),
                ));
            }
        }
        for player in [Color::Black, Color::White] {
            for affected in affected_annotation_cells(&self.board, player, mv) {
                let index = cell_index(self.board.config.board_size, affected);
                previous_annotations.push((
                    player,
                    index,
                    self.search_annotations_for(player)[index].clone(),
                ));
            }
        }

        FrontierDelta {
            mv,
            previous_move_facts,
            previous_annotations,
        }
    }

    fn refresh_delta_cells(&mut self, delta: &FrontierDelta) {
        let size = self.board.config.board_size;
        for &(player, index, _) in &delta.previous_move_facts {
            let mv = move_from_index(size, index);
            self.move_facts_for_mut(player)[index] =
                raw_local_threat_facts_at_existing_move(&self.board, player, mv);
        }
        for &(player, index, _) in &delta.previous_annotations {
            let mv = move_from_index(size, index);
            self.search_annotations_for_mut(player)[index] =
                search_annotation_for_player(&self.board, player, mv);
        }
    }

    fn restore_delta_cells(&mut self, delta: FrontierDelta) {
        for (player, index, facts) in delta.previous_move_facts {
            self.move_facts_for_mut(player)[index] = facts;
        }
        for (player, index, annotation) in delta.previous_annotations {
            self.search_annotations_for_mut(player)[index] = annotation;
        }
    }

    pub fn apply_move(&mut self, mv: Move) -> Result<GameResult, MoveError> {
        let delta = self.capture_delta(mv);
        match self.board.apply_move(mv) {
            Ok(result) => {
                self.refresh_delta_cells(&delta);
                self.undo_stack.push(delta);
                Ok(result)
            }
            Err(err) => Err(err),
        }
    }

    pub fn apply_trusted_legal_move(&mut self, mv: Move) -> GameResult {
        let delta = self.capture_delta(mv);
        let result = self.board.apply_trusted_legal_move(mv);
        self.refresh_delta_cells(&delta);
        self.undo_stack.push(delta);
        result
    }

    pub fn undo_move(&mut self, mv: Move) {
        debug_assert_eq!(
            self.board.history.last(),
            Some(&mv),
            "frontier undo_move called with wrong move"
        );
        let delta = self
            .undo_stack
            .pop()
            .expect("frontier undo_move called without a matching apply");
        debug_assert_eq!(
            delta.mv, mv,
            "frontier undo_move called with mismatched cache delta"
        );
        self.board.undo_move(mv);
        self.restore_delta_cells(delta);
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
            search_annotations: search_annotations_for_board(board),
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
    fn search_annotation_for_move(&self, mv: Move) -> TacticalMoveAnnotation {
        let size = self.board.config.board_size;
        if mv.row >= size || mv.col >= size {
            return SearchThreatPolicy.annotation_for_move(&self.board, mv);
        }

        self.search_annotations[mv.row * size + mv.col].clone()
    }

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
    fn search_annotation_for_move(&self, mv: Move) -> TacticalMoveAnnotation {
        let size = self.board.config.board_size;
        if mv.row >= size || mv.col >= size {
            return search_annotation_for_player(&self.board, self.board.current_player, mv);
        }

        self.search_annotations_for(self.board.current_player)[cell_index(size, mv)].clone()
    }

    fn active_corridor_threats(&self, attacker: Color) -> Vec<LocalThreatFact> {
        // The global active-threat list is canonicalized differently from the
        // per-origin cache used by `has_move_local_corridor_entry`. Keep this
        // path scan-backed until the frontier owns a canonical threat index.
        CorridorThreatPolicy.active_threats(&self.board, attacker)
    }

    fn has_move_local_corridor_entry(&self, attacker: Color, mv: Move) -> bool {
        let size = self.board.config.board_size;
        if mv.row >= size || mv.col >= size || !self.board.has_color(mv.row, mv.col, attacker) {
            return false;
        }

        let policy = CorridorThreatPolicy;
        self.move_facts_for(attacker)[cell_index(size, mv)]
            .iter()
            .any(|fact| policy.is_active_threat(&self.board, attacker, fact))
    }

    fn defender_reply_moves(&self, attacker: Color, actual_reply: Option<Move>) -> Vec<Move> {
        CorridorThreatPolicy.defender_reply_moves(&self.board, attacker, actual_reply)
    }

    fn attacker_move_rank(&self, attacker: Color, mv: Move) -> u8 {
        CorridorThreatPolicy.attacker_move_rank(&self.board, attacker, mv)
    }
}

fn search_annotations_for_board(board: &Board) -> Vec<TacticalMoveAnnotation> {
    search_annotations_for_player(board, board.current_player)
}

fn search_annotations_for_player(board: &Board, player: Color) -> Vec<TacticalMoveAnnotation> {
    let size = board.config.board_size;
    let mut annotations = Vec::with_capacity(size * size);
    for row in 0..size {
        for col in 0..size {
            annotations.push(search_annotation_for_player(
                board,
                player,
                Move { row, col },
            ));
        }
    }
    annotations
}

fn search_annotation_for_player(board: &Board, player: Color, mv: Move) -> TacticalMoveAnnotation {
    let mut board = board.clone();
    board.current_player = player;
    SearchThreatPolicy.annotation_for_move(&board, mv)
}

fn move_facts_by_origin(board: &Board, player: Color) -> Vec<Vec<LocalThreatFact>> {
    let size = board.config.board_size;
    let mut facts = vec![Vec::new(); size * size];
    board.for_each_occupied_color(player, |row, col| {
        let mv = Move { row, col };
        facts[cell_index(size, mv)] = raw_local_threat_facts_at_existing_move(board, player, mv);
    });
    facts
}

fn affected_annotation_cells(board: &Board, player: Color, mv: Move) -> Vec<Move> {
    if board.config.variant == Variant::Renju && player == Color::Black {
        // Black Renju annotations filter threat continuations through exact
        // legality and immediate-win checks after a hypothetical continuation.
        // Those checks can observe non-local board changes, so keep this side
        // globally refreshed until the continuation cache is factored out.
        all_cells(board)
    } else {
        affected_axis_cells(board, mv)
    }
}

fn affected_axis_cells(board: &Board, mv: Move) -> Vec<Move> {
    let size = board.config.board_size;
    if size == 0 || mv.row >= size || mv.col >= size {
        return Vec::new();
    }

    let mut seen = vec![false; size * size];
    let mut cells = Vec::new();

    for (dr, dc) in DIRS {
        for direction in [-1isize, 1] {
            let mut row = mv.row as isize;
            let mut col = mv.col as isize;
            loop {
                if row < 0 || col < 0 || row >= size as isize || col >= size as isize {
                    break;
                }
                let affected = Move {
                    row: row as usize,
                    col: col as usize,
                };
                let index = cell_index(size, affected);
                if !seen[index] {
                    seen[index] = true;
                    cells.push(affected);
                }
                row += dr * direction;
                col += dc * direction;
            }
        }
    }

    cells
}

fn all_cells(board: &Board) -> Vec<Move> {
    let size = board.config.board_size;
    let mut cells = Vec::with_capacity(size * size);
    for row in 0..size {
        for col in 0..size {
            cells.push(Move { row, col });
        }
    }
    cells
}

fn cell_index(size: usize, mv: Move) -> usize {
    mv.row * size + mv.col
}

fn move_from_index(size: usize, index: usize) -> Move {
    Move {
        row: index / size,
        col: index % size,
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

fn normalize_local_threat_facts_by_origin(facts: Vec<LocalThreatFact>) -> Vec<LocalThreatFact> {
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
            view.search_annotation_for_move(probe),
            scan.search_annotation_for_move(probe)
        );
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

    fn assert_all_search_annotations_match_scan(board: &Board, view: &impl ThreatView) {
        let scan = ScanThreatView::new(board);
        let size = board.config.board_size;
        for row in 0..size {
            for col in 0..size {
                let probe = Move { row, col };
                assert_eq!(
                    view.search_annotation_for_move(probe),
                    scan.search_annotation_for_move(probe),
                    "annotation mismatch at {}",
                    probe.to_notation()
                );
            }
        }
    }

    fn assert_all_local_entries_match_scan(board: &Board, view: &impl ThreatView) {
        let scan = ScanThreatView::new(board);
        board.for_each_occupied(|row, col, color| {
            let probe = Move { row, col };
            assert_eq!(
                view.has_move_local_corridor_entry(color, probe),
                scan.has_move_local_corridor_entry(color, probe),
                "local entry mismatch at {} for {:?}",
                probe.to_notation(),
                color
            );
        });
    }

    #[test]
    fn rebuild_frontier_matches_scan_view_for_corridor_queries() {
        let board = board_from_moves(Variant::Renju, &["H8", "A1", "I8", "A2", "J8", "A3", "C3"]);

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

    #[test]
    fn rolling_frontier_invalidates_full_axes_for_search_annotations() {
        let mut board = board_from_moves(
            Variant::Renju,
            &["B8", "A1", "C8", "A2", "D8", "A3", "E8", "A4"],
        );
        let mut frontier = RollingThreatFrontier::from_board(&board);
        let probe = mv("A8");

        assert_eq!(
            frontier.search_annotation_for_move(probe),
            ScanThreatView::new(&board).search_annotation_for_move(probe)
        );

        let played = mv("F8");
        board.apply_move(played).unwrap();
        frontier.apply_move(played).unwrap();

        assert_eq!(
            frontier.search_annotation_for_move(probe),
            ScanThreatView::new(&board).search_annotation_for_move(probe),
            "a same-axis move outside the old local radius must invalidate the probe"
        );
    }

    #[test]
    fn rolling_frontier_matches_all_scan_annotations_through_apply_and_undo() {
        let mut board = Board::new(RuleConfig {
            variant: Variant::Renju,
            ..RuleConfig::default()
        });
        let mut frontier = RollingThreatFrontier::from_board(&board);
        let mut played = Vec::new();

        for step in 0..36 {
            let legal = board.legal_moves();
            if legal.is_empty() {
                break;
            }
            let next = legal[(step * 31 + 17) % legal.len()];
            board.apply_move(next).unwrap();
            frontier.apply_move(next).unwrap();
            played.push(next);

            assert_all_search_annotations_match_scan(&board, &frontier);
            assert_all_local_entries_match_scan(&board, &frontier);

            if board.result != gomoku_core::GameResult::Ongoing {
                break;
            }
        }

        for previous in played.into_iter().rev() {
            board.undo_move(previous);
            frontier.undo_move(previous);
            assert_all_search_annotations_match_scan(&board, &frontier);
            assert_all_local_entries_match_scan(&board, &frontier);
        }
    }

    #[test]
    fn rolling_frontier_matches_scan_across_many_deterministic_sequences() {
        for seed in 0..24usize {
            let mut board = Board::new(RuleConfig {
                variant: Variant::Renju,
                ..RuleConfig::default()
            });
            let mut frontier = RollingThreatFrontier::from_board(&board);

            for step in 0..24 {
                let legal = board.legal_moves();
                if legal.is_empty() {
                    break;
                }
                let next = legal[(seed * 53 + step * 37 + 19) % legal.len()];
                board.apply_move(next).unwrap();
                frontier.apply_move(next).unwrap();

                assert_all_search_annotations_match_scan(&board, &frontier);
                assert_all_local_entries_match_scan(&board, &frontier);

                if board.result != gomoku_core::GameResult::Ongoing {
                    break;
                }
            }
        }
    }
}
