use super::*;

pub(super) fn local_threat_facts_after_legal_move_virtual_for_player(
    board: &Board,
    player: Color,
    mv: Move,
) -> Vec<LocalThreatFact> {
    let after = BoardAfterMove { board, mv, player };
    let search_policy = SearchThreatPolicy;

    let mut facts = DIRS
        .iter()
        .filter_map(|&(dr, dc)| local_threat_fact_in_direction_view(&after, dr, dc))
        .collect::<Vec<_>>();
    facts.sort_by_key(|fact| std::cmp::Reverse(search_policy.rank(fact.kind)));
    facts
}

pub(super) fn renju_effective_black_local_threat_facts_after_legal_move(
    board: &Board,
    mv: Move,
    facts: Vec<LocalThreatFact>,
) -> Vec<LocalThreatFact> {
    let mut after = board.clone();
    after.current_player = Color::Black;
    if !after.is_legal_for_color(mv, Color::Black) {
        return Vec::new();
    }
    after.apply_trusted_legal_move(mv);
    facts
        .into_iter()
        .filter_map(|fact| renju_effective_black_local_threat_fact(&after, fact))
        .collect()
}

pub(super) fn renju_effective_black_local_threat_fact(
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

    if fact.rest_squares.is_empty() {
        fact.defense_squares
            .retain(|&mv| renju_black_local_threat_continuation_is_effective(board_after_gain, mv));
        (!fact.defense_squares.is_empty()).then_some(fact)
    } else {
        fact.rest_squares
            .retain(|&mv| renju_black_local_threat_continuation_is_effective(board_after_gain, mv));
        (!fact.rest_squares.is_empty()).then_some(fact)
    }
}

pub(super) fn renju_black_local_threat_continuation_is_effective(
    board_after_gain: &Board,
    mv: Move,
) -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    let start = std::time::Instant::now();
    let result = renju_black_local_threat_continuation_is_effective_inner(board_after_gain, mv);
    #[cfg(not(target_arch = "wasm32"))]
    record_renju_effective_filter_continuation(start.elapsed());
    #[cfg(target_arch = "wasm32")]
    record_renju_effective_filter_continuation();
    result
}

pub(super) fn renju_black_local_threat_continuation_is_effective_inner(
    board_after_gain: &Board,
    mv: Move,
) -> bool {
    local_forcing_continuation(board_after_gain, Color::Black, mv).is_some()
}

pub(super) struct BoardAfterMove<'a> {
    pub(super) board: &'a Board,
    pub(super) mv: Move,
    pub(super) player: Color,
}

pub(super) struct BoardExistingMove<'a> {
    pub(super) board: &'a Board,
    pub(super) mv: Move,
    pub(super) player: Color,
}

pub(super) trait TacticalBoardView {
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

pub(super) fn local_threat_fact_in_direction_view(
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
            let broken_three = broken_three_squares_through_view(board, dr, dc);
            if broken_three.rest_squares.is_empty() {
                None
            } else {
                Some(LocalThreatFact {
                    player,
                    kind: LocalThreatKind::BrokenThree,
                    origin: board.origin(),
                    defense_squares: broken_three.defense_squares,
                    rest_squares: broken_three.rest_squares,
                })
            }
        }
    }
}

