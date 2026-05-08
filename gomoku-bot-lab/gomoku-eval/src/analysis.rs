use gomoku_core::{replay::ReplayResult, Board, Color, GameResult, Move, Replay, Variant};
use serde::Serialize;

pub const ANALYSIS_SCHEMA_VERSION: u32 = 13;
pub const DEFAULT_MAX_SCAN_PLIES: usize = 64;
const MAX_HYBRID_LOCAL_THREAT_COUNT: usize = 2;
const MAX_HYBRID_LOCAL_THREAT_REPLIES: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProofStatus {
    ForcedWin,
    EscapeFound,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DefensePolicy {
    AllLegalDefense,
    TacticalDefense,
    HybridDefense,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RootCause {
    StrategicLoss,
    MissedDefense,
    MissedWin,
    Unclear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UnclearReason {
    PreviousPrefixUnknown,
    ScanWindowCutoff,
    ProofLimitHit,
    NoFinalForcedInterval,
    DrawOrOngoing,
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
pub enum TacticalNote {
    AccidentalBlunder,
    ConversionError,
    MissedWin,
    StrongAttack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplyClassification {
    IgnoredSingleWin,
    BlockedButForced,
    ConfirmedEscape,
    PossibleEscape,
    NoLegalBlock,
    Unknown,
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

#[derive(Debug, Clone)]
pub struct AnalysisOptions {
    pub defense_policy: DefensePolicy,
    pub max_depth: usize,
    pub max_scan_plies: Option<usize>,
}

impl Default for AnalysisOptions {
    fn default() -> Self {
        Self {
            defense_policy: DefensePolicy::AllLegalDefense,
            max_depth: 4,
            max_scan_plies: Some(DEFAULT_MAX_SCAN_PLIES),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AnalysisModel {
    pub defense_policy: DefensePolicy,
    pub tactical_reply_coverage: Vec<String>,
    pub attacker_move_policy: String,
    pub rule_set: String,
    pub max_depth: usize,
    pub max_scan_plies: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProofResult {
    pub status: ProofStatus,
    pub attacker: Color,
    pub side_to_move: Color,
    pub model: AnalysisModel,
    pub principal_line: Vec<Move>,
    pub escape_moves: Vec<Move>,
    pub threat_evidence: Vec<ThreatSequenceEvidence>,
    pub limit_hit: bool,
    pub limit_causes: Vec<ProofLimitCause>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ThreatSequenceEvidence {
    pub prefix_ply: Option<usize>,
    pub attacker: Color,
    pub defender: Color,
    pub winning_squares: Vec<Move>,
    pub raw_cost_squares: Vec<Move>,
    pub legal_cost_squares: Vec<Move>,
    pub illegal_cost_squares: Vec<Move>,
    pub defender_immediate_wins: Vec<Move>,
    pub actual_reply: Option<Move>,
    pub reply_classification: ReplyClassification,
    pub escape_replies: Vec<Move>,
    pub forced_replies: Vec<Move>,
    pub next_forcing_move: Option<Move>,
    pub proof_status: ProofStatus,
    pub limit_hit: bool,
    pub limit_causes: Vec<ProofLimitCause>,
}

#[derive(Debug, Clone, Copy)]
struct EvidenceAttribution {
    prefix_ply: Option<usize>,
    actual_reply: Option<Move>,
}

struct ThreatEvidenceInput {
    attribution: EvidenceAttribution,
    reply_classification: ReplyClassification,
    escape_replies: Vec<Move>,
    forced_replies: Vec<Move>,
    next_forcing_move: Option<Move>,
    proof_status: ProofStatus,
    limit_causes: Vec<ProofLimitCause>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ForcedInterval {
    pub start_ply: usize,
    pub end_ply: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GameAnalysis {
    pub schema_version: u32,
    pub rule_set: String,
    pub winner: Option<Color>,
    pub loser: Option<Color>,
    pub final_move: Option<Move>,
    pub final_winning_line: Vec<Move>,
    pub model: AnalysisModel,
    pub final_forced_interval_found: bool,
    pub final_forced_interval: ForcedInterval,
    pub proof_intervals: Vec<ForcedInterval>,
    pub unknown_gaps: Vec<usize>,
    pub unclear_reason: Option<UnclearReason>,
    pub unclear_context: Option<UnclearContext>,
    pub last_chance_ply: Option<usize>,
    pub decisive_attack_ply: Option<usize>,
    pub critical_mistake_ply: Option<usize>,
    pub root_cause: RootCause,
    pub tactical_notes: Vec<TacticalNote>,
    pub principal_line: Vec<Move>,
    pub proof_summary: Vec<ProofResult>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UnclearContext {
    pub reason: UnclearReason,
    pub previous_prefix_ply: Option<usize>,
    pub final_forced_interval: ForcedInterval,
    pub previous_proof_status: Option<ProofStatus>,
    pub previous_proof_limit_hit: Option<bool>,
    pub previous_limit_causes: Vec<ProofLimitCause>,
    pub previous_side_to_move: Option<Color>,
    pub winner: Color,
    pub principal_line: Vec<Move>,
    pub principal_line_notation: Vec<String>,
    pub scan_start_ply: usize,
    pub scan_end_ply: Option<usize>,
    pub move_count: usize,
    pub snapshots: Vec<AnalysisBoardSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AnalysisBoardSnapshot {
    pub label: String,
    pub ply: usize,
    pub side_to_move: Color,
    pub rows: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnalysisError {
    InvalidReplayMove { ply: usize, message: String },
}

impl std::fmt::Display for AnalysisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnalysisError::InvalidReplayMove { ply, message } => {
                write!(f, "invalid replay move at ply {ply}: {message}")
            }
        }
    }
}

impl std::error::Error for AnalysisError {}

pub fn analyze_replay(
    replay: &Replay,
    options: AnalysisOptions,
) -> Result<GameAnalysis, AnalysisError> {
    let boards = replay_prefix_boards(replay)?;
    let final_board = boards
        .last()
        .expect("replay prefixes include initial board");
    let winner = replay_winner(replay, final_board);
    let loser = winner.map(Color::opponent);
    let model = corridor_analysis_model(final_board, &options);

    if winner.is_none() {
        return Ok(GameAnalysis {
            schema_version: ANALYSIS_SCHEMA_VERSION,
            rule_set: rule_label(&replay.rules.variant).to_string(),
            winner,
            loser,
            final_move: replay
                .moves
                .last()
                .and_then(|mv| Move::from_notation(&mv.mv).ok()),
            final_winning_line: Vec::new(),
            model,
            final_forced_interval_found: false,
            final_forced_interval: ForcedInterval {
                start_ply: 0,
                end_ply: 0,
            },
            proof_intervals: Vec::new(),
            unknown_gaps: Vec::new(),
            unclear_reason: Some(UnclearReason::DrawOrOngoing),
            unclear_context: None,
            last_chance_ply: None,
            decisive_attack_ply: None,
            critical_mistake_ply: None,
            root_cause: RootCause::Unclear,
            tactical_notes: Vec::new(),
            principal_line: Vec::new(),
            proof_summary: Vec::new(),
        });
    }

    let winner = winner.expect("checked above");
    let actual_moves = replay_moves(replay)?;
    let (scan_start, proof_summary) =
        replay_proof_summary_until_boundary(&boards, &actual_moves, winner, &options);
    let proof_intervals = proof_intervals(&proof_summary, scan_start);
    let (final_forced_interval_found, final_forced_interval) =
        find_final_forced_interval(&proof_intervals, replay.moves.len());
    let unknown_gaps = proof_summary
        .iter()
        .enumerate()
        .filter_map(|(idx, proof)| {
            (proof.status == ProofStatus::Unknown).then_some(scan_start + idx)
        })
        .collect::<Vec<_>>();

    let previous_status = final_forced_interval
        .start_ply
        .checked_sub(1)
        .and_then(|ply| proof_at(&proof_summary, scan_start, ply))
        .map(|proof| proof.status);
    let move_color = color_for_ply(final_forced_interval.start_ply);
    let missed_win_root = loser.is_some_and(|loser| {
        losing_side_missed_immediate_win(replay, &boards, final_forced_interval.start_ply, loser)
    });
    let root_cause = classify_root_cause(previous_status, move_color, winner, missed_win_root);
    let last_chance_ply = find_last_chance(
        &boards,
        &proof_summary,
        scan_start,
        final_forced_interval.start_ply,
        loser,
    );
    let critical_mistake_ply = match root_cause {
        RootCause::MissedDefense | RootCause::MissedWin => Some(final_forced_interval.start_ply),
        _ => None,
    };
    let decisive_attack_ply =
        (move_color == Some(winner)).then_some(final_forced_interval.start_ply);
    let tactical_notes = tactical_notes(TacticalNoteInput {
        replay,
        boards: &boards,
        proofs: &proof_summary,
        scan_start,
        proof_intervals: &proof_intervals,
        final_forced_interval: &final_forced_interval,
        winner,
        root_cause,
    });
    let principal_line = proof_at(&proof_summary, scan_start, final_forced_interval.start_ply)
        .map(|proof| proof.principal_line.clone())
        .unwrap_or_default();
    let unclear_reason = unclear_reason(UnclearReasonInput {
        root_cause,
        final_forced_interval_found,
        final_forced_interval: &final_forced_interval,
        previous_status,
        proof_summary: &proof_summary,
        scan_start,
    });
    let unclear_context = unclear_context(UnclearContextInput {
        root_cause,
        unclear_reason,
        final_forced_interval: &final_forced_interval,
        proof_summary: &proof_summary,
        scan_start,
        boards: &boards,
        winner,
        principal_line: &principal_line,
        move_count: replay.moves.len(),
    });

    Ok(GameAnalysis {
        schema_version: ANALYSIS_SCHEMA_VERSION,
        rule_set: rule_label(&replay.rules.variant).to_string(),
        winner: Some(winner),
        loser,
        final_move: replay
            .moves
            .last()
            .and_then(|mv| Move::from_notation(&mv.mv).ok()),
        final_winning_line: final_board.winning_line(),
        model,
        final_forced_interval_found,
        final_forced_interval,
        proof_intervals,
        unknown_gaps,
        unclear_reason,
        unclear_context,
        last_chance_ply,
        decisive_attack_ply,
        critical_mistake_ply,
        root_cause,
        tactical_notes,
        principal_line,
        proof_summary,
    })
}

pub fn analyze_defender_reply_options(
    board: &Board,
    attacker: Color,
    actual_reply: Option<Move>,
    options: &AnalysisOptions,
) -> Vec<DefenderReplyAnalysis> {
    analyze_defender_reply_options_inner(board, attacker, actual_reply, None, options)
}

pub fn analyze_alternate_defender_reply_options(
    board: &Board,
    attacker: Color,
    excluded_reply: Option<Move>,
    options: &AnalysisOptions,
) -> Vec<DefenderReplyAnalysis> {
    analyze_defender_reply_options_inner(board, attacker, None, excluded_reply, options)
}

pub fn defender_reply_roles_for_move(
    board: &Board,
    attacker: Color,
    mv: Move,
) -> Vec<DefenderReplyRole> {
    if board.current_player != attacker.opponent() || board.result != GameResult::Ongoing {
        return Vec::new();
    }

    let threat = ThreatReplySet::new(board, attacker, true);
    let mut roles = Vec::new();
    if threat.legal_cost_squares.contains(&mv) {
        roles.push(DefenderReplyRole::ImmediateDefense);
    }
    if threat.winning_squares.is_empty() {
        if imminent_defense_reply_moves(board, attacker, Some(mv)).contains(&mv) {
            roles.push(DefenderReplyRole::ImminentDefense);
        }
        if offensive_counter_reply_moves(board, attacker.opponent()).contains(&mv) {
            roles.push(DefenderReplyRole::OffensiveCounter);
        }
    }
    roles
}

fn analyze_defender_reply_options_inner(
    board: &Board,
    attacker: Color,
    actual_reply: Option<Move>,
    excluded_reply: Option<Move>,
    options: &AnalysisOptions,
) -> Vec<DefenderReplyAnalysis> {
    if board.current_player != attacker.opponent() || board.result != GameResult::Ongoing {
        return Vec::new();
    }

    let threat = ThreatReplySet::new(board, attacker, true);
    let mut replies = Vec::<(Move, Vec<DefenderReplyRole>)>::new();
    for mv in threat.legal_cost_squares.iter().copied() {
        push_reply_role(&mut replies, mv, DefenderReplyRole::ImmediateDefense);
    }
    if threat.winning_squares.is_empty() {
        for mv in imminent_defense_reply_moves(board, attacker, actual_reply) {
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
        .into_iter()
        .map(|(mv, roles)| {
            let proof = classify_defender_reply_for_report(board, attacker, mv, options);
            defender_reply_analysis_from_proof(mv, roles, proof)
        })
        .collect()
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

struct DefenderReplyProof {
    outcome: DefenderReplyOutcome,
    principal_line: Vec<Move>,
    limit_causes: Vec<ProofLimitCause>,
    diagnostics: SearchDiagnostics,
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

fn classify_defender_reply_for_report(
    board: &Board,
    attacker: Color,
    mv: Move,
    options: &AnalysisOptions,
) -> DefenderReplyProof {
    classify_defender_reply_for_report_inner(board, attacker, mv, options, options.max_depth)
}

fn classify_defender_reply_for_report_inner(
    board: &Board,
    attacker: Color,
    mv: Move,
    options: &AnalysisOptions,
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
        let mut proof =
            classify_defender_counter_threat_for_report(&next, attacker, options, depth_remaining);
        proof.diagnostics.merge(diagnostics);
        return proof;
    }

    let mut proof =
        classify_attacker_corridor_for_report(&next, attacker, options, depth_remaining);
    proof.diagnostics.merge(diagnostics);
    proof
}

fn classify_defender_counter_threat_for_report(
    board: &Board,
    attacker: Color,
    options: &AnalysisOptions,
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

        let proof =
            classify_narrow_corridor_for_report(&next, attacker, options, depth_remaining - 1);
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

    if saw_unknown {
        return DefenderReplyProof {
            outcome: DefenderReplyOutcome::PossibleEscape,
            principal_line: Vec::new(),
            limit_causes,
            diagnostics,
        };
    }
    if saw_possible_escape {
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

fn classify_attacker_corridor_for_report(
    board: &Board,
    attacker: Color,
    options: &AnalysisOptions,
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

        let proof =
            classify_narrow_corridor_for_report(&next, attacker, options, depth_remaining - 1);
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

    if saw_unknown {
        return DefenderReplyProof {
            outcome: DefenderReplyOutcome::PossibleEscape,
            principal_line: Vec::new(),
            limit_causes,
            diagnostics,
        };
    }
    if saw_possible_escape {
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

fn classify_narrow_corridor_for_report(
    board: &Board,
    attacker: Color,
    options: &AnalysisOptions,
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
    if reply_moves.len() > MAX_HYBRID_LOCAL_THREAT_REPLIES {
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
        let proof =
            classify_defender_reply_for_report_inner(board, attacker, mv, options, depth_remaining);
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
    let threat = ThreatReplySet::new(board, attacker, true);
    if !threat.winning_squares.is_empty() {
        return threat.reply_moves;
    }

    imminent_defense_reply_moves(board, attacker, None)
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

fn imminent_defense_reply_moves(
    board: &Board,
    attacker: Color,
    actual_reply: Option<Move>,
) -> Vec<Move> {
    let defender = attacker.opponent();
    let mut replies = Vec::new();

    let mut facts = local_threat_facts(board, attacker)
        .into_iter()
        .filter(|fact| local_threat_is_forcing_for(board, attacker, fact))
        .collect::<Vec<_>>();
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
        .map(|fact| fact.kind.rank())
        .max()
        .expect("facts are not empty");
    for fact in facts
        .into_iter()
        .filter(|fact| fact.kind.rank() == best_rank)
    {
        add_imminent_defense_replies_for_fact(board, attacker, defender, &fact, &mut replies);
    }

    replies
}

fn add_imminent_defense_replies_for_fact(
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
    attacker: Color,
    defender: Color,
    winning_squares: Vec<Move>,
    raw_cost_squares: Vec<Move>,
    legal_cost_squares: Vec<Move>,
    illegal_cost_squares: Vec<Move>,
    defender_immediate_wins: Vec<Move>,
    local_threat_count: usize,
    local_threat_replies: Vec<Move>,
    reply_moves: Vec<Move>,
}

impl ThreatReplySet {
    fn new(board: &Board, attacker: Color, include_local_threats: bool) -> Self {
        let defender = attacker.opponent();
        let winning_squares = board.immediate_winning_moves_for(attacker);
        let raw_cost_squares = winning_squares.clone();
        let mut legal_cost_squares = Vec::new();
        let mut illegal_cost_squares = Vec::new();
        for mv in raw_cost_squares.iter().copied() {
            if board.is_legal_for_color(mv, defender) {
                legal_cost_squares.push(mv);
            } else {
                illegal_cost_squares.push(mv);
            }
        }
        let defender_immediate_wins = board.immediate_winning_moves_for(defender);
        let mut reply_moves = legal_cost_squares.clone();
        for mv in defender_immediate_wins.iter().copied() {
            if !reply_moves.contains(&mv) {
                reply_moves.push(mv);
            }
        }
        let local_threats = if include_local_threats {
            local_threat_facts(board, attacker)
        } else {
            Vec::new()
        };
        let local_threat_count = local_threats
            .iter()
            .filter(|fact| local_threat_is_forcing_for(board, attacker, fact))
            .count();
        let local_threat_replies = local_threat_reply_moves(board, defender, &local_threats);

        Self {
            attacker,
            defender,
            winning_squares,
            raw_cost_squares,
            legal_cost_squares,
            illegal_cost_squares,
            defender_immediate_wins,
            local_threat_count,
            local_threat_replies,
            reply_moves,
        }
    }

    fn evidence(&self, input: ThreatEvidenceInput) -> ThreatSequenceEvidence {
        let limit_hit = !input.limit_causes.is_empty();
        ThreatSequenceEvidence {
            prefix_ply: input.attribution.prefix_ply,
            attacker: self.attacker,
            defender: self.defender,
            winning_squares: self.winning_squares.clone(),
            raw_cost_squares: self.raw_cost_squares.clone(),
            legal_cost_squares: self.legal_cost_squares.clone(),
            illegal_cost_squares: self.illegal_cost_squares.clone(),
            defender_immediate_wins: self.defender_immediate_wins.clone(),
            actual_reply: input.attribution.actual_reply,
            reply_classification: input.reply_classification,
            escape_replies: input.escape_replies,
            forced_replies: input.forced_replies,
            next_forcing_move: input.next_forcing_move,
            proof_status: input.proof_status,
            limit_hit,
            limit_causes: input.limit_causes,
        }
    }
}

fn next_attacker_move_after_defender_reply(principal_line: &[Move]) -> Option<Move> {
    principal_line.get(1).copied()
}

fn proof_limit_hit_from_evidence(threat_evidence: &[ThreatSequenceEvidence]) -> bool {
    !proof_limit_causes_from_evidence(threat_evidence).is_empty()
        || threat_evidence.iter().any(|evidence| evidence.limit_hit)
}

fn proof_limit_causes_from_evidence(
    threat_evidence: &[ThreatSequenceEvidence],
) -> Vec<ProofLimitCause> {
    let mut causes = Vec::new();
    for evidence in threat_evidence {
        extend_limit_causes(&mut causes, evidence.limit_causes.iter().copied());
    }
    causes
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

fn with_limit_causes(
    mut proof: ProofResult,
    causes: impl IntoIterator<Item = ProofLimitCause>,
) -> ProofResult {
    extend_limit_causes(&mut proof.limit_causes, causes);
    proof.limit_hit = !proof.limit_causes.is_empty();
    proof
}

fn corridor_proof_result(
    board: &Board,
    attacker: Color,
    options: &AnalysisOptions,
    status: ProofStatus,
    principal_line: Vec<Move>,
    escape_moves: Vec<Move>,
    threat_evidence: Vec<ThreatSequenceEvidence>,
) -> ProofResult {
    let limit_causes = proof_limit_causes_from_evidence(&threat_evidence);
    let limit_hit = !limit_causes.is_empty() || proof_limit_hit_from_evidence(&threat_evidence);
    ProofResult {
        status,
        attacker,
        side_to_move: board.current_player,
        model: corridor_analysis_model(board, options),
        principal_line,
        escape_moves,
        threat_evidence,
        limit_hit,
        limit_causes,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CorridorReplyStatus {
    Forced,
    ConfirmedEscape,
    PossibleEscape,
    Unknown,
}

struct CorridorReplyOutcome {
    mv: Move,
    status: CorridorReplyStatus,
    proof: ProofResult,
}

fn replay_corridor_status(
    board: &Board,
    actual_moves: &[Move],
    attacker: Color,
    options: &AnalysisOptions,
    prefix_ply: usize,
) -> ProofResult {
    replay_corridor_status_with_actual_child(
        board,
        actual_moves,
        attacker,
        options,
        prefix_ply,
        None,
    )
}

fn replay_corridor_status_with_actual_child(
    board: &Board,
    actual_moves: &[Move],
    attacker: Color,
    options: &AnalysisOptions,
    prefix_ply: usize,
    actual_child: Option<&ProofResult>,
) -> ProofResult {
    match board.result {
        GameResult::Winner(winner) if winner == attacker => {
            return corridor_proof_result(
                board,
                attacker,
                options,
                ProofStatus::ForcedWin,
                Vec::new(),
                Vec::new(),
                Vec::new(),
            );
        }
        GameResult::Winner(_) | GameResult::Draw => {
            return corridor_proof_result(
                board,
                attacker,
                options,
                ProofStatus::EscapeFound,
                Vec::new(),
                Vec::new(),
                Vec::new(),
            );
        }
        GameResult::Ongoing => {}
    }

    if board.current_player == attacker {
        replay_corridor_attacker_node(
            board,
            actual_moves,
            attacker,
            options,
            prefix_ply,
            actual_child,
        )
    } else {
        replay_corridor_defender_node(
            board,
            actual_moves,
            attacker,
            options,
            prefix_ply,
            actual_child,
        )
    }
}

fn replay_corridor_attacker_node(
    board: &Board,
    actual_moves: &[Move],
    attacker: Color,
    options: &AnalysisOptions,
    prefix_ply: usize,
    actual_child: Option<&ProofResult>,
) -> ProofResult {
    let immediate_wins = board.immediate_winning_moves_for(attacker);
    if let Some(&mv) = immediate_wins.first() {
        return corridor_proof_result(
            board,
            attacker,
            options,
            ProofStatus::ForcedWin,
            vec![mv],
            Vec::new(),
            Vec::new(),
        );
    }

    let Some(&actual_move) = actual_moves.get(prefix_ply) else {
        return corridor_proof_result(
            board,
            attacker,
            options,
            ProofStatus::EscapeFound,
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );
    };
    if !is_corridor_attacker_move(board, attacker, actual_move, options) {
        return corridor_proof_result(
            board,
            attacker,
            options,
            ProofStatus::EscapeFound,
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );
    }

    let mut next = board.clone();
    if next.apply_move(actual_move).is_err() {
        return corridor_proof_result(
            board,
            attacker,
            options,
            ProofStatus::Unknown,
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );
    }

    let child = actual_child.cloned().unwrap_or_else(|| {
        replay_corridor_status(&next, actual_moves, attacker, options, prefix_ply + 1)
    });
    match child.status {
        ProofStatus::ForcedWin => {
            let mut principal_line = Vec::with_capacity(child.principal_line.len() + 1);
            principal_line.push(actual_move);
            principal_line.extend(child.principal_line);
            with_limit_causes(
                corridor_proof_result(
                    board,
                    attacker,
                    options,
                    ProofStatus::ForcedWin,
                    principal_line,
                    Vec::new(),
                    child.threat_evidence,
                ),
                child.limit_causes,
            )
        }
        ProofStatus::EscapeFound => with_limit_causes(
            corridor_proof_result(
                board,
                attacker,
                options,
                ProofStatus::EscapeFound,
                Vec::new(),
                child.escape_moves,
                child.threat_evidence,
            ),
            child.limit_causes,
        ),
        ProofStatus::Unknown => {
            let mut causes = child.limit_causes;
            extend_limit_causes(&mut causes, [ProofLimitCause::AttackerChildUnknown]);
            with_limit_causes(
                corridor_proof_result(
                    board,
                    attacker,
                    options,
                    ProofStatus::Unknown,
                    vec![actual_move],
                    Vec::new(),
                    child.threat_evidence,
                ),
                causes,
            )
        }
    }
}

fn replay_corridor_defender_node(
    board: &Board,
    actual_moves: &[Move],
    attacker: Color,
    options: &AnalysisOptions,
    prefix_ply: usize,
    actual_child: Option<&ProofResult>,
) -> ProofResult {
    let include_local_threats = options.defense_policy == DefensePolicy::HybridDefense;
    let threat = ThreatReplySet::new(board, attacker, include_local_threats);
    let attribution = EvidenceAttribution {
        prefix_ply: Some(prefix_ply),
        actual_reply: actual_moves.get(prefix_ply).copied(),
    };

    if !threat.winning_squares.is_empty()
        && threat.legal_cost_squares.is_empty()
        && threat.defender_immediate_wins.is_empty()
    {
        return corridor_proof_result(
            board,
            attacker,
            options,
            ProofStatus::ForcedWin,
            threat
                .winning_squares
                .first()
                .copied()
                .into_iter()
                .collect(),
            Vec::new(),
            vec![threat.evidence(ThreatEvidenceInput {
                attribution,
                reply_classification: ReplyClassification::NoLegalBlock,
                escape_replies: Vec::new(),
                forced_replies: Vec::new(),
                next_forcing_move: threat.winning_squares.first().copied(),
                proof_status: ProofStatus::ForcedWin,
                limit_causes: Vec::new(),
            })],
        );
    }

    let reply_moves =
        corridor_defender_reply_moves(board, actual_moves, prefix_ply, options, &threat);
    if reply_moves.is_empty() {
        return with_limit_causes(
            corridor_proof_result(
                board,
                attacker,
                options,
                ProofStatus::Unknown,
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ),
            [ProofLimitCause::ModelScopeUnknown],
        );
    }
    let mut outcomes = Vec::new();
    for mv in reply_moves {
        if Some(mv) == actual_moves.get(prefix_ply).copied() {
            // The actual replay reply inherits the already-computed next prefix proof.
            // Only alternate replies need fresh branch probes.
            outcomes.push(classify_actual_corridor_reply(
                board,
                actual_moves,
                attacker,
                options,
                prefix_ply,
                mv,
                actual_child,
            ));
        } else {
            outcomes.push(classify_corridor_reply(board, attacker, options, mv));
        }
    }

    let escape_replies = outcomes
        .iter()
        .filter_map(|outcome| {
            (outcome.status == CorridorReplyStatus::ConfirmedEscape).then_some(outcome.mv)
        })
        .collect::<Vec<_>>();
    if !escape_replies.is_empty() {
        let mut evidence = vec![threat.evidence(ThreatEvidenceInput {
            attribution,
            reply_classification: ReplyClassification::ConfirmedEscape,
            escape_replies: escape_replies.clone(),
            forced_replies: forced_corridor_replies(&outcomes),
            next_forcing_move: None,
            proof_status: ProofStatus::EscapeFound,
            limit_causes: Vec::new(),
        })];
        evidence.extend(first_corridor_branch_evidence(
            &outcomes,
            CorridorReplyStatus::ConfirmedEscape,
        ));
        return corridor_proof_result(
            board,
            attacker,
            options,
            ProofStatus::EscapeFound,
            Vec::new(),
            escape_replies.clone(),
            evidence,
        );
    }

    let possible_escape_replies = outcomes
        .iter()
        .filter_map(|outcome| {
            (outcome.status == CorridorReplyStatus::PossibleEscape).then_some(outcome.mv)
        })
        .collect::<Vec<_>>();
    if !possible_escape_replies.is_empty() {
        let mut limit_causes = Vec::new();
        for outcome in outcomes
            .iter()
            .filter(|outcome| outcome.status == CorridorReplyStatus::PossibleEscape)
        {
            extend_limit_causes(
                &mut limit_causes,
                outcome.proof.limit_causes.iter().copied(),
            );
        }
        extend_limit_causes(&mut limit_causes, [ProofLimitCause::DefenderReplyUnknown]);
        let mut evidence = vec![threat.evidence(ThreatEvidenceInput {
            attribution,
            reply_classification: ReplyClassification::PossibleEscape,
            escape_replies: possible_escape_replies.clone(),
            forced_replies: forced_corridor_replies(&outcomes),
            next_forcing_move: None,
            proof_status: ProofStatus::EscapeFound,
            limit_causes: limit_causes.clone(),
        })];
        evidence.extend(first_corridor_branch_evidence(
            &outcomes,
            CorridorReplyStatus::PossibleEscape,
        ));
        return with_limit_causes(
            corridor_proof_result(
                board,
                attacker,
                options,
                ProofStatus::EscapeFound,
                Vec::new(),
                possible_escape_replies,
                evidence,
            ),
            limit_causes,
        );
    }

    let mut limit_causes = Vec::new();
    for outcome in outcomes
        .iter()
        .filter(|outcome| outcome.status == CorridorReplyStatus::Unknown)
    {
        extend_limit_causes(
            &mut limit_causes,
            outcome.proof.limit_causes.iter().copied(),
        );
    }
    if !limit_causes.is_empty() {
        extend_limit_causes(&mut limit_causes, [ProofLimitCause::DefenderReplyUnknown]);
        let principal_line = first_forced_principal_line(&outcomes);
        let mut evidence = vec![threat.evidence(ThreatEvidenceInput {
            attribution,
            reply_classification: ReplyClassification::Unknown,
            escape_replies: Vec::new(),
            forced_replies: forced_corridor_replies(&outcomes),
            next_forcing_move: next_attacker_move_after_defender_reply(&principal_line),
            proof_status: ProofStatus::Unknown,
            limit_causes: vec![ProofLimitCause::DefenderReplyUnknown],
        })];
        evidence.extend(first_corridor_branch_evidence(
            &outcomes,
            CorridorReplyStatus::Unknown,
        ));
        return with_limit_causes(
            corridor_proof_result(
                board,
                attacker,
                options,
                ProofStatus::Unknown,
                principal_line,
                Vec::new(),
                evidence,
            ),
            limit_causes,
        );
    }

    let principal_line = first_forced_principal_line(&outcomes);
    let mut evidence = vec![threat.evidence(ThreatEvidenceInput {
        attribution,
        reply_classification: ReplyClassification::BlockedButForced,
        escape_replies: Vec::new(),
        forced_replies: forced_corridor_replies(&outcomes),
        next_forcing_move: next_attacker_move_after_defender_reply(&principal_line),
        proof_status: ProofStatus::ForcedWin,
        limit_causes: Vec::new(),
    })];
    evidence.extend(first_corridor_branch_evidence(
        &outcomes,
        CorridorReplyStatus::Forced,
    ));
    corridor_proof_result(
        board,
        attacker,
        options,
        ProofStatus::ForcedWin,
        principal_line.clone(),
        Vec::new(),
        evidence,
    )
}

fn classify_corridor_reply(
    board: &Board,
    attacker: Color,
    options: &AnalysisOptions,
    mv: Move,
) -> CorridorReplyOutcome {
    let mut next = board.clone();
    let applied = next.apply_move(mv).is_ok();
    let proof = if !applied {
        with_limit_causes(
            corridor_proof_result(
                board,
                attacker,
                options,
                ProofStatus::Unknown,
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ),
            [ProofLimitCause::ModelScopeUnknown],
        )
    } else {
        let reply_proof = classify_defender_reply_for_report(board, attacker, mv, options);
        let status = match reply_proof.outcome {
            DefenderReplyOutcome::ForcedLoss | DefenderReplyOutcome::ImmediateLoss => {
                ProofStatus::ForcedWin
            }
            DefenderReplyOutcome::ConfirmedEscape | DefenderReplyOutcome::PossibleEscape => {
                ProofStatus::EscapeFound
            }
            DefenderReplyOutcome::Unknown => ProofStatus::Unknown,
        };
        let escape_moves = match reply_proof.outcome {
            DefenderReplyOutcome::ConfirmedEscape | DefenderReplyOutcome::PossibleEscape => {
                vec![mv]
            }
            _ => Vec::new(),
        };
        with_limit_causes(
            corridor_proof_result(
                &next,
                attacker,
                options,
                status,
                reply_proof.principal_line,
                escape_moves,
                Vec::new(),
            ),
            reply_proof.limit_causes,
        )
    };
    let status = match proof.status {
        ProofStatus::ForcedWin => CorridorReplyStatus::Forced,
        ProofStatus::EscapeFound if applied && proof_has_limit_hit(&proof) => {
            CorridorReplyStatus::PossibleEscape
        }
        ProofStatus::EscapeFound => CorridorReplyStatus::ConfirmedEscape,
        ProofStatus::Unknown if applied => CorridorReplyStatus::PossibleEscape,
        ProofStatus::Unknown => CorridorReplyStatus::Unknown,
    };
    CorridorReplyOutcome { mv, status, proof }
}

fn classify_actual_corridor_reply(
    board: &Board,
    actual_moves: &[Move],
    attacker: Color,
    options: &AnalysisOptions,
    prefix_ply: usize,
    mv: Move,
    actual_child: Option<&ProofResult>,
) -> CorridorReplyOutcome {
    let mut next = board.clone();
    if next.apply_move(mv).is_err() {
        return CorridorReplyOutcome {
            mv,
            status: CorridorReplyStatus::Unknown,
            proof: with_limit_causes(
                corridor_proof_result(
                    board,
                    attacker,
                    options,
                    ProofStatus::Unknown,
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                ),
                [ProofLimitCause::ModelScopeUnknown],
            ),
        };
    }

    let proof = actual_child.cloned().unwrap_or_else(|| {
        replay_corridor_status(&next, actual_moves, attacker, options, prefix_ply + 1)
    });
    let status = match proof.status {
        ProofStatus::ForcedWin => CorridorReplyStatus::Forced,
        ProofStatus::EscapeFound => CorridorReplyStatus::ConfirmedEscape,
        ProofStatus::Unknown => CorridorReplyStatus::Unknown,
    };
    CorridorReplyOutcome { mv, status, proof }
}

fn forced_corridor_replies(outcomes: &[CorridorReplyOutcome]) -> Vec<Move> {
    outcomes
        .iter()
        .filter_map(|outcome| (outcome.status == CorridorReplyStatus::Forced).then_some(outcome.mv))
        .collect()
}

fn first_forced_principal_line(outcomes: &[CorridorReplyOutcome]) -> Vec<Move> {
    outcomes
        .iter()
        .find(|outcome| outcome.status == CorridorReplyStatus::Forced)
        .map(|outcome| {
            let mut line = Vec::with_capacity(outcome.proof.principal_line.len() + 1);
            line.push(outcome.mv);
            line.extend(outcome.proof.principal_line.clone());
            line
        })
        .unwrap_or_default()
}

fn first_corridor_branch_evidence(
    outcomes: &[CorridorReplyOutcome],
    status: CorridorReplyStatus,
) -> Vec<ThreatSequenceEvidence> {
    outcomes
        .iter()
        .find(|outcome| outcome.status == status)
        .map(|outcome| outcome.proof.threat_evidence.clone())
        .unwrap_or_default()
}

fn corridor_defender_reply_moves(
    board: &Board,
    actual_moves: &[Move],
    prefix_ply: usize,
    options: &AnalysisOptions,
    threat: &ThreatReplySet,
) -> Vec<Move> {
    let mut replies = Vec::new();
    for mv in threat.legal_cost_squares.iter().copied() {
        push_unique_move(&mut replies, mv);
    }
    for mv in threat.defender_immediate_wins.iter().copied() {
        push_unique_move(&mut replies, mv);
    }

    if threat.winning_squares.is_empty() {
        for mv in imminent_defense_reply_moves(
            board,
            threat.attacker,
            actual_moves.get(prefix_ply).copied(),
        ) {
            push_unique_move(&mut replies, mv);
        }
        for mv in offensive_counter_reply_moves(board, threat.attacker.opponent()) {
            push_unique_move(&mut replies, mv);
        }
        if options.defense_policy == DefensePolicy::HybridDefense
            && use_hybrid_local_threat_replies(threat)
        {
            for mv in threat.local_threat_replies.iter().copied() {
                push_unique_move(&mut replies, mv);
            }
        }
        if let Some(mv) = next_actual_attacker_corridor_move(
            board,
            actual_moves,
            prefix_ply,
            threat.attacker,
            options,
        ) {
            push_unique_move(&mut replies, mv);
        }
    }
    if let Some(&mv) = actual_moves.get(prefix_ply) {
        push_unique_move(&mut replies, mv);
    }

    replies
}

fn next_actual_attacker_corridor_move(
    board: &Board,
    actual_moves: &[Move],
    prefix_ply: usize,
    attacker: Color,
    options: &AnalysisOptions,
) -> Option<Move> {
    let defender = attacker.opponent();
    let defender_reply = actual_moves.get(prefix_ply).copied()?;
    let attacker_move = actual_moves.get(prefix_ply + 1).copied()?;
    if !board.is_legal_for_color(attacker_move, defender) {
        return None;
    }

    let mut next = board.clone();
    next.apply_move(defender_reply).ok()?;
    if next.current_player != attacker {
        return None;
    }
    is_corridor_attacker_move(&next, attacker, attacker_move, options).then_some(attacker_move)
}

fn is_corridor_attacker_move(
    board: &Board,
    attacker: Color,
    mv: Move,
    _options: &AnalysisOptions,
) -> bool {
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

fn materialized_attacker_corridor_moves(board: &Board, attacker: Color) -> Vec<Move> {
    let mut moves = board
        .legal_moves()
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

fn corridor_attacker_move_rank(board: &Board, attacker: Color, mv: Move) -> u8 {
    if board.current_player != attacker || !board.is_legal_for_color(mv, attacker) {
        return 0;
    }
    let mut next = board.clone();
    if next.apply_move(mv).is_err() {
        return 0;
    }
    match next.result {
        GameResult::Winner(winner) if winner == attacker => return 5,
        GameResult::Winner(_) | GameResult::Draw => return 0,
        GameResult::Ongoing => {}
    }
    if !next.immediate_winning_moves_for(attacker).is_empty() {
        return 4;
    }
    local_threat_facts(&next, attacker)
        .into_iter()
        .filter(|fact| local_threat_is_forcing_for(&next, attacker, fact))
        .map(|fact| fact.kind.rank())
        .max()
        .unwrap_or(0)
}

fn use_hybrid_local_threat_replies(threat: &ThreatReplySet) -> bool {
    (1..=MAX_HYBRID_LOCAL_THREAT_COUNT).contains(&threat.local_threat_count)
        && !threat.local_threat_replies.is_empty()
        && threat.local_threat_replies.len() <= MAX_HYBRID_LOCAL_THREAT_REPLIES
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LocalThreatKind {
    OpenFour,
    ClosedFour,
    BrokenFour,
    OpenThree,
    BrokenThree,
}

impl LocalThreatKind {
    fn is_forcing(self) -> bool {
        matches!(
            self,
            Self::OpenFour
                | Self::ClosedFour
                | Self::BrokenFour
                | Self::OpenThree
                | Self::BrokenThree
        )
    }

    fn rank(self) -> u8 {
        match self {
            Self::OpenFour => 4,
            Self::ClosedFour | Self::BrokenFour => 3,
            Self::OpenThree => 2,
            Self::BrokenThree => 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LocalThreatFact {
    kind: LocalThreatKind,
    defense_squares: Vec<Move>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LocalThreatContinuation {
    mv: Move,
    legal_cost_squares: Vec<Move>,
}

fn local_threat_is_forcing_for(board: &Board, attacker: Color, fact: &LocalThreatFact) -> bool {
    fact.kind.is_forcing()
        && !legal_forcing_continuations_for_fact(board, attacker, fact).is_empty()
}

fn legal_forcing_continuations_for_fact(
    board: &Board,
    attacker: Color,
    fact: &LocalThreatFact,
) -> Vec<LocalThreatContinuation> {
    if !fact.kind.is_forcing() {
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

fn local_threat_reply_moves(
    board: &Board,
    defender: Color,
    local_threats: &[LocalThreatFact],
) -> Vec<Move> {
    let mut replies = Vec::new();
    let attacker = defender.opponent();
    for fact in local_threats {
        if !local_threat_is_forcing_for(board, attacker, fact) {
            continue;
        }
        for continuation in legal_forcing_continuations_for_fact(board, attacker, fact) {
            let mv = continuation.mv;
            if board.is_legal_for_color(mv, defender) {
                push_unique_move(&mut replies, mv);
            }
        }
    }
    replies
}

fn has_forcing_local_threat(board: &Board, player: Color) -> bool {
    local_threat_facts(board, player)
        .iter()
        .any(|fact| local_threat_is_forcing_for(board, player, fact))
}

fn local_threat_facts(board: &Board, player: Color) -> Vec<LocalThreatFact> {
    let mut facts = Vec::new();
    board.for_each_occupied_color(player, |row, col| {
        let mv = Move { row, col };
        for &(dr, dc) in &gomoku_core::DIRS {
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
    facts.sort_by_key(|fact| std::cmp::Reverse(fact.kind.rank()));
    facts
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
            kind: LocalThreatKind::OpenFour,
            defense_squares: vec![before.expect("checked open"), after.expect("checked open")],
        }),
        (4, true, false) => Some(LocalThreatFact {
            kind: LocalThreatKind::ClosedFour,
            defense_squares: vec![before.expect("checked open")],
        }),
        (4, false, true) => Some(LocalThreatFact {
            kind: LocalThreatKind::ClosedFour,
            defense_squares: vec![after.expect("checked open")],
        }),
        (3, true, true) => Some(LocalThreatFact {
            kind: LocalThreatKind::OpenThree,
            defense_squares: open_three_defense_squares(
                board,
                start,
                run.len(),
                dr,
                dc,
                before.expect("checked open"),
                after.expect("checked open"),
            )?,
        }),
        _ => None,
    }
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
            kind: LocalThreatKind::BrokenFour,
            defense_squares: completions,
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
        kind: LocalThreatKind::BrokenThree,
        defense_squares: rest_squares,
    })
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
            if virtual_run_len(board, rest, player, dr, dc) < board.config.win_length - 1 {
                continue;
            }
            push_unique_move(&mut rest_squares, rest);
        }
    }

    rest_squares.sort_by_key(|mv| (mv.row, mv.col));
    rest_squares
}

fn virtual_run_len(board: &Board, rest: Move, player: Color, dr: isize, dc: isize) -> usize {
    1 + virtual_count_in_direction(board, rest, player, dr, dc, -1)
        + virtual_count_in_direction(board, rest, player, dr, dc, 1)
}

fn virtual_count_in_direction(
    board: &Board,
    rest: Move,
    player: Color,
    dr: isize,
    dc: isize,
    step: isize,
) -> usize {
    let mut count = 0usize;
    let mut row = rest.row as isize + dr * step;
    let mut col = rest.col as isize + dc * step;
    while in_bounds(board, row, col)
        && has_color_or_virtual_rest(board, row as usize, col as usize, player, rest)
    {
        count += 1;
        row += dr * step;
        col += dc * step;
    }
    count
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
    if !facts.contains(&fact) {
        facts.push(fact);
    }
}

fn replay_prefix_boards(replay: &Replay) -> Result<Vec<Board>, AnalysisError> {
    let mut board = Board::new(replay.rules.clone());
    let mut boards = vec![board.clone()];
    for (idx, replay_move) in replay.moves.iter().enumerate() {
        let ply = idx + 1;
        let mv = Move::from_notation(&replay_move.mv)
            .map_err(|message| AnalysisError::InvalidReplayMove { ply, message })?;
        board
            .apply_move(mv)
            .map_err(|err| AnalysisError::InvalidReplayMove {
                ply,
                message: err.to_string(),
            })?;
        boards.push(board.clone());
    }
    Ok(boards)
}

fn replay_moves(replay: &Replay) -> Result<Vec<Move>, AnalysisError> {
    replay
        .moves
        .iter()
        .enumerate()
        .map(|(idx, replay_move)| {
            Move::from_notation(&replay_move.mv).map_err(|message| {
                AnalysisError::InvalidReplayMove {
                    ply: idx + 1,
                    message,
                }
            })
        })
        .collect()
}

fn replay_winner(replay: &Replay, final_board: &Board) -> Option<Color> {
    match final_board.result {
        GameResult::Winner(winner) => Some(winner),
        _ => match replay.result {
            ReplayResult::BlackWins => Some(Color::Black),
            ReplayResult::WhiteWins => Some(Color::White),
            ReplayResult::Draw | ReplayResult::Ongoing => None,
        },
    }
}

fn replay_proof_summary(
    boards: &[Board],
    actual_moves: &[Move],
    winner: Color,
    options: &AnalysisOptions,
    scan_start: usize,
) -> Vec<ProofResult> {
    let mut proof_summary = Vec::with_capacity(boards.len() - scan_start);
    let mut actual_child = None;
    for ply in (scan_start..boards.len()).rev() {
        let proof = replay_corridor_status_with_actual_child(
            &boards[ply],
            actual_moves,
            winner,
            options,
            ply,
            actual_child.as_ref(),
        );
        actual_child = Some(proof.clone());
        proof_summary.push(proof);
    }
    proof_summary.reverse();
    proof_summary
}

fn replay_proof_summary_until_boundary(
    boards: &[Board],
    actual_moves: &[Move],
    winner: Color,
    options: &AnalysisOptions,
) -> (usize, Vec<ProofResult>) {
    let Some(max_scan_plies) = options.max_scan_plies else {
        return (
            0,
            replay_proof_summary(boards, actual_moves, winner, options, 0),
        );
    };

    let lower_bound = boards.len().saturating_sub(max_scan_plies + 1);
    let mut scan_start = boards.len();
    let mut proof_summary = Vec::new();
    let mut actual_child = None;

    for ply in (lower_bound..boards.len()).rev() {
        let proof = replay_corridor_status_with_actual_child(
            &boards[ply],
            actual_moves,
            winner,
            options,
            ply,
            actual_child.as_ref(),
        );
        actual_child = Some(proof.clone());
        proof_summary.insert(0, proof);
        scan_start = ply;

        if final_forced_interval_has_boundary(&proof_summary, scan_start, actual_moves.len()) {
            break;
        }
    }

    (scan_start, proof_summary)
}

fn final_forced_interval_has_boundary(
    proof_summary: &[ProofResult],
    scan_start: usize,
    move_count: usize,
) -> bool {
    let proof_intervals = proof_intervals(proof_summary, scan_start);
    let (found, interval) = find_final_forced_interval(&proof_intervals, move_count);
    found && interval.start_ply > scan_start
}

fn find_final_forced_interval(
    proof_intervals: &[ForcedInterval],
    move_count: usize,
) -> (bool, ForcedInterval) {
    let found = proof_intervals
        .iter()
        .any(|interval| interval.end_ply == move_count);
    let interval = proof_intervals
        .iter()
        .rev()
        .find(|interval| interval.end_ply == move_count)
        .cloned()
        .unwrap_or(ForcedInterval {
            start_ply: move_count,
            end_ply: move_count,
        });
    (found, interval)
}

fn proof_intervals(proofs: &[ProofResult], scan_start: usize) -> Vec<ForcedInterval> {
    let mut intervals = Vec::new();
    let mut current_start = None;
    for (idx, proof) in proofs.iter().enumerate() {
        let ply = scan_start + idx;
        if proof.status == ProofStatus::ForcedWin {
            current_start.get_or_insert(ply);
        } else if let Some(start) = current_start.take() {
            intervals.push(ForcedInterval {
                start_ply: start,
                end_ply: ply - 1,
            });
        }
    }
    if let Some(start) = current_start {
        intervals.push(ForcedInterval {
            start_ply: start,
            end_ply: scan_start + proofs.len() - 1,
        });
    }
    intervals
}

fn proof_at(proofs: &[ProofResult], scan_start: usize, ply: usize) -> Option<&ProofResult> {
    proofs.get(ply.checked_sub(scan_start)?)
}

fn find_last_chance(
    boards: &[Board],
    proofs: &[ProofResult],
    scan_start: usize,
    before_ply: usize,
    loser: Option<Color>,
) -> Option<usize> {
    let loser = loser?;
    (scan_start..before_ply).rev().find(|&ply| {
        boards[ply].current_player == loser
            && proof_at(proofs, scan_start, ply)
                .is_some_and(|proof| proof.status == ProofStatus::EscapeFound)
    })
}

fn classify_root_cause(
    previous_status: Option<ProofStatus>,
    move_color: Option<Color>,
    winner: Color,
    missed_win_root: bool,
) -> RootCause {
    if missed_win_root {
        return RootCause::MissedWin;
    }
    match (previous_status, move_color) {
        (Some(ProofStatus::EscapeFound), Some(color)) if color == winner.opponent() => {
            RootCause::MissedDefense
        }
        (Some(ProofStatus::EscapeFound), Some(color)) if color == winner => {
            RootCause::StrategicLoss
        }
        _ => RootCause::Unclear,
    }
}

struct UnclearReasonInput<'a> {
    root_cause: RootCause,
    final_forced_interval_found: bool,
    final_forced_interval: &'a ForcedInterval,
    previous_status: Option<ProofStatus>,
    proof_summary: &'a [ProofResult],
    scan_start: usize,
}

fn unclear_reason(input: UnclearReasonInput<'_>) -> Option<UnclearReason> {
    if input.root_cause != RootCause::Unclear {
        return None;
    }
    if !input.final_forced_interval_found {
        return Some(UnclearReason::NoFinalForcedInterval);
    }
    let previous_ply = input.final_forced_interval.start_ply.checked_sub(1);
    let Some(previous_ply) = previous_ply else {
        return Some(UnclearReason::ScanWindowCutoff);
    };
    let Some(previous_proof) = proof_at(input.proof_summary, input.scan_start, previous_ply) else {
        return Some(UnclearReason::ScanWindowCutoff);
    };
    match input.previous_status {
        Some(ProofStatus::Unknown) if proof_has_limit_hit(previous_proof) => {
            Some(UnclearReason::ProofLimitHit)
        }
        Some(ProofStatus::Unknown) => Some(UnclearReason::PreviousPrefixUnknown),
        None => Some(UnclearReason::ScanWindowCutoff),
        _ => Some(UnclearReason::PreviousPrefixUnknown),
    }
}

fn proof_has_limit_hit(proof: &ProofResult) -> bool {
    proof.limit_hit || !proof.limit_causes.is_empty()
}

struct UnclearContextInput<'a> {
    root_cause: RootCause,
    unclear_reason: Option<UnclearReason>,
    final_forced_interval: &'a ForcedInterval,
    proof_summary: &'a [ProofResult],
    scan_start: usize,
    boards: &'a [Board],
    winner: Color,
    principal_line: &'a [Move],
    move_count: usize,
}

fn unclear_context(input: UnclearContextInput<'_>) -> Option<UnclearContext> {
    if input.root_cause != RootCause::Unclear {
        return None;
    }
    let reason = input.unclear_reason?;
    if reason == UnclearReason::DrawOrOngoing {
        return None;
    }

    let previous_prefix_ply = input.final_forced_interval.start_ply.checked_sub(1);
    let previous_proof =
        previous_prefix_ply.and_then(|ply| proof_at(input.proof_summary, input.scan_start, ply));
    let previous_limit_causes = previous_proof
        .map(|proof| proof.limit_causes.clone())
        .unwrap_or_else(|| vec![ProofLimitCause::OutsideScanWindow]);
    let previous_board = previous_prefix_ply.and_then(|ply| input.boards.get(ply));
    let mut snapshots = Vec::new();
    if let (Some(ply), Some(board)) = (previous_prefix_ply, previous_board) {
        snapshots.push(board_snapshot("previous_prefix", ply, board));
    }
    if snapshots
        .iter()
        .all(|snapshot| snapshot.ply != input.final_forced_interval.start_ply)
    {
        if let Some(board) = input.boards.get(input.final_forced_interval.start_ply) {
            snapshots.push(board_snapshot(
                "final_forced_start",
                input.final_forced_interval.start_ply,
                board,
            ));
        }
    }

    Some(UnclearContext {
        reason,
        previous_prefix_ply,
        final_forced_interval: input.final_forced_interval.clone(),
        previous_proof_status: previous_proof.map(|proof| proof.status),
        previous_proof_limit_hit: previous_proof.map(proof_has_limit_hit),
        previous_limit_causes,
        previous_side_to_move: previous_board.map(|board| board.current_player),
        winner: input.winner,
        principal_line: input.principal_line.to_vec(),
        principal_line_notation: input
            .principal_line
            .iter()
            .map(|mv| mv.to_notation())
            .collect(),
        scan_start_ply: input.scan_start,
        scan_end_ply: if input.proof_summary.is_empty() {
            None
        } else {
            Some(input.scan_start + input.proof_summary.len() - 1)
        },
        move_count: input.move_count,
        snapshots,
    })
}

fn board_snapshot(label: &str, ply: usize, board: &Board) -> AnalysisBoardSnapshot {
    AnalysisBoardSnapshot {
        label: label.to_string(),
        ply,
        side_to_move: board.current_player,
        rows: board_rows(board),
    }
}

fn board_rows(board: &Board) -> Vec<String> {
    let size = board.config.board_size;
    (0..size)
        .map(|row| {
            (0..size)
                .map(|col| board.cell(row, col).map_or('.', Color::to_char))
                .collect()
        })
        .collect()
}

struct TacticalNoteInput<'a> {
    replay: &'a Replay,
    boards: &'a [Board],
    proofs: &'a [ProofResult],
    scan_start: usize,
    proof_intervals: &'a [ForcedInterval],
    final_forced_interval: &'a ForcedInterval,
    winner: Color,
    root_cause: RootCause,
}

fn tactical_notes(input: TacticalNoteInput<'_>) -> Vec<TacticalNote> {
    let mut notes = Vec::new();
    if input.root_cause == RootCause::MissedDefense {
        push_note(&mut notes, TacticalNote::AccidentalBlunder);
    }
    if input.root_cause == RootCause::MissedWin {
        push_note(&mut notes, TacticalNote::MissedWin);
    }
    if input
        .proof_intervals
        .iter()
        .any(|interval| interval.end_ply < input.final_forced_interval.start_ply)
    {
        push_note(&mut notes, TacticalNote::ConversionError);
    }
    if missed_forced_win(
        input.replay,
        input.boards,
        input.proofs,
        input.scan_start,
        input.winner,
    ) {
        push_note(&mut notes, TacticalNote::MissedWin);
    }
    if input.root_cause == RootCause::StrategicLoss {
        push_note(&mut notes, TacticalNote::StrongAttack);
    }
    notes
}

fn losing_side_missed_immediate_win(
    replay: &Replay,
    boards: &[Board],
    forced_start_ply: usize,
    loser: Color,
) -> bool {
    let Some(prefix_ply) = forced_start_ply.checked_sub(1) else {
        return false;
    };
    let Some(board) = boards.get(prefix_ply) else {
        return false;
    };
    if board.current_player != loser {
        return false;
    }
    let immediate_wins = board.immediate_winning_moves_for(loser);
    if immediate_wins.is_empty() {
        return false;
    }
    let Some(actual) = replay.moves.get(prefix_ply) else {
        return false;
    };
    let Ok(actual_move) = Move::from_notation(&actual.mv) else {
        return false;
    };
    !immediate_wins.contains(&actual_move)
}

fn missed_forced_win(
    replay: &Replay,
    boards: &[Board],
    proofs: &[ProofResult],
    scan_start: usize,
    winner: Color,
) -> bool {
    for (ply, board) in boards
        .iter()
        .enumerate()
        .take(replay.moves.len())
        .skip(scan_start)
    {
        if board.current_player != winner {
            continue;
        }
        if !proof_at(proofs, scan_start, ply)
            .is_some_and(|proof| proof.status == ProofStatus::ForcedWin)
        {
            continue;
        }
        let immediate_wins = board.immediate_winning_moves_for(winner);
        if immediate_wins.is_empty() {
            continue;
        }
        let Ok(actual_move) = Move::from_notation(&replay.moves[ply].mv) else {
            continue;
        };
        if !immediate_wins.contains(&actual_move) {
            return true;
        }
    }
    false
}

fn push_note(notes: &mut Vec<TacticalNote>, note: TacticalNote) {
    if !notes.contains(&note) {
        notes.push(note);
    }
}

fn color_for_ply(ply: usize) -> Option<Color> {
    if ply == 0 {
        None
    } else if ply % 2 == 1 {
        Some(Color::Black)
    } else {
        Some(Color::White)
    }
}

pub(crate) fn corridor_analysis_model(board: &Board, options: &AnalysisOptions) -> AnalysisModel {
    AnalysisModel {
        defense_policy: options.defense_policy,
        tactical_reply_coverage: corridor_tactical_reply_coverage(options.defense_policy),
        attacker_move_policy: corridor_attacker_move_policy(options.defense_policy).to_string(),
        rule_set: rule_label(&board.config.variant).to_string(),
        max_depth: options.max_depth,
        max_scan_plies: options.max_scan_plies,
    }
}

fn corridor_attacker_move_policy(policy: DefensePolicy) -> &'static str {
    match policy {
        DefensePolicy::AllLegalDefense | DefensePolicy::TacticalDefense => {
            "actual_corridor_moves; immediate_wins; local_threat_materialization; corridor_depth"
        }
        DefensePolicy::HybridDefense => {
            "actual_corridor_moves; immediate_wins; local_threat_materialization; bounded_local_threats; corridor_depth"
        }
    }
}

fn corridor_tactical_reply_coverage(policy: DefensePolicy) -> Vec<String> {
    let mut coverage = vec![
        "corridor_exit_a_filter".to_string(),
        "legal_cost_replies".to_string(),
        "defender_immediate_wins".to_string(),
        "next_actual_attacker_move".to_string(),
        "forbidden_cost_squares".to_string(),
    ];
    if policy == DefensePolicy::HybridDefense {
        coverage.push("local_threat_replies".to_string());
    }
    coverage
}

pub(crate) fn rule_label(variant: &Variant) -> &'static str {
    match variant {
        Variant::Freestyle => "freestyle",
        Variant::Renju => "renju",
    }
}

#[cfg(test)]
mod tests {
    use gomoku_core::{Board, Color, Move, Replay, RuleConfig, Variant};

    use super::{
        analyze_defender_reply_options, analyze_replay, has_forcing_local_threat,
        legal_forcing_continuations_for_fact, local_threat_facts, replay_moves,
        replay_prefix_boards, replay_proof_summary, AnalysisOptions, DefenderReplyOutcome,
        DefenderReplyRole, DefensePolicy, LocalThreatFact, LocalThreatKind, ProofLimitCause,
        ProofStatus, ReplyClassification, RootCause, TacticalNote, UnclearReason,
    };

    fn mv(notation: &str) -> Move {
        Move::from_notation(notation).expect("test move notation should parse")
    }

    fn replay_from_moves(variant: Variant, moves: &[&str]) -> Replay {
        let rules = RuleConfig {
            variant,
            ..RuleConfig::default()
        };
        let mut board = Board::new(rules.clone());
        let mut replay = Replay::new(rules, "Black", "White");

        for notation in moves {
            let parsed = mv(notation);
            board.apply_move(parsed).expect("fixture move should apply");
            replay.push_move(parsed, 0, board.hash(), None);
        }
        replay.finish(&board.result, Some(0));
        replay
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

    #[test]
    fn defender_reply_options_distinguish_forced_loss_from_unproven_counterplay() {
        let board = board_from_moves(
            Variant::Renju,
            &[
                "H8", "I8", "H7", "I7", "H6", "H5", "I6", "I9", "G6", "J6", "G8", "J5", "G5",
            ],
        );
        let options = AnalysisOptions {
            defense_policy: DefensePolicy::AllLegalDefense,
            max_depth: 4,
            max_scan_plies: Some(8),
        };
        let replies =
            analyze_defender_reply_options(&board, Color::Black, Some(mv("G7")), &options);

        for notation in ["G4", "G7", "G9"] {
            let reply = reply_for(&replies, notation);
            assert!(reply.roles.contains(&DefenderReplyRole::ImminentDefense));
            assert_eq!(
                reply.outcome,
                DefenderReplyOutcome::ForcedLoss,
                "{notation}: line {:?} limits {:?}",
                reply.principal_line_notation,
                reply.limit_causes
            );
        }

        let i10 = reply_for(&replies, "I10");
        assert!(i10.roles.contains(&DefenderReplyRole::OffensiveCounter));
        assert_eq!(
            i10.outcome,
            DefenderReplyOutcome::PossibleEscape,
            "I10: line {:?} limits {:?}",
            i10.principal_line_notation,
            i10.limit_causes
        );
        assert!(i10.limit_causes.contains(&ProofLimitCause::DepthCutoff));

        let i11 = reply_for(&replies, "I11");
        assert!(i11.roles.contains(&DefenderReplyRole::OffensiveCounter));
        assert_eq!(
            i11.outcome,
            DefenderReplyOutcome::ForcedLoss,
            "I11: line {:?} limits {:?}",
            i11.principal_line_notation,
            i11.limit_causes
        );
    }

    #[test]
    fn imminent_open_three_defense_excludes_outer_cost_squares() {
        let board = board_from_moves(
            Variant::Renju,
            &[
                "H8", "I8", "H7", "I7", "H6", "H5", "I6", "I9", "G6", "J6", "G8",
            ],
        );
        let options = AnalysisOptions {
            defense_policy: DefensePolicy::AllLegalDefense,
            max_depth: 4,
            max_scan_plies: Some(8),
        };
        let replies =
            analyze_defender_reply_options(&board, Color::Black, Some(mv("J5")), &options);

        for notation in ["J5", "F9"] {
            let reply = reply_for(&replies, notation);
            assert!(
                reply.roles.contains(&DefenderReplyRole::ImminentDefense),
                "{notation}: roles {:?}",
                reply.roles
            );
        }
        for notation in ["E10", "K4"] {
            assert!(
                replies.iter().all(|reply| reply.notation != notation),
                "{notation} should not be a direct defense to the open three"
            );
        }
    }

    #[test]
    fn open_three_with_blocked_outer_side_includes_far_defense_square() {
        let board = board_from_moves(Variant::Renju, &["J9", "H9", "K9", "A1", "L9"]);
        assert!(
            local_threat_facts(&board, Color::Black).contains(&LocalThreatFact {
                kind: LocalThreatKind::OpenThree,
                defense_squares: vec![mv("I9"), mv("M9"), mv("N9")],
            })
        );

        let options = AnalysisOptions {
            defense_policy: DefensePolicy::AllLegalDefense,
            max_depth: 4,
            max_scan_plies: Some(8),
        };
        let replies =
            analyze_defender_reply_options(&board, Color::Black, Some(mv("N9")), &options);
        let reply = reply_for(&replies, "N9");
        assert!(reply.roles.contains(&DefenderReplyRole::ImminentDefense));
    }

    #[test]
    fn boxed_three_is_not_a_forcing_open_three() {
        let board = board_from_moves(Variant::Renju, &["J9", "H9", "K9", "N9", "L9"]);
        assert!(
            local_threat_facts(&board, Color::Black)
                .iter()
                .all(|fact| fact.kind != LocalThreatKind::OpenThree),
            "{:?}",
            local_threat_facts(&board, Color::Black)
        );

        let options = AnalysisOptions {
            defense_policy: DefensePolicy::AllLegalDefense,
            max_depth: 4,
            max_scan_plies: Some(8),
        };
        let replies = analyze_defender_reply_options(&board, Color::Black, None, &options);
        for notation in ["I9", "M9"] {
            assert!(
                replies
                    .iter()
                    .filter(|reply| reply.notation == notation)
                    .all(|reply| !reply.roles.contains(&DefenderReplyRole::ImminentDefense)),
                "{notation}: {:?}",
                replies
            );
        }
    }

    #[test]
    fn renju_forbidden_only_black_local_threat_is_not_forcing() {
        let board = board_from_moves(
            Variant::Renju,
            &["H8", "G8", "I8", "A1", "J8", "A2", "L8", "A3", "M8"],
        );
        assert!(!board.is_legal_for_color(mv("K8"), Color::Black));
        assert!(!board
            .immediate_winning_moves_for(Color::Black)
            .contains(&mv("K8")));

        let facts = local_threat_facts(&board, Color::Black);
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
        assert!(
            !has_forcing_local_threat(&board, Color::Black),
            "unexpected forcing fact remains: {facts:?}"
        );
    }

    #[test]
    fn renju_mixed_black_local_threat_uses_only_legal_continuations() {
        let board = board_from_moves(
            Variant::Renju,
            &["H8", "A1", "I8", "A2", "J8", "A3", "K8", "A4", "M8"],
        );
        assert!(board.is_legal_for_color(mv("G8"), Color::Black));
        assert!(!board.is_legal_for_color(mv("L8"), Color::Black));

        let facts = local_threat_facts(&board, Color::Black);
        let mixed_open_four = facts
            .iter()
            .find(|fact| {
                fact.kind == LocalThreatKind::OpenFour
                    && fact.defense_squares == vec![mv("G8"), mv("L8")]
            })
            .unwrap_or_else(|| panic!("expected raw mixed open-four fact: {facts:?}"));
        let continuations =
            legal_forcing_continuations_for_fact(&board, Color::Black, mixed_open_four);
        assert_eq!(
            continuations
                .iter()
                .map(|continuation| continuation.mv)
                .collect::<Vec<_>>(),
            vec![mv("G8")]
        );
        assert!(has_forcing_local_threat(&board, Color::Black));
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
    fn all_legal_defense_finds_escape_for_single_closed_four() {
        let board = board_from_moves(
            Variant::Freestyle,
            &["H8", "G8", "I8", "A1", "J8", "A2", "K8"],
        );
        let replies = analyze_defender_reply_options(
            &board,
            Color::Black,
            None,
            &AnalysisOptions {
                defense_policy: DefensePolicy::AllLegalDefense,
                max_depth: 4,
                ..AnalysisOptions::default()
            },
        );

        assert_eq!(
            reply_for(&replies, "L8").outcome,
            DefenderReplyOutcome::ConfirmedEscape
        );
    }

    #[test]
    fn all_legal_defense_proves_open_four_even_if_one_end_is_blocked() {
        let board = board_from_moves(
            Variant::Freestyle,
            &["H8", "A1", "I8", "A2", "J8", "A3", "K8"],
        );
        let replies = analyze_defender_reply_options(
            &board,
            Color::Black,
            None,
            &AnalysisOptions {
                defense_policy: DefensePolicy::AllLegalDefense,
                max_depth: 4,
                ..AnalysisOptions::default()
            },
        );

        assert_eq!(
            reply_for(&replies, "G8").outcome,
            DefenderReplyOutcome::ImmediateLoss
        );
        assert_eq!(
            reply_for(&replies, "L8").outcome,
            DefenderReplyOutcome::ImmediateLoss
        );
    }

    #[test]
    fn corridor_depth_proves_closed_four_block_into_open_four() {
        let replay = replay_from_moves(
            Variant::Freestyle,
            &[
                "H8", "G8", "I8", "A1", "J8", "C1", "K6", "E1", "K7", "G1", "K8", "L8", "K9", "K5",
                "K10",
            ],
        );

        let analysis = analyze_replay(
            &replay,
            AnalysisOptions {
                defense_policy: DefensePolicy::AllLegalDefense,
                max_depth: 4,
                max_scan_plies: Some(6),
            },
        )
        .expect("forced chain replay should analyze");

        assert_eq!(analysis.final_forced_interval.start_ply, 10);
        assert_eq!(analysis.root_cause, RootCause::MissedDefense);
        assert!(
            analysis
                .proof_summary
                .iter()
                .flat_map(|proof| proof.threat_evidence.iter())
                .any(|evidence| evidence.reply_classification
                    == ReplyClassification::BlockedButForced)
        );
    }

    #[test]
    fn corridor_depth_cutoff_is_possible_escape() {
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
            &AnalysisOptions {
                defense_policy: DefensePolicy::AllLegalDefense,
                max_depth: 0,
                ..AnalysisOptions::default()
            },
        );

        let reply = reply_for(&replies, "L8");
        assert_eq!(reply.outcome, DefenderReplyOutcome::PossibleEscape);
        assert!(reply.limit_causes.contains(&ProofLimitCause::DepthCutoff));
    }

    #[test]
    fn tactical_defense_allows_defender_immediate_win_escape() {
        let board = board_from_moves(
            Variant::Freestyle,
            &["A1", "H8", "A2", "I8", "A3", "J8", "A4", "K8"],
        );
        let replies = analyze_defender_reply_options(
            &board,
            Color::White,
            Some(mv("A5")),
            &AnalysisOptions {
                defense_policy: DefensePolicy::TacticalDefense,
                max_depth: 4,
                ..AnalysisOptions::default()
            },
        );

        assert_eq!(
            reply_for(&replies, "A5").outcome,
            DefenderReplyOutcome::ConfirmedEscape
        );
    }

    #[test]
    fn tactical_defense_proves_renju_single_square_with_forbidden_block() {
        let replay = replay_from_moves(
            Variant::Renju,
            &[
                "C3", "D4", "H6", "E5", "H7", "F6", "F8", "G7", "G8", "A15", "A14", "H8",
            ],
        );

        let analysis = analyze_replay(
            &replay,
            AnalysisOptions {
                defense_policy: DefensePolicy::TacticalDefense,
                max_depth: 4,
                max_scan_plies: Some(2),
            },
        )
        .expect("renju fixture should analyze");

        assert!(analysis
            .proof_summary
            .iter()
            .flat_map(|proof| proof.threat_evidence.iter())
            .any(|evidence| {
                evidence.reply_classification == ReplyClassification::NoLegalBlock
                    && evidence.raw_cost_squares == vec![mv("H8")]
                    && evidence.legal_cost_squares.is_empty()
                    && evidence.illegal_cost_squares == vec![mv("H8")]
            }));
    }

    #[test]
    fn hybrid_defense_proves_double_open_three_with_local_replies() {
        let replay = replay_from_moves(
            Variant::Freestyle,
            &[
                "H8", "A1", "I8", "C2", "J6", "E3", "J7", "G4", "J8", "H7", "K8",
            ],
        );

        let analysis = analyze_replay(
            &replay,
            AnalysisOptions {
                defense_policy: DefensePolicy::HybridDefense,
                max_depth: 4,
                max_scan_plies: Some(4),
            },
        )
        .expect("hybrid local-threat replay should analyze");

        assert!(analysis
            .model
            .tactical_reply_coverage
            .contains(&"local_threat_replies".to_string()));
        assert!(analysis
            .proof_summary
            .iter()
            .all(
                |proof| proof.threat_evidence.iter().all(|evidence| !evidence
                    .limit_causes
                    .contains(&ProofLimitCause::ModelScopeUnknown))
            ));
    }

    #[test]
    fn local_threat_facts_report_broken_four_and_broken_three() {
        let broken_four_board = board_from_moves(
            Variant::Freestyle,
            &["H8", "A1", "I8", "C1", "L8", "E1", "J8"],
        );
        assert!(
            local_threat_facts(&broken_four_board, Color::Black).contains(&LocalThreatFact {
                kind: LocalThreatKind::BrokenFour,
                defense_squares: vec![mv("K8")],
            })
        );

        let broken_three_board =
            board_from_moves(Variant::Freestyle, &["H8", "A1", "K8", "C1", "J8"]);
        assert!(
            local_threat_facts(&broken_three_board, Color::Black).contains(&LocalThreatFact {
                kind: LocalThreatKind::BrokenThree,
                defense_squares: vec![mv("I8")],
            })
        );
        assert!(LocalThreatKind::BrokenThree.is_forcing());
    }

    #[test]
    fn replay_analysis_labels_missed_defense_without_overclaiming_previous_position() {
        let replay = replay_from_moves(
            Variant::Freestyle,
            &["H8", "G8", "I8", "A1", "J8", "A2", "K8", "B1", "L8"],
        );

        let analysis = analyze_replay(
            &replay,
            AnalysisOptions {
                defense_policy: DefensePolicy::AllLegalDefense,
                max_depth: 2,
                max_scan_plies: None,
            },
        )
        .expect("finished replay should analyze");

        assert_eq!(analysis.winner, Some(Color::Black));
        assert_eq!(analysis.final_forced_interval.start_ply, 8);
        assert_eq!(analysis.last_chance_ply, Some(7));
        assert_eq!(analysis.critical_mistake_ply, Some(8));
        assert_eq!(analysis.root_cause, RootCause::MissedDefense);
        assert!(analysis
            .tactical_notes
            .contains(&TacticalNote::AccidentalBlunder));
    }

    #[test]
    fn replay_analysis_stops_at_possible_escape_boundary() {
        let replay = replay_from_moves(
            Variant::Renju,
            &[
                "H8", "I8", "H7", "I7", "H6", "H5", "I6", "I9", "G6", "J6", "G8", "J5", "G5", "G7",
                "E6", "F6", "H9", "H10", "F7", "D5", "I10",
            ],
        );

        let analysis = analyze_replay(
            &replay,
            AnalysisOptions {
                defense_policy: DefensePolicy::AllLegalDefense,
                max_depth: 4,
                max_scan_plies: Some(8),
            },
        )
        .expect("finished replay should analyze");

        assert_eq!(analysis.winner, Some(Color::Black));
        assert_eq!(analysis.root_cause, RootCause::MissedDefense);
        assert_eq!(analysis.final_forced_interval.start_ply, 14);
        assert_eq!(analysis.last_chance_ply, Some(13));
        assert_eq!(analysis.critical_mistake_ply, Some(14));

        let scan_start = replay.moves.len() + 1 - analysis.proof_summary.len();
        assert_eq!(scan_start, 13);
        let boundary = analysis
            .proof_summary
            .get(13 - scan_start)
            .expect("escape boundary proof should be within the scan cap");
        assert_eq!(boundary.status, ProofStatus::EscapeFound);
        assert!(boundary.threat_evidence.iter().any(|evidence| {
            evidence.reply_classification == ReplyClassification::PossibleEscape
                && evidence.escape_replies.contains(&mv("I10"))
        }));
    }

    #[test]
    fn replay_analysis_does_not_label_actual_immediate_block_as_escape() {
        let replay = replay_from_moves(
            Variant::Renju,
            &[
                "H8", "H7", "F8", "G9", "G8", "I8", "J9", "I7", "F7", "I10", "I9", "F6", "K9",
                "H9", "J7", "H10", "L9", "M9", "I6", "K8", "E8", "D8", "G6", "H5", "J6", "J10",
                "K10", "G5", "H4", "F11", "G10", "H6", "J8", "J5", "I5", "K7", "J11", "I12", "F9",
                "H11", "D7", "C6", "E7", "C7", "E9", "E6", "D9", "C10", "C9", "B9", "D10", "C11",
                "E11", "E10", "F12", "B8", "G13",
            ],
        );
        let prefix_ply = 49;
        let boards = replay_prefix_boards(&replay).expect("fixture replay should apply");
        let actual_moves = replay_moves(&replay).expect("fixture replay should parse");
        let proof_summary = replay_proof_summary(
            &boards,
            &actual_moves,
            Color::Black,
            &AnalysisOptions {
                defense_policy: DefensePolicy::AllLegalDefense,
                max_depth: 4,
                max_scan_plies: Some(8),
            },
            prefix_ply,
        );
        let proof = proof_summary
            .first()
            .expect("target prefix should be in proof summary");

        assert_eq!(proof.status, ProofStatus::ForcedWin);
        assert!(proof
            .threat_evidence
            .iter()
            .any(|evidence| evidence.actual_reply == Some(mv("B9"))
                && evidence.reply_classification == ReplyClassification::BlockedButForced
                && evidence.escape_replies.is_empty()));
    }

    #[test]
    fn replay_analysis_attaches_actual_reply_to_actual_line_evidence() {
        let replay = replay_from_moves(
            Variant::Freestyle,
            &["H8", "G8", "I8", "A1", "J8", "A2", "K8", "B1", "L8"],
        );

        let analysis = analyze_replay(
            &replay,
            AnalysisOptions {
                defense_policy: DefensePolicy::AllLegalDefense,
                max_depth: 2,
                max_scan_plies: None,
            },
        )
        .expect("finished replay should analyze");

        let proof = analysis
            .proof_summary
            .get(7)
            .expect("ply 7 proof should be scanned");
        let evidence = proof
            .threat_evidence
            .first()
            .expect("ply 7 immediate threat should be explained");
        assert_eq!(evidence.prefix_ply, Some(7));
        assert_eq!(evidence.actual_reply, Some(mv("B1")));
    }

    #[test]
    fn replay_analysis_tracks_conversion_error_before_final_interval() {
        let replay = replay_from_moves(
            Variant::Freestyle,
            &[
                "H8", "G8", "I8", "A1", "J8", "A2", "K8", "B1", "C1", "B2", "L8",
            ],
        );

        let analysis = analyze_replay(
            &replay,
            AnalysisOptions {
                defense_policy: DefensePolicy::AllLegalDefense,
                max_depth: 2,
                max_scan_plies: None,
            },
        )
        .expect("finished replay should analyze");

        assert_eq!(analysis.final_forced_interval.start_ply, 10);
        assert_eq!(analysis.root_cause, RootCause::MissedDefense);
        assert!(analysis
            .tactical_notes
            .contains(&TacticalNote::ConversionError));
        assert!(analysis.tactical_notes.contains(&TacticalNote::MissedWin));
    }

    #[test]
    fn replay_analysis_labels_losing_side_missed_win_as_root_cause() {
        let replay = replay_from_moves(
            Variant::Freestyle,
            &[
                "A1", "H8", "A2", "I8", "A3", "J8", "B1", "K8", "A4", "C1", "A5",
            ],
        );

        let analysis = analyze_replay(
            &replay,
            AnalysisOptions {
                defense_policy: DefensePolicy::AllLegalDefense,
                max_depth: 2,
                ..AnalysisOptions::default()
            },
        )
        .expect("finished replay should analyze");

        assert_eq!(analysis.winner, Some(Color::Black));
        assert_eq!(analysis.root_cause, RootCause::MissedWin);
        assert_eq!(analysis.critical_mistake_ply, Some(10));
        assert!(analysis.tactical_notes.contains(&TacticalNote::MissedWin));
    }

    #[test]
    fn shallow_corridor_analysis_finds_open_four_point_of_no_return() {
        let replay = replay_from_moves(
            Variant::Freestyle,
            &["H8", "A1", "I8", "A2", "J8", "A3", "K8", "B1", "L8"],
        );

        let analysis = analyze_replay(
            &replay,
            AnalysisOptions {
                defense_policy: DefensePolicy::AllLegalDefense,
                max_depth: 1,
                max_scan_plies: Some(4),
            },
        )
        .expect("finished replay should analyze");

        assert_eq!(analysis.root_cause, RootCause::MissedDefense);
        assert!(analysis.final_forced_interval_found);
        assert_eq!(analysis.final_forced_interval.start_ply, 6);
        assert_eq!(analysis.last_chance_ply, Some(5));
    }

    #[test]
    fn ongoing_replay_has_no_winner_and_unknown_root_cause() {
        let replay = replay_from_moves(Variant::Freestyle, &["H8", "A1", "I8"]);

        let analysis = analyze_replay(&replay, AnalysisOptions::default())
            .expect("ongoing replay should still produce a bounded summary");

        assert_eq!(analysis.winner, None);
        assert_eq!(analysis.root_cause, RootCause::Unclear);
        assert_eq!(analysis.unclear_reason, Some(UnclearReason::DrawOrOngoing));
        assert!(!analysis.final_forced_interval_found);
        assert_eq!(analysis.final_forced_interval.start_ply, 0);
        assert_eq!(analysis.final_forced_interval.end_ply, 0);
    }

    #[test]
    fn renju_forbidden_defense_remains_model_visible() {
        let replay = replay_from_moves(Variant::Renju, &["H8", "A1"]);

        let analysis = analyze_replay(
            &replay,
            AnalysisOptions {
                defense_policy: DefensePolicy::TacticalDefense,
                max_depth: 4,
                ..AnalysisOptions::default()
            },
        )
        .expect("ongoing renju replay should analyze");

        assert_eq!(
            analysis.model.defense_policy,
            DefensePolicy::TacticalDefense
        );
        assert_eq!(analysis.root_cause, RootCause::Unclear);
        assert!(analysis
            .model
            .tactical_reply_coverage
            .contains(&"forbidden_cost_squares".to_string()));
    }
}
