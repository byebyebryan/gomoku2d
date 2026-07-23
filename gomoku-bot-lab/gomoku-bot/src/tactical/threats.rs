use super::*;

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

pub fn local_threat_evidence_stones(board: &Board, fact: &LocalThreatFact) -> Vec<Move> {
    let mut evidence = match fact.origin {
        LocalThreatOrigin::AfterMove(mv) => {
            let view = BoardAfterMove {
                board,
                mv,
                player: fact.player,
            };
            local_threat_evidence_stones_view(&view, fact, true)
        }
        LocalThreatOrigin::Existing(mv) => {
            let view = BoardExistingMove {
                board,
                mv,
                player: fact.player,
            };
            local_threat_evidence_stones_view(&view, fact, false)
        }
    };
    normalize_moves(&mut evidence);
    evidence
}

pub fn compound_imminent_evidence_stones(
    board: &Board,
    attacker: Color,
    entries: &[OneStepLethalEntry],
) -> Vec<Move> {
    let entry_moves = entries.iter().map(|entry| entry.mv).collect::<Vec<_>>();
    let mut evidence = Vec::new();
    for fact in raw_local_threat_facts_for_player(board, attacker)
        .iter()
        .filter(|fact| {
            matches!(
                fact.kind,
                LocalThreatKind::OpenThree
                    | LocalThreatKind::ClosedThree
                    | LocalThreatKind::BrokenThree
            )
        })
        .filter(|fact| {
            materialization_squares_for_fact(board, fact)
                .iter()
                .any(|mv| entry_moves.contains(mv))
        })
    {
        for mv in local_threat_evidence_stones(board, fact) {
            push_unique_move(&mut evidence, mv);
        }
    }
    normalize_moves(&mut evidence);
    evidence
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
    raw_local_threat_facts_after_move_for_player(board, board.current_player, mv)
}

pub fn raw_local_threat_facts_after_move_for_player(
    board: &Board,
    player: Color,
    mv: Move,
) -> Vec<LocalThreatFact> {
    if !board.is_legal_for_color(mv, player) {
        return Vec::new();
    }

    normalize_local_threat_facts(local_threat_facts_after_legal_move_virtual_for_player(
        board, player, mv,
    ))
}

pub fn search_local_threat_facts_after_move(board: &Board, mv: Move) -> Vec<LocalThreatFact> {
    search_local_threat_facts_after_move_for_player(board, board.current_player, mv)
}

