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

#[derive(Debug, Clone, Default)]
struct SideStatsAccumulator {
    move_count: u32,
    search_move_count: u32,
    total_time_ms: u64,
    search_nodes: u64,
    safety_nodes: u64,
    corridor_nodes: u64,
    corridor_branch_probes: u64,
    corridor_max_depth: u32,
    corridor_width_exits: u64,
    corridor_depth_exits: u64,
    corridor_neutral_exits: u64,
    corridor_terminal_exits: u64,
    corridor_plies_followed: u64,
    corridor_own_plies_followed: u64,
    corridor_opponent_plies_followed: u64,
    corridor_proof_passes: u64,
    corridor_proof_completed: u64,
    corridor_proof_checks: u64,
    corridor_proof_active: u64,
    corridor_proof_quiet: u64,
    corridor_proof_static_exits: u64,
    corridor_proof_depth_exits: u64,
    corridor_proof_deadline_exits: u64,
    corridor_proof_terminal_exits: u64,
    corridor_proof_terminal_root_candidates: u64,
    corridor_proof_terminal_root_winning_candidates: u64,
    corridor_proof_terminal_root_losing_candidates: u64,
    corridor_proof_terminal_root_overrides: u64,
    corridor_proof_terminal_root_move_changes: u64,
    corridor_proof_terminal_root_move_confirmations: u64,
    corridor_proof_candidates_considered: u64,
    corridor_proof_wins: u64,
    corridor_proof_losses: u64,
    corridor_proof_unknown: u64,
    corridor_proof_deadline_skips: u64,
    corridor_proof_move_changes: u64,
    corridor_proof_move_confirmations: u64,
    corridor_proof_candidate_rank_total: u64,
    corridor_proof_candidate_rank_max: u64,
    corridor_proof_candidate_score_gap_total: u64,
    corridor_proof_candidate_score_gap_max: u64,
    corridor_proof_win_candidate_rank_total: u64,
    corridor_proof_win_candidate_rank_max: u64,
    total_nodes: u64,
    eval_calls: u64,
    line_shape_eval_calls: u64,
    line_shape_eval_ns: u64,
    pattern_eval_calls: u64,
    pattern_eval_ns: u64,
    pattern_frame_queries: u64,
    pattern_frame_query_ns: u64,
    pattern_frame_updates: u64,
    pattern_frame_update_ns: u64,
    pattern_frame_shadow_checks: u64,
    pattern_frame_shadow_mismatches: u64,
    candidate_generations: u64,
    candidate_moves_total: u64,
    candidate_moves_max: u64,
    root_candidate_generations: u64,
    root_candidate_moves_total: u64,
    root_candidate_moves_max: u64,
    search_candidate_generations: u64,
    search_candidate_moves_total: u64,
    search_candidate_moves_max: u64,
    legality_checks: u64,
    illegal_moves_skipped: u64,
    root_legality_checks: u64,
    root_illegal_moves_skipped: u64,
    search_legality_checks: u64,
    search_illegal_moves_skipped: u64,
    renju_forbidden_prefilter_checks: u64,
    renju_forbidden_prefilter_ns: u64,
    renju_forbidden_checks: u64,
    renju_forbidden_ns: u64,
    renju_forbidden_search_gate_checks: u64,
    renju_forbidden_search_gate_ns: u64,
    renju_forbidden_pattern_checks: u64,
    renju_forbidden_pattern_ns: u64,
    renju_forbidden_threat_checks: u64,
    renju_forbidden_threat_ns: u64,
    renju_forbidden_other_checks: u64,
    renju_forbidden_other_ns: u64,
    renju_effective_filter_calls: u64,
    renju_effective_filter_ns: u64,
    renju_effective_filter_continuation_checks: u64,
    renju_effective_filter_continuation_ns: u64,
    stage_move_gen_ns: u64,
    stage_ordering_ns: u64,
    stage_eval_ns: u64,
    stage_threat_ns: u64,
    stage_proof_ns: u64,
    tactical_annotations: u64,
    root_tactical_annotations: u64,
    search_tactical_annotations: u64,
    threat_view_shadow_checks: u64,
    threat_view_shadow_mismatches: u64,
    threat_view_scan_queries: u64,
    threat_view_scan_ns: u64,
    threat_view_frontier_rebuilds: u64,
    threat_view_frontier_rebuild_ns: u64,
    threat_view_frontier_queries: u64,
    threat_view_frontier_query_ns: u64,
    threat_view_frontier_immediate_win_queries: u64,
    threat_view_frontier_immediate_win_query_ns: u64,
    threat_view_frontier_delta_captures: u64,
    threat_view_frontier_delta_capture_ns: u64,
    threat_view_frontier_move_fact_updates: u64,
    threat_view_frontier_move_fact_update_ns: u64,
    threat_view_frontier_annotation_dirty_marks: u64,
    threat_view_frontier_annotation_dirty_mark_ns: u64,
    threat_view_frontier_clean_annotation_queries: u64,
    threat_view_frontier_clean_annotation_query_ns: u64,
    threat_view_frontier_dirty_annotation_queries: u64,
    threat_view_frontier_dirty_annotation_query_ns: u64,
    threat_view_frontier_fallback_annotation_queries: u64,
    threat_view_frontier_fallback_annotation_query_ns: u64,
    threat_view_frontier_memo_annotation_queries: u64,
    threat_view_frontier_memo_annotation_query_ns: u64,
    child_limit_applications: u64,
    root_child_limit_applications: u64,
    search_child_limit_applications: u64,
    child_cap_hits: u64,
    root_child_cap_hits: u64,
    search_child_cap_hits: u64,
    child_moves_before_total: u64,
    root_child_moves_before_total: u64,
    search_child_moves_before_total: u64,
    child_moves_before_max: u64,
    root_child_moves_before_max: u64,
    search_child_moves_before_max: u64,
    child_moves_after_total: u64,
    root_child_moves_after_total: u64,
    search_child_moves_after_total: u64,
    child_moves_after_max: u64,
    root_child_moves_after_max: u64,
    search_child_moves_after_max: u64,
    tt_hits: u64,
    tt_cutoffs: u64,
    beta_cutoffs: u64,
    depth_sum: u64,
    max_depth: u32,
    effective_depth_sum: u64,
    max_effective_depth: u32,
    depth_reached_counts: BTreeMap<u32, u32>,
    budget_exhausted_count: u32,
    pooled_budget_moves: u32,
    pooled_budget_over_base_count: u32,
    pooled_budget_reserve_exhausted_count: u32,
    pooled_budget_reserve_before_total_ms: u64,
    pooled_budget_reserve_after_total_ms: u64,
    pooled_budget_min_reserve_after_ms: Option<u64>,
    pooled_budget_max_move_budget_ms: u64,
}

