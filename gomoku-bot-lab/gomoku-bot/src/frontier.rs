use std::time::{Duration, Instant};

use gomoku_core::{Board, Color, GameResult, Move, MoveError, DIRS};

use crate::tactical::{
    raw_local_threat_facts_at_existing_move, raw_local_threat_facts_for_player,
    CorridorThreatPolicy, LocalThreatFact, ScanThreatView, SearchThreatPolicy,
    TacticalMoveAnnotation, TacticalOrderingSummary, ThreatView,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RollingFrontierFeatures {
    Full,
    TacticalOnly,
}

impl RollingFrontierFeatures {
    const fn maintains_move_facts(self) -> bool {
        matches!(self, Self::Full)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct FrontierUpdateTimings {
    pub delta_capture: Option<Duration>,
    pub move_fact_update: Option<Duration>,
    pub annotation_dirty_mark: Option<Duration>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrontierAnnotationSource {
    CleanCache,
    DirtyRecompute,
    Fallback,
}

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
    features: RollingFrontierFeatures,
    black_move_facts_by_origin: Option<Vec<Vec<LocalThreatFact>>>,
    white_move_facts_by_origin: Option<Vec<Vec<LocalThreatFact>>>,
    black_raw_search_annotations: Vec<TacticalMoveAnnotation>,
    white_raw_search_annotations: Vec<TacticalMoveAnnotation>,
    black_raw_search_annotation_dirty: Vec<bool>,
    white_raw_search_annotation_dirty: Vec<bool>,
    black_immediate_wins: Vec<bool>,
    white_immediate_wins: Vec<bool>,
    black_immediate_win_dirty: Vec<bool>,
    white_immediate_win_dirty: Vec<bool>,
    undo_stack: Vec<FrontierDelta>,
}

#[derive(Debug, Clone)]
struct FrontierDelta {
    mv: Move,
    previous_move_facts: Vec<(Color, usize, Vec<LocalThreatFact>)>,
    previous_annotation_dirty: Vec<(Color, usize, bool)>,
    previous_immediate_wins: Vec<(Color, usize, bool, bool)>,
}

impl RollingThreatFrontier {
    pub fn from_board(board: &Board) -> Self {
        Self::from_board_with_features(board, RollingFrontierFeatures::Full)
    }

    pub fn from_board_with_features(board: &Board, features: RollingFrontierFeatures) -> Self {
        let size = board.config.board_size;
        Self {
            board: board.clone(),
            features,
            black_move_facts_by_origin: features
                .maintains_move_facts()
                .then(|| move_facts_by_origin(board, Color::Black)),
            white_move_facts_by_origin: features
                .maintains_move_facts()
                .then(|| move_facts_by_origin(board, Color::White)),
            black_raw_search_annotations: raw_search_annotations_for_player(board, Color::Black),
            white_raw_search_annotations: raw_search_annotations_for_player(board, Color::White),
            black_raw_search_annotation_dirty: vec![false; size * size],
            white_raw_search_annotation_dirty: vec![false; size * size],
            black_immediate_wins: immediate_wins_for_player(board, Color::Black),
            white_immediate_wins: immediate_wins_for_player(board, Color::White),
            black_immediate_win_dirty: vec![false; size * size],
            white_immediate_win_dirty: vec![false; size * size],
            undo_stack: Vec::with_capacity(size * size),
        }
    }

    pub fn maintains_move_facts(&self) -> bool {
        self.features.maintains_move_facts()
    }

    fn raw_search_annotations_for(&self, player: Color) -> &[TacticalMoveAnnotation] {
        match player {
            Color::Black => &self.black_raw_search_annotations,
            Color::White => &self.white_raw_search_annotations,
        }
    }

    fn raw_search_annotation_dirty_for(&self, player: Color) -> &[bool] {
        match player {
            Color::Black => &self.black_raw_search_annotation_dirty,
            Color::White => &self.white_raw_search_annotation_dirty,
        }
    }

    fn raw_search_annotation_dirty_for_mut(&mut self, player: Color) -> &mut [bool] {
        match player {
            Color::Black => &mut self.black_raw_search_annotation_dirty,
            Color::White => &mut self.white_raw_search_annotation_dirty,
        }
    }

    fn move_facts_for(&self, player: Color) -> Option<&[Vec<LocalThreatFact>]> {
        match player {
            Color::Black => self.black_move_facts_by_origin.as_deref(),
            Color::White => self.white_move_facts_by_origin.as_deref(),
        }
    }

    fn move_facts_for_mut(&mut self, player: Color) -> Option<&mut [Vec<LocalThreatFact>]> {
        match player {
            Color::Black => self.black_move_facts_by_origin.as_deref_mut(),
            Color::White => self.white_move_facts_by_origin.as_deref_mut(),
        }
    }

    fn immediate_wins_for(&self, player: Color) -> &[bool] {
        match player {
            Color::Black => &self.black_immediate_wins,
            Color::White => &self.white_immediate_wins,
        }
    }

    fn immediate_wins_for_mut(&mut self, player: Color) -> &mut [bool] {
        match player {
            Color::Black => &mut self.black_immediate_wins,
            Color::White => &mut self.white_immediate_wins,
        }
    }

    fn immediate_win_dirty_for(&self, player: Color) -> &[bool] {
        match player {
            Color::Black => &self.black_immediate_win_dirty,
            Color::White => &self.white_immediate_win_dirty,
        }
    }

    fn immediate_win_dirty_for_mut(&mut self, player: Color) -> &mut [bool] {
        match player {
            Color::Black => &mut self.black_immediate_win_dirty,
            Color::White => &mut self.white_immediate_win_dirty,
        }
    }

    fn capture_delta(&self, mv: Move) -> (FrontierDelta, Duration) {
        let start = Instant::now();
        let affected_cells = affected_local_axis_cells(&self.board, mv);
        let mut previous_move_facts = Vec::with_capacity(affected_cells.len() * 2);
        let mut previous_annotation_dirty = Vec::with_capacity(affected_cells.len() * 2);
        let mut previous_immediate_wins = Vec::with_capacity(affected_cells.len() * 2);

        for affected in affected_cells {
            let index = cell_index(self.board.config.board_size, affected);
            for player in [Color::Black, Color::White] {
                if let Some(facts) = self.move_facts_for(player) {
                    previous_move_facts.push((player, index, facts[index].clone()));
                }
                previous_annotation_dirty.push((
                    player,
                    index,
                    self.raw_search_annotation_dirty_for(player)[index],
                ));
            }
        }

        for affected in affected_immediate_win_cells(&self.board, mv) {
            let index = cell_index(self.board.config.board_size, affected);
            for player in [Color::Black, Color::White] {
                let wins = self.immediate_wins_for(player);
                let dirty = self.immediate_win_dirty_for(player);
                previous_immediate_wins.push((player, index, wins[index], dirty[index]));
            }
        }

        (
            FrontierDelta {
                mv,
                previous_move_facts,
                previous_annotation_dirty,
                previous_immediate_wins,
            },
            start.elapsed(),
        )
    }

    fn refresh_delta_cells(&mut self, delta: &FrontierDelta) -> FrontierUpdateTimings {
        let size = self.board.config.board_size;
        let move_fact_update = if delta.previous_move_facts.is_empty() {
            None
        } else {
            let start = Instant::now();
            for &(player, index, _) in &delta.previous_move_facts {
                let mv = move_from_index(size, index);
                let facts = raw_local_threat_facts_at_existing_move(&self.board, player, mv);
                if let Some(facts_by_origin) = self.move_facts_for_mut(player) {
                    facts_by_origin[index] = facts;
                }
            }
            Some(start.elapsed())
        };

        let annotation_dirty_mark = if delta.previous_annotation_dirty.is_empty() {
            None
        } else {
            let start = Instant::now();
            for &(player, index, _) in &delta.previous_annotation_dirty {
                self.raw_search_annotation_dirty_for_mut(player)[index] = true;
            }
            Some(start.elapsed())
        };

        for &(player, index, _, _) in &delta.previous_immediate_wins {
            self.immediate_win_dirty_for_mut(player)[index] = true;
        }

        FrontierUpdateTimings {
            delta_capture: None,
            move_fact_update,
            annotation_dirty_mark,
        }
    }

    fn restore_delta_cells(&mut self, delta: FrontierDelta) -> FrontierUpdateTimings {
        let move_fact_update = if delta.previous_move_facts.is_empty() {
            None
        } else {
            let start = Instant::now();
            for (player, index, facts) in delta.previous_move_facts {
                if let Some(facts_by_origin) = self.move_facts_for_mut(player) {
                    facts_by_origin[index] = facts;
                }
            }
            Some(start.elapsed())
        };

        let annotation_dirty_mark = if delta.previous_annotation_dirty.is_empty() {
            None
        } else {
            let start = Instant::now();
            for (player, index, dirty) in delta.previous_annotation_dirty {
                self.raw_search_annotation_dirty_for_mut(player)[index] = dirty;
            }
            Some(start.elapsed())
        };

        for (player, index, is_win, was_dirty) in delta.previous_immediate_wins.into_iter().rev() {
            self.immediate_wins_for_mut(player)[index] = is_win;
            self.immediate_win_dirty_for_mut(player)[index] = was_dirty;
        }

        FrontierUpdateTimings {
            delta_capture: None,
            move_fact_update,
            annotation_dirty_mark,
        }
    }

    pub fn apply_move_profiled(
        &mut self,
        mv: Move,
    ) -> (Result<GameResult, MoveError>, FrontierUpdateTimings) {
        let (delta, delta_capture) = self.capture_delta(mv);
        match self.board.apply_move(mv) {
            Ok(result) => {
                let mut timings = self.refresh_delta_cells(&delta);
                timings.delta_capture = Some(delta_capture);
                self.undo_stack.push(delta);
                (Ok(result), timings)
            }
            Err(err) => (
                Err(err),
                FrontierUpdateTimings {
                    delta_capture: Some(delta_capture),
                    ..FrontierUpdateTimings::default()
                },
            ),
        }
    }

    pub fn apply_move(&mut self, mv: Move) -> Result<GameResult, MoveError> {
        self.apply_move_profiled(mv).0
    }

    pub fn apply_trusted_legal_move_profiled(
        &mut self,
        mv: Move,
    ) -> (GameResult, FrontierUpdateTimings) {
        let (delta, delta_capture) = self.capture_delta(mv);
        let result = self.board.apply_trusted_legal_move(mv);
        let mut timings = self.refresh_delta_cells(&delta);
        timings.delta_capture = Some(delta_capture);
        self.undo_stack.push(delta);
        (result, timings)
    }

    pub fn apply_trusted_legal_move(&mut self, mv: Move) -> GameResult {
        self.apply_trusted_legal_move_profiled(mv).0
    }

    pub fn undo_move_profiled(&mut self, mv: Move) -> FrontierUpdateTimings {
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
        self.restore_delta_cells(delta)
    }

    pub fn undo_move(&mut self, mv: Move) {
        let _ = self.undo_move_profiled(mv);
    }

    pub(crate) fn immediate_winning_moves_for_cached(&mut self, player: Color) -> Vec<Move> {
        if self.board.result != GameResult::Ongoing {
            return Vec::new();
        }
        let size = self.board.config.board_size;
        let mut moves = Vec::new();
        for index in 0..size * size {
            let is_win = if self.immediate_win_dirty_for(player)[index] {
                let mv = move_from_index(size, index);
                let old_win = self.immediate_wins_for(player)[index];
                let old_dirty = self.immediate_win_dirty_for(player)[index];
                let is_win = is_immediate_win_for_player(&self.board, player, mv);
                if let Some(delta) = self.undo_stack.last_mut() {
                    delta
                        .previous_immediate_wins
                        .push((player, index, old_win, old_dirty));
                }
                self.immediate_wins_for_mut(player)[index] = is_win;
                self.immediate_win_dirty_for_mut(player)[index] = false;
                is_win
            } else {
                self.immediate_wins_for(player)[index]
            };
            if is_win {
                moves.push(move_from_index(size, index));
            }
        }
        moves
    }

    pub fn search_annotation_for_move_with_source(
        &self,
        mv: Move,
    ) -> (TacticalMoveAnnotation, FrontierAnnotationSource) {
        self.search_annotation_for_player_with_source(self.board.current_player, mv)
    }

    pub fn search_annotation_for_player_with_source(
        &self,
        player: Color,
        mv: Move,
    ) -> (TacticalMoveAnnotation, FrontierAnnotationSource) {
        let size = self.board.config.board_size;
        if mv.row >= size || mv.col >= size || self.board.result != GameResult::Ongoing {
            return (
                search_annotation_for_player(&self.board, player, mv),
                FrontierAnnotationSource::Fallback,
            );
        }

        let index = cell_index(size, mv);
        let (annotation, source) = if self.raw_search_annotation_dirty_for(player)[index] {
            (
                raw_search_annotation_for_player(&self.board, player, mv),
                FrontierAnnotationSource::DirtyRecompute,
            )
        } else {
            (
                self.raw_search_annotations_for(player)[index].clone(),
                FrontierAnnotationSource::CleanCache,
            )
        };
        (
            SearchThreatPolicy.effective_annotation_from_raw(&self.board, annotation),
            source,
        )
    }

    pub fn search_ordering_summary_for_player_with_source(
        &self,
        player: Color,
        mv: Move,
    ) -> (TacticalOrderingSummary, FrontierAnnotationSource) {
        let size = self.board.config.board_size;
        let policy = SearchThreatPolicy;
        if mv.row >= size || mv.col >= size || self.board.result != GameResult::Ongoing {
            let annotation = search_annotation_for_player(&self.board, player, mv);
            return (
                policy.ordering_summary(&annotation),
                FrontierAnnotationSource::Fallback,
            );
        }

        let index = cell_index(size, mv);
        if self.raw_search_annotation_dirty_for(player)[index] {
            let annotation = raw_search_annotation_for_player(&self.board, player, mv);
            (
                policy.effective_ordering_summary_from_raw(&self.board, &annotation),
                FrontierAnnotationSource::DirtyRecompute,
            )
        } else {
            (
                policy.effective_ordering_summary_from_raw(
                    &self.board,
                    &self.raw_search_annotations_for(player)[index],
                ),
                FrontierAnnotationSource::CleanCache,
            )
        }
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
    fn immediate_winning_moves_for(&self, player: Color) -> Vec<Move> {
        self.board.immediate_winning_moves_for(player)
    }

    fn search_annotation_for_move(&self, mv: Move) -> TacticalMoveAnnotation {
        self.search_annotation_for_player(self.board.current_player, mv)
    }

    fn search_annotation_for_player(&self, player: Color, mv: Move) -> TacticalMoveAnnotation {
        let size = self.board.config.board_size;
        if mv.row >= size || mv.col >= size {
            return SearchThreatPolicy.annotation_for_player(&self.board, player, mv);
        }

        if player == self.board.current_player {
            self.search_annotations[mv.row * size + mv.col].clone()
        } else {
            SearchThreatPolicy.annotation_for_player(&self.board, player, mv)
        }
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
        self.local_corridor_entry_rank(attacker, mv) > 0
    }

    fn local_corridor_entry_rank(&self, attacker: Color, mv: Move) -> u8 {
        local_corridor_entry_rank_from_facts(
            &self.board,
            attacker,
            mv,
            self.move_facts_for(attacker).iter(),
        )
    }

    fn defender_reply_moves(&self, attacker: Color, actual_reply: Option<Move>) -> Vec<Move> {
        // Keep reply selection delegated for the rebuild checkpoint. The cache
        // already owns the active facts; reply-set extraction becomes the first
        // target after fixture parity proves the fact index.
        CorridorThreatPolicy.defender_reply_moves(&self.board, attacker, actual_reply)
    }
}

impl ThreatView for RollingThreatFrontier {
    fn immediate_winning_moves_for(&self, player: Color) -> Vec<Move> {
        if self.board.result != GameResult::Ongoing {
            return Vec::new();
        }
        let size = self.board.config.board_size;
        let mut moves = Vec::new();
        for index in 0..size * size {
            let is_win = if self.immediate_win_dirty_for(player)[index] {
                let mv = move_from_index(size, index);
                is_immediate_win_for_player(&self.board, player, mv)
            } else {
                self.immediate_wins_for(player)[index]
            };
            if is_win {
                moves.push(move_from_index(size, index));
            }
        }
        moves
    }

    fn search_annotation_for_move(&self, mv: Move) -> TacticalMoveAnnotation {
        self.search_annotation_for_move_with_source(mv).0
    }

    fn search_annotation_for_player(&self, player: Color, mv: Move) -> TacticalMoveAnnotation {
        self.search_annotation_for_player_with_source(player, mv).0
    }

    fn active_corridor_threats(&self, attacker: Color) -> Vec<LocalThreatFact> {
        let Some(facts_by_origin) = self.move_facts_for(attacker) else {
            return ScanThreatView::new(&self.board).active_corridor_threats(attacker);
        };
        CorridorThreatPolicy.active_threats_from_facts(
            &self.board,
            attacker,
            facts_by_origin
                .iter()
                .flat_map(|facts| facts.iter().cloned()),
        )
    }

    fn has_move_local_corridor_entry(&self, attacker: Color, mv: Move) -> bool {
        self.local_corridor_entry_rank(attacker, mv) > 0
    }

    fn local_corridor_entry_rank(&self, attacker: Color, mv: Move) -> u8 {
        let size = self.board.config.board_size;
        if mv.row >= size || mv.col >= size || !self.board.has_color(mv.row, mv.col, attacker) {
            return 0;
        }

        let Some(facts_by_origin) = self.move_facts_for(attacker) else {
            return ScanThreatView::new(&self.board).local_corridor_entry_rank(attacker, mv);
        };

        local_corridor_entry_rank_from_facts(
            &self.board,
            attacker,
            mv,
            facts_by_origin[cell_index(size, mv)].iter(),
        )
    }

    fn defender_reply_moves(&self, attacker: Color, actual_reply: Option<Move>) -> Vec<Move> {
        let Some(facts_by_origin) = self.move_facts_for(attacker) else {
            return ScanThreatView::new(&self.board).defender_reply_moves(attacker, actual_reply);
        };
        let active = CorridorThreatPolicy.active_threats_from_facts(
            &self.board,
            attacker,
            facts_by_origin
                .iter()
                .flat_map(|facts| facts.iter().cloned()),
        );
        CorridorThreatPolicy.defender_reply_moves_for_active_threats(
            &self.board,
            attacker,
            active,
            actual_reply,
        )
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
    SearchThreatPolicy.annotation_for_player(board, player, mv)
}

fn immediate_wins_for_player(board: &Board, player: Color) -> Vec<bool> {
    let size = board.config.board_size;
    let mut wins = vec![false; size * size];
    if board.result != GameResult::Ongoing {
        return wins;
    }
    for mv in board.immediate_winning_moves_for(player) {
        wins[cell_index(size, mv)] = true;
    }
    wins
}

fn is_immediate_win_for_player(board: &Board, player: Color, mv: Move) -> bool {
    board.is_immediate_winning_move_for(mv, player)
}

fn local_corridor_entry_rank_from_facts<'a>(
    board: &Board,
    attacker: Color,
    mv: Move,
    facts: impl IntoIterator<Item = &'a LocalThreatFact>,
) -> u8 {
    if !board.has_color(mv.row, mv.col, attacker) {
        return 0;
    }

    let policy = CorridorThreatPolicy;
    facts
        .into_iter()
        .filter(|fact| fact.player == attacker)
        .filter(|fact| fact.origin.mv() == mv)
        .filter(|fact| policy.is_active_threat(board, attacker, fact))
        .map(|fact| policy.rank(fact.kind))
        .max()
        .unwrap_or(0)
}

fn raw_search_annotations_for_player(board: &Board, player: Color) -> Vec<TacticalMoveAnnotation> {
    let size = board.config.board_size;
    let mut annotations = Vec::with_capacity(size * size);
    for row in 0..size {
        for col in 0..size {
            annotations.push(raw_search_annotation_for_player(
                board,
                player,
                Move { row, col },
            ));
        }
    }
    annotations
}

fn raw_search_annotation_for_player(
    board: &Board,
    player: Color,
    mv: Move,
) -> TacticalMoveAnnotation {
    SearchThreatPolicy.raw_annotation_for_player(board, player, mv)
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

fn affected_local_axis_cells(board: &Board, mv: Move) -> Vec<Move> {
    let size = board.config.board_size;
    if size == 0 || mv.row >= size || mv.col >= size {
        return Vec::new();
    }

    let radius = board.config.win_length.saturating_add(1) as isize;
    let mut seen = vec![false; size * size];
    let mut cells = Vec::new();

    for (dr, dc) in DIRS {
        for step in -radius..=radius {
            let row = mv.row as isize + dr * step;
            let col = mv.col as isize + dc * step;
            if row < 0 || col < 0 || row >= size as isize || col >= size as isize {
                continue;
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
        }
    }

    cells
}

fn affected_immediate_win_cells(board: &Board, mv: Move) -> Vec<Move> {
    let size = board.config.board_size;
    if size == 0 || mv.row >= size || mv.col >= size {
        return Vec::new();
    }

    let radius = board.config.win_length as isize;
    let mut seen = vec![false; size * size];
    let mut cells = Vec::new();

    for (dr, dc) in DIRS {
        for step in -radius..=radius {
            let row = mv.row as isize + dr * step;
            let col = mv.col as isize + dc * step;
            if row < 0 || col < 0 || row >= size as isize || col >= size as isize {
                continue;
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
    use super::{
        cell_index, RebuildThreatFrontier, RollingFrontierFeatures, RollingThreatFrontier,
    };
    use crate::tactical::{LocalThreatKind, ScanThreatView, SearchThreatPolicy, ThreatView};
    use gomoku_core::{Board, Color, GameResult, Move, RuleConfig, Variant};

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
        for player in [Color::Black, Color::White] {
            assert_eq!(
                view.search_annotation_for_player(player, probe),
                scan.search_annotation_for_player(player, probe),
                "player annotation mismatch at {} for {:?}",
                probe.to_notation(),
                player
            );
        }
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
        for player in [Color::Black, Color::White] {
            assert_eq!(
                view.immediate_winning_moves_for(player),
                scan.immediate_winning_moves_for(player),
                "immediate win mismatch for {:?}",
                player
            );
        }
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
                for player in [Color::Black, Color::White] {
                    assert_eq!(
                        view.search_annotation_for_player(player, probe),
                        scan.search_annotation_for_player(player, probe),
                        "annotation mismatch at {} for {:?}",
                        probe.to_notation(),
                        player
                    );
                }
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
            assert_eq!(
                view.local_corridor_entry_rank(color, probe),
                scan.local_corridor_entry_rank(color, probe),
                "local entry rank mismatch at {} for {:?}",
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
        let mut board = board_from_moves(Variant::Renju, &["B8", "A1", "C8", "A2", "D8"]);
        let mut frontier = RollingThreatFrontier::from_board(&board);
        let probe = mv("A8");
        let probe_index = cell_index(board.config.board_size, probe);

        assert_eq!(board.current_player, Color::White);
        assert!(!frontier.black_raw_search_annotation_dirty[probe_index]);

        let played = mv("E8");
        board.apply_move(played).unwrap();
        frontier.apply_move(played).unwrap();

        assert_eq!(board.current_player, Color::Black);
        assert!(
            frontier.black_raw_search_annotation_dirty[probe_index],
            "same-axis black annotation should be dirty after white occupies E8"
        );
        assert_eq!(
            frontier.search_annotation_for_move(probe),
            ScanThreatView::new(&board).search_annotation_for_move(probe),
            "dirty same-axis annotation should be computed from the current board"
        );

        board.undo_move(played);
        frontier.undo_move(played);

        assert_eq!(board.current_player, Color::White);
        assert!(
            !frontier.black_raw_search_annotation_dirty[probe_index],
            "undo should restore the previous clean dirty flag"
        );
        assert_eq!(
            frontier.search_annotation_for_move(probe),
            ScanThreatView::new(&board).search_annotation_for_move(probe)
        );
    }

    #[test]
    fn rolling_frontier_dirties_only_local_search_annotation_cells() {
        let mut board = Board::new(RuleConfig::default());
        let mut frontier = RollingThreatFrontier::from_board(&board);
        let near_probe = mv("K8");
        let far_probe = mv("H8");
        let near_index = cell_index(board.config.board_size, near_probe);
        let far_index = cell_index(board.config.board_size, far_probe);

        let played = mv("O8");
        board.apply_move(played).unwrap();
        frontier.apply_move(played).unwrap();

        assert!(
            frontier.black_raw_search_annotation_dirty[near_index],
            "nearby same-axis annotations should be dirtied"
        );
        assert!(
            !frontier.black_raw_search_annotation_dirty[far_index],
            "same-axis annotations beyond the local threat radius should stay clean"
        );
        assert_eq!(
            frontier.search_annotation_for_move(far_probe),
            ScanThreatView::new(&board).search_annotation_for_move(far_probe)
        );
    }

    #[test]
    fn rolling_frontier_returns_empty_annotations_after_terminal_move() {
        let mut board = board_from_moves(
            Variant::Renju,
            &["H8", "A1", "I8", "A2", "J8", "A3", "K8", "A4"],
        );
        let mut frontier = RollingThreatFrontier::from_board(&board);
        let probe = mv("A5");
        let probe_index = cell_index(board.config.board_size, probe);

        assert!(
            !frontier.white_raw_search_annotations[probe_index]
                .local_threats
                .is_empty(),
            "white should have a cached raw winning annotation before the terminal move"
        );
        assert_eq!(
            frontier.search_annotation_for_move(probe),
            ScanThreatView::new(&board).search_annotation_for_move(probe)
        );

        let played = mv("L8");
        board.apply_move(played).unwrap();
        frontier.apply_move(played).unwrap();

        assert_eq!(board.result, GameResult::Winner(Color::Black));
        assert!(
            !frontier.white_raw_search_annotation_dirty[probe_index],
            "off-axis cached white annotation should still be clean so this catches terminal fallback"
        );
        assert_eq!(
            frontier.search_annotation_for_move(probe),
            ScanThreatView::new(&board).search_annotation_for_move(probe),
            "terminal boards must not return stale off-axis cached annotations"
        );
        assert!(frontier
            .search_annotation_for_move(probe)
            .local_threats
            .is_empty());
    }

    #[test]
    fn rolling_frontier_immediate_wins_do_not_depend_on_annotation_cache() {
        let board = board_from_moves(
            Variant::Renju,
            &["H8", "A1", "I8", "A2", "J8", "A3", "K8", "A4"],
        );
        let mut frontier = RollingThreatFrontier::from_board(&board);
        let winning_moves = vec![mv("G8"), mv("L8")];

        assert_eq!(
            ScanThreatView::new(&board).immediate_winning_moves_for(Color::Black),
            winning_moves
        );
        for winning_move in &winning_moves {
            let winning_index = cell_index(board.config.board_size, *winning_move);
            frontier.black_raw_search_annotations[winning_index]
                .local_threats
                .clear();
        }

        assert_eq!(
            frontier.immediate_winning_moves_for(Color::Black),
            winning_moves,
            "immediate wins should come from their own frontier index, not raw annotation cache"
        );
    }

    #[test]
    fn tactical_only_frontier_skips_move_facts_but_preserves_query_semantics() {
        let mut board = board_from_moves(Variant::Renju, &["H8", "A1", "I8", "A2", "J8"]);
        let mut frontier = RollingThreatFrontier::from_board_with_features(
            &board,
            RollingFrontierFeatures::TacticalOnly,
        );

        assert!(!frontier.maintains_move_facts());

        let played = mv("A3");
        board.apply_move(played).unwrap();
        frontier.apply_move(played).unwrap();

        let probe = mv("K8");
        assert_eq!(
            frontier.search_annotation_for_move(probe),
            ScanThreatView::new(&board).search_annotation_for_move(probe)
        );
        assert_eq!(
            frontier.has_move_local_corridor_entry(Color::Black, mv("J8")),
            ScanThreatView::new(&board).has_move_local_corridor_entry(Color::Black, mv("J8")),
            "tactical-only mode should fall back to scan for corridor-entry queries"
        );
    }

    #[test]
    fn tactical_only_frontier_tracks_immediate_wins_through_apply_and_undo() {
        let mut board = board_from_moves(
            Variant::Renju,
            &["H8", "A1", "I8", "A2", "J8", "A3", "K8", "A4"],
        );
        let mut frontier = RollingThreatFrontier::from_board_with_features(
            &board,
            RollingFrontierFeatures::TacticalOnly,
        );

        assert_eq!(
            frontier.immediate_winning_moves_for(Color::Black),
            ScanThreatView::new(&board).immediate_winning_moves_for(Color::Black)
        );

        let terminal = mv("L8");
        board.apply_move(terminal).unwrap();
        frontier.apply_move(terminal).unwrap();

        assert_eq!(board.result, GameResult::Winner(Color::Black));
        assert!(frontier
            .immediate_winning_moves_for(Color::White)
            .is_empty());

        board.undo_move(terminal);
        frontier.undo_move(terminal);

        assert_eq!(
            frontier.immediate_winning_moves_for(Color::Black),
            ScanThreatView::new(&board).immediate_winning_moves_for(Color::Black)
        );
    }

    #[test]
    fn immediate_win_cache_recomputes_dirty_cells_lazily() {
        let mut board = board_from_moves(Variant::Renju, &["H8", "A1", "I8", "A2", "J8", "A3"]);
        let mut frontier = RollingThreatFrontier::from_board_with_features(
            &board,
            RollingFrontierFeatures::TacticalOnly,
        );
        let played = mv("K8");
        let winning_probe = mv("L8");
        let winning_index = cell_index(board.config.board_size, winning_probe);

        board.apply_move(played).unwrap();
        frontier.apply_move(played).unwrap();

        assert!(
            frontier.black_immediate_win_dirty[winning_index],
            "apply should mark affected immediate-win cache entries dirty instead of recomputing eagerly"
        );

        let white_reply = mv("A4");
        board.apply_move(white_reply).unwrap();
        frontier.apply_move(white_reply).unwrap();

        assert_eq!(
            frontier.immediate_winning_moves_for_cached(Color::Black),
            ScanThreatView::new(&board).immediate_winning_moves_for(Color::Black)
        );
        assert!(
            !frontier.black_immediate_win_dirty[winning_index],
            "query should refresh and clean dirty immediate-win entries"
        );

        board.undo_move(white_reply);
        frontier.undo_move(white_reply);
        assert!(
            frontier.black_immediate_win_dirty[winning_index],
            "undo should restore dirty state inherited from the parent node"
        );

        assert_eq!(
            frontier.immediate_winning_moves_for_cached(Color::Black),
            ScanThreatView::new(&board).immediate_winning_moves_for(Color::Black)
        );
        assert!(
            !frontier.black_immediate_win_dirty[winning_index],
            "parent node query should be able to refresh the restored dirty entry"
        );

        board.undo_move(played);
        frontier.undo_move(played);

        assert_eq!(
            frontier.immediate_winning_moves_for(Color::Black),
            ScanThreatView::new(&board).immediate_winning_moves_for(Color::Black)
        );
    }

    #[test]
    fn rolling_frontier_filters_renju_forbidden_immediate_wins() {
        let board = board_from_moves(
            Variant::Renju,
            &["H8", "A1", "I8", "A2", "J8", "A3", "L8", "A4", "M8"],
        );
        let frontier = RollingThreatFrontier::from_board(&board);
        let forbidden_overline = mv("K8");

        assert_eq!(
            frontier.immediate_winning_moves_for(Color::Black),
            ScanThreatView::new(&board).immediate_winning_moves_for(Color::Black)
        );
        assert!(!frontier
            .immediate_winning_moves_for(Color::Black)
            .contains(&forbidden_overline));
    }

    #[test]
    fn rolling_frontier_lazily_filters_black_renju_raw_search_annotations() {
        let board = board_from_moves(
            Variant::Renju,
            &["H8", "G8", "I8", "A1", "J8", "A2", "L8", "A3"],
        );
        let frontier = RollingThreatFrontier::from_board(&board);
        let probe = mv("M8");
        let raw_annotation =
            &frontier.black_raw_search_annotations[probe.row * board.config.board_size + probe.col];

        assert!(
            raw_annotation
                .local_threats
                .iter()
                .any(|fact| fact.kind == LocalThreatKind::BrokenFour),
            "raw cache should retain the local forbidden-gap shape: {raw_annotation:?}"
        );

        let annotation = frontier.search_annotation_for_move(probe);
        assert_eq!(
            annotation,
            ScanThreatView::new(&board).search_annotation_for_move(probe)
        );
        assert!(
            annotation
                .local_threats
                .iter()
                .all(|fact| !SearchThreatPolicy.is_must_keep(fact)),
            "lazy effective filtering should remove forbidden-only forcing threats: {annotation:?}"
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
