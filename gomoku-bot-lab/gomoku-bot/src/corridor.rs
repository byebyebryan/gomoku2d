use gomoku_core::{Board, Color, GameResult, Move};
use serde::Serialize;
use serde_json::{json, Value};

use crate::tactical::{
    corridor_attacker_move_rank, corridor_defender_reply_moves, has_forcing_local_threat,
};
use crate::{Bot, RandomBot, SearchBot, SearchBotConfig};

pub const DEFAULT_MAX_CORRIDOR_DEPTH: usize = 4;
pub const DEFAULT_MAX_CORRIDOR_REPLY_WIDTH: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CorridorOptions {
    pub max_depth: usize,
    pub max_reply_width: usize,
}

impl Default for CorridorOptions {
    fn default() -> Self {
        Self {
            max_depth: DEFAULT_MAX_CORRIDOR_DEPTH,
            max_reply_width: DEFAULT_MAX_CORRIDOR_REPLY_WIDTH,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProofLimitCause {
    DepthCutoff,
    ReplyWidthCutoff,
    AttackerChildUnknown,
    DefenderReplyUnknown,
    ModelScopeUnknown,
    OutsideScanWindow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DefenderReplyRole {
    Actual,
    ImmediateDefense,
    ImminentDefense,
    OffensiveCounter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DefenderReplyOutcome {
    ForcedLoss,
    ConfirmedEscape,
    PossibleEscape,
    ImmediateLoss,
    Unknown,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct SearchDiagnostics {
    pub search_nodes: usize,
    pub branch_probes: usize,
    pub max_depth_reached: usize,
}

impl SearchDiagnostics {
    fn node(max_depth: usize, depth_remaining: usize) -> Self {
        Self {
            search_nodes: 1,
            branch_probes: 0,
            max_depth_reached: max_depth.saturating_sub(depth_remaining),
        }
    }

    fn record_branch_probe(&mut self) {
        self.branch_probes += 1;
    }

    fn merge(&mut self, other: Self) {
        self.search_nodes += other.search_nodes;
        self.branch_probes += other.branch_probes;
        self.max_depth_reached = self.max_depth_reached.max(other.max_depth_reached);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DefenderReplyAnalysis {
    pub mv: Move,
    pub notation: String,
    pub roles: Vec<DefenderReplyRole>,
    pub outcome: DefenderReplyOutcome,
    pub principal_line: Vec<Move>,
    pub principal_line_notation: Vec<String>,
    pub limit_causes: Vec<ProofLimitCause>,
    pub diagnostics: SearchDiagnostics,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefenderReplyCandidate {
    pub mv: Move,
    pub roles: Vec<DefenderReplyRole>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefenderReplyProof {
    pub outcome: DefenderReplyOutcome,
    pub principal_line: Vec<Move>,
    pub limit_causes: Vec<ProofLimitCause>,
    pub diagnostics: SearchDiagnostics,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorridorMoveReason {
    ImmediateWin,
    ConfirmedCorridorAttack,
    DefenseConfirmedEscape,
    DefensePossibleEscape,
    DefenseForcedReply,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorridorMoveChoice {
    pub mv: Move,
    pub reason: CorridorMoveReason,
    pub outcome: Option<DefenderReplyOutcome>,
    pub principal_line: Vec<Move>,
    pub diagnostics: SearchDiagnostics,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorridorBotFallback {
    Random,
    SearchD1,
}

pub struct CorridorBot {
    name: &'static str,
    options: CorridorOptions,
    fallback: CorridorBotFallback,
    random: RandomBot,
    search: SearchBot,
    last_trace: Option<Value>,
}

impl CorridorBot {
    pub fn with_random_fallback(seed: u64) -> Self {
        Self::new(CorridorBotFallback::Random, seed)
    }

    pub fn with_search_d1_fallback(seed: u64) -> Self {
        Self::new(CorridorBotFallback::SearchD1, seed)
    }

    pub fn with_search_fallback_config(seed: u64, search_config: SearchBotConfig) -> Self {
        Self::new_with_search_config(CorridorBotFallback::SearchD1, seed, search_config)
    }

    pub fn new(fallback: CorridorBotFallback, seed: u64) -> Self {
        Self::new_with_search_config(fallback, seed, SearchBotConfig::custom_depth(1))
    }

    fn new_with_search_config(
        fallback: CorridorBotFallback,
        seed: u64,
        search_config: SearchBotConfig,
    ) -> Self {
        let name = match fallback {
            CorridorBotFallback::Random => "corridor-random",
            CorridorBotFallback::SearchD1 => "corridor-d1",
        };
        Self {
            name,
            options: CorridorOptions {
                max_depth: 2,
                ..CorridorOptions::default()
            },
            fallback,
            random: RandomBot::seeded(seed),
            search: SearchBot::with_config(search_config),
            last_trace: None,
        }
    }
}

impl Bot for CorridorBot {
    fn name(&self) -> &str {
        self.name
    }

    fn choose_move(&mut self, board: &Board) -> Move {
        if let Some(choice) = choose_corridor_move(board, &self.options) {
            let mv = choice.mv;
            self.last_trace = Some(corridor_choice_trace(&choice));
            return mv;
        }

        match self.fallback {
            CorridorBotFallback::Random => {
                self.last_trace = None;
                self.random.choose_move(board)
            }
            CorridorBotFallback::SearchD1 => {
                let mv = self.search.choose_move(board);
                self.last_trace = self.search.trace().map(fallback_search_trace);
                mv
            }
        }
    }

    fn trace(&self) -> Option<serde_json::Value> {
        self.last_trace.clone()
    }
}

fn corridor_choice_trace(choice: &CorridorMoveChoice) -> Value {
    let proof_nodes = choice.diagnostics.search_nodes as u64;
    json!({
        "source": "corridor",
        "move": choice.mv.to_notation(),
        "reason": format!("{:?}", choice.reason),
        "outcome": choice.outcome.map(|outcome| format!("{outcome:?}")),
        "principal_line": choice
            .principal_line
            .iter()
            .map(|mv| mv.to_notation())
            .collect::<Vec<_>>(),
        "nodes": 0,
        "safety_nodes": proof_nodes,
        "total_nodes": proof_nodes,
        "depth": choice.diagnostics.max_depth_reached,
        "budget_exhausted": false,
        "corridor": {
            "search_nodes": proof_nodes,
            "branch_probes": choice.diagnostics.branch_probes,
            "max_depth_reached": choice.diagnostics.max_depth_reached,
        },
    })
}

fn fallback_search_trace(mut trace: Value) -> Value {
    if let Some(object) = trace.as_object_mut() {
        object.insert("source".to_string(), json!("corridor-fallback"));
        object.insert("fallback".to_string(), json!("search-d1"));
    }
    trace
}

pub fn choose_corridor_move(
    board: &Board,
    options: &CorridorOptions,
) -> Option<CorridorMoveChoice> {
    if board.result != GameResult::Ongoing {
        return None;
    }

    let player = board.current_player;
    if let Some(mv) = board.immediate_winning_moves_for(player).first().copied() {
        return Some(CorridorMoveChoice {
            mv,
            reason: CorridorMoveReason::ImmediateWin,
            outcome: None,
            principal_line: vec![mv],
            diagnostics: SearchDiagnostics::default(),
        });
    }

    let opponent = player.opponent();
    if !board.immediate_winning_moves_for(opponent).is_empty()
        || has_forcing_local_threat(board, opponent)
    {
        let replies = analyze_defender_reply_options(board, opponent, None, options);
        if !replies.is_empty() {
            if let Some(reply) =
                first_reply_with_outcome(&replies, DefenderReplyOutcome::ConfirmedEscape)
            {
                return Some(choice_from_reply(
                    reply,
                    CorridorMoveReason::DefenseConfirmedEscape,
                ));
            }
            if let Some(reply) =
                first_reply_with_outcome(&replies, DefenderReplyOutcome::PossibleEscape)
            {
                return Some(choice_from_reply(
                    reply,
                    CorridorMoveReason::DefensePossibleEscape,
                ));
            }
            if let Some(reply) = replies.iter().find(|reply| {
                matches!(
                    reply.outcome,
                    DefenderReplyOutcome::ForcedLoss | DefenderReplyOutcome::ImmediateLoss
                )
            }) {
                return Some(choice_from_reply(
                    reply,
                    CorridorMoveReason::DefenseForcedReply,
                ));
            }
        }
    }

    for mv in materialized_candidate_attacker_corridor_moves(board, player) {
        let mut next = board.clone();
        if next.apply_move(mv).is_err() {
            continue;
        }
        let proof = classify_attacker_corridor(&next, player, options, options.max_depth);
        if matches!(
            proof.outcome,
            DefenderReplyOutcome::ForcedLoss | DefenderReplyOutcome::ImmediateLoss
        ) {
            let mut principal_line = Vec::with_capacity(proof.principal_line.len() + 1);
            principal_line.push(mv);
            principal_line.extend(proof.principal_line);
            return Some(CorridorMoveChoice {
                mv,
                reason: CorridorMoveReason::ConfirmedCorridorAttack,
                outcome: Some(proof.outcome),
                principal_line,
                diagnostics: proof.diagnostics,
            });
        }
    }

    None
}

fn first_reply_with_outcome(
    replies: &[DefenderReplyAnalysis],
    outcome: DefenderReplyOutcome,
) -> Option<&DefenderReplyAnalysis> {
    replies.iter().find(|reply| reply.outcome == outcome)
}

fn choice_from_reply(
    reply: &DefenderReplyAnalysis,
    reason: CorridorMoveReason,
) -> CorridorMoveChoice {
    let mut principal_line = Vec::with_capacity(reply.principal_line.len() + 1);
    principal_line.push(reply.mv);
    principal_line.extend(reply.principal_line.iter().copied());
    CorridorMoveChoice {
        mv: reply.mv,
        reason,
        outcome: Some(reply.outcome),
        principal_line,
        diagnostics: reply.diagnostics,
    }
}

pub fn analyze_defender_reply_options(
    board: &Board,
    attacker: Color,
    actual_reply: Option<Move>,
    options: &CorridorOptions,
) -> Vec<DefenderReplyAnalysis> {
    analyze_defender_reply_options_inner(board, attacker, actual_reply, None, options)
}

pub fn analyze_alternate_defender_reply_options(
    board: &Board,
    attacker: Color,
    excluded_reply: Option<Move>,
    options: &CorridorOptions,
) -> Vec<DefenderReplyAnalysis> {
    analyze_defender_reply_options_inner(board, attacker, None, excluded_reply, options)
}

pub fn defender_reply_candidates(
    board: &Board,
    attacker: Color,
    actual_reply: Option<Move>,
) -> Vec<DefenderReplyCandidate> {
    defender_reply_candidates_inner(board, attacker, actual_reply, None)
        .into_iter()
        .map(|(mv, roles)| DefenderReplyCandidate { mv, roles })
        .collect()
}

pub fn defender_model_reply_candidates(
    board: &Board,
    attacker: Color,
    actual_reply: Option<Move>,
) -> Vec<DefenderReplyCandidate> {
    defender_reply_candidates_inner(board, attacker, actual_reply, None)
        .into_iter()
        .filter(|(_, roles)| !roles.iter().all(|role| *role == DefenderReplyRole::Actual))
        .map(|(mv, roles)| DefenderReplyCandidate { mv, roles })
        .collect()
}

pub fn defender_reply_roles_for_move(
    board: &Board,
    attacker: Color,
    mv: Move,
) -> Vec<DefenderReplyRole> {
    defender_reply_candidates_inner(board, attacker, None, None)
        .into_iter()
        .find_map(|(candidate, roles)| (candidate == mv).then_some(roles))
        .unwrap_or_default()
}

fn analyze_defender_reply_options_inner(
    board: &Board,
    attacker: Color,
    actual_reply: Option<Move>,
    excluded_reply: Option<Move>,
    options: &CorridorOptions,
) -> Vec<DefenderReplyAnalysis> {
    defender_reply_candidates_inner(board, attacker, actual_reply, excluded_reply)
        .into_iter()
        .map(|(mv, roles)| {
            let proof = classify_defender_reply(board, attacker, mv, options);
            defender_reply_analysis_from_proof(mv, roles, proof)
        })
        .collect()
}

fn defender_reply_candidates_inner(
    board: &Board,
    attacker: Color,
    actual_reply: Option<Move>,
    excluded_reply: Option<Move>,
) -> Vec<(Move, Vec<DefenderReplyRole>)> {
    if board.current_player != attacker.opponent() || board.result != GameResult::Ongoing {
        return Vec::new();
    }

    let threat = ThreatReplySet::new(board, attacker);
    let mut replies = Vec::<(Move, Vec<DefenderReplyRole>)>::new();
    for mv in threat.legal_cost_squares.iter().copied() {
        push_reply_role(&mut replies, mv, DefenderReplyRole::ImmediateDefense);
    }
    for mv in threat.defender_immediate_wins.iter().copied() {
        push_reply_role(&mut replies, mv, DefenderReplyRole::OffensiveCounter);
    }
    if threat.winning_squares.is_empty() {
        for mv in corridor_defender_reply_moves(board, attacker, actual_reply) {
            push_reply_role(&mut replies, mv, DefenderReplyRole::ImminentDefense);
        }
        for mv in offensive_counter_reply_moves(board, attacker.opponent()) {
            push_reply_role(&mut replies, mv, DefenderReplyRole::OffensiveCounter);
        }
    }
    if let Some(mv) = actual_reply {
        push_reply_role(&mut replies, mv, DefenderReplyRole::Actual);
    }
    if let Some(excluded_reply) = excluded_reply {
        replies.retain(|(mv, _)| *mv != excluded_reply);
    }

    replies
}

fn defender_reply_analysis_from_proof(
    mv: Move,
    roles: Vec<DefenderReplyRole>,
    proof: DefenderReplyProof,
) -> DefenderReplyAnalysis {
    let mut diagnostics = proof.diagnostics;
    diagnostics.record_branch_probe();
    DefenderReplyAnalysis {
        mv,
        notation: mv.to_notation(),
        roles,
        outcome: proof.outcome,
        principal_line_notation: proof
            .principal_line
            .iter()
            .map(|mv| mv.to_notation())
            .collect(),
        principal_line: proof.principal_line,
        limit_causes: proof.limit_causes,
        diagnostics,
    }
}

fn push_reply_role(
    replies: &mut Vec<(Move, Vec<DefenderReplyRole>)>,
    mv: Move,
    role: DefenderReplyRole,
) {
    if let Some((_, roles)) = replies.iter_mut().find(|(existing, _)| *existing == mv) {
        if !roles.contains(&role) {
            roles.push(role);
        }
        return;
    }
    replies.push((mv, vec![role]));
}

pub fn classify_defender_reply(
    board: &Board,
    attacker: Color,
    mv: Move,
    options: &CorridorOptions,
) -> DefenderReplyProof {
    classify_defender_reply_inner(board, attacker, mv, options, options.max_depth)
}

fn classify_defender_reply_inner(
    board: &Board,
    attacker: Color,
    mv: Move,
    options: &CorridorOptions,
    depth_remaining: usize,
) -> DefenderReplyProof {
    let diagnostics = SearchDiagnostics::node(options.max_depth, depth_remaining);
    let mut next = board.clone();
    if next.apply_move(mv).is_err() {
        return DefenderReplyProof {
            outcome: DefenderReplyOutcome::Unknown,
            principal_line: Vec::new(),
            limit_causes: vec![ProofLimitCause::ModelScopeUnknown],
            diagnostics,
        };
    }

    match next.result {
        GameResult::Winner(winner) if winner == attacker.opponent() => {
            return DefenderReplyProof {
                outcome: DefenderReplyOutcome::ConfirmedEscape,
                principal_line: Vec::new(),
                limit_causes: Vec::new(),
                diagnostics,
            };
        }
        GameResult::Winner(winner) if winner == attacker => {
            return DefenderReplyProof {
                outcome: DefenderReplyOutcome::ImmediateLoss,
                principal_line: Vec::new(),
                limit_causes: Vec::new(),
                diagnostics,
            };
        }
        GameResult::Winner(_) | GameResult::Draw => {
            return DefenderReplyProof {
                outcome: DefenderReplyOutcome::ConfirmedEscape,
                principal_line: Vec::new(),
                limit_causes: Vec::new(),
                diagnostics,
            };
        }
        GameResult::Ongoing => {}
    }

    let immediate_wins = next.immediate_winning_moves_for(attacker);
    if let Some(&winning_move) = immediate_wins.first() {
        return DefenderReplyProof {
            outcome: DefenderReplyOutcome::ImmediateLoss,
            principal_line: vec![winning_move],
            limit_causes: Vec::new(),
            diagnostics,
        };
    }

    let defender = attacker.opponent();
    if !next.immediate_winning_moves_for(defender).is_empty() {
        let mut proof = classify_defender_counter_threat(&next, attacker, options, depth_remaining);
        proof.diagnostics.merge(diagnostics);
        return proof;
    }

    let mut proof = classify_attacker_corridor(&next, attacker, options, depth_remaining);
    proof.diagnostics.merge(diagnostics);
    proof
}

fn classify_defender_counter_threat(
    board: &Board,
    attacker: Color,
    options: &CorridorOptions,
    depth_remaining: usize,
) -> DefenderReplyProof {
    let mut diagnostics = SearchDiagnostics::node(options.max_depth, depth_remaining);
    if depth_remaining == 0 {
        return DefenderReplyProof {
            outcome: DefenderReplyOutcome::PossibleEscape,
            principal_line: Vec::new(),
            limit_causes: vec![ProofLimitCause::DepthCutoff],
            diagnostics,
        };
    }

    let defender = attacker.opponent();
    let mut saw_unknown = false;
    let mut saw_possible_escape = false;
    let mut limit_causes = Vec::new();

    for mv in counter_threat_answer_moves(board, defender) {
        diagnostics.record_branch_probe();
        let mut next = board.clone();
        if next.apply_move(mv).is_err() {
            continue;
        }

        match next.result {
            GameResult::Winner(winner) if winner == attacker => {
                return DefenderReplyProof {
                    outcome: DefenderReplyOutcome::ForcedLoss,
                    principal_line: vec![mv],
                    limit_causes: Vec::new(),
                    diagnostics,
                };
            }
            GameResult::Winner(_) | GameResult::Draw => {
                continue;
            }
            GameResult::Ongoing => {}
        }

        if !next.immediate_winning_moves_for(defender).is_empty() {
            continue;
        }

        let proof = classify_narrow_corridor(&next, attacker, options, depth_remaining - 1);
        diagnostics.merge(proof.diagnostics);
        match proof.outcome {
            DefenderReplyOutcome::ForcedLoss | DefenderReplyOutcome::ImmediateLoss => {
                let mut principal_line = Vec::with_capacity(proof.principal_line.len() + 1);
                principal_line.push(mv);
                principal_line.extend(proof.principal_line);
                return DefenderReplyProof {
                    outcome: DefenderReplyOutcome::ForcedLoss,
                    principal_line,
                    limit_causes: proof.limit_causes,
                    diagnostics,
                };
            }
            DefenderReplyOutcome::ConfirmedEscape => {}
            DefenderReplyOutcome::PossibleEscape => {
                saw_possible_escape = true;
                extend_limit_causes(&mut limit_causes, proof.limit_causes);
            }
            DefenderReplyOutcome::Unknown => {
                saw_unknown = true;
                extend_limit_causes(&mut limit_causes, proof.limit_causes);
            }
        }
    }

    if saw_unknown || saw_possible_escape {
        return DefenderReplyProof {
            outcome: DefenderReplyOutcome::PossibleEscape,
            principal_line: Vec::new(),
            limit_causes,
            diagnostics,
        };
    }

    DefenderReplyProof {
        outcome: DefenderReplyOutcome::ConfirmedEscape,
        principal_line: Vec::new(),
        limit_causes: Vec::new(),
        diagnostics,
    }
}

fn classify_attacker_corridor(
    board: &Board,
    attacker: Color,
    options: &CorridorOptions,
    depth_remaining: usize,
) -> DefenderReplyProof {
    let mut diagnostics = SearchDiagnostics::node(options.max_depth, depth_remaining);
    if depth_remaining == 0 {
        return DefenderReplyProof {
            outcome: DefenderReplyOutcome::PossibleEscape,
            principal_line: Vec::new(),
            limit_causes: vec![ProofLimitCause::DepthCutoff],
            diagnostics,
        };
    }

    if board.current_player != attacker || board.result != GameResult::Ongoing {
        return DefenderReplyProof {
            outcome: DefenderReplyOutcome::ConfirmedEscape,
            principal_line: Vec::new(),
            limit_causes: Vec::new(),
            diagnostics,
        };
    }

    if let Some(winning_move) = board.immediate_winning_moves_for(attacker).first().copied() {
        return DefenderReplyProof {
            outcome: DefenderReplyOutcome::ForcedLoss,
            principal_line: vec![winning_move],
            limit_causes: Vec::new(),
            diagnostics,
        };
    }

    let mut saw_unknown = false;
    let mut saw_possible_escape = false;
    let mut limit_causes = Vec::new();
    for mv in materialized_attacker_corridor_moves(board, attacker) {
        diagnostics.record_branch_probe();
        let mut next = board.clone();
        if next.apply_move(mv).is_err() {
            continue;
        }

        match next.result {
            GameResult::Winner(winner) if winner == attacker => {
                return DefenderReplyProof {
                    outcome: DefenderReplyOutcome::ForcedLoss,
                    principal_line: vec![mv],
                    limit_causes: Vec::new(),
                    diagnostics,
                };
            }
            GameResult::Winner(_) | GameResult::Draw => continue,
            GameResult::Ongoing => {}
        }

        let proof = classify_narrow_corridor(&next, attacker, options, depth_remaining - 1);
        diagnostics.merge(proof.diagnostics);
        match proof.outcome {
            DefenderReplyOutcome::ForcedLoss | DefenderReplyOutcome::ImmediateLoss => {
                let mut principal_line = Vec::with_capacity(proof.principal_line.len() + 1);
                principal_line.push(mv);
                principal_line.extend(proof.principal_line);
                return DefenderReplyProof {
                    outcome: DefenderReplyOutcome::ForcedLoss,
                    principal_line,
                    limit_causes: proof.limit_causes,
                    diagnostics,
                };
            }
            DefenderReplyOutcome::ConfirmedEscape => {}
            DefenderReplyOutcome::PossibleEscape => {
                saw_possible_escape = true;
                extend_limit_causes(&mut limit_causes, proof.limit_causes);
            }
            DefenderReplyOutcome::Unknown => {
                saw_unknown = true;
                extend_limit_causes(&mut limit_causes, proof.limit_causes);
            }
        }
    }

    if saw_unknown || saw_possible_escape {
        return DefenderReplyProof {
            outcome: DefenderReplyOutcome::PossibleEscape,
            principal_line: Vec::new(),
            limit_causes,
            diagnostics,
        };
    }

    DefenderReplyProof {
        outcome: DefenderReplyOutcome::ConfirmedEscape,
        principal_line: Vec::new(),
        limit_causes: Vec::new(),
        diagnostics,
    }
}

fn classify_narrow_corridor(
    board: &Board,
    attacker: Color,
    options: &CorridorOptions,
    depth_remaining: usize,
) -> DefenderReplyProof {
    let mut diagnostics = SearchDiagnostics::node(options.max_depth, depth_remaining);
    if board.current_player != attacker.opponent() || board.result != GameResult::Ongoing {
        return DefenderReplyProof {
            outcome: DefenderReplyOutcome::ConfirmedEscape,
            principal_line: Vec::new(),
            limit_causes: Vec::new(),
            diagnostics,
        };
    }

    let reply_moves = narrow_corridor_reply_moves(board, attacker);
    if reply_moves.is_empty() {
        return DefenderReplyProof {
            outcome: DefenderReplyOutcome::ConfirmedEscape,
            principal_line: Vec::new(),
            limit_causes: Vec::new(),
            diagnostics,
        };
    }
    if reply_moves.len() > options.max_reply_width {
        return DefenderReplyProof {
            outcome: DefenderReplyOutcome::PossibleEscape,
            principal_line: Vec::new(),
            limit_causes: vec![ProofLimitCause::ReplyWidthCutoff],
            diagnostics,
        };
    }

    let mut principal_line = Vec::new();
    let mut saw_possible_escape = false;
    let mut saw_unknown = false;
    let mut limit_causes = Vec::new();
    for mv in reply_moves {
        diagnostics.record_branch_probe();
        let proof = classify_defender_reply_inner(board, attacker, mv, options, depth_remaining);
        diagnostics.merge(proof.diagnostics);
        match proof.outcome {
            DefenderReplyOutcome::ForcedLoss | DefenderReplyOutcome::ImmediateLoss => {
                if principal_line.is_empty() {
                    principal_line.push(mv);
                    principal_line.extend(proof.principal_line);
                }
            }
            DefenderReplyOutcome::ConfirmedEscape => {
                return DefenderReplyProof {
                    outcome: DefenderReplyOutcome::ConfirmedEscape,
                    principal_line: Vec::new(),
                    limit_causes: Vec::new(),
                    diagnostics,
                };
            }
            DefenderReplyOutcome::PossibleEscape => {
                saw_possible_escape = true;
                extend_limit_causes(&mut limit_causes, proof.limit_causes);
            }
            DefenderReplyOutcome::Unknown => {
                saw_unknown = true;
                extend_limit_causes(&mut limit_causes, proof.limit_causes);
            }
        }
    }

    if saw_possible_escape {
        return DefenderReplyProof {
            outcome: DefenderReplyOutcome::PossibleEscape,
            principal_line: Vec::new(),
            limit_causes,
            diagnostics,
        };
    }
    if saw_unknown {
        return DefenderReplyProof {
            outcome: DefenderReplyOutcome::Unknown,
            principal_line: Vec::new(),
            limit_causes,
            diagnostics,
        };
    }

    DefenderReplyProof {
        outcome: DefenderReplyOutcome::ForcedLoss,
        principal_line,
        limit_causes: Vec::new(),
        diagnostics,
    }
}

fn narrow_corridor_reply_moves(board: &Board, attacker: Color) -> Vec<Move> {
    let threat = ThreatReplySet::new(board, attacker);
    if !threat.winning_squares.is_empty() {
        return threat.reply_moves;
    }

    corridor_defender_reply_moves(board, attacker, None)
}

fn counter_threat_answer_moves(board: &Board, defender: Color) -> Vec<Move> {
    let mut moves = Vec::new();
    for mv in board.immediate_winning_moves_for(defender) {
        if board.is_legal_for_color(mv, defender.opponent()) {
            push_unique_move(&mut moves, mv);
        }
    }
    moves
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

struct ThreatReplySet {
    winning_squares: Vec<Move>,
    legal_cost_squares: Vec<Move>,
    defender_immediate_wins: Vec<Move>,
    reply_moves: Vec<Move>,
}

impl ThreatReplySet {
    fn new(board: &Board, attacker: Color) -> Self {
        let defender = attacker.opponent();
        let winning_squares = board.immediate_winning_moves_for(attacker);
        let mut legal_cost_squares = Vec::new();
        for mv in winning_squares.iter().copied() {
            if board.is_legal_for_color(mv, defender) {
                legal_cost_squares.push(mv);
            }
        }
        let defender_immediate_wins = board.immediate_winning_moves_for(defender);
        let mut reply_moves = legal_cost_squares.clone();
        for mv in defender_immediate_wins.iter().copied() {
            push_unique_move(&mut reply_moves, mv);
        }

        Self {
            winning_squares,
            legal_cost_squares,
            defender_immediate_wins,
            reply_moves,
        }
    }
}

pub fn is_corridor_attacker_move(board: &Board, attacker: Color, mv: Move) -> bool {
    if board.current_player != attacker || !board.is_legal_for_color(mv, attacker) {
        return false;
    }
    let mut next = board.clone();
    if next.apply_move(mv).is_err() {
        return false;
    }
    match next.result {
        GameResult::Winner(winner) if winner == attacker => return true,
        GameResult::Winner(_) | GameResult::Draw => return false,
        GameResult::Ongoing => {}
    }
    if !next.immediate_winning_moves_for(attacker).is_empty() {
        return true;
    }
    has_forcing_local_threat(&next, attacker)
}

pub fn materialized_attacker_corridor_moves(board: &Board, attacker: Color) -> Vec<Move> {
    materialized_attacker_corridor_moves_from_candidates(board, attacker, board.legal_moves())
}

fn materialized_candidate_attacker_corridor_moves(board: &Board, attacker: Color) -> Vec<Move> {
    materialized_attacker_corridor_moves_from_candidates(
        board,
        attacker,
        corridor_candidate_moves(board, 2),
    )
}

fn materialized_attacker_corridor_moves_from_candidates(
    board: &Board,
    attacker: Color,
    candidates: Vec<Move>,
) -> Vec<Move> {
    let mut moves = candidates
        .into_iter()
        .filter_map(|mv| {
            let rank = corridor_attacker_move_rank(board, attacker, mv);
            (rank > 0).then_some((mv, rank))
        })
        .collect::<Vec<_>>();
    let Some(best_rank) = moves.iter().map(|(_, rank)| *rank).max() else {
        return Vec::new();
    };
    moves.retain(|(_, rank)| *rank == best_rank);
    moves.sort_by_key(|(mv, _)| (mv.row, mv.col));
    moves.into_iter().map(|(mv, _)| mv).collect()
}

fn corridor_candidate_moves(board: &Board, radius: usize) -> Vec<Move> {
    if board.history.is_empty() {
        let center = board.config.board_size / 2;
        return vec![Move {
            row: center,
            col: center,
        }];
    }

    let size = board.config.board_size;
    let mut seen = vec![false; size * size];
    let mut moves = Vec::new();
    let radius = radius as isize;
    board.for_each_occupied(|row, col, _| {
        let row = row as isize;
        let col = col as isize;
        for dr in -radius..=radius {
            for dc in -radius..=radius {
                let r = row + dr;
                let c = col + dc;
                if r < 0 || r >= size as isize || c < 0 || c >= size as isize {
                    continue;
                }

                let mv = Move {
                    row: r as usize,
                    col: c as usize,
                };
                let idx = mv.row * size + mv.col;
                if seen[idx] || !board.is_legal(mv) {
                    continue;
                }
                seen[idx] = true;
                moves.push(mv);
            }
        }
    });
    moves.sort_by_key(|mv| (mv.row, mv.col));
    moves
}

fn push_unique_move(moves: &mut Vec<Move>, mv: Move) {
    if !moves.contains(&mv) {
        moves.push(mv);
    }
}

fn extend_limit_causes(
    causes: &mut Vec<ProofLimitCause>,
    new_causes: impl IntoIterator<Item = ProofLimitCause>,
) {
    for cause in new_causes {
        if !causes.contains(&cause) {
            causes.push(cause);
        }
    }
    causes.sort();
}

#[cfg(test)]
mod tests {
    use super::{
        analyze_defender_reply_options, DefenderReplyOutcome, DefenderReplyRole, ProofLimitCause,
    };
    use crate::{Bot, CorridorBot, CorridorOptions, SearchBotConfig};
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
            board
                .apply_move(mv(notation))
                .expect("fixture move should apply");
        }
        board
    }

    fn reply_for<'a>(
        replies: &'a [super::DefenderReplyAnalysis],
        notation: &str,
    ) -> &'a super::DefenderReplyAnalysis {
        replies
            .iter()
            .find(|reply| reply.notation == notation)
            .unwrap_or_else(|| panic!("expected reply {notation}"))
    }

    #[test]
    fn corridor_replies_distinguish_forced_loss_from_unproven_counterplay() {
        let board = board_from_moves(
            Variant::Renju,
            &[
                "H8", "I8", "H7", "I7", "H6", "H5", "I6", "I9", "G6", "J6", "G8", "J5", "G5",
            ],
        );
        let replies = analyze_defender_reply_options(
            &board,
            Color::Black,
            Some(mv("G7")),
            &Default::default(),
        );

        for notation in ["G4", "G7", "G9"] {
            let reply = reply_for(&replies, notation);
            assert!(reply.roles.contains(&DefenderReplyRole::ImminentDefense));
            assert_eq!(reply.outcome, DefenderReplyOutcome::ForcedLoss);
        }

        let i10 = reply_for(&replies, "I10");
        assert!(i10.roles.contains(&DefenderReplyRole::OffensiveCounter));
        assert_eq!(i10.outcome, DefenderReplyOutcome::PossibleEscape);
        assert!(i10.limit_causes.contains(&ProofLimitCause::DepthCutoff));
    }

    #[test]
    fn open_three_with_blocked_outer_side_includes_far_defense_square() {
        let board = board_from_moves(Variant::Renju, &["J9", "H9", "K9", "A1", "L9"]);

        let replies = analyze_defender_reply_options(
            &board,
            Color::Black,
            Some(mv("N9")),
            &Default::default(),
        );
        let reply = reply_for(&replies, "N9");
        assert!(reply.roles.contains(&DefenderReplyRole::ImminentDefense));
    }

    #[test]
    fn open_three_with_right_blocked_outer_side_includes_far_defense_square() {
        let board = board_from_moves(Variant::Renju, &["J9", "N9", "K9", "A1", "L9"]);

        let replies = analyze_defender_reply_options(
            &board,
            Color::Black,
            Some(mv("I9")),
            &Default::default(),
        );
        let reply = reply_for(&replies, "I9");
        assert!(reply.roles.contains(&DefenderReplyRole::ImminentDefense));
    }

    #[test]
    fn corridor_bot_takes_immediate_win_before_fallback() {
        let board = board_from_moves(
            Variant::Freestyle,
            &["H8", "A1", "I8", "A2", "J8", "A3", "K8"],
        );
        let mut bot = CorridorBot::with_random_fallback(7);

        assert_eq!(bot.choose_move(&board), mv("G8"));
    }

    #[test]
    fn corridor_bot_search_fallback_preserves_search_trace() {
        let board = Board::new(RuleConfig::default());
        let mut bot = CorridorBot::with_search_d1_fallback(7);

        let _ = bot.choose_move(&board);
        let trace = bot
            .trace()
            .expect("search fallback should expose the underlying search trace");

        assert_eq!(trace["source"], "corridor-fallback");
        assert_eq!(trace["fallback"], "search-d1");
        assert!(trace["total_nodes"].as_u64().unwrap_or_default() > 0);
        assert_eq!(trace["config"]["max_depth"], 1);
    }

    #[test]
    fn corridor_bot_search_fallback_accepts_search_config() {
        let board = Board::new(RuleConfig::default());
        let mut config = SearchBotConfig::custom_depth(1);
        config.cpu_time_budget_ms = Some(123);
        let mut bot = CorridorBot::with_search_fallback_config(7, config);

        let _ = bot.choose_move(&board);
        let trace = bot
            .trace()
            .expect("configured search fallback should expose the underlying search trace");

        assert_eq!(trace["config"]["max_depth"], 1);
        assert_eq!(trace["config"]["cpu_time_budget_ms"], 123);
    }

    #[test]
    fn zero_depth_marks_corridor_reply_as_possible_escape() {
        let board = board_from_moves(
            Variant::Freestyle,
            &[
                "H8", "G8", "I8", "A1", "J8", "C1", "K6", "E1", "K7", "G1", "K8",
            ],
        );
        let replies = analyze_defender_reply_options(
            &board,
            Color::Black,
            Some(mv("L8")),
            &CorridorOptions {
                max_depth: 0,
                ..CorridorOptions::default()
            },
        );

        let reply = reply_for(&replies, "L8");
        assert_eq!(reply.outcome, DefenderReplyOutcome::PossibleEscape);
        assert!(reply.limit_causes.contains(&ProofLimitCause::DepthCutoff));
    }
}