impl SideStatsAccumulator {
    fn record_move(&mut self, time_ms: u64, trace: Option<&Value>) {
        self.move_count += 1;
        self.total_time_ms += time_ms;

        let Some(trace) = trace else {
            return;
        };

        self.search_move_count += 1;
        self.search_nodes += trace_value_u64(trace, "nodes");
        self.safety_nodes += trace_value_u64(trace, "safety_nodes");
        self.total_nodes += trace_value_u64(trace, "total_nodes");
        if let Some(corridor) = trace.get("corridor") {
            self.corridor_nodes += trace_value_u64(corridor, "search_nodes");
            self.corridor_branch_probes += trace_value_u64(corridor, "branch_probes");
            self.corridor_max_depth = self
                .corridor_max_depth
                .max(trace_value_u64(corridor, "max_depth_reached") as u32);
        }
        if let Some(metrics) = trace.get("metrics") {
            self.eval_calls += trace_value_u64(metrics, "eval_calls");
            self.line_shape_eval_calls += trace_value_u64(metrics, "line_shape_eval_calls");
            self.line_shape_eval_ns += trace_value_u64(metrics, "line_shape_eval_ns");
            self.pattern_eval_calls += trace_value_u64(metrics, "pattern_eval_calls");
            self.pattern_eval_ns += trace_value_u64(metrics, "pattern_eval_ns");
            self.pattern_frame_queries += trace_value_u64(metrics, "pattern_frame_queries");
            self.pattern_frame_query_ns += trace_value_u64(metrics, "pattern_frame_query_ns");
            self.pattern_frame_updates += trace_value_u64(metrics, "pattern_frame_updates");
            self.pattern_frame_update_ns += trace_value_u64(metrics, "pattern_frame_update_ns");
            self.pattern_frame_shadow_checks +=
                trace_value_u64(metrics, "pattern_frame_shadow_checks");
            self.pattern_frame_shadow_mismatches +=
                trace_value_u64(metrics, "pattern_frame_shadow_mismatches");
            self.candidate_generations += trace_value_u64(metrics, "candidate_generations");
            self.candidate_moves_total += trace_value_u64(metrics, "candidate_moves_total");
            self.candidate_moves_max = self
                .candidate_moves_max
                .max(trace_value_u64(metrics, "candidate_moves_max"));
            self.root_candidate_generations +=
                trace_value_u64(metrics, "root_candidate_generations");
            self.root_candidate_moves_total +=
                trace_value_u64(metrics, "root_candidate_moves_total");
            self.root_candidate_moves_max = self
                .root_candidate_moves_max
                .max(trace_value_u64(metrics, "root_candidate_moves_max"));
            self.search_candidate_generations +=
                trace_value_u64(metrics, "search_candidate_generations");
            self.search_candidate_moves_total +=
                trace_value_u64(metrics, "search_candidate_moves_total");
            self.search_candidate_moves_max = self
                .search_candidate_moves_max
                .max(trace_value_u64(metrics, "search_candidate_moves_max"));
            self.legality_checks += trace_value_u64(metrics, "legality_checks");
            self.illegal_moves_skipped += trace_value_u64(metrics, "illegal_moves_skipped");
            self.root_legality_checks += trace_value_u64(metrics, "root_legality_checks");
            self.root_illegal_moves_skipped +=
                trace_value_u64(metrics, "root_illegal_moves_skipped");
            self.search_legality_checks += trace_value_u64(metrics, "search_legality_checks");
            self.search_illegal_moves_skipped +=
                trace_value_u64(metrics, "search_illegal_moves_skipped");
            self.renju_forbidden_prefilter_checks +=
                trace_value_u64(metrics, "renju_forbidden_prefilter_checks");
            self.renju_forbidden_prefilter_ns +=
                trace_value_u64(metrics, "renju_forbidden_prefilter_ns");
            self.renju_forbidden_checks += trace_value_u64(metrics, "renju_forbidden_checks");
            self.renju_forbidden_ns += trace_value_u64(metrics, "renju_forbidden_ns");
            self.renju_forbidden_search_gate_checks +=
                trace_value_u64(metrics, "renju_forbidden_search_gate_checks");
            self.renju_forbidden_search_gate_ns +=
                trace_value_u64(metrics, "renju_forbidden_search_gate_ns");
            self.renju_forbidden_pattern_checks +=
                trace_value_u64(metrics, "renju_forbidden_pattern_checks");
            self.renju_forbidden_pattern_ns +=
                trace_value_u64(metrics, "renju_forbidden_pattern_ns");
            self.renju_forbidden_threat_checks +=
                trace_value_u64(metrics, "renju_forbidden_threat_checks");
            self.renju_forbidden_threat_ns += trace_value_u64(metrics, "renju_forbidden_threat_ns");
            self.renju_forbidden_other_checks +=
                trace_value_u64(metrics, "renju_forbidden_other_checks");
            self.renju_forbidden_other_ns += trace_value_u64(metrics, "renju_forbidden_other_ns");
            self.renju_effective_filter_calls +=
                trace_value_u64(metrics, "renju_effective_filter_calls");
            self.renju_effective_filter_ns += trace_value_u64(metrics, "renju_effective_filter_ns");
            self.renju_effective_filter_continuation_checks +=
                trace_value_u64(metrics, "renju_effective_filter_continuation_checks");
            self.renju_effective_filter_continuation_ns +=
                trace_value_u64(metrics, "renju_effective_filter_continuation_ns");
            self.stage_move_gen_ns += trace_value_u64(metrics, "stage_move_gen_ns");
            self.stage_ordering_ns += trace_value_u64(metrics, "stage_ordering_ns");
            self.stage_eval_ns += trace_value_u64(metrics, "stage_eval_ns");
            self.stage_threat_ns += trace_value_u64(metrics, "stage_threat_ns");
            self.stage_proof_ns += trace_value_u64(metrics, "stage_proof_ns");
            self.tactical_annotations += trace_value_u64(metrics, "tactical_annotations");
            self.root_tactical_annotations += trace_value_u64(metrics, "root_tactical_annotations");
            self.search_tactical_annotations +=
                trace_value_u64(metrics, "search_tactical_annotations");
            self.threat_view_shadow_checks += trace_value_u64(metrics, "threat_view_shadow_checks");
            self.threat_view_shadow_mismatches +=
                trace_value_u64(metrics, "threat_view_shadow_mismatches");
            self.threat_view_scan_queries += trace_value_u64(metrics, "threat_view_scan_queries");
            self.threat_view_scan_ns += trace_value_u64(metrics, "threat_view_scan_ns");
            self.threat_view_frontier_rebuilds +=
                trace_value_u64(metrics, "threat_view_frontier_rebuilds");
            self.threat_view_frontier_rebuild_ns +=
                trace_value_u64(metrics, "threat_view_frontier_rebuild_ns");
            self.threat_view_frontier_queries +=
                trace_value_u64(metrics, "threat_view_frontier_queries");
            self.threat_view_frontier_query_ns +=
                trace_value_u64(metrics, "threat_view_frontier_query_ns");
            self.threat_view_frontier_immediate_win_queries +=
                trace_value_u64(metrics, "threat_view_frontier_immediate_win_queries");
            self.threat_view_frontier_immediate_win_query_ns +=
                trace_value_u64(metrics, "threat_view_frontier_immediate_win_query_ns");
            self.threat_view_frontier_delta_captures +=
                trace_value_u64(metrics, "threat_view_frontier_delta_captures");
            self.threat_view_frontier_delta_capture_ns +=
                trace_value_u64(metrics, "threat_view_frontier_delta_capture_ns");
            self.threat_view_frontier_move_fact_updates +=
                trace_value_u64(metrics, "threat_view_frontier_move_fact_updates");
            self.threat_view_frontier_move_fact_update_ns +=
                trace_value_u64(metrics, "threat_view_frontier_move_fact_update_ns");
            self.threat_view_frontier_annotation_dirty_marks +=
                trace_value_u64(metrics, "threat_view_frontier_annotation_dirty_marks");
            self.threat_view_frontier_annotation_dirty_mark_ns +=
                trace_value_u64(metrics, "threat_view_frontier_annotation_dirty_mark_ns");
            self.threat_view_frontier_clean_annotation_queries +=
                trace_value_u64(metrics, "threat_view_frontier_clean_annotation_queries");
            self.threat_view_frontier_clean_annotation_query_ns +=
                trace_value_u64(metrics, "threat_view_frontier_clean_annotation_query_ns");
            self.threat_view_frontier_dirty_annotation_queries +=
                trace_value_u64(metrics, "threat_view_frontier_dirty_annotation_queries");
            self.threat_view_frontier_dirty_annotation_query_ns +=
                trace_value_u64(metrics, "threat_view_frontier_dirty_annotation_query_ns");
            self.threat_view_frontier_fallback_annotation_queries +=
                trace_value_u64(metrics, "threat_view_frontier_fallback_annotation_queries");
            self.threat_view_frontier_fallback_annotation_query_ns +=
                trace_value_u64(metrics, "threat_view_frontier_fallback_annotation_query_ns");
            self.threat_view_frontier_memo_annotation_queries +=
                trace_value_u64(metrics, "threat_view_frontier_memo_annotation_queries");
            self.threat_view_frontier_memo_annotation_query_ns +=
                trace_value_u64(metrics, "threat_view_frontier_memo_annotation_query_ns");
            self.child_limit_applications += trace_value_u64(metrics, "child_limit_applications");
            self.root_child_limit_applications +=
                trace_value_u64(metrics, "root_child_limit_applications");
            self.search_child_limit_applications +=
                trace_value_u64(metrics, "search_child_limit_applications");
            self.child_cap_hits += trace_value_u64(metrics, "child_cap_hits");
            self.root_child_cap_hits += trace_value_u64(metrics, "root_child_cap_hits");
            self.search_child_cap_hits += trace_value_u64(metrics, "search_child_cap_hits");
            self.child_moves_before_total += trace_value_u64(metrics, "child_moves_before_total");
            self.root_child_moves_before_total +=
                trace_value_u64(metrics, "root_child_moves_before_total");
            self.search_child_moves_before_total +=
                trace_value_u64(metrics, "search_child_moves_before_total");
            self.child_moves_before_max = self
                .child_moves_before_max
                .max(trace_value_u64(metrics, "child_moves_before_max"));
            self.root_child_moves_before_max = self
                .root_child_moves_before_max
                .max(trace_value_u64(metrics, "root_child_moves_before_max"));
            self.search_child_moves_before_max = self
                .search_child_moves_before_max
                .max(trace_value_u64(metrics, "search_child_moves_before_max"));
            self.child_moves_after_total += trace_value_u64(metrics, "child_moves_after_total");
            self.root_child_moves_after_total +=
                trace_value_u64(metrics, "root_child_moves_after_total");
            self.search_child_moves_after_total +=
                trace_value_u64(metrics, "search_child_moves_after_total");
            self.child_moves_after_max = self
                .child_moves_after_max
                .max(trace_value_u64(metrics, "child_moves_after_max"));
            self.root_child_moves_after_max = self
                .root_child_moves_after_max
                .max(trace_value_u64(metrics, "root_child_moves_after_max"));
            self.search_child_moves_after_max = self
                .search_child_moves_after_max
                .max(trace_value_u64(metrics, "search_child_moves_after_max"));
            self.tt_hits += trace_value_u64(metrics, "tt_hits");
            self.tt_cutoffs += trace_value_u64(metrics, "tt_cutoffs");
            self.beta_cutoffs += trace_value_u64(metrics, "beta_cutoffs");
            self.corridor_width_exits += trace_value_u64(metrics, "corridor_width_exits");
            self.corridor_depth_exits += trace_value_u64(metrics, "corridor_depth_exits");
            self.corridor_neutral_exits += trace_value_u64(metrics, "corridor_neutral_exits");
            self.corridor_terminal_exits += trace_value_u64(metrics, "corridor_terminal_exits");
            self.corridor_plies_followed += trace_value_u64(metrics, "corridor_plies_followed");
            self.corridor_own_plies_followed +=
                trace_value_u64(metrics, "corridor_own_plies_followed");
            self.corridor_opponent_plies_followed +=
                trace_value_u64(metrics, "corridor_opponent_plies_followed");
            self.corridor_proof_passes += trace_value_u64(metrics, "corridor_proof_passes");
            self.corridor_proof_completed += trace_value_u64(metrics, "corridor_proof_completed");
            self.corridor_proof_checks += trace_value_u64(metrics, "corridor_proof_checks");
            self.corridor_proof_active += trace_value_u64(metrics, "corridor_proof_active");
            self.corridor_proof_quiet += trace_value_u64(metrics, "corridor_proof_quiet");
            self.corridor_proof_static_exits +=
                trace_value_u64(metrics, "corridor_proof_static_exits");
            self.corridor_proof_depth_exits +=
                trace_value_u64(metrics, "corridor_proof_depth_exits");
            self.corridor_proof_deadline_exits +=
                trace_value_u64(metrics, "corridor_proof_deadline_exits");
            self.corridor_proof_terminal_exits +=
                trace_value_u64(metrics, "corridor_proof_terminal_exits");
            self.corridor_proof_terminal_root_candidates +=
                trace_value_u64(metrics, "corridor_proof_terminal_root_candidates");
            self.corridor_proof_terminal_root_winning_candidates +=
                trace_value_u64(metrics, "corridor_proof_terminal_root_winning_candidates");
            self.corridor_proof_terminal_root_losing_candidates +=
                trace_value_u64(metrics, "corridor_proof_terminal_root_losing_candidates");
            self.corridor_proof_terminal_root_overrides +=
                trace_value_u64(metrics, "corridor_proof_terminal_root_overrides");
            self.corridor_proof_terminal_root_move_changes +=
                trace_value_u64(metrics, "corridor_proof_terminal_root_move_changes");
            self.corridor_proof_terminal_root_move_confirmations +=
                trace_value_u64(metrics, "corridor_proof_terminal_root_move_confirmations");
            self.corridor_proof_candidates_considered +=
                trace_value_u64(metrics, "corridor_proof_candidates_considered");
            self.corridor_proof_wins += trace_value_u64(metrics, "corridor_proof_wins");
            self.corridor_proof_losses += trace_value_u64(metrics, "corridor_proof_losses");
            self.corridor_proof_unknown += trace_value_u64(metrics, "corridor_proof_unknown");
            self.corridor_proof_deadline_skips +=
                trace_value_u64(metrics, "corridor_proof_deadline_skips");
            self.corridor_proof_move_changes +=
                trace_value_u64(metrics, "corridor_proof_move_changes");
            self.corridor_proof_move_confirmations +=
                trace_value_u64(metrics, "corridor_proof_move_confirmations");
            self.corridor_proof_candidate_rank_total +=
                trace_value_u64(metrics, "corridor_proof_candidate_rank_total");
            self.corridor_proof_candidate_rank_max = self.corridor_proof_candidate_rank_max.max(
                trace_value_u64(metrics, "corridor_proof_candidate_rank_max"),
            );
            self.corridor_proof_candidate_score_gap_total +=
                trace_value_u64(metrics, "corridor_proof_candidate_score_gap_total");
            self.corridor_proof_candidate_score_gap_max = self
                .corridor_proof_candidate_score_gap_max
                .max(trace_value_u64(
                    metrics,
                    "corridor_proof_candidate_score_gap_max",
                ));
            self.corridor_proof_win_candidate_rank_total +=
                trace_value_u64(metrics, "corridor_proof_win_candidate_rank_total");
            self.corridor_proof_win_candidate_rank_max = self
                .corridor_proof_win_candidate_rank_max
                .max(trace_value_u64(
                    metrics,
                    "corridor_proof_win_candidate_rank_max",
                ));
        }
        if let Some(depth) = trace.get("depth").and_then(Value::as_u64) {
            self.depth_sum += depth;
            self.max_depth = self.max_depth.max(depth as u32);
            *self.depth_reached_counts.entry(depth as u32).or_insert(0) += 1;
            let effective_depth = trace
                .get("effective_depth")
                .and_then(Value::as_u64)
                .unwrap_or(depth);
            self.effective_depth_sum += effective_depth;
            self.max_effective_depth = self.max_effective_depth.max(effective_depth as u32);
        }
        if trace
            .get("budget_exhausted")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            self.budget_exhausted_count += 1;
        }
        if let Some(pool) = trace.get("budget_pool") {
            self.pooled_budget_moves += 1;
            let reserve_before_ms = trace_value_u64(pool, "reserve_before_ms");
            let reserve_after_ms = trace_value_u64(pool, "reserve_after_ms");
            self.pooled_budget_reserve_before_total_ms += reserve_before_ms;
            self.pooled_budget_reserve_after_total_ms += reserve_after_ms;
            self.pooled_budget_min_reserve_after_ms = Some(
                self.pooled_budget_min_reserve_after_ms
                    .map_or(reserve_after_ms, |current| current.min(reserve_after_ms)),
            );
            self.pooled_budget_max_move_budget_ms = self
                .pooled_budget_max_move_budget_ms
                .max(trace_value_u64(pool, "move_budget_ms"));
            if trace_value_u64(pool, "consumed_ms") > trace_value_u64(pool, "base_ms")
                || reserve_after_ms < reserve_before_ms
            {
                self.pooled_budget_over_base_count += 1;
            }
            if pool
                .get("budget_exhausted")
                .or_else(|| pool.get("reserve_exhausted"))
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                self.pooled_budget_reserve_exhausted_count += 1;
            }
        }
    }

    fn add_report(&mut self, stats: &SideStatsReport) {
        self.move_count += stats.move_count;
        self.search_move_count += stats.search_move_count;
        self.total_time_ms += stats.total_time_ms;
        self.search_nodes += stats.search_nodes;
        self.safety_nodes += stats.safety_nodes;
        self.corridor_nodes += stats.corridor_nodes;
        self.corridor_branch_probes += stats.corridor_branch_probes;
        self.corridor_max_depth = self.corridor_max_depth.max(stats.corridor_max_depth);
        self.corridor_width_exits += stats.corridor_width_exits;
        self.corridor_depth_exits += stats.corridor_depth_exits;
        self.corridor_neutral_exits += stats.corridor_neutral_exits;
        self.corridor_terminal_exits += stats.corridor_terminal_exits;
        self.corridor_plies_followed += stats.corridor_plies_followed;
        self.corridor_own_plies_followed += stats.corridor_own_plies_followed;
        self.corridor_opponent_plies_followed += stats.corridor_opponent_plies_followed;
        self.corridor_proof_passes += stats.corridor_proof_passes;
        self.corridor_proof_completed += stats.corridor_proof_completed;
        self.corridor_proof_checks += stats.corridor_proof_checks;
        self.corridor_proof_active += stats.corridor_proof_active;
        self.corridor_proof_quiet += stats.corridor_proof_quiet;
        self.corridor_proof_static_exits += stats.corridor_proof_static_exits;
        self.corridor_proof_depth_exits += stats.corridor_proof_depth_exits;
        self.corridor_proof_deadline_exits += stats.corridor_proof_deadline_exits;
        self.corridor_proof_terminal_exits += stats.corridor_proof_terminal_exits;
        self.corridor_proof_terminal_root_candidates +=
            stats.corridor_proof_terminal_root_candidates;
        self.corridor_proof_terminal_root_winning_candidates +=
            stats.corridor_proof_terminal_root_winning_candidates;
        self.corridor_proof_terminal_root_losing_candidates +=
            stats.corridor_proof_terminal_root_losing_candidates;
        self.corridor_proof_terminal_root_overrides += stats.corridor_proof_terminal_root_overrides;
        self.corridor_proof_terminal_root_move_changes +=
            stats.corridor_proof_terminal_root_move_changes;
        self.corridor_proof_terminal_root_move_confirmations +=
            stats.corridor_proof_terminal_root_move_confirmations;
        self.corridor_proof_candidates_considered += stats.corridor_proof_candidates_considered;
        self.corridor_proof_wins += stats.corridor_proof_wins;
        self.corridor_proof_losses += stats.corridor_proof_losses;
        self.corridor_proof_unknown += stats.corridor_proof_unknown;
        self.corridor_proof_deadline_skips += stats.corridor_proof_deadline_skips;
        self.corridor_proof_move_changes += stats.corridor_proof_move_changes;
        self.corridor_proof_move_confirmations += stats.corridor_proof_move_confirmations;
        self.corridor_proof_candidate_rank_total += stats.corridor_proof_candidate_rank_total;
        self.corridor_proof_candidate_rank_max = self
            .corridor_proof_candidate_rank_max
            .max(stats.corridor_proof_candidate_rank_max);
        self.corridor_proof_candidate_score_gap_total +=
            stats.corridor_proof_candidate_score_gap_total;
        self.corridor_proof_candidate_score_gap_max = self
            .corridor_proof_candidate_score_gap_max
            .max(stats.corridor_proof_candidate_score_gap_max);
        self.corridor_proof_win_candidate_rank_total +=
            stats.corridor_proof_win_candidate_rank_total;
        self.corridor_proof_win_candidate_rank_max = self
            .corridor_proof_win_candidate_rank_max
            .max(stats.corridor_proof_win_candidate_rank_max);
        self.total_nodes += stats.total_nodes;
        self.eval_calls += stats.eval_calls;
        self.line_shape_eval_calls += stats.line_shape_eval_calls;
        self.line_shape_eval_ns += stats.line_shape_eval_ns;
        self.pattern_eval_calls += stats.pattern_eval_calls;
        self.pattern_eval_ns += stats.pattern_eval_ns;
        self.pattern_frame_queries += stats.pattern_frame_queries;
        self.pattern_frame_query_ns += stats.pattern_frame_query_ns;
        self.pattern_frame_updates += stats.pattern_frame_updates;
        self.pattern_frame_update_ns += stats.pattern_frame_update_ns;
        self.pattern_frame_shadow_checks += stats.pattern_frame_shadow_checks;
        self.pattern_frame_shadow_mismatches += stats.pattern_frame_shadow_mismatches;
        self.candidate_generations += stats.candidate_generations;
        self.candidate_moves_total += stats.candidate_moves_total;
        self.candidate_moves_max = self.candidate_moves_max.max(stats.candidate_moves_max);
        self.root_candidate_generations += stats.root_candidate_generations;
        self.root_candidate_moves_total += stats.root_candidate_moves_total;
        self.root_candidate_moves_max = self
            .root_candidate_moves_max
            .max(stats.root_candidate_moves_max);
        self.search_candidate_generations += stats.search_candidate_generations;
        self.search_candidate_moves_total += stats.search_candidate_moves_total;
        self.search_candidate_moves_max = self
            .search_candidate_moves_max
            .max(stats.search_candidate_moves_max);
        self.legality_checks += stats.legality_checks;
        self.illegal_moves_skipped += stats.illegal_moves_skipped;
        self.root_legality_checks += stats.root_legality_checks;
        self.root_illegal_moves_skipped += stats.root_illegal_moves_skipped;
        self.search_legality_checks += stats.search_legality_checks;
        self.search_illegal_moves_skipped += stats.search_illegal_moves_skipped;
        self.renju_forbidden_prefilter_checks += stats.renju_forbidden_prefilter_checks;
        self.renju_forbidden_prefilter_ns += stats.renju_forbidden_prefilter_ns;
        self.renju_forbidden_checks += stats.renju_forbidden_checks;
        self.renju_forbidden_ns += stats.renju_forbidden_ns;
        self.renju_forbidden_search_gate_checks += stats.renju_forbidden_search_gate_checks;
        self.renju_forbidden_search_gate_ns += stats.renju_forbidden_search_gate_ns;
        self.renju_forbidden_pattern_checks += stats.renju_forbidden_pattern_checks;
        self.renju_forbidden_pattern_ns += stats.renju_forbidden_pattern_ns;
        self.renju_forbidden_threat_checks += stats.renju_forbidden_threat_checks;
        self.renju_forbidden_threat_ns += stats.renju_forbidden_threat_ns;
        self.renju_forbidden_other_checks += stats.renju_forbidden_other_checks;
        self.renju_forbidden_other_ns += stats.renju_forbidden_other_ns;
        self.renju_effective_filter_calls += stats.renju_effective_filter_calls;
        self.renju_effective_filter_ns += stats.renju_effective_filter_ns;
        self.renju_effective_filter_continuation_checks +=
            stats.renju_effective_filter_continuation_checks;
        self.renju_effective_filter_continuation_ns += stats.renju_effective_filter_continuation_ns;
        self.stage_move_gen_ns += stats.stage_move_gen_ns;
        self.stage_ordering_ns += stats.stage_ordering_ns;
        self.stage_eval_ns += stats.stage_eval_ns;
        self.stage_threat_ns += stats.stage_threat_ns;
        self.stage_proof_ns += stats.stage_proof_ns;
        self.tactical_annotations += stats.tactical_annotations;
        self.root_tactical_annotations += stats.root_tactical_annotations;
        self.search_tactical_annotations += stats.search_tactical_annotations;
        self.threat_view_shadow_checks += stats.threat_view_shadow_checks;
        self.threat_view_shadow_mismatches += stats.threat_view_shadow_mismatches;
        self.threat_view_scan_queries += stats.threat_view_scan_queries;
        self.threat_view_scan_ns += stats.threat_view_scan_ns;
        self.threat_view_frontier_rebuilds += stats.threat_view_frontier_rebuilds;
        self.threat_view_frontier_rebuild_ns += stats.threat_view_frontier_rebuild_ns;
        self.threat_view_frontier_queries += stats.threat_view_frontier_queries;
        self.threat_view_frontier_query_ns += stats.threat_view_frontier_query_ns;
        self.threat_view_frontier_immediate_win_queries +=
            stats.threat_view_frontier_immediate_win_queries;
        self.threat_view_frontier_immediate_win_query_ns +=
            stats.threat_view_frontier_immediate_win_query_ns;
        self.threat_view_frontier_delta_captures += stats.threat_view_frontier_delta_captures;
        self.threat_view_frontier_delta_capture_ns += stats.threat_view_frontier_delta_capture_ns;
        self.threat_view_frontier_move_fact_updates += stats.threat_view_frontier_move_fact_updates;
        self.threat_view_frontier_move_fact_update_ns +=
            stats.threat_view_frontier_move_fact_update_ns;
        self.threat_view_frontier_annotation_dirty_marks +=
            stats.threat_view_frontier_annotation_dirty_marks;
        self.threat_view_frontier_annotation_dirty_mark_ns +=
            stats.threat_view_frontier_annotation_dirty_mark_ns;
        self.threat_view_frontier_clean_annotation_queries +=
            stats.threat_view_frontier_clean_annotation_queries;
        self.threat_view_frontier_clean_annotation_query_ns +=
            stats.threat_view_frontier_clean_annotation_query_ns;
        self.threat_view_frontier_dirty_annotation_queries +=
            stats.threat_view_frontier_dirty_annotation_queries;
        self.threat_view_frontier_dirty_annotation_query_ns +=
            stats.threat_view_frontier_dirty_annotation_query_ns;
        self.threat_view_frontier_fallback_annotation_queries +=
            stats.threat_view_frontier_fallback_annotation_queries;
        self.threat_view_frontier_fallback_annotation_query_ns +=
            stats.threat_view_frontier_fallback_annotation_query_ns;
        self.threat_view_frontier_memo_annotation_queries +=
            stats.threat_view_frontier_memo_annotation_queries;
        self.threat_view_frontier_memo_annotation_query_ns +=
            stats.threat_view_frontier_memo_annotation_query_ns;
        self.child_limit_applications += stats.child_limit_applications;
        self.root_child_limit_applications += stats.root_child_limit_applications;
        self.search_child_limit_applications += stats.search_child_limit_applications;
        self.child_cap_hits += stats.child_cap_hits;
        self.root_child_cap_hits += stats.root_child_cap_hits;
        self.search_child_cap_hits += stats.search_child_cap_hits;
        self.child_moves_before_total += stats.child_moves_before_total;
        self.root_child_moves_before_total += stats.root_child_moves_before_total;
        self.search_child_moves_before_total += stats.search_child_moves_before_total;
        self.child_moves_before_max = self
            .child_moves_before_max
            .max(stats.child_moves_before_max);
        self.root_child_moves_before_max = self
            .root_child_moves_before_max
            .max(stats.root_child_moves_before_max);
        self.search_child_moves_before_max = self
            .search_child_moves_before_max
            .max(stats.search_child_moves_before_max);
        self.child_moves_after_total += stats.child_moves_after_total;
        self.root_child_moves_after_total += stats.root_child_moves_after_total;
        self.search_child_moves_after_total += stats.search_child_moves_after_total;
        self.child_moves_after_max = self.child_moves_after_max.max(stats.child_moves_after_max);
        self.root_child_moves_after_max = self
            .root_child_moves_after_max
            .max(stats.root_child_moves_after_max);
        self.search_child_moves_after_max = self
            .search_child_moves_after_max
            .max(stats.search_child_moves_after_max);
        self.tt_hits += stats.tt_hits;
        self.tt_cutoffs += stats.tt_cutoffs;
        self.beta_cutoffs += stats.beta_cutoffs;
        self.depth_sum += stats.depth_sum;
        self.max_depth = self.max_depth.max(stats.max_depth);
        self.effective_depth_sum += stats.effective_depth_sum;
        self.max_effective_depth = self.max_effective_depth.max(stats.max_effective_depth);
        for count in &stats.depth_reached_counts {
            *self.depth_reached_counts.entry(count.depth).or_insert(0) += count.count;
        }
        self.budget_exhausted_count += stats.budget_exhausted_count;
        self.pooled_budget_moves += stats.pooled_budget_moves;
        self.pooled_budget_over_base_count += stats.pooled_budget_over_base_count;
        self.pooled_budget_reserve_exhausted_count += stats.pooled_budget_reserve_exhausted_count;
        self.pooled_budget_reserve_before_total_ms += (stats.pooled_budget_avg_reserve_before_ms
            * stats.pooled_budget_moves as f64)
            .round() as u64;
        self.pooled_budget_reserve_after_total_ms += (stats.pooled_budget_avg_reserve_after_ms
            * stats.pooled_budget_moves as f64)
            .round() as u64;
        if stats.pooled_budget_moves > 0 {
            self.pooled_budget_min_reserve_after_ms = Some(
                self.pooled_budget_min_reserve_after_ms
                    .map_or(stats.pooled_budget_min_reserve_after_ms, |current| {
                        current.min(stats.pooled_budget_min_reserve_after_ms)
                    }),
            );
        }
        self.pooled_budget_max_move_budget_ms = self
            .pooled_budget_max_move_budget_ms
            .max(stats.pooled_budget_max_move_budget_ms);
    }

    fn finish(self) -> SideStatsReport {
        let avg_search_time_ms = avg(self.total_time_ms as f64, self.search_move_count);
        let avg_nodes = avg(self.total_nodes as f64, self.search_move_count);
        let avg_eval_calls = avg(self.eval_calls as f64, self.search_move_count);
        let avg_line_shape_eval_ns = avg(
            self.line_shape_eval_ns as f64,
            self.line_shape_eval_calls as u32,
        );
        let avg_pattern_eval_ns = avg(self.pattern_eval_ns as f64, self.pattern_eval_calls as u32);
        let avg_pattern_frame_query_ns = avg(
            self.pattern_frame_query_ns as f64,
            self.pattern_frame_queries as u32,
        );
        let avg_pattern_frame_update_ns = avg(
            self.pattern_frame_update_ns as f64,
            self.pattern_frame_updates as u32,
        );
        let avg_candidate_generations =
            avg(self.candidate_generations as f64, self.search_move_count);
        let avg_candidate_moves = avg(
            self.candidate_moves_total as f64,
            self.candidate_generations as u32,
        );
        let avg_child_moves_before = avg(
            self.child_moves_before_total as f64,
            self.child_limit_applications as u32,
        );
        let avg_child_moves_after = avg(
            self.child_moves_after_total as f64,
            self.child_limit_applications as u32,
        );
        let avg_legality_checks = avg(self.legality_checks as f64, self.search_move_count);
        let avg_renju_forbidden_prefilter_checks = avg(
            self.renju_forbidden_prefilter_checks as f64,
            self.search_move_count,
        );
        let avg_renju_forbidden_prefilter_ns = avg(
            self.renju_forbidden_prefilter_ns as f64,
            self.renju_forbidden_prefilter_checks as u32,
        );
        let avg_renju_forbidden_checks =
            avg(self.renju_forbidden_checks as f64, self.search_move_count);
        let avg_renju_forbidden_ns = avg(
            self.renju_forbidden_ns as f64,
            self.renju_forbidden_checks as u32,
        );
        let avg_renju_effective_filter_calls = avg(
            self.renju_effective_filter_calls as f64,
            self.search_move_count,
        );
        let avg_renju_effective_filter_ns = avg(
            self.renju_effective_filter_ns as f64,
            self.renju_effective_filter_calls as u32,
        );
        let avg_renju_effective_filter_continuation_checks = avg(
            self.renju_effective_filter_continuation_checks as f64,
            self.search_move_count,
        );
        let avg_renju_effective_filter_continuation_ns = avg(
            self.renju_effective_filter_continuation_ns as f64,
            self.renju_effective_filter_continuation_checks as u32,
        );
        let avg_depth = avg(self.depth_sum as f64, self.search_move_count);
        let avg_effective_depth = avg(self.effective_depth_sum as f64, self.search_move_count);
        let budget_exhausted_rate = avg(self.budget_exhausted_count as f64, self.search_move_count);
        let pooled_budget_over_base_rate = avg(
            self.pooled_budget_over_base_count as f64,
            self.pooled_budget_moves,
        );
        let pooled_budget_reserve_exhausted_rate = avg(
            self.pooled_budget_reserve_exhausted_count as f64,
            self.pooled_budget_moves,
        );
        let pooled_budget_avg_reserve_before_ms = avg(
            self.pooled_budget_reserve_before_total_ms as f64,
            self.pooled_budget_moves,
        );
        let pooled_budget_avg_reserve_after_ms = avg(
            self.pooled_budget_reserve_after_total_ms as f64,
            self.pooled_budget_moves,
        );
        let depth_reached_counts = self
            .depth_reached_counts
            .into_iter()
            .map(|(depth, count)| DepthCountReport { depth, count })
            .collect();

        SideStatsReport {
            move_count: self.move_count,
            search_move_count: self.search_move_count,
            total_time_ms: self.total_time_ms,
            avg_search_time_ms,
            search_nodes: self.search_nodes,
            safety_nodes: self.safety_nodes,
            corridor_nodes: self.corridor_nodes,
            corridor_branch_probes: self.corridor_branch_probes,
            corridor_max_depth: self.corridor_max_depth,
            corridor_width_exits: self.corridor_width_exits,
            corridor_depth_exits: self.corridor_depth_exits,
            corridor_neutral_exits: self.corridor_neutral_exits,
            corridor_terminal_exits: self.corridor_terminal_exits,
            corridor_plies_followed: self.corridor_plies_followed,
            corridor_own_plies_followed: self.corridor_own_plies_followed,
            corridor_opponent_plies_followed: self.corridor_opponent_plies_followed,
            corridor_proof_passes: self.corridor_proof_passes,
            corridor_proof_completed: self.corridor_proof_completed,
            corridor_proof_checks: self.corridor_proof_checks,
            corridor_proof_active: self.corridor_proof_active,
            corridor_proof_quiet: self.corridor_proof_quiet,
            corridor_proof_static_exits: self.corridor_proof_static_exits,
            corridor_proof_depth_exits: self.corridor_proof_depth_exits,
            corridor_proof_deadline_exits: self.corridor_proof_deadline_exits,
            corridor_proof_terminal_exits: self.corridor_proof_terminal_exits,
            corridor_proof_terminal_root_candidates: self.corridor_proof_terminal_root_candidates,
            corridor_proof_terminal_root_winning_candidates: self
                .corridor_proof_terminal_root_winning_candidates,
            corridor_proof_terminal_root_losing_candidates: self
                .corridor_proof_terminal_root_losing_candidates,
            corridor_proof_terminal_root_overrides: self.corridor_proof_terminal_root_overrides,
            corridor_proof_terminal_root_move_changes: self
                .corridor_proof_terminal_root_move_changes,
            corridor_proof_terminal_root_move_confirmations: self
                .corridor_proof_terminal_root_move_confirmations,
            corridor_proof_candidates_considered: self.corridor_proof_candidates_considered,
            corridor_proof_wins: self.corridor_proof_wins,
            corridor_proof_losses: self.corridor_proof_losses,
            corridor_proof_unknown: self.corridor_proof_unknown,
            corridor_proof_deadline_skips: self.corridor_proof_deadline_skips,
            corridor_proof_move_changes: self.corridor_proof_move_changes,
            corridor_proof_move_confirmations: self.corridor_proof_move_confirmations,
            corridor_proof_candidate_rank_total: self.corridor_proof_candidate_rank_total,
            corridor_proof_candidate_rank_max: self.corridor_proof_candidate_rank_max,
            corridor_proof_candidate_score_gap_total: self.corridor_proof_candidate_score_gap_total,
            corridor_proof_candidate_score_gap_max: self.corridor_proof_candidate_score_gap_max,
            corridor_proof_win_candidate_rank_total: self.corridor_proof_win_candidate_rank_total,
            corridor_proof_win_candidate_rank_max: self.corridor_proof_win_candidate_rank_max,
            total_nodes: self.total_nodes,
            avg_nodes,
            eval_calls: self.eval_calls,
            avg_eval_calls,
            line_shape_eval_calls: self.line_shape_eval_calls,
            line_shape_eval_ns: self.line_shape_eval_ns,
            avg_line_shape_eval_ns,
            pattern_eval_calls: self.pattern_eval_calls,
            pattern_eval_ns: self.pattern_eval_ns,
            avg_pattern_eval_ns,
            pattern_frame_queries: self.pattern_frame_queries,
            pattern_frame_query_ns: self.pattern_frame_query_ns,
            avg_pattern_frame_query_ns,
            pattern_frame_updates: self.pattern_frame_updates,
            pattern_frame_update_ns: self.pattern_frame_update_ns,
            avg_pattern_frame_update_ns,
            pattern_frame_shadow_checks: self.pattern_frame_shadow_checks,
            pattern_frame_shadow_mismatches: self.pattern_frame_shadow_mismatches,
            candidate_generations: self.candidate_generations,
            avg_candidate_generations,
            candidate_moves_total: self.candidate_moves_total,
            avg_candidate_moves,
            candidate_moves_max: self.candidate_moves_max,
            root_candidate_generations: self.root_candidate_generations,
            root_candidate_moves_total: self.root_candidate_moves_total,
            root_candidate_moves_max: self.root_candidate_moves_max,
            search_candidate_generations: self.search_candidate_generations,
            search_candidate_moves_total: self.search_candidate_moves_total,
            search_candidate_moves_max: self.search_candidate_moves_max,
            legality_checks: self.legality_checks,
            avg_legality_checks,
            illegal_moves_skipped: self.illegal_moves_skipped,
            root_legality_checks: self.root_legality_checks,
            root_illegal_moves_skipped: self.root_illegal_moves_skipped,
            search_legality_checks: self.search_legality_checks,
            search_illegal_moves_skipped: self.search_illegal_moves_skipped,
            renju_forbidden_prefilter_checks: self.renju_forbidden_prefilter_checks,
            avg_renju_forbidden_prefilter_checks,
            renju_forbidden_prefilter_ns: self.renju_forbidden_prefilter_ns,
            avg_renju_forbidden_prefilter_ns,
            renju_forbidden_checks: self.renju_forbidden_checks,
            avg_renju_forbidden_checks,
            renju_forbidden_ns: self.renju_forbidden_ns,
            avg_renju_forbidden_ns,
            renju_forbidden_search_gate_checks: self.renju_forbidden_search_gate_checks,
            renju_forbidden_search_gate_ns: self.renju_forbidden_search_gate_ns,
            renju_forbidden_pattern_checks: self.renju_forbidden_pattern_checks,
            renju_forbidden_pattern_ns: self.renju_forbidden_pattern_ns,
            renju_forbidden_threat_checks: self.renju_forbidden_threat_checks,
            renju_forbidden_threat_ns: self.renju_forbidden_threat_ns,
            renju_forbidden_other_checks: self.renju_forbidden_other_checks,
            renju_forbidden_other_ns: self.renju_forbidden_other_ns,
            renju_effective_filter_calls: self.renju_effective_filter_calls,
            avg_renju_effective_filter_calls,
            renju_effective_filter_ns: self.renju_effective_filter_ns,
            avg_renju_effective_filter_ns,
            renju_effective_filter_continuation_checks: self
                .renju_effective_filter_continuation_checks,
            avg_renju_effective_filter_continuation_checks,
            renju_effective_filter_continuation_ns: self.renju_effective_filter_continuation_ns,
            avg_renju_effective_filter_continuation_ns,
            stage_move_gen_ns: self.stage_move_gen_ns,
            stage_ordering_ns: self.stage_ordering_ns,
            stage_eval_ns: self.stage_eval_ns,
            stage_threat_ns: self.stage_threat_ns,
            stage_proof_ns: self.stage_proof_ns,
            tactical_annotations: self.tactical_annotations,
            root_tactical_annotations: self.root_tactical_annotations,
            search_tactical_annotations: self.search_tactical_annotations,
            threat_view_shadow_checks: self.threat_view_shadow_checks,
            threat_view_shadow_mismatches: self.threat_view_shadow_mismatches,
            threat_view_scan_queries: self.threat_view_scan_queries,
            threat_view_scan_ns: self.threat_view_scan_ns,
            threat_view_frontier_rebuilds: self.threat_view_frontier_rebuilds,
            threat_view_frontier_rebuild_ns: self.threat_view_frontier_rebuild_ns,
            threat_view_frontier_queries: self.threat_view_frontier_queries,
            threat_view_frontier_query_ns: self.threat_view_frontier_query_ns,
            threat_view_frontier_immediate_win_queries: self
                .threat_view_frontier_immediate_win_queries,
            threat_view_frontier_immediate_win_query_ns: self
                .threat_view_frontier_immediate_win_query_ns,
            threat_view_frontier_delta_captures: self.threat_view_frontier_delta_captures,
            threat_view_frontier_delta_capture_ns: self.threat_view_frontier_delta_capture_ns,
            threat_view_frontier_move_fact_updates: self.threat_view_frontier_move_fact_updates,
            threat_view_frontier_move_fact_update_ns: self.threat_view_frontier_move_fact_update_ns,
            threat_view_frontier_annotation_dirty_marks: self
                .threat_view_frontier_annotation_dirty_marks,
            threat_view_frontier_annotation_dirty_mark_ns: self
                .threat_view_frontier_annotation_dirty_mark_ns,
            threat_view_frontier_clean_annotation_queries: self
                .threat_view_frontier_clean_annotation_queries,
            threat_view_frontier_clean_annotation_query_ns: self
                .threat_view_frontier_clean_annotation_query_ns,
            threat_view_frontier_dirty_annotation_queries: self
                .threat_view_frontier_dirty_annotation_queries,
            threat_view_frontier_dirty_annotation_query_ns: self
                .threat_view_frontier_dirty_annotation_query_ns,
            threat_view_frontier_fallback_annotation_queries: self
                .threat_view_frontier_fallback_annotation_queries,
            threat_view_frontier_fallback_annotation_query_ns: self
                .threat_view_frontier_fallback_annotation_query_ns,
            threat_view_frontier_memo_annotation_queries: self
                .threat_view_frontier_memo_annotation_queries,
            threat_view_frontier_memo_annotation_query_ns: self
                .threat_view_frontier_memo_annotation_query_ns,
            child_limit_applications: self.child_limit_applications,
            root_child_limit_applications: self.root_child_limit_applications,
            search_child_limit_applications: self.search_child_limit_applications,
            child_cap_hits: self.child_cap_hits,
            root_child_cap_hits: self.root_child_cap_hits,
            search_child_cap_hits: self.search_child_cap_hits,
            child_moves_before_total: self.child_moves_before_total,
            root_child_moves_before_total: self.root_child_moves_before_total,
            search_child_moves_before_total: self.search_child_moves_before_total,
            child_moves_before_max: self.child_moves_before_max,
            root_child_moves_before_max: self.root_child_moves_before_max,
            search_child_moves_before_max: self.search_child_moves_before_max,
            child_moves_after_total: self.child_moves_after_total,
            root_child_moves_after_total: self.root_child_moves_after_total,
            search_child_moves_after_total: self.search_child_moves_after_total,
            child_moves_after_max: self.child_moves_after_max,
            root_child_moves_after_max: self.root_child_moves_after_max,
            search_child_moves_after_max: self.search_child_moves_after_max,
            avg_child_moves_before,
            avg_child_moves_after,
            tt_hits: self.tt_hits,
            tt_cutoffs: self.tt_cutoffs,
            beta_cutoffs: self.beta_cutoffs,
            depth_sum: self.depth_sum,
            avg_depth,
            max_depth: self.max_depth,
            effective_depth_sum: self.effective_depth_sum,
            avg_effective_depth,
            max_effective_depth: self.max_effective_depth,
            depth_reached_counts,
            budget_exhausted_count: self.budget_exhausted_count,
            budget_exhausted_rate,
            pooled_budget_moves: self.pooled_budget_moves,
            pooled_budget_over_base_count: self.pooled_budget_over_base_count,
            pooled_budget_over_base_rate,
            pooled_budget_reserve_exhausted_count: self.pooled_budget_reserve_exhausted_count,
            pooled_budget_reserve_exhausted_rate,
            pooled_budget_avg_reserve_before_ms,
            pooled_budget_avg_reserve_after_ms,
            pooled_budget_min_reserve_after_ms: self
                .pooled_budget_min_reserve_after_ms
                .unwrap_or(0),
            pooled_budget_max_move_budget_ms: self.pooled_budget_max_move_budget_ms,
        }
    }
}

