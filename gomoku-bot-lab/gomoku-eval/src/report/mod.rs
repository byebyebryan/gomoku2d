use crate::arena::MatchEndReason;
#[cfg(test)]
use crate::bot_label::compact_bot_label as shared_compact_bot_label;
use crate::elo::{expected_score, DEFAULT_INITIAL_RATING, DEFAULT_K_FACTOR};
use crate::tournament::TournamentResults;
use gomoku_core::{Color, GameResult, Move, RuleConfig};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

pub const TOURNAMENT_REPORT_SCHEMA_VERSION: u32 = 1;
pub const PUBLISHED_TOURNAMENT_REPORT_SCHEMA_VERSION: u32 = 2;
pub const MOVE_CODEC: &str = "cell_index_v1";
const SHUFFLED_ELO_SAMPLES: usize = 256;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TournamentRunReport {
    pub bots: Vec<String>,
    #[serde(default = "default_schedule")]
    pub schedule: String,
    pub rules: RuleConfig,
    pub games_per_pair: u32,
    pub seed: u64,
    pub opening_plies: usize,
    #[serde(default = "default_opening_policy")]
    pub opening_policy: String,
    pub threads: usize,
    pub search_time_ms: Option<u64>,
    pub search_cpu_time_ms: Option<u64>,
    #[serde(default = "default_search_budget_mode")]
    pub search_budget_mode: String,
    #[serde(default)]
    pub search_cpu_reserve_ms: Option<u64>,
    #[serde(default)]
    pub search_cpu_max_move_ms: Option<u64>,
    pub max_moves: Option<usize>,
    pub max_game_ms: Option<u64>,
    #[serde(default)]
    pub total_wall_time_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TournamentReport {
    pub schema_version: u32,
    pub report_kind: String,
    pub board_size: usize,
    pub move_codec: String,
    pub shuffled_elo_samples: usize,
    #[serde(default)]
    pub provenance: ReportProvenance,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_anchors: Option<AnchorReferenceReport>,
    pub run: TournamentRunReport,
    pub standings: Vec<StandingReport>,
    pub pairwise: Vec<PairwiseReport>,
    pub color_splits: Vec<ColorSplitReport>,
    pub end_reasons: Vec<CountReport>,
    pub matches: Vec<MatchReport>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReportProvenance {
    pub generated_at_utc: Option<String>,
    pub generated_at_local: Option<String>,
    pub git_commit: Option<String>,
    pub git_dirty: Option<bool>,
    pub command: Vec<String>,
    pub host: Option<HostReport>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HostReport {
    pub os: String,
    pub arch: String,
    pub logical_cpus: Option<usize>,
    pub cpu_model: Option<String>,
    pub cpu_mhz: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorReferenceReport {
    pub source: AnchorReferenceSource,
    pub anchors: Vec<AnchorStandingReport>,
    #[serde(default)]
    pub pairwise: Vec<PairwiseReport>,
    #[serde(default)]
    pub pair_search: Vec<ReferencePairSearchReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorReferenceSource {
    pub path: Option<String>,
    pub schedule: String,
    pub git_commit: Option<String>,
    pub git_dirty: Option<bool>,
    pub rules: RuleConfig,
    pub games_per_pair: u32,
    pub opening_policy: String,
    pub opening_plies: usize,
    pub seed: u64,
    pub search_time_ms: Option<u64>,
    pub search_cpu_time_ms: Option<u64>,
    #[serde(default = "default_search_budget_mode")]
    pub search_budget_mode: String,
    #[serde(default)]
    pub search_cpu_reserve_ms: Option<u64>,
    #[serde(default)]
    pub search_cpu_max_move_ms: Option<u64>,
    #[serde(default)]
    pub max_moves: Option<usize>,
    #[serde(default)]
    pub max_game_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorStandingReport {
    pub bot: String,
    pub sequential_elo: f64,
    pub shuffled_elo_avg: f64,
    pub shuffled_elo_stddev: f64,
    pub match_count: u32,
    pub score_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferencePairSearchReport {
    pub bot_a: String,
    pub bot_b: String,
    pub bot_a_search_move_count: u32,
    pub bot_a_total_time_ms: u64,
    pub bot_a_total_nodes: u64,
    pub bot_b_search_move_count: u32,
    pub bot_b_total_time_ms: u64,
    pub bot_b_total_nodes: u64,
}

impl ReportProvenance {
    pub(crate) fn capture() -> Self {
        Self {
            generated_at_utc: Some(detect_generated_at_utc()),
            generated_at_local: Some(detect_generated_at_local()),
            git_commit: detect_git_commit(),
            git_dirty: detect_git_dirty(),
            command: std::env::args().collect(),
            host: Some(HostReport::capture()),
        }
    }
}

impl HostReport {
    fn capture() -> Self {
        let (cpu_model, cpu_mhz) = detect_linux_cpu_info();

        Self {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            logical_cpus: std::thread::available_parallelism()
                .ok()
                .map(std::num::NonZeroUsize::get),
            cpu_model,
            cpu_mhz,
        }
    }
}

impl AnchorReferenceReport {
    pub fn from_published_report(
        source_path: Option<String>,
        source_report: &PublishedTournamentReport,
        anchor_names: &[String],
    ) -> Result<Self, String> {
        if source_report.run.schedule != "round-robin" {
            return Err(format!(
                "anchor report must come from a round-robin reference report, got {}",
                source_report.run.schedule
            ));
        }

        let mut standings_by_bot: HashMap<&str, &PublishedStandingReport> = HashMap::new();
        for standing in &source_report.standings {
            standings_by_bot.insert(&standing.bot, standing);
        }

        let mut missing = Vec::new();
        let mut anchors = Vec::new();
        for anchor_name in anchor_names {
            let Some(standing) = standings_by_bot.get(anchor_name.as_str()) else {
                missing.push(anchor_name.clone());
                continue;
            };
            anchors.push(AnchorStandingReport::from_published_standing(standing));
        }

        if !missing.is_empty() {
            return Err(format!(
                "anchor report is missing standings for: {}",
                missing.join(", ")
            ));
        }

        let anchor_set = anchor_names
            .iter()
            .map(String::as_str)
            .collect::<HashSet<_>>();
        let pairwise: Vec<PairwiseReport> = source_report
            .pairwise
            .iter()
            .filter(|pair| {
                anchor_set.contains(pair.bot_a.as_str()) && anchor_set.contains(pair.bot_b.as_str())
            })
            .cloned()
            .collect();

        Ok(Self {
            source: AnchorReferenceSource {
                path: source_path,
                schedule: source_report.run.schedule.clone(),
                git_commit: source_report.provenance.git_commit.clone(),
                git_dirty: source_report.provenance.git_dirty,
                rules: source_report.run.rules.clone(),
                games_per_pair: source_report.run.games_per_pair,
                opening_policy: source_report.run.opening_policy.clone(),
                opening_plies: source_report.run.opening_plies,
                seed: source_report.run.seed,
                search_time_ms: source_report.run.search_time_ms,
                search_cpu_time_ms: source_report.run.search_cpu_time_ms,
                search_budget_mode: source_report.run.search_budget_mode.clone(),
                search_cpu_reserve_ms: source_report.run.search_cpu_reserve_ms,
                search_cpu_max_move_ms: source_report.run.search_cpu_max_move_ms,
                max_moves: source_report.run.max_moves,
                max_game_ms: source_report.run.max_game_ms,
            },
            anchors,
            pairwise,
            pair_search: Vec::new(),
        })
    }

    pub fn from_report(
        source_path: Option<String>,
        source_report: &TournamentReport,
        anchor_names: &[String],
    ) -> Result<Self, String> {
        if source_report.run.schedule != "round-robin" {
            return Err(format!(
                "anchor report must come from a round-robin reference report, got {}",
                source_report.run.schedule
            ));
        }

        let mut standings_by_bot: HashMap<&str, &StandingReport> = HashMap::new();
        for standing in &source_report.standings {
            standings_by_bot.insert(&standing.bot, standing);
        }

        let mut missing = Vec::new();
        let mut anchors = Vec::new();
        for anchor_name in anchor_names {
            let Some(standing) = standings_by_bot.get(anchor_name.as_str()) else {
                missing.push(anchor_name.clone());
                continue;
            };
            anchors.push(AnchorStandingReport::from_standing(standing));
        }

        if !missing.is_empty() {
            return Err(format!(
                "anchor report is missing standings for: {}",
                missing.join(", ")
            ));
        }

        let anchor_set = anchor_names
            .iter()
            .map(String::as_str)
            .collect::<HashSet<_>>();
        let pairwise: Vec<PairwiseReport> = source_report
            .pairwise
            .iter()
            .filter(|pair| {
                anchor_set.contains(pair.bot_a.as_str()) && anchor_set.contains(pair.bot_b.as_str())
            })
            .cloned()
            .collect();
        let pair_search = reference_pair_search_reports(source_report, &pairwise);

        Ok(Self {
            source: AnchorReferenceSource {
                path: source_path,
                schedule: source_report.run.schedule.clone(),
                git_commit: source_report.provenance.git_commit.clone(),
                git_dirty: source_report.provenance.git_dirty,
                rules: source_report.run.rules.clone(),
                games_per_pair: source_report.run.games_per_pair,
                opening_policy: source_report.run.opening_policy.clone(),
                opening_plies: source_report.run.opening_plies,
                seed: source_report.run.seed,
                search_time_ms: source_report.run.search_time_ms,
                search_cpu_time_ms: source_report.run.search_cpu_time_ms,
                search_budget_mode: source_report.run.search_budget_mode.clone(),
                search_cpu_reserve_ms: source_report.run.search_cpu_reserve_ms,
                search_cpu_max_move_ms: source_report.run.search_cpu_max_move_ms,
                max_moves: source_report.run.max_moves,
                max_game_ms: source_report.run.max_game_ms,
            },
            anchors,
            pairwise,
            pair_search,
        })
    }

    pub fn validate_compatible_run(&self, run: &TournamentRunReport) -> Result<(), String> {
        let source = &self.source;
        let mut mismatches = Vec::new();

        if source.rules.board_size != run.rules.board_size
            || source.rules.win_length != run.rules.win_length
            || source.rules.variant != run.rules.variant
        {
            mismatches.push("rules".to_string());
        }
        if source.opening_policy != run.opening_policy {
            mismatches.push("opening_policy".to_string());
        }
        if source.opening_plies != run.opening_plies {
            mismatches.push("opening_plies".to_string());
        }
        if source.search_time_ms != run.search_time_ms {
            mismatches.push("search_time_ms".to_string());
        }
        if source.search_cpu_time_ms != run.search_cpu_time_ms {
            mismatches.push("search_cpu_time_ms".to_string());
        }
        if source.search_budget_mode != run.search_budget_mode {
            mismatches.push("search_budget_mode".to_string());
        }
        if source.search_cpu_reserve_ms != run.search_cpu_reserve_ms {
            mismatches.push("search_cpu_reserve_ms".to_string());
        }
        if source.search_cpu_max_move_ms != run.search_cpu_max_move_ms {
            mismatches.push("search_cpu_max_move_ms".to_string());
        }
        if source.max_moves != run.max_moves {
            mismatches.push("max_moves".to_string());
        }
        if source.max_game_ms != run.max_game_ms {
            mismatches.push("max_game_ms".to_string());
        }

        if mismatches.is_empty() {
            return Ok(());
        }

        Err(format!(
            "anchor report eval context does not match current run: {}",
            mismatches.join(", ")
        ))
    }
}

impl AnchorStandingReport {
    fn from_standing(standing: &StandingReport) -> Self {
        Self {
            bot: standing.bot.clone(),
            sequential_elo: standing.sequential_elo,
            shuffled_elo_avg: standing.shuffled_elo_avg,
            shuffled_elo_stddev: standing.shuffled_elo_stddev,
            match_count: standing.match_count,
            score_percentage: score_rate(standing.wins, standing.draws, standing.match_count)
                * 100.0,
        }
    }

    fn from_published_standing(standing: &PublishedStandingReport) -> Self {
        Self {
            bot: standing.bot.clone(),
            sequential_elo: standing.sequential_elo,
            shuffled_elo_avg: standing.shuffled_elo_avg,
            shuffled_elo_stddev: standing.shuffled_elo_stddev,
            match_count: standing.match_count,
            score_percentage: score_rate(standing.wins, standing.draws, standing.match_count)
                * 100.0,
        }
    }
}

impl TournamentReport {
    pub fn from_results(
        run: TournamentRunReport,
        results: &TournamentResults,
    ) -> Result<Self, String> {
        let board_size = run.rules.board_size;
        let matches = results
            .matches
            .iter()
            .map(|record| MatchReport::from_record(record, board_size))
            .collect::<Result<Vec<_>, _>>()?;
        let shuffled_elo = shuffled_elo_stats(&run.bots, &matches, SHUFFLED_ELO_SAMPLES);

        Ok(Self {
            schema_version: TOURNAMENT_REPORT_SCHEMA_VERSION,
            report_kind: "tournament".to_string(),
            board_size,
            move_codec: MOVE_CODEC.to_string(),
            shuffled_elo_samples: SHUFFLED_ELO_SAMPLES,
            provenance: ReportProvenance::capture(),
            reference_anchors: None,
            standings: standings(&run.bots, results, &matches, &shuffled_elo),
            pairwise: pairwise(&run.bots, &matches),
            color_splits: color_splits(&matches),
            end_reasons: end_reasons(results),
            matches,
            run,
        })
    }

    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }

    pub fn shadow_mismatch_count(&self) -> u64 {
        self.standings
            .iter()
            .map(|row| row.threat_view_shadow_mismatches)
            .sum()
    }

    pub fn from_json(input: &str) -> Result<Self, String> {
        let report: Self = serde_json::from_str(input).map_err(|err| err.to_string())?;
        report.validate()?;
        Ok(report)
    }

    fn validate(&self) -> Result<(), String> {
        if self.schema_version != TOURNAMENT_REPORT_SCHEMA_VERSION {
            return Err(format!(
                "unsupported tournament report schema version: {}",
                self.schema_version
            ));
        }
        if self.report_kind != "tournament" {
            return Err(format!("unsupported report kind: {}", self.report_kind));
        }
        if self.move_codec != MOVE_CODEC {
            return Err(format!("unsupported move codec: {}", self.move_codec));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishedTournamentReport {
    pub schema_version: u32,
    pub report_kind: String,
    pub source_schema_version: u32,
    pub board_size: usize,
    pub move_codec: String,
    #[serde(default)]
    pub provenance: ReportProvenance,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_anchors: Option<AnchorReferenceReport>,
    pub run: TournamentRunReport,
    pub standings: Vec<PublishedStandingReport>,
    pub pairwise: Vec<PairwiseReport>,
    pub end_reasons: Vec<CountReport>,
    pub matches: Vec<PublishedMatchReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishedStandingReport {
    pub bot: String,
    pub wins: u32,
    pub draws: u32,
    pub losses: u32,
    pub sequential_elo: f64,
    pub shuffled_elo_avg: f64,
    pub shuffled_elo_stddev: f64,
    pub match_count: u32,
    pub move_count: u32,
    pub search_move_count: u32,
    pub total_time_ms: u64,
    pub avg_search_time_ms: f64,
    pub total_nodes: u64,
    pub avg_nodes: f64,
    pub avg_depth: f64,
    pub max_depth: u32,
    #[serde(default)]
    pub avg_effective_depth: f64,
    #[serde(default)]
    pub max_effective_depth: u32,
    #[serde(default)]
    pub avg_child_moves_before: f64,
    #[serde(default)]
    pub avg_child_moves_after: f64,
    pub budget_exhausted_rate: f64,
    #[serde(default)]
    pub pooled_budget_over_base_rate: f64,
    #[serde(default)]
    pub pooled_budget_reserve_exhausted_rate: f64,
    #[serde(default)]
    pub stage_move_gen_ns: u64,
    #[serde(default)]
    pub stage_ordering_ns: u64,
    #[serde(default)]
    pub stage_eval_ns: u64,
    #[serde(default)]
    pub stage_threat_ns: u64,
    #[serde(default)]
    pub stage_proof_ns: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishedMatchReport {
    pub match_index: usize,
    pub black: String,
    pub white: String,
    pub result: String,
    pub winner: Option<String>,
    pub end_reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opening: Option<MatchOpeningReport>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub move_cells: Vec<usize>,
    pub move_count: usize,
}

impl PublishedTournamentReport {
    pub fn from_tournament_report(report: &TournamentReport) -> Self {
        Self {
            schema_version: PUBLISHED_TOURNAMENT_REPORT_SCHEMA_VERSION,
            report_kind: "published_tournament".to_string(),
            source_schema_version: report.schema_version,
            board_size: report.board_size,
            move_codec: report.move_codec.clone(),
            provenance: report.provenance.clone(),
            reference_anchors: report.reference_anchors.clone(),
            run: report.run.clone(),
            standings: report
                .standings
                .iter()
                .map(PublishedStandingReport::from_standing_report)
                .collect(),
            pairwise: report.pairwise.clone(),
            end_reasons: report.end_reasons.clone(),
            matches: report
                .matches
                .iter()
                .map(PublishedMatchReport::from_match_report)
                .collect(),
        }
    }

    pub fn from_published_report(report: &PublishedTournamentReport) -> Self {
        let mut next = report.clone();
        next.schema_version = PUBLISHED_TOURNAMENT_REPORT_SCHEMA_VERSION;
        for match_report in &mut next.matches {
            match_report.opening = None;
        }
        next
    }

    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }

    pub fn from_json(input: &str) -> Result<Self, String> {
        let report: Self = serde_json::from_str(input).map_err(|err| err.to_string())?;
        report.validate()?;
        Ok(report)
    }

    fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1
            && self.schema_version != PUBLISHED_TOURNAMENT_REPORT_SCHEMA_VERSION
        {
            return Err(format!(
                "unsupported published tournament report schema version: {}",
                self.schema_version
            ));
        }
        if self.report_kind != "published_tournament" {
            return Err(format!("unsupported report kind: {}", self.report_kind));
        }
        if self.move_codec != MOVE_CODEC {
            return Err(format!("unsupported move codec: {}", self.move_codec));
        }
        Ok(())
    }
}

impl PublishedStandingReport {
    fn from_standing_report(standing: &StandingReport) -> Self {
        Self {
            bot: standing.bot.clone(),
            wins: standing.wins,
            draws: standing.draws,
            losses: standing.losses,
            sequential_elo: standing.sequential_elo,
            shuffled_elo_avg: standing.shuffled_elo_avg,
            shuffled_elo_stddev: standing.shuffled_elo_stddev,
            match_count: standing.match_count,
            move_count: standing.move_count,
            search_move_count: standing.search_move_count,
            total_time_ms: standing.total_time_ms,
            avg_search_time_ms: standing.avg_search_time_ms,
            total_nodes: standing.total_nodes,
            avg_nodes: standing.avg_nodes,
            avg_depth: standing.avg_depth,
            max_depth: standing.max_depth,
            avg_effective_depth: standing.avg_effective_depth,
            max_effective_depth: standing.max_effective_depth,
            avg_child_moves_before: standing.avg_child_moves_before,
            avg_child_moves_after: standing.avg_child_moves_after,
            budget_exhausted_rate: standing.budget_exhausted_rate,
            pooled_budget_over_base_rate: standing.pooled_budget_over_base_rate,
            pooled_budget_reserve_exhausted_rate: standing.pooled_budget_reserve_exhausted_rate,
            stage_move_gen_ns: standing.stage_move_gen_ns,
            stage_ordering_ns: standing.stage_ordering_ns,
            stage_eval_ns: standing.stage_eval_ns,
            stage_threat_ns: standing.stage_threat_ns,
            stage_proof_ns: standing.stage_proof_ns,
        }
    }
}

impl PublishedMatchReport {
    fn from_match_report(report_match: &MatchReport) -> Self {
        Self {
            match_index: report_match.match_index,
            black: report_match.black.clone(),
            white: report_match.white.clone(),
            result: report_match.result.clone(),
            winner: report_match.winner.clone(),
            end_reason: report_match.end_reason.clone(),
            opening: None,
            move_cells: report_match.move_cells.clone(),
            move_count: report_match.move_count,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandingReport {
    pub bot: String,
    pub wins: u32,
    pub draws: u32,
    pub losses: u32,
    pub sequential_elo: f64,
    pub shuffled_elo_avg: f64,
    pub shuffled_elo_stddev: f64,
    pub match_count: u32,
    pub move_count: u32,
    pub search_move_count: u32,
    pub total_time_ms: u64,
    pub avg_search_time_ms: f64,
    #[serde(default)]
    pub search_nodes: u64,
    #[serde(default)]
    pub safety_nodes: u64,
    #[serde(default)]
    pub corridor_nodes: u64,
    #[serde(default)]
    pub corridor_branch_probes: u64,
    #[serde(default)]
    pub corridor_max_depth: u32,
    #[serde(default)]
    pub corridor_width_exits: u64,
    #[serde(default)]
    pub corridor_depth_exits: u64,
    #[serde(default)]
    pub corridor_neutral_exits: u64,
    #[serde(default)]
    pub corridor_terminal_exits: u64,
    #[serde(default)]
    pub corridor_plies_followed: u64,
    #[serde(default)]
    pub corridor_own_plies_followed: u64,
    #[serde(default)]
    pub corridor_opponent_plies_followed: u64,
    #[serde(default)]
    pub corridor_proof_passes: u64,
    #[serde(default)]
    pub corridor_proof_completed: u64,
    #[serde(default)]
    pub corridor_proof_checks: u64,
    #[serde(default)]
    pub corridor_proof_active: u64,
    #[serde(default)]
    pub corridor_proof_quiet: u64,
    #[serde(default)]
    pub corridor_proof_static_exits: u64,
    #[serde(default)]
    pub corridor_proof_depth_exits: u64,
    #[serde(default)]
    pub corridor_proof_deadline_exits: u64,
    #[serde(default)]
    pub corridor_proof_terminal_exits: u64,
    #[serde(default)]
    pub corridor_proof_terminal_root_candidates: u64,
    #[serde(default)]
    pub corridor_proof_terminal_root_winning_candidates: u64,
    #[serde(default)]
    pub corridor_proof_terminal_root_losing_candidates: u64,
    #[serde(default)]
    pub corridor_proof_terminal_root_overrides: u64,
    #[serde(default)]
    pub corridor_proof_terminal_root_move_changes: u64,
    #[serde(default)]
    pub corridor_proof_terminal_root_move_confirmations: u64,
    #[serde(default)]
    pub corridor_proof_candidates_considered: u64,
    #[serde(default)]
    pub corridor_proof_wins: u64,
    #[serde(default)]
    pub corridor_proof_losses: u64,
    #[serde(default)]
    pub corridor_proof_unknown: u64,
    #[serde(default)]
    pub corridor_proof_deadline_skips: u64,
    #[serde(default)]
    pub corridor_proof_move_changes: u64,
    #[serde(default)]
    pub corridor_proof_move_confirmations: u64,
    #[serde(default)]
    pub corridor_proof_candidate_rank_total: u64,
    #[serde(default)]
    pub corridor_proof_candidate_rank_max: u64,
    #[serde(default)]
    pub corridor_proof_candidate_score_gap_total: u64,
    #[serde(default)]
    pub corridor_proof_candidate_score_gap_max: u64,
    #[serde(default)]
    pub corridor_proof_win_candidate_rank_total: u64,
    #[serde(default)]
    pub corridor_proof_win_candidate_rank_max: u64,
    pub total_nodes: u64,
    pub avg_nodes: f64,
    #[serde(default)]
    pub eval_calls: u64,
    #[serde(default)]
    pub avg_eval_calls: f64,
    #[serde(default)]
    pub line_shape_eval_calls: u64,
    #[serde(default)]
    pub line_shape_eval_ns: u64,
    #[serde(default)]
    pub avg_line_shape_eval_ns: f64,
    #[serde(default)]
    pub pattern_eval_calls: u64,
    #[serde(default)]
    pub pattern_eval_ns: u64,
    #[serde(default)]
    pub avg_pattern_eval_ns: f64,
    #[serde(default)]
    pub pattern_frame_queries: u64,
    #[serde(default)]
    pub pattern_frame_query_ns: u64,
    #[serde(default)]
    pub avg_pattern_frame_query_ns: f64,
    #[serde(default)]
    pub pattern_frame_updates: u64,
    #[serde(default)]
    pub pattern_frame_update_ns: u64,
    #[serde(default)]
    pub avg_pattern_frame_update_ns: f64,
    #[serde(default)]
    pub pattern_frame_shadow_checks: u64,
    #[serde(default)]
    pub pattern_frame_shadow_mismatches: u64,
    #[serde(default)]
    pub candidate_generations: u64,
    #[serde(default)]
    pub avg_candidate_generations: f64,
    #[serde(default)]
    pub candidate_moves_total: u64,
    #[serde(default)]
    pub avg_candidate_moves: f64,
    #[serde(default)]
    pub candidate_moves_max: u64,
    #[serde(default)]
    pub root_candidate_generations: u64,
    #[serde(default)]
    pub root_candidate_moves_total: u64,
    #[serde(default)]
    pub root_candidate_moves_max: u64,
    #[serde(default)]
    pub search_candidate_generations: u64,
    #[serde(default)]
    pub search_candidate_moves_total: u64,
    #[serde(default)]
    pub search_candidate_moves_max: u64,
    #[serde(default)]
    pub legality_checks: u64,
    #[serde(default)]
    pub avg_legality_checks: f64,
    #[serde(default)]
    pub illegal_moves_skipped: u64,
    #[serde(default)]
    pub root_legality_checks: u64,
    #[serde(default)]
    pub root_illegal_moves_skipped: u64,
    #[serde(default)]
    pub search_legality_checks: u64,
    #[serde(default)]
    pub search_illegal_moves_skipped: u64,
    #[serde(default)]
    pub renju_forbidden_prefilter_checks: u64,
    #[serde(default)]
    pub avg_renju_forbidden_prefilter_checks: f64,
    #[serde(default)]
    pub renju_forbidden_prefilter_ns: u64,
    #[serde(default)]
    pub avg_renju_forbidden_prefilter_ns: f64,
    #[serde(default)]
    pub renju_forbidden_checks: u64,
    #[serde(default)]
    pub avg_renju_forbidden_checks: f64,
    #[serde(default)]
    pub renju_forbidden_ns: u64,
    #[serde(default)]
    pub avg_renju_forbidden_ns: f64,
    #[serde(default)]
    pub renju_forbidden_search_gate_checks: u64,
    #[serde(default)]
    pub renju_forbidden_search_gate_ns: u64,
    #[serde(default)]
    pub renju_forbidden_pattern_checks: u64,
    #[serde(default)]
    pub renju_forbidden_pattern_ns: u64,
    #[serde(default)]
    pub renju_forbidden_threat_checks: u64,
    #[serde(default)]
    pub renju_forbidden_threat_ns: u64,
    #[serde(default)]
    pub renju_forbidden_other_checks: u64,
    #[serde(default)]
    pub renju_forbidden_other_ns: u64,
    #[serde(default)]
    pub renju_effective_filter_calls: u64,
    #[serde(default)]
    pub avg_renju_effective_filter_calls: f64,
    #[serde(default)]
    pub renju_effective_filter_ns: u64,
    #[serde(default)]
    pub avg_renju_effective_filter_ns: f64,
    #[serde(default)]
    pub renju_effective_filter_continuation_checks: u64,
    #[serde(default)]
    pub avg_renju_effective_filter_continuation_checks: f64,
    #[serde(default)]
    pub renju_effective_filter_continuation_ns: u64,
    #[serde(default)]
    pub avg_renju_effective_filter_continuation_ns: f64,
    #[serde(default)]
    pub stage_move_gen_ns: u64,
    #[serde(default)]
    pub stage_ordering_ns: u64,
    #[serde(default)]
    pub stage_eval_ns: u64,
    #[serde(default)]
    pub stage_threat_ns: u64,
    #[serde(default)]
    pub stage_proof_ns: u64,
    #[serde(default)]
    pub tactical_annotations: u64,
    #[serde(default)]
    pub root_tactical_annotations: u64,
    #[serde(default)]
    pub search_tactical_annotations: u64,
    #[serde(default)]
    pub threat_view_shadow_checks: u64,
    #[serde(default)]
    pub threat_view_shadow_mismatches: u64,
    #[serde(default)]
    pub threat_view_scan_queries: u64,
    #[serde(default)]
    pub threat_view_scan_ns: u64,
    #[serde(default)]
    pub threat_view_frontier_rebuilds: u64,
    #[serde(default)]
    pub threat_view_frontier_rebuild_ns: u64,
    #[serde(default)]
    pub threat_view_frontier_queries: u64,
    #[serde(default)]
    pub threat_view_frontier_query_ns: u64,
    #[serde(default)]
    pub threat_view_frontier_immediate_win_queries: u64,
    #[serde(default)]
    pub threat_view_frontier_immediate_win_query_ns: u64,
    #[serde(default)]
    pub threat_view_frontier_delta_captures: u64,
    #[serde(default)]
    pub threat_view_frontier_delta_capture_ns: u64,
    #[serde(default)]
    pub threat_view_frontier_move_fact_updates: u64,
    #[serde(default)]
    pub threat_view_frontier_move_fact_update_ns: u64,
    #[serde(default)]
    pub threat_view_frontier_annotation_dirty_marks: u64,
    #[serde(default)]
    pub threat_view_frontier_annotation_dirty_mark_ns: u64,
    #[serde(default)]
    pub threat_view_frontier_clean_annotation_queries: u64,
    #[serde(default)]
    pub threat_view_frontier_clean_annotation_query_ns: u64,
    #[serde(default)]
    pub threat_view_frontier_dirty_annotation_queries: u64,
    #[serde(default)]
    pub threat_view_frontier_dirty_annotation_query_ns: u64,
    #[serde(default)]
    pub threat_view_frontier_fallback_annotation_queries: u64,
    #[serde(default)]
    pub threat_view_frontier_fallback_annotation_query_ns: u64,
    #[serde(default)]
    pub threat_view_frontier_memo_annotation_queries: u64,
    #[serde(default)]
    pub threat_view_frontier_memo_annotation_query_ns: u64,
    #[serde(default)]
    pub child_limit_applications: u64,
    #[serde(default)]
    pub root_child_limit_applications: u64,
    #[serde(default)]
    pub search_child_limit_applications: u64,
    #[serde(default)]
    pub child_cap_hits: u64,
    #[serde(default)]
    pub root_child_cap_hits: u64,
    #[serde(default)]
    pub search_child_cap_hits: u64,
    #[serde(default)]
    pub child_moves_before_total: u64,
    #[serde(default)]
    pub root_child_moves_before_total: u64,
    #[serde(default)]
    pub search_child_moves_before_total: u64,
    #[serde(default)]
    pub child_moves_before_max: u64,
    #[serde(default)]
    pub root_child_moves_before_max: u64,
    #[serde(default)]
    pub search_child_moves_before_max: u64,
    #[serde(default)]
    pub child_moves_after_total: u64,
    #[serde(default)]
    pub root_child_moves_after_total: u64,
    #[serde(default)]
    pub search_child_moves_after_total: u64,
    #[serde(default)]
    pub child_moves_after_max: u64,
    #[serde(default)]
    pub root_child_moves_after_max: u64,
    #[serde(default)]
    pub search_child_moves_after_max: u64,
    #[serde(default)]
    pub avg_child_moves_before: f64,
    #[serde(default)]
    pub avg_child_moves_after: f64,
    #[serde(default)]
    pub tt_hits: u64,
    #[serde(default)]
    pub tt_cutoffs: u64,
    #[serde(default)]
    pub beta_cutoffs: u64,
    pub avg_depth: f64,
    pub max_depth: u32,
    #[serde(default)]
    pub effective_depth_sum: u64,
    #[serde(default)]
    pub avg_effective_depth: f64,
    #[serde(default)]
    pub max_effective_depth: u32,
    #[serde(default)]
    pub depth_reached_counts: Vec<DepthCountReport>,
    pub budget_exhausted_count: u32,
    pub budget_exhausted_rate: f64,
    #[serde(default)]
    pub pooled_budget_moves: u32,
    #[serde(default)]
    pub pooled_budget_over_base_count: u32,
    #[serde(default)]
    pub pooled_budget_over_base_rate: f64,
    #[serde(default)]
    pub pooled_budget_reserve_exhausted_count: u32,
    #[serde(default)]
    pub pooled_budget_reserve_exhausted_rate: f64,
    #[serde(default)]
    pub pooled_budget_avg_reserve_before_ms: f64,
    #[serde(default)]
    pub pooled_budget_avg_reserve_after_ms: f64,
    #[serde(default)]
    pub pooled_budget_min_reserve_after_ms: u64,
    #[serde(default)]
    pub pooled_budget_max_move_budget_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairwiseReport {
    pub bot_a: String,
    pub bot_b: String,
    pub wins_a: u32,
    pub wins_b: u32,
    pub draws: u32,
    pub total: u32,
    pub score_a: f64,
    pub score_b: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorSplitReport {
    pub black: String,
    pub white: String,
    pub black_wins: u32,
    pub white_wins: u32,
    pub draws: u32,
    pub total: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountReport {
    pub key: String,
    pub count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DepthCountReport {
    pub depth: u32,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchReport {
    pub match_index: usize,
    pub black: String,
    pub white: String,
    pub result: String,
    pub winner: Option<String>,
    pub end_reason: String,
    pub duration_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opening: Option<MatchOpeningReport>,
    pub move_cells: Vec<usize>,
    pub move_count: usize,
    pub black_stats: SideStatsReport,
    pub white_stats: SideStatsReport,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchOpeningReport {
    pub policy: String,
    pub index: u32,
    pub ply_count: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suite_index: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template_index: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transform_index: Option<usize>,
}

impl MatchReport {
    fn from_record(
        record: &crate::tournament::TournamentMatchRecord,
        board_size: usize,
    ) -> Result<Self, String> {
        let mut black_stats = SideStatsAccumulator::default();
        let mut white_stats = SideStatsAccumulator::default();
        let mut move_cells = Vec::with_capacity(record.replay.moves.len());

        for (idx, replay_move) in record.replay.moves.iter().enumerate() {
            let mv = Move::from_notation(&replay_move.mv)?;
            move_cells.push(encode_move_cell(mv, board_size)?);
            let target = if idx % 2 == 0 {
                &mut black_stats
            } else {
                &mut white_stats
            };
            target.record_move(replay_move.time_ms, replay_move.trace.as_ref());
        }

        Ok(Self {
            match_index: record.match_idx,
            black: record.black_name.clone(),
            white: record.white_name.clone(),
            result: result_code(&record.result).to_string(),
            winner: winner_name(&record.result, &record.black_name, &record.white_name),
            end_reason: end_reason_code(record.end_reason).to_string(),
            duration_ms: record.replay.duration_ms,
            opening: record.opening.as_ref().map(|opening| MatchOpeningReport {
                policy: opening.policy.clone(),
                index: opening.index,
                ply_count: opening.ply_count,
                suite_index: opening.suite_index,
                template_index: opening.template_index,
                transform_index: opening.transform_index,
            }),
            move_count: move_cells.len(),
            move_cells,
            black_stats: black_stats.finish(),
            white_stats: white_stats.finish(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SideStatsReport {
    pub move_count: u32,
    pub search_move_count: u32,
    pub total_time_ms: u64,
    pub avg_search_time_ms: f64,
    #[serde(default)]
    pub search_nodes: u64,
    #[serde(default, alias = "prefilter_nodes")]
    pub safety_nodes: u64,
    #[serde(default)]
    pub corridor_nodes: u64,
    #[serde(default)]
    pub corridor_branch_probes: u64,
    #[serde(default)]
    pub corridor_max_depth: u32,
    #[serde(default)]
    pub corridor_width_exits: u64,
    #[serde(default)]
    pub corridor_depth_exits: u64,
    #[serde(default)]
    pub corridor_neutral_exits: u64,
    #[serde(default)]
    pub corridor_terminal_exits: u64,
    #[serde(default)]
    pub corridor_plies_followed: u64,
    #[serde(default)]
    pub corridor_own_plies_followed: u64,
    #[serde(default)]
    pub corridor_opponent_plies_followed: u64,
    #[serde(default)]
    pub corridor_proof_passes: u64,
    #[serde(default)]
    pub corridor_proof_completed: u64,
    #[serde(default)]
    pub corridor_proof_checks: u64,
    #[serde(default)]
    pub corridor_proof_active: u64,
    #[serde(default)]
    pub corridor_proof_quiet: u64,
    #[serde(default)]
    pub corridor_proof_static_exits: u64,
    #[serde(default)]
    pub corridor_proof_depth_exits: u64,
    #[serde(default)]
    pub corridor_proof_deadline_exits: u64,
    #[serde(default)]
    pub corridor_proof_terminal_exits: u64,
    #[serde(default)]
    pub corridor_proof_terminal_root_candidates: u64,
    #[serde(default)]
    pub corridor_proof_terminal_root_winning_candidates: u64,
    #[serde(default)]
    pub corridor_proof_terminal_root_losing_candidates: u64,
    #[serde(default)]
    pub corridor_proof_terminal_root_overrides: u64,
    #[serde(default)]
    pub corridor_proof_terminal_root_move_changes: u64,
    #[serde(default)]
    pub corridor_proof_terminal_root_move_confirmations: u64,
    #[serde(default)]
    pub corridor_proof_candidates_considered: u64,
    #[serde(default)]
    pub corridor_proof_wins: u64,
    #[serde(default)]
    pub corridor_proof_losses: u64,
    #[serde(default)]
    pub corridor_proof_unknown: u64,
    #[serde(default)]
    pub corridor_proof_deadline_skips: u64,
    #[serde(default)]
    pub corridor_proof_move_changes: u64,
    #[serde(default)]
    pub corridor_proof_move_confirmations: u64,
    #[serde(default)]
    pub corridor_proof_candidate_rank_total: u64,
    #[serde(default)]
    pub corridor_proof_candidate_rank_max: u64,
    #[serde(default)]
    pub corridor_proof_candidate_score_gap_total: u64,
    #[serde(default)]
    pub corridor_proof_candidate_score_gap_max: u64,
    #[serde(default)]
    pub corridor_proof_win_candidate_rank_total: u64,
    #[serde(default)]
    pub corridor_proof_win_candidate_rank_max: u64,
    #[serde(default)]
    pub total_nodes: u64,
    pub avg_nodes: f64,
    #[serde(default)]
    pub eval_calls: u64,
    #[serde(default)]
    pub avg_eval_calls: f64,
    #[serde(default)]
    pub line_shape_eval_calls: u64,
    #[serde(default)]
    pub line_shape_eval_ns: u64,
    #[serde(default)]
    pub avg_line_shape_eval_ns: f64,
    #[serde(default)]
    pub pattern_eval_calls: u64,
    #[serde(default)]
    pub pattern_eval_ns: u64,
    #[serde(default)]
    pub avg_pattern_eval_ns: f64,
    #[serde(default)]
    pub pattern_frame_queries: u64,
    #[serde(default)]
    pub pattern_frame_query_ns: u64,
    #[serde(default)]
    pub avg_pattern_frame_query_ns: f64,
    #[serde(default)]
    pub pattern_frame_updates: u64,
    #[serde(default)]
    pub pattern_frame_update_ns: u64,
    #[serde(default)]
    pub avg_pattern_frame_update_ns: f64,
    #[serde(default)]
    pub pattern_frame_shadow_checks: u64,
    #[serde(default)]
    pub pattern_frame_shadow_mismatches: u64,
    #[serde(default)]
    pub candidate_generations: u64,
    #[serde(default)]
    pub avg_candidate_generations: f64,
    #[serde(default)]
    pub candidate_moves_total: u64,
    #[serde(default)]
    pub avg_candidate_moves: f64,
    #[serde(default)]
    pub candidate_moves_max: u64,
    #[serde(default)]
    pub root_candidate_generations: u64,
    #[serde(default)]
    pub root_candidate_moves_total: u64,
    #[serde(default)]
    pub root_candidate_moves_max: u64,
    #[serde(default)]
    pub search_candidate_generations: u64,
    #[serde(default)]
    pub search_candidate_moves_total: u64,
    #[serde(default)]
    pub search_candidate_moves_max: u64,
    #[serde(default)]
    pub legality_checks: u64,
    #[serde(default)]
    pub avg_legality_checks: f64,
    #[serde(default)]
    pub illegal_moves_skipped: u64,
    #[serde(default)]
    pub root_legality_checks: u64,
    #[serde(default)]
    pub root_illegal_moves_skipped: u64,
    #[serde(default)]
    pub search_legality_checks: u64,
    #[serde(default)]
    pub search_illegal_moves_skipped: u64,
    #[serde(default)]
    pub renju_forbidden_prefilter_checks: u64,
    #[serde(default)]
    pub avg_renju_forbidden_prefilter_checks: f64,
    #[serde(default)]
    pub renju_forbidden_prefilter_ns: u64,
    #[serde(default)]
    pub avg_renju_forbidden_prefilter_ns: f64,
    #[serde(default)]
    pub renju_forbidden_checks: u64,
    #[serde(default)]
    pub avg_renju_forbidden_checks: f64,
    #[serde(default)]
    pub renju_forbidden_ns: u64,
    #[serde(default)]
    pub avg_renju_forbidden_ns: f64,
    #[serde(default)]
    pub renju_forbidden_search_gate_checks: u64,
    #[serde(default)]
    pub renju_forbidden_search_gate_ns: u64,
    #[serde(default)]
    pub renju_forbidden_pattern_checks: u64,
    #[serde(default)]
    pub renju_forbidden_pattern_ns: u64,
    #[serde(default)]
    pub renju_forbidden_threat_checks: u64,
    #[serde(default)]
    pub renju_forbidden_threat_ns: u64,
    #[serde(default)]
    pub renju_forbidden_other_checks: u64,
    #[serde(default)]
    pub renju_forbidden_other_ns: u64,
    #[serde(default)]
    pub renju_effective_filter_calls: u64,
    #[serde(default)]
    pub avg_renju_effective_filter_calls: f64,
    #[serde(default)]
    pub renju_effective_filter_ns: u64,
    #[serde(default)]
    pub avg_renju_effective_filter_ns: f64,
    #[serde(default)]
    pub renju_effective_filter_continuation_checks: u64,
    #[serde(default)]
    pub avg_renju_effective_filter_continuation_checks: f64,
    #[serde(default)]
    pub renju_effective_filter_continuation_ns: u64,
    #[serde(default)]
    pub avg_renju_effective_filter_continuation_ns: f64,
    #[serde(default)]
    pub stage_move_gen_ns: u64,
    #[serde(default)]
    pub stage_ordering_ns: u64,
    #[serde(default)]
    pub stage_eval_ns: u64,
    #[serde(default)]
    pub stage_threat_ns: u64,
    #[serde(default)]
    pub stage_proof_ns: u64,
    #[serde(default)]
    pub tactical_annotations: u64,
    #[serde(default)]
    pub root_tactical_annotations: u64,
    #[serde(default)]
    pub search_tactical_annotations: u64,
    #[serde(default)]
    pub threat_view_shadow_checks: u64,
    #[serde(default)]
    pub threat_view_shadow_mismatches: u64,
    #[serde(default)]
    pub threat_view_scan_queries: u64,
    #[serde(default)]
    pub threat_view_scan_ns: u64,
    #[serde(default)]
    pub threat_view_frontier_rebuilds: u64,
    #[serde(default)]
    pub threat_view_frontier_rebuild_ns: u64,
    #[serde(default)]
    pub threat_view_frontier_queries: u64,
    #[serde(default)]
    pub threat_view_frontier_query_ns: u64,
    #[serde(default)]
    pub threat_view_frontier_immediate_win_queries: u64,
    #[serde(default)]
    pub threat_view_frontier_immediate_win_query_ns: u64,
    #[serde(default)]
    pub threat_view_frontier_delta_captures: u64,
    #[serde(default)]
    pub threat_view_frontier_delta_capture_ns: u64,
    #[serde(default)]
    pub threat_view_frontier_move_fact_updates: u64,
    #[serde(default)]
    pub threat_view_frontier_move_fact_update_ns: u64,
    #[serde(default)]
    pub threat_view_frontier_annotation_dirty_marks: u64,
    #[serde(default)]
    pub threat_view_frontier_annotation_dirty_mark_ns: u64,
    #[serde(default)]
    pub threat_view_frontier_clean_annotation_queries: u64,
    #[serde(default)]
    pub threat_view_frontier_clean_annotation_query_ns: u64,
    #[serde(default)]
    pub threat_view_frontier_dirty_annotation_queries: u64,
    #[serde(default)]
    pub threat_view_frontier_dirty_annotation_query_ns: u64,
    #[serde(default)]
    pub threat_view_frontier_fallback_annotation_queries: u64,
    #[serde(default)]
    pub threat_view_frontier_fallback_annotation_query_ns: u64,
    #[serde(default)]
    pub threat_view_frontier_memo_annotation_queries: u64,
    #[serde(default)]
    pub threat_view_frontier_memo_annotation_query_ns: u64,
    #[serde(default)]
    pub child_limit_applications: u64,
    #[serde(default)]
    pub root_child_limit_applications: u64,
    #[serde(default)]
    pub search_child_limit_applications: u64,
    #[serde(default)]
    pub child_cap_hits: u64,
    #[serde(default)]
    pub root_child_cap_hits: u64,
    #[serde(default)]
    pub search_child_cap_hits: u64,
    #[serde(default)]
    pub child_moves_before_total: u64,
    #[serde(default)]
    pub root_child_moves_before_total: u64,
    #[serde(default)]
    pub search_child_moves_before_total: u64,
    #[serde(default)]
    pub child_moves_before_max: u64,
    #[serde(default)]
    pub root_child_moves_before_max: u64,
    #[serde(default)]
    pub search_child_moves_before_max: u64,
    #[serde(default)]
    pub child_moves_after_total: u64,
    #[serde(default)]
    pub root_child_moves_after_total: u64,
    #[serde(default)]
    pub search_child_moves_after_total: u64,
    #[serde(default)]
    pub child_moves_after_max: u64,
    #[serde(default)]
    pub root_child_moves_after_max: u64,
    #[serde(default)]
    pub search_child_moves_after_max: u64,
    #[serde(default)]
    pub avg_child_moves_before: f64,
    #[serde(default)]
    pub avg_child_moves_after: f64,
    #[serde(default)]
    pub tt_hits: u64,
    #[serde(default)]
    pub tt_cutoffs: u64,
    #[serde(default)]
    pub beta_cutoffs: u64,
    pub depth_sum: u64,
    pub avg_depth: f64,
    pub max_depth: u32,
    #[serde(default)]
    pub effective_depth_sum: u64,
    #[serde(default)]
    pub avg_effective_depth: f64,
    #[serde(default)]
    pub max_effective_depth: u32,
    #[serde(default)]
    pub depth_reached_counts: Vec<DepthCountReport>,
    pub budget_exhausted_count: u32,
    pub budget_exhausted_rate: f64,
    #[serde(default)]
    pub pooled_budget_moves: u32,
    #[serde(default)]
    pub pooled_budget_over_base_count: u32,
    #[serde(default)]
    pub pooled_budget_over_base_rate: f64,
    #[serde(default)]
    pub pooled_budget_reserve_exhausted_count: u32,
    #[serde(default)]
    pub pooled_budget_reserve_exhausted_rate: f64,
    #[serde(default)]
    pub pooled_budget_avg_reserve_before_ms: f64,
    #[serde(default)]
    pub pooled_budget_avg_reserve_after_ms: f64,
    #[serde(default)]
    pub pooled_budget_min_reserve_after_ms: u64,
    #[serde(default)]
    pub pooled_budget_max_move_budget_ms: u64,
}

mod aggregate;
mod provenance;

use aggregate::*;
use provenance::*;

#[cfg(test)]
mod tests;
