use crate::arena::MatchEndReason;
use crate::elo::{expected_score, DEFAULT_INITIAL_RATING, DEFAULT_K_FACTOR};
use crate::tournament::TournamentResults;
use gomoku_core::{Color, GameResult, Move, RuleConfig};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
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
                max_moves: source_report.run.max_moves,
                max_game_ms: source_report.run.max_game_ms,
            },
            anchors,
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
    pub total_nodes: u64,
    pub avg_nodes: f64,
    #[serde(default)]
    pub eval_calls: u64,
    #[serde(default)]
    pub avg_eval_calls: f64,
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
    pub tt_hits: u64,
    #[serde(default)]
    pub tt_cutoffs: u64,
    #[serde(default)]
    pub beta_cutoffs: u64,
    pub avg_depth: f64,
    pub max_depth: u32,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchReport {
    pub match_index: usize,
    pub black: String,
    pub white: String,
    pub result: String,
    pub winner: Option<String>,
    pub end_reason: String,
    pub duration_ms: Option<u64>,
    pub move_cells: Vec<usize>,
    pub move_count: usize,
    pub black_stats: SideStatsReport,
    pub white_stats: SideStatsReport,
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
    pub search_nodes: u64,
    pub safety_nodes: u64,
    pub total_nodes: u64,
    pub avg_nodes: f64,
    #[serde(default)]
    pub eval_calls: u64,
    #[serde(default)]
    pub avg_eval_calls: f64,
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
    pub tt_hits: u64,
    #[serde(default)]
    pub tt_cutoffs: u64,
    #[serde(default)]
    pub beta_cutoffs: u64,
    pub depth_sum: u64,
    pub avg_depth: f64,
    pub max_depth: u32,
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
    total_nodes: u64,
    eval_calls: u64,
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
    tt_hits: u64,
    tt_cutoffs: u64,
    beta_cutoffs: u64,
    depth_sum: u64,
    max_depth: u32,
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
        if let Some(metrics) = trace.get("metrics") {
            self.eval_calls += trace_value_u64(metrics, "eval_calls");
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
            self.tt_hits += trace_value_u64(metrics, "tt_hits");
            self.tt_cutoffs += trace_value_u64(metrics, "tt_cutoffs");
            self.beta_cutoffs += trace_value_u64(metrics, "beta_cutoffs");
        }
        if let Some(depth) = trace.get("depth").and_then(Value::as_u64) {
            self.depth_sum += depth;
            self.max_depth = self.max_depth.max(depth as u32);
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
        self.total_nodes += stats.total_nodes;
        self.eval_calls += stats.eval_calls;
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
        self.tt_hits += stats.tt_hits;
        self.tt_cutoffs += stats.tt_cutoffs;
        self.beta_cutoffs += stats.beta_cutoffs;
        self.depth_sum += stats.depth_sum;
        self.max_depth = self.max_depth.max(stats.max_depth);
        self.budget_exhausted_count += stats.budget_exhausted_count;
    }

    fn finish(self) -> SideStatsReport {
        let avg_search_time_ms = avg(self.total_time_ms as f64, self.search_move_count);
        let avg_nodes = avg(self.total_nodes as f64, self.search_move_count);
        let avg_eval_calls = avg(self.eval_calls as f64, self.search_move_count);
        let avg_candidate_generations =
            avg(self.candidate_generations as f64, self.search_move_count);
        let avg_candidate_moves = avg(
            self.candidate_moves_total as f64,
            self.candidate_generations as u32,
        );
        let avg_legality_checks = avg(self.legality_checks as f64, self.search_move_count);
        let avg_depth = avg(self.depth_sum as f64, self.search_move_count);
        let budget_exhausted_rate = avg(self.budget_exhausted_count as f64, self.search_move_count);

        SideStatsReport {
            move_count: self.move_count,
            search_move_count: self.search_move_count,
            total_time_ms: self.total_time_ms,
            avg_search_time_ms,
            search_nodes: self.search_nodes,
            safety_nodes: self.safety_nodes,
            total_nodes: self.total_nodes,
            avg_nodes,
            eval_calls: self.eval_calls,
            avg_eval_calls,
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
            tt_hits: self.tt_hits,
            tt_cutoffs: self.tt_cutoffs,
            beta_cutoffs: self.beta_cutoffs,
            depth_sum: self.depth_sum,
            avg_depth,
            max_depth: self.max_depth,
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
                total_nodes: side_stats.total_nodes,
                avg_nodes: side_stats.avg_nodes,
                eval_calls: side_stats.eval_calls,
                avg_eval_calls: side_stats.avg_eval_calls,
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
                tt_hits: side_stats.tt_hits,
                tt_cutoffs: side_stats.tt_cutoffs,
                beta_cutoffs: side_stats.beta_cutoffs,
                avg_depth: side_stats.avg_depth,
                max_depth: side_stats.max_depth,
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
}

pub fn render_tournament_report_html_with_options(
    report: &TournamentReport,
    options: &ReportRenderOptions,
) -> String {
    let mut html = String::new();
    let leader = report_leader(report);
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
    html.push_str(
        "<nav class=\"top-links\"><a href=\"/\">Game</a><a href=\"/assets/\">Asset previews</a>",
    );
    if let Some(href) = &options.raw_json_href {
        html.push_str(&format!("<a href=\"{}\">Raw JSON</a>", html_escape(href)));
    }
    html.push_str("</nav>");
    html.push_str("<p class=\"eyebrow\">Gomoku2D Bot Lab</p><h1>Bot Lab Report</h1>");
    html.push_str("<p class=\"lede\">A bot evaluation report for comparing specs under one rule set, opening policy, and search budget.</p></header>");
    if report.provenance.git_dirty == Some(true) {
        html.push_str(
            "<p class=\"run-warning\">Development run: generated from a dirty git worktree.</p>",
        );
    }

    html.push_str("<section class=\"cards\"><div class=\"card-group\"><h2>Tournament</h2><div class=\"card-row\">");
    metric_card(&mut html, "Workflow", report.run.schedule.clone());
    metric_card(&mut html, "Entrants", entrant_summary(report));
    metric_card(&mut html, "Schedule", schedule_summary(report));
    metric_card(
        &mut html,
        "Rule",
        variant_label(&report.run.rules).to_string(),
    );
    metric_card(&mut html, "Opening", opening_summary(report));
    html.push_str("</div></div><div class=\"card-group\"><h2>Summary</h2><div class=\"card-row\">");
    metric_card(&mut html, "Matches", report.matches.len().to_string());
    metric_card(&mut html, "Finish", finish_summary(report));
    metric_card(&mut html, "Result By Color", color_summary(report));
    html.push_str("</div></div><div class=\"card-group\"><h2>Ranking</h2><div class=\"card-row\">");
    metric_card(&mut html, "Leader", leader);
    metric_card(
        &mut html,
        "Elo Start",
        format!("{DEFAULT_INITIAL_RATING:.0}"),
    );
    metric_card(&mut html, "K Factor", format!("{DEFAULT_K_FACTOR:.0}"));
    metric_card(
        &mut html,
        "Shuffled Samples",
        report.shuffled_elo_samples.to_string(),
    );
    html.push_str("</div></div><div class=\"card-group\"><h2>Run</h2><div class=\"card-row\">");
    metric_card(&mut html, "Budget", budget);
    metric_card(
        &mut html,
        "Wall Clock",
        format_duration_ms(report.run.total_wall_time_ms),
    );
    metric_card(&mut html, "Eval Threads", report.run.threads.to_string());
    metric_card(&mut html, "CPU", host_cpu_summary(report));
    html.push_str("</div></div>");
    html.push_str("</section>");

    render_reference_anchors_section(&mut html, report);

    html.push_str("<section><div class=\"section-heading\"><h2>Standings</h2><p>Sorted by shuffled Elo.</p></div><table><thead><tr>");
    for head in [
        "Spec",
        "Score %",
        "W-D-L",
        "Run-order Elo",
        "Shuffled Elo",
        "Avg ms",
        "Avg nodes",
        "Avg depth",
        "Budget hit",
    ] {
        html.push_str(&format!("<th>{head}</th>"));
    }
    html.push_str("</tr></thead><tbody>");
    for row in &report.standings {
        html.push_str(&format!(
            "<tr><td>{}</td><td>{:.1}%</td><td>{}-{}-{}</td><td>{:.1}</td><td>{:.1} +/- {:.1}</td><td>{:.1}</td><td>{:.0}</td><td>{:.2}</td><td>{:.0}%</td></tr>",
            html_escape(&compact_bot_label(report, &row.bot)),
            score_rate(row.wins, row.draws, row.match_count) * 100.0,
            row.wins,
            row.draws,
            row.losses,
            row.sequential_elo,
            row.shuffled_elo_avg,
            row.shuffled_elo_stddev,
            row.avg_search_time_ms,
            row.avg_nodes,
            row.avg_depth,
            row.budget_exhausted_rate * 100.0,
        ));
    }
    html.push_str("</tbody></table></section>");

    render_search_cost_section(&mut html, report);
    render_match_tree(&mut html, report);
    render_how_to_read_section(&mut html);

    html.push_str("<section class=\"provenance\"><div class=\"section-heading\"><h2>Provenance</h2><p>Enough context to reproduce the run or compare against a later tuning pass.</p></div>");
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
    html.push_str("</dl>");
    html.push_str(&format!(
        "<p class=\"command\"><code>{}</code></p>",
        html_escape(&command)
    ));
    html.push_str("</section></main></body></html>");
    html
}

fn render_reference_anchors_section(html: &mut String, report: &TournamentReport) {
    let Some(reference) = &report.reference_anchors else {
        return;
    };

    html.push_str("<section><div class=\"section-heading\"><h2>Reference Anchors</h2>");
    html.push_str("<p>Cached ratings copied from a curated full report. Use them as working calibration for this gauntlet, not as permanent truth.</p></div>");
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
    match (source.search_cpu_time_ms, source.search_time_ms) {
        (Some(cpu_ms), Some(wall_ms)) => {
            format!("CPU {cpu_ms} ms/move, wall {wall_ms} ms/move")
        }
        (Some(cpu_ms), None) => format!("CPU {cpu_ms} ms/move"),
        (None, Some(wall_ms)) => format!("Wall {wall_ms} ms/move"),
        (None, None) => "no per-move budget".to_string(),
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

fn report_leader(report: &TournamentReport) -> String {
    report
        .standings
        .first()
        .map(|row| {
            format!(
                "{} ({:.1})",
                bot_label(report, &row.bot),
                row.shuffled_elo_avg
            )
        })
        .unwrap_or_else(|| "none".to_string())
}

fn budget_label(run: &TournamentRunReport) -> String {
    match (run.search_cpu_time_ms, run.search_time_ms) {
        (Some(cpu_ms), Some(wall_ms)) => {
            format!("CPU {cpu_ms} ms/move, wall {wall_ms} ms/move")
        }
        (Some(cpu_ms), None) => format!("CPU {cpu_ms} ms/move"),
        (None, Some(wall_ms)) => format!("Wall {wall_ms} ms/move"),
        (None, None) => "no per-move budget".to_string(),
    }
}

fn color_summary(report: &TournamentReport) -> String {
    let mut black_wins = 0u32;
    let mut white_wins = 0u32;
    let mut draws = 0u32;

    for row in &report.color_splits {
        black_wins += row.black_wins;
        white_wins += row.white_wins;
        draws += row.draws;
    }

    let total = black_wins + white_wins + draws;
    if total == 0 {
        return "none".to_string();
    }

    format!(
        "Black {} ({:.1}%) / White {} ({:.1}%) / Draw {} ({:.1}%)",
        black_wins,
        black_wins as f64 * 100.0 / total as f64,
        white_wins,
        white_wins as f64 * 100.0 / total as f64,
        draws,
        draws as f64 * 100.0 / total as f64
    )
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

fn entrant_summary(report: &TournamentReport) -> String {
    let labels = report
        .run
        .bots
        .iter()
        .map(|bot| entrant_label(bot, &report.run))
        .collect::<Vec<_>>();

    collapse_searchbot_depth_labels(&labels).unwrap_or_else(|| labels.join(", "))
}

fn collapse_searchbot_depth_labels(labels: &[String]) -> Option<String> {
    let prefix = "SearchBot @ depth ";
    let depths = labels
        .iter()
        .map(|label| label.strip_prefix(prefix))
        .collect::<Option<Vec<_>>>()?;

    Some(format!("{prefix}{}", depths.join(" / ")))
}

fn schedule_summary(report: &TournamentReport) -> String {
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
    "random-legal".to_string()
}

fn default_schedule() -> String {
    "round-robin".to_string()
}

fn bot_label(report: &TournamentReport, bot: &str) -> String {
    entrant_label(bot, &report.run)
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

fn entrant_label(bot: &str, run: &TournamentRunReport) -> String {
    if bot == "random" {
        return "RandomBot".to_string();
    }

    if let Some((depth, features)) = searchbot_spec(bot, run) {
        let mut label = format!("SearchBot @ depth {depth}");
        for feature in features {
            label.push_str(" + ");
            label.push_str(&full_searchbot_feature_label(feature));
        }
        return label;
    }

    bot.to_string()
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
    if let Some(cap) = feature.strip_prefix("child-cap-") {
        return format!("Cap{cap}");
    }
    if let Some(radius) = feature.strip_prefix("near-all-r") {
        return format!("NearR{radius}");
    }

    match feature {
        "pattern-eval" => "Pattern".to_string(),
        "tactical-first" => "Tactical".to_string(),
        "no-safety" => "NoSafety".to_string(),
        "opponent-reply-search-probe" => "SearchProbe".to_string(),
        "opponent-reply-local-threat-probe" => "LocalThreat".to_string(),
        _ => feature.to_string(),
    }
}

fn full_searchbot_feature_label(feature: &str) -> String {
    if let Some(cap) = feature.strip_prefix("tactical-cap-") {
        return format!("tactical cap {cap}");
    }
    if let Some(cap) = feature.strip_prefix("child-cap-") {
        return format!("child cap {cap}");
    }
    if let Some(radius) = feature.strip_prefix("near-all-r") {
        return format!("near all r{radius}");
    }

    match feature {
        "pattern-eval" => "pattern eval".to_string(),
        "tactical-first" => "tactical first".to_string(),
        "no-safety" => "no safety".to_string(),
        "opponent-reply-search-probe" => "opponent reply search probe".to_string(),
        "opponent-reply-local-threat-probe" => "opponent reply local threat probe".to_string(),
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
    html.push_str("<section><div class=\"section-heading\"><h2>How To Read This</h2><p>Compact definitions for the metrics above.</p></div><div class=\"term-grid\">");
    term_card(
        html,
        "Run Shape",
        "Workflow names the pairing mode. Schedule shows played pair count, games per pair, and total matches. Opening shows the seeded legal moves before bots take over.",
    );
    term_card(
        html,
        "Elo",
        "Relative rating for this report only. Shuffled Elo averages repeated Elo passes over shuffled match orders to reduce run-order noise.",
    );
    term_card(
        html,
        "Score",
        "Score % is wins plus half draws over games. A-D-B means A wins, draws, then B wins for the listed pair.",
    );
    term_card(
        html,
        "Color Result",
        "Pair groups show each bot's win rate as black and white. Useful when a result is carried by first-player edge or opening assignment.",
    );
    term_card(
        html,
        "Timing",
        "CPU-time budget limits search CPU per move. Wall clock plus hardware gives context for comparing runs.",
    );
    term_card(
        html,
        "Search Cost",
        "Cand r/s and legal r/s split root-stage work from alpha-beta search work. Width is average candidate moves per generated candidate set.",
    );
    html.push_str("</div></section>");
}

fn render_search_cost_section(html: &mut String, report: &TournamentReport) {
    if !report.standings.iter().any(has_search_cost_metrics) {
        return;
    }

    html.push_str("<section><div class=\"section-heading\"><h2>Search Cost</h2><p>Per-search-move instrumentation. Split cells show root-stage / search costs.</p></div><table><thead><tr>");
    for head in [
        "Spec",
        "Avg eval",
        "Cand gen r/s",
        "Avg width",
        "Legal r/s",
        "TT hit/cut",
        "Beta cuts",
    ] {
        html.push_str(&format!("<th>{head}</th>"));
    }
    html.push_str("</tr></thead><tbody>");
    for row in &report.standings {
        html.push_str(&format!(
            "<tr><td>{}</td><td>{:.1}</td><td>{}</td><td>{:.1}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            html_escape(&compact_bot_label(report, &row.bot)),
            row.avg_eval_calls,
            html_escape(&phase_average_label(
                row.root_candidate_generations,
                row.search_candidate_generations,
                row.search_move_count,
            )),
            row.avg_candidate_moves,
            html_escape(&phase_average_label(
                row.root_legality_checks,
                row.search_legality_checks,
                row.search_move_count,
            )),
            html_escape(&phase_average_label_zero_valid(
                row.tt_hits,
                row.tt_cutoffs,
                row.search_move_count,
            )),
            html_escape(&average_label(row.beta_cutoffs, row.search_move_count)),
        ));
    }
    html.push_str("</tbody></table></section>");
}

fn render_match_tree(html: &mut String, report: &TournamentReport) {
    html.push_str("<section><div class=\"section-heading\"><h2>Matches By Pair</h2><p>Expand a bot pair to inspect individual games. Finished boards and human move notation come first; raw cell indexes stay tucked under each match.</p></div><div class=\"pair-tree\">");
    for pair in &report.pairwise {
        let matches = report
            .matches
            .iter()
            .filter(|report_match| same_pair(report_match, &pair.bot_a, &pair.bot_b))
            .collect::<Vec<_>>();

        html.push_str("<details class=\"pair-group\"><summary>");
        html.push_str(&format!(
            "<strong>{}</strong><span>{} matches</span><span>{}</span><span>{}</span></summary>",
            html_escape(&pair_label(report, pair)),
            matches.len(),
            html_escape(&pair_record_label(pair)),
            html_escape(&pair_score_label(pair)),
        ));
        render_pair_overview(html, report, pair);
        html.push_str("<div class=\"match-list\">");
        for report_match in matches {
            render_match(html, report, pair, report_match);
        }
        html.push_str("</div></details>");
    }
    html.push_str("</div></section>");
}

fn render_pair_overview(html: &mut String, report: &TournamentReport, pair: &PairwiseReport) {
    html.push_str("<div class=\"pair-overview\">");
    html.push_str(&format!(
        "<p><b>Pair result</b><br>{}; {}; {} points rate {:.1}%</p>",
        html_escape(&pair_record_label(pair)),
        html_escape(&pair_score_label(pair)),
        html_escape(&compact_bot_label(report, &pair.bot_a)),
        avg(pair.score_a * 100.0, pair.total),
    ));

    let color_lines = color_result_lines(report, pair);
    if !color_lines.is_empty() {
        html.push_str(&format!(
            "<p><b>Color result</b><br>{}</p>",
            color_lines
                .iter()
                .map(|line| html_escape(line))
                .collect::<Vec<_>>()
                .join("<br>")
        ));
    }
    html.push_str("</div>");
}

fn render_match(
    html: &mut String,
    report: &TournamentReport,
    pair: &PairwiseReport,
    report_match: &MatchReport,
) {
    html.push_str("<details class=\"match\"><summary>");
    let bot_a_label = match_side_label(report, &pair.bot_a, report_match);
    let bot_b_label = match_side_label(report, &pair.bot_b, report_match);
    html.push_str(&format!(
        "<span>#{:03}</span><strong>{} vs {}</strong><span>{}</span><span>{} moves</span><span>{}</span></summary>",
        report_match.match_index,
        html_escape(&bot_a_label),
        html_escape(&bot_b_label),
        html_escape(&result_label(report, report_match)),
        report_match.move_count,
        html_escape(&report_match.end_reason),
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
        "<p><b>{} stats</b><br>{}</p>",
        html_escape(&bot_a_label),
        html_escape(&side_stats_label(side_stats_for_bot(
            report_match,
            &pair.bot_a
        )))
    ));
    html.push_str(&format!(
        "<p><b>{} stats</b><br>{}</p>",
        html_escape(&bot_b_label),
        html_escape(&side_stats_label(side_stats_for_bot(
            report_match,
            &pair.bot_b
        )))
    ));
    html.push_str(&format!(
        "<details class=\"raw-data\"><summary>Raw data</summary><p><b>Move cells</b><br>{}</p></details>",
        report_match
            .move_cells
            .iter()
            .map(usize::to_string)
            .collect::<Vec<_>>()
            .join(" ")
    ));
    html.push_str("</div></details>");
}

fn match_side_label(report: &TournamentReport, bot: &str, report_match: &MatchReport) -> String {
    let side = if report_match.black == bot {
        "B"
    } else if report_match.white == bot {
        "W"
    } else {
        "?"
    };
    format!("{} ({side})", compact_bot_label(report, bot))
}

fn side_stats_for_bot<'a>(report_match: &'a MatchReport, bot: &str) -> &'a SideStatsReport {
    if report_match.black == bot {
        &report_match.black_stats
    } else {
        &report_match.white_stats
    }
}

fn side_stats_label(stats: &SideStatsReport) -> String {
    let base = format!(
        "{:.1} ms, {:.0} nodes, depth {:.2}, budget {:.0}%",
        stats.avg_search_time_ms,
        stats.avg_nodes,
        stats.avg_depth,
        stats.budget_exhausted_rate * 100.0,
    );

    if !has_side_search_cost_metrics(stats) {
        return base;
    }

    format!(
        "{base}; eval {:.0}, cand r/s {}, legal r/s {}, tt {}/{}",
        stats.avg_eval_calls,
        phase_average_label(
            stats.root_candidate_generations,
            stats.search_candidate_generations,
            stats.search_move_count,
        ),
        phase_average_label(
            stats.root_legality_checks,
            stats.search_legality_checks,
            stats.search_move_count,
        ),
        average_label(stats.tt_hits, stats.search_move_count),
        average_label(stats.tt_cutoffs, stats.search_move_count),
    )
}

fn has_search_cost_metrics(row: &StandingReport) -> bool {
    row.eval_calls > 0
        || row.candidate_generations > 0
        || row.legality_checks > 0
        || row.tt_hits > 0
        || row.tt_cutoffs > 0
        || row.beta_cutoffs > 0
}

fn has_side_search_cost_metrics(stats: &SideStatsReport) -> bool {
    stats.eval_calls > 0
        || stats.candidate_generations > 0
        || stats.legality_checks > 0
        || stats.tt_hits > 0
        || stats.tt_cutoffs > 0
        || stats.beta_cutoffs > 0
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

fn pair_label(report: &TournamentReport, pair: &PairwiseReport) -> String {
    format!(
        "{} vs {}",
        compact_bot_label(report, &pair.bot_a),
        compact_bot_label(report, &pair.bot_b)
    )
}

fn pair_record_label(pair: &PairwiseReport) -> String {
    format!("{}-{}-{} A-D-B", pair.wins_a, pair.draws, pair.wins_b)
}

fn pair_score_label(pair: &PairwiseReport) -> String {
    format!("{:.1}-{:.1} points", pair.score_a, pair.score_b)
}

fn color_result_lines(report: &TournamentReport, pair: &PairwiseReport) -> Vec<String> {
    let mut splits = report
        .color_splits
        .iter()
        .filter(|split| same_bot_pair(&split.black, &split.white, &pair.bot_a, &pair.bot_b))
        .collect::<Vec<_>>();
    splits.sort_by_key(|split| {
        if split.black == pair.bot_a {
            0
        } else if split.black == pair.bot_b {
            1
        } else {
            2
        }
    });
    [
        color_result_label(report, pair, &pair.bot_a, &splits),
        color_result_label(report, pair, &pair.bot_b, &splits),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn same_bot_pair(left_a: &str, left_b: &str, right_a: &str, right_b: &str) -> bool {
    (left_a == right_a && left_b == right_b) || (left_a == right_b && left_b == right_a)
}

fn color_result_label(
    report: &TournamentReport,
    pair: &PairwiseReport,
    bot: &str,
    splits: &[&ColorSplitReport],
) -> Option<String> {
    let as_black = splits.iter().find(|split| split.black == bot)?;
    let opponent = if bot == pair.bot_a {
        &pair.bot_b
    } else {
        &pair.bot_a
    };
    let as_white = splits
        .iter()
        .find(|split| split.black == *opponent && split.white == bot)?;

    let black_win_rate = avg(as_black.black_wins as f64 * 100.0, as_black.total);
    let white_win_rate = avg(as_white.white_wins as f64 * 100.0, as_white.total);

    Some(format!(
        "{}: black {:.1}% ({}/{}), white {:.1}% ({}/{})",
        compact_bot_label(report, bot),
        black_win_rate,
        as_black.black_wins,
        as_black.total,
        white_win_rate,
        as_white.white_wins,
        as_white.total,
    ))
}

fn metric_card(html: &mut String, label: &str, value: String) {
    html.push_str(&format!(
        "<article><span>{}</span><strong>{}</strong></article>",
        html_escape(label),
        html_escape(&value)
    ));
}

fn term_card(html: &mut String, title: &str, body: &str) {
    html.push_str(&format!(
        "<article class=\"term\"><h3>{}</h3><p>{}</p></article>",
        html_escape(title),
        body,
    ));
}

fn variant_label(rules: &RuleConfig) -> String {
    match rules.variant {
        gomoku_core::Variant::Freestyle => "freestyle".to_string(),
        gomoku_core::Variant::Renju => "renju".to_string(),
    }
}

fn result_label(report: &TournamentReport, report_match: &MatchReport) -> String {
    match report_match.winner.as_deref() {
        Some(winner) => format!("{} wins", compact_bot_label(report, winner)),
        None => "draw".to_string(),
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
.hero,section,.run-warning{background:var(--surface);border:2px solid var(--border);display:grid;gap:16px;padding:20px;overflow:auto}.run-warning{border-color:var(--accent);color:var(--accent)}.top-links{display:flex;flex-wrap:wrap;gap:8px}.top-links a{background:var(--surface-strong);border:2px solid var(--border);color:var(--text);display:inline-block;padding:8px 12px;text-transform:uppercase}.top-links a:hover,.top-links a:focus{border-color:var(--teal);outline:none}
.eyebrow{color:var(--accent);font-size:12px;letter-spacing:.16em;text-transform:uppercase}h1{font-size:clamp(34px,7vw,64px);line-height:1}.lede{color:var(--text);font-size:clamp(17px,2vw,21px);max-width:78ch}.section-heading p,.match summary span,.match-grid,.note{color:var(--text-muted)}
.cards{display:grid;gap:18px}.card-group{display:grid;gap:10px}.card-group h2{color:var(--accent);font-size:1.2rem}.card-row{display:grid;gap:14px;grid-template-columns:repeat(auto-fit,minmax(180px,1fr))}article,.pair-group,.match{background:var(--card);border:1px solid var(--border);display:grid;gap:10px;padding:16px}article:hover,.pair-group:hover,.match:hover{border-color:var(--teal)}article span{color:var(--text-muted);font-size:12px;letter-spacing:.1em;text-transform:uppercase}article strong{color:var(--green);font-size:clamp(18px,2vw,24px);line-height:1.18;word-break:break-word}
.term-grid{display:grid;gap:14px;grid-template-columns:repeat(auto-fit,minmax(260px,1fr))}.term{align-content:start}.term h3{color:var(--green);font-size:1rem;margin:0}.term p{color:var(--text-muted);margin:0}.term code{color:var(--accent)}
.section-heading{display:grid;gap:8px}.section-heading h2{color:var(--accent);font-size:1.2rem}.section-heading p{max-width:78ch}
table{border-collapse:collapse;min-width:820px;width:100%}th,td{border-bottom:1px solid var(--border);padding:9px 10px;text-align:right;white-space:nowrap}th:first-child,td:first-child{text-align:left}th{color:var(--text-muted);font-size:12px;letter-spacing:.08em;text-transform:uppercase}
.pair-tree,.match-list{display:grid;gap:12px}.pair-group,.match{padding:0}.pair-group summary,.match summary{cursor:pointer;display:grid;gap:12px;align-items:center;padding:12px 14px}.pair-group summary{grid-template-columns:1fr auto auto auto}.match summary{grid-template-columns:72px 1fr auto auto auto}.pair-group summary strong,.match summary strong{color:var(--text)}.pair-overview{display:grid;gap:12px;grid-template-columns:repeat(auto-fit,minmax(260px,1fr));padding:0 18px 16px}.pair-overview p{background:var(--surface-strong);border:1px solid var(--border);margin:0;padding:12px}.pair-group>.match-list{padding:8px 18px 18px}.match-grid{display:grid;gap:12px;grid-template-columns:1.4fr 1fr 1fr;padding:0 14px 14px}.match-grid p{margin:0;word-break:break-word}.pair-overview b,.match-grid b{color:var(--text)}.board-panel,.raw-data{grid-column:1/-1}.board-ascii,.raw-data{background:var(--surface-strong);border:1px solid var(--border)}.board-ascii{color:var(--text);font:14px/1.35 ui-monospace,SFMono-Regular,Menlo,Consolas,monospace;margin:8px 0 0;overflow:auto;padding:12px;white-space:pre}.raw-data{padding:10px}.raw-data summary{cursor:pointer;padding:0}.raw-data p{margin:8px 0 0}
.provenance dl{display:grid;gap:8px 18px;grid-template-columns:max-content 1fr;margin:0}.provenance dt{color:var(--text-muted);font-size:12px;letter-spacing:.08em;text-transform:uppercase}.provenance dd{margin:0}.command{background:var(--surface-strong);border:1px solid var(--border);margin:0;overflow:auto;padding:12px}
@media (max-width:760px){main{padding:20px}.pair-group summary,.match summary{grid-template-columns:1fr}.match-grid{grid-template-columns:1fr}.provenance dl{grid-template-columns:1fr}table{min-width:760px}}
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
    fn color_summary_includes_counts_and_precise_percentages() {
        let mut report = sample_report();
        report.color_splits = vec![ColorSplitReport {
            black: "fast".to_string(),
            white: "balanced".to_string(),
            black_wins: 105,
            white_wins: 84,
            draws: 3,
            total: 192,
        }];

        assert_eq!(
            color_summary(&report),
            "Black 105 (54.7%) / White 84 (43.8%) / Draw 3 (1.6%)"
        );
    }

    #[test]
    fn html_report_groups_matches_by_pair_and_demotes_raw_cells() {
        let report = sample_report();
        let html = render_tournament_report_html(&report);

        assert!(html.contains("<details class=\"pair-group\">"));
        assert!(html.contains("SearchBot_D2 vs SearchBot_D3"));
        assert!(html.contains("2 matches"));
        assert!(html.contains("<div class=\"pair-overview\">"));
        assert!(html.contains("<b>Pair result</b>"));
        assert!(html.contains("0-0-2 A-D-B; 0.0-2.0 points; SearchBot_D2 points rate 0.0%"));
        assert!(html.contains("<b>Color result</b>"));
        assert!(html.contains("SearchBot_D2: black 0.0% (0/1), white 0.0% (0/1)"));
        assert!(html.contains("SearchBot_D3: black 100.0% (1/1), white 100.0% (1/1)"));
        assert!(html.contains("#001</span><strong>SearchBot_D2 (B) vs SearchBot_D3 (W)</strong>"));
        assert!(html.contains("#002</span><strong>SearchBot_D2 (W) vs SearchBot_D3 (B)</strong>"));
        assert!(html.contains("<b>SearchBot_D2 (W) stats</b>"));
        assert!(html.contains("<b>SearchBot_D3 (B) stats</b>"));
        assert!(!html.contains("<h2>Pairwise</h2>"));
        assert!(!html.contains("<h2>Color Splits</h2>"));
        assert!(html.contains("<details class=\"raw-data\">"));

        let pair_pos = html.find("SearchBot_D2 vs SearchBot_D3").unwrap();
        let overview_pos = html.find("<div class=\"pair-overview\">").unwrap();
        let color_a_pos = html.find("SearchBot_D2: black").unwrap();
        let color_b_pos = html.find("SearchBot_D3: black").unwrap();
        let match_pos = html.find("#001").unwrap();
        let how_to_read_pos = html.find("<h2>How To Read This</h2>").unwrap();
        let provenance_pos = html.find("<h2>Provenance</h2>").unwrap();
        let match_body = &html[match_pos..];
        let board_pos = match_pos + match_body.find("Finished board").unwrap();
        let moves_pos = match_pos + match_body.find("<b>Moves</b>").unwrap();
        let raw_pos = match_pos + match_body.find("Raw data").unwrap();
        assert!(pair_pos < overview_pos);
        assert!(overview_pos < color_a_pos);
        assert!(color_a_pos < color_b_pos);
        assert!(color_b_pos < match_pos);
        assert!(overview_pos < match_pos);
        assert!(match_pos < board_pos);
        assert!(board_pos < moves_pos);
        assert!(moves_pos < raw_pos);
        assert!(raw_pos < how_to_read_pos);
        assert!(how_to_read_pos < provenance_pos);
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

        assert!(html.contains("<h2>Search Cost</h2>"));
        assert!(html.contains("SearchBot_D2"));
        assert!(html.contains("100.0"));
        assert!(html.contains("1.0 / 4.0"));
        assert!(html.contains("2.0 / 4.0"));
        assert!(html.contains("TT hit/cut"));
        assert!(html.contains("0.0 / 0.0"));
        assert!(html.contains("cand r/s 1.0 / 4.0"));
        assert!(html.contains("legal r/s 2.0 / 4.0"));
        assert!(html.contains("<h3>Search Cost</h3>"));
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
    fn searchbot_labels_keep_report_variants_distinct() {
        let report = sample_report();

        assert_eq!(compact_bot_label(&report, "search-d5"), "SearchBot_D5");
        assert_eq!(
            compact_bot_label(&report, "search-d5+tactical-cap-8"),
            "SearchBot_D5+TCap8"
        );
        assert_eq!(
            compact_bot_label(&report, "search-d5+tactical-cap-8+pattern-eval"),
            "SearchBot_D5+TCap8+Pattern"
        );
        assert_eq!(
            bot_label(&report, "search-d5+tactical-cap-8+pattern-eval"),
            "SearchBot @ depth 5 + tactical cap 8 + pattern eval"
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
              "safety_nodes": 10,
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
        assert_eq!(report.standings[0].eval_calls, 0);
        assert_eq!(report.standings[0].search_candidate_generations, 0);
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
            total_nodes: 1000,
            avg_nodes: 200.0,
            eval_calls: 500,
            avg_eval_calls: 100.0,
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
            tt_hits: 7,
            tt_cutoffs: 3,
            beta_cutoffs: 9,
            avg_depth: 3.0,
            max_depth: 3,
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
            total_nodes: 1000,
            avg_nodes: 200.0,
            eval_calls: 500,
            avg_eval_calls: 100.0,
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
            tt_hits: 7,
            tt_cutoffs: 3,
            beta_cutoffs: 9,
            depth_sum: 15,
            avg_depth: 3.0,
            max_depth: 3,
            budget_exhausted_count: 1,
            budget_exhausted_rate: 0.2,
        }
    }
}
