use crate::arena::MatchEndReason;
use crate::elo::{expected_score, DEFAULT_INITIAL_RATING, DEFAULT_K_FACTOR};
use crate::tournament::TournamentResults;
use gomoku_core::{Color, GameResult, Move, RuleConfig};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

pub const TOURNAMENT_REPORT_SCHEMA_VERSION: u32 = 1;
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
    fn capture() -> Self {
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
    pub corridor_extra_plies: u64,
    #[serde(default)]
    pub avg_corridor_extra_plies: f64,
    #[serde(default)]
    pub corridor_entry_checks: u64,
    #[serde(default)]
    pub corridor_entries_accepted: u64,
    #[serde(default)]
    pub corridor_entry_acceptance_rate: f64,
    #[serde(default)]
    pub corridor_own_entries_accepted: u64,
    #[serde(default)]
    pub corridor_opponent_entries_accepted: u64,
    #[serde(default)]
    pub corridor_resume_searches: u64,
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
    pub leaf_corridor_passes: u64,
    #[serde(default)]
    pub leaf_corridor_completed: u64,
    #[serde(default)]
    pub leaf_corridor_checks: u64,
    #[serde(default)]
    pub leaf_corridor_active: u64,
    #[serde(default)]
    pub leaf_corridor_quiet: u64,
    #[serde(default)]
    pub leaf_corridor_static_exits: u64,
    #[serde(default)]
    pub leaf_corridor_depth_exits: u64,
    #[serde(default)]
    pub leaf_corridor_deadline_exits: u64,
    #[serde(default)]
    pub leaf_corridor_terminal_exits: u64,
    #[serde(default)]
    pub leaf_corridor_terminal_root_candidates: u64,
    #[serde(default)]
    pub leaf_corridor_terminal_root_winning_candidates: u64,
    #[serde(default)]
    pub leaf_corridor_terminal_root_losing_candidates: u64,
    #[serde(default)]
    pub leaf_corridor_terminal_root_overrides: u64,
    #[serde(default)]
    pub leaf_corridor_terminal_root_move_changes: u64,
    #[serde(default)]
    pub leaf_corridor_terminal_root_move_confirmations: u64,
    #[serde(default)]
    pub leaf_corridor_proof_candidates_considered: u64,
    #[serde(default)]
    pub leaf_corridor_proof_wins: u64,
    #[serde(default)]
    pub leaf_corridor_proof_losses: u64,
    #[serde(default)]
    pub leaf_corridor_proof_unknown: u64,
    #[serde(default)]
    pub leaf_corridor_proof_deadline_skips: u64,
    #[serde(default)]
    pub leaf_corridor_proof_move_changes: u64,
    #[serde(default)]
    pub leaf_corridor_proof_move_confirmations: u64,
    #[serde(default)]
    pub leaf_corridor_proof_candidate_rank_total: u64,
    #[serde(default)]
    pub leaf_corridor_proof_candidate_rank_max: u64,
    #[serde(default)]
    pub leaf_corridor_proof_candidate_score_gap_total: u64,
    #[serde(default)]
    pub leaf_corridor_proof_candidate_score_gap_max: u64,
    #[serde(default)]
    pub leaf_corridor_proof_win_candidate_rank_total: u64,
    #[serde(default)]
    pub leaf_corridor_proof_win_candidate_rank_max: u64,
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
    pub tactical_lite_entry_rank_queries: u64,
    #[serde(default)]
    pub root_tactical_lite_entry_rank_queries: u64,
    #[serde(default)]
    pub search_tactical_lite_entry_rank_queries: u64,
    #[serde(default)]
    pub tactical_lite_rank_scan_queries: u64,
    #[serde(default)]
    pub tactical_lite_rank_scan_ns: u64,
    #[serde(default)]
    pub tactical_lite_rank_frontier_clean_queries: u64,
    #[serde(default)]
    pub tactical_lite_rank_frontier_clean_ns: u64,
    #[serde(default)]
    pub tactical_lite_rank_frontier_dirty_queries: u64,
    #[serde(default)]
    pub tactical_lite_rank_frontier_dirty_ns: u64,
    #[serde(default)]
    pub tactical_lite_rank_frontier_fallback_queries: u64,
    #[serde(default)]
    pub tactical_lite_rank_frontier_fallback_ns: u64,
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
    pub corridor_extra_plies: u64,
    #[serde(default)]
    pub avg_corridor_extra_plies: f64,
    #[serde(default)]
    pub corridor_entry_checks: u64,
    #[serde(default)]
    pub corridor_entries_accepted: u64,
    #[serde(default)]
    pub corridor_entry_acceptance_rate: f64,
    #[serde(default)]
    pub corridor_own_entries_accepted: u64,
    #[serde(default)]
    pub corridor_opponent_entries_accepted: u64,
    #[serde(default)]
    pub corridor_resume_searches: u64,
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
    pub leaf_corridor_passes: u64,
    #[serde(default)]
    pub leaf_corridor_completed: u64,
    #[serde(default)]
    pub leaf_corridor_checks: u64,
    #[serde(default)]
    pub leaf_corridor_active: u64,
    #[serde(default)]
    pub leaf_corridor_quiet: u64,
    #[serde(default)]
    pub leaf_corridor_static_exits: u64,
    #[serde(default)]
    pub leaf_corridor_depth_exits: u64,
    #[serde(default)]
    pub leaf_corridor_deadline_exits: u64,
    #[serde(default)]
    pub leaf_corridor_terminal_exits: u64,
    #[serde(default)]
    pub leaf_corridor_terminal_root_candidates: u64,
    #[serde(default)]
    pub leaf_corridor_terminal_root_winning_candidates: u64,
    #[serde(default)]
    pub leaf_corridor_terminal_root_losing_candidates: u64,
    #[serde(default)]
    pub leaf_corridor_terminal_root_overrides: u64,
    #[serde(default)]
    pub leaf_corridor_terminal_root_move_changes: u64,
    #[serde(default)]
    pub leaf_corridor_terminal_root_move_confirmations: u64,
    #[serde(default)]
    pub leaf_corridor_proof_candidates_considered: u64,
    #[serde(default)]
    pub leaf_corridor_proof_wins: u64,
    #[serde(default)]
    pub leaf_corridor_proof_losses: u64,
    #[serde(default)]
    pub leaf_corridor_proof_unknown: u64,
    #[serde(default)]
    pub leaf_corridor_proof_deadline_skips: u64,
    #[serde(default)]
    pub leaf_corridor_proof_move_changes: u64,
    #[serde(default)]
    pub leaf_corridor_proof_move_confirmations: u64,
    #[serde(default)]
    pub leaf_corridor_proof_candidate_rank_total: u64,
    #[serde(default)]
    pub leaf_corridor_proof_candidate_rank_max: u64,
    #[serde(default)]
    pub leaf_corridor_proof_candidate_score_gap_total: u64,
    #[serde(default)]
    pub leaf_corridor_proof_candidate_score_gap_max: u64,
    #[serde(default)]
    pub leaf_corridor_proof_win_candidate_rank_total: u64,
    #[serde(default)]
    pub leaf_corridor_proof_win_candidate_rank_max: u64,
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
    pub tactical_lite_entry_rank_queries: u64,
    #[serde(default)]
    pub root_tactical_lite_entry_rank_queries: u64,
    #[serde(default)]
    pub search_tactical_lite_entry_rank_queries: u64,
    #[serde(default)]
    pub tactical_lite_rank_scan_queries: u64,
    #[serde(default)]
    pub tactical_lite_rank_scan_ns: u64,
    #[serde(default)]
    pub tactical_lite_rank_frontier_clean_queries: u64,
    #[serde(default)]
    pub tactical_lite_rank_frontier_clean_ns: u64,
    #[serde(default)]
    pub tactical_lite_rank_frontier_dirty_queries: u64,
    #[serde(default)]
    pub tactical_lite_rank_frontier_dirty_ns: u64,
    #[serde(default)]
    pub tactical_lite_rank_frontier_fallback_queries: u64,
    #[serde(default)]
    pub tactical_lite_rank_frontier_fallback_ns: u64,
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
    corridor_extra_plies: u64,
    corridor_entry_checks: u64,
    corridor_entries_accepted: u64,
    corridor_own_entries_accepted: u64,
    corridor_opponent_entries_accepted: u64,
    corridor_resume_searches: u64,
    corridor_width_exits: u64,
    corridor_depth_exits: u64,
    corridor_neutral_exits: u64,
    corridor_terminal_exits: u64,
    corridor_plies_followed: u64,
    corridor_own_plies_followed: u64,
    corridor_opponent_plies_followed: u64,
    leaf_corridor_passes: u64,
    leaf_corridor_completed: u64,
    leaf_corridor_checks: u64,
    leaf_corridor_active: u64,
    leaf_corridor_quiet: u64,
    leaf_corridor_static_exits: u64,
    leaf_corridor_depth_exits: u64,
    leaf_corridor_deadline_exits: u64,
    leaf_corridor_terminal_exits: u64,
    leaf_corridor_terminal_root_candidates: u64,
    leaf_corridor_terminal_root_winning_candidates: u64,
    leaf_corridor_terminal_root_losing_candidates: u64,
    leaf_corridor_terminal_root_overrides: u64,
    leaf_corridor_terminal_root_move_changes: u64,
    leaf_corridor_terminal_root_move_confirmations: u64,
    leaf_corridor_proof_candidates_considered: u64,
    leaf_corridor_proof_wins: u64,
    leaf_corridor_proof_losses: u64,
    leaf_corridor_proof_unknown: u64,
    leaf_corridor_proof_deadline_skips: u64,
    leaf_corridor_proof_move_changes: u64,
    leaf_corridor_proof_move_confirmations: u64,
    leaf_corridor_proof_candidate_rank_total: u64,
    leaf_corridor_proof_candidate_rank_max: u64,
    leaf_corridor_proof_candidate_score_gap_total: u64,
    leaf_corridor_proof_candidate_score_gap_max: u64,
    leaf_corridor_proof_win_candidate_rank_total: u64,
    leaf_corridor_proof_win_candidate_rank_max: u64,
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
    stage_move_gen_ns: u64,
    stage_ordering_ns: u64,
    stage_eval_ns: u64,
    stage_threat_ns: u64,
    stage_proof_ns: u64,
    tactical_annotations: u64,
    root_tactical_annotations: u64,
    search_tactical_annotations: u64,
    tactical_lite_entry_rank_queries: u64,
    root_tactical_lite_entry_rank_queries: u64,
    search_tactical_lite_entry_rank_queries: u64,
    tactical_lite_rank_scan_queries: u64,
    tactical_lite_rank_scan_ns: u64,
    tactical_lite_rank_frontier_clean_queries: u64,
    tactical_lite_rank_frontier_clean_ns: u64,
    tactical_lite_rank_frontier_dirty_queries: u64,
    tactical_lite_rank_frontier_dirty_ns: u64,
    tactical_lite_rank_frontier_fallback_queries: u64,
    tactical_lite_rank_frontier_fallback_ns: u64,
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
            self.corridor_extra_plies += trace_value_u64(corridor, "extra_plies");
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
            self.stage_move_gen_ns += trace_value_u64(metrics, "stage_move_gen_ns");
            self.stage_ordering_ns += trace_value_u64(metrics, "stage_ordering_ns");
            self.stage_eval_ns += trace_value_u64(metrics, "stage_eval_ns");
            self.stage_threat_ns += trace_value_u64(metrics, "stage_threat_ns");
            self.stage_proof_ns += trace_value_u64(metrics, "stage_proof_ns");
            self.tactical_annotations += trace_value_u64(metrics, "tactical_annotations");
            self.root_tactical_annotations += trace_value_u64(metrics, "root_tactical_annotations");
            self.search_tactical_annotations +=
                trace_value_u64(metrics, "search_tactical_annotations");
            self.tactical_lite_entry_rank_queries +=
                trace_value_u64(metrics, "tactical_lite_entry_rank_queries");
            self.root_tactical_lite_entry_rank_queries +=
                trace_value_u64(metrics, "root_tactical_lite_entry_rank_queries");
            self.search_tactical_lite_entry_rank_queries +=
                trace_value_u64(metrics, "search_tactical_lite_entry_rank_queries");
            self.tactical_lite_rank_scan_queries +=
                trace_value_u64(metrics, "tactical_lite_rank_scan_queries");
            self.tactical_lite_rank_scan_ns +=
                trace_value_u64(metrics, "tactical_lite_rank_scan_ns");
            self.tactical_lite_rank_frontier_clean_queries +=
                trace_value_u64(metrics, "tactical_lite_rank_frontier_clean_queries");
            self.tactical_lite_rank_frontier_clean_ns +=
                trace_value_u64(metrics, "tactical_lite_rank_frontier_clean_ns");
            self.tactical_lite_rank_frontier_dirty_queries +=
                trace_value_u64(metrics, "tactical_lite_rank_frontier_dirty_queries");
            self.tactical_lite_rank_frontier_dirty_ns +=
                trace_value_u64(metrics, "tactical_lite_rank_frontier_dirty_ns");
            self.tactical_lite_rank_frontier_fallback_queries +=
                trace_value_u64(metrics, "tactical_lite_rank_frontier_fallback_queries");
            self.tactical_lite_rank_frontier_fallback_ns +=
                trace_value_u64(metrics, "tactical_lite_rank_frontier_fallback_ns");
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
            self.corridor_entry_checks += trace_value_u64(metrics, "corridor_entry_checks");
            self.corridor_entries_accepted += trace_value_u64(metrics, "corridor_entries_accepted");
            self.corridor_own_entries_accepted +=
                trace_value_u64(metrics, "corridor_own_entries_accepted");
            self.corridor_opponent_entries_accepted +=
                trace_value_u64(metrics, "corridor_opponent_entries_accepted");
            self.corridor_resume_searches += trace_value_u64(metrics, "corridor_resume_searches");
            self.corridor_width_exits += trace_value_u64(metrics, "corridor_width_exits");
            self.corridor_depth_exits += trace_value_u64(metrics, "corridor_depth_exits");
            self.corridor_neutral_exits += trace_value_u64(metrics, "corridor_neutral_exits");
            self.corridor_terminal_exits += trace_value_u64(metrics, "corridor_terminal_exits");
            self.corridor_plies_followed += trace_value_u64(metrics, "corridor_plies_followed");
            self.corridor_own_plies_followed +=
                trace_value_u64(metrics, "corridor_own_plies_followed");
            self.corridor_opponent_plies_followed +=
                trace_value_u64(metrics, "corridor_opponent_plies_followed");
            self.leaf_corridor_passes += trace_value_u64(metrics, "leaf_corridor_passes");
            self.leaf_corridor_completed += trace_value_u64(metrics, "leaf_corridor_completed");
            self.leaf_corridor_checks += trace_value_u64(metrics, "leaf_corridor_checks");
            self.leaf_corridor_active += trace_value_u64(metrics, "leaf_corridor_active");
            self.leaf_corridor_quiet += trace_value_u64(metrics, "leaf_corridor_quiet");
            self.leaf_corridor_static_exits +=
                trace_value_u64(metrics, "leaf_corridor_static_exits");
            self.leaf_corridor_depth_exits += trace_value_u64(metrics, "leaf_corridor_depth_exits");
            self.leaf_corridor_deadline_exits +=
                trace_value_u64(metrics, "leaf_corridor_deadline_exits");
            self.leaf_corridor_terminal_exits +=
                trace_value_u64(metrics, "leaf_corridor_terminal_exits");
            self.leaf_corridor_terminal_root_candidates +=
                trace_value_u64(metrics, "leaf_corridor_terminal_root_candidates");
            self.leaf_corridor_terminal_root_winning_candidates +=
                trace_value_u64(metrics, "leaf_corridor_terminal_root_winning_candidates");
            self.leaf_corridor_terminal_root_losing_candidates +=
                trace_value_u64(metrics, "leaf_corridor_terminal_root_losing_candidates");
            self.leaf_corridor_terminal_root_overrides +=
                trace_value_u64(metrics, "leaf_corridor_terminal_root_overrides");
            self.leaf_corridor_terminal_root_move_changes +=
                trace_value_u64(metrics, "leaf_corridor_terminal_root_move_changes");
            self.leaf_corridor_terminal_root_move_confirmations +=
                trace_value_u64(metrics, "leaf_corridor_terminal_root_move_confirmations");
            self.leaf_corridor_proof_candidates_considered +=
                trace_value_u64(metrics, "leaf_corridor_proof_candidates_considered");
            self.leaf_corridor_proof_wins += trace_value_u64(metrics, "leaf_corridor_proof_wins");
            self.leaf_corridor_proof_losses +=
                trace_value_u64(metrics, "leaf_corridor_proof_losses");
            self.leaf_corridor_proof_unknown +=
                trace_value_u64(metrics, "leaf_corridor_proof_unknown");
            self.leaf_corridor_proof_deadline_skips +=
                trace_value_u64(metrics, "leaf_corridor_proof_deadline_skips");
            self.leaf_corridor_proof_move_changes +=
                trace_value_u64(metrics, "leaf_corridor_proof_move_changes");
            self.leaf_corridor_proof_move_confirmations +=
                trace_value_u64(metrics, "leaf_corridor_proof_move_confirmations");
            self.leaf_corridor_proof_candidate_rank_total +=
                trace_value_u64(metrics, "leaf_corridor_proof_candidate_rank_total");
            self.leaf_corridor_proof_candidate_rank_max = self
                .leaf_corridor_proof_candidate_rank_max
                .max(trace_value_u64(
                    metrics,
                    "leaf_corridor_proof_candidate_rank_max",
                ));
            self.leaf_corridor_proof_candidate_score_gap_total +=
                trace_value_u64(metrics, "leaf_corridor_proof_candidate_score_gap_total");
            self.leaf_corridor_proof_candidate_score_gap_max = self
                .leaf_corridor_proof_candidate_score_gap_max
                .max(trace_value_u64(
                    metrics,
                    "leaf_corridor_proof_candidate_score_gap_max",
                ));
            self.leaf_corridor_proof_win_candidate_rank_total +=
                trace_value_u64(metrics, "leaf_corridor_proof_win_candidate_rank_total");
            self.leaf_corridor_proof_win_candidate_rank_max = self
                .leaf_corridor_proof_win_candidate_rank_max
                .max(trace_value_u64(
                    metrics,
                    "leaf_corridor_proof_win_candidate_rank_max",
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
        self.corridor_extra_plies += stats.corridor_extra_plies;
        self.corridor_entry_checks += stats.corridor_entry_checks;
        self.corridor_entries_accepted += stats.corridor_entries_accepted;
        self.corridor_own_entries_accepted += stats.corridor_own_entries_accepted;
        self.corridor_opponent_entries_accepted += stats.corridor_opponent_entries_accepted;
        self.corridor_resume_searches += stats.corridor_resume_searches;
        self.corridor_width_exits += stats.corridor_width_exits;
        self.corridor_depth_exits += stats.corridor_depth_exits;
        self.corridor_neutral_exits += stats.corridor_neutral_exits;
        self.corridor_terminal_exits += stats.corridor_terminal_exits;
        self.corridor_plies_followed += stats.corridor_plies_followed;
        self.corridor_own_plies_followed += stats.corridor_own_plies_followed;
        self.corridor_opponent_plies_followed += stats.corridor_opponent_plies_followed;
        self.leaf_corridor_passes += stats.leaf_corridor_passes;
        self.leaf_corridor_completed += stats.leaf_corridor_completed;
        self.leaf_corridor_checks += stats.leaf_corridor_checks;
        self.leaf_corridor_active += stats.leaf_corridor_active;
        self.leaf_corridor_quiet += stats.leaf_corridor_quiet;
        self.leaf_corridor_static_exits += stats.leaf_corridor_static_exits;
        self.leaf_corridor_depth_exits += stats.leaf_corridor_depth_exits;
        self.leaf_corridor_deadline_exits += stats.leaf_corridor_deadline_exits;
        self.leaf_corridor_terminal_exits += stats.leaf_corridor_terminal_exits;
        self.leaf_corridor_terminal_root_candidates += stats.leaf_corridor_terminal_root_candidates;
        self.leaf_corridor_terminal_root_winning_candidates +=
            stats.leaf_corridor_terminal_root_winning_candidates;
        self.leaf_corridor_terminal_root_losing_candidates +=
            stats.leaf_corridor_terminal_root_losing_candidates;
        self.leaf_corridor_terminal_root_overrides += stats.leaf_corridor_terminal_root_overrides;
        self.leaf_corridor_terminal_root_move_changes +=
            stats.leaf_corridor_terminal_root_move_changes;
        self.leaf_corridor_terminal_root_move_confirmations +=
            stats.leaf_corridor_terminal_root_move_confirmations;
        self.leaf_corridor_proof_candidates_considered +=
            stats.leaf_corridor_proof_candidates_considered;
        self.leaf_corridor_proof_wins += stats.leaf_corridor_proof_wins;
        self.leaf_corridor_proof_losses += stats.leaf_corridor_proof_losses;
        self.leaf_corridor_proof_unknown += stats.leaf_corridor_proof_unknown;
        self.leaf_corridor_proof_deadline_skips += stats.leaf_corridor_proof_deadline_skips;
        self.leaf_corridor_proof_move_changes += stats.leaf_corridor_proof_move_changes;
        self.leaf_corridor_proof_move_confirmations += stats.leaf_corridor_proof_move_confirmations;
        self.leaf_corridor_proof_candidate_rank_total +=
            stats.leaf_corridor_proof_candidate_rank_total;
        self.leaf_corridor_proof_candidate_rank_max = self
            .leaf_corridor_proof_candidate_rank_max
            .max(stats.leaf_corridor_proof_candidate_rank_max);
        self.leaf_corridor_proof_candidate_score_gap_total +=
            stats.leaf_corridor_proof_candidate_score_gap_total;
        self.leaf_corridor_proof_candidate_score_gap_max = self
            .leaf_corridor_proof_candidate_score_gap_max
            .max(stats.leaf_corridor_proof_candidate_score_gap_max);
        self.leaf_corridor_proof_win_candidate_rank_total +=
            stats.leaf_corridor_proof_win_candidate_rank_total;
        self.leaf_corridor_proof_win_candidate_rank_max = self
            .leaf_corridor_proof_win_candidate_rank_max
            .max(stats.leaf_corridor_proof_win_candidate_rank_max);
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
        self.stage_move_gen_ns += stats.stage_move_gen_ns;
        self.stage_ordering_ns += stats.stage_ordering_ns;
        self.stage_eval_ns += stats.stage_eval_ns;
        self.stage_threat_ns += stats.stage_threat_ns;
        self.stage_proof_ns += stats.stage_proof_ns;
        self.tactical_annotations += stats.tactical_annotations;
        self.root_tactical_annotations += stats.root_tactical_annotations;
        self.search_tactical_annotations += stats.search_tactical_annotations;
        self.tactical_lite_entry_rank_queries += stats.tactical_lite_entry_rank_queries;
        self.root_tactical_lite_entry_rank_queries += stats.root_tactical_lite_entry_rank_queries;
        self.search_tactical_lite_entry_rank_queries +=
            stats.search_tactical_lite_entry_rank_queries;
        self.tactical_lite_rank_scan_queries += stats.tactical_lite_rank_scan_queries;
        self.tactical_lite_rank_scan_ns += stats.tactical_lite_rank_scan_ns;
        self.tactical_lite_rank_frontier_clean_queries +=
            stats.tactical_lite_rank_frontier_clean_queries;
        self.tactical_lite_rank_frontier_clean_ns += stats.tactical_lite_rank_frontier_clean_ns;
        self.tactical_lite_rank_frontier_dirty_queries +=
            stats.tactical_lite_rank_frontier_dirty_queries;
        self.tactical_lite_rank_frontier_dirty_ns += stats.tactical_lite_rank_frontier_dirty_ns;
        self.tactical_lite_rank_frontier_fallback_queries +=
            stats.tactical_lite_rank_frontier_fallback_queries;
        self.tactical_lite_rank_frontier_fallback_ns +=
            stats.tactical_lite_rank_frontier_fallback_ns;
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
        let avg_depth = avg(self.depth_sum as f64, self.search_move_count);
        let avg_corridor_extra_plies =
            avg(self.corridor_extra_plies as f64, self.search_move_count);
        let corridor_entry_acceptance_rate =
            ratio_u64(self.corridor_entries_accepted, self.corridor_entry_checks);
        let avg_effective_depth = avg(self.effective_depth_sum as f64, self.search_move_count);
        let budget_exhausted_rate = avg(self.budget_exhausted_count as f64, self.search_move_count);
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
            corridor_extra_plies: self.corridor_extra_plies,
            avg_corridor_extra_plies,
            corridor_entry_checks: self.corridor_entry_checks,
            corridor_entries_accepted: self.corridor_entries_accepted,
            corridor_entry_acceptance_rate,
            corridor_own_entries_accepted: self.corridor_own_entries_accepted,
            corridor_opponent_entries_accepted: self.corridor_opponent_entries_accepted,
            corridor_resume_searches: self.corridor_resume_searches,
            corridor_width_exits: self.corridor_width_exits,
            corridor_depth_exits: self.corridor_depth_exits,
            corridor_neutral_exits: self.corridor_neutral_exits,
            corridor_terminal_exits: self.corridor_terminal_exits,
            corridor_plies_followed: self.corridor_plies_followed,
            corridor_own_plies_followed: self.corridor_own_plies_followed,
            corridor_opponent_plies_followed: self.corridor_opponent_plies_followed,
            leaf_corridor_passes: self.leaf_corridor_passes,
            leaf_corridor_completed: self.leaf_corridor_completed,
            leaf_corridor_checks: self.leaf_corridor_checks,
            leaf_corridor_active: self.leaf_corridor_active,
            leaf_corridor_quiet: self.leaf_corridor_quiet,
            leaf_corridor_static_exits: self.leaf_corridor_static_exits,
            leaf_corridor_depth_exits: self.leaf_corridor_depth_exits,
            leaf_corridor_deadline_exits: self.leaf_corridor_deadline_exits,
            leaf_corridor_terminal_exits: self.leaf_corridor_terminal_exits,
            leaf_corridor_terminal_root_candidates: self.leaf_corridor_terminal_root_candidates,
            leaf_corridor_terminal_root_winning_candidates: self
                .leaf_corridor_terminal_root_winning_candidates,
            leaf_corridor_terminal_root_losing_candidates: self
                .leaf_corridor_terminal_root_losing_candidates,
            leaf_corridor_terminal_root_overrides: self.leaf_corridor_terminal_root_overrides,
            leaf_corridor_terminal_root_move_changes: self.leaf_corridor_terminal_root_move_changes,
            leaf_corridor_terminal_root_move_confirmations: self
                .leaf_corridor_terminal_root_move_confirmations,
            leaf_corridor_proof_candidates_considered: self
                .leaf_corridor_proof_candidates_considered,
            leaf_corridor_proof_wins: self.leaf_corridor_proof_wins,
            leaf_corridor_proof_losses: self.leaf_corridor_proof_losses,
            leaf_corridor_proof_unknown: self.leaf_corridor_proof_unknown,
            leaf_corridor_proof_deadline_skips: self.leaf_corridor_proof_deadline_skips,
            leaf_corridor_proof_move_changes: self.leaf_corridor_proof_move_changes,
            leaf_corridor_proof_move_confirmations: self.leaf_corridor_proof_move_confirmations,
            leaf_corridor_proof_candidate_rank_total: self.leaf_corridor_proof_candidate_rank_total,
            leaf_corridor_proof_candidate_rank_max: self.leaf_corridor_proof_candidate_rank_max,
            leaf_corridor_proof_candidate_score_gap_total: self
                .leaf_corridor_proof_candidate_score_gap_total,
            leaf_corridor_proof_candidate_score_gap_max: self
                .leaf_corridor_proof_candidate_score_gap_max,
            leaf_corridor_proof_win_candidate_rank_total: self
                .leaf_corridor_proof_win_candidate_rank_total,
            leaf_corridor_proof_win_candidate_rank_max: self
                .leaf_corridor_proof_win_candidate_rank_max,
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
            stage_move_gen_ns: self.stage_move_gen_ns,
            stage_ordering_ns: self.stage_ordering_ns,
            stage_eval_ns: self.stage_eval_ns,
            stage_threat_ns: self.stage_threat_ns,
            stage_proof_ns: self.stage_proof_ns,
            tactical_annotations: self.tactical_annotations,
            root_tactical_annotations: self.root_tactical_annotations,
            search_tactical_annotations: self.search_tactical_annotations,
            tactical_lite_entry_rank_queries: self.tactical_lite_entry_rank_queries,
            root_tactical_lite_entry_rank_queries: self.root_tactical_lite_entry_rank_queries,
            search_tactical_lite_entry_rank_queries: self.search_tactical_lite_entry_rank_queries,
            tactical_lite_rank_scan_queries: self.tactical_lite_rank_scan_queries,
            tactical_lite_rank_scan_ns: self.tactical_lite_rank_scan_ns,
            tactical_lite_rank_frontier_clean_queries: self
                .tactical_lite_rank_frontier_clean_queries,
            tactical_lite_rank_frontier_clean_ns: self.tactical_lite_rank_frontier_clean_ns,
            tactical_lite_rank_frontier_dirty_queries: self
                .tactical_lite_rank_frontier_dirty_queries,
            tactical_lite_rank_frontier_dirty_ns: self.tactical_lite_rank_frontier_dirty_ns,
            tactical_lite_rank_frontier_fallback_queries: self
                .tactical_lite_rank_frontier_fallback_queries,
            tactical_lite_rank_frontier_fallback_ns: self.tactical_lite_rank_frontier_fallback_ns,
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
                corridor_extra_plies: side_stats.corridor_extra_plies,
                avg_corridor_extra_plies: side_stats.avg_corridor_extra_plies,
                corridor_entry_checks: side_stats.corridor_entry_checks,
                corridor_entries_accepted: side_stats.corridor_entries_accepted,
                corridor_entry_acceptance_rate: side_stats.corridor_entry_acceptance_rate,
                corridor_own_entries_accepted: side_stats.corridor_own_entries_accepted,
                corridor_opponent_entries_accepted: side_stats.corridor_opponent_entries_accepted,
                corridor_resume_searches: side_stats.corridor_resume_searches,
                corridor_width_exits: side_stats.corridor_width_exits,
                corridor_depth_exits: side_stats.corridor_depth_exits,
                corridor_neutral_exits: side_stats.corridor_neutral_exits,
                corridor_terminal_exits: side_stats.corridor_terminal_exits,
                corridor_plies_followed: side_stats.corridor_plies_followed,
                corridor_own_plies_followed: side_stats.corridor_own_plies_followed,
                corridor_opponent_plies_followed: side_stats.corridor_opponent_plies_followed,
                leaf_corridor_passes: side_stats.leaf_corridor_passes,
                leaf_corridor_completed: side_stats.leaf_corridor_completed,
                leaf_corridor_checks: side_stats.leaf_corridor_checks,
                leaf_corridor_active: side_stats.leaf_corridor_active,
                leaf_corridor_quiet: side_stats.leaf_corridor_quiet,
                leaf_corridor_static_exits: side_stats.leaf_corridor_static_exits,
                leaf_corridor_depth_exits: side_stats.leaf_corridor_depth_exits,
                leaf_corridor_deadline_exits: side_stats.leaf_corridor_deadline_exits,
                leaf_corridor_terminal_exits: side_stats.leaf_corridor_terminal_exits,
                leaf_corridor_terminal_root_candidates: side_stats
                    .leaf_corridor_terminal_root_candidates,
                leaf_corridor_terminal_root_winning_candidates: side_stats
                    .leaf_corridor_terminal_root_winning_candidates,
                leaf_corridor_terminal_root_losing_candidates: side_stats
                    .leaf_corridor_terminal_root_losing_candidates,
                leaf_corridor_terminal_root_overrides: side_stats
                    .leaf_corridor_terminal_root_overrides,
                leaf_corridor_terminal_root_move_changes: side_stats
                    .leaf_corridor_terminal_root_move_changes,
                leaf_corridor_terminal_root_move_confirmations: side_stats
                    .leaf_corridor_terminal_root_move_confirmations,
                leaf_corridor_proof_candidates_considered: side_stats
                    .leaf_corridor_proof_candidates_considered,
                leaf_corridor_proof_wins: side_stats.leaf_corridor_proof_wins,
                leaf_corridor_proof_losses: side_stats.leaf_corridor_proof_losses,
                leaf_corridor_proof_unknown: side_stats.leaf_corridor_proof_unknown,
                leaf_corridor_proof_deadline_skips: side_stats.leaf_corridor_proof_deadline_skips,
                leaf_corridor_proof_move_changes: side_stats.leaf_corridor_proof_move_changes,
                leaf_corridor_proof_move_confirmations: side_stats
                    .leaf_corridor_proof_move_confirmations,
                leaf_corridor_proof_candidate_rank_total: side_stats
                    .leaf_corridor_proof_candidate_rank_total,
                leaf_corridor_proof_candidate_rank_max: side_stats
                    .leaf_corridor_proof_candidate_rank_max,
                leaf_corridor_proof_candidate_score_gap_total: side_stats
                    .leaf_corridor_proof_candidate_score_gap_total,
                leaf_corridor_proof_candidate_score_gap_max: side_stats
                    .leaf_corridor_proof_candidate_score_gap_max,
                leaf_corridor_proof_win_candidate_rank_total: side_stats
                    .leaf_corridor_proof_win_candidate_rank_total,
                leaf_corridor_proof_win_candidate_rank_max: side_stats
                    .leaf_corridor_proof_win_candidate_rank_max,
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
                stage_move_gen_ns: side_stats.stage_move_gen_ns,
                stage_ordering_ns: side_stats.stage_ordering_ns,
                stage_eval_ns: side_stats.stage_eval_ns,
                stage_threat_ns: side_stats.stage_threat_ns,
                stage_proof_ns: side_stats.stage_proof_ns,
                tactical_annotations: side_stats.tactical_annotations,
                root_tactical_annotations: side_stats.root_tactical_annotations,
                search_tactical_annotations: side_stats.search_tactical_annotations,
                tactical_lite_entry_rank_queries: side_stats.tactical_lite_entry_rank_queries,
                root_tactical_lite_entry_rank_queries: side_stats
                    .root_tactical_lite_entry_rank_queries,
                search_tactical_lite_entry_rank_queries: side_stats
                    .search_tactical_lite_entry_rank_queries,
                tactical_lite_rank_scan_queries: side_stats.tactical_lite_rank_scan_queries,
                tactical_lite_rank_scan_ns: side_stats.tactical_lite_rank_scan_ns,
                tactical_lite_rank_frontier_clean_queries: side_stats
                    .tactical_lite_rank_frontier_clean_queries,
                tactical_lite_rank_frontier_clean_ns: side_stats
                    .tactical_lite_rank_frontier_clean_ns,
                tactical_lite_rank_frontier_dirty_queries: side_stats
                    .tactical_lite_rank_frontier_dirty_queries,
                tactical_lite_rank_frontier_dirty_ns: side_stats
                    .tactical_lite_rank_frontier_dirty_ns,
                tactical_lite_rank_frontier_fallback_queries: side_stats
                    .tactical_lite_rank_frontier_fallback_queries,
                tactical_lite_rank_frontier_fallback_ns: side_stats
                    .tactical_lite_rank_frontier_fallback_ns,
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

fn ratio_u64(numerator: u64, denominator: u64) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
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

pub fn render_tournament_report_html(report: &TournamentReport) -> String {
    render_tournament_report_html_with_options(report, &ReportRenderOptions::default())
}

#[derive(Debug, Clone, Default)]
pub struct ReportRenderOptions {
    pub raw_json_href: Option<String>,
    pub include_rolling_health: bool,
}

pub fn render_tournament_report_html_with_options(
    report: &TournamentReport,
    options: &ReportRenderOptions,
) -> String {
    let mut html = String::new();
    let budget = budget_label(&report.run);
    let generated_at_utc = report
        .provenance
        .generated_at_utc
        .as_deref()
        .unwrap_or("unknown");
    let generated_at_local = report
        .provenance
        .generated_at_local
        .as_deref()
        .unwrap_or(generated_at_utc);
    let git_revision = git_revision_label(&report.provenance);
    let command = command_line(&report.provenance.command);

    html.push_str("<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">");
    html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">");
    html.push_str("<title>Gomoku2D Bot Lab Report</title>");
    html.push_str(STYLE);
    html.push_str("</head><body><main><header class=\"hero\">");
    html.push_str("<nav class=\"top-links\"><a href=\"/\">Game</a><a href=\"/assets/\">Assets</a><a href=\"/analysis-report/\">Analysis</a></nav>");
    html.push_str("<p class=\"eyebrow\">Gomoku2D Bot Lab</p><h1>Bot Lab Report</h1>");
    html.push_str("<div class=\"run-strip\" aria-label=\"Run summary\">");
    run_chip(&mut html, "Schedule", schedule_summary(report));
    run_chip(
        &mut html,
        "Rule",
        variant_label(&report.run.rules).to_string(),
    );
    run_chip(&mut html, "Opening", opening_summary(report));
    run_chip(&mut html, "Budget", budget);
    run_chip(
        &mut html,
        "Wall",
        format_duration_ms(report.run.total_wall_time_ms),
    );
    run_chip(&mut html, "Finish", finish_summary(report));
    html.push_str("</div></header>");
    if report.provenance.git_dirty == Some(true) {
        html.push_str(
            "<p class=\"run-warning\">Development run: generated from a dirty git worktree.</p>",
        );
    }

    render_reference_anchors_section(&mut html, report);
    if options.include_rolling_health {
        render_threat_view_health_section(&mut html, report);
    }

    render_entrant_workbench(&mut html, report);
    render_how_to_read_section(&mut html);

    html.push_str(
        "<section class=\"provenance\"><div class=\"section-heading\"><h2>Provenance</h2></div>",
    );
    html.push_str("<dl>");
    html.push_str(&format!(
        "<dt>Generated local</dt><dd>{}</dd>",
        html_escape(generated_at_local)
    ));
    html.push_str(&format!(
        "<dt>Generated UTC</dt><dd>{}</dd>",
        html_escape(generated_at_utc)
    ));
    html.push_str(&format!(
        "<dt>Wall clock</dt><dd>{}</dd>",
        html_escape(&format_duration_ms(report.run.total_wall_time_ms))
    ));
    html.push_str(&format!(
        "<dt>Host</dt><dd>{} / {}</dd>",
        html_escape(&host_cpu_summary(report)),
        html_escape(&host_os_arch(report))
    ));
    html.push_str(&format!(
        "<dt>Git revision</dt><dd>{}</dd>",
        html_escape(&git_revision)
    ));
    html.push_str(&format!(
        "<dt>Schema</dt><dd>v{} / {}</dd>",
        report.schema_version,
        html_escape(&report.move_codec)
    ));
    if let Some(href) = &options.raw_json_href {
        let escaped_href = html_escape(href);
        html.push_str(&format!(
            "<dt>Raw JSON</dt><dd><a href=\"{escaped_href}\">{escaped_href}</a></dd>"
        ));
    }
    html.push_str("</dl>");
    html.push_str(&format!(
        "<p class=\"command\"><code>{}</code></p>",
        html_escape(&command)
    ));
    html.push_str("</section></main>");
    html.push_str("</body></html>");
    html
}

fn render_reference_anchors_section(html: &mut String, report: &TournamentReport) {
    let Some(reference) = &report.reference_anchors else {
        return;
    };

    html.push_str("<section><div class=\"section-heading\"><h2>Reference Anchors</h2></div>");
    html.push_str("<div class=\"pair-overview\">");
    html.push_str(&format!(
        "<p><b>Source</b><br>{}<br>{}, {} games/pair, {}, {} plies, seed {}</p>",
        html_escape(
            reference
                .source
                .path
                .as_deref()
                .unwrap_or("embedded report")
        ),
        html_escape(&reference.source.schedule),
        reference.source.games_per_pair,
        html_escape(&reference.source.opening_policy),
        reference.source.opening_plies,
        reference.source.seed,
    ));
    html.push_str(&format!(
        "<p><b>Reference</b><br>{}; {}; {}; git {}</p>",
        html_escape(&variant_label(&reference.source.rules)),
        html_escape(&anchor_budget_label(&reference.source)),
        html_escape(&anchor_match_cap_label(&reference.source)),
        html_escape(&anchor_reference_revision(&reference.source)),
    ));
    html.push_str("</div>");
    html.push_str("<table><thead><tr>");
    for head in [
        "Anchor",
        "Run-order Elo",
        "Shuffled Elo",
        "Score %",
        "Matches",
    ] {
        html.push_str(&format!("<th>{head}</th>"));
    }
    html.push_str("</tr></thead><tbody>");
    for anchor in &reference.anchors {
        html.push_str(&format!(
            "<tr><td>{}</td><td>{:.1}</td><td>{:.1} +/- {:.1}</td><td>{:.1}%</td><td>{}</td></tr>",
            html_escape(&compact_bot_label(report, &anchor.bot)),
            anchor.sequential_elo,
            anchor.shuffled_elo_avg,
            anchor.shuffled_elo_stddev,
            anchor.score_percentage,
            anchor.match_count,
        ));
    }
    html.push_str("</tbody></table></section>");
}

#[derive(Debug, Default)]
struct ThreatViewHealthTotals {
    shadow_checks: u64,
    shadow_mismatches: u64,
    scan_queries: u64,
    scan_ns: u64,
    frontier_rebuilds: u64,
    frontier_rebuild_ns: u64,
    frontier_queries: u64,
    frontier_query_ns: u64,
    frontier_immediate_win_queries: u64,
    frontier_immediate_win_query_ns: u64,
    frontier_delta_captures: u64,
    frontier_delta_capture_ns: u64,
    frontier_move_fact_updates: u64,
    frontier_move_fact_update_ns: u64,
    frontier_annotation_dirty_marks: u64,
    frontier_annotation_dirty_mark_ns: u64,
    frontier_clean_annotation_queries: u64,
    frontier_clean_annotation_query_ns: u64,
    frontier_dirty_annotation_queries: u64,
    frontier_dirty_annotation_query_ns: u64,
    frontier_fallback_annotation_queries: u64,
    frontier_fallback_annotation_query_ns: u64,
    frontier_memo_annotation_queries: u64,
    frontier_memo_annotation_query_ns: u64,
}

impl ThreatViewHealthTotals {
    fn from_report(report: &TournamentReport) -> Self {
        let mut totals = Self::default();
        for row in &report.standings {
            totals.shadow_checks += row.threat_view_shadow_checks;
            totals.shadow_mismatches += row.threat_view_shadow_mismatches;
            totals.scan_queries += row.threat_view_scan_queries;
            totals.scan_ns += row.threat_view_scan_ns;
            totals.frontier_rebuilds += row.threat_view_frontier_rebuilds;
            totals.frontier_rebuild_ns += row.threat_view_frontier_rebuild_ns;
            totals.frontier_queries += row.threat_view_frontier_queries;
            totals.frontier_query_ns += row.threat_view_frontier_query_ns;
            totals.frontier_immediate_win_queries += row.threat_view_frontier_immediate_win_queries;
            totals.frontier_immediate_win_query_ns +=
                row.threat_view_frontier_immediate_win_query_ns;
            totals.frontier_delta_captures += row.threat_view_frontier_delta_captures;
            totals.frontier_delta_capture_ns += row.threat_view_frontier_delta_capture_ns;
            totals.frontier_move_fact_updates += row.threat_view_frontier_move_fact_updates;
            totals.frontier_move_fact_update_ns += row.threat_view_frontier_move_fact_update_ns;
            totals.frontier_annotation_dirty_marks +=
                row.threat_view_frontier_annotation_dirty_marks;
            totals.frontier_annotation_dirty_mark_ns +=
                row.threat_view_frontier_annotation_dirty_mark_ns;
            totals.frontier_clean_annotation_queries +=
                row.threat_view_frontier_clean_annotation_queries;
            totals.frontier_clean_annotation_query_ns +=
                row.threat_view_frontier_clean_annotation_query_ns;
            totals.frontier_dirty_annotation_queries +=
                row.threat_view_frontier_dirty_annotation_queries;
            totals.frontier_dirty_annotation_query_ns +=
                row.threat_view_frontier_dirty_annotation_query_ns;
            totals.frontier_fallback_annotation_queries +=
                row.threat_view_frontier_fallback_annotation_queries;
            totals.frontier_fallback_annotation_query_ns +=
                row.threat_view_frontier_fallback_annotation_query_ns;
            totals.frontier_memo_annotation_queries +=
                row.threat_view_frontier_memo_annotation_queries;
            totals.frontier_memo_annotation_query_ns +=
                row.threat_view_frontier_memo_annotation_query_ns;
        }
        totals
    }

    fn has_threat_view_metrics(&self) -> bool {
        self.shadow_checks
            + self.scan_queries
            + self.frontier_rebuilds
            + self.frontier_queries
            + self.frontier_immediate_win_queries
            + self.frontier_memo_annotation_queries
            > 0
    }

    fn frontier_update_ns(&self) -> u64 {
        // `frontier_rebuild_ns` is the total frontier construction/apply/undo
        // timing recorded by search; the delta/move-fact/dirty timings below
        // are subparts of apply/undo work, not additive.
        self.frontier_rebuild_ns
    }

    fn frontier_annotation_query_ns(&self) -> u64 {
        self.frontier_clean_annotation_query_ns
            + self.frontier_dirty_annotation_query_ns
            + self.frontier_fallback_annotation_query_ns
            + self.frontier_memo_annotation_query_ns
    }
}

fn render_threat_view_health_section(html: &mut String, report: &TournamentReport) {
    let totals = ThreatViewHealthTotals::from_report(report);
    if !totals.has_threat_view_metrics() {
        return;
    }

    html.push_str("<details class=\"threat-health diagnostic-panel\"><summary><h2>Rolling Health</h2><span>Diagnostics</span></summary>");
    html.push_str("<div class=\"health-grid\">");
    let shadow_secondary = if totals.shadow_checks == 0 {
        "not active".to_string()
    } else {
        format!(
            "{:.2}% mismatch rate",
            ratio_u64(totals.shadow_mismatches, totals.shadow_checks) * 100.0
        )
    };
    health_card(
        html,
        "Shadow",
        &format!(
            "{} mismatches / {} checks",
            totals.shadow_mismatches, totals.shadow_checks
        ),
        &shadow_secondary,
    );
    health_card(
        html,
        "Scan",
        &duration_ns_label(totals.scan_ns),
        &format!("{} queries", compact_u64_label(totals.scan_queries)),
    );
    health_card(
        html,
        "Frontier",
        &duration_ns_label(totals.frontier_query_ns),
        &format!(
            "{} queries / {} wins / {} update",
            compact_u64_label(totals.frontier_queries),
            compact_u64_label(totals.frontier_immediate_win_queries),
            duration_ns_label(totals.frontier_update_ns())
        ),
    );
    health_card(
        html,
        "Annotation",
        &format!(
            "{} clean / {} dirty",
            compact_u64_label(totals.frontier_clean_annotation_queries),
            compact_u64_label(totals.frontier_dirty_annotation_queries)
        ),
        &format!(
            "{} memo / {} fallback / {}",
            compact_u64_label(totals.frontier_memo_annotation_queries),
            compact_u64_label(totals.frontier_fallback_annotation_queries),
            duration_ns_label(totals.frontier_annotation_query_ns())
        ),
    );
    html.push_str("</div>");

    let comparisons = rolling_cost_comparisons(report);
    if !comparisons.is_empty() {
        html.push_str("<div class=\"rolling-comparison-list\">");
        for comparison in comparisons {
            html.push_str(&format!(
                "<p><b>{}</b><span>{}</span><span>{}</span></p>",
                html_escape(&comparison.bot_label),
                html_escape(&comparison.time_label),
                html_escape(&comparison.node_label)
            ));
        }
        html.push_str("</div>");
    }
    html.push_str("</details>");
}

fn health_card(html: &mut String, label: &str, primary: &str, secondary: &str) {
    html.push_str(&format!(
        "<article class=\"health-card\"><span>{}</span><strong>{}</strong><em>{}</em></article>",
        html_escape(label),
        html_escape(primary),
        html_escape(secondary),
    ));
}

struct RollingCostComparison {
    bot_label: String,
    time_label: String,
    node_label: String,
}

fn rolling_cost_comparisons(report: &TournamentReport) -> Vec<RollingCostComparison> {
    let standings = report
        .standings
        .iter()
        .map(|row| (row.bot.as_str(), row))
        .collect::<HashMap<_, _>>();
    let mut comparisons = Vec::new();

    for row in &report.standings {
        let Some(base_bot) = rolling_base_bot(&row.bot) else {
            continue;
        };
        let Some(base) = standings.get(base_bot.as_str()) else {
            continue;
        };

        comparisons.push(RollingCostComparison {
            bot_label: format!("{} vs scan", compact_bot_label(report, &row.bot)),
            time_label: format!(
                "{} vs {} ({})",
                ms_label(row.avg_search_time_ms),
                ms_label(base.avg_search_time_ms),
                signed_percent_delta_label(row.avg_search_time_ms, base.avg_search_time_ms)
            ),
            node_label: format!(
                "{} vs {}",
                nodes_label(row.avg_nodes),
                nodes_label(base.avg_nodes)
            ),
        });
    }

    comparisons
}

fn rolling_base_bot(bot: &str) -> Option<String> {
    if let Some(base) = bot.strip_suffix("+rolling-frontier-shadow") {
        Some(base.to_string())
    } else {
        bot.strip_suffix("+rolling-frontier")
            .map(|base| base.to_string())
    }
}

fn anchor_reference_revision(source: &AnchorReferenceSource) -> String {
    let mut revision = source
        .git_commit
        .as_deref()
        .unwrap_or("unknown")
        .to_string();
    if source.git_dirty == Some(true) {
        revision.push_str("_dirty");
    }
    revision
}

fn anchor_budget_label(source: &AnchorReferenceSource) -> String {
    let base = match (source.search_cpu_time_ms, source.search_time_ms) {
        (Some(cpu_ms), Some(wall_ms)) => {
            format!("CPU {cpu_ms} ms/move, wall {wall_ms} ms/move")
        }
        (Some(cpu_ms), None) => format!("CPU {cpu_ms} ms/move"),
        (None, Some(wall_ms)) => format!("Wall {wall_ms} ms/move"),
        (None, None) => "no per-move budget".to_string(),
    };
    if source.search_budget_mode == "pooled" {
        match source.search_cpu_reserve_ms {
            Some(reserve_ms) => format!("{base}, pooled reserve {reserve_ms} ms"),
            None => format!("{base}, pooled"),
        }
    } else {
        base
    }
}

fn anchor_match_cap_label(source: &AnchorReferenceSource) -> String {
    match (source.max_moves, source.max_game_ms) {
        (Some(max_moves), Some(max_game_ms)) => {
            format!(
                "max {max_moves} moves, {}",
                format_duration_ms(Some(max_game_ms))
            )
        }
        (Some(max_moves), None) => format!("max {max_moves} moves"),
        (None, Some(max_game_ms)) => format!("max {}", format_duration_ms(Some(max_game_ms))),
        (None, None) => "no match cap".to_string(),
    }
}

fn git_revision_label(provenance: &ReportProvenance) -> String {
    let mut revision = provenance
        .git_commit
        .as_deref()
        .unwrap_or("unknown")
        .to_string();

    if provenance.git_dirty == Some(true) {
        revision.push_str("_dirty");
    }

    revision
}

fn budget_label(run: &TournamentRunReport) -> String {
    let base = match (run.search_cpu_time_ms, run.search_time_ms) {
        (Some(cpu_ms), Some(wall_ms)) => {
            format!("CPU {cpu_ms} ms/move, wall {wall_ms} ms/move")
        }
        (Some(cpu_ms), None) => format!("CPU {cpu_ms} ms/move"),
        (None, Some(wall_ms)) => format!("Wall {wall_ms} ms/move"),
        (None, None) => "no per-move budget".to_string(),
    };
    if run.search_budget_mode == "pooled" {
        match run.search_cpu_reserve_ms {
            Some(reserve_ms) => format!("{base}, pooled reserve {reserve_ms} ms"),
            None => format!("{base}, pooled"),
        }
    } else {
        base
    }
}

fn finish_summary(report: &TournamentReport) -> String {
    if report.end_reasons.is_empty() {
        return "none".to_string();
    }

    let finished = count_end_reason(report, "natural");
    let max_moves = count_end_reason(report, "max_moves");
    let max_time = count_end_reason(report, "max_game_time");
    let mut parts = Vec::new();

    if finished > 0 {
        parts.push(format!("{finished} finished"));
    }
    if max_moves > 0 {
        parts.push(format!("{max_moves} max moves"));
    }
    if max_time > 0 {
        parts.push(format!("{max_time} max time"));
    }

    if parts.is_empty() {
        "none".to_string()
    } else {
        parts.join(" / ")
    }
}

fn count_end_reason(report: &TournamentReport, key: &str) -> u32 {
    report
        .end_reasons
        .iter()
        .find(|reason| reason.key == key)
        .map(|reason| reason.count)
        .unwrap_or(0)
}

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

fn opening_summary(report: &TournamentReport) -> String {
    format!(
        "{}, base seed {}, {} plies",
        report.run.opening_policy, report.run.seed, report.run.opening_plies
    )
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

fn compact_bot_label(report: &TournamentReport, bot: &str) -> String {
    if bot == "random" {
        return "RandomBot".to_string();
    }

    if let Some((depth, features)) = searchbot_spec(bot, &report.run) {
        let mut label = format!("SearchBot_D{depth}");
        for feature in features {
            label.push('+');
            label.push_str(&compact_searchbot_feature_label(feature));
        }
        return label;
    }

    bot.to_string()
}

fn compact_bot_label_parts(report: &TournamentReport, bot: &str) -> (String, Option<String>) {
    let label = compact_bot_label(report, bot);
    let Some((primary, modifiers)) = label.split_once('+') else {
        return (label, None);
    };

    (
        primary.to_string(),
        Some(modifiers.split('+').collect::<Vec<_>>().join(" + ")),
    )
}

fn searchbot_spec<'a>(bot: &'a str, run: &TournamentRunReport) -> Option<(i32, Vec<&'a str>)> {
    let mut parts = bot.split('+');
    let base = parts.next()?;
    let depth = searchbot_base_depth(base, run)?;
    Some((depth, parts.collect()))
}

fn searchbot_base_depth(bot: &str, run: &TournamentRunReport) -> Option<i32> {
    match bot {
        "fast" => Some(2),
        "balanced" => Some(3),
        "deep" => Some(5),
        "baseline" | "search"
            if run.search_time_ms.is_some() || run.search_cpu_time_ms.is_some() =>
        {
            Some(20)
        }
        "baseline" | "search" => Some(5),
        _ => bot
            .strip_prefix("baseline-")
            .or_else(|| bot.strip_prefix("search-"))
            .map(|depth| depth.strip_prefix('d').unwrap_or(depth))
            .and_then(|depth| depth.parse::<i32>().ok()),
    }
}

fn compact_searchbot_feature_label(feature: &str) -> String {
    if let Some(cap) = feature.strip_prefix("tactical-cap-") {
        return format!("TCap{cap}");
    }
    if let Some(cap) = feature.strip_prefix("tactical-full-cap-") {
        return format!("TFullCap{cap}");
    }
    if let Some(cap) = feature.strip_prefix("child-cap-") {
        return format!("Cap{cap}");
    }
    if let Some(radius) = feature.strip_prefix("near-all-r") {
        return format!("NearR{radius}");
    }
    if let Some(rest) = feature.strip_prefix("near-self-r") {
        if let Some((self_radius, opponent_radius)) = rest.split_once("-opponent-r") {
            return format!("SelfR{self_radius}OppR{opponent_radius}");
        }
    }
    if let Some(rest) = feature.strip_prefix("corridor-own-d") {
        if let Some((depth, width)) = rest.split_once("-w") {
            return format!("OwnCorrD{depth}W{width}");
        }
    }
    if let Some(rest) = feature.strip_prefix("corridor-opponent-d") {
        if let Some((depth, width)) = rest.split_once("-w") {
            return format!("OppCorrD{depth}W{width}");
        }
    }
    if feature.starts_with("corridor-proof-") {
        return "Corridor Proof".to_string();
    }
    if let Some(rest) = feature.strip_prefix("leaf-corridor-d") {
        if let Some((depth, width)) = rest.split_once("-w") {
            return format!("LeafCorrD{depth}W{width}");
        }
    }
    if let Some(count) = feature.strip_prefix("leaf-proof-c") {
        return format!("ProofC{count}");
    }
    match feature {
        "pattern-eval" => "Pattern".to_string(),
        "rolling-frontier" => "Rolling".to_string(),
        "rolling-frontier-shadow" => "RollingShadow".to_string(),
        "tactical-full" => "TFull".to_string(),
        "no-safety" => "NoSafety".to_string(),
        "opponent-reply-search-probe" => "SearchProbe".to_string(),
        "opponent-reply-local-threat-probe" => "LocalThreat".to_string(),
        _ => feature.to_string(),
    }
}

fn format_duration_ms(value: Option<u64>) -> String {
    let Some(ms) = value else {
        return "not captured".to_string();
    };
    if ms < 1_000 {
        format!("{ms} ms")
    } else if ms < 60_000 {
        format!("{:.2} s", ms as f64 / 1_000.0)
    } else {
        let minutes = ms / 60_000;
        let seconds = (ms % 60_000) as f64 / 1_000.0;
        format!("{minutes}m {seconds:.1}s")
    }
}

fn duration_ns_label(ns: u64) -> String {
    if ns == 0 {
        "0 ms".to_string()
    } else if ns < 1_000 {
        format!("{ns} ns")
    } else if ns < 1_000_000 {
        format!("{:.1} us", ns as f64 / 1_000.0)
    } else if ns < 1_000_000_000 {
        format!("{:.1} ms", ns as f64 / 1_000_000.0)
    } else {
        format!("{:.2} s", ns as f64 / 1_000_000_000.0)
    }
}

fn compact_u64_label(value: u64) -> String {
    if value < 1_000 {
        value.to_string()
    } else if value < 1_000_000 {
        format!("{:.1}k", value as f64 / 1_000.0)
    } else {
        format!("{:.1}M", value as f64 / 1_000_000.0)
    }
}

fn signed_percent_delta_label(value: f64, baseline: f64) -> String {
    if baseline == 0.0 {
        return "n/a".to_string();
    }

    let delta = ((value - baseline) / baseline) * 100.0;
    format!("{delta:+.1}%")
}

fn host_cpu_summary(report: &TournamentReport) -> String {
    let model = report
        .provenance
        .host
        .as_ref()
        .and_then(|host| host.cpu_model.clone())
        .map(|model| {
            model
                .replace(" 12-Core Processor", "")
                .replace(" Processor", "")
        })
        .unwrap_or_else(|| "unknown CPU".to_string());
    let Some(mhz) = report
        .provenance
        .host
        .as_ref()
        .and_then(|host| host.cpu_mhz)
    else {
        return model;
    };

    format!("{model} @ {:.1} GHz", mhz / 1_000.0)
}

fn host_os_arch(report: &TournamentReport) -> String {
    report
        .provenance
        .host
        .as_ref()
        .map(|host| format!("{} {}", host.os, host.arch))
        .unwrap_or_else(|| "unknown host".to_string())
}

fn score_rate(wins: u32, draws: u32, total: u32) -> f64 {
    avg(wins as f64 + draws as f64 * 0.5, total)
}

fn ms_label(value: f64) -> String {
    format!("{value:.1} ms")
}

fn signed_ms_label(value: f64) -> String {
    format!("{:+.1} ms", normalized_zero(value, 0.05))
}

fn nodes_label(value: f64) -> String {
    format!("{} nodes", compact_number_label(value))
}

fn signed_nodes_label(value: f64) -> String {
    let normalized = normalized_zero(value, 0.5);
    format!(
        "{}{} nodes",
        sign_prefix(normalized),
        compact_number_label(normalized.abs())
    )
}

fn normalized_zero(value: f64, threshold: f64) -> f64 {
    if value.abs() < threshold {
        0.0
    } else {
        value
    }
}

fn sign_prefix(value: f64) -> &'static str {
    if value >= 0.0 {
        "+"
    } else {
        "-"
    }
}

fn compact_number_label(value: f64) -> String {
    let abs = value.abs();
    if abs >= 1_000_000.0 {
        format!("{:.1}m", value / 1_000_000.0)
    } else if abs >= 1_000.0 {
        format!("{:.1}k", value / 1_000.0)
    } else {
        format!("{value:.0}")
    }
}

fn command_line(command: &[String]) -> String {
    if command.is_empty() {
        return "not captured".to_string();
    }
    command
        .iter()
        .map(|part| {
            if part.contains(char::is_whitespace) {
                format!("{part:?}")
            } else {
                part.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn render_how_to_read_section(html: &mut String) {
    html.push_str("<section class=\"how-to-read\"><div class=\"section-heading\"><h2>How To Read This</h2></div><dl class=\"term-list\">");
    term_row(
        html,
        "Run Shape",
        "Schedule shows the pairing count, games per pair, and total matches. Opening shows the seeded legal moves before bots take over.",
    );
    term_row(
        html,
        "Elo",
        "Relative rating within this report only. Shuffled Elo averages repeated Elo passes over randomized match order to reduce run-order noise.",
    );
    term_row(
        html,
        "Score",
        "Score % counts wins plus half draws. W-D-L is wins, draws, then losses. Comparisons above 50% are marked green.",
    );
    term_row(
        html,
        "Budget Exhausted",
        "Share of searched moves that hit the per-move CPU cap before search finished naturally.",
    );
    term_row(
        html,
        "Search Cost",
        "Width is the average number of moves searched. The Search tab splits measured time into move generation, ordering, scoring, threat detection, corridor proof, and uncategorized search overhead.",
    );
    html.push_str("</dl></section>");
}

fn render_entrant_workbench(html: &mut String, report: &TournamentReport) {
    html.push_str("<section class=\"entrant-workbench\"><div class=\"section-heading\"><h2>Results</h2></div>");
    html.push_str("<div class=\"view-toggle\" aria-label=\"Entrant table mode\">");
    html.push_str("<input class=\"report-view-radio\" type=\"radio\" name=\"entrant-view\" id=\"view-results\" checked>");
    html.push_str("<label for=\"view-results\">Ranking</label>");
    html.push_str("<input class=\"report-view-radio\" type=\"radio\" name=\"entrant-view\" id=\"view-search\">");
    html.push_str("<label for=\"view-search\">Search</label>");
    html.push_str("<input class=\"report-view-radio\" type=\"radio\" name=\"entrant-view\" id=\"view-pairwise\">");
    html.push_str("<label for=\"view-pairwise\">Pairwise</label>");
    html.push_str("</div>");
    html.push_str("<div class=\"entrant-grid\">");
    render_entrant_header(html);
    for (index, row) in report.standings.iter().enumerate() {
        render_entrant_row(html, report, row, index + 1);
    }
    html.push_str("</div></section>");
}

fn render_entrant_header(html: &mut String) {
    html.push_str("<div class=\"entrant-head\">");
    html.push_str("<span>Spec</span>");
    for head in [
        "Rank",
        "Score %",
        "W-D-L",
        "Shuffled Elo",
        "Depth",
        "Width",
        "Avg ms",
        "Budget exhausted",
    ] {
        let metric_class = if head == "W-D-L" {
            "metric metric-results metric-nowrap"
        } else {
            "metric metric-results"
        };
        html.push_str(&format!("<span class=\"{metric_class}\">{head}</span>"));
    }
    for head in [
        "Nodes",
        "Move gen",
        "Ordering",
        "Scoring",
        "Threat detection",
        "Proof",
        "Other",
        "TT",
    ] {
        html.push_str(&format!(
            "<span class=\"metric metric-search\">{head}</span>"
        ));
    }
    for head in ["Pairs", "Best", "Worst"] {
        html.push_str(&format!(
            "<span class=\"metric metric-pairwise\">{head}</span>"
        ));
    }
    html.push_str("</div>");
}

fn render_entrant_row(
    html: &mut String,
    report: &TournamentReport,
    row: &StandingReport,
    rank: usize,
) {
    let pairwise_entries = ranked_pairwise_entries_for_bot(report, &row.bot);
    let best_pair = best_pair_for_bot(report, &row.bot, &pairwise_entries);
    let worst_pair = worst_pair_for_bot(report, &row.bot, &pairwise_entries);
    let role = gauntlet_role(report, &row.bot);
    let role_class = role
        .map(|role| format!(" role-{}", role.key()))
        .unwrap_or_default();

    html.push_str(&format!(
        "<details class=\"entrant-row{role_class}\"><summary>"
    ));
    render_bot_label(html, report, &row.bot);
    render_metric_cell(html, "metric-results", "Rank", &format!("#{rank}"), None);
    render_metric_cell(
        html,
        "metric-results",
        "Score %",
        &format!(
            "{:.1}%",
            score_rate(row.wins, row.draws, row.match_count) * 100.0
        ),
        None,
    );
    render_metric_cell(
        html,
        "metric-results metric-nowrap",
        "W-D-L",
        &format!("{}-{}-{}", row.wins, row.draws, row.losses),
        None,
    );
    render_metric_cell(
        html,
        "metric-results",
        "Shuffled Elo",
        &format!("{:.1}", row.shuffled_elo_avg),
        Some(format!("+/- {:.1}", row.shuffled_elo_stddev)),
    );
    render_metric_cell(
        html,
        "metric-results",
        "Depth",
        &format!("{:.2}", row.avg_depth),
        (row.avg_effective_depth > row.avg_depth)
            .then(|| format!("eff {:.2}", row.avg_effective_depth)),
    );
    render_width_metric_cell(html, "metric-results", "Width", row);
    render_metric_cell(
        html,
        "metric-results",
        "Avg ms",
        &format!("{:.1}", row.avg_search_time_ms),
        None,
    );
    render_metric_cell(
        html,
        "metric-results",
        "Budget exhausted",
        &format!("{:.0}%", row.budget_exhausted_rate * 100.0),
        None,
    );
    render_metric_cell(
        html,
        "metric-search",
        "Nodes",
        &compact_number_label(row.avg_nodes),
        None,
    );
    render_stage_time_metric_cell(html, "Move gen", row.stage_move_gen_ns, row);
    render_stage_time_metric_cell(html, "Ordering", row.stage_ordering_ns, row);
    render_stage_time_metric_cell(html, "Scoring", row.stage_eval_ns, row);
    render_stage_time_metric_cell(html, "Threat detection", row.stage_threat_ns, row);
    render_stage_time_metric_cell(html, "Proof", row.stage_proof_ns, row);
    render_stage_time_metric_cell(html, "Other", stage_other_ns(row), row);
    render_metric_cell(
        html,
        "metric-search",
        "TT",
        &phase_average_label(row.tt_hits, row.tt_cutoffs, row.search_move_count),
        None,
    );
    render_metric_cell(
        html,
        "metric-pairwise",
        "Pairs",
        &format!("{} opponents", pairwise_entries.len()),
        None,
    );
    render_metric_cell(html, "metric-pairwise", "Best", &best_pair.0, best_pair.1);
    render_metric_cell(
        html,
        "metric-pairwise",
        "Worst",
        &worst_pair.0,
        worst_pair.1,
    );
    html.push_str("</summary>");
    render_entrant_result_comparisons(html, report, row, &pairwise_entries);
    render_entrant_search_comparisons(html, report, row, &pairwise_entries);
    render_entrant_pairwise(html, report, &row.bot, &pairwise_entries);
    html.push_str("</details>");
}

fn render_bot_label(html: &mut String, report: &TournamentReport, bot: &str) {
    render_bot_label_with_prefix(html, report, bot, "");
}

fn render_bot_label_with_prefix(
    html: &mut String,
    report: &TournamentReport,
    bot: &str,
    prefix: &str,
) {
    let (primary, modifiers) = compact_bot_label_parts(report, bot);
    html.push_str(&format!(
        "<strong class=\"bot-label\"><span>{}</span>",
        html_escape(&format!("{prefix}{primary}"))
    ));
    if let Some(modifiers) = modifiers {
        html.push_str(&format!("<span>{}</span>", html_escape(&modifiers)));
    }
    if let Some(role) = gauntlet_role(report, bot) {
        html.push_str(&format!(
            "<span class=\"role-badge role-badge-{}\">{}</span>",
            role.key(),
            role.label()
        ));
    }
    html.push_str("</strong>");
}

#[derive(Clone, Copy)]
enum GauntletRole {
    Candidate,
    Anchor,
}

impl GauntletRole {
    fn key(self) -> &'static str {
        match self {
            Self::Candidate => "candidate",
            Self::Anchor => "anchor",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Candidate => "candidate",
            Self::Anchor => "anchor",
        }
    }
}

fn gauntlet_role(report: &TournamentReport, bot: &str) -> Option<GauntletRole> {
    if report.run.schedule != "gauntlet" {
        return None;
    }

    let reference = report.reference_anchors.as_ref()?;
    if reference.anchors.iter().any(|anchor| anchor.bot == bot) {
        Some(GauntletRole::Anchor)
    } else {
        Some(GauntletRole::Candidate)
    }
}

fn render_metric_cell(
    html: &mut String,
    metric_class: &str,
    label: &str,
    primary: &str,
    secondary: Option<String>,
) {
    html.push_str(&format!(
        "<span class=\"metric {metric_class}\" data-label=\"{}\"><span>{}</span>",
        html_escape(label),
        html_escape(primary)
    ));
    if let Some(secondary) = secondary {
        html.push_str(&format!("<span>{}</span>", html_escape(&secondary)));
    }
    html.push_str("</span>");
}

fn render_width_metric_cell(
    html: &mut String,
    metric_class: &str,
    label: &str,
    row: &StandingReport,
) {
    let (primary, secondary) = width_metric_label(row);
    render_metric_cell(html, metric_class, label, &primary, secondary);
}

fn width_metric_label(row: &StandingReport) -> (String, Option<String>) {
    if row.child_limit_applications > 0 {
        (
            format!("{:.1}", row.avg_child_moves_after),
            Some(format!("pre {:.1}", row.avg_child_moves_before)),
        )
    } else {
        (format!("{:.1}", row.avg_candidate_moves), None)
    }
}

fn render_stage_time_metric_cell(
    html: &mut String,
    label: &str,
    stage_ns: u64,
    row: &StandingReport,
) {
    let denominator_ns = stage_time_denominator_ns(row);
    let pct = if denominator_ns == 0 {
        0.0
    } else {
        stage_ns as f64 * 100.0 / denominator_ns as f64
    };
    render_metric_cell(
        html,
        "metric-search",
        label,
        &format!("{:.0}%", pct),
        Some(stage_avg_ms_label(stage_ns, row.search_move_count)),
    );
}

fn stage_avg_ms_label(stage_ns: u64, search_move_count: u32) -> String {
    let avg_ms = avg(stage_ns as f64 / 1_000_000.0, search_move_count);
    if avg_ms < 0.05 {
        "0 ms".to_string()
    } else {
        format!("{avg_ms:.1} ms")
    }
}

fn stage_known_ns(row: &StandingReport) -> u64 {
    row.stage_move_gen_ns
        .saturating_add(row.stage_ordering_ns)
        .saturating_add(row.stage_eval_ns)
        .saturating_add(row.stage_threat_ns)
        .saturating_add(row.stage_proof_ns)
}

fn stage_time_denominator_ns(row: &StandingReport) -> u64 {
    let wall_ns = row.total_time_ms.saturating_mul(1_000_000);
    wall_ns.max(stage_known_ns(row))
}

fn stage_other_ns(row: &StandingReport) -> u64 {
    let denominator_ns = stage_time_denominator_ns(row);
    let known_ns = stage_known_ns(row);
    denominator_ns.saturating_sub(known_ns)
}

fn delta_cell(label: &str, delta_class: &str, data_label: &str) -> String {
    format!(
        "<span class=\"delta {delta_class}\" data-label=\"{}\">{}</span>",
        html_escape(data_label),
        html_escape(label)
    )
}

fn cost_delta_class(value: f64, threshold: f64) -> &'static str {
    if value < -threshold {
        "delta-good"
    } else if value > threshold {
        "delta-bad"
    } else {
        "delta-neutral"
    }
}

fn score_cell(score: f64) -> String {
    let score_class = if score > 50.0 {
        "score-good"
    } else {
        "score-bad"
    };
    format!("<span class=\"score {score_class}\" data-label=\"Score\">{score:.1}%</span>")
}

fn render_entrant_result_comparisons(
    html: &mut String,
    report: &TournamentReport,
    row: &StandingReport,
    pairs: &[PairwiseEntry<'_>],
) {
    html.push_str("<div class=\"entrant-result-comparisons\">");
    html.push_str(
        "<div class=\"comparison-head\"><span>Opponent</span><span>Source</span><span>Score</span><span>Record</span></div>",
    );
    for entry in pairs {
        let pair = entry.pair;
        let opponent = opponent_for_pair(pair, &row.bot);
        let pair_rate = pair_score_rate_for_bot(pair, &row.bot);
        html.push_str("<div class=\"comparison-row\">");
        html.push_str(&format!(
            "<span data-label=\"Opponent\">Vs {}</span><span data-label=\"Source\">{}</span>{}<span data-label=\"Record\">{}</span>",
            html_escape(&compact_bot_label(report, opponent)),
            html_escape(entry.source.label()),
            score_cell(pair_rate),
            html_escape(&pair_record_for_bot_standing_label(pair, &row.bot)),
        ));
        html.push_str("</div>");
    }
    html.push_str("</div>");
}

fn render_entrant_search_comparisons(
    html: &mut String,
    report: &TournamentReport,
    row: &StandingReport,
    pairs: &[PairwiseEntry<'_>],
) {
    html.push_str("<div class=\"entrant-search-comparisons\">");
    html.push_str(
        "<div class=\"comparison-head\"><span>Opponent</span><span>Source</span><span>Avg ms</span><span>Vs overall</span><span>Avg nodes</span><span>Vs overall</span></div>",
    );
    for entry in pairs {
        let pair = entry.pair;
        let opponent = opponent_for_pair(pair, &row.bot);
        let pair_stats = pair_search_stats_for_entry(report, *entry, &row.bot);
        let time_delta = pair_stats.avg_search_time_ms() - row.avg_search_time_ms;
        let nodes_delta = pair_stats.avg_nodes() - row.avg_nodes;
        html.push_str("<div class=\"comparison-row\">");
        html.push_str(&format!(
            "<span data-label=\"Opponent\">Vs {}</span><span data-label=\"Source\">{}</span><span data-label=\"Avg ms\">{}</span>{}<span data-label=\"Avg nodes\">{}</span>{}",
            html_escape(&compact_bot_label(report, opponent)),
            html_escape(entry.source.label()),
            html_escape(&ms_label(pair_stats.avg_search_time_ms())),
            delta_cell(
                &signed_ms_label(time_delta),
                cost_delta_class(time_delta, 0.05),
                "Vs overall"
            ),
            html_escape(&nodes_label(pair_stats.avg_nodes())),
            delta_cell(
                &signed_nodes_label(nodes_delta),
                cost_delta_class(nodes_delta, 0.5),
                "Vs overall"
            ),
        ));
        html.push_str("</div>");
    }
    html.push_str("</div>");
}

fn render_entrant_pairwise(
    html: &mut String,
    report: &TournamentReport,
    bot: &str,
    pairs: &[PairwiseEntry<'_>],
) {
    html.push_str("<div class=\"entrant-pairs\">");
    for entry in pairs {
        let pair = entry.pair;
        let opponent = opponent_for_pair(pair, bot);
        let matches = report
            .matches
            .iter()
            .filter(|report_match| same_pair(report_match, &pair.bot_a, &pair.bot_b))
            .collect::<Vec<_>>();
        let match_label = match entry.source {
            PairwiseSource::Current => format!("{} matches", matches.len()),
            PairwiseSource::Reference => format!("{} reference matches", pair.total),
        };

        html.push_str("<details class=\"opponent-row\"><summary>");
        render_bot_label_with_prefix(html, report, opponent, "Vs ");
        html.push_str(&format!(
            "<span data-label=\"Matches\">{}</span><span data-label=\"W-D-L\">{}</span><span data-label=\"Points\">{}</span></summary>",
            html_escape(&match_label),
            html_escape(&pair_record_for_bot_standing_label(pair, bot)),
            html_escape(&pair_score_for_bot_label(pair, bot)),
        ));
        html.push_str("<div class=\"match-list\">");
        for report_match in matches {
            render_match(html, report, pair, bot, report_match);
        }
        if entry.source == PairwiseSource::Reference {
            html.push_str(
                "<p class=\"reference-pair-note\">Reference anchor aggregate; per-match details live in the source anchor report.</p>",
            );
        }
        html.push_str("</div></details>");
    }
    html.push_str("</div>");
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PairwiseSource {
    Current,
    Reference,
}

impl PairwiseSource {
    fn label(self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::Reference => "reference",
        }
    }
}

#[derive(Clone, Copy)]
struct PairwiseEntry<'a> {
    pair: &'a PairwiseReport,
    source: PairwiseSource,
}

fn ranked_pairwise_entries_for_bot<'a>(
    report: &'a TournamentReport,
    bot: &str,
) -> Vec<PairwiseEntry<'a>> {
    let ranking = report
        .standings
        .iter()
        .enumerate()
        .map(|(index, row)| (row.bot.as_str(), index))
        .collect::<HashMap<_, _>>();
    let mut entries = report
        .pairwise
        .iter()
        .filter(|pair| pair.bot_a == bot || pair.bot_b == bot)
        .map(|pair| PairwiseEntry {
            pair,
            source: PairwiseSource::Current,
        })
        .collect::<Vec<_>>();

    if let Some(reference) = &report.reference_anchors {
        if reference.anchors.iter().any(|anchor| anchor.bot == bot) {
            entries.extend(
                reference
                    .pairwise
                    .iter()
                    .filter(|pair| pair.bot_a == bot || pair.bot_b == bot)
                    .map(|pair| PairwiseEntry {
                        pair,
                        source: PairwiseSource::Reference,
                    }),
            );
        }
    }

    entries.sort_by_key(|entry| {
        ranking
            .get(opponent_for_pair(entry.pair, bot))
            .copied()
            .unwrap_or(usize::MAX)
    });
    entries
}

fn opponent_for_pair<'a>(pair: &'a PairwiseReport, bot: &str) -> &'a str {
    if pair.bot_a == bot {
        &pair.bot_b
    } else {
        &pair.bot_a
    }
}

fn pair_record_for_bot_standing_label(pair: &PairwiseReport, bot: &str) -> String {
    if pair.bot_a == bot {
        format!("{}-{}-{} W-D-L", pair.wins_a, pair.draws, pair.wins_b)
    } else {
        format!("{}-{}-{} W-D-L", pair.wins_b, pair.draws, pair.wins_a)
    }
}

fn pair_score_for_bot_label(pair: &PairwiseReport, bot: &str) -> String {
    if pair.bot_a == bot {
        format!("{:.1}-{:.1} points", pair.score_a, pair.score_b)
    } else {
        format!("{:.1}-{:.1} points", pair.score_b, pair.score_a)
    }
}

fn pair_score_rate_for_bot(pair: &PairwiseReport, bot: &str) -> f64 {
    let score = if pair.bot_a == bot {
        pair.score_a
    } else {
        pair.score_b
    };
    avg(score * 100.0, pair.total)
}

fn best_pair_for_bot(
    report: &TournamentReport,
    bot: &str,
    pairs: &[PairwiseEntry<'_>],
) -> (String, Option<String>) {
    pairs
        .iter()
        .max_by(|a, b| {
            pair_score_rate_for_bot(a.pair, bot)
                .partial_cmp(&pair_score_rate_for_bot(b.pair, bot))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|entry| {
            (
                format!("{:.1}%", pair_score_rate_for_bot(entry.pair, bot)),
                Some(compact_bot_label(
                    report,
                    opponent_for_pair(entry.pair, bot),
                )),
            )
        })
        .unwrap_or_else(|| ("n/a".to_string(), None))
}

fn worst_pair_for_bot(
    report: &TournamentReport,
    bot: &str,
    pairs: &[PairwiseEntry<'_>],
) -> (String, Option<String>) {
    pairs
        .iter()
        .min_by(|a, b| {
            pair_score_rate_for_bot(a.pair, bot)
                .partial_cmp(&pair_score_rate_for_bot(b.pair, bot))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|entry| {
            (
                format!("{:.1}%", pair_score_rate_for_bot(entry.pair, bot)),
                Some(compact_bot_label(
                    report,
                    opponent_for_pair(entry.pair, bot),
                )),
            )
        })
        .unwrap_or_else(|| ("n/a".to_string(), None))
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

    fn avg_search_time_ms(&self) -> f64 {
        avg(self.total_time_ms as f64, self.search_move_count)
    }

    fn avg_nodes(&self) -> f64 {
        avg(self.total_nodes as f64, self.search_move_count)
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

    fn matches_pair(&self, pair: &PairwiseReport) -> bool {
        self.bot_a == pair.bot_a && self.bot_b == pair.bot_b
    }

    fn stats_for_bot(&self, bot: &str) -> PairSearchStats {
        if self.bot_a == bot {
            PairSearchStats {
                search_move_count: self.bot_a_search_move_count,
                total_time_ms: self.bot_a_total_time_ms,
                total_nodes: self.bot_a_total_nodes,
            }
        } else {
            PairSearchStats {
                search_move_count: self.bot_b_search_move_count,
                total_time_ms: self.bot_b_total_time_ms,
                total_nodes: self.bot_b_total_nodes,
            }
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

fn pair_search_stats_for_bot(
    report: &TournamentReport,
    pair: &PairwiseReport,
    bot: &str,
) -> PairSearchStats {
    pair_search_stats_for_matches(&report.matches, pair, bot)
}

fn pair_search_stats_for_entry(
    report: &TournamentReport,
    entry: PairwiseEntry<'_>,
    bot: &str,
) -> PairSearchStats {
    match entry.source {
        PairwiseSource::Current => pair_search_stats_for_bot(report, entry.pair, bot),
        PairwiseSource::Reference => report
            .reference_anchors
            .as_ref()
            .and_then(|reference| {
                reference
                    .pair_search
                    .iter()
                    .find(|search| search.matches_pair(entry.pair))
            })
            .map(|search| search.stats_for_bot(bot))
            .unwrap_or_default(),
    }
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

fn render_match(
    html: &mut String,
    report: &TournamentReport,
    pair: &PairwiseReport,
    bot: &str,
    report_match: &MatchReport,
) {
    html.push_str("<details class=\"match\"><summary>");
    let opponent = opponent_for_pair(pair, bot);
    html.push_str(&format!(
        "<span data-label=\"Side\">{}</span><span data-label=\"Result\">{}</span><span data-label=\"Moves\">{} moves</span><span data-label=\"End\">{}</span></summary>",
        html_escape(&match_color_label(report_match, bot, opponent)),
        html_escape(match_result_for_bot(report_match, bot)),
        report_match.move_count,
        html_escape(match_end_label(report_match)),
    ));
    html.push_str("<div class=\"match-grid\">");
    html.push_str(&format!(
        "<div class=\"board-panel\"><b>Finished board</b><pre class=\"board-ascii\">{}</pre></div>",
        html_escape(&finished_board_ascii(
            &report_match.move_cells,
            report.board_size
        ))
    ));
    html.push_str(&format!(
        "<p><b>Moves</b><br>{}</p>",
        html_escape(&move_notations(&report_match.move_cells, report.board_size).join(" "))
    ));
    html.push_str(&format!(
        "<details class=\"raw-data\"><summary>Raw data</summary><p><b>Opening</b><br>{}</p><p><b>Move cells</b><br>{}</p></details>",
        html_escape(&opening_label(report_match)),
        report_match
            .move_cells
            .iter()
            .map(usize::to_string)
            .collect::<Vec<_>>()
            .join(" ")
    ));
    html.push_str("</div></details>");
}

fn match_color_label(report_match: &MatchReport, bot: &str, opponent: &str) -> String {
    format!(
        "{} vs {}",
        side_code_for_bot(report_match, bot),
        side_code_for_bot(report_match, opponent)
    )
}

fn side_code_for_bot(report_match: &MatchReport, bot: &str) -> &'static str {
    if report_match.black == bot {
        "B"
    } else if report_match.white == bot {
        "W"
    } else {
        "?"
    }
}

fn match_result_for_bot(report_match: &MatchReport, bot: &str) -> &'static str {
    match report_match.winner.as_deref() {
        Some(winner) if winner == bot => "win",
        Some(_) => "lose",
        None => "draw",
    }
}

fn match_end_label(report_match: &MatchReport) -> &str {
    match report_match.end_reason.as_str() {
        "max_moves" => "max moves",
        "natural" => "finished",
        _ => report_match.end_reason.as_str(),
    }
}

fn opening_label(report_match: &MatchReport) -> String {
    let Some(opening) = &report_match.opening else {
        return "not captured".to_string();
    };

    let mut parts = vec![
        format!("{} #{}", opening.policy, opening.index),
        format!("{} plies", opening.ply_count),
    ];
    if let Some(suite_index) = opening.suite_index {
        parts.push(format!("suite {suite_index}"));
    }
    if let Some(template_index) = opening.template_index {
        parts.push(format!("template {template_index}"));
    }
    if let Some(transform_index) = opening.transform_index {
        parts.push(format!("transform {transform_index}"));
    }

    parts.join(", ")
}

fn phase_average_label(left_total: u64, right_total: u64, count: u32) -> String {
    if count == 0 || (left_total == 0 && right_total == 0) {
        return "n/a".to_string();
    }

    phase_average_label_zero_valid(left_total, right_total, count)
}

fn phase_average_label_zero_valid(left_total: u64, right_total: u64, count: u32) -> String {
    if count == 0 {
        return "n/a".to_string();
    }

    format!(
        "{} / {}",
        average_label(left_total, count),
        average_label(right_total, count)
    )
}

fn average_label(total: u64, count: u32) -> String {
    if count == 0 {
        return "n/a".to_string();
    }

    format!("{:.1}", avg(total as f64, count))
}

fn same_pair(report_match: &MatchReport, bot_a: &str, bot_b: &str) -> bool {
    (report_match.black == bot_a && report_match.white == bot_b)
        || (report_match.black == bot_b && report_match.white == bot_a)
}

fn run_chip(html: &mut String, label: &str, value: String) {
    html.push_str(&format!(
        "<div class=\"run-chip\"><span>{}</span><strong>{}</strong></div>",
        html_escape(label),
        html_escape(&value)
    ));
}

fn term_row(html: &mut String, title: &str, body: &str) {
    html.push_str(&format!(
        "<div class=\"term-row\"><dt>{}</dt><dd>{}</dd></div>",
        html_escape(title),
        html_escape(body),
    ));
}

fn variant_label(rules: &RuleConfig) -> String {
    match rules.variant {
        gomoku_core::Variant::Freestyle => "freestyle".to_string(),
        gomoku_core::Variant::Renju => "renju".to_string(),
    }
}

fn move_notations(move_cells: &[usize], board_size: usize) -> Vec<String> {
    move_cells
        .iter()
        .map(|cell| {
            let row = cell / board_size;
            let col = cell % board_size;
            Move { row, col }.to_notation()
        })
        .collect()
}

fn finished_board_ascii(move_cells: &[usize], board_size: usize) -> String {
    let mut cells = vec![None; board_size.saturating_mul(board_size)];
    for (idx, cell) in move_cells.iter().copied().enumerate() {
        if cell >= cells.len() {
            continue;
        }
        cells[cell] = Some(if idx % 2 == 0 {
            Color::Black
        } else {
            Color::White
        });
    }

    let mut output = String::new();
    output.push_str("   ");
    for col in 0..board_size {
        if col > 0 {
            output.push(' ');
        }
        output.push(column_label(col));
    }
    output.push('\n');

    for row in 0..board_size {
        output.push_str(&format!("{:2} ", row + 1));
        for col in 0..board_size {
            if col > 0 {
                output.push(' ');
            }
            let cell = row * board_size + col;
            output.push(cells[cell].map_or('.', Color::to_char));
        }
        if row + 1 < board_size {
            output.push('\n');
        }
    }

    output
}

fn column_label(col: usize) -> char {
    char::from(b'A'.saturating_add(col as u8))
}

fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

const STYLE: &str = r#"
<style>
:root{color-scheme:dark;--bg:#1e1e1e;--surface:#2a2a2a;--surface-strong:#333333;--card:#232323;--border:#575756;--text:#f5f5f5;--text-muted:#a6a6a0;--accent:#fccb57;--green:#5ad17a;--teal:#5fc7c2}
*{box-sizing:border-box}body{margin:0;background:var(--bg);color:var(--text);font:16px/1.4 ui-monospace,SFMono-Regular,Menlo,Consolas,monospace}
main{display:grid;gap:24px;margin:0 auto;max-width:1180px;padding:32px}h1,h2,p{margin:0}a{color:inherit;text-decoration:none}code{color:var(--accent)}
.hero,section,.run-warning,.diagnostic-panel{background:var(--surface);border:2px solid var(--border);display:grid;gap:16px;padding:20px;overflow:auto}.run-warning{border-color:var(--accent);color:var(--accent)}.diagnostic-panel summary{align-items:center;cursor:pointer;display:flex;gap:16px;justify-content:space-between;list-style:none}.diagnostic-panel summary::-webkit-details-marker{display:none}.diagnostic-panel summary span{color:var(--text-muted);font-size:12px;letter-spacing:.1em;text-transform:uppercase}.top-links{display:flex;flex-wrap:wrap;gap:8px}.top-links a{background:var(--surface-strong);border:2px solid var(--border);color:var(--text);display:inline-block;padding:8px 12px;text-transform:uppercase}.top-links a:hover,.top-links a:focus{border-color:var(--teal);outline:none}
.eyebrow{color:var(--accent);font-size:12px;letter-spacing:.16em;text-transform:uppercase}h1{font-size:clamp(34px,7vw,64px);line-height:1}.match summary span,.match-grid,.note{color:var(--text-muted)}
.run-strip{display:flex;flex-wrap:wrap;gap:8px;padding:0}.run-chip{background:var(--card);border:1px solid var(--border);display:inline-flex;gap:8px;align-items:baseline;min-width:0;padding:7px 10px}.run-chip span{color:var(--text-muted);font-size:11px;letter-spacing:.1em;text-transform:uppercase}.run-chip strong{color:var(--green);font-size:14px;line-height:1.2}.entrant-row,.opponent-row,.match{background:var(--card);border:1px solid var(--border);display:grid;gap:10px;padding:16px}.entrant-row:hover,.opponent-row:hover,.match:hover{border-color:var(--teal)}
.health-grid{display:grid;gap:12px;grid-template-columns:repeat(auto-fit,minmax(180px,1fr))}.health-card{background:var(--surface-strong);border:1px solid var(--border);display:grid;gap:4px;padding:12px}.health-card span{color:var(--text-muted);font-size:11px;letter-spacing:.1em;text-transform:uppercase}.health-card strong{color:var(--green);font-size:18px;line-height:1.15}.health-card em{color:var(--text-muted);font-style:normal}.rolling-comparison-list{display:grid;gap:8px;margin-top:12px}.rolling-comparison-list p{background:var(--surface-strong);border:1px solid var(--border);display:grid;gap:8px;grid-template-columns:minmax(180px,1fr) minmax(130px,max-content) minmax(130px,max-content);margin:0;padding:10px 12px}.rolling-comparison-list b{color:var(--text);overflow-wrap:anywhere}.rolling-comparison-list span{color:var(--text-muted);font-variant-numeric:tabular-nums;text-align:right}
.entrant-row.role-candidate{border-left:4px solid var(--green)}.entrant-row.role-anchor{border-left:4px solid var(--border);opacity:.86}.role-badge{border:1px solid var(--border);display:inline-block!important;font-size:10px!important;letter-spacing:.1em!important;line-height:1;margin-top:6px!important;padding:4px 6px;text-transform:uppercase;width:max-content}.role-badge-candidate{border-color:rgba(90,209,122,.5);color:var(--green)!important}.role-badge-anchor{color:var(--text-muted)!important}
.term-list{display:grid;gap:0;margin:0}.term-row{border-top:1px solid var(--border);display:grid;gap:18px;grid-template-columns:minmax(140px,.28fr) 1fr;padding:10px 0}.term-row:first-child{border-top:0;padding-top:0}.term-row:last-child{padding-bottom:0}.term-row dt{color:var(--green);font-size:13px;letter-spacing:.08em;text-transform:uppercase}.term-row dd{color:var(--text-muted);margin:0;max-width:86ch}.term-row code{color:var(--accent)}
.section-heading{display:grid}.section-heading h2{color:var(--accent);font-size:1.2rem}
table{border-collapse:collapse;min-width:820px;width:100%}th,td{border-bottom:1px solid var(--border);padding:9px 10px;text-align:right;white-space:nowrap}th:first-child,td:first-child{text-align:left}th{color:var(--text-muted);font-size:12px;letter-spacing:.08em;text-transform:uppercase}
.view-toggle{display:flex;flex-wrap:wrap;gap:8px}.report-view-radio{height:1px;opacity:0;position:absolute;width:1px}.view-toggle label{background:var(--surface-strong);border:1px solid var(--border);cursor:pointer;padding:8px 12px;text-transform:uppercase}.view-toggle label:hover{border-color:var(--teal)}.entrant-workbench:has(#view-results:checked) label[for=view-results],.entrant-workbench:has(#view-search:checked) label[for=view-search],.entrant-workbench:has(#view-pairwise:checked) label[for=view-pairwise]{border-color:var(--accent);color:var(--accent)}
.entrant-grid,.match-list{display:grid;gap:12px}.entrant-head,.entrant-row summary{display:grid;gap:10px;align-items:center}.entrant-workbench:has(#view-results:checked) .entrant-head,.entrant-workbench:has(#view-results:checked) .entrant-row summary{grid-template-columns:minmax(260px,1.6fr) repeat(8,minmax(82px,1fr))}.entrant-workbench:has(#view-search:checked) .entrant-head,.entrant-workbench:has(#view-search:checked) .entrant-row summary{grid-template-columns:minmax(240px,1.4fr) repeat(8,minmax(84px,1fr))}.entrant-workbench:has(#view-pairwise:checked) .entrant-head,.entrant-workbench:has(#view-pairwise:checked) .entrant-row summary{grid-template-columns:minmax(280px,1.6fr) repeat(3,minmax(120px,1fr))}.entrant-head{color:var(--text-muted);font-size:12px;letter-spacing:.08em;padding:0 14px;text-transform:uppercase}.entrant-row,.opponent-row,.match{padding:0}.entrant-row summary,.opponent-row summary,.match summary{cursor:pointer;padding:12px 14px}.entrant-row summary>*,.opponent-row summary>*{min-width:0}.entrant-row summary .bot-label,.opponent-row summary strong,.match summary strong{color:var(--text);overflow-wrap:anywhere}.bot-label span,.metric span{display:block}.metric-nowrap,.metric-nowrap span{white-space:nowrap}.entrant-row summary .bot-label span:first-child{color:var(--text)}.entrant-row summary .bot-label span+span,.metric span+span{color:var(--text-muted);font-size:11px;letter-spacing:.08em;margin-top:2px}.entrant-row summary span,.opponent-row summary span,.match summary span{color:var(--text-muted)}.metric-search,.metric-pairwise{display:none}.entrant-workbench:has(#view-search:checked) .metric-results,.entrant-workbench:has(#view-search:checked) .metric-pairwise,.entrant-workbench:has(#view-pairwise:checked) .metric-results,.entrant-workbench:has(#view-pairwise:checked) .metric-search{display:none}.entrant-workbench:has(#view-search:checked) .metric-search,.entrant-workbench:has(#view-pairwise:checked) .metric-pairwise{display:block}.entrant-head .metric,.entrant-row summary .metric{border-left:1px solid var(--border);font-variant-numeric:tabular-nums;line-height:1.22;padding-left:10px;text-align:right}.entrant-result-comparisons,.entrant-search-comparisons,.entrant-pairs{display:none;gap:10px;padding:8px 18px 18px}.entrant-workbench:has(#view-results:checked) .entrant-result-comparisons,.entrant-workbench:has(#view-search:checked) .entrant-search-comparisons,.entrant-workbench:has(#view-pairwise:checked) .entrant-pairs{display:grid}.comparison-head,.comparison-row{align-items:center;display:grid;gap:12px}.entrant-result-comparisons .comparison-head,.entrant-result-comparisons .comparison-row{grid-template-columns:minmax(180px,1fr) minmax(78px,96px) minmax(92px,120px) minmax(120px,140px)}.entrant-search-comparisons .comparison-head,.entrant-search-comparisons .comparison-row{grid-template-columns:minmax(180px,1fr) minmax(78px,96px) minmax(86px,120px) minmax(96px,120px) minmax(104px,130px) minmax(96px,120px)}.comparison-head{border:1px solid transparent;color:var(--text-muted);font-size:11px;letter-spacing:.08em;padding:0 12px;text-transform:uppercase}.comparison-row{background:var(--surface-strong);border:1px solid var(--border);padding:10px 12px}.comparison-row span:not(:first-child),.comparison-head span:not(:first-child){border-left:1px solid var(--border);font-variant-numeric:tabular-nums;padding-left:12px;text-align:right}.delta-good,.score-good{color:var(--green)!important}.delta-bad,.score-bad{color:#e78f85!important}.delta-neutral{color:var(--text-muted)!important}.opponent-row summary{display:grid;gap:12px;grid-template-columns:minmax(0,1fr) repeat(3,max-content);align-items:center}.opponent-row summary span{border-left:1px solid var(--border);font-variant-numeric:tabular-nums;padding-left:12px;text-align:right}.match summary{display:grid;gap:12px;grid-template-columns:repeat(4,minmax(82px,max-content));align-items:center}.pair-overview{display:grid;gap:12px;grid-template-columns:repeat(auto-fit,minmax(260px,1fr));padding:0 18px 16px}.pair-overview p{background:var(--surface-strong);border:1px solid var(--border);margin:0;padding:12px}.match-grid{display:grid;gap:12px;grid-template-columns:1fr;padding:0 14px 14px}.match-grid p{margin:0;word-break:break-word}.reference-pair-note{background:var(--surface-strong);border:1px solid var(--border);color:var(--text-muted);margin:0;padding:10px 12px}.pair-overview b,.match-grid b{color:var(--text)}.board-panel,.raw-data{grid-column:1/-1}.board-ascii,.raw-data{background:var(--surface-strong);border:1px solid var(--border)}.board-ascii{color:var(--text);font:14px/1.35 ui-monospace,SFMono-Regular,Menlo,Consolas,monospace;margin:8px 0 0;overflow:auto;padding:12px;white-space:pre}.raw-data{padding:10px}.raw-data summary{cursor:pointer;padding:0}.raw-data p{margin:8px 0 0}
.provenance dl{display:grid;gap:8px 18px;grid-template-columns:max-content 1fr;margin:0}.provenance dt{color:var(--text-muted);font-size:12px;letter-spacing:.08em;text-transform:uppercase}.provenance dd{margin:0}.command{background:var(--surface-strong);border:1px solid var(--border);margin:0;overflow:auto;padding:12px}
@media (max-width:760px){main{padding:16px}.hero,section,.run-warning{padding:16px}.run-chip{justify-content:space-between;width:100%}.rolling-comparison-list p{grid-template-columns:1fr}.rolling-comparison-list span{text-align:left}.entrant-head,.comparison-head{display:none}.entrant-grid,.entrant-row,.opponent-row,.match,.entrant-result-comparisons,.entrant-search-comparisons,.entrant-pairs,.comparison-row{min-width:0;width:100%}.entrant-row summary,.entrant-workbench:has(#view-results:checked) .entrant-row summary,.entrant-workbench:has(#view-search:checked) .entrant-row summary,.entrant-workbench:has(#view-pairwise:checked) .entrant-row summary{gap:8px 12px;grid-template-columns:repeat(2,minmax(0,1fr))}.entrant-row summary .bot-label{grid-column:1/-1}.entrant-row summary .metric{border-left:0;display:grid;gap:2px 10px;grid-template-columns:minmax(0,1fr) auto;padding:8px 0 0;text-align:right}.entrant-row summary .metric::before,.comparison-row span::before,.opponent-row summary span::before,.match summary span::before{color:var(--text-muted);content:attr(data-label);font-size:11px;letter-spacing:.08em;text-align:left;text-transform:uppercase}.entrant-row summary .metric::before{align-self:start;grid-column:1;grid-row:1/span 2}.entrant-row summary .metric span{grid-column:2}.entrant-row summary .metric-search,.entrant-row summary .metric-pairwise,.entrant-workbench:has(#view-search:checked) .entrant-row summary .metric-results,.entrant-workbench:has(#view-search:checked) .entrant-row summary .metric-pairwise,.entrant-workbench:has(#view-pairwise:checked) .entrant-row summary .metric-results,.entrant-workbench:has(#view-pairwise:checked) .entrant-row summary .metric-search{display:none}.entrant-workbench:has(#view-search:checked) .entrant-row summary .metric-search,.entrant-workbench:has(#view-pairwise:checked) .entrant-row summary .metric-pairwise{display:grid}.opponent-row summary,.match summary,.entrant-result-comparisons .comparison-row,.entrant-search-comparisons .comparison-row{grid-template-columns:1fr}.comparison-row span,.opponent-row summary span,.match summary span{border-left:0!important;display:flex;gap:12px;justify-content:space-between;min-width:0;overflow-wrap:anywhere;padding-left:0!important;text-align:right}.comparison-row span:not(:first-child),.opponent-row summary span,.match summary span{border-top:1px solid var(--border);padding-top:8px}.opponent-row summary span:first-of-type,.match summary span:first-child{border-top:0;padding-top:0}.entrant-result-comparisons,.entrant-search-comparisons,.entrant-pairs{padding:4px 12px 14px}.match-grid,.term-row{grid-template-columns:1fr}.term-row{gap:4px}.board-ascii{font-size:12px}.provenance dl{grid-template-columns:1fr}table{min-width:760px}}
@media (max-width:420px){.entrant-row summary,.entrant-workbench:has(#view-results:checked) .entrant-row summary,.entrant-workbench:has(#view-search:checked) .entrant-row summary,.entrant-workbench:has(#view-pairwise:checked) .entrant-row summary{grid-template-columns:1fr}}
</style>
"#;

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
    fn html_escape_handles_special_chars() {
        assert_eq!(html_escape("<bot & 'x'>"), "&lt;bot &amp; &#39;x&#39;&gt;");
    }

    #[test]
    fn finished_board_ascii_matches_cli_shape() {
        let output = finished_board_ascii(&[0, 4, 12], 5);

        assert_eq!(
            output,
            "   A B C D E\n 1 B . . . W\n 2 . . . . .\n 3 . . B . .\n 4 . . . . .\n 5 . . . . ."
        );
    }

    #[test]
    fn html_report_uses_compact_run_strip_before_entrant_workbench() {
        let mut report = sample_report();
        report.standings = vec![
            sample_standing_with_search_costs("search-d7+tactical-cap-8+pattern-eval"),
            sample_standing_with_search_costs("fast"),
            sample_standing_with_search_costs("balanced"),
        ];

        let html = render_tournament_report_html_with_options(
            &report,
            &ReportRenderOptions {
                raw_json_href: Some("latest.json".to_string()),
                include_rolling_health: false,
            },
        );

        assert!(html.contains("<div class=\"run-strip\" aria-label=\"Run summary\">"));
        assert!(html.contains("<nav class=\"top-links\"><a href=\"/\">Game</a><a href=\"/assets/\">Assets</a><a href=\"/analysis-report/\">Analysis</a></nav>"));
        assert!(!html.contains("<section class=\"run-strip\" aria-label=\"Run summary\">"));
        assert!(!html.contains("<nav class=\"top-links\"><a href=\"/\">Game</a><a href=\"/assets/\">Assets</a><a href=\"latest.json\">Raw JSON</a></nav>"));
        assert!(html.contains("<dt>Raw JSON</dt><dd><a href=\"latest.json\">latest.json</a></dd>"));
        assert!(html.contains("<div class=\"run-chip\"><span>Schedule</span>"));
        assert!(!html.contains(".run-chip:hover"));
        assert!(!html.contains("A bot evaluation report for comparing specs"));
        assert!(!html.contains("<span>Leader</span>"));
        assert!(!html.contains("<span>Entrants</span>"));
        assert!(!html.contains("SearchBot @ depth 7 + tactical cap 8 + pattern eval"));
        assert!(html.contains("<h2>Results</h2>"));
        assert!(!html.contains("<h2>Entrants</h2>"));
        assert!(!html.contains("<h2>Standings</h2>"));
        assert!(!html.contains("<h2>Search Cost</h2>"));
        assert!(!html.contains("<h2>Matches By Pair</h2>"));

        let hero_pos = html.find("<header class=\"hero\">").unwrap();
        let run_strip_pos = html.find("class=\"run-strip\"").unwrap();
        let header_close_pos = html.find("</header>").unwrap();
        let entrants_pos = html.find("<h2>Results</h2>").unwrap();
        assert!(hero_pos < run_strip_pos);
        assert!(run_strip_pos < header_close_pos);
        assert!(run_strip_pos < entrants_pos);
    }

    #[test]
    fn html_report_labels_pooled_search_budget() {
        let mut report = sample_report();
        report.run.search_budget_mode = "pooled".to_string();
        report.run.search_cpu_reserve_ms = Some(4_000);

        let html = render_tournament_report_html(&report);

        assert!(html.contains("CPU 1000 ms/move, pooled reserve 4000 ms"));
    }

    #[test]
    fn html_report_renders_how_to_read_as_quiet_glossary() {
        let report = sample_report();
        let html = render_tournament_report_html(&report);

        assert!(html.contains("<section class=\"how-to-read\">"));
        assert!(html.contains("<dl class=\"term-list\">"));
        assert!(html.contains("<div class=\"term-row\"><dt>Run Shape</dt><dd>"));
        assert!(html.contains("<div class=\"term-row\"><dt>Search Cost</dt><dd>"));
        assert!(!html.contains("<div class=\"term-grid\">"));
        assert!(!html.contains("<article class=\"term\">"));
    }

    #[test]
    fn html_report_surfaces_rolling_frontier_health() {
        let mut report = sample_report();
        let mut scan = sample_standing_with_search_costs("search-d7+tactical-cap-8");
        scan.avg_search_time_ms = 100.0;
        scan.avg_nodes = 2000.0;
        scan.threat_view_scan_queries = 50;
        scan.threat_view_scan_ns = 2_000_000;

        let mut rolling =
            sample_standing_with_search_costs("search-d7+tactical-cap-8+rolling-frontier");
        rolling.avg_search_time_ms = 75.0;
        rolling.avg_nodes = 1800.0;
        rolling.threat_view_frontier_rebuilds = 20;
        rolling.threat_view_frontier_rebuild_ns = 900_000;
        rolling.threat_view_frontier_queries = 40;
        rolling.threat_view_frontier_query_ns = 900_000;
        rolling.threat_view_frontier_immediate_win_queries = 6;
        rolling.threat_view_frontier_immediate_win_query_ns = 30_000;
        rolling.threat_view_frontier_delta_captures = 20;
        rolling.threat_view_frontier_delta_capture_ns = 100_000;
        rolling.threat_view_frontier_annotation_dirty_marks = 20;
        rolling.threat_view_frontier_annotation_dirty_mark_ns = 150_000;
        rolling.threat_view_frontier_clean_annotation_queries = 15;
        rolling.threat_view_frontier_dirty_annotation_queries = 8;
        rolling.threat_view_frontier_memo_annotation_queries = 7;
        rolling.threat_view_frontier_fallback_annotation_queries = 0;

        report.run.bots = vec![scan.bot.clone(), rolling.bot.clone()];
        report.standings = vec![scan, rolling];
        let html = render_tournament_report_html(&report);
        assert!(!html.contains("<details class=\"threat-health diagnostic-panel\">"));

        let html = render_tournament_report_html_with_options(
            &report,
            &ReportRenderOptions {
                raw_json_href: None,
                include_rolling_health: true,
            },
        );

        assert!(html.contains("<details class=\"threat-health diagnostic-panel\">"));
        assert!(html.contains("<summary><h2>Rolling Health</h2><span>Diagnostics</span></summary>"));
        assert!(html.contains("0 mismatches / 0 checks"));
        assert!(html.contains("<article class=\"health-card\"><span>Scan</span>"));
        assert!(html.contains("<article class=\"health-card\"><span>Frontier</span>"));
        assert!(html.contains("40 queries / 6 wins / 900.0 us update"));
        assert!(html.contains("<article class=\"health-card\"><span>Annotation</span>"));
        assert!(html.contains("<b>SearchBot_D7+TCap8+Rolling vs scan</b>"));
        assert!(html.contains("75.0 ms vs 100.0 ms (-25.0%)"));
        assert!(html.contains("1.8k nodes vs 2.0k nodes"));
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
    fn html_report_omits_hero_and_section_subtitles() {
        let mut report = sample_report();
        report.reference_anchors = Some(AnchorReferenceReport {
            source: AnchorReferenceSource {
                path: Some("reports/latest.json".to_string()),
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
                max_moves: Some(120),
                max_game_ms: None,
            },
            anchors: vec![],
            pairwise: vec![],
            pair_search: vec![],
        });
        let html = render_tournament_report_html(&report);

        assert!(!html.contains("class=\"lede\""));
        assert!(!html.contains("<div class=\"section-heading\"><h2>Reference Anchors</h2><p>"));
        assert!(!html.contains("<div class=\"section-heading\"><h2>Results</h2><p>"));
        assert!(!html.contains("<div class=\"section-heading\"><h2>Entrants</h2>"));
        assert!(!html.contains("<div class=\"section-heading\"><h2>How To Read This</h2><p>"));
        assert!(!html.contains("<div class=\"section-heading\"><h2>Provenance</h2><p>"));
    }

    #[test]
    fn html_report_drills_from_entrant_to_opponent_matches() {
        let mut report = sample_report();
        report.standings = vec![
            sample_standing_with_search_costs("fast"),
            sample_standing_with_search_costs("balanced"),
        ];
        let html = render_tournament_report_html(&report);

        assert!(html.contains("<section class=\"entrant-workbench\">"));
        assert!(html.contains("<details class=\"entrant-row\">"));
        assert!(html.contains("<details class=\"opponent-row\">"));
        assert!(html.contains("<strong class=\"bot-label\"><span>Vs SearchBot_D3</span></strong>"));
        assert!(!html.contains("SearchBot_D2 vs SearchBot_D3"));
        assert!(html.contains("2 matches"));
        assert!(html.contains("0-0-2 W-D-L"));
        assert!(html.contains("0.0-2.0 points"));
        assert!(!html.contains("<div class=\"pair-overview\">"));
        assert!(!html.contains("<b>Pair result</b>"));
        assert!(!html.contains("<b>Color result</b>"));
        assert!(!html.contains("A-D-B"));
        assert!(html.contains(
            "<summary><span data-label=\"Side\">B vs W</span><span data-label=\"Result\">lose</span><span data-label=\"Moves\">5 moves</span><span data-label=\"End\">finished</span></summary>"
        ));
        assert!(html.contains(
            "<summary><span data-label=\"Side\">W vs B</span><span data-label=\"Result\">lose</span><span data-label=\"Moves\">5 moves</span><span data-label=\"End\">finished</span></summary>"
        ));
        assert!(!html.contains("#001</span>"));
        assert!(!html.contains("#002</span>"));
        assert!(!html.contains(
            "#001</span><strong class=\"bot-label\"><span>SearchBot_D2</span><span>vs SearchBot_D3</span></strong>"
        ));
        assert!(!html.contains("<b>SearchBot_D2 (W) stats</b>"));
        assert!(!html.contains("<b>SearchBot_D3 (B) stats</b>"));
        assert!(html.contains("<label for=\"view-pairwise\">Pairwise</label>"));
        assert!(!html.contains("<h2>Pairwise</h2>"));
        assert!(!html.contains("<h2>Color Splits</h2>"));
        assert!(html.contains("<details class=\"raw-data\">"));

        let entrant_pos = html.find("<details class=\"entrant-row\">").unwrap();
        let pair_pos = html.find("Vs SearchBot_D3").unwrap();
        let match_pos = html
            .find("<summary><span data-label=\"Side\">B vs W</span>")
            .unwrap();
        let how_to_read_pos = html.find("<h2>How To Read This</h2>").unwrap();
        let provenance_pos = html.find("<h2>Provenance</h2>").unwrap();
        let match_body = &html[match_pos..];
        let board_pos = match_pos + match_body.find("Finished board").unwrap();
        let moves_pos = match_pos + match_body.find("<b>Moves</b>").unwrap();
        let opening_pos = match_pos + match_body.find("<b>Opening</b>").unwrap();
        let raw_pos = match_pos + match_body.find("Raw data").unwrap();
        assert!(entrant_pos < pair_pos);
        assert!(pair_pos < match_pos);
        assert!(match_pos < board_pos);
        assert!(board_pos < moves_pos);
        assert!(moves_pos < raw_pos);
        assert!(raw_pos < opening_pos);
        assert!(raw_pos < how_to_read_pos);
        assert!(how_to_read_pos < provenance_pos);
    }

    #[test]
    fn html_report_formats_entrant_rows_for_scan_and_drilldown_modes() {
        let mut report = sample_report();
        report.standings = vec![
            sample_standing_with_search_costs("search-d7+tactical-cap-8+pattern-eval"),
            sample_standing_with_search_costs("fast"),
            sample_standing_with_search_costs("balanced"),
        ];

        let html = render_tournament_report_html(&report);

        assert!(html.contains(
            "<strong class=\"bot-label\"><span>SearchBot_D7</span><span>TCap8 + Pattern</span></strong>"
        ));
        assert!(html.contains(
            "<span class=\"metric metric-results\" data-label=\"Shuffled Elo\"><span>1000.0</span><span>+/- 0.0</span></span>"
        ));
        assert!(html.contains("<span class=\"metric metric-results\">Rank</span>"));
        assert!(html.contains("<span class=\"metric metric-results metric-nowrap\">W-D-L</span>"));
        assert!(html.contains(
            "<span class=\"metric metric-results metric-nowrap\" data-label=\"W-D-L\"><span>1-0-1</span></span>"
        ));
        assert!(html.contains("<span class=\"metric metric-results\">Depth</span>"));
        assert!(html.contains("<span class=\"metric metric-results\">Width</span>"));
        assert!(html.contains("<span class=\"metric metric-results\">Avg ms</span>"));
        assert!(html.contains("<span class=\"metric metric-results\">Budget exhausted</span>"));
        assert!(!html.contains("<span class=\"metric metric-results\">Avg depth</span>"));
        assert!(!html.contains("<span class=\"metric metric-results\">Breadth</span>"));
        assert!(!html.contains("<span class=\"metric metric-results\">Budget hit</span>"));
        assert!(!html.contains("<span class=\"metric metric-results\">Best</span>"));
        assert!(!html.contains("<span class=\"metric metric-results\">Worst</span>"));
        assert!(html.contains(
            "<span class=\"metric metric-results\" data-label=\"Width\"><span>8.0</span><span>pre 12.0</span></span>"
        ));
        assert!(html.contains("id=\"view-pairwise\""));
        assert!(!html.contains("event.preventDefault()"));
        assert!(!html.contains("removeAttribute('open')"));
        assert!(!html.contains("<span class=\"metric metric-pairwise\">Open row</span>"));
        assert!(html.contains(
            "<span class=\"metric metric-pairwise\" data-label=\"Worst\"><span>0.0%</span><span>SearchBot_D3</span>"
        ));
        assert!(html.contains(
            ".entrant-result-comparisons,.entrant-search-comparisons,.entrant-pairs{display:none"
        ));
        assert!(html.contains(
            ".entrant-workbench:has(#view-pairwise:checked) .entrant-pairs{display:grid"
        ));
        assert!(html.contains(".comparison-head{border:1px solid transparent"));
        assert!(html.contains(".entrant-row summary .metric::before"));
        assert!(html.contains("content:attr(data-label)"));
        assert!(html.contains(
            ".entrant-row summary,.entrant-workbench:has(#view-results:checked) .entrant-row summary"
        ));
        assert!(html.contains(
            ".entrant-result-comparisons .comparison-head,.entrant-result-comparisons .comparison-row{grid-template-columns:minmax(180px,1fr) minmax(78px,96px) minmax(92px,120px) minmax(120px,140px)}"
        ));
        assert!(html.contains(
            ".entrant-search-comparisons .comparison-head,.entrant-search-comparisons .comparison-row{grid-template-columns:minmax(180px,1fr) minmax(78px,96px) minmax(86px,120px) minmax(96px,120px) minmax(104px,130px) minmax(96px,120px)}"
        ));
        assert!(!html.contains(
            ".entrant-result-comparisons .comparison-head,.entrant-result-comparisons .comparison-row{grid-template-columns:minmax(180px,1fr) repeat(3,minmax(90px,max-content))}"
        ));
        assert!(!html.contains(
            ".entrant-search-comparisons .comparison-head,.entrant-search-comparisons .comparison-row{grid-template-columns:minmax(180px,1fr) repeat(4,minmax(90px,max-content))}"
        ));

        assert!(html.contains("<span class=\"metric metric-search\">Nodes</span>"));
        assert!(html.contains("<span class=\"metric metric-search\">Move gen</span>"));
        assert!(html.contains("<span class=\"metric metric-search\">Ordering</span>"));
        assert!(html.contains("<span class=\"metric metric-search\">Scoring</span>"));
        assert!(html.contains("<span class=\"metric metric-search\">Threat detection</span>"));
        assert!(html.contains("<span class=\"metric metric-search\">Proof</span>"));
        assert!(html.contains("<span class=\"metric metric-search\">Other</span>"));
        assert!(html.contains("<span class=\"metric metric-search\">TT</span>"));
        assert!(!html.contains("<span class=\"metric metric-search\">Avg nodes</span>"));
        assert!(!html.contains("<span class=\"metric metric-search\">Eval cost</span>"));
        assert!(!html.contains("<span class=\"metric metric-search\">Cand gen</span>"));
        assert!(!html.contains("<span class=\"metric metric-search\">Breadth</span>"));
        assert!(!html.contains("<span class=\"metric metric-search\">Legal</span>"));
        assert!(!html.contains("<span class=\"metric metric-search\">Portal</span>"));
        assert!(!html.contains("<span class=\"metric metric-search\">Exit</span>"));
        assert!(!html.contains("<span class=\"metric metric-search\">TT hit/cut</span>"));
        assert!(!html.contains("<span class=\"metric metric-search\">Child width</span>"));
        assert!(!html.contains("<span class=\"metric metric-search\">Cand width r/s</span>"));
        assert!(!html.contains("<span class=\"metric metric-search\">Tactical ann r/s</span>"));
    }

    #[test]
    fn html_report_expands_results_to_ranked_opponent_comparisons() {
        let mut report = sample_report();
        report.standings = vec![
            sample_standing_with_search_costs("balanced"),
            sample_standing_with_search_costs("fast"),
        ];

        let html = render_tournament_report_html(&report);

        assert!(html.contains("<div class=\"entrant-result-comparisons\">"));
        assert!(html.contains("<div class=\"comparison-head\"><span>Opponent</span><span>Source</span><span>Score</span><span>Record</span></div>"));
        assert!(html.contains("<span data-label=\"Opponent\">Vs SearchBot_D3</span><span data-label=\"Source\">current</span><span class=\"score score-bad\" data-label=\"Score\">0.0%</span><span data-label=\"Record\">0-0-2 W-D-L</span>"));
        assert!(html.contains("<span data-label=\"Opponent\">Vs SearchBot_D2</span><span data-label=\"Source\">current</span><span class=\"score score-good\" data-label=\"Score\">100.0%</span><span data-label=\"Record\">2-0-0 W-D-L</span>"));
        assert!(!html.contains("<span>Vs overall</span><span>Record</span>"));
    }

    #[test]
    fn html_report_expands_search_to_ranked_opponent_cost_comparisons() {
        let mut report = sample_report();
        report.standings = vec![
            sample_standing_with_search_costs("balanced"),
            sample_standing_with_search_costs("fast"),
        ];
        for report_match in &mut report.matches {
            report_match.black_stats = sample_side_stats_with_search_costs();
            report_match.white_stats = sample_side_stats_with_search_costs();
        }

        let html = render_tournament_report_html(&report);

        assert!(html.contains("<div class=\"entrant-search-comparisons\">"));
        assert!(html.contains("<span data-label=\"Opponent\">Vs SearchBot_D3</span><span data-label=\"Source\">current</span><span data-label=\"Avg ms\">10.0 ms</span><span class=\"delta delta-neutral\" data-label=\"Vs overall\">+0.0 ms</span><span data-label=\"Avg nodes\">200 nodes</span><span class=\"delta delta-neutral\" data-label=\"Vs overall\">+0 nodes</span>"));
        assert!(html.contains("<span data-label=\"Opponent\">Vs SearchBot_D2</span><span data-label=\"Source\">current</span><span data-label=\"Avg ms\">10.0 ms</span><span class=\"delta delta-neutral\" data-label=\"Vs overall\">+0.0 ms</span><span data-label=\"Avg nodes\">200 nodes</span><span class=\"delta delta-neutral\" data-label=\"Vs overall\">+0 nodes</span>"));
    }

    #[test]
    fn html_report_surfaces_match_opening_metadata() {
        let mut report = sample_report();
        report.standings = vec![
            sample_standing_with_search_costs("fast"),
            sample_standing_with_search_costs("balanced"),
        ];
        let html = render_tournament_report_html(&report);

        assert!(html.contains("<b>Opening</b>"));
        assert!(html.contains("centered-suite #0, 4 plies, suite 3, template 0, transform 3"));
    }

    #[test]
    fn html_report_surfaces_search_cost_metrics() {
        let mut report = sample_report();
        let mut zero_tt_standing = sample_standing_with_search_costs("search-d2");
        zero_tt_standing.tt_hits = 0;
        zero_tt_standing.tt_cutoffs = 0;
        report.run.bots = vec!["search-d2".to_string()];
        report.standings = vec![zero_tt_standing];
        report.matches[0].black_stats = sample_side_stats_with_search_costs();

        let html = render_tournament_report_html(&report);

        assert!(html.contains("<h2>Results</h2>"));
        assert!(html.contains("<label for=\"view-results\">Ranking</label>"));
        assert!(!html.contains("<label for=\"view-results\">Results</label>"));
        assert!(html.contains("<label for=\"view-search\">Search</label>"));
        assert!(html.contains("<label for=\"view-pairwise\">Pairwise</label>"));
        assert!(!html.contains("<h2>Search Cost</h2>"));
        assert!(html.contains("SearchBot_D2"));
        assert!(html.contains("Nodes"));
        assert!(html.contains("Move gen"));
        assert!(html.contains("Ordering"));
        assert!(html.contains("Threat detection"));
        assert!(html.contains("Proof"));
        assert!(html.contains("Other"));
        assert!(!html.contains("Eval cost"));
        assert!(!html.contains("Pattern scan"));
        assert!(html.contains("200"));
        assert!(html.contains("Avg ms"));
        assert!(html.contains("10.0"));
        assert!(html.contains("Depth"));
        assert!(html.contains("3.00"));
        assert!(html.contains("Width"));
        assert!(html.contains("8.0"));
        assert!(html.contains("pre 12.0"));
        assert!(!html.contains("Child width"));
        assert!(html.contains(
            "<span class=\"metric metric-search\" data-label=\"Move gen\"><span>10%</span><span>1.0 ms</span></span>"
        ));
        assert!(html.contains(
            "<span class=\"metric metric-search\" data-label=\"Ordering\"><span>20%</span><span>2.0 ms</span></span>"
        ));
        assert!(html.contains(
            "<span class=\"metric metric-search\" data-label=\"Scoring\"><span>30%</span><span>3.0 ms</span></span>"
        ));
        assert!(html.contains(
            "<span class=\"metric metric-search\" data-label=\"Proof\"><span>0%</span><span>0 ms</span></span>"
        ));
        assert!(html.contains(
            "<span class=\"metric metric-search\" data-label=\"Other\"><span>35%</span><span>3.5 ms</span></span>"
        ));
        assert!(html.contains("Budget exhausted"));
        assert!(html.contains("Share of searched moves"));
        assert!(html.contains("20%"));
        assert!(html.contains("TT"));
        assert!(!html.contains("0.0 / 0.0"));
        assert!(html.contains("<dt>Search Cost</dt>"));
        assert!(html.contains("Comparisons above 50% are marked green."));
    }

    #[test]
    fn html_report_normalizes_stage_time_when_instrumented_time_exceeds_wall_time() {
        let mut report = sample_report();
        let mut standing = sample_standing_with_search_costs("search-d2");
        standing.total_time_ms = 10;
        report.run.bots = vec!["search-d2".to_string()];
        report.standings = vec![standing];

        let html = render_tournament_report_html(&report);

        assert!(html.contains(
            "<span class=\"metric metric-search\" data-label=\"Move gen\"><span>15%</span><span>1.0 ms</span></span>"
        ));
        assert!(html.contains(
            "<span class=\"metric metric-search\" data-label=\"Ordering\"><span>31%</span><span>2.0 ms</span></span>"
        ));
        assert!(html.contains(
            "<span class=\"metric metric-search\" data-label=\"Scoring\"><span>46%</span><span>3.0 ms</span></span>"
        ));
        assert!(html.contains(
            "<span class=\"metric metric-search\" data-label=\"Threat detection\"><span>8%</span><span>0.5 ms</span></span>"
        ));
        assert!(html.contains(
            "<span class=\"metric metric-search\" data-label=\"Other\"><span>0%</span><span>0 ms</span></span>"
        ));
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
            "tactical_lite_entry_rank_queries": 11,
            "root_tactical_lite_entry_rank_queries": 3,
            "search_tactical_lite_entry_rank_queries": 8,
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
            "corridor_entry_checks": 12,
            "corridor_entries_accepted": 3,
            "corridor_own_entries_accepted": 2,
            "corridor_opponent_entries_accepted": 1,
            "corridor_resume_searches": 4,
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
        metrics["leaf_corridor_passes"] = serde_json::json!(10);
        metrics["leaf_corridor_completed"] = serde_json::json!(11);
        metrics["leaf_corridor_checks"] = serde_json::json!(12);
        metrics["leaf_corridor_active"] = serde_json::json!(13);
        metrics["leaf_corridor_quiet"] = serde_json::json!(14);
        metrics["leaf_corridor_static_exits"] = serde_json::json!(15);
        metrics["leaf_corridor_depth_exits"] = serde_json::json!(16);
        metrics["leaf_corridor_deadline_exits"] = serde_json::json!(17);
        metrics["leaf_corridor_terminal_exits"] = serde_json::json!(18);
        metrics["leaf_corridor_terminal_root_candidates"] = serde_json::json!(19);
        metrics["leaf_corridor_terminal_root_winning_candidates"] = serde_json::json!(20);
        metrics["leaf_corridor_terminal_root_losing_candidates"] = serde_json::json!(21);
        metrics["leaf_corridor_terminal_root_overrides"] = serde_json::json!(22);
        metrics["leaf_corridor_terminal_root_move_changes"] = serde_json::json!(23);
        metrics["leaf_corridor_terminal_root_move_confirmations"] = serde_json::json!(24);
        metrics["leaf_corridor_proof_candidates_considered"] = serde_json::json!(25);
        metrics["leaf_corridor_proof_wins"] = serde_json::json!(26);
        metrics["leaf_corridor_proof_losses"] = serde_json::json!(27);
        metrics["leaf_corridor_proof_unknown"] = serde_json::json!(28);
        metrics["leaf_corridor_proof_deadline_skips"] = serde_json::json!(29);
        metrics["leaf_corridor_proof_move_changes"] = serde_json::json!(30);
        metrics["leaf_corridor_proof_move_confirmations"] = serde_json::json!(31);
        metrics["leaf_corridor_proof_candidate_rank_total"] = serde_json::json!(32);
        metrics["leaf_corridor_proof_candidate_rank_max"] = serde_json::json!(6);
        metrics["leaf_corridor_proof_candidate_score_gap_total"] = serde_json::json!(123_456);
        metrics["leaf_corridor_proof_candidate_score_gap_max"] = serde_json::json!(50_000);
        metrics["leaf_corridor_proof_win_candidate_rank_total"] = serde_json::json!(7);
        metrics["leaf_corridor_proof_win_candidate_rank_max"] = serde_json::json!(2);
        metrics["pattern_frame_queries"] = serde_json::json!(15);
        metrics["pattern_frame_query_ns"] = serde_json::json!(150);
        metrics["pattern_frame_updates"] = serde_json::json!(8);
        metrics["pattern_frame_update_ns"] = serde_json::json!(800);
        metrics["pattern_frame_shadow_checks"] = serde_json::json!(15);
        metrics["pattern_frame_shadow_mismatches"] = serde_json::json!(0);
        metrics["tactical_lite_rank_scan_queries"] = serde_json::json!(5);
        metrics["tactical_lite_rank_scan_ns"] = serde_json::json!(50);
        metrics["tactical_lite_rank_frontier_clean_queries"] = serde_json::json!(6);
        metrics["tactical_lite_rank_frontier_clean_ns"] = serde_json::json!(60);
        metrics["tactical_lite_rank_frontier_dirty_queries"] = serde_json::json!(7);
        metrics["tactical_lite_rank_frontier_dirty_ns"] = serde_json::json!(70);
        metrics["tactical_lite_rank_frontier_fallback_queries"] = serde_json::json!(8);
        metrics["tactical_lite_rank_frontier_fallback_ns"] = serde_json::json!(80);
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
            "metrics": metrics
        });

        stats.record_move(11, Some(&trace));
        let report = stats.finish();

        assert_eq!(report.search_nodes, 100);
        assert_eq!(report.safety_nodes, 20);
        assert_eq!(report.corridor_nodes, 7);
        assert_eq!(report.corridor_branch_probes, 3);
        assert_eq!(report.corridor_max_depth, 2);
        assert_eq!(report.corridor_extra_plies, 3);
        assert_eq!(report.avg_corridor_extra_plies, 3.0);
        assert_eq!(report.corridor_entry_checks, 12);
        assert_eq!(report.corridor_entries_accepted, 3);
        assert_eq!(report.corridor_entry_acceptance_rate, 0.25);
        assert_eq!(report.corridor_own_entries_accepted, 2);
        assert_eq!(report.corridor_opponent_entries_accepted, 1);
        assert_eq!(report.corridor_resume_searches, 4);
        assert_eq!(report.corridor_width_exits, 5);
        assert_eq!(report.corridor_depth_exits, 6);
        assert_eq!(report.corridor_neutral_exits, 7);
        assert_eq!(report.corridor_terminal_exits, 8);
        assert_eq!(report.corridor_plies_followed, 9);
        assert_eq!(report.corridor_own_plies_followed, 6);
        assert_eq!(report.corridor_opponent_plies_followed, 3);
        assert_eq!(report.leaf_corridor_passes, 10);
        assert_eq!(report.leaf_corridor_completed, 11);
        assert_eq!(report.leaf_corridor_checks, 12);
        assert_eq!(report.leaf_corridor_active, 13);
        assert_eq!(report.leaf_corridor_quiet, 14);
        assert_eq!(report.leaf_corridor_static_exits, 15);
        assert_eq!(report.leaf_corridor_depth_exits, 16);
        assert_eq!(report.leaf_corridor_deadline_exits, 17);
        assert_eq!(report.leaf_corridor_terminal_exits, 18);
        assert_eq!(report.leaf_corridor_terminal_root_candidates, 19);
        assert_eq!(report.leaf_corridor_terminal_root_winning_candidates, 20);
        assert_eq!(report.leaf_corridor_terminal_root_losing_candidates, 21);
        assert_eq!(report.leaf_corridor_terminal_root_overrides, 22);
        assert_eq!(report.leaf_corridor_terminal_root_move_changes, 23);
        assert_eq!(report.leaf_corridor_terminal_root_move_confirmations, 24);
        assert_eq!(report.leaf_corridor_proof_candidates_considered, 25);
        assert_eq!(report.leaf_corridor_proof_wins, 26);
        assert_eq!(report.leaf_corridor_proof_losses, 27);
        assert_eq!(report.leaf_corridor_proof_unknown, 28);
        assert_eq!(report.leaf_corridor_proof_deadline_skips, 29);
        assert_eq!(report.leaf_corridor_proof_move_changes, 30);
        assert_eq!(report.leaf_corridor_proof_move_confirmations, 31);
        assert_eq!(report.leaf_corridor_proof_candidate_rank_total, 32);
        assert_eq!(report.leaf_corridor_proof_candidate_rank_max, 6);
        assert_eq!(
            report.leaf_corridor_proof_candidate_score_gap_total,
            123_456
        );
        assert_eq!(report.leaf_corridor_proof_candidate_score_gap_max, 50_000);
        assert_eq!(report.leaf_corridor_proof_win_candidate_rank_total, 7);
        assert_eq!(report.leaf_corridor_proof_win_candidate_rank_max, 2);
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
        assert_eq!(report.tactical_lite_entry_rank_queries, 11);
        assert_eq!(report.root_tactical_lite_entry_rank_queries, 3);
        assert_eq!(report.search_tactical_lite_entry_rank_queries, 8);
        assert_eq!(report.tactical_lite_rank_scan_queries, 5);
        assert_eq!(report.tactical_lite_rank_scan_ns, 50);
        assert_eq!(report.tactical_lite_rank_frontier_clean_queries, 6);
        assert_eq!(report.tactical_lite_rank_frontier_clean_ns, 60);
        assert_eq!(report.tactical_lite_rank_frontier_dirty_queries, 7);
        assert_eq!(report.tactical_lite_rank_frontier_dirty_ns, 70);
        assert_eq!(report.tactical_lite_rank_frontier_fallback_queries, 8);
        assert_eq!(report.tactical_lite_rank_frontier_fallback_ns, 80);
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
        first_match.black_stats.corridor_extra_plies = 11;
        first_match.black_stats.avg_corridor_extra_plies = 2.2;
        first_match.black_stats.corridor_entry_checks = 40;
        first_match.black_stats.corridor_entries_accepted = 10;
        first_match.black_stats.corridor_entry_acceptance_rate = 0.25;
        first_match.black_stats.corridor_own_entries_accepted = 7;
        first_match.black_stats.corridor_opponent_entries_accepted = 3;
        first_match.black_stats.corridor_resume_searches = 8;
        first_match.black_stats.corridor_width_exits = 6;
        first_match.black_stats.corridor_depth_exits = 5;
        first_match.black_stats.corridor_neutral_exits = 4;
        first_match.black_stats.corridor_terminal_exits = 3;
        first_match.black_stats.corridor_plies_followed = 12;
        first_match.black_stats.corridor_own_plies_followed = 9;
        first_match.black_stats.corridor_opponent_plies_followed = 3;
        first_match.black_stats.leaf_corridor_terminal_exits = 13;
        first_match
            .black_stats
            .leaf_corridor_terminal_root_candidates = 7;
        first_match
            .black_stats
            .leaf_corridor_terminal_root_winning_candidates = 5;
        first_match
            .black_stats
            .leaf_corridor_terminal_root_losing_candidates = 2;
        first_match
            .black_stats
            .leaf_corridor_terminal_root_overrides = 2;
        first_match
            .black_stats
            .leaf_corridor_terminal_root_move_changes = 1;
        first_match
            .black_stats
            .leaf_corridor_terminal_root_move_confirmations = 1;
        first_match
            .black_stats
            .leaf_corridor_proof_candidates_considered = 9;
        first_match.black_stats.leaf_corridor_proof_wins = 4;
        first_match.black_stats.leaf_corridor_proof_losses = 3;
        first_match.black_stats.leaf_corridor_proof_unknown = 2;
        first_match.black_stats.leaf_corridor_proof_deadline_skips = 1;
        first_match.black_stats.leaf_corridor_proof_move_changes = 1;
        first_match
            .black_stats
            .leaf_corridor_proof_move_confirmations = 1;
        first_match
            .black_stats
            .leaf_corridor_proof_candidate_rank_total = 12;
        first_match
            .black_stats
            .leaf_corridor_proof_candidate_rank_max = 4;
        first_match
            .black_stats
            .leaf_corridor_proof_candidate_score_gap_total = 75_000;
        first_match
            .black_stats
            .leaf_corridor_proof_candidate_score_gap_max = 50_000;
        first_match
            .black_stats
            .leaf_corridor_proof_win_candidate_rank_total = 3;
        first_match
            .black_stats
            .leaf_corridor_proof_win_candidate_rank_max = 2;
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
        assert_eq!(row.corridor_extra_plies, 11);
        assert_eq!(row.avg_corridor_extra_plies, 2.2);
        assert_eq!(row.corridor_entry_checks, 40);
        assert_eq!(row.corridor_entries_accepted, 10);
        assert_eq!(row.corridor_entry_acceptance_rate, 0.25);
        assert_eq!(row.corridor_own_entries_accepted, 7);
        assert_eq!(row.corridor_opponent_entries_accepted, 3);
        assert_eq!(row.corridor_resume_searches, 8);
        assert_eq!(row.corridor_width_exits, 6);
        assert_eq!(row.corridor_depth_exits, 5);
        assert_eq!(row.corridor_neutral_exits, 4);
        assert_eq!(row.corridor_terminal_exits, 3);
        assert_eq!(row.corridor_plies_followed, 12);
        assert_eq!(row.corridor_own_plies_followed, 9);
        assert_eq!(row.corridor_opponent_plies_followed, 3);
        assert_eq!(row.leaf_corridor_terminal_exits, 13);
        assert_eq!(row.leaf_corridor_terminal_root_candidates, 7);
        assert_eq!(row.leaf_corridor_terminal_root_winning_candidates, 5);
        assert_eq!(row.leaf_corridor_terminal_root_losing_candidates, 2);
        assert_eq!(row.leaf_corridor_terminal_root_overrides, 2);
        assert_eq!(row.leaf_corridor_terminal_root_move_changes, 1);
        assert_eq!(row.leaf_corridor_terminal_root_move_confirmations, 1);
        assert_eq!(row.leaf_corridor_proof_candidates_considered, 9);
        assert_eq!(row.leaf_corridor_proof_wins, 4);
        assert_eq!(row.leaf_corridor_proof_losses, 3);
        assert_eq!(row.leaf_corridor_proof_unknown, 2);
        assert_eq!(row.leaf_corridor_proof_deadline_skips, 1);
        assert_eq!(row.leaf_corridor_proof_move_changes, 1);
        assert_eq!(row.leaf_corridor_proof_move_confirmations, 1);
        assert_eq!(row.leaf_corridor_proof_candidate_rank_total, 12);
        assert_eq!(row.leaf_corridor_proof_candidate_rank_max, 4);
        assert_eq!(row.leaf_corridor_proof_candidate_score_gap_total, 75_000);
        assert_eq!(row.leaf_corridor_proof_candidate_score_gap_max, 50_000);
        assert_eq!(row.leaf_corridor_proof_win_candidate_rank_total, 3);
        assert_eq!(row.leaf_corridor_proof_win_candidate_rank_max, 2);
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
                path: Some("reports/latest.json".to_string()),
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
                "search-d5+corridor-own-d6-w3+corridor-opponent-d4-w2"
            ),
            "SearchBot_D5+OwnCorrD6W3+OppCorrD4W2"
        );
        assert_eq!(
            compact_bot_label(&report, "search-d5+tactical-cap-8+corridor-own-d1-w3"),
            "SearchBot_D5+TCap8+OwnCorrD1W3"
        );
        assert_eq!(
            compact_bot_label(&report, "search-d5+leaf-corridor-d8-w3"),
            "SearchBot_D5+LeafCorrD8W3"
        );
        assert_eq!(
            compact_bot_label(&report, "search-d5+leaf-corridor-d8-w3+leaf-proof-c6"),
            "SearchBot_D5+LeafCorrD8W3+ProofC6"
        );
        assert_eq!(
            compact_bot_label(
                &report,
                "search-d5+leaf-corridor-d8-w3+leaf-proof-any-score"
            ),
            "SearchBot_D5+LeafCorrD8W3+leaf-proof-any-score"
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
            Some("reports/latest.json".to_string()),
            &source,
            &["anchor-a".to_string(), "anchor-b".to_string()],
        )
        .expect("anchors should be copied");

        assert_eq!(
            reference.source.path.as_deref(),
            Some("reports/latest.json")
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
        let err = reference.validate_compatible_run(&run).unwrap_err();

        assert!(err.contains("search_budget_mode"));
        assert!(err.contains("search_cpu_reserve_ms"));
    }

    #[test]
    fn html_report_renders_reference_anchors() {
        let mut report = sample_report();
        report.run.schedule = "gauntlet".to_string();
        report.reference_anchors = Some(AnchorReferenceReport {
            source: AnchorReferenceSource {
                path: Some("reports/latest.json".to_string()),
                schedule: "round-robin".to_string(),
                git_commit: Some("abc123".to_string()),
                git_dirty: Some(false),
                rules: report.run.rules.clone(),
                games_per_pair: 64,
                opening_policy: "centered-suite".to_string(),
                opening_plies: 4,
                seed: 48,
                search_time_ms: None,
                search_cpu_time_ms: Some(1000),
                search_budget_mode: "strict".to_string(),
                search_cpu_reserve_ms: None,
                max_moves: Some(120),
                max_game_ms: None,
            },
            anchors: vec![AnchorStandingReport {
                bot: "anchor-a".to_string(),
                sequential_elo: 1220.0,
                shuffled_elo_avg: 1234.5,
                shuffled_elo_stddev: 12.0,
                match_count: 128,
                score_percentage: 56.27,
            }],
            pairwise: vec![],
            pair_search: vec![],
        });

        let html = render_tournament_report_html(&report);

        assert!(html.contains("<h2>Reference Anchors</h2>"));
        assert!(html.contains("reports/latest.json"));
        assert!(html.contains("CPU 1000 ms/move"));
        assert!(html.contains("max 120 moves"));
        assert!(html.contains("abc123"));
        assert!(html.contains("anchor-a"));
        assert!(html.contains("1234.5 +/- 12.0"));
        assert!(html.contains("56.3%"));
    }

    #[test]
    fn html_report_marks_gauntlet_candidates_and_anchors() {
        let mut report = sample_report();
        report.run.schedule = "gauntlet".to_string();
        report.run.bots = vec!["candidate-a".to_string(), "anchor-a".to_string()];
        report.standings = vec![
            sample_standing_with_search_costs("candidate-a"),
            sample_standing_with_search_costs("anchor-a"),
        ];
        report.reference_anchors = Some(AnchorReferenceReport {
            source: AnchorReferenceSource {
                path: Some("reports/latest.json".to_string()),
                schedule: "round-robin".to_string(),
                git_commit: Some("abc123".to_string()),
                git_dirty: Some(false),
                rules: report.run.rules.clone(),
                games_per_pair: 64,
                opening_policy: "centered-suite".to_string(),
                opening_plies: 4,
                seed: 48,
                search_time_ms: None,
                search_cpu_time_ms: Some(1000),
                search_budget_mode: "strict".to_string(),
                search_cpu_reserve_ms: None,
                max_moves: Some(120),
                max_game_ms: None,
            },
            anchors: vec![AnchorStandingReport {
                bot: "anchor-a".to_string(),
                sequential_elo: 1220.0,
                shuffled_elo_avg: 1234.5,
                shuffled_elo_stddev: 12.0,
                match_count: 128,
                score_percentage: 56.27,
            }],
            pairwise: vec![],
            pair_search: vec![],
        });

        let html = render_tournament_report_html(&report);

        assert!(html.contains("<details class=\"entrant-row role-candidate\">"));
        assert!(html.contains("<span class=\"role-badge role-badge-candidate\">candidate</span>"));
        assert!(html.contains("<details class=\"entrant-row role-anchor\">"));
        assert!(html.contains("<span class=\"role-badge role-badge-anchor\">anchor</span>"));
    }

    #[test]
    fn html_report_adds_reference_anchor_pairs_to_anchor_rows() {
        let mut report = sample_report();
        report.run.schedule = "gauntlet".to_string();
        report.run.bots = vec![
            "candidate-a".to_string(),
            "anchor-a".to_string(),
            "anchor-b".to_string(),
        ];
        report.standings = vec![
            sample_standing_with_search_costs("candidate-a"),
            sample_standing_with_search_costs("anchor-a"),
            sample_standing_with_search_costs("anchor-b"),
        ];
        report.pairwise = vec![PairwiseReport {
            bot_a: "candidate-a".to_string(),
            bot_b: "anchor-a".to_string(),
            wins_a: 12,
            wins_b: 20,
            draws: 0,
            total: 32,
            score_a: 12.0,
            score_b: 20.0,
        }];
        report.reference_anchors = Some(AnchorReferenceReport {
            source: AnchorReferenceSource {
                path: Some("reports/latest.json".to_string()),
                schedule: "round-robin".to_string(),
                git_commit: Some("abc123".to_string()),
                git_dirty: Some(false),
                rules: report.run.rules.clone(),
                games_per_pair: 64,
                opening_policy: "centered-suite".to_string(),
                opening_plies: 4,
                seed: 48,
                search_time_ms: None,
                search_cpu_time_ms: Some(1000),
                search_budget_mode: "strict".to_string(),
                search_cpu_reserve_ms: None,
                max_moves: Some(120),
                max_game_ms: None,
            },
            anchors: vec![
                AnchorStandingReport {
                    bot: "anchor-a".to_string(),
                    sequential_elo: 1220.0,
                    shuffled_elo_avg: 1234.5,
                    shuffled_elo_stddev: 12.0,
                    match_count: 128,
                    score_percentage: 56.27,
                },
                AnchorStandingReport {
                    bot: "anchor-b".to_string(),
                    sequential_elo: 1180.0,
                    shuffled_elo_avg: 1188.0,
                    shuffled_elo_stddev: 12.0,
                    match_count: 128,
                    score_percentage: 43.73,
                },
            ],
            pairwise: vec![PairwiseReport {
                bot_a: "anchor-a".to_string(),
                bot_b: "anchor-b".to_string(),
                wins_a: 35,
                wins_b: 29,
                draws: 0,
                total: 64,
                score_a: 35.0,
                score_b: 29.0,
            }],
            pair_search: vec![],
        });

        let html = render_tournament_report_html(&report);

        assert!(html.contains(
            "<span class=\"metric metric-pairwise\" data-label=\"Pairs\"><span>2 opponents</span>"
        ));
        assert!(html.contains("<span data-label=\"Matches\">64 reference matches</span>"));
        assert!(html.contains(
            "Reference anchor aggregate; per-match details live in the source anchor report."
        ));
    }

    #[test]
    fn html_report_adds_reference_pairs_to_result_and_search_comparisons() {
        let mut source = sample_report();
        source.run.schedule = "round-robin".to_string();
        source.standings = vec![
            sample_standing_with_search_costs("anchor-a"),
            sample_standing_with_search_costs("anchor-b"),
        ];
        source.pairwise = vec![PairwiseReport {
            bot_a: "anchor-a".to_string(),
            bot_b: "anchor-b".to_string(),
            wins_a: 35,
            wins_b: 29,
            draws: 0,
            total: 64,
            score_a: 35.0,
            score_b: 29.0,
        }];
        let mut reference_match = sample_match(1, "anchor-a", "anchor-b", Some("anchor-a"));
        reference_match.black_stats = sample_side_stats_with_search_costs();
        reference_match.black_stats.total_time_ms = 60;
        reference_match.black_stats.total_nodes = 6000;
        reference_match.white_stats = sample_side_stats_with_search_costs();
        reference_match.white_stats.total_time_ms = 90;
        reference_match.white_stats.total_nodes = 1500;
        source.matches = vec![reference_match];
        let reference = AnchorReferenceReport::from_report(
            Some("reports/latest.json".to_string()),
            &source,
            &["anchor-a".to_string(), "anchor-b".to_string()],
        )
        .expect("reference should include anchor pair data");

        let mut report = sample_report();
        report.run.schedule = "gauntlet".to_string();
        report.run.bots = vec![
            "candidate-a".to_string(),
            "anchor-a".to_string(),
            "anchor-b".to_string(),
        ];
        report.standings = vec![
            sample_standing_with_search_costs("candidate-a"),
            sample_standing_with_search_costs("anchor-a"),
            sample_standing_with_search_costs("anchor-b"),
        ];
        report.pairwise = vec![PairwiseReport {
            bot_a: "candidate-a".to_string(),
            bot_b: "anchor-a".to_string(),
            wins_a: 12,
            wins_b: 20,
            draws: 0,
            total: 32,
            score_a: 12.0,
            score_b: 20.0,
        }];
        report.reference_anchors = Some(reference);

        let html = render_tournament_report_html(&report);

        assert!(html.contains("<span data-label=\"Source\">reference</span>"));
        assert!(html.contains("<span class=\"score score-good\" data-label=\"Score\">54.7%</span>"));
        assert!(html.contains("<span data-label=\"Avg ms\">12.0 ms</span>"));
        assert!(html.contains("<span data-label=\"Avg nodes\">1.2k nodes</span>"));
    }

    #[test]
    fn html_report_combines_git_commit_and_dirty_flag() {
        let mut report = sample_report();
        report.provenance.git_commit = Some("abcdef123456".to_string());
        report.provenance.git_dirty = Some(true);

        let html = render_tournament_report_html(&report);

        assert!(html.contains("<dt>Git revision</dt><dd>abcdef123456_dirty</dd>"));
        assert!(!html.contains("<dt>Git commit</dt>"));
        assert!(!html.contains("<dt>Git dirty</dt>"));
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
            "bots": ["fast", "balanced"],
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
            "bot": "fast",
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
            "black": "fast",
            "white": "balanced",
            "result": "black_won",
            "winner": "fast",
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
                bots: vec!["fast".to_string(), "balanced".to_string()],
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
                max_moves: Some(120),
                max_game_ms: None,
                total_wall_time_ms: Some(100),
            },
            standings: Vec::new(),
            pairwise: vec![PairwiseReport {
                bot_a: "fast".to_string(),
                bot_b: "balanced".to_string(),
                wins_a: 0,
                wins_b: 2,
                draws: 0,
                total: 2,
                score_a: 0.0,
                score_b: 2.0,
            }],
            color_splits: vec![
                ColorSplitReport {
                    black: "fast".to_string(),
                    white: "balanced".to_string(),
                    black_wins: 0,
                    white_wins: 1,
                    draws: 0,
                    total: 1,
                },
                ColorSplitReport {
                    black: "balanced".to_string(),
                    white: "fast".to_string(),
                    black_wins: 1,
                    white_wins: 0,
                    draws: 0,
                    total: 1,
                },
            ],
            end_reasons: Vec::new(),
            matches: vec![
                sample_match(1, "fast", "balanced", Some("balanced")),
                sample_match(2, "balanced", "fast", Some("balanced")),
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
            corridor_extra_plies: 0,
            avg_corridor_extra_plies: 0.0,
            corridor_entry_checks: 0,
            corridor_entries_accepted: 0,
            corridor_entry_acceptance_rate: 0.0,
            corridor_own_entries_accepted: 0,
            corridor_opponent_entries_accepted: 0,
            corridor_resume_searches: 0,
            corridor_width_exits: 0,
            corridor_depth_exits: 0,
            corridor_neutral_exits: 0,
            corridor_terminal_exits: 0,
            corridor_plies_followed: 0,
            corridor_own_plies_followed: 0,
            corridor_opponent_plies_followed: 0,
            leaf_corridor_passes: 0,
            leaf_corridor_completed: 0,
            leaf_corridor_checks: 0,
            leaf_corridor_active: 0,
            leaf_corridor_quiet: 0,
            leaf_corridor_static_exits: 0,
            leaf_corridor_depth_exits: 0,
            leaf_corridor_deadline_exits: 0,
            leaf_corridor_terminal_exits: 0,
            leaf_corridor_terminal_root_candidates: 0,
            leaf_corridor_terminal_root_winning_candidates: 0,
            leaf_corridor_terminal_root_losing_candidates: 0,
            leaf_corridor_terminal_root_overrides: 0,
            leaf_corridor_terminal_root_move_changes: 0,
            leaf_corridor_terminal_root_move_confirmations: 0,
            leaf_corridor_proof_candidates_considered: 0,
            leaf_corridor_proof_wins: 0,
            leaf_corridor_proof_losses: 0,
            leaf_corridor_proof_unknown: 0,
            leaf_corridor_proof_deadline_skips: 0,
            leaf_corridor_proof_move_changes: 0,
            leaf_corridor_proof_move_confirmations: 0,
            leaf_corridor_proof_candidate_rank_total: 0,
            leaf_corridor_proof_candidate_rank_max: 0,
            leaf_corridor_proof_candidate_score_gap_total: 0,
            leaf_corridor_proof_candidate_score_gap_max: 0,
            leaf_corridor_proof_win_candidate_rank_total: 0,
            leaf_corridor_proof_win_candidate_rank_max: 0,
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
            stage_move_gen_ns: 5_000_000,
            stage_ordering_ns: 10_000_000,
            stage_eval_ns: 15_000_000,
            stage_threat_ns: 2_500_000,
            stage_proof_ns: 0,
            tactical_annotations: 8,
            root_tactical_annotations: 2,
            search_tactical_annotations: 6,
            tactical_lite_entry_rank_queries: 0,
            root_tactical_lite_entry_rank_queries: 0,
            search_tactical_lite_entry_rank_queries: 0,
            tactical_lite_rank_scan_queries: 0,
            tactical_lite_rank_scan_ns: 0,
            tactical_lite_rank_frontier_clean_queries: 0,
            tactical_lite_rank_frontier_clean_ns: 0,
            tactical_lite_rank_frontier_dirty_queries: 0,
            tactical_lite_rank_frontier_dirty_ns: 0,
            tactical_lite_rank_frontier_fallback_queries: 0,
            tactical_lite_rank_frontier_fallback_ns: 0,
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
            corridor_extra_plies: 0,
            avg_corridor_extra_plies: 0.0,
            corridor_entry_checks: 0,
            corridor_entries_accepted: 0,
            corridor_entry_acceptance_rate: 0.0,
            corridor_own_entries_accepted: 0,
            corridor_opponent_entries_accepted: 0,
            corridor_resume_searches: 0,
            corridor_width_exits: 0,
            corridor_depth_exits: 0,
            corridor_neutral_exits: 0,
            corridor_terminal_exits: 0,
            corridor_plies_followed: 0,
            corridor_own_plies_followed: 0,
            corridor_opponent_plies_followed: 0,
            leaf_corridor_passes: 0,
            leaf_corridor_completed: 0,
            leaf_corridor_checks: 0,
            leaf_corridor_active: 0,
            leaf_corridor_quiet: 0,
            leaf_corridor_static_exits: 0,
            leaf_corridor_depth_exits: 0,
            leaf_corridor_deadline_exits: 0,
            leaf_corridor_terminal_exits: 0,
            leaf_corridor_terminal_root_candidates: 0,
            leaf_corridor_terminal_root_winning_candidates: 0,
            leaf_corridor_terminal_root_losing_candidates: 0,
            leaf_corridor_terminal_root_overrides: 0,
            leaf_corridor_terminal_root_move_changes: 0,
            leaf_corridor_terminal_root_move_confirmations: 0,
            leaf_corridor_proof_candidates_considered: 0,
            leaf_corridor_proof_wins: 0,
            leaf_corridor_proof_losses: 0,
            leaf_corridor_proof_unknown: 0,
            leaf_corridor_proof_deadline_skips: 0,
            leaf_corridor_proof_move_changes: 0,
            leaf_corridor_proof_move_confirmations: 0,
            leaf_corridor_proof_candidate_rank_total: 0,
            leaf_corridor_proof_candidate_rank_max: 0,
            leaf_corridor_proof_candidate_score_gap_total: 0,
            leaf_corridor_proof_candidate_score_gap_max: 0,
            leaf_corridor_proof_win_candidate_rank_total: 0,
            leaf_corridor_proof_win_candidate_rank_max: 0,
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
            stage_move_gen_ns: 5_000_000,
            stage_ordering_ns: 10_000_000,
            stage_eval_ns: 15_000_000,
            stage_threat_ns: 2_500_000,
            stage_proof_ns: 0,
            tactical_annotations: 8,
            root_tactical_annotations: 2,
            search_tactical_annotations: 6,
            tactical_lite_entry_rank_queries: 0,
            root_tactical_lite_entry_rank_queries: 0,
            search_tactical_lite_entry_rank_queries: 0,
            tactical_lite_rank_scan_queries: 0,
            tactical_lite_rank_scan_ns: 0,
            tactical_lite_rank_frontier_clean_queries: 0,
            tactical_lite_rank_frontier_clean_ns: 0,
            tactical_lite_rank_frontier_dirty_queries: 0,
            tactical_lite_rank_frontier_dirty_ns: 0,
            tactical_lite_rank_frontier_fallback_queries: 0,
            tactical_lite_rank_frontier_fallback_ns: 0,
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
        }
    }
}
