use super::*;

#[derive(Debug, Clone, Copy)]
pub struct ScanThreatView<'a> {
    board: &'a Board,
}

impl<'a> ScanThreatView<'a> {
    pub fn new(board: &'a Board) -> Self {
        Self { board }
    }
}

pub fn terminal_lethal_threat(board: &Board, attacker: Color) -> Option<LethalThreat> {
    terminal_lethal_threat_analysis(board, attacker).lethal_threat()
}

pub fn lethal_threat(board: &Board, attacker: Color) -> Option<LethalThreat> {
    let terminal = terminal_lethal_threat_analysis(board, attacker);
    terminal.lethal_threat().or_else(|| {
        one_step_lethal_threat_analysis_with_terminal(board, attacker, terminal).lethal_threat()
    })
}

pub fn one_step_lethal_threat(board: &Board, attacker: Color) -> Option<LethalThreat> {
    one_step_lethal_threat_analysis(board, attacker).lethal_threat()
}

pub fn terminal_lethal_threat_analysis(
    board: &Board,
    attacker: Color,
) -> TerminalLethalThreatAnalysis {
    let defender = attacker.opponent();
    let mut analysis = TerminalLethalThreatAnalysis {
        attacker,
        defender,
        terminal_targets: Vec::new(),
        defender_immediate_wins: Vec::new(),
        covering_replies: Vec::new(),
    };

    if board.result != GameResult::Ongoing || board.current_player != defender {
        return analysis;
    }

    analysis.terminal_targets = board.immediate_winning_moves_for(attacker);
    normalize_moves(&mut analysis.terminal_targets);
    if analysis.terminal_targets.is_empty() {
        return analysis;
    }

    analysis.defender_immediate_wins = board.immediate_winning_moves_for(defender);
    normalize_moves(&mut analysis.defender_immediate_wins);
    if !analysis.defender_immediate_wins.is_empty() {
        return analysis;
    }

    analysis.covering_replies = terminal_threat_covering_replies(board, attacker);
    normalize_moves(&mut analysis.covering_replies);
    analysis
}

pub fn one_step_lethal_threat_analysis(
    board: &Board,
    attacker: Color,
) -> OneStepLethalThreatAnalysis {
    let terminal = terminal_lethal_threat_analysis(board, attacker);
    one_step_lethal_threat_analysis_with_terminal(board, attacker, terminal)
}

pub(super) fn one_step_lethal_threat_analysis_with_terminal(
    board: &Board,
    attacker: Color,
    terminal: TerminalLethalThreatAnalysis,
) -> OneStepLethalThreatAnalysis {
    let defender = attacker.opponent();
    let mut analysis = OneStepLethalThreatAnalysis {
        attacker,
        defender,
        terminal,
        defender_immediate_wins: Vec::new(),
        defender_replies: Vec::new(),
        escaping_replies: Vec::new(),
    };

    if board.result != GameResult::Ongoing || board.current_player != defender {
        return analysis;
    }
    if analysis.terminal.lethal_threat().is_some() {
        return analysis;
    }

    analysis.defender_immediate_wins = board.immediate_winning_moves_for(defender);
    normalize_moves(&mut analysis.defender_immediate_wins);
    if !analysis.defender_immediate_wins.is_empty() {
        return analysis;
    }

    let replies = one_step_defender_reply_moves(board, attacker, &analysis.terminal);
    if replies.is_empty() {
        return analysis;
    }

    for reply in replies {
        let mut after_reply = board.clone();
        if after_reply.apply_move(reply).is_err() {
            continue;
        }

        let lethal_entries = match after_reply.result {
            GameResult::Winner(winner) if winner == defender => Vec::new(),
            GameResult::Winner(_) | GameResult::Draw => Vec::new(),
            GameResult::Ongoing => one_step_lethal_entries(&after_reply, attacker),
        };

        if lethal_entries.is_empty() {
            push_unique_move(&mut analysis.escaping_replies, reply);
        }
        analysis
            .defender_replies
            .push(OneStepDefenderReplyAnalysis {
                reply,
                lethal_entries,
            });
    }

    analysis
        .defender_replies
        .sort_by_key(|reply| (reply.reply.row, reply.reply.col));
    normalize_moves(&mut analysis.escaping_replies);
    analysis
}

impl ThreatView for ScanThreatView<'_> {
    fn immediate_winning_moves_for(&self, player: Color) -> Vec<Move> {
        self.board.immediate_winning_moves_for(player)
    }

    fn search_annotation_for_move(&self, mv: Move) -> TacticalMoveAnnotation {
        SearchThreatPolicy.annotation_for_move(self.board, mv)
    }

    fn search_annotation_for_player(&self, player: Color, mv: Move) -> TacticalMoveAnnotation {
        SearchThreatPolicy.annotation_for_player(self.board, player, mv)
    }

    fn candidate_corridor_entry_rank(&self, attacker: Color, mv: Move) -> u8 {
        SearchThreatPolicy
            .corridor_entry_rank_for_player(self.board, attacker, mv)
            .corridor_entry_rank
    }

    fn active_corridor_threats(&self, attacker: Color) -> Vec<LocalThreatFact> {
        CorridorThreatPolicy.active_threats(self.board, attacker)
    }

    fn threat_obligation(&self, attacker: Color) -> Option<ThreatObligation> {
        threat_obligation_from_facts(
            self.board,
            attacker,
            self.immediate_winning_moves_for(attacker),
            raw_local_threat_facts_for_player(self.board, attacker),
        )
    }

    fn has_move_local_corridor_entry(&self, attacker: Color, mv: Move) -> bool {
        self.local_corridor_entry_rank(attacker, mv) > 0
    }

    fn local_corridor_entry_rank(&self, attacker: Color, mv: Move) -> u8 {
        if !self.board.has_color(mv.row, mv.col, attacker) {
            return 0;
        }

        let policy = CorridorThreatPolicy;
        let existing = BoardExistingMove {
            board: self.board,
            mv,
            player: attacker,
        };
        DIRS.iter()
            .filter_map(|&(dr, dc)| local_threat_fact_in_direction_view(&existing, dr, dc))
            .filter(|fact| policy.is_active_threat(self.board, attacker, fact))
            .map(|fact| policy.rank(fact.kind))
            .max()
            .unwrap_or(0)
    }

    fn defender_reply_moves(&self, attacker: Color, actual_reply: Option<Move>) -> Vec<Move> {
        CorridorThreatPolicy.defender_reply_moves(self.board, attacker, actual_reply)
    }

    fn defender_reply_candidates(
        &self,
        attacker: Color,
        actual_reply: Option<Move>,
    ) -> Vec<DefenderReplyCandidate> {
        defender_reply_candidates_from_view(self.board, self, attacker, actual_reply)
    }
}