#[derive(Debug, Clone, Default)]
struct EloAggregate {
    sum: f64,
    sum_sq: f64,
}

impl EloAggregate {
    fn add(&mut self, rating: f64) {
        self.sum += rating;
        self.sum_sq += rating * rating;
    }

    fn finish(&self, samples: usize) -> (f64, f64) {
        if samples == 0 {
            return (DEFAULT_INITIAL_RATING, 0.0);
        }
        let mean = self.sum / samples as f64;
        let variance = (self.sum_sq / samples as f64) - mean * mean;
        (mean, variance.max(0.0).sqrt())
    }
}

fn standings(
    bots: &[String],
    results: &TournamentResults,
    matches: &[MatchReport],
    shuffled_elo: &HashMap<String, (f64, f64)>,
) -> Vec<StandingReport> {
    let mut stats: HashMap<String, SideStatsAccumulator> = bots
        .iter()
        .map(|bot| (bot.clone(), SideStatsAccumulator::default()))
        .collect();

    for report_match in matches {
        stats
            .entry(report_match.black.clone())
            .or_default()
            .add_report(&report_match.black_stats);
        stats
            .entry(report_match.white.clone())
            .or_default()
            .add_report(&report_match.white_stats);
    }

    let mut standings = bots
        .iter()
        .map(|bot| {
            let side_stats = stats.remove(bot).unwrap_or_default().finish();
            let wins = *results.wins.get(bot).unwrap_or(&0);
            let draws = *results.draws.get(bot).unwrap_or(&0);
            let losses = *results.losses.get(bot).unwrap_or(&0);
            let (shuffled_elo_avg, shuffled_elo_stddev) = shuffled_elo
                .get(bot)
                .copied()
                .unwrap_or((DEFAULT_INITIAL_RATING, 0.0));

            StandingReport {
                bot: bot.clone(),
                wins,
                draws,
                losses,
                sequential_elo: results.elo_tracker.get_rating(bot),
                shuffled_elo_avg,
                shuffled_elo_stddev,
                match_count: wins + draws + losses,
                move_count: side_stats.move_count,
                search_move_count: side_stats.search_move_count,
                total_time_ms: side_stats.total_time_ms,
                avg_search_time_ms: side_stats.avg_search_time_ms,
                search_nodes: side_stats.search_nodes,
                safety_nodes: side_stats.safety_nodes,
                corridor_nodes: side_stats.corridor_nodes,
                corridor_branch_probes: side_stats.corridor_branch_probes,
                corridor_max_depth: side_stats.corridor_max_depth,
                corridor_width_exits: side_stats.corridor_width_exits,
                corridor_depth_exits: side_stats.corridor_depth_exits,
                corridor_neutral_exits: side_stats.corridor_neutral_exits,
                corridor_terminal_exits: side_stats.corridor_terminal_exits,
                corridor_plies_followed: side_stats.corridor_plies_followed,
                corridor_own_plies_followed: side_stats.corridor_own_plies_followed,
                corridor_opponent_plies_followed: side_stats.corridor_opponent_plies_followed,
                corridor_proof_passes: side_stats.corridor_proof_passes,
                corridor_proof_completed: side_stats.corridor_proof_completed,
                corridor_proof_checks: side_stats.corridor_proof_checks,
                corridor_proof_active: side_stats.corridor_proof_active,
                corridor_proof_quiet: side_stats.corridor_proof_quiet,
                corridor_proof_static_exits: side_stats.corridor_proof_static_exits,
                corridor_proof_depth_exits: side_stats.corridor_proof_depth_exits,
                corridor_proof_deadline_exits: side_stats.corridor_proof_deadline_exits,
                corridor_proof_terminal_exits: side_stats.corridor_proof_terminal_exits,
                corridor_proof_terminal_root_candidates: side_stats
                    .corridor_proof_terminal_root_candidates,
                corridor_proof_terminal_root_winning_candidates: side_stats
                    .corridor_proof_terminal_root_winning_candidates,
                corridor_proof_terminal_root_losing_candidates: side_stats
                    .corridor_proof_terminal_root_losing_candidates,
                corridor_proof_terminal_root_overrides: side_stats
                    .corridor_proof_terminal_root_overrides,
                corridor_proof_terminal_root_move_changes: side_stats
                    .corridor_proof_terminal_root_move_changes,
                corridor_proof_terminal_root_move_confirmations: side_stats
                    .corridor_proof_terminal_root_move_confirmations,
                corridor_proof_candidates_considered: side_stats
                    .corridor_proof_candidates_considered,
                corridor_proof_wins: side_stats.corridor_proof_wins,
                corridor_proof_losses: side_stats.corridor_proof_losses,
                corridor_proof_unknown: side_stats.corridor_proof_unknown,
                corridor_proof_deadline_skips: side_stats.corridor_proof_deadline_skips,
                corridor_proof_move_changes: side_stats.corridor_proof_move_changes,
                corridor_proof_move_confirmations: side_stats.corridor_proof_move_confirmations,
                corridor_proof_candidate_rank_total: side_stats.corridor_proof_candidate_rank_total,
                corridor_proof_candidate_rank_max: side_stats.corridor_proof_candidate_rank_max,
                corridor_proof_candidate_score_gap_total: side_stats
                    .corridor_proof_candidate_score_gap_total,
                corridor_proof_candidate_score_gap_max: side_stats
                    .corridor_proof_candidate_score_gap_max,
                corridor_proof_win_candidate_rank_total: side_stats
                    .corridor_proof_win_candidate_rank_total,
                corridor_proof_win_candidate_rank_max: side_stats
                    .corridor_proof_win_candidate_rank_max,
                total_nodes: side_stats.total_nodes,
                avg_nodes: side_stats.avg_nodes,
                eval_calls: side_stats.eval_calls,
                avg_eval_calls: side_stats.avg_eval_calls,
                line_shape_eval_calls: side_stats.line_shape_eval_calls,
                line_shape_eval_ns: side_stats.line_shape_eval_ns,
                avg_line_shape_eval_ns: side_stats.avg_line_shape_eval_ns,
                pattern_eval_calls: side_stats.pattern_eval_calls,
                pattern_eval_ns: side_stats.pattern_eval_ns,
                avg_pattern_eval_ns: side_stats.avg_pattern_eval_ns,
                pattern_frame_queries: side_stats.pattern_frame_queries,
                pattern_frame_query_ns: side_stats.pattern_frame_query_ns,
                avg_pattern_frame_query_ns: side_stats.avg_pattern_frame_query_ns,
                pattern_frame_updates: side_stats.pattern_frame_updates,
                pattern_frame_update_ns: side_stats.pattern_frame_update_ns,
                avg_pattern_frame_update_ns: side_stats.avg_pattern_frame_update_ns,
                pattern_frame_shadow_checks: side_stats.pattern_frame_shadow_checks,
                pattern_frame_shadow_mismatches: side_stats.pattern_frame_shadow_mismatches,
                candidate_generations: side_stats.candidate_generations,
                avg_candidate_generations: side_stats.avg_candidate_generations,
                candidate_moves_total: side_stats.candidate_moves_total,
                avg_candidate_moves: side_stats.avg_candidate_moves,
                candidate_moves_max: side_stats.candidate_moves_max,
                root_candidate_generations: side_stats.root_candidate_generations,
                root_candidate_moves_total: side_stats.root_candidate_moves_total,
                root_candidate_moves_max: side_stats.root_candidate_moves_max,
                search_candidate_generations: side_stats.search_candidate_generations,
                search_candidate_moves_total: side_stats.search_candidate_moves_total,
                search_candidate_moves_max: side_stats.search_candidate_moves_max,
                legality_checks: side_stats.legality_checks,
                avg_legality_checks: side_stats.avg_legality_checks,
                illegal_moves_skipped: side_stats.illegal_moves_skipped,
                root_legality_checks: side_stats.root_legality_checks,
                root_illegal_moves_skipped: side_stats.root_illegal_moves_skipped,
                search_legality_checks: side_stats.search_legality_checks,
                search_illegal_moves_skipped: side_stats.search_illegal_moves_skipped,
                renju_forbidden_prefilter_checks: side_stats.renju_forbidden_prefilter_checks,
                avg_renju_forbidden_prefilter_checks: side_stats
                    .avg_renju_forbidden_prefilter_checks,
                renju_forbidden_prefilter_ns: side_stats.renju_forbidden_prefilter_ns,
                avg_renju_forbidden_prefilter_ns: side_stats.avg_renju_forbidden_prefilter_ns,
                renju_forbidden_checks: side_stats.renju_forbidden_checks,
                avg_renju_forbidden_checks: side_stats.avg_renju_forbidden_checks,
                renju_forbidden_ns: side_stats.renju_forbidden_ns,
                avg_renju_forbidden_ns: side_stats.avg_renju_forbidden_ns,
                renju_forbidden_search_gate_checks: side_stats.renju_forbidden_search_gate_checks,
                renju_forbidden_search_gate_ns: side_stats.renju_forbidden_search_gate_ns,
                renju_forbidden_pattern_checks: side_stats.renju_forbidden_pattern_checks,
                renju_forbidden_pattern_ns: side_stats.renju_forbidden_pattern_ns,
                renju_forbidden_threat_checks: side_stats.renju_forbidden_threat_checks,
                renju_forbidden_threat_ns: side_stats.renju_forbidden_threat_ns,
                renju_forbidden_other_checks: side_stats.renju_forbidden_other_checks,
                renju_forbidden_other_ns: side_stats.renju_forbidden_other_ns,
                renju_effective_filter_calls: side_stats.renju_effective_filter_calls,
                avg_renju_effective_filter_calls: side_stats.avg_renju_effective_filter_calls,
                renju_effective_filter_ns: side_stats.renju_effective_filter_ns,
                avg_renju_effective_filter_ns: side_stats.avg_renju_effective_filter_ns,
                renju_effective_filter_continuation_checks: side_stats
                    .renju_effective_filter_continuation_checks,
                avg_renju_effective_filter_continuation_checks: side_stats
                    .avg_renju_effective_filter_continuation_checks,
                renju_effective_filter_continuation_ns: side_stats
                    .renju_effective_filter_continuation_ns,
                avg_renju_effective_filter_continuation_ns: side_stats
                    .avg_renju_effective_filter_continuation_ns,
                stage_move_gen_ns: side_stats.stage_move_gen_ns,
                stage_ordering_ns: side_stats.stage_ordering_ns,
                stage_eval_ns: side_stats.stage_eval_ns,
                stage_threat_ns: side_stats.stage_threat_ns,
                stage_proof_ns: side_stats.stage_proof_ns,
                tactical_annotations: side_stats.tactical_annotations,
                root_tactical_annotations: side_stats.root_tactical_annotations,
                search_tactical_annotations: side_stats.search_tactical_annotations,
                threat_view_shadow_checks: side_stats.threat_view_shadow_checks,
                threat_view_shadow_mismatches: side_stats.threat_view_shadow_mismatches,
                threat_view_scan_queries: side_stats.threat_view_scan_queries,
                threat_view_scan_ns: side_stats.threat_view_scan_ns,
                threat_view_frontier_rebuilds: side_stats.threat_view_frontier_rebuilds,
                threat_view_frontier_rebuild_ns: side_stats.threat_view_frontier_rebuild_ns,
                threat_view_frontier_queries: side_stats.threat_view_frontier_queries,
                threat_view_frontier_query_ns: side_stats.threat_view_frontier_query_ns,
                threat_view_frontier_immediate_win_queries: side_stats
                    .threat_view_frontier_immediate_win_queries,
                threat_view_frontier_immediate_win_query_ns: side_stats
                    .threat_view_frontier_immediate_win_query_ns,
                threat_view_frontier_delta_captures: side_stats.threat_view_frontier_delta_captures,
                threat_view_frontier_delta_capture_ns: side_stats
                    .threat_view_frontier_delta_capture_ns,
                threat_view_frontier_move_fact_updates: side_stats
                    .threat_view_frontier_move_fact_updates,
                threat_view_frontier_move_fact_update_ns: side_stats
                    .threat_view_frontier_move_fact_update_ns,
                threat_view_frontier_annotation_dirty_marks: side_stats
                    .threat_view_frontier_annotation_dirty_marks,
                threat_view_frontier_annotation_dirty_mark_ns: side_stats
                    .threat_view_frontier_annotation_dirty_mark_ns,
                threat_view_frontier_clean_annotation_queries: side_stats
                    .threat_view_frontier_clean_annotation_queries,
                threat_view_frontier_clean_annotation_query_ns: side_stats
                    .threat_view_frontier_clean_annotation_query_ns,
                threat_view_frontier_dirty_annotation_queries: side_stats
                    .threat_view_frontier_dirty_annotation_queries,
                threat_view_frontier_dirty_annotation_query_ns: side_stats
                    .threat_view_frontier_dirty_annotation_query_ns,
                threat_view_frontier_fallback_annotation_queries: side_stats
                    .threat_view_frontier_fallback_annotation_queries,
                threat_view_frontier_fallback_annotation_query_ns: side_stats
                    .threat_view_frontier_fallback_annotation_query_ns,
                threat_view_frontier_memo_annotation_queries: side_stats
                    .threat_view_frontier_memo_annotation_queries,
                threat_view_frontier_memo_annotation_query_ns: side_stats
                    .threat_view_frontier_memo_annotation_query_ns,
                child_limit_applications: side_stats.child_limit_applications,
                root_child_limit_applications: side_stats.root_child_limit_applications,
                search_child_limit_applications: side_stats.search_child_limit_applications,
                child_cap_hits: side_stats.child_cap_hits,
                root_child_cap_hits: side_stats.root_child_cap_hits,
                search_child_cap_hits: side_stats.search_child_cap_hits,
                child_moves_before_total: side_stats.child_moves_before_total,
                root_child_moves_before_total: side_stats.root_child_moves_before_total,
                search_child_moves_before_total: side_stats.search_child_moves_before_total,
                child_moves_before_max: side_stats.child_moves_before_max,
                root_child_moves_before_max: side_stats.root_child_moves_before_max,
                search_child_moves_before_max: side_stats.search_child_moves_before_max,
                child_moves_after_total: side_stats.child_moves_after_total,
                root_child_moves_after_total: side_stats.root_child_moves_after_total,
                search_child_moves_after_total: side_stats.search_child_moves_after_total,
                child_moves_after_max: side_stats.child_moves_after_max,
                root_child_moves_after_max: side_stats.root_child_moves_after_max,
                search_child_moves_after_max: side_stats.search_child_moves_after_max,
                avg_child_moves_before: side_stats.avg_child_moves_before,
                avg_child_moves_after: side_stats.avg_child_moves_after,
                tt_hits: side_stats.tt_hits,
                tt_cutoffs: side_stats.tt_cutoffs,
                beta_cutoffs: side_stats.beta_cutoffs,
                avg_depth: side_stats.avg_depth,
                max_depth: side_stats.max_depth,
                effective_depth_sum: side_stats.effective_depth_sum,
                avg_effective_depth: side_stats.avg_effective_depth,
                max_effective_depth: side_stats.max_effective_depth,
                depth_reached_counts: side_stats.depth_reached_counts,
                budget_exhausted_count: side_stats.budget_exhausted_count,
                budget_exhausted_rate: side_stats.budget_exhausted_rate,
                pooled_budget_moves: side_stats.pooled_budget_moves,
                pooled_budget_over_base_count: side_stats.pooled_budget_over_base_count,
                pooled_budget_over_base_rate: side_stats.pooled_budget_over_base_rate,
                pooled_budget_reserve_exhausted_count: side_stats
                    .pooled_budget_reserve_exhausted_count,
                pooled_budget_reserve_exhausted_rate: side_stats
                    .pooled_budget_reserve_exhausted_rate,
                pooled_budget_avg_reserve_before_ms: side_stats.pooled_budget_avg_reserve_before_ms,
                pooled_budget_avg_reserve_after_ms: side_stats.pooled_budget_avg_reserve_after_ms,
                pooled_budget_min_reserve_after_ms: side_stats.pooled_budget_min_reserve_after_ms,
                pooled_budget_max_move_budget_ms: side_stats.pooled_budget_max_move_budget_ms,
            }
        })
        .collect::<Vec<_>>();

    standings.sort_by(|a, b| {
        b.shuffled_elo_avg
            .partial_cmp(&a.shuffled_elo_avg)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    standings
}

fn pairwise(bots: &[String], matches: &[MatchReport]) -> Vec<PairwiseReport> {
    let order = bot_order(bots);
    let mut map: HashMap<(String, String), PairwiseReport> = HashMap::new();

    for report_match in matches {
        let (bot_a, bot_b) = ordered_pair(&report_match.black, &report_match.white, &order);
        let entry = map
            .entry((bot_a.clone(), bot_b.clone()))
            .or_insert(PairwiseReport {
                bot_a,
                bot_b,
                wins_a: 0,
                wins_b: 0,
                draws: 0,
                total: 0,
                score_a: 0.0,
                score_b: 0.0,
            });
        entry.total += 1;

        match report_match.winner.as_deref() {
            Some(winner) if winner == entry.bot_a => {
                entry.wins_a += 1;
                entry.score_a += 1.0;
            }
            Some(winner) if winner == entry.bot_b => {
                entry.wins_b += 1;
                entry.score_b += 1.0;
            }
            None => {
                entry.draws += 1;
                entry.score_a += 0.5;
                entry.score_b += 0.5;
            }
            _ => {}
        }
    }

    let mut values = map.into_values().collect::<Vec<_>>();
    values.sort_by_key(|entry| {
        (
            order.get(&entry.bot_a).copied().unwrap_or(usize::MAX),
            order.get(&entry.bot_b).copied().unwrap_or(usize::MAX),
        )
    });
    values
}

fn color_splits(matches: &[MatchReport]) -> Vec<ColorSplitReport> {
    let mut map: HashMap<(String, String), ColorSplitReport> = HashMap::new();

    for report_match in matches {
        let entry = map
            .entry((report_match.black.clone(), report_match.white.clone()))
            .or_insert(ColorSplitReport {
                black: report_match.black.clone(),
                white: report_match.white.clone(),
                black_wins: 0,
                white_wins: 0,
                draws: 0,
                total: 0,
            });
        entry.total += 1;

        match report_match.winner.as_deref() {
            Some(winner) if winner == entry.black => entry.black_wins += 1,
            Some(winner) if winner == entry.white => entry.white_wins += 1,
            None => entry.draws += 1,
            _ => {}
        }
    }

    let mut values = map.into_values().collect::<Vec<_>>();
    values.sort_by(|a, b| a.black.cmp(&b.black).then(a.white.cmp(&b.white)));
    values
}

fn end_reasons(results: &TournamentResults) -> Vec<CountReport> {
    let mut values = results
        .end_reasons
        .iter()
        .map(|(reason, count)| CountReport {
            key: end_reason_code(*reason).to_string(),
            count: *count,
        })
        .collect::<Vec<_>>();
    values.sort_by(|a, b| a.key.cmp(&b.key));
    values
}

fn shuffled_elo_stats(
    bots: &[String],
    matches: &[MatchReport],
    samples: usize,
) -> HashMap<String, (f64, f64)> {
    let mut aggregate: HashMap<String, EloAggregate> = bots
        .iter()
        .map(|bot| (bot.clone(), EloAggregate::default()))
        .collect();

    for sample in 0..samples {
        let mut indices = (0..matches.len()).collect::<Vec<_>>();
        shuffle_indices(&mut indices, sample as u64);
        let ratings = elo_for_order(bots, matches, &indices);
        for (bot, rating) in ratings {
            aggregate.entry(bot).or_default().add(rating);
        }
    }

    aggregate
        .into_iter()
        .map(|(bot, aggregate)| (bot, aggregate.finish(samples)))
        .collect()
}

fn elo_for_order(
    bots: &[String],
    matches: &[MatchReport],
    indices: &[usize],
) -> HashMap<String, f64> {
    let mut ratings: HashMap<String, f64> = bots
        .iter()
        .map(|bot| (bot.clone(), DEFAULT_INITIAL_RATING))
        .collect();

    for &idx in indices {
        let report_match = &matches[idx];
        let black_rating = *ratings
            .get(&report_match.black)
            .unwrap_or(&DEFAULT_INITIAL_RATING);
        let white_rating = *ratings
            .get(&report_match.white)
            .unwrap_or(&DEFAULT_INITIAL_RATING);
        let expected_black = expected_score(black_rating, white_rating);
        let expected_white = expected_score(white_rating, black_rating);
        let (score_black, score_white) = match report_match.winner.as_deref() {
            Some(winner) if winner == report_match.black => (1.0, 0.0),
            Some(winner) if winner == report_match.white => (0.0, 1.0),
            None => (0.5, 0.5),
            _ => (0.5, 0.5),
        };

        ratings.insert(
            report_match.black.clone(),
            black_rating + DEFAULT_K_FACTOR * (score_black - expected_black),
        );
        ratings.insert(
            report_match.white.clone(),
            white_rating + DEFAULT_K_FACTOR * (score_white - expected_white),
        );
    }

    ratings
}

fn shuffle_indices(indices: &mut [usize], sample: u64) {
    let mut state = 0x9e37_79b9_7f4a_7c15_u64 ^ sample.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    for i in (1..indices.len()).rev() {
        state = xorshift64(state);
        let j = (state as usize) % (i + 1);
        indices.swap(i, j);
    }
}

fn xorshift64(mut value: u64) -> u64 {
    value ^= value << 13;
    value ^= value >> 7;
    value ^= value << 17;
    value
}

fn bot_order(bots: &[String]) -> HashMap<String, usize> {
    bots.iter()
        .enumerate()
        .map(|(idx, bot)| (bot.clone(), idx))
        .collect()
}

fn ordered_pair(first: &str, second: &str, order: &HashMap<String, usize>) -> (String, String) {
    let first_order = order.get(first).copied().unwrap_or(usize::MAX);
    let second_order = order.get(second).copied().unwrap_or(usize::MAX);
    if first_order < second_order || (first_order == second_order && first <= second) {
        (first.to_string(), second.to_string())
    } else {
        (second.to_string(), first.to_string())
    }
}

fn encode_move_cell(mv: Move, board_size: usize) -> Result<usize, String> {
    if mv.row >= board_size || mv.col >= board_size {
        return Err(format!(
            "move outside board: {} for board size {}",
            mv.to_notation(),
            board_size
        ));
    }
    Ok(mv.row * board_size + mv.col)
}

fn result_code(result: &GameResult) -> &'static str {
    match result {
        GameResult::Winner(Color::Black) => "black_won",
        GameResult::Winner(Color::White) => "white_won",
        GameResult::Draw => "draw",
        GameResult::Ongoing => "ongoing",
    }
}

fn winner_name(result: &GameResult, black: &str, white: &str) -> Option<String> {
    match result {
        GameResult::Winner(Color::Black) => Some(black.to_string()),
        GameResult::Winner(Color::White) => Some(white.to_string()),
        GameResult::Draw | GameResult::Ongoing => None,
    }
}

fn end_reason_code(reason: MatchEndReason) -> &'static str {
    match reason {
        MatchEndReason::Natural => "natural",
        MatchEndReason::MaxMoves => "max_moves",
        MatchEndReason::MaxGameTime => "max_game_time",
    }
}

