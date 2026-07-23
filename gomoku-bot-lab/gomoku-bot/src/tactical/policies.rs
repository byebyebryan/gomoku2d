use super::*;

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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TacticalOrderingSummary {
    pub score: i32,
    pub must_keep: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CorridorEntryRank {
    pub corridor_entry_rank: u8,
    pub latent_potential_rank: u8,
}

impl CorridorEntryRank {
    pub fn from_annotation(
        policy: CorridorThreatPolicy,
        annotation: &TacticalMoveAnnotation,
    ) -> Self {
        Self {
            corridor_entry_rank: policy.candidate_entry_rank(annotation),
            latent_potential_rank: 0,
        }
    }

    pub fn ordering_score(self) -> i32 {
        if self.corridor_entry_rank > 0 {
            60_000 + i32::from(self.corridor_entry_rank) * 1_000
        } else if self.latent_potential_rank > 0 {
            20_000 + i32::from(self.latent_potential_rank) * 100
        } else {
            0
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DefenderReplyRole {
    Actual,
    ImmediateDefense,
    ImminentDefense,
    OffensiveCounter,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefenderReplyCandidate {
    pub mv: Move,
    pub roles: Vec<DefenderReplyRole>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreatObligationKind {
    Immediate,
    Imminent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreatObligation {
    pub attacker: Color,
    pub defender: Color,
    pub kind: ThreatObligationKind,
    pub local_facts: Vec<LocalThreatFact>,
    pub compound_entries: Vec<OneStepLethalEntry>,
    /// Empty response squares relevant to this obligation, before defender-side
    /// legality filtering. Analysis uses these to show forbidden responses.
    pub candidate_replies: Vec<Move>,
    /// Legal defender responses usable by search/safety gates.
    pub legal_replies: Vec<Move>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LethalThreatKind {
    TerminalCoverage,
    OneStepCoverage,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OneStepLethalEntry {
    pub mv: Move,
    pub terminal_targets: Vec<Move>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OneStepDefenderReplyAnalysis {
    pub reply: Move,
    pub lethal_entries: Vec<OneStepLethalEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TerminalLethalThreatAnalysis {
    pub attacker: Color,
    pub defender: Color,
    pub terminal_targets: Vec<Move>,
    pub defender_immediate_wins: Vec<Move>,
    pub covering_replies: Vec<Move>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OneStepLethalThreatAnalysis {
    pub attacker: Color,
    pub defender: Color,
    pub terminal: TerminalLethalThreatAnalysis,
    pub defender_immediate_wins: Vec<Move>,
    pub defender_replies: Vec<OneStepDefenderReplyAnalysis>,
    pub escaping_replies: Vec<Move>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LethalThreat {
    pub attacker: Color,
    pub defender: Color,
    pub kind: LethalThreatKind,
    pub terminal_targets: Vec<Move>,
    pub covering_replies: Vec<Move>,
    pub one_step_replies: Vec<OneStepDefenderReplyAnalysis>,
}

impl TerminalLethalThreatAnalysis {
    pub fn lethal_threat(&self) -> Option<LethalThreat> {
        if self.terminal_targets.is_empty()
            || !self.defender_immediate_wins.is_empty()
            || !self.covering_replies.is_empty()
        {
            return None;
        }

        Some(LethalThreat {
            attacker: self.attacker,
            defender: self.defender,
            kind: LethalThreatKind::TerminalCoverage,
            terminal_targets: self.terminal_targets.clone(),
            covering_replies: self.covering_replies.clone(),
            one_step_replies: Vec::new(),
        })
    }
}

impl OneStepLethalThreatAnalysis {
    pub fn lethal_threat(&self) -> Option<LethalThreat> {
        if self.terminal.lethal_threat().is_some()
            || !self.defender_immediate_wins.is_empty()
            || self.defender_replies.is_empty()
            || !self.escaping_replies.is_empty()
            || self
                .defender_replies
                .iter()
                .any(|reply| reply.lethal_entries.is_empty())
        {
            return None;
        }

        Some(LethalThreat {
            attacker: self.attacker,
            defender: self.defender,
            kind: LethalThreatKind::OneStepCoverage,
            terminal_targets: self.terminal.terminal_targets.clone(),
            covering_replies: Vec::new(),
            one_step_replies: self.defender_replies.clone(),
        })
    }
}

impl TacticalOrderingSummary {
    fn include_fact(&mut self, policy: SearchThreatPolicy, fact: &LocalThreatFact) {
        self.score = self.score.max(policy.ordering_score(fact.kind));
        self.must_keep |= policy.is_must_keep(fact);
    }
}

impl SearchThreatPolicy {
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

    pub fn ordering_score(self, kind: LocalThreatKind) -> i32 {
        match kind {
            LocalThreatKind::Five => 100_000,
            LocalThreatKind::OpenFour => 80_000,
            LocalThreatKind::ClosedFour | LocalThreatKind::BrokenFour => 70_000,
            LocalThreatKind::OpenThree => 50_000,
            LocalThreatKind::BrokenThree => 40_000,
            LocalThreatKind::ClosedThree => 10_000,
        }
    }

    pub fn is_must_keep(self, fact: &LocalThreatFact) -> bool {
        !matches!(fact.kind, LocalThreatKind::ClosedThree)
    }

    pub fn facts_after_move(self, board: &Board, mv: Move) -> Vec<LocalThreatFact> {
        local_threat_facts_after_move(board, mv)
    }

    pub fn raw_annotation_for_player(
        self,
        board: &Board,
        player: Color,
        mv: Move,
    ) -> TacticalMoveAnnotation {
        if !board.is_legal_for_color(mv, player) {
            return TacticalMoveAnnotation {
                player,
                mv,
                local_threats: Vec::new(),
            };
        }

        TacticalMoveAnnotation {
            player,
            mv,
            local_threats: raw_local_threat_facts_after_move_for_player(board, player, mv)
                .into_iter()
                .filter(|fact| fact.player == player)
                .collect(),
        }
    }

    pub fn raw_annotation_for_legal_player(
        self,
        board: &Board,
        player: Color,
        mv: Move,
    ) -> TacticalMoveAnnotation {
        TacticalMoveAnnotation {
            player,
            mv,
            local_threats: normalize_local_threat_facts(
                local_threat_facts_after_legal_move_virtual_for_player(board, player, mv),
            )
            .into_iter()
            .filter(|fact| fact.player == player)
            .collect(),
        }
    }

    pub fn raw_ordering_summary_for_legal_player(
        self,
        board: &Board,
        player: Color,
        mv: Move,
    ) -> TacticalOrderingSummary {
        let after = BoardAfterMove { board, mv, player };
        let mut summary = TacticalOrderingSummary::default();
        for &(dr, dc) in &DIRS {
            if let Some(fact) = local_threat_fact_in_direction_view(&after, dr, dc) {
                if fact.player == player {
                    summary.include_fact(self, &fact);
                }
            }
        }
        summary
    }

    pub fn effective_annotation_from_raw(
        self,
        board: &Board,
        mut annotation: TacticalMoveAnnotation,
    ) -> TacticalMoveAnnotation {
        if !self.needs_renju_effective_filter(board, &annotation) {
            return annotation;
        }
        #[cfg(not(target_arch = "wasm32"))]
        let start = std::time::Instant::now();
        if !board.is_legal_for_color(annotation.mv, annotation.player) {
            annotation.local_threats.clear();
            #[cfg(not(target_arch = "wasm32"))]
            record_renju_effective_filter(start.elapsed());
            #[cfg(target_arch = "wasm32")]
            record_renju_effective_filter();
            return annotation;
        }

        annotation.local_threats = normalize_local_threat_facts(
            renju_effective_black_local_threat_facts_after_legal_move(
                board,
                annotation.mv,
                annotation.local_threats,
            ),
        );
        #[cfg(not(target_arch = "wasm32"))]
        record_renju_effective_filter(start.elapsed());
        #[cfg(target_arch = "wasm32")]
        record_renju_effective_filter();
        annotation
    }

    pub fn annotation_for_player(
        self,
        board: &Board,
        player: Color,
        mv: Move,
    ) -> TacticalMoveAnnotation {
        let annotation = self.raw_annotation_for_player(board, player, mv);
        self.effective_annotation_from_raw(board, annotation)
    }

    pub fn annotation_for_move(self, board: &Board, mv: Move) -> TacticalMoveAnnotation {
        self.annotation_for_player(board, board.current_player, mv)
    }

    pub fn ordering_summary(self, annotation: &TacticalMoveAnnotation) -> TacticalOrderingSummary {
        let mut summary = TacticalOrderingSummary::default();
        for fact in &annotation.local_threats {
            summary.include_fact(self, fact);
        }
        summary
    }

    pub fn effective_ordering_summary_from_raw(
        self,
        board: &Board,
        annotation: &TacticalMoveAnnotation,
    ) -> TacticalOrderingSummary {
        if !self.needs_renju_effective_filter(board, annotation) {
            return self.ordering_summary(annotation);
        }

        self.ordering_summary(&self.effective_annotation_from_raw(board, annotation.clone()))
    }

    pub fn ordering_summary_for_legal_player(
        self,
        board: &Board,
        player: Color,
        mv: Move,
    ) -> TacticalOrderingSummary {
        if board.config.variant != Variant::Renju || player != Color::Black {
            return self.raw_ordering_summary_for_legal_player(board, player, mv);
        }

        let annotation = self.raw_annotation_for_legal_player(board, player, mv);
        self.effective_ordering_summary_from_raw(board, &annotation)
    }

    pub fn corridor_entry_rank_for_player(
        self,
        board: &Board,
        player: Color,
        mv: Move,
    ) -> CorridorEntryRank {
        if !board.is_legal_for_color(mv, player) {
            return CorridorEntryRank::default();
        }
        self.corridor_entry_rank_for_legal_player(board, player, mv)
    }

    pub fn corridor_entry_rank_for_legal_player(
        self,
        board: &Board,
        player: Color,
        mv: Move,
    ) -> CorridorEntryRank {
        if board.config.variant == Variant::Renju && player == Color::Black {
            let annotation = self.raw_annotation_for_legal_player(board, player, mv);
            let annotation = self.effective_annotation_from_raw(board, annotation);
            return CorridorEntryRank::from_annotation(CorridorThreatPolicy, &annotation);
        }

        let policy = CorridorThreatPolicy;
        let after = BoardAfterMove { board, mv, player };
        let mut rank = 0;
        for &(dr, dc) in &DIRS {
            if let Some(fact) = local_threat_fact_in_direction_view(&after, dr, dc) {
                if fact.player == player && policy.is_corridor_kind(fact.kind) {
                    rank = rank.max(policy.rank(fact.kind));
                }
            }
        }
        CorridorEntryRank {
            corridor_entry_rank: rank,
            latent_potential_rank: 0,
        }
    }

    fn needs_renju_effective_filter(
        self,
        board: &Board,
        annotation: &TacticalMoveAnnotation,
    ) -> bool {
        board.config.variant == Variant::Renju
            && annotation.player == Color::Black
            && annotation
                .local_threats
                .iter()
                .any(|fact| self.is_must_keep(fact) && fact.kind != LocalThreatKind::Five)
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

    pub fn active_threats_from_facts(
        self,
        board: &Board,
        attacker: Color,
        facts: impl IntoIterator<Item = LocalThreatFact>,
    ) -> Vec<LocalThreatFact> {
        let mut facts = normalize_local_threat_facts(
            facts
                .into_iter()
                .filter(|fact| fact.player == attacker)
                .filter(|fact| self.is_active_threat(board, attacker, fact))
                .collect(),
        );
        facts.sort_by_key(|fact| std::cmp::Reverse(self.rank(fact.kind)));
        facts
    }

    pub fn has_active_threat(self, board: &Board, attacker: Color) -> bool {
        raw_local_threat_facts_for_player(board, attacker)
            .iter()
            .any(|fact| self.is_active_threat(board, attacker, fact))
    }

    pub fn is_visible_imminent_hint(
        self,
        board: &Board,
        attacker: Color,
        fact: &LocalThreatFact,
    ) -> bool {
        matches!(
            fact.kind,
            LocalThreatKind::OpenThree | LocalThreatKind::BrokenThree
        ) && self.is_active_threat(board, attacker, fact)
    }

    pub fn defender_reply_moves(
        self,
        board: &Board,
        attacker: Color,
        actual_reply: Option<Move>,
    ) -> Vec<Move> {
        let mut replies = threat_obligation_from_facts(
            board,
            attacker,
            board.immediate_winning_moves_for(attacker),
            raw_local_threat_facts_for_player(board, attacker),
        )
        .map(|obligation| obligation.legal_replies)
        .unwrap_or_default();
        if let Some(mv) = actual_reply {
            if board.is_legal_for_color(mv, attacker.opponent()) {
                push_unique_move(&mut replies, mv);
            }
        }
        normalize_moves(&mut replies);
        replies
    }

    pub fn defender_reply_moves_for_active_threats(
        self,
        board: &Board,
        attacker: Color,
        facts: Vec<LocalThreatFact>,
        _actual_reply: Option<Move>,
    ) -> Vec<Move> {
        let defender = attacker.opponent();
        let mut replies = Vec::new();

        if facts.is_empty() {
            return replies;
        }

        for fact in facts {
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

    pub fn candidate_entry_rank(self, annotation: &TacticalMoveAnnotation) -> u8 {
        annotation
            .local_threats
            .iter()
            .filter(|fact| fact.player == annotation.player)
            .filter(|fact| self.is_corridor_kind(fact.kind))
            .map(|fact| self.rank(fact.kind))
            .max()
            .unwrap_or(0)
    }
}

pub trait ThreatView {
    /// Legal immediate winning moves for `player` on this board.
    fn immediate_winning_moves_for(&self, player: Color) -> Vec<Move>;
    /// Search-ordering tactical annotation for a candidate before it is played.
    fn search_annotation_for_move(&self, mv: Move) -> TacticalMoveAnnotation;
    /// Search-ordering tactical annotation for an explicit side before it is played.
    fn search_annotation_for_player(&self, player: Color, mv: Move) -> TacticalMoveAnnotation;
    /// Rank for the local corridor entry a candidate would create before it is played.
    fn candidate_corridor_entry_rank(&self, attacker: Color, mv: Move) -> u8;
    /// Active immediate/imminent corridor threats for `attacker` on this board.
    fn active_corridor_threats(&self, attacker: Color) -> Vec<LocalThreatFact>;
    /// Highest-priority position-level obligation currently imposed by `attacker`.
    fn threat_obligation(&self, attacker: Color) -> Option<ThreatObligation>;
    /// True when `mv` is already occupied by `attacker` and that local move is
    /// itself part of an active corridor threat.
    fn has_move_local_corridor_entry(&self, attacker: Color, mv: Move) -> bool;
    /// Rank for an already-occupied attacker move that materialized a local corridor.
    fn local_corridor_entry_rank(&self, attacker: Color, mv: Move) -> u8;
    /// Legal defender replies to all active corridor threats.
    fn defender_reply_moves(&self, attacker: Color, actual_reply: Option<Move>) -> Vec<Move>;
    /// Legal defender replies annotated by why they matter.
    fn defender_reply_candidates(
        &self,
        attacker: Color,
        actual_reply: Option<Move>,
    ) -> Vec<DefenderReplyCandidate>;
}
