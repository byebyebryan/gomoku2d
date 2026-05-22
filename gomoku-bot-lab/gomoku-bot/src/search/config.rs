use std::time::Duration;

use gomoku_core::Color;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidateSource {
    NearAll {
        radius: usize,
    },
    NearSelfOpponent {
        self_radius: usize,
        opponent_radius: usize,
    },
}

impl CandidateSource {
    fn name(self) -> String {
        match self {
            CandidateSource::NearAll { radius } => format!("near_all_r{radius}"),
            CandidateSource::NearSelfOpponent {
                self_radius,
                opponent_radius,
            } => format!("near_self_r{self_radius}_opponent_r{opponent_radius}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegalityGate {
    ExactRules,
}

impl LegalityGate {
    const fn name(self) -> &'static str {
        match self {
            LegalityGate::ExactRules => "exact_rules",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafetyGate {
    None,
    CurrentObligation,
}

impl SafetyGate {
    const fn name(self) -> &'static str {
        match self {
            SafetyGate::None => "none",
            SafetyGate::CurrentObligation => "current_obligation",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveOrdering {
    TranspositionFirstBoardOrder,
    TacticalFull,
    Tactical,
}

impl MoveOrdering {
    const fn name(self) -> &'static str {
        match self {
            MoveOrdering::TranspositionFirstBoardOrder => "tt_first_board_order",
            MoveOrdering::TacticalFull => "tactical_full",
            MoveOrdering::Tactical => "tactical",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchAlgorithm {
    AlphaBetaIterativeDeepening,
}

impl SearchAlgorithm {
    const fn name(self) -> &'static str {
        match self {
            SearchAlgorithm::AlphaBetaIterativeDeepening => "alpha_beta_id",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaticEvaluation {
    LineShapeEval,
    PatternEval,
}

impl StaticEvaluation {
    const fn name(self) -> &'static str {
        match self {
            StaticEvaluation::LineShapeEval => "line_shape_eval",
            StaticEvaluation::PatternEval => "pattern_eval",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreatViewMode {
    Scan,
    RollingShadow,
    Rolling,
}

impl ThreatViewMode {
    const fn name(self) -> &'static str {
        match self {
            ThreatViewMode::Scan => "scan",
            ThreatViewMode::RollingShadow => "rolling_shadow",
            ThreatViewMode::Rolling => "rolling",
        }
    }

    pub(super) const fn uses_frontier(self) -> bool {
        !matches!(self, ThreatViewMode::Scan)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NullCellCulling {
    Disabled,
    Enabled,
}

impl NullCellCulling {
    const fn name(self) -> &'static str {
        match self {
            NullCellCulling::Disabled => "disabled",
            NullCellCulling::Enabled => "enabled",
        }
    }

    pub(super) const fn enabled(self) -> bool {
        matches!(self, NullCellCulling::Enabled)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct CorridorProofConfig {
    pub enabled: bool,
    pub max_depth: usize,
    pub max_reply_width: usize,
    pub proof_candidate_limit: usize,
}

impl CorridorProofConfig {
    pub const DEFAULT_PROOF_CANDIDATE_LIMIT: usize = 3;

    pub const DISABLED: Self = Self {
        enabled: false,
        max_depth: 0,
        max_reply_width: 0,
        proof_candidate_limit: 0,
    };

    pub(super) fn trace(self) -> serde_json::Value {
        serde_json::json!({
            "enabled": self.enabled,
            "max_depth": self.max_depth,
            "max_reply_width": self.max_reply_width,
            "proof_candidate_limit": self.proof_candidate_limit,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CorridorSide {
    Own,
    Opponent,
}

impl CorridorSide {
    pub(super) const fn for_player(player: Color, root_color: Color) -> Self {
        if matches!(
            (player, root_color),
            (Color::Black, Color::Black) | (Color::White, Color::White)
        ) {
            Self::Own
        } else {
            Self::Opponent
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchBotConfig {
    pub max_depth: i32,
    pub time_budget_ms: Option<u64>,
    pub cpu_time_budget_ms: Option<u64>,
    pub max_tt_entries: Option<usize>,
    pub candidate_radius: usize,
    pub candidate_opponent_radius: Option<usize>,
    pub safety_gate: SafetyGate,
    pub move_ordering: MoveOrdering,
    pub child_limit: Option<usize>,
    pub search_algorithm: SearchAlgorithm,
    pub static_eval: StaticEvaluation,
    pub corridor_proof: CorridorProofConfig,
    pub threat_view_mode: ThreatViewMode,
    pub null_cell_culling: NullCellCulling,
}

impl SearchBotConfig {
    pub const fn custom_depth(max_depth: i32) -> Self {
        Self {
            max_depth,
            time_budget_ms: None,
            cpu_time_budget_ms: None,
            max_tt_entries: None,
            candidate_radius: 2,
            candidate_opponent_radius: None,
            safety_gate: SafetyGate::CurrentObligation,
            move_ordering: MoveOrdering::TranspositionFirstBoardOrder,
            child_limit: None,
            search_algorithm: SearchAlgorithm::AlphaBetaIterativeDeepening,
            static_eval: StaticEvaluation::LineShapeEval,
            corridor_proof: CorridorProofConfig::DISABLED,
            threat_view_mode: ThreatViewMode::Rolling,
            null_cell_culling: NullCellCulling::Disabled,
        }
    }

    pub const fn custom_time_budget(time_budget_ms: u64) -> Self {
        Self {
            max_depth: 20,
            time_budget_ms: Some(time_budget_ms),
            cpu_time_budget_ms: None,
            max_tt_entries: None,
            candidate_radius: 2,
            candidate_opponent_radius: None,
            safety_gate: SafetyGate::CurrentObligation,
            move_ordering: MoveOrdering::TranspositionFirstBoardOrder,
            child_limit: None,
            search_algorithm: SearchAlgorithm::AlphaBetaIterativeDeepening,
            static_eval: StaticEvaluation::LineShapeEval,
            corridor_proof: CorridorProofConfig::DISABLED,
            threat_view_mode: ThreatViewMode::Rolling,
            null_cell_culling: NullCellCulling::Disabled,
        }
    }

    pub const fn custom_cpu_time_budget(cpu_time_budget_ms: u64) -> Self {
        Self {
            max_depth: 20,
            time_budget_ms: None,
            cpu_time_budget_ms: Some(cpu_time_budget_ms),
            max_tt_entries: None,
            candidate_radius: 2,
            candidate_opponent_radius: None,
            safety_gate: SafetyGate::CurrentObligation,
            move_ordering: MoveOrdering::TranspositionFirstBoardOrder,
            child_limit: None,
            search_algorithm: SearchAlgorithm::AlphaBetaIterativeDeepening,
            static_eval: StaticEvaluation::LineShapeEval,
            corridor_proof: CorridorProofConfig::DISABLED,
            threat_view_mode: ThreatViewMode::Rolling,
            null_cell_culling: NullCellCulling::Disabled,
        }
    }

    pub(super) fn time_budget(self) -> Option<Duration> {
        self.time_budget_ms.map(Duration::from_millis)
    }

    pub(super) fn cpu_time_budget(self) -> Option<Duration> {
        self.cpu_time_budget_ms.map(Duration::from_millis)
    }

    pub const fn candidate_source(self) -> CandidateSource {
        match self.candidate_opponent_radius {
            Some(opponent_radius) if opponent_radius != self.candidate_radius => {
                CandidateSource::NearSelfOpponent {
                    self_radius: self.candidate_radius,
                    opponent_radius,
                }
            }
            _ => CandidateSource::NearAll {
                radius: self.candidate_radius,
            },
        }
    }

    pub const fn legality_gate(self) -> LegalityGate {
        LegalityGate::ExactRules
    }

    pub const fn safety_gate(self) -> SafetyGate {
        self.safety_gate
    }

    pub(super) fn trace(self) -> serde_json::Value {
        serde_json::json!({
            "max_depth": self.max_depth,
            "time_budget_ms": self.time_budget_ms,
            "cpu_time_budget_ms": self.cpu_time_budget_ms,
            "max_tt_entries": self.max_tt_entries,
            "candidate_radius": self.candidate_radius,
            "candidate_opponent_radius": self.candidate_opponent_radius,
            "candidate_source": self.candidate_source().name(),
            "legality_gate": self.legality_gate().name(),
            "safety_gate": self.safety_gate().name(),
            "move_ordering": self.move_ordering.name(),
            "child_limit": self.child_limit,
            "search_algorithm": self.search_algorithm.name(),
            "static_eval": self.static_eval.name(),
            "corridor_proof": self.corridor_proof.trace(),
            "threat_view_mode": self.threat_view_mode.name(),
            "null_cell_culling": self.null_cell_culling.name(),
        })
    }
}
