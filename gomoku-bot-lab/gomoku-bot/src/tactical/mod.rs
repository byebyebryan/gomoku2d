use serde::Serialize;
use std::cell::Cell as MetricCell;

use gomoku_core::{Board, Color, GameResult, Move, Variant, DIRS};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TacticalMetrics {
    pub renju_effective_filter_calls: u64,
    pub renju_effective_filter_ns: u64,
    pub renju_effective_filter_continuation_checks: u64,
    pub renju_effective_filter_continuation_ns: u64,
    pub compound_imminent_queries: u64,
    pub compound_imminent_ns: u64,
    pub compound_imminent_prefilter_candidates: u64,
    pub compound_imminent_confirmed_entries: u64,
    pub compound_imminent_hits: u64,
}

thread_local! {
    static TACTICAL_METRICS: MetricCell<TacticalMetrics> = const { MetricCell::new(TacticalMetrics {
        renju_effective_filter_calls: 0,
        renju_effective_filter_ns: 0,
        renju_effective_filter_continuation_checks: 0,
        renju_effective_filter_continuation_ns: 0,
        compound_imminent_queries: 0,
        compound_imminent_ns: 0,
        compound_imminent_prefilter_candidates: 0,
        compound_imminent_confirmed_entries: 0,
        compound_imminent_hits: 0,
    }) };
}

pub fn tactical_metrics_snapshot() -> TacticalMetrics {
    TACTICAL_METRICS.with(MetricCell::get)
}

#[cfg(not(target_arch = "wasm32"))]
fn record_renju_effective_filter(elapsed: std::time::Duration) {
    TACTICAL_METRICS.with(|metrics| {
        let mut current = metrics.get();
        current.renju_effective_filter_calls =
            current.renju_effective_filter_calls.saturating_add(1);
        current.renju_effective_filter_ns = current
            .renju_effective_filter_ns
            .saturating_add(u64::try_from(elapsed.as_nanos()).unwrap_or(u64::MAX).max(1));
        metrics.set(current);
    });
}