fn trace_value_u64(trace: &Value, key: &str) -> u64 {
    trace.get(key).and_then(Value::as_u64).unwrap_or(0)
}

fn avg(total: f64, count: u32) -> f64 {
    if count == 0 {
        0.0
    } else {
        total / count as f64
    }
}

fn default_opening_policy() -> String {
    "centered-suite".to_string()
}

fn default_search_budget_mode() -> String {
    "strict".to_string()
}

fn default_schedule() -> String {
    "round-robin".to_string()
}

fn score_rate(wins: u32, draws: u32, total: u32) -> f64 {
    avg(wins as f64 + draws as f64 * 0.5, total)
}

#[cfg(test)]
fn schedule_summary(report: &TournamentReport) -> String {
    if report.run.schedule == "gauntlet" {
        if let Some(reference) = &report.reference_anchors {
            let anchor_count = reference.anchors.len();
            if anchor_count > 0 && report.run.bots.len() > anchor_count {
                let candidate_count = report.run.bots.len() - anchor_count;
                let candidate_word = if candidate_count == 1 {
                    "candidate"
                } else {
                    "candidates"
                };
                let anchor_word = if anchor_count == 1 {
                    "anchor"
                } else {
                    "anchors"
                };
                return format!(
                    "{} {} x {} {} x {} games = {} matches",
                    candidate_count,
                    candidate_word,
                    anchor_count,
                    anchor_word,
                    report.run.games_per_pair,
                    report.matches.len()
                );
            }
        }
    }

    let pair_count = report.pairwise.len();
    let pair_word = if pair_count == 1 { "pair" } else { "pairs" };
    format!(
        "{} {} x {} games = {} matches",
        pair_count,
        pair_word,
        report.run.games_per_pair,
        report.matches.len()
    )
}

