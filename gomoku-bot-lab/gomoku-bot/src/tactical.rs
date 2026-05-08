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

impl LocalThreatKind {
    pub fn search_rank(self) -> u8 {
        match self {
            Self::Five => 5,
            Self::OpenFour => 4,
            Self::ClosedFour | Self::BrokenFour => 3,
            Self::OpenThree => 2,
            Self::ClosedThree | Self::BrokenThree => 1,
        }
    }

    pub fn corridor_rank(self) -> u8 {
        match self {
            Self::Five => 5,
            Self::OpenFour => 4,
            Self::ClosedFour | Self::BrokenFour => 3,
            Self::OpenThree => 2,
            Self::BrokenThree => 1,
            Self::ClosedThree => 0,
        }
    }

    pub fn is_search_forcing(self) -> bool {
        !matches!(self, Self::ClosedThree | Self::BrokenThree)
    }

    pub fn is_corridor_forcing(self) -> bool {
        matches!(
            self,
            Self::Five
                | Self::OpenFour
                | Self::ClosedFour
                | Self::BrokenFour
                | Self::OpenThree
                | Self::BrokenThree
        )
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

impl LocalThreatFact {
    pub fn is_search_forcing(&self) -> bool {
        self.kind.is_search_forcing()
    }

    pub fn is_corridor_forcing(&self) -> bool {
        self.kind.is_corridor_forcing()
    }

    pub fn origin_move(&self) -> Move {
        self.origin.mv()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalThreatContinuation {
    pub mv: Move,
    pub legal_cost_squares: Vec<Move>,
}

pub fn local_threat_facts_after_move(board: &Board, mv: Move) -> Vec<LocalThreatFact> {
    if !board.is_legal(mv) {
        return Vec::new();
    }

    local_threat_facts_after_legal_move(board, mv)
}

pub fn local_threat_facts_for_player(board: &Board, player: Color) -> Vec<LocalThreatFact> {
    let mut facts = Vec::new();
    board.for_each_occupied_color(player, |row, col| {
        let mv = Move { row, col };
        for &(dr, dc) in &DIRS {
            if is_run_start(board, mv, player, dr, dc) {
                if let Some(fact) = local_threat_fact_from_run_start(board, mv, player, dr, dc) {
                    push_unique_fact(&mut facts, fact);
                }
            }
            if let Some(fact) = broken_four_fact_through_move(board, mv, player, dr, dc) {
                push_unique_fact(&mut facts, fact);
            }
            if let Some(fact) = broken_three_fact_through_move(board, mv, player, dr, dc) {
                push_unique_fact(&mut facts, fact);
            }
        }
    });
    facts.sort_by_key(|fact| std::cmp::Reverse(fact.kind.corridor_rank()));
    facts
}

pub fn has_forcing_local_threat(board: &Board, player: Color) -> bool {
    local_threat_facts_for_player(board, player)
        .iter()
        .any(|fact| local_threat_is_corridor_forcing_for(board, player, fact))
}

pub fn local_threat_is_corridor_forcing_for(
    board: &Board,
    attacker: Color,
    fact: &LocalThreatFact,
) -> bool {
    fact.kind.is_corridor_forcing()
        && !legal_forcing_continuations_for_fact(board, attacker, fact).is_empty()
}

pub fn legal_forcing_continuations_for_fact(
    board: &Board,
    attacker: Color,
    fact: &LocalThreatFact,
) -> Vec<LocalThreatContinuation> {
    if !fact.kind.is_corridor_forcing() {
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

fn local_threat_facts_after_legal_move(board: &Board, mv: Move) -> Vec<LocalThreatFact> {
    let facts = local_threat_facts_after_legal_move_virtual(board, mv);
    if board.config.variant == Variant::Renju && board.current_player == Color::Black {
        renju_effective_black_local_threat_facts_after_legal_move(board, mv, facts)
    } else {
        facts
    }
}

fn local_threat_facts_after_legal_move_virtual(board: &Board, mv: Move) -> Vec<LocalThreatFact> {
    let player = board.current_player;
    let after = BoardAfterMove { board, mv, player };

    let mut facts = DIRS
        .iter()
        .filter_map(|&(dr, dc)| local_threat_fact_in_direction_view(&after, dr, dc))
        .collect::<Vec<_>>();
    facts.sort_by_key(|fact| std::cmp::Reverse(fact.kind.search_rank()));
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
    if fact.player != Color::Black
        || !fact.kind.is_search_forcing()
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

fn local_threat_fact_from_run_start(
    board: &Board,
    start: Move,
    player: Color,
    dr: isize,
    dc: isize,
) -> Option<LocalThreatFact> {
    let mut run = Vec::new();
    let mut row = start.row as isize;
    let mut col = start.col as isize;
    while in_bounds(board, row, col) && board.has_color(row as usize, col as usize, player) {
        run.push(Move {
            row: row as usize,
            col: col as usize,
        });
        row += dr;
        col += dc;
    }

    let before = offset_move(board, start, -dr, -dc, 1);
    let after = in_bounds(board, row, col).then_some(Move {
        row: row as usize,
        col: col as usize,
    });
    let before_open = before.is_some_and(|mv| board.is_empty(mv.row, mv.col));
    let after_open = after.is_some_and(|mv| board.is_empty(mv.row, mv.col));

    match (run.len(), before_open, after_open) {
        (4, true, true) => Some(LocalThreatFact {
            player,
            kind: LocalThreatKind::OpenFour,
            origin: LocalThreatOrigin::Existing(start),
            defense_squares: vec![before.expect("checked open"), after.expect("checked open")],
            rest_squares: Vec::new(),
        }),
        (4, true, false) => Some(LocalThreatFact {
            player,
            kind: LocalThreatKind::ClosedFour,
            origin: LocalThreatOrigin::Existing(start),
            defense_squares: vec![before.expect("checked open")],
            rest_squares: Vec::new(),
        }),
        (4, false, true) => Some(LocalThreatFact {
            player,
            kind: LocalThreatKind::ClosedFour,
            origin: LocalThreatOrigin::Existing(start),
            defense_squares: vec![after.expect("checked open")],
            rest_squares: Vec::new(),
        }),
        (3, true, true) => Some(LocalThreatFact {
            player,
            kind: LocalThreatKind::OpenThree,
            origin: LocalThreatOrigin::Existing(start),
            defense_squares: open_three_defense_squares(
                board,
                start,
                run.len(),
                dr,
                dc,
                before.expect("checked open"),
                after.expect("checked open"),
            )?,
            rest_squares: Vec::new(),
        }),
        _ => None,
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

fn open_three_defense_squares(
    board: &Board,
    start: Move,
    run_len: usize,
    dr: isize,
    dc: isize,
    before: Move,
    after: Move,
) -> Option<Vec<Move>> {
    let mut defenses = vec![before, after];
    let before_outer = offset_move(board, start, -dr, -dc, 2);
    let after_outer = offset_move(board, start, dr, dc, run_len + 1);
    let before_outer_open = before_outer.is_some_and(|mv| board.is_empty(mv.row, mv.col));
    let after_outer_open = after_outer.is_some_and(|mv| board.is_empty(mv.row, mv.col));

    if !before_outer_open && !after_outer_open {
        return None;
    }

    if !before_outer_open {
        if let Some(after_outer) = after_outer.filter(|mv| board.is_empty(mv.row, mv.col)) {
            push_unique_move(&mut defenses, after_outer);
        }
    }
    if !after_outer_open {
        if let Some(before_outer) = before_outer.filter(|mv| board.is_empty(mv.row, mv.col)) {
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

fn four_completion_squares_through_move(
    board: &Board,
    mv: Move,
    player: Color,
    dr: isize,
    dc: isize,
) -> Vec<Move> {
    let win_len = board.config.win_length as isize;
    let mut completions = Vec::new();

    for start in -(win_len - 1)..=0 {
        let mut player_count = 0usize;
        let mut empty_square = None;
        let mut blocked = false;

        for offset in start..start + win_len {
            let row = mv.row as isize + dr * offset;
            let col = mv.col as isize + dc * offset;
            if !in_bounds(board, row, col) {
                blocked = true;
                break;
            }

            let candidate = Move {
                row: row as usize,
                col: col as usize,
            };
            if board.has_color(candidate.row, candidate.col, player) {
                player_count += 1;
            } else if board.is_empty(candidate.row, candidate.col) && empty_square.is_none() {
                empty_square = Some(candidate);
            } else {
                blocked = true;
                break;
            }
        }

        let Some(empty_square) = empty_square else {
            continue;
        };
        if !blocked
            && player_count == board.config.win_length.saturating_sub(1)
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

fn broken_four_fact_through_move(
    board: &Board,
    mv: Move,
    player: Color,
    dr: isize,
    dc: isize,
) -> Option<LocalThreatFact> {
    let completions = four_completion_squares_through_move(board, mv, player, dr, dc);
    if completions.len() == 1
        && contiguous_run_len_through_move(board, mv, player, dr, dc) < board.config.win_length - 1
    {
        Some(LocalThreatFact {
            player,
            kind: LocalThreatKind::BrokenFour,
            origin: LocalThreatOrigin::Existing(mv),
            defense_squares: completions,
            rest_squares: Vec::new(),
        })
    } else {
        None
    }
}

fn broken_three_fact_through_move(
    board: &Board,
    mv: Move,
    player: Color,
    dr: isize,
    dc: isize,
) -> Option<LocalThreatFact> {
    let rest_squares = broken_three_rest_squares_through_move(board, mv, player, dr, dc);
    (!rest_squares.is_empty()).then_some(LocalThreatFact {
        player,
        kind: LocalThreatKind::BrokenThree,
        origin: LocalThreatOrigin::Existing(mv),
        defense_squares: rest_squares.clone(),
        rest_squares,
    })
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

fn broken_three_rest_squares_through_move(
    board: &Board,
    mv: Move,
    player: Color,
    dr: isize,
    dc: isize,
) -> Vec<Move> {
    let mut rest_squares = Vec::new();
    let win_len = board.config.win_length as isize;

    for start in -(win_len - 1)..=0 {
        let mut player_offsets = Vec::new();
        let mut empty_offsets = Vec::new();
        let mut blocked = false;

        for offset in start..start + win_len {
            let row = mv.row as isize + dr * offset;
            let col = mv.col as isize + dc * offset;
            if !in_bounds(board, row, col) {
                blocked = true;
                break;
            }

            let candidate = Move {
                row: row as usize,
                col: col as usize,
            };
            if board.has_color(candidate.row, candidate.col, player) {
                player_offsets.push(offset);
            } else if board.is_empty(candidate.row, candidate.col) {
                empty_offsets.push(offset);
            } else {
                blocked = true;
                break;
            }
        }

        if blocked
            || player_offsets.len() != board.config.win_length.saturating_sub(2)
            || empty_offsets.len() != 2
        {
            continue;
        }
        if player_offsets.windows(2).all(|pair| pair[1] == pair[0] + 1) {
            continue;
        }

        for offset in empty_offsets {
            let row = mv.row as isize + dr * offset;
            let col = mv.col as isize + dc * offset;
            if !in_bounds(board, row, col) {
                continue;
            }

            let rest = Move {
                row: row as usize,
                col: col as usize,
            };
            if four_completion_squares_after_virtual_rest_through_move(
                board, mv, player, dr, dc, rest,
            )
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

fn four_completion_squares_after_virtual_rest_through_move(
    board: &Board,
    mv: Move,
    player: Color,
    dr: isize,
    dc: isize,
    rest: Move,
) -> Vec<Move> {
    let win_len = board.config.win_length as isize;
    let mut completions = Vec::new();

    for start in -(win_len - 1)..=0 {
        let mut player_count = 0usize;
        let mut empty_square = None;
        let mut blocked = false;

        for offset in start..start + win_len {
            let row = mv.row as isize + dr * offset;
            let col = mv.col as isize + dc * offset;
            if !in_bounds(board, row, col) {
                blocked = true;
                break;
            }

            let candidate = Move {
                row: row as usize,
                col: col as usize,
            };
            if has_color_or_virtual_rest(board, candidate.row, candidate.col, player, rest) {
                player_count += 1;
            } else if board.is_empty(candidate.row, candidate.col) && empty_square.is_none() {
                empty_square = Some(candidate);
            } else {
                blocked = true;
                break;
            }
        }

        let Some(empty_square) = empty_square else {
            continue;
        };
        if !blocked
            && player_count == board.config.win_length.saturating_sub(1)
            && !completions.contains(&empty_square)
        {
            completions.push(empty_square);
        }
    }

    completions.sort_by_key(|mv| (mv.row, mv.col));
    completions
}

fn has_color_or_virtual_rest(
    board: &Board,
    row: usize,
    col: usize,
    player: Color,
    rest: Move,
) -> bool {
    (row == rest.row && col == rest.col) || board.has_color(row, col, player)
}

fn contiguous_run_len_through_move(
    board: &Board,
    mv: Move,
    player: Color,
    dr: isize,
    dc: isize,
) -> usize {
    1 + count_player_from_move(board, mv, player, dr, dc)
        + count_player_from_move(board, mv, player, -dr, -dc)
}

fn count_player_from_move(board: &Board, mv: Move, player: Color, dr: isize, dc: isize) -> usize {
    let mut count = 0usize;
    let mut row = mv.row as isize + dr;
    let mut col = mv.col as isize + dc;
    while in_bounds(board, row, col) && board.has_color(row as usize, col as usize, player) {
        count += 1;
        row += dr;
        col += dc;
    }
    count
}

fn is_run_start(board: &Board, mv: Move, player: Color, dr: isize, dc: isize) -> bool {
    let previous_row = mv.row as isize - dr;
    let previous_col = mv.col as isize - dc;
    !in_bounds(board, previous_row, previous_col)
        || !board.has_color(previous_row as usize, previous_col as usize, player)
}

fn offset_move(board: &Board, mv: Move, dr: isize, dc: isize, distance: usize) -> Option<Move> {
    let row = mv.row as isize + dr * distance as isize;
    let col = mv.col as isize + dc * distance as isize;
    in_bounds(board, row, col).then_some(Move {
        row: row as usize,
        col: col as usize,
    })
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

#[cfg(test)]
mod tests {
    use super::{
        has_forcing_local_threat, legal_forcing_continuations_for_fact,
        local_threat_facts_after_move, local_threat_facts_for_player, LocalThreatFact,
        LocalThreatKind, LocalThreatOrigin,
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

    fn board_from_moves(variant: Variant, moves: &[&str]) -> Board {
        let mut board = Board::new(RuleConfig {
            variant,
            ..RuleConfig::default()
        });
        apply_moves(&mut board, moves);
        board
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
                defense_squares: vec![mv("I9"), mv("M9"), mv("H9")],
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
                    defense_squares: vec![mv("I9"), mv("M9"), mv("H9")],
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
    fn renju_black_forbidden_only_local_threat_gets_no_tactical_credit() {
        let board = board_from_moves(
            Variant::Renju,
            &["H8", "G8", "I8", "A1", "J8", "A2", "L8", "A3"],
        );
        assert!(board.is_legal_for_color(mv("M8"), Color::Black));

        let facts = local_threat_facts_after_move(&board, mv("M8"));
        assert!(
            facts.iter().all(|fact| !fact.is_search_forcing()),
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
}