pub(super) fn open_three_defense_squares_view(
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

pub(super) fn four_completion_squares_through_view(
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

pub(super) fn count_player_in_direction_view(
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

pub(super) fn empty_offset_move_view(
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

#[derive(Debug, Default)]
pub(super) struct BrokenThreeSquares {
    pub(super) defense_squares: Vec<Move>,
    pub(super) rest_squares: Vec<Move>,
}

pub(super) fn broken_three_squares_through_view(
    board: &impl TacticalBoardView,
    dr: isize,
    dc: isize,
) -> BrokenThreeSquares {
    let mut squares = BrokenThreeSquares::default();
    let mv = board.mv();
    let player = board.player();

    for start in -3isize..=0 {
        let mut player_offsets = Vec::new();
        let mut empty_offsets = Vec::new();
        let mut blocked = false;

        for offset in start..start + 4 {
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

        if blocked || player_offsets.len() != 3 || empty_offsets.len() != 1 {
            continue;
        }

        let gap_offset = empty_offsets[0];
        if gap_offset == start || gap_offset == start + 3 {
            continue;
        }

        let before = mv.row as isize + dr * (start - 1);
        let before_col = mv.col as isize + dc * (start - 1);
        let after = mv.row as isize + dr * (start + 4);
        let after_col = mv.col as isize + dc * (start + 4);
        let gap_row = mv.row as isize + dr * gap_offset;
        let gap_col = mv.col as isize + dc * gap_offset;

        if !board.in_bounds(before, before_col)
            || !board.in_bounds(after, after_col)
            || !board.is_empty(before as usize, before_col as usize)
            || !board.is_empty(after as usize, after_col as usize)
        {
            continue;
        }

        if !board.in_bounds(gap_row, gap_col) {
            continue;
        }
        let before = Move {
            row: before as usize,
            col: before_col as usize,
        };
        let gap = Move {
            row: gap_row as usize,
            col: gap_col as usize,
        };
        let after = Move {
            row: after as usize,
            col: after_col as usize,
        };

        push_unique_move(&mut squares.defense_squares, before);
        push_unique_move(&mut squares.defense_squares, gap);
        push_unique_move(&mut squares.defense_squares, after);
        push_unique_move(&mut squares.rest_squares, gap);
    }

    squares.defense_squares.sort_by_key(|mv| (mv.row, mv.col));
    squares.rest_squares.sort_by_key(|mv| (mv.row, mv.col));
    squares
}

pub(super) fn local_threat_evidence_stones_view(
    view: &impl TacticalBoardView,
    fact: &LocalThreatFact,
    exclude_origin: bool,
) -> Vec<Move> {
    match fact.kind {
        LocalThreatKind::Five => five_evidence_stones_view(view, exclude_origin),
        LocalThreatKind::OpenFour | LocalThreatKind::ClosedFour | LocalThreatKind::BrokenFour => {
            let mut evidence = Vec::new();
            for defense in fact.defense_squares.iter().copied() {
                completion_window_evidence_stones_view(
                    view,
                    defense,
                    exclude_origin,
                    &mut evidence,
                );
            }
            if evidence.is_empty() {
                span_evidence_stones_view(view, fact, exclude_origin)
            } else {
                evidence
            }
        }
        LocalThreatKind::OpenThree
        | LocalThreatKind::ClosedThree
        | LocalThreatKind::BrokenThree => span_evidence_stones_view(view, fact, exclude_origin),
    }
}

pub(super) fn five_evidence_stones_view(
    view: &impl TacticalBoardView,
    exclude_origin: bool,
) -> Vec<Move> {
    let mut evidence = Vec::new();
    let origin = view.mv();

    for &(dr, dc) in &DIRS {
        let before = count_player_in_direction_view(view, -dr, -dc, view.player());
        let after = count_player_in_direction_view(view, dr, dc, view.player());
        if before + 1 + after < view.win_length() {
            continue;
        }

        for offset in -(before as isize)..=(after as isize) {
            let row = origin.row as isize + dr * offset;
            let col = origin.col as isize + dc * offset;
            if !view.in_bounds(row, col) {
                continue;
            }
            let mv = Move {
                row: row as usize,
                col: col as usize,
            };
            if exclude_origin && mv == origin {
                continue;
            }
            if view.has_color(mv.row, mv.col, view.player()) {
                push_unique_move(&mut evidence, mv);
            }
        }
    }

    evidence
}

pub(super) fn completion_window_evidence_stones_view(
    view: &impl TacticalBoardView,
    completion: Move,
    exclude_origin: bool,
    evidence: &mut Vec<Move>,
) {
    let origin = view.mv();
    let win_len = view.win_length() as isize;

    for &(dr, dc) in &DIRS {
        for start in -(win_len - 1)..=0 {
            let mut window_stones = Vec::new();
            let mut saw_completion = false;
            let mut blocked = false;

            for offset in start..start + win_len {
                let row = completion.row as isize + dr * offset;
                let col = completion.col as isize + dc * offset;
                if !view.in_bounds(row, col) {
                    blocked = true;
                    break;
                }

                let mv = Move {
                    row: row as usize,
                    col: col as usize,
                };
                if mv == completion {
                    saw_completion = true;
                } else if view.has_color(mv.row, mv.col, view.player()) {
                    if !(exclude_origin && mv == origin) {
                        window_stones.push(mv);
                    }
                } else {
                    blocked = true;
                    break;
                }
            }

            if !blocked && saw_completion && window_stones.len() == view.win_length() - 1 {
                for mv in window_stones {
                    push_unique_move(evidence, mv);
                }
            }
        }
    }
}

pub(super) fn span_evidence_stones_view(
    view: &impl TacticalBoardView,
    fact: &LocalThreatFact,
    exclude_origin: bool,
) -> Vec<Move> {
    let Some((dr, dc, mut min_offset, mut max_offset)) = evidence_line_span(view, fact) else {
        return Vec::new();
    };

    let origin = view.mv();
    while player_at_offset(view, dr, dc, min_offset - 1) {
        min_offset -= 1;
    }
    while player_at_offset(view, dr, dc, max_offset + 1) {
        max_offset += 1;
    }

    let mut evidence = Vec::new();
    for offset in min_offset..=max_offset {
        let row = origin.row as isize + dr * offset;
        let col = origin.col as isize + dc * offset;
        if !view.in_bounds(row, col) {
            continue;
        }
        let mv = Move {
            row: row as usize,
            col: col as usize,
        };
        if exclude_origin && mv == origin {
            continue;
        }
        if view.has_color(mv.row, mv.col, view.player()) {
            push_unique_move(&mut evidence, mv);
        }
    }

    evidence
}

pub(super) fn evidence_line_span(
    view: &impl TacticalBoardView,
    fact: &LocalThreatFact,
) -> Option<(isize, isize, isize, isize)> {
    let points = fact
        .defense_squares
        .iter()
        .chain(fact.rest_squares.iter())
        .copied()
        .collect::<Vec<_>>();
    if points.is_empty() {
        return None;
    }

    for &(dr, dc) in &DIRS {
        let mut offsets = vec![0isize];
        let mut all_aligned = true;
        for point in points.iter().copied() {
            match line_offset(view.mv(), point, dr, dc) {
                Some(offset) => offsets.push(offset),
                None => {
                    all_aligned = false;
                    break;
                }
            }
        }
        if all_aligned {
            let min_offset = *offsets.iter().min()?;
            let max_offset = *offsets.iter().max()?;
            return Some((dr, dc, min_offset, max_offset));
        }
    }

    None
}

pub(super) fn line_offset(origin: Move, point: Move, dr: isize, dc: isize) -> Option<isize> {
    let row_delta = point.row as isize - origin.row as isize;
    let col_delta = point.col as isize - origin.col as isize;

    match (dr, dc) {
        (0, dc) => {
            if row_delta == 0 && dc != 0 && col_delta % dc == 0 {
                Some(col_delta / dc)
            } else {
                None
            }
        }
        (dr, 0) => {
            if col_delta == 0 && dr != 0 && row_delta % dr == 0 {
                Some(row_delta / dr)
            } else {
                None
            }
        }
        (dr, dc) => {
            if dr == 0 || dc == 0 || row_delta % dr != 0 || col_delta % dc != 0 {
                return None;
            }
            let row_offset = row_delta / dr;
            let col_offset = col_delta / dc;
            (row_offset == col_offset).then_some(row_offset)
        }
    }
}

pub(super) fn player_at_offset(
    view: &impl TacticalBoardView,
    dr: isize,
    dc: isize,
    offset: isize,
) -> bool {
    let origin = view.mv();
    let row = origin.row as isize + dr * offset;
    let col = origin.col as isize + dc * offset;
    view.in_bounds(row, col) && view.has_color(row as usize, col as usize, view.player())
}

pub(super) fn in_bounds(board: &Board, row: isize, col: isize) -> bool {
    let size = board.config.board_size as isize;
    row >= 0 && row < size && col >= 0 && col < size
}

pub(super) fn push_unique_move(moves: &mut Vec<Move>, mv: Move) {
    if !moves.contains(&mv) {
        moves.push(mv);
    }
}

pub(super) fn normalize_moves(moves: &mut Vec<Move>) {
    moves.sort_by_key(|mv| (mv.row, mv.col));
    moves.dedup();
}

pub(super) fn push_unique_fact(facts: &mut Vec<LocalThreatFact>, fact: LocalThreatFact) {
    if !facts
        .iter()
        .any(|existing| same_shape_fact(existing, &fact))
    {
        facts.push(fact);
    }
}

pub(super) fn same_shape_fact(left: &LocalThreatFact, right: &LocalThreatFact) -> bool {
    left.player == right.player
        && left.kind == right.kind
        && left.defense_squares == right.defense_squares
        && left.rest_squares == right.rest_squares
}

pub(super) fn local_threat_fact_sort_key(
    fact: &LocalThreatFact,
) -> (u8, u8, usize, usize, String, String) {
    (
        fact.player as u8,
        local_threat_kind_sort_key(fact.kind),
        fact.origin.mv().row,
        fact.origin.mv().col,
        move_list_sort_key(&fact.defense_squares),
        move_list_sort_key(&fact.rest_squares),
    )
}

pub(super) fn local_threat_kind_sort_key(kind: LocalThreatKind) -> u8 {
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

pub(super) fn move_list_sort_key(moves: &[Move]) -> String {
    moves
        .iter()
        .map(|mv| format!("{:02}:{:02}", mv.row, mv.col))
        .collect::<Vec<_>>()
        .join("|")
}