#[cfg(test)]
fn compact_bot_label(report: &TournamentReport, bot: &str) -> String {
    shared_compact_bot_label(bot, report_uses_budgeted_unqualified_search(&report.run))
}

#[cfg(test)]
fn report_uses_budgeted_unqualified_search(run: &TournamentRunReport) -> bool {
    run.search_time_ms.is_some() || run.search_cpu_time_ms.is_some()
}

#[derive(Default)]
struct PairSearchStats {
    search_move_count: u32,
    total_time_ms: u64,
    total_nodes: u64,
}

impl PairSearchStats {
    fn record(&mut self, stats: &SideStatsReport) {
        self.search_move_count += stats.search_move_count;
        self.total_time_ms += stats.total_time_ms;
        self.total_nodes += stats.total_nodes;
    }
}

impl ReferencePairSearchReport {
    fn from_pair_and_stats(
        pair: &PairwiseReport,
        bot_a_stats: &PairSearchStats,
        bot_b_stats: &PairSearchStats,
    ) -> Self {
        Self {
            bot_a: pair.bot_a.clone(),
            bot_b: pair.bot_b.clone(),
            bot_a_search_move_count: bot_a_stats.search_move_count,
            bot_a_total_time_ms: bot_a_stats.total_time_ms,
            bot_a_total_nodes: bot_a_stats.total_nodes,
            bot_b_search_move_count: bot_b_stats.search_move_count,
            bot_b_total_time_ms: bot_b_stats.total_time_ms,
            bot_b_total_nodes: bot_b_stats.total_nodes,
        }
    }
}

fn reference_pair_search_reports(
    source_report: &TournamentReport,
    pairwise: &[PairwiseReport],
) -> Vec<ReferencePairSearchReport> {
    pairwise
        .iter()
        .map(|pair| {
            let bot_a_stats =
                pair_search_stats_for_matches(&source_report.matches, pair, &pair.bot_a);
            let bot_b_stats =
                pair_search_stats_for_matches(&source_report.matches, pair, &pair.bot_b);
            ReferencePairSearchReport::from_pair_and_stats(pair, &bot_a_stats, &bot_b_stats)
        })
        .collect()
}

fn pair_search_stats_for_matches(
    matches: &[MatchReport],
    pair: &PairwiseReport,
    bot: &str,
) -> PairSearchStats {
    let mut stats = PairSearchStats::default();
    for report_match in matches {
        if !same_pair(report_match, &pair.bot_a, &pair.bot_b) {
            continue;
        }

        if report_match.black == bot {
            stats.record(&report_match.black_stats);
        } else if report_match.white == bot {
            stats.record(&report_match.white_stats);
        }
    }
    stats
}

fn same_pair(report_match: &MatchReport, bot_a: &str, bot_b: &str) -> bool {
    (report_match.black == bot_a && report_match.white == bot_b)
        || (report_match.black == bot_b && report_match.white == bot_a)
}