#[cfg(target_arch = "wasm32")]
fn record_renju_effective_filter() {
    TACTICAL_METRICS.with(|metrics| {
        let mut current = metrics.get();
        current.renju_effective_filter_calls =
            current.renju_effective_filter_calls.saturating_add(1);
        metrics.set(current);
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn record_renju_effective_filter_continuation(elapsed: std::time::Duration) {
    TACTICAL_METRICS.with(|metrics| {
        let mut current = metrics.get();
        current.renju_effective_filter_continuation_checks = current
            .renju_effective_filter_continuation_checks
            .saturating_add(1);
        current.renju_effective_filter_continuation_ns = current
            .renju_effective_filter_continuation_ns
            .saturating_add(u64::try_from(elapsed.as_nanos()).unwrap_or(u64::MAX).max(1));
        metrics.set(current);
    });
}

#[cfg(target_arch = "wasm32")]
fn record_renju_effective_filter_continuation() {
    TACTICAL_METRICS.with(|metrics| {
        let mut current = metrics.get();
        current.renju_effective_filter_continuation_checks = current
            .renju_effective_filter_continuation_checks
            .saturating_add(1);
        metrics.set(current);
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn record_compound_imminent_query(
    elapsed: std::time::Duration,
    prefilter_candidates: usize,
    confirmed_entries: usize,
) {
    TACTICAL_METRICS.with(|metrics| {
        let mut current = metrics.get();
        current.compound_imminent_queries = current.compound_imminent_queries.saturating_add(1);
        current.compound_imminent_ns = current
            .compound_imminent_ns
            .saturating_add(u64::try_from(elapsed.as_nanos()).unwrap_or(u64::MAX).max(1));
        current.compound_imminent_prefilter_candidates = current
            .compound_imminent_prefilter_candidates
            .saturating_add(prefilter_candidates as u64);
        current.compound_imminent_confirmed_entries = current
            .compound_imminent_confirmed_entries
            .saturating_add(confirmed_entries as u64);
        if confirmed_entries > 0 {
            current.compound_imminent_hits = current.compound_imminent_hits.saturating_add(1);
        }
        metrics.set(current);
    });
}

#[cfg(target_arch = "wasm32")]
fn record_compound_imminent_query(prefilter_candidates: usize, confirmed_entries: usize) {
    TACTICAL_METRICS.with(|metrics| {
        let mut current = metrics.get();
        current.compound_imminent_queries = current.compound_imminent_queries.saturating_add(1);
        current.compound_imminent_prefilter_candidates = current
            .compound_imminent_prefilter_candidates
            .saturating_add(prefilter_candidates as u64);
        current.compound_imminent_confirmed_entries = current
            .compound_imminent_confirmed_entries
            .saturating_add(confirmed_entries as u64);
        if confirmed_entries > 0 {
            current.compound_imminent_hits = current.compound_imminent_hits.saturating_add(1);
        }
        metrics.set(current);
    });
}

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

fn one_step_lethal_threat_analysis_with_terminal(
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

fn local_forcing_continuation(
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

fn local_immediate_winning_squares_after_continuation(
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

fn forcing_continuation_squares(fact: &LocalThreatFact) -> &[Move] {
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

fn legal_defender_replies(board: &Board, defender: Color, candidates: &[Move]) -> Vec<Move> {
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

fn offensive_counter_reply_moves(board: &Board, defender: Color) -> Vec<Move> {
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

fn compound_imminent_candidate_replies_for_entries(
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

fn defender_move_neutralizes_compound_imminent(board: &Board, attacker: Color, mv: Move) -> bool {
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

fn compound_imminent_entries(board: &Board, attacker: Color) -> Vec<OneStepLethalEntry> {
    compound_imminent_entries_from_facts(
        board,
        attacker,
        raw_local_threat_facts_for_player(board, attacker).iter(),
    )
}

fn compound_imminent_entries_from_facts<'a>(
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

fn compound_imminent_entry_candidates(board: &Board, material: &[LocalThreatFact]) -> Vec<Move> {
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

fn materialization_squares_for_fact(board: &Board, fact: &LocalThreatFact) -> Vec<Move> {
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

fn compound_imminent_entry(board: &Board, attacker: Color, mv: Move) -> Option<OneStepLethalEntry> {
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

fn terminal_targets_if_lethal_after_entry(board: &Board, attacker: Color) -> Option<Vec<Move>> {
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

fn terminal_threat_covering_replies(board: &Board, attacker: Color) -> Vec<Move> {
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

fn one_step_defender_reply_moves(
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

fn one_step_lethal_entries(
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

fn push_reply_role(replies: &mut Vec<DefenderReplyCandidate>, mv: Move, role: DefenderReplyRole) {
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

fn add_corridor_defender_candidate_replies_for_fact(
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

fn local_threat_facts_after_legal_move_virtual_for_player(
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

fn renju_effective_black_local_threat_facts_after_legal_move(
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

fn renju_black_local_threat_continuation_is_effective(board_after_gain: &Board, mv: Move) -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    let start = std::time::Instant::now();
    let result = renju_black_local_threat_continuation_is_effective_inner(board_after_gain, mv);
    #[cfg(not(target_arch = "wasm32"))]
    record_renju_effective_filter_continuation(start.elapsed());
    #[cfg(target_arch = "wasm32")]
    record_renju_effective_filter_continuation();
    result
}

fn renju_black_local_threat_continuation_is_effective_inner(
    board_after_gain: &Board,
    mv: Move,
) -> bool {
    local_forcing_continuation(board_after_gain, Color::Black, mv).is_some()
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

#[derive(Debug, Default)]
struct BrokenThreeSquares {
    defense_squares: Vec<Move>,
    rest_squares: Vec<Move>,
}

fn broken_three_squares_through_view(
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

fn local_threat_evidence_stones_view(
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

fn five_evidence_stones_view(view: &impl TacticalBoardView, exclude_origin: bool) -> Vec<Move> {
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

fn completion_window_evidence_stones_view(
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

fn span_evidence_stones_view(
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

fn evidence_line_span(
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

fn line_offset(origin: Move, point: Move, dr: isize, dc: isize) -> Option<isize> {
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

fn player_at_offset(view: &impl TacticalBoardView, dr: isize, dc: isize, offset: isize) -> bool {
    let origin = view.mv();
    let row = origin.row as isize + dr * offset;
    let col = origin.col as isize + dc * offset;
    view.in_bounds(row, col) && view.has_color(row as usize, col as usize, view.player())
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
        corridor_active_threats, corridor_defender_reply_moves, defender_hint_reply_candidates,
        defender_reply_candidates, has_forcing_local_threat, has_forcing_local_threat_at_move,
        legal_forcing_continuations_for_fact, lethal_threat, local_threat_evidence_stones,
        local_threat_facts_after_move, local_threat_facts_for_player, normalize_local_threat_facts,
        one_step_lethal_threat, one_step_lethal_threat_analysis, raw_local_threat_facts_after_move,
        raw_local_threat_facts_for_player, terminal_lethal_threat, terminal_lethal_threat_analysis,
        CorridorThreatPolicy, DefenderReplyCandidate, DefenderReplyRole, LethalThreatKind,
        LocalThreatFact, LocalThreatKind, LocalThreatOrigin, ScanThreatView, SearchThreatPolicy,
        ThreatObligationKind, ThreatView,
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

    fn notation_list(moves: &[Move]) -> Vec<String> {
        moves.iter().map(|mv| mv.to_notation()).collect()
    }

    fn one_step_reply_entries(
        analysis: &super::OneStepLethalThreatAnalysis,
    ) -> Vec<(String, Vec<String>)> {
        analysis
            .defender_replies
            .iter()
            .map(|reply| {
                (
                    reply.reply.to_notation(),
                    reply
                        .lethal_entries
                        .iter()
                        .map(|entry| entry.mv.to_notation())
                        .collect(),
                )
            })
            .collect()
    }

    fn has_reply_role(
        candidates: &[DefenderReplyCandidate],
        notation: &str,
        role: DefenderReplyRole,
    ) -> bool {
        let mv = mv(notation);
        candidates
            .iter()
            .any(|candidate| candidate.mv == mv && candidate.roles.contains(&role))
    }

    #[test]
    fn terminal_lethal_threat_detects_open_four_coverage() {
        let board = board_from_moves(
            Variant::Freestyle,
            &["H8", "A1", "I8", "A2", "J8", "A3", "K8"],
        );
        assert_eq!(board.current_player, Color::White);

        let analysis = terminal_lethal_threat_analysis(&board, Color::Black);

        assert_eq!(notation_list(&analysis.terminal_targets), vec!["G8", "L8"]);
        assert!(analysis.defender_immediate_wins.is_empty());
        assert!(analysis.covering_replies.is_empty());
        assert_eq!(
            terminal_lethal_threat(&board, Color::Black)
                .expect("open four should be terminal lethal")
                .terminal_targets,
            vec![mv("G8"), mv("L8")]
        );
    }

    #[test]
    fn terminal_lethal_threat_rejects_single_blockable_four() {
        let board = board_from_moves(
            Variant::Freestyle,
            &["H8", "G8", "I8", "A1", "J8", "A2", "K8"],
        );
        assert_eq!(board.current_player, Color::White);

        let analysis = terminal_lethal_threat_analysis(&board, Color::Black);

        assert_eq!(notation_list(&analysis.terminal_targets), vec!["L8"]);
        assert_eq!(notation_list(&analysis.covering_replies), vec!["L8"]);
        assert!(terminal_lethal_threat(&board, Color::Black).is_none());
    }

    #[test]
    fn terminal_lethal_threat_rejects_when_defender_can_win_now() {
        let board = board_from_moves(
            Variant::Freestyle,
            &["B1", "H8", "B2", "I8", "B3", "J8", "B4", "K8"],
        );
        assert_eq!(board.current_player, Color::Black);

        let analysis = terminal_lethal_threat_analysis(&board, Color::White);

        assert_eq!(notation_list(&analysis.terminal_targets), vec!["G8", "L8"]);
        assert_eq!(notation_list(&analysis.defender_immediate_wins), vec!["B5"]);
        assert!(analysis.covering_replies.is_empty());
        assert!(terminal_lethal_threat(&board, Color::White).is_none());
    }

    #[test]
    fn terminal_lethal_threat_uses_renju_forbidden_replies_as_missing_coverage() {
        let board = board_from_moves(
            Variant::Renju,
            &[
                "H8", "H7", "J8", "I9", "I8", "G8", "F9", "F7", "H9", "G7", "I7", "E7", "D7", "G9",
                "G6", "G11",
            ],
        );
        assert_eq!(board.current_player, Color::Black);
        assert!(!board.is_legal_for_color(mv("G10"), Color::Black));

        let analysis = terminal_lethal_threat_analysis(&board, Color::White);

        assert_eq!(notation_list(&analysis.terminal_targets), vec!["G10"]);
        assert!(analysis.defender_immediate_wins.is_empty());
        assert!(
            analysis.covering_replies.is_empty(),
            "forbidden direct blocks should not count as legal coverage: {:?}",
            analysis.covering_replies
        );
        assert!(terminal_lethal_threat(&board, Color::White).is_some());
    }

    #[test]
    fn one_step_lethal_threat_detects_four_three_coverage() {
        let board = board_from_moves(
            Variant::Freestyle,
            &[
                "H8", "G8", "I8", "A1", "J8", "O1", "K8", "A15", "I7", "O15", "I9",
            ],
        );
        assert_eq!(board.current_player, Color::White);

        let analysis = one_step_lethal_threat_analysis(&board, Color::Black);

        assert_eq!(
            notation_list(&analysis.terminal.terminal_targets),
            vec!["L8"]
        );
        assert_eq!(
            notation_list(&analysis.terminal.covering_replies),
            vec!["L8"]
        );
        assert!(
            analysis.escaping_replies.is_empty(),
            "unexpected escapes: {:?}; replies: {:?}",
            notation_list(&analysis.escaping_replies),
            one_step_reply_entries(&analysis)
        );
        assert_eq!(
            one_step_reply_entries(&analysis),
            vec![("L8".to_string(), vec!["I6".to_string(), "I10".to_string()])]
        );
        assert_eq!(
            one_step_lethal_threat(&board, Color::Black)
                .expect("4+3 should be one-step lethal")
                .kind,
            LethalThreatKind::OneStepCoverage
        );
        assert_eq!(
            lethal_threat(&board, Color::Black)
                .expect("general classifier should find 4+3 lethal")
                .kind,
            LethalThreatKind::OneStepCoverage
        );
    }

    #[test]
    fn one_step_lethal_threat_detects_double_three_coverage() {
        let board = board_from_moves(
            Variant::Freestyle,
            &["H8", "A1", "I8", "O1", "J8", "A15", "I7", "O15", "I9"],
        );
        assert_eq!(board.current_player, Color::White);

        let analysis = one_step_lethal_threat_analysis(&board, Color::Black);

        assert!(analysis.terminal.terminal_targets.is_empty());
        assert!(
            analysis.escaping_replies.is_empty(),
            "unexpected escapes: {:?}; replies: {:?}",
            notation_list(&analysis.escaping_replies),
            one_step_reply_entries(&analysis)
        );
        assert_eq!(
            one_step_reply_entries(&analysis),
            vec![
                ("I6".to_string(), vec!["G8".to_string(), "K8".to_string()]),
                ("G8".to_string(), vec!["I6".to_string(), "I10".to_string()]),
                ("K8".to_string(), vec!["I6".to_string(), "I10".to_string()]),
                ("I10".to_string(), vec!["G8".to_string(), "K8".to_string()]),
            ]
        );
        assert_eq!(
            one_step_lethal_threat(&board, Color::Black)
                .expect("3+3 should be one-step lethal")
                .kind,
            LethalThreatKind::OneStepCoverage
        );
    }

    #[test]
    fn one_step_lethal_threat_rejects_open_cross_shared_block() {
        let board = board_from_moves(
            Variant::Freestyle,
            &[
                "G8", "A1", "H8", "O1", "J8", "A15", "I6", "O15", "I7", "C3", "I9",
            ],
        );
        assert_eq!(board.current_player, Color::White);

        let analysis = one_step_lethal_threat_analysis(&board, Color::Black);

        assert!(
            notation_list(&analysis.escaping_replies).contains(&"I8".to_string()),
            "the open crossing point should be a shared escape: {:?}; replies: {:?}",
            notation_list(&analysis.escaping_replies),
            one_step_reply_entries(&analysis)
        );
        assert!(one_step_lethal_threat(&board, Color::Black).is_none());
        assert!(lethal_threat(&board, Color::Black).is_none());
    }

    #[test]
    fn one_step_lethal_threat_rejects_single_open_three() {
        let board = board_from_moves(Variant::Freestyle, &["H8", "A1", "I8", "C1", "J8"]);
        assert_eq!(board.current_player, Color::White);

        let analysis = one_step_lethal_threat_analysis(&board, Color::Black);

        assert_eq!(notation_list(&analysis.escaping_replies), vec!["G8", "K8"]);
        assert!(one_step_lethal_threat(&board, Color::Black).is_none());
        assert!(lethal_threat(&board, Color::Black).is_none());
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

    fn assert_no_raw_broken_three_after_move(before_moves: &[&str], gain: &str) {
        let mut before = Board::new(RuleConfig::default());
        apply_moves(&mut before, before_moves);

        let facts = raw_local_threat_facts_after_move(&before, mv(gain));
        assert!(
            facts
                .iter()
                .all(|fact| fact.kind != LocalThreatKind::BrokenThree),
            "shape should not be a forcing broken three: {facts:?}"
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
        apply_moves(&mut broken_three_board, &["H8", "A1", "I8", "C1"]);
        assert_eq!(
            local_threat_facts_after_move(&broken_three_board, mv("K8")),
            vec![LocalThreatFact {
                player: Color::Black,
                kind: LocalThreatKind::BrokenThree,
                origin: LocalThreatOrigin::AfterMove(mv("K8")),
                defense_squares: vec![mv("G8"), mv("J8"), mv("L8")],
                rest_squares: vec![mv("J8")],
            }]
        );
    }

    #[test]
    fn fixed_window_broken_threes_are_not_forcing() {
        assert_no_raw_broken_three_after_move(&["H8", "A1", "J8", "C1"], "L8"); // X_X_X
        assert_no_raw_broken_three_after_move(&["H8", "A1", "I8", "C1"], "L8"); // XX__X
        assert_no_raw_broken_three_after_move(&["H8", "A1", "K8", "C1"], "L8"); // X__XX
    }

    #[test]
    fn one_side_blocked_sliding_broken_threes_are_not_forcing() {
        assert_no_raw_broken_three_after_move(&["H8", "G8", "I8", "A1"], "K8"); // OXX_X_
        assert_no_raw_broken_three_after_move(&["H8", "L8", "I8", "A1"], "K8"); // _XX_XO
        assert_no_raw_broken_three_after_move(&["H8", "G8", "J8", "A1"], "K8"); // OX_XX_
        assert_no_raw_broken_three_after_move(&["H8", "L8", "J8", "A1"], "K8"); // _X_XXO
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

        let split_three = board_from_moves(Variant::Freestyle, &["H8", "A1", "I8", "C1", "K8"]);
        assert!(
            local_threat_facts_for_player(&split_three, Color::Black).contains(&LocalThreatFact {
                player: Color::Black,
                kind: LocalThreatKind::BrokenThree,
                origin: LocalThreatOrigin::Existing(mv("H8")),
                defense_squares: vec![mv("G8"), mv("J8"), mv("L8")],
                rest_squares: vec![mv("J8")],
            })
        );
    }

    #[test]
    fn local_threat_evidence_stones_identify_existing_open_three_shape() {
        let board = board_from_moves(Variant::Freestyle, &["H8", "A1", "I8", "A2", "J8"]);
        let fact = fact(
            Color::Black,
            LocalThreatKind::OpenThree,
            "H8",
            &["G8", "K8"],
            &[],
        );

        assert_eq!(
            notation_list(&local_threat_evidence_stones(&board, &fact)),
            vec!["H8", "I8", "J8"],
        );
    }

    #[test]
    fn local_threat_evidence_stones_exclude_virtual_candidate_move() {
        let board = board_from_moves(
            Variant::Freestyle,
            &["H8", "A1", "I8", "A2", "J8", "A3", "K8"],
        );
        let annotation = SearchThreatPolicy.annotation_for_player(&board, Color::Black, mv("L8"));
        let five = annotation
            .local_threats
            .iter()
            .find(|fact| fact.kind == LocalThreatKind::Five)
            .expect("candidate should complete a five");

        assert_eq!(
            notation_list(&local_threat_evidence_stones(&board, five)),
            vec!["H8", "I8", "J8", "K8"],
        );
    }

    #[test]
    fn closed_three_endpoint_is_not_a_corridor_reply() {
        let left_blocked = board_from_moves(Variant::Renju, &["G8", "H8", "A1", "I8", "A2", "J8"]);
        assert_eq!(left_blocked.current_player, Color::Black);

        let replies = CorridorThreatPolicy.defender_reply_moves(&left_blocked, Color::White, None);
        assert!(
            !replies.contains(&mv("K8")),
            "the open endpoint of OXXX_ is a closed-three extension, not a forced reply: {replies:?}"
        );

        let right_blocked = board_from_moves(Variant::Renju, &["K8", "H8", "A1", "I8", "A2", "J8"]);
        assert_eq!(right_blocked.current_player, Color::Black);

        let replies = CorridorThreatPolicy.defender_reply_moves(&right_blocked, Color::White, None);
        assert!(
            !replies.contains(&mv("G8")),
            "the open endpoint of _XXXO is a closed-three extension, not a forced reply: {replies:?}"
        );
    }

    #[test]
    fn defender_hint_candidates_require_imminent_threat_for_counter() {
        let board = board_from_moves(Variant::Freestyle, &["H8", "A1", "I8", "O1", "J8", "A15"]);
        assert_eq!(board.current_player, Color::Black);

        let candidates = defender_hint_reply_candidates(&board, Color::White);

        assert!(
            candidates.iter().all(|candidate| !candidate
                .roles
                .contains(&DefenderReplyRole::OffensiveCounter)),
            "quiet positions should not expose offensive counters as UI hints: {candidates:?}"
        );
    }

    #[test]
    fn defender_reply_candidates_require_imminent_threat_for_counter() {
        let board = board_from_moves(Variant::Freestyle, &["H8", "A1", "I8", "O1", "J8", "A15"]);
        assert_eq!(board.current_player, Color::Black);

        let candidates = defender_reply_candidates(&board, Color::White, None);

        assert!(
            candidates.iter().all(|candidate| !candidate
                .roles
                .contains(&DefenderReplyRole::OffensiveCounter)),
            "quiet positions should not expose offensive counters as corridor replies: {candidates:?}"
        );
    }

    #[test]
    fn defender_hint_candidates_prioritize_immediate_replies_over_imminent_replies() {
        let board = board_from_moves(
            Variant::Freestyle,
            &[
                "A1", "H8", "C2", "I8", "E3", "J8", "G4", "K8", "I5", "F6", "K6", "G6", "M7", "H6",
            ],
        );
        assert_eq!(board.current_player, Color::Black);

        let candidates = defender_hint_reply_candidates(&board, Color::White);

        assert!(has_reply_role(
            &candidates,
            "G8",
            DefenderReplyRole::ImmediateDefense
        ));
        assert!(has_reply_role(
            &candidates,
            "L8",
            DefenderReplyRole::ImmediateDefense
        ));
        assert!(
            candidates.iter().all(|candidate| !candidate
                .roles
                .contains(&DefenderReplyRole::ImminentDefense)),
            "imminent replies should be suppressed while immediate replies exist: {candidates:?}"
        );
    }

    #[test]
    fn defender_reply_candidates_prioritize_immediate_replies_over_imminent_replies() {
        let board = board_from_moves(
            Variant::Freestyle,
            &[
                "A1", "H8", "C2", "I8", "E3", "J8", "G4", "K8", "I5", "F6", "K6", "G6", "M7", "H6",
            ],
        );
        assert_eq!(board.current_player, Color::Black);

        let candidates = defender_reply_candidates(&board, Color::White, None);

        assert!(has_reply_role(
            &candidates,
            "G8",
            DefenderReplyRole::ImmediateDefense
        ));
        assert!(has_reply_role(
            &candidates,
            "L8",
            DefenderReplyRole::ImmediateDefense
        ));
        assert!(
            candidates.iter().all(|candidate| !candidate
                .roles
                .contains(&DefenderReplyRole::ImminentDefense)),
            "4+3 combos should probe the immediate 4 blocks, not lower-priority 3 replies: {candidates:?}"
        );
    }

    #[test]
    fn defender_reply_candidates_cover_all_imminent_combo_threats() {
        let board = board_from_moves(
            Variant::Renju,
            &[
                "H8", "H7", "F8", "G9", "G8", "I8", "G6", "D9", "F9", "F10", "D7", "G10", "F7",
                "E10", "E8", "D8", "C6", "B5", "D10", "F11", "F6", "F5", "D6", "E6", "H5", "I4",
            ],
        );
        assert_eq!(board.current_player, Color::Black);

        let candidates = defender_reply_candidates(&board, Color::White, Some(mv("C8")));

        for notation in ["J7", "H9", "E12", "G12"] {
            assert!(
                has_reply_role(&candidates, notation, DefenderReplyRole::ImminentDefense),
                "{notation} should be probed as a response to the 3+3 corridor: {candidates:?}"
            );
        }
        assert!(has_reply_role(
            &candidates,
            "C8",
            DefenderReplyRole::ImminentDefense
        ));
        assert!(has_reply_role(&candidates, "C8", DefenderReplyRole::Actual));
    }

    #[test]
    fn defender_reply_candidates_cover_combo_from_individually_nonforcing_threes() {
        let board = board_from_moves(
            Variant::Renju,
            &[
                "H8", "I9", "G8", "H6", "F8", "I8", "I7", "G9", "H9", "E6", "I10", "J11", "H10",
                "H11", "G10", "F10", "E8", "D8", "F11", "E9", "G11", "F6", "G6", "E7", "G5", "D6",
                "E12", "D13", "G4", "G7", "G3", "G2",
            ],
        );
        assert_eq!(board.current_player, Color::Black);

        let obligation = ScanThreatView::new(&board)
            .threat_obligation(Color::White)
            .expect("compound imminent threat should produce a position obligation");
        assert_eq!(obligation.kind, ThreatObligationKind::Imminent);
        assert!(
            obligation
                .compound_entries
                .iter()
                .any(|entry| entry.mv == mv("B6")),
            "B6 should be recognized as a one-step entry into lethal coverage: {obligation:?}"
        );

        let candidates = defender_reply_candidates(&board, Color::White, Some(mv("B6")));

        assert!(
            has_reply_role(&candidates, "B6", DefenderReplyRole::ImminentDefense),
            "B6 should be visible as a defensive reply to the combined imminent threat: {candidates:?}"
        );
        assert!(has_reply_role(&candidates, "B6", DefenderReplyRole::Actual));
        for notation in ["J10", "K10"] {
            assert!(
                has_reply_role(&candidates, notation, DefenderReplyRole::OffensiveCounter),
                "{notation} should remain visible as counter-threat escape: {candidates:?}"
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
            &["H8", "A1", "I8", "C1"],
            "K8",
            Color::Black,
            LocalThreatKind::BrokenThree,
            &["G8", "J8", "L8"],
            &["J8"],
        );
        assert_raw_fact_parity(
            &["I8", "A1", "K8", "C1"],
            "L8",
            Color::Black,
            LocalThreatKind::BrokenThree,
            &["H8", "J8", "M8"],
            &["J8"],
        );
    }

    #[test]
    fn search_and_corridor_policies_treat_valid_broken_three_as_forcing() {
        let mut board = Board::new(RuleConfig::default());
        apply_moves(&mut board, &["H8", "A1", "K8", "C1"]);

        let annotation = SearchThreatPolicy.annotation_for_move(&board, mv("J8"));
        let broken_three = annotation
            .local_threats
            .iter()
            .find(|fact| fact.kind == LocalThreatKind::BrokenThree)
            .expect("search policy should retain broken-three material");
        assert!(SearchThreatPolicy.is_must_keep(broken_three));

        let mut existing = board.clone();
        existing.apply_move(mv("J8")).unwrap();
        let corridor_fact = raw_local_threat_facts_for_player(&existing, Color::Black)
            .into_iter()
            .find(|fact| fact.kind == LocalThreatKind::BrokenThree)
            .expect("corridor policy should see the existing broken three");
        assert!(CorridorThreatPolicy.is_active_threat(&existing, Color::Black, &corridor_fact));
        let continuations =
            legal_forcing_continuations_for_fact(&existing, Color::Black, &corridor_fact);
        assert_eq!(
            continuations
                .iter()
                .map(|continuation| continuation.mv)
                .collect::<Vec<_>>(),
            vec![mv("I8")]
        );
        assert_eq!(
            continuations[0].legal_cost_squares,
            vec![mv("G8"), mv("L8")]
        );
        assert_eq!(
            corridor_defender_reply_moves(&existing, Color::Black, None),
            vec![mv("G8"), mv("I8"), mv("L8")]
        );
    }

    #[test]
    fn player_explicit_annotation_matches_current_player_annotation() {
        let board = board_from_moves(Variant::Renju, &["H8", "A1", "I8", "A2"]);
        let policy = SearchThreatPolicy;

        assert_eq!(
            policy.annotation_for_player(&board, Color::Black, mv("J8")),
            policy.annotation_for_move(&board, mv("J8"))
        );

        let mut white_turn = board.clone();
        white_turn.current_player = Color::White;
        assert_eq!(
            policy.annotation_for_player(&board, Color::White, mv("B2")),
            policy.annotation_for_move(&white_turn, mv("B2"))
        );
    }

    #[test]
    fn known_legal_ordering_summary_matches_full_annotation_summary() {
        let policy = SearchThreatPolicy;

        let freestyle = board_from_moves(Variant::Freestyle, &["H8", "A1", "I8", "A2"]);
        let renju_white = board_from_moves(Variant::Renju, &["H8", "A1", "I8"]);
        let renju_forbidden_gap = board_from_moves(
            Variant::Renju,
            &["H8", "G8", "I8", "A1", "J8", "A2", "L8", "A3"],
        );

        let cases = [
            (&freestyle, Color::Black, mv("J8")),
            (&freestyle, Color::Black, mv("B2")),
            (&renju_white, Color::White, mv("B2")),
            (&renju_forbidden_gap, Color::Black, mv("M8")),
        ];

        for (board, player, probe) in cases {
            assert!(board.is_legal_for_color(probe, player));
            let annotation = policy.annotation_for_player(board, player, probe);
            assert_eq!(
                policy.ordering_summary_for_legal_player(board, player, probe),
                policy.ordering_summary(&annotation),
                "{player:?} {probe:?}"
            );
        }
    }

    #[test]
    fn raw_known_legal_ordering_summary_matches_raw_annotation_summary() {
        let policy = SearchThreatPolicy;

        let freestyle = board_from_moves(Variant::Freestyle, &["H8", "A1", "I8", "A2"]);
        let renju_black_raw = board_from_moves(
            Variant::Renju,
            &["H8", "G8", "I8", "A1", "J8", "A2", "L8", "A3"],
        );

        for (board, player, probe) in [
            (&freestyle, Color::Black, mv("J8")),
            (&freestyle, Color::Black, mv("B2")),
            (&renju_black_raw, Color::Black, mv("M8")),
        ] {
            assert!(board.is_legal_for_color(probe, player));
            let raw_annotation = policy.raw_annotation_for_legal_player(board, player, probe);
            assert_eq!(
                policy.raw_ordering_summary_for_legal_player(board, player, probe),
                policy.ordering_summary(&raw_annotation),
                "{player:?} {probe:?}"
            );
        }
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
            view.has_move_local_corridor_entry(Color::Black, mv("J8")),
            has_forcing_local_threat_at_move(&board, Color::Black, mv("J8"))
        );
        assert_eq!(
            view.local_corridor_entry_rank(Color::Black, mv("J8")) > 0,
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

        let raw_annotation =
            SearchThreatPolicy.raw_annotation_for_player(&board, Color::Black, mv("M8"));
        assert!(
            raw_annotation
                .local_threats
                .iter()
                .any(|fact| fact.kind == LocalThreatKind::BrokenFour),
            "raw annotation should preserve the forbidden-gap shape: {raw_annotation:?}"
        );

        let effective_annotation =
            SearchThreatPolicy.effective_annotation_from_raw(&board, raw_annotation);
        assert!(
            effective_annotation
                .local_threats
                .iter()
                .all(|fact| !SearchThreatPolicy.is_must_keep(fact)),
            "effective annotation should remove forbidden-only forcing threats: {effective_annotation:?}"
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