pub fn search_local_threat_facts_after_move_for_player(
    board: &Board,
    player: Color,
    mv: Move,
) -> Vec<LocalThreatFact> {
    if !board.is_legal_for_color(mv, player) {
        return Vec::new();
    }

    let facts = local_threat_facts_after_legal_move_virtual_for_player(board, player, mv);
    let facts = if board.config.variant == Variant::Renju && player == Color::Black {
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

pub fn defender_reply_candidates(
    board: &Board,
    attacker: Color,
    actual_reply: Option<Move>,
) -> Vec<DefenderReplyCandidate> {
    ScanThreatView::new(board).defender_reply_candidates(attacker, actual_reply)
}

pub fn defender_hint_reply_candidates(
    board: &Board,
    attacker: Color,
) -> Vec<DefenderReplyCandidate> {
    defender_hint_reply_candidates_from_view(board, &ScanThreatView::new(board), attacker)
}

pub fn corridor_attacker_move_rank(board: &Board, attacker: Color, mv: Move) -> u8 {
    CorridorThreatPolicy.attacker_move_rank(board, attacker, mv)
}

pub fn legal_forcing_continuations_for_fact(
    board: &Board,
    attacker: Color,
    fact: &LocalThreatFact,
) -> Vec<LocalThreatContinuation> {
    if !CorridorThreatPolicy.is_corridor_kind(fact.kind) {
        return Vec::new();
    }

    let mut continuations = Vec::new();
    for mv in forcing_continuation_squares(fact).iter().copied() {
        if let Some(continuation) = local_forcing_continuation(board, attacker, mv) {
            continuations.push(continuation);
        }
    }
    continuations
}

pub(super) fn local_forcing_continuation(
    board: &Board,
    attacker: Color,
    mv: Move,
) -> Option<LocalThreatContinuation> {
    let mut attacker_turn = board.clone();
    attacker_turn.current_player = attacker;
    if !attacker_turn.is_legal_for_color(mv, attacker) {
        return None;
    }

    let mut after_forcing = attacker_turn;
    match after_forcing.apply_trusted_legal_move(mv) {
        GameResult::Winner(winner) if winner == attacker => Some(LocalThreatContinuation {
            mv,
            legal_cost_squares: vec![mv],
        }),
        GameResult::Winner(_) | GameResult::Draw => None,
        GameResult::Ongoing => {
            let legal_cost_squares =
                local_immediate_winning_squares_after_continuation(&after_forcing, attacker, mv);
            (!legal_cost_squares.is_empty()).then_some(LocalThreatContinuation {
                mv,
                legal_cost_squares,
            })
        }
    }
}

pub(super) fn local_immediate_winning_squares_after_continuation(
    board_after_forcing: &Board,
    attacker: Color,
    mv: Move,
) -> Vec<Move> {
    let mut wins = Vec::new();
    for fact in raw_local_threat_facts_at_existing_move(board_after_forcing, attacker, mv) {
        if !matches!(
            fact.kind,
            LocalThreatKind::OpenFour | LocalThreatKind::ClosedFour | LocalThreatKind::BrokenFour
        ) {
            continue;
        }

        for completion in fact.defense_squares {
            if board_after_forcing.is_immediate_winning_move_for(completion, attacker) {
                push_unique_move(&mut wins, completion);
            }
        }
    }
    wins.sort_by_key(|mv| (mv.row, mv.col));
    wins
}

pub(super) fn forcing_continuation_squares(fact: &LocalThreatFact) -> &[Move] {
    if fact.rest_squares.is_empty() {
        &fact.defense_squares
    } else {
        &fact.rest_squares
    }
}

pub fn threat_obligation_from_facts(
    board: &Board,
    attacker: Color,
    mut immediate_targets: Vec<Move>,
    facts: impl IntoIterator<Item = LocalThreatFact>,
) -> Option<ThreatObligation> {
    let defender = attacker.opponent();
    if board.current_player != defender || board.result != GameResult::Ongoing {
        return None;
    }

    normalize_moves(&mut immediate_targets);
    if !immediate_targets.is_empty() {
        let legal_replies = legal_defender_replies(board, defender, &immediate_targets);
        return Some(ThreatObligation {
            attacker,
            defender,
            kind: ThreatObligationKind::Immediate,
            local_facts: Vec::new(),
            compound_entries: Vec::new(),
            candidate_replies: immediate_targets,
            legal_replies,
        });
    }

    let facts = normalize_local_threat_facts(
        facts
            .into_iter()
            .filter(|fact| fact.player == attacker)
            .collect(),
    );
    let policy = CorridorThreatPolicy;
    let local_facts = policy
        .active_threats_from_facts(board, attacker, facts.iter().cloned())
        .into_iter()
        .filter(|fact| {
            matches!(
                fact.kind,
                LocalThreatKind::OpenThree | LocalThreatKind::BrokenThree
            )
        })
        .collect::<Vec<_>>();
    let compound_entries = compound_imminent_entries_from_facts(board, attacker, facts.iter());

    let mut candidate_replies = Vec::new();
    for fact in &local_facts {
        add_corridor_defender_candidate_replies_for_fact(
            board,
            attacker,
            defender,
            fact,
            &mut candidate_replies,
        );
    }
    for mv in compound_imminent_candidate_replies_for_entries(board, attacker, &compound_entries) {
        push_unique_move(&mut candidate_replies, mv);
    }
    normalize_moves(&mut candidate_replies);
    if candidate_replies.is_empty() {
        return None;
    }

    let legal_replies = legal_defender_replies(board, defender, &candidate_replies);
    Some(ThreatObligation {
        attacker,
        defender,
        kind: ThreatObligationKind::Imminent,
        local_facts,
        compound_entries,
        candidate_replies,
        legal_replies,
    })
}

pub(super) fn legal_defender_replies(
    board: &Board,
    defender: Color,
    candidates: &[Move],
) -> Vec<Move> {
    let mut legal = candidates
        .iter()
        .copied()
        .filter(|&mv| board.is_legal_for_color(mv, defender))
        .collect::<Vec<_>>();
    normalize_moves(&mut legal);
    legal
}

pub(crate) fn defender_reply_candidates_from_view<V: ThreatView + ?Sized>(
    board: &Board,
    view: &V,
    attacker: Color,
    actual_reply: Option<Move>,
) -> Vec<DefenderReplyCandidate> {
    if board.current_player != attacker.opponent() || board.result != GameResult::Ongoing {
        return Vec::new();
    }

    let defender = attacker.opponent();
    let mut replies = Vec::<DefenderReplyCandidate>::new();

    if let Some(obligation) = view.threat_obligation(attacker) {
        for mv in view.immediate_winning_moves_for(defender) {
            push_reply_role(&mut replies, mv, DefenderReplyRole::OffensiveCounter);
        }

        let role = match obligation.kind {
            ThreatObligationKind::Immediate => DefenderReplyRole::ImmediateDefense,
            ThreatObligationKind::Imminent => DefenderReplyRole::ImminentDefense,
        };
        for mv in obligation.candidate_replies {
            push_reply_role(&mut replies, mv, role);
        }

        if obligation.kind == ThreatObligationKind::Imminent {
            for mv in offensive_counter_reply_moves(board, defender) {
                push_reply_role(&mut replies, mv, DefenderReplyRole::OffensiveCounter);
            }
        }
    }
    if let Some(mv) = actual_reply {
        push_reply_role(&mut replies, mv, DefenderReplyRole::Actual);
    }

    replies
}

pub fn defender_hint_reply_candidates_from_view<V: ThreatView + ?Sized>(
    board: &Board,
    view: &V,
    attacker: Color,
) -> Vec<DefenderReplyCandidate> {
    if board.current_player != attacker.opponent() || board.result != GameResult::Ongoing {
        return Vec::new();
    }

    let defender = attacker.opponent();
    let mut replies = Vec::<DefenderReplyCandidate>::new();

    if let Some(obligation) = view.threat_obligation(attacker) {
        let role = match obligation.kind {
            ThreatObligationKind::Immediate => DefenderReplyRole::ImmediateDefense,
            ThreatObligationKind::Imminent => DefenderReplyRole::ImminentDefense,
        };
        for mv in obligation.legal_replies {
            push_reply_role(&mut replies, mv, role);
        }
        if obligation.kind == ThreatObligationKind::Immediate {
            return replies;
        }
        for mv in offensive_counter_reply_moves(board, defender) {
            push_reply_role(&mut replies, mv, DefenderReplyRole::OffensiveCounter);
        }
    }

    replies
}

pub(super) fn offensive_counter_reply_moves(board: &Board, defender: Color) -> Vec<Move> {
    board
        .legal_moves()
        .into_iter()
        .filter(|&mv| {
            let mut next = board.clone();
            next.apply_move(mv).is_ok()
                && next.result == GameResult::Ongoing
                && !next.immediate_winning_moves_for(defender).is_empty()
        })
        .collect()
}

pub(super) fn compound_imminent_candidate_replies_for_entries(
    board: &Board,
    attacker: Color,
    entries: &[OneStepLethalEntry],
) -> Vec<Move> {
    let mut candidates = Vec::new();
    for entry in entries {
        push_unique_move(&mut candidates, entry.mv);
        for &target in &entry.terminal_targets {
            push_unique_move(&mut candidates, target);
        }
    }

    let mut replies = Vec::new();
    for mv in candidates {
        if !board.is_empty(mv.row, mv.col) {
            continue;
        }
        if defender_move_neutralizes_compound_imminent(board, attacker, mv) {
            push_unique_move(&mut replies, mv);
        }
    }
    normalize_moves(&mut replies);
    replies
}

pub(super) fn defender_move_neutralizes_compound_imminent(
    board: &Board,
    attacker: Color,
    mv: Move,
) -> bool {
    let defender = attacker.opponent();
    if !board.is_empty(mv.row, mv.col) {
        return false;
    }

    let mut after_reply = board.clone();
    after_reply.current_player = defender;
    match after_reply.apply_trusted_legal_move(mv) {
        GameResult::Winner(winner) => winner == defender,
        GameResult::Draw => true,
        GameResult::Ongoing => {
            after_reply.immediate_winning_moves_for(attacker).is_empty()
                && compound_imminent_entries(&after_reply, attacker).is_empty()
        }
    }
}

pub(super) fn compound_imminent_entries(board: &Board, attacker: Color) -> Vec<OneStepLethalEntry> {
    compound_imminent_entries_from_facts(
        board,
        attacker,
        raw_local_threat_facts_for_player(board, attacker).iter(),
    )
}

pub(super) fn compound_imminent_entries_from_facts<'a>(
    board: &Board,
    attacker: Color,
    facts: impl IntoIterator<Item = &'a LocalThreatFact>,
) -> Vec<OneStepLethalEntry> {
    #[cfg(not(target_arch = "wasm32"))]
    let start = std::time::Instant::now();

    let material = normalize_local_threat_facts(
        facts
            .into_iter()
            .filter(|fact| fact.player == attacker)
            .filter(|fact| {
                matches!(
                    fact.kind,
                    LocalThreatKind::OpenThree
                        | LocalThreatKind::ClosedThree
                        | LocalThreatKind::BrokenThree
                )
            })
            .cloned()
            .collect(),
    );
    if material.len() < 2 {
        #[cfg(not(target_arch = "wasm32"))]
        record_compound_imminent_query(start.elapsed(), 0, 0);
        #[cfg(target_arch = "wasm32")]
        record_compound_imminent_query(0, 0);
        return Vec::new();
    }

    let candidates = compound_imminent_entry_candidates(board, &material);

    let mut entries = Vec::new();
    for mv in candidates.iter().copied() {
        if let Some(entry) = compound_imminent_entry(board, attacker, mv) {
            entries.push(entry);
        }
    }
    entries.sort_by_key(|entry| (entry.mv.row, entry.mv.col));
    entries.dedup_by_key(|entry| (entry.mv.row, entry.mv.col));
    #[cfg(not(target_arch = "wasm32"))]
    record_compound_imminent_query(start.elapsed(), candidates.len(), entries.len());
    #[cfg(target_arch = "wasm32")]
    record_compound_imminent_query(candidates.len(), entries.len());
    entries
}

pub(super) fn compound_imminent_entry_candidates(
    board: &Board,
    material: &[LocalThreatFact],
) -> Vec<Move> {
    let mut candidates = Vec::new();
    for fact in material {
        let mut moves = materialization_squares_for_fact(board, fact)
            .into_iter()
            .filter(|mv| board.is_empty(mv.row, mv.col))
            .collect::<Vec<_>>();
        normalize_moves(&mut moves);
        for mv in moves {
            push_unique_move(&mut candidates, mv);
        }
    }
    normalize_moves(&mut candidates);
    candidates
}

pub(super) fn materialization_squares_for_fact(board: &Board, fact: &LocalThreatFact) -> Vec<Move> {
    let mut moves = forcing_continuation_squares(fact).to_vec();
    let origin = fact.origin_move();
    for &mv in forcing_continuation_squares(fact) {
        let dr = (mv.row as isize - origin.row as isize).signum();
        let dc = (mv.col as isize - origin.col as isize).signum();
        if dr == 0 && dc == 0 {
            continue;
        }
        let outer_row = mv.row as isize + dr;
        let outer_col = mv.col as isize + dc;
        if in_bounds(board, outer_row, outer_col)
            && board.is_empty(outer_row as usize, outer_col as usize)
        {
            push_unique_move(
                &mut moves,
                Move {
                    row: outer_row as usize,
                    col: outer_col as usize,
                },
            );
        }
    }
    normalize_moves(&mut moves);
    moves
}

pub(super) fn compound_imminent_entry(
    board: &Board,
    attacker: Color,
    mv: Move,
) -> Option<OneStepLethalEntry> {
    let mut attacker_turn = board.clone();
    attacker_turn.current_player = attacker;
    if !attacker_turn.is_legal_for_color(mv, attacker) {
        return None;
    }

    match attacker_turn.apply_trusted_legal_move(mv) {
        GameResult::Winner(winner) if winner == attacker => Some(OneStepLethalEntry {
            mv,
            terminal_targets: vec![mv],
        }),
        GameResult::Winner(_) | GameResult::Draw => None,
        GameResult::Ongoing => terminal_targets_if_lethal_after_entry(&attacker_turn, attacker)
            .map(|terminal_targets| OneStepLethalEntry {
                mv,
                terminal_targets,
            }),
    }
}

pub(super) fn terminal_targets_if_lethal_after_entry(
    board: &Board,
    attacker: Color,
) -> Option<Vec<Move>> {
    let defender = attacker.opponent();
    if board.result != GameResult::Ongoing || board.current_player != defender {
        return None;
    }

    let mut terminal_targets = board.immediate_winning_moves_for(attacker);
    normalize_moves(&mut terminal_targets);
    if terminal_targets.is_empty() {
        return None;
    }

    let mut defender_wins = board.immediate_winning_moves_for(defender);
    normalize_moves(&mut defender_wins);
    if !defender_wins.is_empty() {
        return None;
    }

    let legal_covers = terminal_targets
        .iter()
        .filter(|&&mv| board.is_legal_for_color(mv, defender))
        .count();
    if terminal_targets.len() >= 2 || legal_covers == 0 {
        Some(terminal_targets)
    } else {
        None
    }
}

pub(super) fn terminal_threat_covering_replies(board: &Board, attacker: Color) -> Vec<Move> {
    let defender = attacker.opponent();
    board
        .legal_moves()
        .into_iter()
        .filter(|&mv| {
            let mut next = board.clone();
            if next.apply_move(mv).is_err() {
                return false;
            }
            match next.result {
                GameResult::Winner(winner) if winner == defender => true,
                GameResult::Winner(_) => false,
                GameResult::Draw => true,
                GameResult::Ongoing => next.immediate_winning_moves_for(attacker).is_empty(),
            }
        })
        .collect()
}

pub(super) fn one_step_defender_reply_moves(
    board: &Board,
    attacker: Color,
    terminal: &TerminalLethalThreatAnalysis,
) -> Vec<Move> {
    let defender = attacker.opponent();
    let mut replies = if !terminal.terminal_targets.is_empty() {
        terminal.covering_replies.clone()
    } else {
        defender_reply_candidates(board, attacker, None)
            .into_iter()
            .map(|candidate| candidate.mv)
            .filter(|&mv| board.is_legal_for_color(mv, defender))
            .collect::<Vec<_>>()
    };
    normalize_moves(&mut replies);
    replies
}

pub(super) fn one_step_lethal_entries(
    board_after_defender_reply: &Board,
    attacker: Color,
) -> Vec<OneStepLethalEntry> {
    let mut attacker_turn = board_after_defender_reply.clone();
    attacker_turn.current_player = attacker;
    let mut entries = Vec::new();

    for mv in attacker_turn.legal_moves() {
        let mut after_entry = attacker_turn.clone();
        if after_entry.apply_move(mv).is_err() {
            continue;
        }

        let mut terminal_targets = match after_entry.result {
            GameResult::Winner(winner) if winner == attacker => vec![mv],
            GameResult::Winner(_) | GameResult::Draw => Vec::new(),
            GameResult::Ongoing => {
                let terminal = terminal_lethal_threat_analysis(&after_entry, attacker);
                if terminal.lethal_threat().is_some() {
                    terminal.terminal_targets
                } else {
                    Vec::new()
                }
            }
        };
        if terminal_targets.is_empty() {
            continue;
        }

        normalize_moves(&mut terminal_targets);
        entries.push(OneStepLethalEntry {
            mv,
            terminal_targets,
        });
    }

    entries.sort_by_key(|entry| (entry.mv.row, entry.mv.col));
    entries
}

pub(super) fn push_reply_role(
    replies: &mut Vec<DefenderReplyCandidate>,
    mv: Move,
    role: DefenderReplyRole,
) {
    if let Some(reply) = replies.iter_mut().find(|reply| reply.mv == mv) {
        if !reply.roles.contains(&role) {
            reply.roles.push(role);
        }
        return;
    }
    replies.push(DefenderReplyCandidate {
        mv,
        roles: vec![role],
    });
}

pub(super) fn add_corridor_defender_replies_for_fact(
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

pub(super) fn add_corridor_defender_candidate_replies_for_fact(
    board: &Board,
    attacker: Color,
    _defender: Color,
    fact: &LocalThreatFact,
    replies: &mut Vec<Move>,
) {
    let legal_forcing_continuations = legal_forcing_continuations_for_fact(board, attacker, fact);
    for continuation in &legal_forcing_continuations {
        let mv = continuation.mv;
        if board.is_empty(mv.row, mv.col) {
            push_unique_move(replies, mv);
        }
    }

    let mut shared_cost_squares: Option<Vec<Move>> = None;
    for continuation in legal_forcing_continuations {
        let costs = continuation
            .legal_cost_squares
            .into_iter()
            .filter(|&mv| board.is_empty(mv.row, mv.col))
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
