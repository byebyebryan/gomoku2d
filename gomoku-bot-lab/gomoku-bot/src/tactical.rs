use gomoku_core::{Board, Color, GameResult, Move, Variant, DIRS};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalThreatKind {
    Five,
    OpenFour,
    ClosedFour,
    BrokenFour,
    OpenThree,
    ClosedThree,
    BrokenThree,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SearchThreatPolicy;

impl SearchThreatPolicy {
    pub fn rank(self, kind: LocalThreatKind) -> u8 {
        match kind {
            LocalThreatKind::Five => 5,
            LocalThreatKind::OpenFour => 4,
            LocalThreatKind::ClosedFour | LocalThreatKind::BrokenFour => 3,
            LocalThreatKind::OpenThree => 2,
            LocalThreatKind::ClosedThree | LocalThreatKind::BrokenThree => 1,
        }
    }

    pub fn ordering_score(self, kind: LocalThreatKind) -> i32 {
        match kind {
            LocalThreatKind::Five => 100_000,
            LocalThreatKind::OpenFour => 80_000,
            LocalThreatKind::ClosedFour | LocalThreatKind::BrokenFour => 70_000,
            LocalThreatKind::OpenThree => 50_000,
            LocalThreatKind::ClosedThree | LocalThreatKind::BrokenThree => 10_000,
        }
    }

    pub fn is_must_keep(self, fact: &LocalThreatFact) -> bool {
        !matches!(
            fact.kind,
            LocalThreatKind::ClosedThree | LocalThreatKind::BrokenThree
        )
    }

    pub fn facts_after_move(self, board: &Board, mv: Move) -> Vec<LocalThreatFact> {
        local_threat_facts_after_move(board, mv)
    }

    pub fn annotation_for_move(self, board: &Board, mv: Move) -> TacticalMoveAnnotation {
        if !board.is_legal(mv) {
            return TacticalMoveAnnotation {
                player: board.current_player,
                mv,
                local_threats: Vec::new(),
            };
        }

        let player = board.current_player;
        TacticalMoveAnnotation {
            player,
            mv,
            local_threats: self
                .facts_after_move(board, mv)
                .into_iter()
                .filter(|fact| fact.player == player)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CorridorThreatPolicy;

impl CorridorThreatPolicy {
    pub fn rank(self, kind: LocalThreatKind) -> u8 {
        match kind {
            LocalThreatKind::Five => 5,
            LocalThreatKind::OpenFour => 4,
            LocalThreatKind::ClosedFour | LocalThreatKind::BrokenFour => 3,
            LocalThreatKind::OpenThree => 2,
            LocalThreatKind::BrokenThree => 1,
            LocalThreatKind::ClosedThree => 0,
        }
    }

    pub fn is_corridor_kind(self, kind: LocalThreatKind) -> bool {
        matches!(
            kind,
            LocalThreatKind::Five
                | LocalThreatKind::OpenFour
                | LocalThreatKind::ClosedFour
                | LocalThreatKind::BrokenFour
                | LocalThreatKind::OpenThree
                | LocalThreatKind::BrokenThree
        )
    }

    pub fn is_active_threat(self, board: &Board, attacker: Color, fact: &LocalThreatFact) -> bool {
        self.is_corridor_kind(fact.kind)
            && !legal_forcing_continuations_for_fact(board, attacker, fact).is_empty()
    }

    pub fn active_threats(self, board: &Board, attacker: Color) -> Vec<LocalThreatFact> {
        let mut facts = raw_local_threat_facts_for_player(board, attacker)
            .into_iter()
            .filter(|fact| self.is_active_threat(board, attacker, fact))
            .collect::<Vec<_>>();
        facts.sort_by_key(|fact| std::cmp::Reverse(self.rank(fact.kind)));
        facts
    }

    pub fn has_active_threat(self, board: &Board, attacker: Color) -> bool {
        raw_local_threat_facts_for_player(board, attacker)
            .iter()
            .any(|fact| self.is_active_threat(board, attacker, fact))
    }

    pub fn defender_reply_moves(
        self,
        board: &Board,
        attacker: Color,
        actual_reply: Option<Move>,
    ) -> Vec<Move> {
        let defender = attacker.opponent();
        let mut replies = Vec::new();

        let mut facts = self.active_threats(board, attacker);
        if facts.is_empty() {
            return replies;
        }

        if let Some(actual_reply) = actual_reply {
            let actual_facts = facts
                .iter()
                .filter(|fact| fact.defense_squares.contains(&actual_reply))
                .cloned()
                .collect::<Vec<_>>();
            if !actual_facts.is_empty() {
                facts = actual_facts;
            }
        }

        let best_rank = facts
            .iter()
            .map(|fact| self.rank(fact.kind))
            .max()
            .expect("facts are not empty");
        for fact in facts
            .into_iter()
            .filter(|fact| self.rank(fact.kind) == best_rank)
        {
            add_corridor_defender_replies_for_fact(board, attacker, defender, &fact, &mut replies);
        }

        replies
    }

    pub fn attacker_move_rank(self, board: &Board, attacker: Color, mv: Move) -> u8 {
        if board.current_player != attacker || !board.is_legal_for_color(mv, attacker) {
            return 0;
        }
        let mut next = board.clone();
        if next.apply_move(mv).is_err() {
            return 0;
        }
        match next.result {
            GameResult::Winner(winner) if winner == attacker => {
                return self.rank(LocalThreatKind::Five)
            }
            GameResult::Winner(_) | GameResult::Draw => return 0,
            GameResult::Ongoing => {}
        }
        if !next.immediate_winning_moves_for(attacker).is_empty() {
            return self.rank(LocalThreatKind::OpenFour);
        }
        self.active_threats(&next, attacker)
            .into_iter()
            .map(|fact| self.rank(fact.kind))
            .max()
            .unwrap_or(0)
    }
}

pub trait ThreatView {
    /// Active immediate/imminent corridor threats for `attacker` on this board.
    fn active_corridor_threats(&self, attacker: Color) -> Vec<LocalThreatFact>;
    /// True when `mv` is already occupied by `attacker` and that local move is
    /// itself part of an active corridor threat.
    fn has_move_local_corridor_entry(&self, attacker: Color, mv: Move) -> bool;
    /// Legal defender replies to the strongest active corridor threat.
    fn defender_reply_moves(&self, attacker: Color, actual_reply: Option<Move>) -> Vec<Move>;
    /// Pre-move rank for an attacker candidate that may materialize a corridor.
    fn attacker_move_rank(&self, attacker: Color, mv: Move) -> u8;
}

#[derive(Debug, Clone, Copy)]
pub struct ScanThreatView<'a> {
    board: &'a Board,
}

impl<'a> ScanThreatView<'a> {
    pub fn new(board: &'a Board) -> Self {
        Self { board }
    }
}

impl ThreatView for ScanThreatView<'_> {
    fn active_corridor_threats(&self, attacker: Color) -> Vec<LocalThreatFact> {
        CorridorThreatPolicy.active_threats(self.board, attacker)
    }

    fn has_move_local_corridor_entry(&self, attacker: Color, mv: Move) -> bool {
        if !self.board.has_color(mv.row, mv.col, attacker) {
            return false;
        }

        let policy = CorridorThreatPolicy;
        let existing = BoardExistingMove {
            board: self.board,
            mv,
            player: attacker,
        };
        DIRS.iter().any(|&(dr, dc)| {
            local_threat_fact_in_direction_view(&existing, dr, dc)
                .is_some_and(|fact| policy.is_active_threat(self.board, attacker, &fact))
        })
    }

    fn defender_reply_moves(&self, attacker: Color, actual_reply: Option<Move>) -> Vec<Move> {
        CorridorThreatPolicy.defender_reply_moves(self.board, attacker, actual_reply)
    }

    fn attacker_move_rank(&self, attacker: Color, mv: Move) -> u8 {
        CorridorThreatPolicy.attacker_move_rank(self.board, attacker, mv)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalThreatOrigin {
    AfterMove(Move),
    Existing(Move),
}

impl LocalThreatOrigin {
    pub fn mv(self) -> Move {
        match self {
            Self::AfterMove(mv) | Self::Existing(mv) => mv,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalThreatFact {
    pub player: Color,
    pub kind: LocalThreatKind,
    pub origin: LocalThreatOrigin,
    pub defense_squares: Vec<Move>,
    pub rest_squares: Vec<Move>,
}

pub fn normalize_local_threat_fact(mut fact: LocalThreatFact) -> LocalThreatFact {
    normalize_moves(&mut fact.defense_squares);
    normalize_moves(&mut fact.rest_squares);
    fact
}

pub fn normalize_local_threat_facts(facts: Vec<LocalThreatFact>) -> Vec<LocalThreatFact> {
    let mut normalized = Vec::new();
    for fact in facts.into_iter().map(normalize_local_threat_fact) {
        push_unique_fact(&mut normalized, fact);
    }
    normalized.sort_by_key(local_threat_fact_sort_key);
    normalized
}

impl LocalThreatFact {
    pub fn origin_move(&self) -> Move {
        self.origin.mv()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TacticalMoveAnnotation {
    pub player: Color,
    pub mv: Move,
    pub local_threats: Vec<LocalThreatFact>,
}

impl TacticalMoveAnnotation {
    pub fn creates_immediate_or_multi_threat(&self) -> bool {
        let mut completion_squares = Vec::new();
        for fact in self.local_threats.iter() {
            match fact.kind {
                LocalThreatKind::Five | LocalThreatKind::OpenFour => return true,
                LocalThreatKind::ClosedFour | LocalThreatKind::BrokenFour => {
                    for defense in fact.defense_squares.iter().copied() {
                        if !completion_squares.contains(&defense) {
                            completion_squares.push(defense);
                        }
                    }
                    if completion_squares.len() >= 2 {
                        return true;
                    }
                }
                LocalThreatKind::OpenThree
                | LocalThreatKind::ClosedThree
                | LocalThreatKind::BrokenThree => {}
            }
        }

        false
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalThreatContinuation {
    pub mv: Move,
    pub legal_cost_squares: Vec<Move>,
}

pub fn local_threat_facts_after_move(board: &Board, mv: Move) -> Vec<LocalThreatFact> {
    search_local_threat_facts_after_move(board, mv)
}

pub fn raw_local_threat_facts_after_move(board: &Board, mv: Move) -> Vec<LocalThreatFact> {
    if !board.is_legal(mv) {
        return Vec::new();
    }

    normalize_local_threat_facts(local_threat_facts_after_legal_move_virtual(board, mv))
}

pub fn search_local_threat_facts_after_move(board: &Board, mv: Move) -> Vec<LocalThreatFact> {
    if !board.is_legal(mv) {
        return Vec::new();
    }

    let facts = local_threat_facts_after_legal_move_virtual(board, mv);
    let facts = if board.config.variant == Variant::Renju && board.current_player == Color::Black {
        renju_effective_black_local_threat_facts_after_legal_move(board, mv, facts)
    } else {
        facts
    };
    normalize_local_threat_facts(facts)
}

pub fn local_threat_facts_for_player(board: &Board, player: Color) -> Vec<LocalThreatFact> {
    raw_local_threat_facts_for_player(board, player)
}

pub fn raw_local_threat_facts_for_player(board: &Board, player: Color) -> Vec<LocalThreatFact> {
    let mut facts = Vec::new();
    board.for_each_occupied_color(player, |row, col| {
        for fact in raw_local_threat_facts_at_existing_move(board, player, Move { row, col }) {
            push_unique_fact(&mut facts, fact);
        }
    });
    normalize_local_threat_facts(facts)
}

pub fn raw_local_threat_facts_at_existing_move(
    board: &Board,
    player: Color,
    mv: Move,
) -> Vec<LocalThreatFact> {
    if !board.has_color(mv.row, mv.col, player) {
        return Vec::new();
    }

    let existing = BoardExistingMove { board, mv, player };
    normalize_local_threat_facts(
        DIRS.iter()
            .filter_map(|&(dr, dc)| local_threat_fact_in_direction_view(&existing, dr, dc))
            .collect(),
    )
}

pub fn has_forcing_local_threat(board: &Board, player: Color) -> bool {
    !ScanThreatView::new(board)
        .active_corridor_threats(player)
        .is_empty()
}

pub fn has_forcing_local_threat_at_move(board: &Board, player: Color, mv: Move) -> bool {
    ScanThreatView::new(board).has_move_local_corridor_entry(player, mv)
}

pub fn corridor_active_threats(board: &Board, attacker: Color) -> Vec<LocalThreatFact> {
    ScanThreatView::new(board).active_corridor_threats(attacker)
}

pub fn corridor_defender_reply_moves(
    board: &Board,
    attacker: Color,
    actual_reply: Option<Move>,
) -> Vec<Move> {
    ScanThreatView::new(board).defender_reply_moves(attacker, actual_reply)
}

pub fn corridor_attacker_move_rank(board: &Board, attacker: Color, mv: Move) -> u8 {
    ScanThreatView::new(board).attacker_move_rank(attacker, mv)
}

pub fn legal_forcing_continuations_for_fact(
    board: &Board,
    attacker: Color,
    fact: &LocalThreatFact,
) -> Vec<LocalThreatContinuation> {
    if !CorridorThreatPolicy.is_corridor_kind(fact.kind) {
        return Vec::new();
    }

    let mut attacker_turn = board.clone();
    attacker_turn.current_player = attacker;
    let mut continuations = Vec::new();
    for mv in fact.defense_squares.iter().copied() {
        if !attacker_turn.is_legal_for_color(mv, attacker) {
            continue;
        }

        let mut after_forcing = attacker_turn.clone();
        if after_forcing.apply_move(mv).is_err() {
            continue;
        }
        let legal_cost_squares = match after_forcing.result {
            GameResult::Winner(winner) if winner == attacker => vec![mv],
            GameResult::Winner(_) | GameResult::Draw => Vec::new(),
            GameResult::Ongoing => after_forcing.immediate_winning_moves_for(attacker),
        };
        if !legal_cost_squares.is_empty() {
            continuations.push(LocalThreatContinuation {
                mv,
                legal_cost_squares,
            });
        }
    }
    continuations
}

fn add_corridor_defender_replies_for_fact(
    board: &Board,
    attacker: Color,
    defender: Color,
    fact: &LocalThreatFact,
    replies: &mut Vec<Move>,
) {
    let legal_forcing_continuations = legal_forcing_continuations_for_fact(board, attacker, fact);
    for continuation in &legal_forcing_continuations {
        let mv = continuation.mv;
        if board.is_legal_for_color(mv, defender) {
            push_unique_move(replies, mv);
        }
    }

    let mut shared_cost_squares: Option<Vec<Move>> = None;
    for continuation in legal_forcing_continuations {
        let costs = continuation
            .legal_cost_squares
            .into_iter()
            .filter(|&mv| board.is_legal_for_color(mv, defender))
            .collect::<Vec<_>>();

        shared_cost_squares = Some(match shared_cost_squares {
            Some(shared) => shared
                .into_iter()
                .filter(|mv| costs.contains(mv))
                .collect::<Vec<_>>(),
            None => costs,
        });
    }

    for mv in shared_cost_squares.unwrap_or_default() {
        push_unique_move(replies, mv);
    }
}

fn local_threat_facts_after_legal_move_virtual(board: &Board, mv: Move) -> Vec<LocalThreatFact> {
    let player = board.current_player;
    let after = BoardAfterMove { board, mv, player };
    let search_policy = SearchThreatPolicy;

    let mut facts = DIRS
        .iter()
        .filter_map(|&(dr, dc)| local_threat_fact_in_direction_view(&after, dr, dc))
        .collect::<Vec<_>>();
    facts.sort_by_key(|fact| std::cmp::Reverse(search_policy.rank(fact.kind)));
    facts
}

fn renju_effective_black_local_threat_facts_after_legal_move(
    board: &Board,
    mv: Move,
    facts: Vec<LocalThreatFact>,
) -> Vec<LocalThreatFact> {
    let mut after = board.clone();
    after.apply_trusted_legal_move(mv);
    facts
        .into_iter()
        .filter_map(|fact| renju_effective_black_local_threat_fact(&after, fact))
        .collect()
}

fn renju_effective_black_local_threat_fact(
    board_after_gain: &Board,
    mut fact: LocalThreatFact,
) -> Option<LocalThreatFact> {
    let search_policy = SearchThreatPolicy;
    if fact.player != Color::Black
        || !search_policy.is_must_keep(&fact)
        || fact.kind == LocalThreatKind::Five
    {
        return Some(fact);
    }

    fact.defense_squares
        .retain(|&mv| renju_black_local_threat_continuation_is_effective(board_after_gain, mv));
    (!fact.defense_squares.is_empty()).then_some(fact)
}

fn renju_black_local_threat_continuation_is_effective(board_after_gain: &Board, mv: Move) -> bool {
    let mut attacker_turn = board_after_gain.clone();
    attacker_turn.current_player = Color::Black;
    if !attacker_turn.is_legal_for_color(mv, Color::Black) {
        return false;
    }

    let mut after_forcing = attacker_turn.clone();
    if after_forcing.apply_move(mv).is_err() {
        return false;
    }
    match after_forcing.result {
        GameResult::Winner(Color::Black) => true,
        GameResult::Winner(_) | GameResult::Draw => false,
        GameResult::Ongoing => !after_forcing
            .immediate_winning_moves_for(Color::Black)
            .is_empty(),
    }
}

struct BoardAfterMove<'a> {
    board: &'a Board,
    mv: Move,
    player: Color,
}

struct BoardExistingMove<'a> {
    board: &'a Board,
    mv: Move,
    player: Color,
}

trait TacticalBoardView {
    fn board(&self) -> &Board;
    fn mv(&self) -> Move;
    fn player(&self) -> Color;
    fn origin(&self) -> LocalThreatOrigin;
    fn has_color(&self, row: usize, col: usize, color: Color) -> bool;
    fn is_empty(&self, row: usize, col: usize) -> bool;

    fn win_length(&self) -> usize {
        self.board().config.win_length
    }

    fn in_bounds(&self, row: isize, col: isize) -> bool {
        in_bounds(self.board(), row, col)
    }

    fn has_color_or_extra_rest(&self, row: usize, col: usize, color: Color, rest: Move) -> bool {
        if row == rest.row && col == rest.col {
            color == self.player()
        } else {
            self.has_color(row, col, color)
        }
    }
}

impl TacticalBoardView for BoardAfterMove<'_> {
    fn board(&self) -> &Board {
        self.board
    }

    fn mv(&self) -> Move {
        self.mv
    }

    fn player(&self) -> Color {
        self.player
    }

    fn origin(&self) -> LocalThreatOrigin {
        LocalThreatOrigin::AfterMove(self.mv)
    }

    fn has_color(&self, row: usize, col: usize, color: Color) -> bool {
        if row == self.mv.row && col == self.mv.col {
            color == self.player
        } else {
            self.board.has_color(row, col, color)
        }
    }

    fn is_empty(&self, row: usize, col: usize) -> bool {
        !(row == self.mv.row && col == self.mv.col) && self.board.is_empty(row, col)
    }
}

impl TacticalBoardView for BoardExistingMove<'_> {
    fn board(&self) -> &Board {
        self.board
    }

    fn mv(&self) -> Move {
        self.mv
    }

    fn player(&self) -> Color {
        self.player
    }

    fn origin(&self) -> LocalThreatOrigin {
        LocalThreatOrigin::Existing(self.mv)
    }

    fn has_color(&self, row: usize, col: usize, color: Color) -> bool {
        self.board.has_color(row, col, color)
    }

    fn is_empty(&self, row: usize, col: usize) -> bool {
        self.board.is_empty(row, col)
    }
}

fn local_threat_fact_in_direction_view(
    board: &impl TacticalBoardView,
    dr: isize,
    dc: isize,
) -> Option<LocalThreatFact> {
    let player = board.player();
    let before = count_player_in_direction_view(board, -dr, -dc, player);
    let after = count_player_in_direction_view(board, dr, dc, player);
    let run_len = before + 1 + after;

    if run_len >= board.win_length() {
        return Some(LocalThreatFact {
            player,
            kind: LocalThreatKind::Five,
            origin: board.origin(),
            defense_squares: Vec::new(),
            rest_squares: Vec::new(),
        });
    }

    let four_completion_squares = four_completion_squares_through_view(board, dr, dc);
    match four_completion_squares.len() {
        2.. => {
            return Some(LocalThreatFact {
                player,
                kind: LocalThreatKind::OpenFour,
                origin: board.origin(),
                defense_squares: four_completion_squares,
                rest_squares: Vec::new(),
            });
        }
        1 => {
            let kind = if run_len == 4 {
                LocalThreatKind::ClosedFour
            } else {
                LocalThreatKind::BrokenFour
            };
            return Some(LocalThreatFact {
                player,
                kind,
                origin: board.origin(),
                defense_squares: four_completion_squares,
                rest_squares: Vec::new(),
            });
        }
        0 => {}
    }

    let mut open_ends = Vec::new();
    if let Some(open_before) = empty_offset_move_view(board, -dr, -dc, before + 1) {
        open_ends.push(open_before);
    }
    if let Some(open_after) = empty_offset_move_view(board, dr, dc, after + 1) {
        open_ends.push(open_after);
    }

    match (run_len, open_ends.len()) {
        (3, 2) => open_three_defense_squares_view(
            board,
            dr,
            dc,
            before,
            after,
            open_ends[0],
            open_ends[1],
        )
        .map(|defense_squares| LocalThreatFact {
            player,
            kind: LocalThreatKind::OpenThree,
            origin: board.origin(),
            defense_squares,
            rest_squares: Vec::new(),
        }),
        (3, 1) => Some(LocalThreatFact {
            player,
            kind: LocalThreatKind::ClosedThree,
            origin: board.origin(),
            defense_squares: open_ends,
            rest_squares: Vec::new(),
        }),
        _ => {
            let rest_squares = broken_three_rest_squares_through_view(board, dr, dc);
            if rest_squares.is_empty() {
                None
            } else {
                Some(LocalThreatFact {
                    player,
                    kind: LocalThreatKind::BrokenThree,
                    origin: board.origin(),
                    defense_squares: rest_squares.clone(),
                    rest_squares,
                })
            }
        }
    }
}

fn open_three_defense_squares_view(
    board: &impl TacticalBoardView,
    dr: isize,
    dc: isize,
    before_count: usize,
    after_count: usize,
    before: Move,
    after: Move,
) -> Option<Vec<Move>> {
    let mut defenses = vec![before, after];
    let before_outer = empty_offset_move_view(board, -dr, -dc, before_count + 2);
    let after_outer = empty_offset_move_view(board, dr, dc, after_count + 2);

    if before_outer.is_none() && after_outer.is_none() {
        return None;
    }

    if before_outer.is_none() {
        if let Some(after_outer) = after_outer {
            push_unique_move(&mut defenses, after_outer);
        }
    }
    if after_outer.is_none() {
        if let Some(before_outer) = before_outer {
            push_unique_move(&mut defenses, before_outer);
        }
    }

    Some(defenses)
}

fn four_completion_squares_through_view(
    board: &impl TacticalBoardView,
    dr: isize,
    dc: isize,
) -> Vec<Move> {
    let mv = board.mv();
    let player = board.player();
    let win_len = board.win_length() as isize;
    let mut completions = Vec::new();

    for start in -(win_len - 1)..=0 {
        let mut player_count = 0usize;
        let mut empty_square = None;
        let mut blocked = false;

        for offset in start..start + win_len {
            let row = mv.row as isize + dr * offset;
            let col = mv.col as isize + dc * offset;
            if !board.in_bounds(row, col) {
                blocked = true;
                break;
            }

            let row = row as usize;
            let col = col as usize;
            if board.has_color(row, col, player) {
                player_count += 1;
            } else if board.is_empty(row, col) && empty_square.is_none() {
                empty_square = Some(Move { row, col });
            } else {
                blocked = true;
                break;
            }
        }

        let Some(empty_square) = empty_square else {
            continue;
        };
        if !blocked
            && player_count == board.win_length().saturating_sub(1)
            && !completions.contains(&empty_square)
        {
            completions.push(empty_square);
        }
    }

    completions.sort_by_key(|mv| (mv.row, mv.col));
    completions
}

fn count_player_in_direction_view(
    board: &impl TacticalBoardView,
    dr: isize,
    dc: isize,
    player: Color,
) -> usize {
    let mut count = 0usize;
    let mv = board.mv();
    let mut row = mv.row as isize + dr;
    let mut col = mv.col as isize + dc;
    while board.in_bounds(row, col) && board.has_color(row as usize, col as usize, player) {
        count += 1;
        row += dr;
        col += dc;
    }
    count
}

fn empty_offset_move_view(
    board: &impl TacticalBoardView,
    dr: isize,
    dc: isize,
    distance: usize,
) -> Option<Move> {
    let mv = board.mv();
    let row = mv.row as isize + dr * distance as isize;
    let col = mv.col as isize + dc * distance as isize;
    if board.in_bounds(row, col) && board.is_empty(row as usize, col as usize) {
        Some(Move {
            row: row as usize,
            col: col as usize,
        })
    } else {
        None
    }
}

fn broken_three_rest_squares_through_view(
    board: &impl TacticalBoardView,
    dr: isize,
    dc: isize,
) -> Vec<Move> {
    let mut rest_squares = Vec::new();
    let mv = board.mv();
    let player = board.player();

    for start in -4isize..=0 {
        let mut player_offsets = Vec::new();
        let mut empty_offsets = Vec::new();
        let mut blocked = false;

        for offset in start..start + 5 {
            let row = mv.row as isize + dr * offset;
            let col = mv.col as isize + dc * offset;
            if !board.in_bounds(row, col) {
                blocked = true;
                break;
            }

            let row = row as usize;
            let col = col as usize;
            if board.has_color(row, col, player) {
                player_offsets.push(offset);
            } else if board.is_empty(row, col) {
                empty_offsets.push(offset);
            } else {
                blocked = true;
                break;
            }
        }

        if blocked || player_offsets.len() != 3 || empty_offsets.len() != 2 {
            continue;
        }
        if player_offsets.windows(2).all(|pair| pair[1] == pair[0] + 1) {
            continue;
        }

        for offset in empty_offsets {
            let row = mv.row as isize + dr * offset;
            let col = mv.col as isize + dc * offset;
            if !board.in_bounds(row, col) {
                continue;
            }

            let rest = Move {
                row: row as usize,
                col: col as usize,
            };
            if four_completion_squares_after_virtual_rest_through_view(board, dr, dc, rest)
                .is_empty()
            {
                continue;
            }
            push_unique_move(&mut rest_squares, rest);
        }
    }

    rest_squares.sort_by_key(|mv| (mv.row, mv.col));
    rest_squares
}

fn four_completion_squares_after_virtual_rest_through_view(
    board: &impl TacticalBoardView,
    dr: isize,
    dc: isize,
    rest: Move,
) -> Vec<Move> {
    let mv = board.mv();
    let player = board.player();
    let win_len = board.win_length() as isize;
    let mut completions = Vec::new();

    for start in -(win_len - 1)..=0 {
        let mut player_count = 0usize;
        let mut empty_square = None;
        let mut blocked = false;

        for offset in start..start + win_len {
            let row = mv.row as isize + dr * offset;
            let col = mv.col as isize + dc * offset;
            if !board.in_bounds(row, col) {
                blocked = true;
                break;
            }

            let row = row as usize;
            let col = col as usize;
            if board.has_color_or_extra_rest(row, col, player, rest) {
                player_count += 1;
            } else if board.is_empty(row, col) && empty_square.is_none() {
                empty_square = Some(Move { row, col });
            } else {
                blocked = true;
                break;
            }
        }

        let Some(empty_square) = empty_square else {
            continue;
        };
        if !blocked
            && player_count == board.win_length().saturating_sub(1)
            && !completions.contains(&empty_square)
        {
            completions.push(empty_square);
        }
    }

    completions.sort_by_key(|mv| (mv.row, mv.col));
    completions
}

fn in_bounds(board: &Board, row: isize, col: isize) -> bool {
    let size = board.config.board_size as isize;
    row >= 0 && row < size && col >= 0 && col < size
}

fn push_unique_move(moves: &mut Vec<Move>, mv: Move) {
    if !moves.contains(&mv) {
        moves.push(mv);
    }
}

fn normalize_moves(moves: &mut Vec<Move>) {
    moves.sort_by_key(|mv| (mv.row, mv.col));
    moves.dedup();
}

fn push_unique_fact(facts: &mut Vec<LocalThreatFact>, fact: LocalThreatFact) {
    if !facts
        .iter()
        .any(|existing| same_shape_fact(existing, &fact))
    {
        facts.push(fact);
    }
}

fn same_shape_fact(left: &LocalThreatFact, right: &LocalThreatFact) -> bool {
    left.player == right.player
        && left.kind == right.kind
        && left.defense_squares == right.defense_squares
        && left.rest_squares == right.rest_squares
}

fn local_threat_fact_sort_key(fact: &LocalThreatFact) -> (u8, u8, usize, usize, String, String) {
    (
        fact.player as u8,
        local_threat_kind_sort_key(fact.kind),
        fact.origin.mv().row,
        fact.origin.mv().col,
        move_list_sort_key(&fact.defense_squares),
        move_list_sort_key(&fact.rest_squares),
    )
}

fn local_threat_kind_sort_key(kind: LocalThreatKind) -> u8 {
    match kind {
        LocalThreatKind::Five => 0,
        LocalThreatKind::OpenFour => 1,
        LocalThreatKind::ClosedFour => 2,
        LocalThreatKind::BrokenFour => 3,
        LocalThreatKind::OpenThree => 4,
        LocalThreatKind::BrokenThree => 5,
        LocalThreatKind::ClosedThree => 6,
    }
}

fn move_list_sort_key(moves: &[Move]) -> String {
    moves
        .iter()
        .map(|mv| format!("{:02}:{:02}", mv.row, mv.col))
        .collect::<Vec<_>>()
        .join("|")
}

#[cfg(test)]
mod tests {
    use super::{
        corridor_active_threats, corridor_attacker_move_rank, corridor_defender_reply_moves,
        has_forcing_local_threat, has_forcing_local_threat_at_move,
        legal_forcing_continuations_for_fact, local_threat_facts_after_move,
        local_threat_facts_for_player, normalize_local_threat_facts,
        raw_local_threat_facts_after_move, raw_local_threat_facts_for_player, CorridorThreatPolicy,
        LocalThreatFact, LocalThreatKind, LocalThreatOrigin, ScanThreatView, SearchThreatPolicy,
        ThreatView,
    };
    use gomoku_core::{Board, Color, Move, RuleConfig, Variant};

    fn mv(notation: &str) -> Move {
        Move::from_notation(notation).expect("test move notation should parse")
    }

    fn apply_moves(board: &mut Board, moves: &[&str]) {
        for notation in moves {
            board.apply_move(mv(notation)).unwrap();
        }
    }

    fn fact(
        player: Color,
        kind: LocalThreatKind,
        origin: &str,
        defense_squares: &[&str],
        rest_squares: &[&str],
    ) -> LocalThreatFact {
        LocalThreatFact {
            player,
            kind,
            origin: LocalThreatOrigin::Existing(mv(origin)),
            defense_squares: defense_squares
                .iter()
                .map(|notation| mv(notation))
                .collect(),
            rest_squares: rest_squares.iter().map(|notation| mv(notation)).collect(),
        }
    }

    fn board_from_moves(variant: Variant, moves: &[&str]) -> Board {
        let mut board = Board::new(RuleConfig {
            variant,
            ..RuleConfig::default()
        });
        apply_moves(&mut board, moves);
        board
    }

    #[test]
    fn normalize_local_threat_facts_sorts_inner_moves_and_dedups_shapes() {
        let facts = vec![
            fact(
                Color::Black,
                LocalThreatKind::OpenThree,
                "J8",
                &["L8", "H8"],
                &["K8", "I8"],
            ),
            fact(
                Color::Black,
                LocalThreatKind::OpenThree,
                "I8",
                &["H8", "L8"],
                &["I8", "K8"],
            ),
            fact(
                Color::White,
                LocalThreatKind::ClosedFour,
                "C3",
                &["B3"],
                &[],
            ),
        ];

        let normalized = normalize_local_threat_facts(facts);

        assert_eq!(
            normalized,
            vec![
                fact(
                    Color::Black,
                    LocalThreatKind::OpenThree,
                    "J8",
                    &["H8", "L8"],
                    &["I8", "K8"],
                ),
                fact(
                    Color::White,
                    LocalThreatKind::ClosedFour,
                    "C3",
                    &["B3"],
                    &[],
                ),
            ]
        );
    }

    fn assert_raw_fact_parity(
        before_moves: &[&str],
        gain: &str,
        player: Color,
        kind: LocalThreatKind,
        defense_squares: &[&str],
        rest_squares: &[&str],
    ) {
        let mut before = Board::new(RuleConfig::default());
        apply_moves(&mut before, before_moves);

        let mut expected_defense_squares = defense_squares
            .iter()
            .map(|notation| mv(notation))
            .collect::<Vec<_>>();
        let mut expected_rest_squares = rest_squares
            .iter()
            .map(|notation| mv(notation))
            .collect::<Vec<_>>();
        expected_defense_squares.sort_by_key(|mv| (mv.row, mv.col));
        expected_rest_squares.sort_by_key(|mv| (mv.row, mv.col));

        let after_move_fact = raw_local_threat_facts_after_move(&before, mv(gain))
            .into_iter()
            .find(|fact| {
                fact.player == player
                    && fact.kind == kind
                    && fact.defense_squares == expected_defense_squares
                    && fact.rest_squares == expected_rest_squares
            })
            .unwrap_or_else(|| {
                panic!("after-move detector should see {kind:?} with expected squares")
            });

        let mut existing = before.clone();
        existing.apply_move(mv(gain)).unwrap();
        assert!(
            raw_local_threat_facts_for_player(&existing, player)
                .iter()
                .any(|fact| {
                    fact.kind == after_move_fact.kind
                        && fact.defense_squares == after_move_fact.defense_squares
                        && fact.rest_squares == after_move_fact.rest_squares
                }),
            "existing-board detector should produce the same raw shape as after-move detector"
        );
    }

    #[test]
    fn local_threat_facts_after_move_report_five_open_four_and_closed_four() {
        let mut five_board = Board::new(RuleConfig::default());
        apply_moves(
            &mut five_board,
            &["H8", "A1", "I8", "A2", "J8", "A3", "K8", "A4"],
        );
        assert_eq!(
            local_threat_facts_after_move(&five_board, mv("L8")),
            vec![LocalThreatFact {
                player: Color::Black,
                kind: LocalThreatKind::Five,
                origin: LocalThreatOrigin::AfterMove(mv("L8")),
                defense_squares: vec![],
                rest_squares: vec![],
            }]
        );

        let mut open_four_board = Board::new(RuleConfig::default());
        apply_moves(&mut open_four_board, &["H8", "A1", "I8", "A2", "J8", "A3"]);
        assert_eq!(
            local_threat_facts_after_move(&open_four_board, mv("K8")),
            vec![LocalThreatFact {
                player: Color::Black,
                kind: LocalThreatKind::OpenFour,
                origin: LocalThreatOrigin::AfterMove(mv("K8")),
                defense_squares: vec![mv("G8"), mv("L8")],
                rest_squares: vec![],
            }]
        );

        let mut closed_four_board = Board::new(RuleConfig::default());
        apply_moves(
            &mut closed_four_board,
            &["H8", "G8", "I8", "A1", "J8", "A2"],
        );
        assert_eq!(
            local_threat_facts_after_move(&closed_four_board, mv("K8")),
            vec![LocalThreatFact {
                player: Color::Black,
                kind: LocalThreatKind::ClosedFour,
                origin: LocalThreatOrigin::AfterMove(mv("K8")),
                defense_squares: vec![mv("L8")],
                rest_squares: vec![],
            }]
        );
    }

    #[test]
    fn local_threat_facts_after_move_report_open_closed_and_broken_three() {
        let mut open_three_board = Board::new(RuleConfig::default());
        apply_moves(&mut open_three_board, &["H8", "A1", "I8", "A2"]);
        assert_eq!(
            local_threat_facts_after_move(&open_three_board, mv("J8")),
            vec![LocalThreatFact {
                player: Color::Black,
                kind: LocalThreatKind::OpenThree,
                origin: LocalThreatOrigin::AfterMove(mv("J8")),
                defense_squares: vec![mv("G8"), mv("K8")],
                rest_squares: vec![],
            }]
        );

        let mut closed_three_board = Board::new(RuleConfig::default());
        apply_moves(&mut closed_three_board, &["H8", "G8", "I8", "A1"]);
        assert_eq!(
            local_threat_facts_after_move(&closed_three_board, mv("J8")),
            vec![LocalThreatFact {
                player: Color::Black,
                kind: LocalThreatKind::ClosedThree,
                origin: LocalThreatOrigin::AfterMove(mv("J8")),
                defense_squares: vec![mv("K8")],
                rest_squares: vec![],
            }]
        );

        let mut broken_three_board = Board::new(RuleConfig::default());
        apply_moves(&mut broken_three_board, &["H8", "A1", "K8", "C1"]);
        assert_eq!(
            local_threat_facts_after_move(&broken_three_board, mv("J8")),
            vec![LocalThreatFact {
                player: Color::Black,
                kind: LocalThreatKind::BrokenThree,
                origin: LocalThreatOrigin::AfterMove(mv("J8")),
                defense_squares: vec![mv("G8"), mv("I8"), mv("L8")],
                rest_squares: vec![mv("G8"), mv("I8"), mv("L8")],
            }]
        );
    }

    #[test]
    fn local_threat_facts_after_move_report_split_broken_three_rest_squares() {
        let mut split_three_board = Board::new(RuleConfig::default());
        apply_moves(&mut split_three_board, &["H8", "A1", "J8", "C1"]);
        assert_eq!(
            local_threat_facts_after_move(&split_three_board, mv("L8")),
            vec![LocalThreatFact {
                player: Color::Black,
                kind: LocalThreatKind::BrokenThree,
                origin: LocalThreatOrigin::AfterMove(mv("L8")),
                defense_squares: vec![mv("I8"), mv("K8")],
                rest_squares: vec![mv("I8"), mv("K8")],
            }]
        );

        let mut two_gap_three_board = Board::new(RuleConfig::default());
        apply_moves(&mut two_gap_three_board, &["H8", "A1", "K8", "C1"]);
        assert_eq!(
            local_threat_facts_after_move(&two_gap_three_board, mv("L8")),
            vec![LocalThreatFact {
                player: Color::Black,
                kind: LocalThreatKind::BrokenThree,
                origin: LocalThreatOrigin::AfterMove(mv("L8")),
                defense_squares: vec![mv("I8"), mv("J8")],
                rest_squares: vec![mv("I8"), mv("J8")],
            }]
        );
    }

    #[test]
    fn local_threat_facts_after_move_report_open_three_blocked_outer_variants() {
        let mut left_blocked_board = Board::new(RuleConfig::default());
        apply_moves(&mut left_blocked_board, &["J9", "H9", "K9", "A1"]);
        assert_eq!(
            local_threat_facts_after_move(&left_blocked_board, mv("L9")),
            vec![LocalThreatFact {
                player: Color::Black,
                kind: LocalThreatKind::OpenThree,
                origin: LocalThreatOrigin::AfterMove(mv("L9")),
                defense_squares: vec![mv("I9"), mv("M9"), mv("N9")],
                rest_squares: vec![],
            }]
        );

        let mut right_blocked_board = Board::new(RuleConfig::default());
        apply_moves(&mut right_blocked_board, &["J9", "N9", "K9", "A1"]);
        assert_eq!(
            local_threat_facts_after_move(&right_blocked_board, mv("L9")),
            vec![LocalThreatFact {
                player: Color::Black,
                kind: LocalThreatKind::OpenThree,
                origin: LocalThreatOrigin::AfterMove(mv("L9")),
                defense_squares: vec![mv("H9"), mv("I9"), mv("M9")],
                rest_squares: vec![],
            }]
        );
    }

    #[test]
    fn boxed_three_is_not_an_active_open_three() {
        let board = board_from_moves(Variant::Freestyle, &["J9", "H9", "K9", "N9", "L9"]);
        let facts = local_threat_facts_for_player(&board, Color::Black);
        assert!(
            facts
                .iter()
                .all(|fact| fact.kind != LocalThreatKind::OpenThree),
            "{facts:?}"
        );
    }

    #[test]
    fn local_threat_facts_for_player_report_open_closed_and_broken_fours() {
        let open_four = board_from_moves(
            Variant::Freestyle,
            &["H8", "A1", "I8", "A2", "J8", "A3", "K8"],
        );
        assert!(
            local_threat_facts_for_player(&open_four, Color::Black).contains(&LocalThreatFact {
                player: Color::Black,
                kind: LocalThreatKind::OpenFour,
                origin: LocalThreatOrigin::Existing(mv("H8")),
                defense_squares: vec![mv("G8"), mv("L8")],
                rest_squares: vec![],
            })
        );

        let closed_four = board_from_moves(
            Variant::Freestyle,
            &["H8", "G8", "I8", "A1", "J8", "A2", "K8"],
        );
        assert!(
            local_threat_facts_for_player(&closed_four, Color::Black).contains(&LocalThreatFact {
                player: Color::Black,
                kind: LocalThreatKind::ClosedFour,
                origin: LocalThreatOrigin::Existing(mv("H8")),
                defense_squares: vec![mv("L8")],
                rest_squares: vec![],
            })
        );

        let broken_four = board_from_moves(
            Variant::Freestyle,
            &["H8", "A1", "I8", "A2", "K8", "A3", "L8"],
        );
        assert!(
            local_threat_facts_for_player(&broken_four, Color::Black).contains(&LocalThreatFact {
                player: Color::Black,
                kind: LocalThreatKind::BrokenFour,
                origin: LocalThreatOrigin::Existing(mv("H8")),
                defense_squares: vec![mv("J8")],
                rest_squares: vec![],
            })
        );
    }

    #[test]
    fn local_threat_facts_for_player_report_open_three_outer_variants_and_broken_three() {
        let left_blocked = board_from_moves(Variant::Renju, &["J9", "H9", "K9", "A1", "L9"]);
        assert!(
            local_threat_facts_for_player(&left_blocked, Color::Black).contains(&LocalThreatFact {
                player: Color::Black,
                kind: LocalThreatKind::OpenThree,
                origin: LocalThreatOrigin::Existing(mv("J9")),
                defense_squares: vec![mv("I9"), mv("M9"), mv("N9")],
                rest_squares: vec![],
            })
        );

        let right_blocked = board_from_moves(Variant::Renju, &["J9", "N9", "K9", "A1", "L9"]);
        assert!(
            local_threat_facts_for_player(&right_blocked, Color::Black).contains(
                &LocalThreatFact {
                    player: Color::Black,
                    kind: LocalThreatKind::OpenThree,
                    origin: LocalThreatOrigin::Existing(mv("J9")),
                    defense_squares: vec![mv("H9"), mv("I9"), mv("M9")],
                    rest_squares: vec![],
                }
            )
        );

        let split_three = board_from_moves(Variant::Freestyle, &["H8", "A1", "J8", "C1", "L8"]);
        assert!(
            local_threat_facts_for_player(&split_three, Color::Black).contains(&LocalThreatFact {
                player: Color::Black,
                kind: LocalThreatKind::BrokenThree,
                origin: LocalThreatOrigin::Existing(mv("H8")),
                defense_squares: vec![mv("I8"), mv("K8")],
                rest_squares: vec![mv("I8"), mv("K8")],
            })
        );
    }

    #[test]
    fn match_1729_closed_three_endpoint_is_not_a_corridor_reply() {
        const MATCH_1729_PREFIX_38: &[&str] = &[
            "H8", "I8", "H7", "I7", "H6", "H5", "I6", "G10", "J5", "G8", "G6", "J6", "F6", "E6",
            "G7", "I9", "K4", "L3", "E5", "D4", "H9", "H10", "I5", "J4", "F8", "E9", "F10", "F7",
            "F11", "F12", "G11", "H11", "E11", "I12", "F9", "D12", "I13", "H12",
        ];
        const MATCH_1729_PREFIX_39: &[&str] = &[
            "H8", "I8", "H7", "I7", "H6", "H5", "I6", "G10", "J5", "G8", "G6", "J6", "F6", "E6",
            "G7", "I9", "K4", "L3", "E5", "D4", "H9", "H10", "I5", "J4", "F8", "E9", "F10", "F7",
            "F11", "F12", "G11", "H11", "E11", "I12", "F9", "D12", "I13", "H12", "G12",
        ];
        const MATCH_1729_PREFIX_40: &[&str] = &[
            "H8", "I8", "H7", "I7", "H6", "H5", "I6", "G10", "J5", "G8", "G6", "J6", "F6", "E6",
            "G7", "I9", "K4", "L3", "E5", "D4", "H9", "H10", "I5", "J4", "F8", "E9", "F10", "F7",
            "F11", "F12", "G11", "H11", "E11", "I12", "F9", "D12", "I13", "H12", "G12", "K14",
        ];
        const MATCH_1729_PREFIX_41: &[&str] = &[
            "H8", "I8", "H7", "I7", "H6", "H5", "I6", "G10", "J5", "G8", "G6", "J6", "F6", "E6",
            "G7", "I9", "K4", "L3", "E5", "D4", "H9", "H10", "I5", "J4", "F8", "E9", "F10", "F7",
            "F11", "F12", "G11", "H11", "E11", "I12", "F9", "D12", "I13", "H12", "G12", "K14",
            "J13",
        ];

        for moves in [MATCH_1729_PREFIX_38, MATCH_1729_PREFIX_40] {
            let board = board_from_moves(Variant::Renju, moves);
            assert_eq!(board.current_player, Color::Black);
            assert!(board.is_legal_for_color(mv("H13"), Color::Black));
            assert!(
                !board
                    .immediate_winning_moves_for(Color::White)
                    .contains(&mv("H13")),
                "H13 is not an immediate white win at this prefix"
            );

            let replies = CorridorThreatPolicy.defender_reply_moves(&board, Color::White, None);
            assert!(
                !replies.contains(&mv("H13")),
                "H13 is only the endpoint of a closed white three and should not be a forced corridor reply: {replies:?}"
            );
        }

        for moves in [MATCH_1729_PREFIX_39, MATCH_1729_PREFIX_41] {
            let board = board_from_moves(Variant::Renju, moves);
            assert_eq!(board.current_player, Color::White);
            assert!(
                !board
                    .immediate_winning_moves_for(Color::White)
                    .contains(&mv("H13")),
                "H13 is not an immediate white win at this prefix"
            );
        }
    }

    #[test]
    fn raw_after_move_and_existing_board_facts_share_shape_logic() {
        assert_raw_fact_parity(
            &["H8", "A1", "I8", "A2"],
            "J8",
            Color::Black,
            LocalThreatKind::OpenThree,
            &["G8", "K8"],
            &[],
        );
        assert_raw_fact_parity(
            &["J9", "H9", "K9", "A1"],
            "L9",
            Color::Black,
            LocalThreatKind::OpenThree,
            &["I9", "M9", "N9"],
            &[],
        );
        assert_raw_fact_parity(
            &["J9", "N9", "K9", "A1"],
            "L9",
            Color::Black,
            LocalThreatKind::OpenThree,
            &["I9", "M9", "H9"],
            &[],
        );
        assert_raw_fact_parity(
            &["H8", "A1", "J8", "C1"],
            "L8",
            Color::Black,
            LocalThreatKind::BrokenThree,
            &["I8", "K8"],
            &["I8", "K8"],
        );
        assert_raw_fact_parity(
            &["H8", "A1", "K8", "C1"],
            "L8",
            Color::Black,
            LocalThreatKind::BrokenThree,
            &["I8", "J8"],
            &["I8", "J8"],
        );
    }

    #[test]
    fn search_and_corridor_policies_split_broken_three_semantics() {
        let mut board = Board::new(RuleConfig::default());
        apply_moves(&mut board, &["H8", "A1", "K8", "C1"]);

        let annotation = SearchThreatPolicy.annotation_for_move(&board, mv("J8"));
        let broken_three = annotation
            .local_threats
            .iter()
            .find(|fact| fact.kind == LocalThreatKind::BrokenThree)
            .expect("search policy should retain broken-three material");
        assert!(!SearchThreatPolicy.is_must_keep(broken_three));

        let mut existing = board.clone();
        existing.apply_move(mv("J8")).unwrap();
        let corridor_fact = raw_local_threat_facts_for_player(&existing, Color::Black)
            .into_iter()
            .find(|fact| fact.kind == LocalThreatKind::BrokenThree)
            .expect("corridor policy should see the existing broken three");
        assert!(CorridorThreatPolicy.is_active_threat(&existing, Color::Black, &corridor_fact));
    }

    #[test]
    fn scan_threat_view_matches_existing_corridor_queries() {
        let board = board_from_moves(Variant::Renju, &["H8", "A1", "I8", "A2", "J8", "A3", "C3"]);
        let view = ScanThreatView::new(&board);

        assert_eq!(
            view.active_corridor_threats(Color::Black),
            corridor_active_threats(&board, Color::Black)
        );
        assert_eq!(
            view.defender_reply_moves(Color::Black, None),
            corridor_defender_reply_moves(&board, Color::Black, None)
        );
        assert_eq!(
            view.attacker_move_rank(Color::Black, mv("K8")),
            corridor_attacker_move_rank(&board, Color::Black, mv("K8"))
        );
        assert_eq!(
            view.has_move_local_corridor_entry(Color::Black, mv("J8")),
            has_forcing_local_threat_at_move(&board, Color::Black, mv("J8"))
        );
        assert!(
            !view.has_move_local_corridor_entry(Color::Black, mv("C3")),
            "quiet existing stones should not become corridor entries"
        );
    }

    #[test]
    fn renju_black_forbidden_only_local_threat_gets_no_tactical_credit() {
        let board = board_from_moves(
            Variant::Renju,
            &["H8", "G8", "I8", "A1", "J8", "A2", "L8", "A3"],
        );
        assert!(board.is_legal_for_color(mv("M8"), Color::Black));

        let raw_facts = raw_local_threat_facts_after_move(&board, mv("M8"));
        assert!(
            raw_facts
                .iter()
                .any(|fact| fact.kind == LocalThreatKind::BrokenFour),
            "raw detector should preserve the forbidden-gap shape: {raw_facts:?}"
        );

        let facts = local_threat_facts_after_move(&board, mv("M8"));
        let search_policy = SearchThreatPolicy;
        assert!(
            facts.iter().all(|fact| !search_policy.is_must_keep(fact)),
            "forbidden-only local threat should not be forcing: {facts:?}"
        );
    }

    #[test]
    fn renju_forbidden_only_existing_local_threat_is_not_forcing() {
        let board = board_from_moves(
            Variant::Renju,
            &["H8", "G8", "I8", "A1", "J8", "A2", "L8", "A3", "M8"],
        );
        assert!(!board.is_legal_for_color(mv("K8"), Color::Black));

        let facts = local_threat_facts_for_player(&board, Color::Black);
        let forbidden_gap_four = facts
            .iter()
            .find(|fact| {
                fact.kind == LocalThreatKind::BrokenFour && fact.defense_squares == vec![mv("K8")]
            })
            .unwrap_or_else(|| panic!("expected raw forbidden broken-four fact: {facts:?}"));
        assert!(
            legal_forcing_continuations_for_fact(&board, Color::Black, forbidden_gap_four)
                .is_empty()
        );
        assert!(!has_forcing_local_threat(&board, Color::Black));
    }

    #[test]
    fn localized_forcing_threat_gate_checks_only_requested_move() {
        let board = board_from_moves(
            Variant::Freestyle,
            &["H8", "A1", "I8", "A2", "J8", "A3", "C3"],
        );

        assert!(has_forcing_local_threat_at_move(
            &board,
            Color::Black,
            mv("J8")
        ));
        assert!(!has_forcing_local_threat_at_move(
            &board,
            Color::Black,
            mv("C3")
        ));
        assert!(!has_forcing_local_threat_at_move(
            &board,
            Color::White,
            mv("J8")
        ));
    }
}