fn detect_git_commit() -> Option<String> {
    if let Some(sha) = option_env!("GITHUB_SHA") {
        return Some(sha.chars().take(12).collect());
    }

    let output = Command::new("git")
        .args(["rev-parse", "--short=12", "HEAD"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let value = String::from_utf8(output.stdout).ok()?.trim().to_string();
    (!value.is_empty()).then_some(value)
}

fn detect_git_dirty() -> Option<bool> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    Some(!output.stdout.is_empty())
}

fn detect_generated_at_utc() -> String {
    if let Ok(output) = Command::new("date")
        .args(["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .output()
    {
        if output.status.success() {
            if let Ok(value) = String::from_utf8(output.stdout) {
                let value = value.trim().to_string();
                if !value.is_empty() {
                    return value;
                }
            }
        }
    }

    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    format!("unix:{seconds}")
}

fn detect_generated_at_local() -> String {
    if let Ok(output) = Command::new("date")
        .args(["+%Y-%m-%d %H:%M:%S %Z"])
        .output()
    {
        if output.status.success() {
            if let Ok(value) = String::from_utf8(output.stdout) {
                let value = value.trim().to_string();
                if !value.is_empty() {
                    return value;
                }
            }
        }
    }

    detect_generated_at_utc()
}

fn detect_linux_cpu_info() -> (Option<String>, Option<f64>) {
    let Ok(cpuinfo) = std::fs::read_to_string("/proc/cpuinfo") else {
        return (None, None);
    };

    let mut model = None;
    let mut mhz = None;
    for line in cpuinfo.lines() {
        if model.is_none() {
            if let Some(value) = line.strip_prefix("model name") {
                model = cpuinfo_value(value);
            }
        }
        if mhz.is_none() {
            if let Some(value) = line.strip_prefix("cpu MHz") {
                mhz = cpuinfo_value(value).and_then(|value| value.parse::<f64>().ok());
            }
        }
        if model.is_some() && mhz.is_some() {
            break;
        }
    }

    (model, mhz)
}

fn cpuinfo_value(input: &str) -> Option<String> {
    input
        .split_once(':')
        .map(|(_, value)| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_cells_match_saved_match_codec() {
        assert_eq!(encode_move_cell(Move { row: 0, col: 0 }, 15).unwrap(), 0);
        assert_eq!(encode_move_cell(Move { row: 7, col: 7 }, 15).unwrap(), 112);
        assert_eq!(
            encode_move_cell(Move { row: 14, col: 14 }, 15).unwrap(),
            224
        );
    }

    #[test]
    fn report_sums_shadow_mismatches_from_standings() {
        let mut report = sample_report();
        let mut first = sample_standing_with_search_costs("search-d3+rolling-frontier-shadow");
        first.threat_view_shadow_mismatches = 2;
        let mut second = sample_standing_with_search_costs("search-d5+rolling-frontier-shadow");
        second.threat_view_shadow_mismatches = 3;
        report.standings = vec![first, second];

        assert_eq!(report.shadow_mismatch_count(), 5);
    }

    #[test]
    fn side_stats_capture_child_caps_tactical_annotations_and_depth_distribution() {
        let mut stats = SideStatsAccumulator::default();
        let mut metrics = serde_json::json!({
            "eval_calls": 30,
            "line_shape_eval_calls": 10,
            "line_shape_eval_ns": 1000,
            "pattern_eval_calls": 20,
            "pattern_eval_ns": 8000,
            "tactical_annotations": 9,
            "root_tactical_annotations": 2,
            "search_tactical_annotations": 7,
            "child_limit_applications": 4,
            "root_child_limit_applications": 0,
            "search_child_limit_applications": 4,
            "child_cap_hits": 3,
            "root_child_cap_hits": 0,
            "search_child_cap_hits": 3,
            "child_moves_before_total": 48,
            "root_child_moves_before_total": 0,
            "search_child_moves_before_total": 48,
            "child_moves_before_max": 14,
            "root_child_moves_before_max": 0,
            "search_child_moves_before_max": 14,
            "child_moves_after_total": 32,
            "root_child_moves_after_total": 0,
            "search_child_moves_after_total": 32,
            "child_moves_after_max": 9,
            "root_child_moves_after_max": 0,
            "search_child_moves_after_max": 9,
            "corridor_width_exits": 5,
            "corridor_depth_exits": 6,
            "corridor_neutral_exits": 7,
            "corridor_terminal_exits": 8,
            "corridor_plies_followed": 9,
            "corridor_own_plies_followed": 6,
            "corridor_opponent_plies_followed": 3
        });
        metrics["stage_move_gen_ns"] = serde_json::json!(100);
        metrics["stage_ordering_ns"] = serde_json::json!(200);
        metrics["stage_eval_ns"] = serde_json::json!(300);
        metrics["stage_threat_ns"] = serde_json::json!(400);
        metrics["stage_proof_ns"] = serde_json::json!(500);
        metrics["corridor_proof_passes"] = serde_json::json!(10);
        metrics["corridor_proof_completed"] = serde_json::json!(11);
        metrics["corridor_proof_checks"] = serde_json::json!(12);
        metrics["corridor_proof_active"] = serde_json::json!(13);
        metrics["corridor_proof_quiet"] = serde_json::json!(14);
        metrics["corridor_proof_static_exits"] = serde_json::json!(15);
        metrics["corridor_proof_depth_exits"] = serde_json::json!(16);
        metrics["corridor_proof_deadline_exits"] = serde_json::json!(17);
        metrics["corridor_proof_terminal_exits"] = serde_json::json!(18);
        metrics["corridor_proof_terminal_root_candidates"] = serde_json::json!(19);
        metrics["corridor_proof_terminal_root_winning_candidates"] = serde_json::json!(20);
        metrics["corridor_proof_terminal_root_losing_candidates"] = serde_json::json!(21);
        metrics["corridor_proof_terminal_root_overrides"] = serde_json::json!(22);
        metrics["corridor_proof_terminal_root_move_changes"] = serde_json::json!(23);
        metrics["corridor_proof_terminal_root_move_confirmations"] = serde_json::json!(24);
        metrics["corridor_proof_candidates_considered"] = serde_json::json!(25);
        metrics["corridor_proof_wins"] = serde_json::json!(26);
        metrics["corridor_proof_losses"] = serde_json::json!(27);
        metrics["corridor_proof_unknown"] = serde_json::json!(28);
        metrics["corridor_proof_deadline_skips"] = serde_json::json!(29);
        metrics["corridor_proof_move_changes"] = serde_json::json!(30);
        metrics["corridor_proof_move_confirmations"] = serde_json::json!(31);
        metrics["corridor_proof_candidate_rank_total"] = serde_json::json!(32);
        metrics["corridor_proof_candidate_rank_max"] = serde_json::json!(6);
        metrics["corridor_proof_candidate_score_gap_total"] = serde_json::json!(123_456);
        metrics["corridor_proof_candidate_score_gap_max"] = serde_json::json!(50_000);
        metrics["corridor_proof_win_candidate_rank_total"] = serde_json::json!(7);
        metrics["corridor_proof_win_candidate_rank_max"] = serde_json::json!(2);
        metrics["pattern_frame_queries"] = serde_json::json!(15);
        metrics["pattern_frame_query_ns"] = serde_json::json!(150);
        metrics["pattern_frame_updates"] = serde_json::json!(8);
        metrics["pattern_frame_update_ns"] = serde_json::json!(800);
        metrics["pattern_frame_shadow_checks"] = serde_json::json!(15);
        metrics["pattern_frame_shadow_mismatches"] = serde_json::json!(0);
        let trace = serde_json::json!({
            "nodes": 100,
            "safety_nodes": 20,
            "total_nodes": 120,
            "depth": 5,
            "effective_depth": 8,
            "corridor": {
                "search_nodes": 7,
                "branch_probes": 3,
                "max_depth_reached": 2,
                "extra_plies": 3
            },
            "budget_pool": {
                "mode": "pooled_cpu",
                "base_ms": 1000,
                "move_budget_ms": 1750,
                "reserve_cap_ms": 4000,
                "max_move_ms": null,
                "reserve_before_ms": 750,
                "reserve_after_ms": 250,
                "consumed_ms": 1500,
                "budget_exhausted": false
            },
            "metrics": metrics
        });

        stats.record_move(11, Some(&trace));
        let report = stats.finish();

        assert_eq!(report.search_nodes, 100);
        assert_eq!(report.safety_nodes, 20);
        assert_eq!(report.corridor_nodes, 7);
        assert_eq!(report.corridor_branch_probes, 3);
        assert_eq!(report.corridor_max_depth, 2);
        assert_eq!(report.corridor_width_exits, 5);
        assert_eq!(report.corridor_depth_exits, 6);
        assert_eq!(report.corridor_neutral_exits, 7);
        assert_eq!(report.corridor_terminal_exits, 8);
        assert_eq!(report.corridor_plies_followed, 9);
        assert_eq!(report.corridor_own_plies_followed, 6);
        assert_eq!(report.corridor_opponent_plies_followed, 3);
        assert_eq!(report.corridor_proof_passes, 10);
        assert_eq!(report.corridor_proof_completed, 11);
        assert_eq!(report.corridor_proof_checks, 12);
        assert_eq!(report.corridor_proof_active, 13);
        assert_eq!(report.corridor_proof_quiet, 14);
        assert_eq!(report.corridor_proof_static_exits, 15);
        assert_eq!(report.corridor_proof_depth_exits, 16);
        assert_eq!(report.corridor_proof_deadline_exits, 17);
        assert_eq!(report.corridor_proof_terminal_exits, 18);
        assert_eq!(report.corridor_proof_terminal_root_candidates, 19);
        assert_eq!(report.corridor_proof_terminal_root_winning_candidates, 20);
        assert_eq!(report.corridor_proof_terminal_root_losing_candidates, 21);
        assert_eq!(report.corridor_proof_terminal_root_overrides, 22);
        assert_eq!(report.corridor_proof_terminal_root_move_changes, 23);
        assert_eq!(report.corridor_proof_terminal_root_move_confirmations, 24);
        assert_eq!(report.corridor_proof_candidates_considered, 25);
        assert_eq!(report.corridor_proof_wins, 26);
        assert_eq!(report.corridor_proof_losses, 27);
        assert_eq!(report.corridor_proof_unknown, 28);
        assert_eq!(report.corridor_proof_deadline_skips, 29);
        assert_eq!(report.corridor_proof_move_changes, 30);
        assert_eq!(report.corridor_proof_move_confirmations, 31);
        assert_eq!(report.corridor_proof_candidate_rank_total, 32);
        assert_eq!(report.corridor_proof_candidate_rank_max, 6);
        assert_eq!(report.corridor_proof_candidate_score_gap_total, 123_456);
        assert_eq!(report.corridor_proof_candidate_score_gap_max, 50_000);
        assert_eq!(report.corridor_proof_win_candidate_rank_total, 7);
        assert_eq!(report.corridor_proof_win_candidate_rank_max, 2);
        assert_eq!(report.eval_calls, 30);
        assert_eq!(report.avg_eval_calls, 30.0);
        assert_eq!(report.line_shape_eval_calls, 10);
        assert_eq!(report.line_shape_eval_ns, 1000);
        assert_eq!(report.avg_line_shape_eval_ns, 100.0);
        assert_eq!(report.pattern_eval_calls, 20);
        assert_eq!(report.pattern_eval_ns, 8000);
        assert_eq!(report.avg_pattern_eval_ns, 400.0);
        assert_eq!(report.pattern_frame_queries, 15);
        assert_eq!(report.pattern_frame_query_ns, 150);
        assert_eq!(report.avg_pattern_frame_query_ns, 10.0);
        assert_eq!(report.pattern_frame_updates, 8);
        assert_eq!(report.pattern_frame_update_ns, 800);
        assert_eq!(report.avg_pattern_frame_update_ns, 100.0);
        assert_eq!(report.pattern_frame_shadow_checks, 15);
        assert_eq!(report.pattern_frame_shadow_mismatches, 0);
        assert_eq!(report.stage_move_gen_ns, 100);
        assert_eq!(report.stage_ordering_ns, 200);
        assert_eq!(report.stage_eval_ns, 300);
        assert_eq!(report.stage_threat_ns, 400);
        assert_eq!(report.stage_proof_ns, 500);
        assert_eq!(report.effective_depth_sum, 8);
        assert_eq!(report.avg_effective_depth, 8.0);
        assert_eq!(report.max_effective_depth, 8);
        assert_eq!(report.tactical_annotations, 9);
        assert_eq!(report.root_tactical_annotations, 2);
        assert_eq!(report.search_tactical_annotations, 7);
        assert_eq!(report.child_limit_applications, 4);
        assert_eq!(report.search_child_limit_applications, 4);
        assert_eq!(report.child_cap_hits, 3);
        assert_eq!(report.search_child_cap_hits, 3);
        assert_eq!(report.child_moves_before_total, 48);
        assert_eq!(report.child_moves_after_total, 32);
        assert_eq!(report.avg_child_moves_before, 12.0);
        assert_eq!(report.avg_child_moves_after, 8.0);
        assert_eq!(
            report.depth_reached_counts,
            vec![DepthCountReport { depth: 5, count: 1 }]
        );
        assert_eq!(report.pooled_budget_moves, 1);
        assert_eq!(report.pooled_budget_over_base_count, 1);
        assert_eq!(report.pooled_budget_over_base_rate, 1.0);
        assert_eq!(report.pooled_budget_reserve_exhausted_count, 0);
        assert_eq!(report.pooled_budget_reserve_exhausted_rate, 0.0);
        assert_eq!(report.pooled_budget_avg_reserve_before_ms, 750.0);
        assert_eq!(report.pooled_budget_avg_reserve_after_ms, 250.0);
        assert_eq!(report.pooled_budget_min_reserve_after_ms, 250);
        assert_eq!(report.pooled_budget_max_move_budget_ms, 1750);
    }

    #[test]
    fn side_stats_capture_threat_view_metrics() {
        let mut stats = SideStatsAccumulator::default();
        let trace = serde_json::json!({
            "metrics": {
                "threat_view_shadow_checks": 11,
                "threat_view_shadow_mismatches": 1,
                "threat_view_scan_queries": 13,
                "threat_view_scan_ns": 1700,
                "threat_view_frontier_rebuilds": 5,
                "threat_view_frontier_rebuild_ns": 2300,
                "threat_view_frontier_queries": 19,
                "threat_view_frontier_query_ns": 2900,
                "threat_view_frontier_immediate_win_queries": 20,
                "threat_view_frontier_immediate_win_query_ns": 3000,
                "threat_view_frontier_delta_captures": 7,
                "threat_view_frontier_delta_capture_ns": 3100,
                "threat_view_frontier_move_fact_updates": 8,
                "threat_view_frontier_move_fact_update_ns": 3200,
                "threat_view_frontier_annotation_dirty_marks": 9,
                "threat_view_frontier_annotation_dirty_mark_ns": 3300,
                "threat_view_frontier_clean_annotation_queries": 14,
                "threat_view_frontier_clean_annotation_query_ns": 3400,
                "threat_view_frontier_dirty_annotation_queries": 15,
                "threat_view_frontier_dirty_annotation_query_ns": 3500,
                "threat_view_frontier_fallback_annotation_queries": 16,
                "threat_view_frontier_fallback_annotation_query_ns": 3600,
                "threat_view_frontier_memo_annotation_queries": 17,
                "threat_view_frontier_memo_annotation_query_ns": 3700
            }
        });

        stats.record_move(11, Some(&trace));
        let report = stats.finish();

        assert_eq!(report.threat_view_shadow_checks, 11);
        assert_eq!(report.threat_view_shadow_mismatches, 1);
        assert_eq!(report.threat_view_scan_queries, 13);
        assert_eq!(report.threat_view_scan_ns, 1700);
        assert_eq!(report.threat_view_frontier_rebuilds, 5);
        assert_eq!(report.threat_view_frontier_rebuild_ns, 2300);
        assert_eq!(report.threat_view_frontier_queries, 19);
        assert_eq!(report.threat_view_frontier_query_ns, 2900);
        assert_eq!(report.threat_view_frontier_immediate_win_queries, 20);
        assert_eq!(report.threat_view_frontier_immediate_win_query_ns, 3000);
        assert_eq!(report.threat_view_frontier_delta_captures, 7);
        assert_eq!(report.threat_view_frontier_delta_capture_ns, 3100);
        assert_eq!(report.threat_view_frontier_move_fact_updates, 8);
        assert_eq!(report.threat_view_frontier_move_fact_update_ns, 3200);
        assert_eq!(report.threat_view_frontier_annotation_dirty_marks, 9);
        assert_eq!(report.threat_view_frontier_annotation_dirty_mark_ns, 3300);
        assert_eq!(report.threat_view_frontier_clean_annotation_queries, 14);
        assert_eq!(report.threat_view_frontier_clean_annotation_query_ns, 3400);
        assert_eq!(report.threat_view_frontier_dirty_annotation_queries, 15);
        assert_eq!(report.threat_view_frontier_dirty_annotation_query_ns, 3500);
        assert_eq!(report.threat_view_frontier_fallback_annotation_queries, 16);
        assert_eq!(
            report.threat_view_frontier_fallback_annotation_query_ns,
            3600
        );
        assert_eq!(report.threat_view_frontier_memo_annotation_queries, 17);
        assert_eq!(report.threat_view_frontier_memo_annotation_query_ns, 3700);
    }

    #[test]
    fn standings_preserve_search_node_split_and_child_cap_metrics() {
        let mut report = sample_report();
        let mut first_match = sample_match(1, "search-d5+tactical-cap-8", "search-d3", None);
        first_match.black_stats = sample_side_stats_with_search_costs();
        first_match.black_stats.search_nodes = 900;
        first_match.black_stats.safety_nodes = 100;
        first_match.black_stats.corridor_nodes = 17;
        first_match.black_stats.corridor_branch_probes = 9;
        first_match.black_stats.corridor_max_depth = 2;
        first_match.black_stats.corridor_width_exits = 6;
        first_match.black_stats.corridor_depth_exits = 5;
        first_match.black_stats.corridor_neutral_exits = 4;
        first_match.black_stats.corridor_terminal_exits = 3;
        first_match.black_stats.corridor_plies_followed = 12;
        first_match.black_stats.corridor_own_plies_followed = 9;
        first_match.black_stats.corridor_opponent_plies_followed = 3;
        first_match.black_stats.corridor_proof_terminal_exits = 13;
        first_match
            .black_stats
            .corridor_proof_terminal_root_candidates = 7;
        first_match
            .black_stats
            .corridor_proof_terminal_root_winning_candidates = 5;
        first_match
            .black_stats
            .corridor_proof_terminal_root_losing_candidates = 2;
        first_match
            .black_stats
            .corridor_proof_terminal_root_overrides = 2;
        first_match
            .black_stats
            .corridor_proof_terminal_root_move_changes = 1;
        first_match
            .black_stats
            .corridor_proof_terminal_root_move_confirmations = 1;
        first_match.black_stats.corridor_proof_candidates_considered = 9;
        first_match.black_stats.corridor_proof_wins = 4;
        first_match.black_stats.corridor_proof_losses = 3;
        first_match.black_stats.corridor_proof_unknown = 2;
        first_match.black_stats.corridor_proof_deadline_skips = 1;
        first_match.black_stats.corridor_proof_move_changes = 1;
        first_match.black_stats.corridor_proof_move_confirmations = 1;
        first_match.black_stats.corridor_proof_candidate_rank_total = 12;
        first_match.black_stats.corridor_proof_candidate_rank_max = 4;
        first_match
            .black_stats
            .corridor_proof_candidate_score_gap_total = 75_000;
        first_match
            .black_stats
            .corridor_proof_candidate_score_gap_max = 50_000;
        first_match
            .black_stats
            .corridor_proof_win_candidate_rank_total = 3;
        first_match
            .black_stats
            .corridor_proof_win_candidate_rank_max = 2;
        first_match.black_stats.effective_depth_sum = 36;
        first_match.black_stats.avg_effective_depth = 7.2;
        first_match.black_stats.max_effective_depth = 9;
        first_match.black_stats.tactical_annotations = 20;
        first_match.black_stats.search_tactical_annotations = 20;
        first_match.black_stats.threat_view_shadow_checks = 30;
        first_match.black_stats.threat_view_shadow_mismatches = 2;
        first_match.black_stats.threat_view_scan_queries = 40;
        first_match.black_stats.threat_view_scan_ns = 5000;
        first_match.black_stats.threat_view_frontier_rebuilds = 6;
        first_match.black_stats.threat_view_frontier_rebuild_ns = 7000;
        first_match.black_stats.threat_view_frontier_queries = 80;
        first_match.black_stats.threat_view_frontier_query_ns = 9000;
        first_match.black_stats.child_limit_applications = 10;
        first_match.black_stats.search_child_limit_applications = 10;
        first_match.black_stats.child_cap_hits = 8;
        first_match.black_stats.search_child_cap_hits = 8;
        first_match.black_stats.child_moves_before_total = 120;
        first_match.black_stats.search_child_moves_before_total = 120;
        first_match.black_stats.child_moves_after_total = 80;
        first_match.black_stats.search_child_moves_after_total = 80;
        first_match.black_stats.avg_child_moves_before = 12.0;
        first_match.black_stats.avg_child_moves_after = 8.0;
        first_match.black_stats.depth_reached_counts =
            vec![DepthCountReport { depth: 5, count: 5 }];
        report.matches = vec![first_match];
        report.run.bots = vec![
            "search-d5+tactical-cap-8".to_string(),
            "search-d3".to_string(),
        ];
        let results = TournamentResults::new();

        let rows = standings(&report.run.bots, &results, &report.matches, &HashMap::new());
        let row = rows
            .iter()
            .find(|row| row.bot == "search-d5+tactical-cap-8")
            .expect("standing row should exist");

        assert_eq!(row.search_nodes, 900);
        assert_eq!(row.safety_nodes, 100);
        assert_eq!(row.corridor_nodes, 17);
        assert_eq!(row.corridor_branch_probes, 9);
        assert_eq!(row.corridor_max_depth, 2);
        assert_eq!(row.corridor_width_exits, 6);
        assert_eq!(row.corridor_depth_exits, 5);
        assert_eq!(row.corridor_neutral_exits, 4);
        assert_eq!(row.corridor_terminal_exits, 3);
        assert_eq!(row.corridor_plies_followed, 12);
        assert_eq!(row.corridor_own_plies_followed, 9);
        assert_eq!(row.corridor_opponent_plies_followed, 3);
        assert_eq!(row.corridor_proof_terminal_exits, 13);
        assert_eq!(row.corridor_proof_terminal_root_candidates, 7);
        assert_eq!(row.corridor_proof_terminal_root_winning_candidates, 5);
        assert_eq!(row.corridor_proof_terminal_root_losing_candidates, 2);
        assert_eq!(row.corridor_proof_terminal_root_overrides, 2);
        assert_eq!(row.corridor_proof_terminal_root_move_changes, 1);
        assert_eq!(row.corridor_proof_terminal_root_move_confirmations, 1);
        assert_eq!(row.corridor_proof_candidates_considered, 9);
        assert_eq!(row.corridor_proof_wins, 4);
        assert_eq!(row.corridor_proof_losses, 3);
        assert_eq!(row.corridor_proof_unknown, 2);
        assert_eq!(row.corridor_proof_deadline_skips, 1);
        assert_eq!(row.corridor_proof_move_changes, 1);
        assert_eq!(row.corridor_proof_move_confirmations, 1);
        assert_eq!(row.corridor_proof_candidate_rank_total, 12);
        assert_eq!(row.corridor_proof_candidate_rank_max, 4);
        assert_eq!(row.corridor_proof_candidate_score_gap_total, 75_000);
        assert_eq!(row.corridor_proof_candidate_score_gap_max, 50_000);
        assert_eq!(row.corridor_proof_win_candidate_rank_total, 3);
        assert_eq!(row.corridor_proof_win_candidate_rank_max, 2);
        assert_eq!(row.effective_depth_sum, 36);
        assert_eq!(row.avg_effective_depth, 7.2);
        assert_eq!(row.max_effective_depth, 9);
        assert_eq!(row.tactical_annotations, 20);
        assert_eq!(row.threat_view_shadow_checks, 30);
        assert_eq!(row.threat_view_shadow_mismatches, 2);
        assert_eq!(row.threat_view_scan_queries, 40);
        assert_eq!(row.threat_view_scan_ns, 5000);
        assert_eq!(row.threat_view_frontier_rebuilds, 6);
        assert_eq!(row.threat_view_frontier_rebuild_ns, 7000);
        assert_eq!(row.threat_view_frontier_queries, 80);
        assert_eq!(row.threat_view_frontier_query_ns, 9000);
        assert_eq!(row.child_limit_applications, 10);
        assert_eq!(row.child_cap_hits, 8);
        assert_eq!(row.child_moves_before_total, 120);
        assert_eq!(row.child_moves_after_total, 80);
        assert_eq!(row.avg_child_moves_before, 12.0);
        assert_eq!(row.avg_child_moves_after, 8.0);
        assert_eq!(
            row.depth_reached_counts,
            vec![DepthCountReport { depth: 5, count: 5 }]
        );
    }

    #[test]
    fn schedule_summary_uses_played_pairs_for_sparse_schedules() {
        let mut report = sample_report();
        report.run.bots = vec![
            "candidate".to_string(),
            "anchor-a".to_string(),
            "anchor-b".to_string(),
        ];

        assert_eq!(schedule_summary(&report), "1 pair x 2 games = 2 matches");
    }

    #[test]
    fn schedule_summary_describes_batch_gauntlet_shape() {
        let mut report = sample_report();
        report.run.schedule = "gauntlet".to_string();
        report.run.bots = vec![
            "candidate-a".to_string(),
            "candidate-b".to_string(),
            "anchor-a".to_string(),
            "anchor-b".to_string(),
        ];
        report.matches = (0..8)
            .map(|index| sample_match(index + 1, "candidate-a", "anchor-a", None))
            .collect();
        report.reference_anchors = Some(AnchorReferenceReport {
            source: AnchorReferenceSource {
                path: Some("../reports/lab/bot-report.json".to_string()),
                schedule: "round-robin".to_string(),
                git_commit: Some("abc123".to_string()),
                git_dirty: Some(false),
                rules: report.run.rules.clone(),
                games_per_pair: 64,
                opening_policy: "centered-suite".to_string(),
                opening_plies: 4,
                seed: 63,
                search_time_ms: None,
                search_cpu_time_ms: Some(1000),
                search_budget_mode: "strict".to_string(),
                search_cpu_reserve_ms: None,
                search_cpu_max_move_ms: None,
                max_moves: Some(120),
                max_game_ms: None,
            },
            anchors: vec![
                AnchorStandingReport {
                    bot: "anchor-a".to_string(),
                    sequential_elo: 1200.0,
                    shuffled_elo_avg: 1200.0,
                    shuffled_elo_stddev: 0.0,
                    match_count: 64,
                    score_percentage: 50.0,
                },
                AnchorStandingReport {
                    bot: "anchor-b".to_string(),
                    sequential_elo: 1200.0,
                    shuffled_elo_avg: 1200.0,
                    shuffled_elo_stddev: 0.0,
                    match_count: 64,
                    score_percentage: 50.0,
                },
            ],
            pairwise: vec![],
            pair_search: vec![],
        });

        assert_eq!(
            schedule_summary(&report),
            "2 candidates x 2 anchors x 2 games = 8 matches"
        );
    }

    #[test]
    fn searchbot_labels_keep_report_variants_distinct() {
        let report = sample_report();

        assert_eq!(compact_bot_label(&report, "search-d5"), "SearchBot_D5");
        assert_eq!(
            compact_bot_label(&report, "search-d5+tactical-cap-8"),
            "SearchBot_D5+TCap8"
        );
        assert_eq!(
            compact_bot_label(&report, "search-d5+tactical-full-cap-8"),
            "SearchBot_D5+TFullCap8"
        );
        assert_eq!(
            compact_bot_label(&report, "search-d5+tactical-full"),
            "SearchBot_D5+TFull"
        );
        assert_eq!(
            compact_bot_label(&report, "search-d5+tactical-cap-8+pattern-eval"),
            "SearchBot_D5+TCap8+Pattern"
        );
        assert_eq!(
            compact_bot_label(&report, "search-d5+tactical-cap-8+near-self-r2-opponent-r1"),
            "SearchBot_D5+TCap8+SelfR2OppR1"
        );
        assert_eq!(
            compact_bot_label(
                &report,
                "search-d5+tactical-cap-8+pattern-eval+corridor-proof-c16-d8-w3"
            ),
            "SearchBot_D5+TCap8+Pattern+Corridor Proof"
        );
    }

    #[test]
    fn reference_anchors_copy_requested_standings_from_source_report() {
        let mut source = sample_report();
        source.run.schedule = "round-robin".to_string();
        source.provenance.git_commit = Some("abc123".to_string());
        source.provenance.git_dirty = Some(false);
        source.standings = vec![
            sample_standing_with_search_costs("candidate"),
            sample_standing_with_search_costs("anchor-a"),
            sample_standing_with_search_costs("anchor-b"),
        ];
        source.standings[1].shuffled_elo_avg = 1234.5;
        source.standings[1].shuffled_elo_stddev = 12.0;
        source.standings[2].shuffled_elo_avg = 1175.0;
        source.pairwise = vec![
            PairwiseReport {
                bot_a: "anchor-a".to_string(),
                bot_b: "anchor-b".to_string(),
                wins_a: 35,
                wins_b: 29,
                draws: 0,
                total: 64,
                score_a: 35.0,
                score_b: 29.0,
            },
            PairwiseReport {
                bot_a: "candidate".to_string(),
                bot_b: "anchor-a".to_string(),
                wins_a: 31,
                wins_b: 33,
                draws: 0,
                total: 64,
                score_a: 31.0,
                score_b: 33.0,
            },
        ];

        let reference = AnchorReferenceReport::from_report(
            Some("../reports/lab/bot-report.json".to_string()),
            &source,
            &["anchor-a".to_string(), "anchor-b".to_string()],
        )
        .expect("anchors should be copied");

        assert_eq!(
            reference.source.path.as_deref(),
            Some("../reports/lab/bot-report.json")
        );
        assert_eq!(reference.source.schedule, "round-robin");
        assert_eq!(reference.source.git_commit.as_deref(), Some("abc123"));
        assert_eq!(reference.anchors.len(), 2);
        assert_eq!(reference.anchors[0].bot, "anchor-a");
        assert_eq!(reference.anchors[0].shuffled_elo_avg, 1234.5);
        assert_eq!(reference.anchors[0].shuffled_elo_stddev, 12.0);
        assert_eq!(reference.anchors[1].bot, "anchor-b");
        assert_eq!(reference.anchors[1].shuffled_elo_avg, 1175.0);
        assert_eq!(reference.pairwise.len(), 1);
        assert_eq!(reference.pairwise[0].bot_a, "anchor-a");
        assert_eq!(reference.pairwise[0].bot_b, "anchor-b");
    }

    #[test]
    fn reference_anchors_reject_missing_anchor_names() {
        let mut source = sample_report();
        source.standings = vec![sample_standing_with_search_costs("anchor-a")];

        let err = AnchorReferenceReport::from_report(
            None,
            &source,
            &["anchor-a".to_string(), "missing-anchor".to_string()],
        )
        .unwrap_err();

        assert!(err.contains("missing-anchor"));
    }

    #[test]
    fn reference_anchors_require_round_robin_source_report() {
        let mut source = sample_report();
        source.run.schedule = "gauntlet".to_string();
        source.standings = vec![sample_standing_with_search_costs("anchor-a")];

        let err = AnchorReferenceReport::from_report(None, &source, &["anchor-a".to_string()])
            .unwrap_err();

        assert!(err.contains("round-robin"));
    }

    #[test]
    fn reference_anchors_copy_max_limits_from_source_report() {
        let mut source = sample_report();
        source.run.max_moves = Some(120);
        source.run.max_game_ms = Some(10_000);
        source.standings = vec![sample_standing_with_search_costs("anchor-a")];

        let reference =
            AnchorReferenceReport::from_report(None, &source, &["anchor-a".to_string()])
                .expect("anchor reference should copy limits");

        assert_eq!(reference.source.max_moves, Some(120));
        assert_eq!(reference.source.max_game_ms, Some(10_000));
    }

    #[test]
    fn reference_anchors_validate_matching_eval_context() {
        let mut source = sample_report();
        source.run.max_moves = Some(120);
        source.standings = vec![sample_standing_with_search_costs("anchor-a")];
        let reference =
            AnchorReferenceReport::from_report(None, &source, &["anchor-a".to_string()])
                .expect("anchor reference should parse");
        let mut run = source.run.clone();

        reference
            .validate_compatible_run(&run)
            .expect("same context should be compatible");

        run.search_cpu_time_ms = Some(500);
        let err = reference.validate_compatible_run(&run).unwrap_err();

        assert!(err.contains("search_cpu_time_ms"));

        let mut run = source.run.clone();
        run.search_budget_mode = "pooled".to_string();
        run.search_cpu_reserve_ms = Some(4_000);
        run.search_cpu_max_move_ms = Some(2_000);
        let err = reference.validate_compatible_run(&run).unwrap_err();

        assert!(err.contains("search_budget_mode"));
        assert!(err.contains("search_cpu_reserve_ms"));
        assert!(err.contains("search_cpu_max_move_ms"));
    }

    #[test]
    fn published_report_keeps_replay_cells_and_drops_debug_metrics() {
        let report = sample_report();
        let published = PublishedTournamentReport::from_tournament_report(&report);
        let json = published
            .to_json()
            .expect("published report should serialize");

        assert_eq!(
            published.schema_version,
            PUBLISHED_TOURNAMENT_REPORT_SCHEMA_VERSION
        );
        assert_eq!(published.report_kind, "published_tournament");
        assert_eq!(
            published.matches[0].move_cells,
            report.matches[0].move_cells
        );
        assert!(json.contains("move_cells"));
        assert!(!json.contains("black_stats"));
        assert!(!json.contains("white_stats"));
        assert!(!json.contains("duration_ms"));
        assert!(!json.contains("\"opening\":"));
        assert!(!json.contains("renju_forbidden_prefilter_checks"));

        let parsed =
            PublishedTournamentReport::from_json(&json).expect("published report should parse");
        assert_eq!(parsed.matches.len(), report.matches.len());
    }

    #[test]
    fn from_json_defaults_missing_search_metrics() {
        let input = r#"{
          "schema_version": 1,
          "report_kind": "tournament",
          "board_size": 15,
          "move_codec": "cell_index_v1",
          "shuffled_elo_samples": 256,
          "run": {
            "bots": ["search-d1", "search-d3"],
            "rules": {"board_size": 15, "win_length": 5, "variant": "renju"},
            "games_per_pair": 2,
            "seed": 42,
            "opening_plies": 4,
            "threads": 1,
            "search_time_ms": null,
            "search_cpu_time_ms": 1000,
            "max_moves": 120,
            "max_game_ms": null
          },
          "standings": [{
            "bot": "search-d1",
            "wins": 1,
            "draws": 0,
            "losses": 1,
            "sequential_elo": 1000.0,
            "shuffled_elo_avg": 1000.0,
            "shuffled_elo_stddev": 0.0,
            "match_count": 2,
            "move_count": 10,
            "search_move_count": 10,
            "total_time_ms": 100,
            "avg_search_time_ms": 10.0,
            "total_nodes": 1000,
            "avg_nodes": 100.0,
            "avg_depth": 3.0,
            "max_depth": 3,
            "budget_exhausted_count": 0,
            "budget_exhausted_rate": 0.0
          }],
          "pairwise": [],
          "color_splits": [],
          "end_reasons": [],
          "matches": [{
            "match_index": 1,
            "black": "search-d1",
            "white": "search-d3",
            "result": "black_won",
            "winner": "search-d1",
            "end_reason": "natural",
            "duration_ms": 100,
            "move_cells": [112, 113],
            "move_count": 2,
            "black_stats": {
              "move_count": 1,
              "search_move_count": 1,
              "total_time_ms": 10,
              "avg_search_time_ms": 10.0,
              "search_nodes": 100,
              "prefilter_nodes": 10,
              "total_nodes": 110,
              "avg_nodes": 110.0,
              "depth_sum": 3,
              "avg_depth": 3.0,
              "max_depth": 3,
              "budget_exhausted_count": 0,
              "budget_exhausted_rate": 0.0
            },
            "white_stats": {
              "move_count": 1,
              "search_move_count": 1,
              "total_time_ms": 10,
              "avg_search_time_ms": 10.0,
              "search_nodes": 100,
              "safety_nodes": 10,
              "total_nodes": 110,
              "avg_nodes": 110.0,
              "depth_sum": 3,
              "avg_depth": 3.0,
              "max_depth": 3,
              "budget_exhausted_count": 0,
              "budget_exhausted_rate": 0.0
            }
          }]
        }"#;

        let report = TournamentReport::from_json(input).expect("report should parse");

        assert_eq!(report.run.schedule, "round-robin");
        assert_eq!(report.run.opening_policy, "centered-suite");
        assert_eq!(report.run.search_budget_mode, "strict");
        assert_eq!(report.run.search_cpu_reserve_ms, None);
        assert_eq!(report.standings[0].eval_calls, 0);
        assert_eq!(report.standings[0].search_candidate_generations, 0);
        assert!(report.matches[0].opening.is_none());
        assert_eq!(report.matches[0].black_stats.safety_nodes, 10);
        assert_eq!(report.matches[0].white_stats.safety_nodes, 10);
        assert_eq!(report.matches[0].black_stats.root_legality_checks, 0);
        assert_eq!(report.matches[0].white_stats.search_legality_checks, 0);
    }

    #[test]
    fn from_json_rejects_unsupported_schema() {
        let input = r#"{
          "schema_version": 999,
          "report_kind": "tournament",
          "board_size": 15,
          "move_codec": "cell_index_v1",
          "shuffled_elo_samples": 256,
          "run": {
            "bots": [],
            "rules": {"board_size": 15, "win_length": 5, "variant": "renju"},
            "games_per_pair": 0,
            "seed": 0,
            "opening_plies": 0,
            "threads": 1,
            "search_time_ms": null,
            "search_cpu_time_ms": null,
            "max_moves": null,
            "max_game_ms": null
          },
          "standings": [],
          "pairwise": [],
          "color_splits": [],
          "end_reasons": [],
          "matches": []
        }"#;

        let err = TournamentReport::from_json(input).unwrap_err();
        assert!(err.contains("unsupported tournament report schema version"));
    }

    fn sample_report() -> TournamentReport {
        TournamentReport {
            schema_version: TOURNAMENT_REPORT_SCHEMA_VERSION,
            report_kind: "tournament".to_string(),
            board_size: 15,
            move_codec: MOVE_CODEC.to_string(),
            shuffled_elo_samples: SHUFFLED_ELO_SAMPLES,
            provenance: ReportProvenance::default(),
            reference_anchors: None,
            run: TournamentRunReport {
                bots: vec!["search-d1".to_string(), "search-d3".to_string()],
                schedule: "round-robin".to_string(),
                rules: RuleConfig {
                    board_size: 15,
                    win_length: 5,
                    variant: gomoku_core::Variant::Renju,
                },
                games_per_pair: 2,
                seed: 42,
                opening_plies: 4,
                opening_policy: "centered-suite".to_string(),
                threads: 1,
                search_time_ms: None,
                search_cpu_time_ms: Some(1000),
                search_budget_mode: "strict".to_string(),
                search_cpu_reserve_ms: None,
                search_cpu_max_move_ms: None,
                max_moves: Some(120),
                max_game_ms: None,
                total_wall_time_ms: Some(100),
            },
            standings: Vec::new(),
            pairwise: vec![PairwiseReport {
                bot_a: "search-d1".to_string(),
                bot_b: "search-d3".to_string(),
                wins_a: 0,
                wins_b: 2,
                draws: 0,
                total: 2,
                score_a: 0.0,
                score_b: 2.0,
            }],
            color_splits: vec![
                ColorSplitReport {
                    black: "search-d1".to_string(),
                    white: "search-d3".to_string(),
                    black_wins: 0,
                    white_wins: 1,
                    draws: 0,
                    total: 1,
                },
                ColorSplitReport {
                    black: "search-d3".to_string(),
                    white: "search-d1".to_string(),
                    black_wins: 1,
                    white_wins: 0,
                    draws: 0,
                    total: 1,
                },
            ],
            end_reasons: Vec::new(),
            matches: vec![
                sample_match(1, "search-d1", "search-d3", Some("search-d3")),
                sample_match(2, "search-d3", "search-d1", Some("search-d3")),
            ],
        }
    }

    fn sample_match(index: usize, black: &str, white: &str, winner: Option<&str>) -> MatchReport {
        MatchReport {
            match_index: index,
            black: black.to_string(),
            white: white.to_string(),
            result: if winner.is_some() { "win" } else { "draw" }.to_string(),
            winner: winner.map(str::to_string),
            end_reason: "natural".to_string(),
            duration_ms: Some(100),
            opening: Some(MatchOpeningReport {
                policy: "centered-suite".to_string(),
                index: 0,
                ply_count: 4,
                suite_index: Some(3),
                template_index: Some(0),
                transform_index: Some(3),
            }),
            move_cells: vec![112, 113, 127, 128, 142],
            move_count: 5,
            black_stats: SideStatsReport::default(),
            white_stats: SideStatsReport::default(),
        }
    }

    fn sample_standing_with_search_costs(bot: &str) -> StandingReport {
        StandingReport {
            bot: bot.to_string(),
            wins: 1,
            draws: 0,
            losses: 1,
            sequential_elo: 1000.0,
            shuffled_elo_avg: 1000.0,
            shuffled_elo_stddev: 0.0,
            match_count: 2,
            move_count: 10,
            search_move_count: 5,
            total_time_ms: 50,
            avg_search_time_ms: 10.0,
            search_nodes: 900,
            safety_nodes: 100,
            corridor_nodes: 0,
            corridor_branch_probes: 0,
            corridor_max_depth: 0,
            corridor_width_exits: 0,
            corridor_depth_exits: 0,
            corridor_neutral_exits: 0,
            corridor_terminal_exits: 0,
            corridor_plies_followed: 0,
            corridor_own_plies_followed: 0,
            corridor_opponent_plies_followed: 0,
            corridor_proof_passes: 0,
            corridor_proof_completed: 0,
            corridor_proof_checks: 0,
            corridor_proof_active: 0,
            corridor_proof_quiet: 0,
            corridor_proof_static_exits: 0,
            corridor_proof_depth_exits: 0,
            corridor_proof_deadline_exits: 0,
            corridor_proof_terminal_exits: 0,
            corridor_proof_terminal_root_candidates: 0,
            corridor_proof_terminal_root_winning_candidates: 0,
            corridor_proof_terminal_root_losing_candidates: 0,
            corridor_proof_terminal_root_overrides: 0,
            corridor_proof_terminal_root_move_changes: 0,
            corridor_proof_terminal_root_move_confirmations: 0,
            corridor_proof_candidates_considered: 0,
            corridor_proof_wins: 0,
            corridor_proof_losses: 0,
            corridor_proof_unknown: 0,
            corridor_proof_deadline_skips: 0,
            corridor_proof_move_changes: 0,
            corridor_proof_move_confirmations: 0,
            corridor_proof_candidate_rank_total: 0,
            corridor_proof_candidate_rank_max: 0,
            corridor_proof_candidate_score_gap_total: 0,
            corridor_proof_candidate_score_gap_max: 0,
            corridor_proof_win_candidate_rank_total: 0,
            corridor_proof_win_candidate_rank_max: 0,
            total_nodes: 1000,
            avg_nodes: 200.0,
            eval_calls: 500,
            avg_eval_calls: 100.0,
            line_shape_eval_calls: 0,
            line_shape_eval_ns: 0,
            avg_line_shape_eval_ns: 0.0,
            pattern_eval_calls: 500,
            pattern_eval_ns: 1_000_000,
            avg_pattern_eval_ns: 2000.0,
            pattern_frame_queries: 0,
            pattern_frame_query_ns: 0,
            avg_pattern_frame_query_ns: 0.0,
            pattern_frame_updates: 0,
            pattern_frame_update_ns: 0,
            avg_pattern_frame_update_ns: 0.0,
            pattern_frame_shadow_checks: 0,
            pattern_frame_shadow_mismatches: 0,
            candidate_generations: 25,
            avg_candidate_generations: 5.0,
            candidate_moves_total: 2500,
            avg_candidate_moves: 100.0,
            candidate_moves_max: 120,
            root_candidate_generations: 5,
            root_candidate_moves_total: 400,
            root_candidate_moves_max: 90,
            search_candidate_generations: 20,
            search_candidate_moves_total: 2100,
            search_candidate_moves_max: 120,
            legality_checks: 30,
            avg_legality_checks: 6.0,
            illegal_moves_skipped: 2,
            root_legality_checks: 10,
            root_illegal_moves_skipped: 1,
            search_legality_checks: 20,
            search_illegal_moves_skipped: 1,
            renju_forbidden_prefilter_checks: 30,
            avg_renju_forbidden_prefilter_checks: 6.0,
            renju_forbidden_prefilter_ns: 500_000,
            avg_renju_forbidden_prefilter_ns: 16_666.7,
            renju_forbidden_checks: 12,
            avg_renju_forbidden_checks: 2.4,
            renju_forbidden_ns: 1_000_000,
            avg_renju_forbidden_ns: 83_333.3,
            renju_forbidden_search_gate_checks: 2,
            renju_forbidden_search_gate_ns: 100_000,
            renju_forbidden_pattern_checks: 6,
            renju_forbidden_pattern_ns: 600_000,
            renju_forbidden_threat_checks: 3,
            renju_forbidden_threat_ns: 250_000,
            renju_forbidden_other_checks: 1,
            renju_forbidden_other_ns: 50_000,
            renju_effective_filter_calls: 8,
            avg_renju_effective_filter_calls: 1.6,
            renju_effective_filter_ns: 2_000_000,
            avg_renju_effective_filter_ns: 250_000.0,
            renju_effective_filter_continuation_checks: 16,
            avg_renju_effective_filter_continuation_checks: 3.2,
            renju_effective_filter_continuation_ns: 1_200_000,
            avg_renju_effective_filter_continuation_ns: 75_000.0,
            stage_move_gen_ns: 5_000_000,
            stage_ordering_ns: 10_000_000,
            stage_eval_ns: 15_000_000,
            stage_threat_ns: 2_500_000,
            stage_proof_ns: 0,
            tactical_annotations: 8,
            root_tactical_annotations: 2,
            search_tactical_annotations: 6,
            threat_view_shadow_checks: 0,
            threat_view_shadow_mismatches: 0,
            threat_view_scan_queries: 0,
            threat_view_scan_ns: 0,
            threat_view_frontier_rebuilds: 0,
            threat_view_frontier_rebuild_ns: 0,
            threat_view_frontier_queries: 0,
            threat_view_frontier_query_ns: 0,
            threat_view_frontier_immediate_win_queries: 0,
            threat_view_frontier_immediate_win_query_ns: 0,
            threat_view_frontier_delta_captures: 0,
            threat_view_frontier_delta_capture_ns: 0,
            threat_view_frontier_move_fact_updates: 0,
            threat_view_frontier_move_fact_update_ns: 0,
            threat_view_frontier_annotation_dirty_marks: 0,
            threat_view_frontier_annotation_dirty_mark_ns: 0,
            threat_view_frontier_clean_annotation_queries: 0,
            threat_view_frontier_clean_annotation_query_ns: 0,
            threat_view_frontier_dirty_annotation_queries: 0,
            threat_view_frontier_dirty_annotation_query_ns: 0,
            threat_view_frontier_fallback_annotation_queries: 0,
            threat_view_frontier_fallback_annotation_query_ns: 0,
            threat_view_frontier_memo_annotation_queries: 0,
            threat_view_frontier_memo_annotation_query_ns: 0,
            child_limit_applications: 4,
            root_child_limit_applications: 0,
            search_child_limit_applications: 4,
            child_cap_hits: 3,
            root_child_cap_hits: 0,
            search_child_cap_hits: 3,
            child_moves_before_total: 48,
            root_child_moves_before_total: 0,
            search_child_moves_before_total: 48,
            child_moves_before_max: 14,
            root_child_moves_before_max: 0,
            search_child_moves_before_max: 14,
            child_moves_after_total: 32,
            root_child_moves_after_total: 0,
            search_child_moves_after_total: 32,
            child_moves_after_max: 9,
            root_child_moves_after_max: 0,
            search_child_moves_after_max: 9,
            avg_child_moves_before: 12.0,
            avg_child_moves_after: 8.0,
            tt_hits: 7,
            tt_cutoffs: 3,
            beta_cutoffs: 9,
            avg_depth: 3.0,
            max_depth: 3,
            effective_depth_sum: 15,
            avg_effective_depth: 3.0,
            max_effective_depth: 3,
            depth_reached_counts: vec![DepthCountReport { depth: 3, count: 5 }],
            budget_exhausted_count: 1,
            budget_exhausted_rate: 0.2,
            pooled_budget_moves: 0,
            pooled_budget_over_base_count: 0,
            pooled_budget_over_base_rate: 0.0,
            pooled_budget_reserve_exhausted_count: 0,
            pooled_budget_reserve_exhausted_rate: 0.0,
            pooled_budget_avg_reserve_before_ms: 0.0,
            pooled_budget_avg_reserve_after_ms: 0.0,
            pooled_budget_min_reserve_after_ms: 0,
            pooled_budget_max_move_budget_ms: 0,
        }
    }

    fn sample_side_stats_with_search_costs() -> SideStatsReport {
        SideStatsReport {
            move_count: 5,
            search_move_count: 5,
            total_time_ms: 50,
            avg_search_time_ms: 10.0,
            search_nodes: 900,
            safety_nodes: 100,
            corridor_nodes: 0,
            corridor_branch_probes: 0,
            corridor_max_depth: 0,
            corridor_width_exits: 0,
            corridor_depth_exits: 0,
            corridor_neutral_exits: 0,
            corridor_terminal_exits: 0,
            corridor_plies_followed: 0,
            corridor_own_plies_followed: 0,
            corridor_opponent_plies_followed: 0,
            corridor_proof_passes: 0,
            corridor_proof_completed: 0,
            corridor_proof_checks: 0,
            corridor_proof_active: 0,
            corridor_proof_quiet: 0,
            corridor_proof_static_exits: 0,
            corridor_proof_depth_exits: 0,
            corridor_proof_deadline_exits: 0,
            corridor_proof_terminal_exits: 0,
            corridor_proof_terminal_root_candidates: 0,
            corridor_proof_terminal_root_winning_candidates: 0,
            corridor_proof_terminal_root_losing_candidates: 0,
            corridor_proof_terminal_root_overrides: 0,
            corridor_proof_terminal_root_move_changes: 0,
            corridor_proof_terminal_root_move_confirmations: 0,
            corridor_proof_candidates_considered: 0,
            corridor_proof_wins: 0,
            corridor_proof_losses: 0,
            corridor_proof_unknown: 0,
            corridor_proof_deadline_skips: 0,
            corridor_proof_move_changes: 0,
            corridor_proof_move_confirmations: 0,
            corridor_proof_candidate_rank_total: 0,
            corridor_proof_candidate_rank_max: 0,
            corridor_proof_candidate_score_gap_total: 0,
            corridor_proof_candidate_score_gap_max: 0,
            corridor_proof_win_candidate_rank_total: 0,
            corridor_proof_win_candidate_rank_max: 0,
            total_nodes: 1000,
            avg_nodes: 200.0,
            eval_calls: 500,
            avg_eval_calls: 100.0,
            line_shape_eval_calls: 0,
            line_shape_eval_ns: 0,
            avg_line_shape_eval_ns: 0.0,
            pattern_eval_calls: 500,
            pattern_eval_ns: 1_000_000,
            avg_pattern_eval_ns: 2000.0,
            pattern_frame_queries: 0,
            pattern_frame_query_ns: 0,
            avg_pattern_frame_query_ns: 0.0,
            pattern_frame_updates: 0,
            pattern_frame_update_ns: 0,
            avg_pattern_frame_update_ns: 0.0,
            pattern_frame_shadow_checks: 0,
            pattern_frame_shadow_mismatches: 0,
            candidate_generations: 25,
            avg_candidate_generations: 5.0,
            candidate_moves_total: 2500,
            avg_candidate_moves: 100.0,
            candidate_moves_max: 120,
            root_candidate_generations: 5,
            root_candidate_moves_total: 400,
            root_candidate_moves_max: 90,
            search_candidate_generations: 20,
            search_candidate_moves_total: 2100,
            search_candidate_moves_max: 120,
            legality_checks: 30,
            avg_legality_checks: 6.0,
            illegal_moves_skipped: 2,
            root_legality_checks: 10,
            root_illegal_moves_skipped: 1,
            search_legality_checks: 20,
            search_illegal_moves_skipped: 1,
            renju_forbidden_prefilter_checks: 30,
            avg_renju_forbidden_prefilter_checks: 6.0,
            renju_forbidden_prefilter_ns: 500_000,
            avg_renju_forbidden_prefilter_ns: 16_666.7,
            renju_forbidden_checks: 12,
            avg_renju_forbidden_checks: 2.4,
            renju_forbidden_ns: 1_000_000,
            avg_renju_forbidden_ns: 83_333.3,
            renju_forbidden_search_gate_checks: 2,
            renju_forbidden_search_gate_ns: 100_000,
            renju_forbidden_pattern_checks: 6,
            renju_forbidden_pattern_ns: 600_000,
            renju_forbidden_threat_checks: 3,
            renju_forbidden_threat_ns: 250_000,
            renju_forbidden_other_checks: 1,
            renju_forbidden_other_ns: 50_000,
            renju_effective_filter_calls: 8,
            avg_renju_effective_filter_calls: 1.6,
            renju_effective_filter_ns: 2_000_000,
            avg_renju_effective_filter_ns: 250_000.0,
            renju_effective_filter_continuation_checks: 16,
            avg_renju_effective_filter_continuation_checks: 3.2,
            renju_effective_filter_continuation_ns: 1_200_000,
            avg_renju_effective_filter_continuation_ns: 75_000.0,
            stage_move_gen_ns: 5_000_000,
            stage_ordering_ns: 10_000_000,
            stage_eval_ns: 15_000_000,
            stage_threat_ns: 2_500_000,
            stage_proof_ns: 0,
            tactical_annotations: 8,
            root_tactical_annotations: 2,
            search_tactical_annotations: 6,
            threat_view_shadow_checks: 0,
            threat_view_shadow_mismatches: 0,
            threat_view_scan_queries: 0,
            threat_view_scan_ns: 0,
            threat_view_frontier_rebuilds: 0,
            threat_view_frontier_rebuild_ns: 0,
            threat_view_frontier_queries: 0,
            threat_view_frontier_query_ns: 0,
            threat_view_frontier_immediate_win_queries: 0,
            threat_view_frontier_immediate_win_query_ns: 0,
            threat_view_frontier_delta_captures: 0,
            threat_view_frontier_delta_capture_ns: 0,
            threat_view_frontier_move_fact_updates: 0,
            threat_view_frontier_move_fact_update_ns: 0,
            threat_view_frontier_annotation_dirty_marks: 0,
            threat_view_frontier_annotation_dirty_mark_ns: 0,
            threat_view_frontier_clean_annotation_queries: 0,
            threat_view_frontier_clean_annotation_query_ns: 0,
            threat_view_frontier_dirty_annotation_queries: 0,
            threat_view_frontier_dirty_annotation_query_ns: 0,
            threat_view_frontier_fallback_annotation_queries: 0,
            threat_view_frontier_fallback_annotation_query_ns: 0,
            threat_view_frontier_memo_annotation_queries: 0,
            threat_view_frontier_memo_annotation_query_ns: 0,
            child_limit_applications: 4,
            root_child_limit_applications: 0,
            search_child_limit_applications: 4,
            child_cap_hits: 3,
            root_child_cap_hits: 0,
            search_child_cap_hits: 3,
            child_moves_before_total: 48,
            root_child_moves_before_total: 0,
            search_child_moves_before_total: 48,
            child_moves_before_max: 14,
            root_child_moves_before_max: 0,
            search_child_moves_before_max: 14,
            child_moves_after_total: 32,
            root_child_moves_after_total: 0,
            search_child_moves_after_total: 32,
            child_moves_after_max: 9,
            root_child_moves_after_max: 0,
            search_child_moves_after_max: 9,
            avg_child_moves_before: 12.0,
            avg_child_moves_after: 8.0,
            tt_hits: 7,
            tt_cutoffs: 3,
            beta_cutoffs: 9,
            depth_sum: 15,
            avg_depth: 3.0,
            max_depth: 3,
            effective_depth_sum: 15,
            avg_effective_depth: 3.0,
            max_effective_depth: 3,
            depth_reached_counts: vec![DepthCountReport { depth: 3, count: 5 }],
            budget_exhausted_count: 1,
            budget_exhausted_rate: 0.2,
            pooled_budget_moves: 0,
            pooled_budget_over_base_count: 0,
            pooled_budget_over_base_rate: 0.0,
            pooled_budget_reserve_exhausted_count: 0,
            pooled_budget_reserve_exhausted_rate: 0.0,
            pooled_budget_avg_reserve_before_ms: 0.0,
            pooled_budget_avg_reserve_after_ms: 0.0,
            pooled_budget_min_reserve_after_ms: 0,
            pooled_budget_max_move_budget_ms: 0,
        }
    }
}
