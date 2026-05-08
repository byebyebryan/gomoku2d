use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use gomoku_bot::{CorridorBot, RandomBot, SearchBot};
use gomoku_core::{Color, GameResult, Move, Replay, RuleConfig, Variant};
use gomoku_eval::analysis::{analyze_replay, AnalysisOptions, DEFAULT_MAX_SCAN_PLIES};
use gomoku_eval::analysis_batch::{
    render_analysis_batch_report_html, run_analysis_batch_replays_with_options,
    run_analysis_batch_with_options, AnalysisBatchReport, AnalysisBatchRunOptions,
    ReplayAnalysisInput,
};
use gomoku_eval::analysis_fixture::{
    render_analysis_fixture_report_html, run_analysis_fixtures, AnalysisFixtureReport,
    AnalysisFixtureResult,
};
use gomoku_eval::analysis_report::{report_match_to_replay, select_report_matches};
use gomoku_eval::arena::{run_match_series_with_limits, MatchEndReason, MatchLimits, MatchResult};
use gomoku_eval::opening::{OpeningPolicy, CENTERED_SUITE_MAX_PLIES};
use gomoku_eval::report::{
    render_tournament_report_html_with_options, AnchorReferenceReport, ReportRenderOptions,
    TournamentReport, TournamentRunReport,
};
use gomoku_eval::scenario::{
    run_tactical_scenarios, ScenarioSearchConfig, TacticalScenarioGroupSummary,
    TacticalScenarioReport, TacticalScenarioResult, TACTICAL_SCENARIO_CASES,
};
use gomoku_eval::seed::derive_seed;
use gomoku_eval::tournament::{
    default_thread_count, round_robin_pairs, run_scheduled_pairs_parallel, TournamentBotFactory,
    TournamentOptions, TournamentPair,
};

#[path = "../../benchmarks/search_configs.rs"]
mod search_configs;

#[derive(Parser, Debug)]
#[command(name = "gomoku-eval", about = "Evaluation harness for Gomoku bots")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
#[clap(rename_all = "kebab-case")]
enum CliOpeningPolicy {
    CenteredSuite,
    RandomLegal,
}

impl From<CliOpeningPolicy> for OpeningPolicy {
    fn from(value: CliOpeningPolicy) -> Self {
        match value {
            CliOpeningPolicy::CenteredSuite => OpeningPolicy::CenteredSuite,
            CliOpeningPolicy::RandomLegal => OpeningPolicy::RandomLegal,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[clap(rename_all = "kebab-case")]
enum CliTournamentSchedule {
    RoundRobin,
    HeadToHead,
    Gauntlet,
}

impl CliTournamentSchedule {
    fn label(self) -> &'static str {
        match self {
            CliTournamentSchedule::RoundRobin => "round-robin",
            CliTournamentSchedule::HeadToHead => "head-to-head",
            CliTournamentSchedule::Gauntlet => "gauntlet",
        }
    }
}

#[derive(Args, Debug, Clone)]
struct EvalOptions {
    /// Rule variant: "renju" (default) or "freestyle"
    #[arg(long, default_value = "renju")]
    rule: String,

    /// Per-move search budget for search bots, in milliseconds
    #[arg(long)]
    search_time_ms: Option<u64>,

    /// Per-move Linux thread CPU-time budget for search bots, in milliseconds
    #[arg(long)]
    search_cpu_time_ms: Option<u64>,

    /// Stop a game after this many moves and record it as a draw
    #[arg(long)]
    max_moves: Option<usize>,

    /// Stop a game after this wall-clock duration and record it as a draw
    #[arg(long)]
    max_game_ms: Option<u64>,

    /// Base seed for reproducible random bots and tournament openings
    #[arg(long, default_value_t = 0)]
    seed: u64,
}

struct EvalContext {
    config: RuleConfig,
    rule_label: &'static str,
    limits: MatchLimits,
    search_time_ms: Option<u64>,
    search_cpu_time_ms: Option<u64>,
    seed: u64,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run N games between two different bots
    Versus {
        #[command(flatten)]
        options: EvalOptions,

        #[arg(long, default_value = "baseline")]
        bot_a: String,

        #[arg(long, default_value = "random")]
        bot_b: String,

        #[arg(long, default_value_t = 10)]
        games: u32,

        #[arg(long)]
        replay_dir: Option<PathBuf>,
    },
    /// Run N games of a single bot playing against itself
    SelfPlay {
        #[command(flatten)]
        options: EvalOptions,

        #[arg(long, default_value = "baseline")]
        bot: String,

        #[arg(long, default_value_t = 10)]
        games: u32,

        #[arg(long)]
        replay_dir: Option<PathBuf>,
    },
    /// Render a saved tournament JSON report to standalone HTML
    ReportHtml {
        #[arg(long)]
        input: PathBuf,

        #[arg(long)]
        output: PathBuf,

        /// Link to the raw JSON from the rendered HTML; defaults to the input file name
        #[arg(long)]
        json_href: Option<String>,
    },
    /// Run a round-robin tournament among a list of bots
    Tournament {
        #[command(flatten)]
        options: EvalOptions,

        /// Pairing workflow used for tournament jobs
        #[arg(long, value_enum, default_value = "round-robin")]
        schedule: CliTournamentSchedule,

        /// Comma-separated list of bots (e.g. "random,search-d1,search-d3,search-d5+tactical-cap-8")
        #[arg(long)]
        bots: Option<String>,

        /// Candidate bot for gauntlet mode
        #[arg(long)]
        candidate: Option<String>,

        /// Comma-separated candidate bots for batch gauntlet mode
        #[arg(long)]
        candidates: Option<String>,

        /// Comma-separated anchor bots for gauntlet mode
        #[arg(long)]
        anchors: Option<String>,

        /// Full tournament report used as the reference anchor source for gauntlet mode
        #[arg(long)]
        anchor_report: Option<PathBuf>,

        /// Number of games each pair plays
        #[arg(long, default_value_t = 2)]
        games_per_pair: u32,

        #[arg(long)]
        replay_dir: Option<PathBuf>,

        /// Write reusable tournament report JSON
        #[arg(long)]
        report_json: Option<PathBuf>,

        /// Number of opening plies before bots take over
        #[arg(long, default_value_t = 4)]
        opening_plies: usize,

        /// Opening policy used before bots take over
        #[arg(long, value_enum, default_value = "centered-suite")]
        opening_policy: CliOpeningPolicy,

        /// Worker threads used to run tournament games
        #[arg(long)]
        threads: Option<usize>,
    },
    /// Run focused one-move tactical diagnostics against search configs
    TacticalScenarios {
        /// Comma-separated search configs (e.g. "search-d2,search-d3,search-d5")
        #[arg(long, default_value = "search-d2,search-d3,search-d5")]
        bots: String,

        /// Per-move search budget for search bots, in milliseconds
        #[arg(long)]
        search_time_ms: Option<u64>,

        /// Per-move Linux thread CPU-time budget for search bots, in milliseconds
        #[arg(long)]
        search_cpu_time_ms: Option<u64>,

        /// Write reusable tactical scenario report JSON
        #[arg(long)]
        report_json: Option<PathBuf>,
    },
    /// Analyze a saved replay and emit bounded proof/classification JSON
    AnalyzeReplay {
        /// Replay JSON to analyze
        #[arg(long)]
        input: PathBuf,

        /// Write analysis JSON to this path instead of stdout
        #[arg(long)]
        output: Option<PathBuf>,

        /// Maximum forced-corridor proof depth
        #[arg(long, default_value_t = 4)]
        max_depth: usize,

        /// Max plies to scan backward from the final board
        #[arg(long, default_value_t = DEFAULT_MAX_SCAN_PLIES)]
        max_scan_plies: usize,
    },
    /// Analyze every replay JSON in a directory and emit grouped reports
    AnalyzeReplayBatch {
        /// Directory containing replay JSON files
        #[arg(long)]
        replay_dir: PathBuf,

        /// Write reusable batch report JSON
        #[arg(long)]
        report_json: Option<PathBuf>,

        /// Write standalone batch report HTML
        #[arg(long)]
        report_html: Option<PathBuf>,

        /// Maximum forced-corridor proof depth
        #[arg(long, default_value_t = 4)]
        max_depth: usize,

        /// Max plies to scan backward from the final board
        #[arg(long, default_value_t = DEFAULT_MAX_SCAN_PLIES)]
        max_scan_plies: usize,

        /// Include proof snapshots for decisive replay analyses
        #[arg(long)]
        include_proof_details: bool,
    },
    /// Analyze sampled replays embedded in a tournament report
    AnalyzeReportReplays {
        /// Tournament report JSON containing compact match move cells
        #[arg(long)]
        report: PathBuf,

        /// First entrant in the head-to-head matchup; defaults to standing #1
        #[arg(long)]
        entrant_a: Option<String>,

        /// Second entrant in the head-to-head matchup; defaults to standing #2
        #[arg(long)]
        entrant_b: Option<String>,

        /// Number of head-to-head games to sample before analysis
        #[arg(long, default_value_t = 8)]
        sample_size: usize,

        /// Write reusable batch report JSON
        #[arg(long)]
        report_json: Option<PathBuf>,

        /// Write standalone batch report HTML
        #[arg(long)]
        report_html: Option<PathBuf>,

        /// Maximum forced-corridor proof depth
        #[arg(long, default_value_t = 4)]
        max_depth: usize,

        /// Max plies to scan backward from the final board
        #[arg(long, default_value_t = DEFAULT_MAX_SCAN_PLIES)]
        max_scan_plies: usize,

        /// Include proof snapshots for decisive replay analyses
        #[arg(long)]
        include_proof_details: bool,
    },
    /// Run curated replay-analysis fixtures and emit expected-vs-actual labels
    AnalysisFixtures {
        /// Write reusable fixture report JSON
        #[arg(long)]
        report_json: Option<PathBuf>,

        /// Write standalone fixture report HTML
        #[arg(long)]
        report_html: Option<PathBuf>,

        /// Maximum forced-corridor proof depth
        #[arg(long, default_value_t = 4)]
        max_depth: usize,

        /// Max plies to scan backward from the final board unless a fixture overrides it
        #[arg(long, default_value_t = DEFAULT_MAX_SCAN_PLIES)]
        max_scan_plies: usize,
    },
}

type BotFactory = TournamentBotFactory;
type NamedBotFactory = (String, BotFactory);

#[derive(Debug, Clone, PartialEq, Eq)]
struct TournamentPlan {
    bot_names: Vec<String>,
    anchor_names: Vec<String>,
    anchor_report: Option<String>,
    pairs: Vec<TournamentPair>,
}

fn tournament_plan(
    schedule: CliTournamentSchedule,
    bots: Option<&str>,
    candidate: Option<&str>,
    candidates: Option<&str>,
    anchors: Option<&str>,
    anchor_report: Option<&str>,
) -> Result<TournamentPlan, String> {
    match schedule {
        CliTournamentSchedule::RoundRobin => {
            reject_anchor_report_args(schedule, anchor_report)?;
            reject_gauntlet_args(schedule, candidate, candidates, anchors)?;
            let bot_names =
                parse_required_bot_list(bots, "Round-robin tournament requires --bots.")?;
            if bot_names.len() < 2 {
                return Err("Round-robin tournament requires at least 2 bots.".to_string());
            }
            validate_unique_bot_names(&bot_names)?;
            let pairs = round_robin_pairs(bot_names.len());
            Ok(TournamentPlan {
                bot_names,
                anchor_names: Vec::new(),
                anchor_report: None,
                pairs,
            })
        }
        CliTournamentSchedule::HeadToHead => {
            reject_anchor_report_args(schedule, anchor_report)?;
            reject_gauntlet_args(schedule, candidate, candidates, anchors)?;
            let bot_names =
                parse_required_bot_list(bots, "Head-to-head tournament requires --bots.")?;
            if bot_names.len() != 2 {
                return Err("Head-to-head tournament requires exactly 2 bots.".to_string());
            }
            validate_unique_bot_names(&bot_names)?;
            Ok(TournamentPlan {
                bot_names,
                anchor_names: Vec::new(),
                anchor_report: None,
                pairs: vec![TournamentPair {
                    bot_a_idx: 0,
                    bot_b_idx: 1,
                }],
            })
        }
        CliTournamentSchedule::Gauntlet => {
            if bots.is_some() {
                return Err(
                    "Gauntlet tournament uses --candidate/--candidates and --anchors instead of --bots."
                        .to_string(),
                );
            }
            let candidate_names = parse_gauntlet_candidates(candidate, candidates)?;
            let anchor_names =
                parse_required_bot_list(anchors, "Gauntlet tournament requires --anchors.")?;
            if anchor_names.is_empty() {
                return Err("Gauntlet tournament requires at least 1 anchor.".to_string());
            }

            let candidate_count = candidate_names.len();
            let mut bot_names = candidate_names;
            bot_names.extend(anchor_names.clone());
            validate_unique_bot_names(&bot_names)?;
            let pairs = (0..candidate_count)
                .flat_map(|candidate_idx| {
                    (candidate_count..bot_names.len()).map(move |anchor_idx| TournamentPair {
                        bot_a_idx: candidate_idx,
                        bot_b_idx: anchor_idx,
                    })
                })
                .collect();
            Ok(TournamentPlan {
                bot_names,
                anchor_names,
                anchor_report: anchor_report.map(ToString::to_string),
                pairs,
            })
        }
    }
}

fn parse_required_bot_list(input: Option<&str>, message: &str) -> Result<Vec<String>, String> {
    let Some(input) = input else {
        return Err(message.to_string());
    };
    let bot_names = parse_bot_list(input);
    if bot_names.is_empty() {
        return Err(message.to_string());
    }
    Ok(bot_names)
}

fn parse_gauntlet_candidates(
    candidate: Option<&str>,
    candidates: Option<&str>,
) -> Result<Vec<String>, String> {
    match (candidate, candidates) {
        (Some(_), Some(_)) => Err(
            "Gauntlet tournament uses either --candidate or --candidates, not both.".to_string(),
        ),
        (Some(candidate), None) => {
            let candidate_names = parse_required_bot_list(
                Some(candidate),
                "Gauntlet tournament requires --candidate or --candidates.",
            )?;
            if candidate_names.len() != 1 {
                return Err(
                    "Gauntlet --candidate accepts exactly 1 bot; use --candidates for batch gauntlets."
                        .to_string(),
                );
            }
            Ok(candidate_names)
        }
        (None, Some(candidates)) => parse_required_bot_list(
            Some(candidates),
            "Gauntlet tournament requires --candidate or --candidates.",
        ),
        (None, None) => {
            Err("Gauntlet tournament requires --candidate or --candidates.".to_string())
        }
    }
}

fn parse_bot_list(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn reject_gauntlet_args(
    schedule: CliTournamentSchedule,
    candidate: Option<&str>,
    candidates: Option<&str>,
    anchors: Option<&str>,
) -> Result<(), String> {
    if candidate.is_some() || candidates.is_some() || anchors.is_some() {
        return Err(format!(
            "{} tournament uses --bots, not --candidate/--candidates/--anchors.",
            schedule.label()
        ));
    }
    Ok(())
}

fn reject_anchor_report_args(
    schedule: CliTournamentSchedule,
    anchor_report: Option<&str>,
) -> Result<(), String> {
    if anchor_report.is_some() {
        return Err(format!(
            "{} tournament does not use --anchor-report.",
            schedule.label()
        ));
    }
    Ok(())
}

fn validate_unique_bot_names(bot_names: &[String]) -> Result<(), String> {
    for (idx, name) in bot_names.iter().enumerate() {
        if bot_names.iter().skip(idx + 1).any(|other| other == name) {
            return Err(format!("Duplicate bot in tournament schedule: {name}"));
        }
    }
    Ok(())
}

fn load_anchor_reference(
    path: &PathBuf,
    source_path: String,
    anchor_names: &[String],
) -> Result<AnchorReferenceReport, String> {
    let json = std::fs::read_to_string(path)
        .map_err(|err| format!("Failed to read anchor report {}: {err}", path.display()))?;
    let source_report = TournamentReport::from_json(&json)
        .map_err(|err| format!("Failed to parse anchor report {}: {err}", path.display()))?;
    AnchorReferenceReport::from_report(Some(source_path), &source_report, anchor_names)
}

fn make_bot_factory(
    spec: &str,
    search_time_ms: Option<u64>,
    search_cpu_time_ms: Option<u64>,
) -> Result<BotFactory, String> {
    let spec = spec.to_string();
    if spec == "random" {
        return Ok(Arc::new(|seed| Box::new(RandomBot::seeded(seed))));
    }
    if spec == "corridor-random" {
        return Ok(Arc::new(move |seed| {
            Box::new(CorridorBot::with_random_fallback(seed))
        }));
    }
    if spec == "corridor-d1" {
        let fallback_config = search_configs::search_config_from_lab_spec(
            "search-d1",
            5,
            search_time_ms,
            search_cpu_time_ms,
        )
        .expect("search-d1 fallback spec should parse");
        return Ok(Arc::new(move |seed| {
            Box::new(CorridorBot::with_search_fallback_config(
                seed,
                fallback_config,
            ))
        }));
    }

    if let Some(config) =
        search_configs::search_config_from_lab_spec(&spec, 5, search_time_ms, search_cpu_time_ms)
    {
        return Ok(Arc::new(move |_| Box::new(SearchBot::with_config(config))));
    }

    Err(format!(
        "Unknown bot type: '{spec}'. Use random, corridor-random, corridor-d1, baseline-N, search-dN, or search-dN+suffixes."
    ))
}

fn exit_with_error(message: impl AsRef<str>) -> ! {
    eprintln!("{}", message.as_ref());
    std::process::exit(2);
}

fn variant_label(variant: &Variant) -> &'static str {
    match variant {
        Variant::Freestyle => "freestyle",
        Variant::Renju => "renju",
    }
}

fn eval_context(options: &EvalOptions) -> EvalContext {
    let variant = match options.rule.as_str() {
        "renju" => Variant::Renju,
        "freestyle" => Variant::Freestyle,
        other => exit_with_error(format!(
            "Unknown rule variant '{other}'. Use 'renju' or 'freestyle'."
        )),
    };
    let rule_label = variant_label(&variant);
    EvalContext {
        config: RuleConfig {
            variant,
            ..Default::default()
        },
        rule_label,
        limits: MatchLimits {
            max_moves: options.max_moves,
            max_game_ms: options.max_game_ms,
        },
        search_time_ms: options.search_time_ms,
        search_cpu_time_ms: options.search_cpu_time_ms,
        seed: options.seed,
    }
}

fn print_move_progress(move_num: usize, game_idx: u32, player: Color, mv: Move, time_ms: u64) {
    let time_str = if time_ms >= 1000 {
        format!("{:.1}s", time_ms as f64 / 1000.0)
    } else {
        format!("{}ms", time_ms)
    };
    println!(
        "  Game {:3}  move {:3}  {} {:3}  ({})",
        game_idx + 1,
        move_num,
        match player {
            Color::Black => "B",
            Color::White => "W",
        },
        mv.to_notation(),
        time_str,
    );
}

fn end_reason_suffix(reason: MatchEndReason) -> String {
    match reason {
        MatchEndReason::Natural => String::new(),
        reason => format!(" ({})", reason.label()),
    }
}

fn print_game_result(i: u32, total: u32, mr: &MatchResult) {
    let suffix = end_reason_suffix(mr.end_reason);
    match &mr.result {
        GameResult::Winner(c) => println!("  Game {:3}/{:3}  {:?} wins{}", i + 1, total, c, suffix),
        GameResult::Draw => println!("  Game {:3}/{:3}  Draw{}", i + 1, total, suffix),
        GameResult::Ongoing => unreachable!(),
    }
}

fn parse_search_config_specs(
    specs: &str,
    search_time_ms: Option<u64>,
    search_cpu_time_ms: Option<u64>,
) -> Result<Vec<ScenarioSearchConfig>, String> {
    let names: Vec<String> = specs
        .split(',')
        .map(|spec| spec.trim().to_string())
        .filter(|spec| !spec.is_empty())
        .collect();

    if names.is_empty() {
        return Err("At least one search config is required.".to_string());
    }

    names
        .into_iter()
        .map(|name| {
            let config = search_configs::search_config_from_lab_spec(
                &name,
                5,
                search_time_ms,
                search_cpu_time_ms,
            )
            .ok_or_else(|| {
                format!("Unknown search config: '{name}'. Use search-dN or search-dN+suffixes.")
            })?;
            Ok(ScenarioSearchConfig { id: name, config })
        })
        .collect()
}

fn print_tactical_scenario_result(result: &TacticalScenarioResult) {
    let status = if result.passed { "PASS" } else { "FAIL" };
    let expected = if result.expected_moves.is_empty() {
        "-".to_string()
    } else {
        result.expected_moves.join("/")
    };
    let shape = result.shape.unwrap_or("-");
    println!(
        "{:<5} {:<10} {:<16} {:<8} {:<13} {:<12} {:?}/{:?} {:<48} actual {:<3} expect {:<7} depth {:>2} nodes {:>8} safety {:>5} eval {:>7} cand r/s {:>5}/{:<5} child {:>5}->{:<5} cap {:>4} legal r/s {:>6}/{:<6} tt {:>5}/{:<5} cut {:>5} time {:>4}ms",
        status,
        result.config_id,
        result.role,
        result.layer,
        result.intent,
        shape,
        result.variant,
        result.to_move,
        result.case_id,
        result.actual_move,
        expected,
        result.metrics.depth_reached,
        result.metrics.nodes,
        result.metrics.safety_nodes,
        result.metrics.eval_calls,
        result.metrics.root_candidate_generations,
        result.metrics.search_candidate_generations,
        result.metrics.child_moves_before_total,
        result.metrics.child_moves_after_total,
        result.metrics.child_cap_hits,
        result.metrics.root_legality_checks,
        result.metrics.search_legality_checks,
        result.metrics.tt_hits,
        result.metrics.tt_cutoffs,
        result.metrics.beta_cutoffs,
        result.metrics.time_ms
    );
}

fn print_tactical_group_summary(title: &str, summaries: &[TacticalScenarioGroupSummary]) {
    println!("\n{title}");
    for summary in summaries {
        println!(
            "  {:<16} {:>3}/{:<3} passed, {:>3} failed, avg depth {:>4.1}, avg total nodes {:>8.0}, avg safety {:>7.0}, avg time {:>5.1}ms",
            summary.key,
            summary.passed,
            summary.total,
            summary.failed,
            summary.avg_depth_reached,
            summary.avg_total_nodes,
            summary.avg_safety_nodes,
            summary.avg_time_ms
        );
    }
}

fn print_tactical_report_summary(report: &TacticalScenarioReport) {
    println!(
        "\n--- Summary ---\n{} passed / {} total ({} failed)",
        report.passed, report.total, report.failed
    );
    print_tactical_group_summary("By role", &report.role_summaries);
    print_tactical_group_summary("By layer", &report.layer_summaries);
    print_tactical_group_summary("By intent", &report.intent_summaries);
}

fn print_analysis_fixture_result(result: &AnalysisFixtureResult) {
    let status = if result.passed { "PASS" } else { "FAIL" };
    let notes = if result.actual.tactical_notes.is_empty() {
        "-".to_string()
    } else {
        result
            .actual
            .tactical_notes
            .iter()
            .map(|note| format!("{note:?}"))
            .collect::<Vec<_>>()
            .join("/")
    };
    println!(
        "{:<5} {:<34} winner {:<7} root {:<14} forced {:>2}..{:<2} chance {:<4} critical {:<4} notes {}",
        status,
        result.case_id,
        result
            .actual
            .winner
            .map(|winner| format!("{winner:?}"))
            .unwrap_or_else(|| "-".to_string()),
        format!("{:?}", result.actual.root_cause),
        result.actual.final_forced_interval.start_ply,
        result.actual.final_forced_interval.end_ply,
        result
            .actual
            .last_chance_ply
            .map(|ply| ply.to_string())
            .unwrap_or_else(|| "-".to_string()),
        result
            .actual
            .critical_mistake_ply
            .map(|ply| ply.to_string())
            .unwrap_or_else(|| "-".to_string()),
        notes
    );
    for failure in &result.failures {
        println!("      {failure}");
    }
}

fn print_analysis_fixture_report_summary(report: &AnalysisFixtureReport) {
    println!(
        "\n--- Summary ---\n{} passed / {} total ({} failed)",
        report.passed, report.total, report.failed
    );
}

fn print_analysis_batch_report_summary(report: &AnalysisBatchReport) {
    println!(
        "\n--- Summary ---\n{} analyzed / {} total ({} failed)",
        report.analyzed, report.total, report.failed
    );
    println!(
        "loss: mistake {}, tactical error {}, strategic {}, unclear {}, ongoing/draw {}, errors {}",
        report.summary.mistake,
        report.summary.tactical_error,
        report.summary.strategic_loss,
        report.summary.unclear,
        report.summary.ongoing_or_draw,
        report.summary.analysis_error
    );
    println!(
        "cause: missed defense {}, missed win {}",
        report.summary.missed_defense, report.summary.missed_win
    );
}

fn resolve_report_replay_entrants(
    standing_bots: &[String],
    entrant_a: Option<String>,
    entrant_b: Option<String>,
) -> Result<(String, String), String> {
    match (entrant_a, entrant_b) {
        (Some(a), Some(b)) if a == b => Err("report replay entrants must be different".to_string()),
        (Some(a), Some(b)) => Ok((a, b)),
        (Some(a), None) => {
            let b = highest_different_standing(standing_bots, &a)
                .ok_or_else(|| format!("Tournament report has no standing different from {a}."))?;
            Ok((a, b))
        }
        (None, Some(b)) => {
            let a = highest_different_standing(standing_bots, &b)
                .ok_or_else(|| format!("Tournament report has no standing different from {b}."))?;
            Ok((a, b))
        }
        (None, None) => {
            let a = standing_bots
                .first()
                .cloned()
                .ok_or_else(|| "Tournament report has no standing #1.".to_string())?;
            let b = highest_different_standing(standing_bots, &a)
                .ok_or_else(|| "Tournament report has no standing #2.".to_string())?;
            Ok((a, b))
        }
    }
}

fn highest_different_standing(standing_bots: &[String], entrant: &str) -> Option<String> {
    standing_bots
        .iter()
        .find(|bot| bot.as_str() != entrant)
        .cloned()
}

fn report_replay_source_label(
    report: &Path,
    entrant_a: &str,
    entrant_b: &str,
    default_top_two: bool,
) -> String {
    let selector = if default_top_two {
        "Top 2 entrants".to_string()
    } else {
        format!("{entrant_a} vs {entrant_b}")
    };
    format!("{}:{selector}", report.display())
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Versus {
            options,
            bot_a,
            bot_b,
            games,
            replay_dir,
        } => {
            let EvalContext {
                config,
                rule_label,
                limits,
                search_time_ms,
                search_cpu_time_ms,
                seed,
            } = eval_context(&options);
            println!("--- Versus: {} vs {} ({} games) ---", bot_a, bot_b, games);
            println!("Rule: {rule_label}");
            if let Some(ms) = search_time_ms {
                println!("Search time budget: {ms} ms/move");
            }
            if let Some(ms) = search_cpu_time_ms {
                println!("Search CPU-time budget: {ms} ms/move");
            }

            if let Some(dir) = &replay_dir {
                std::fs::create_dir_all(dir).unwrap();
            }

            let bot_a_factory = match make_bot_factory(&bot_a, search_time_ms, search_cpu_time_ms) {
                Ok(factory) => factory,
                Err(err) => exit_with_error(err),
            };
            let bot_b_factory = match make_bot_factory(&bot_b, search_time_ms, search_cpu_time_ms) {
                Ok(factory) => factory,
                Err(err) => exit_with_error(err),
            };
            let mut game_seed_idx = 0u64;
            let make_bots = || {
                let current_seed_idx = game_seed_idx;
                game_seed_idx += 1;
                let a = bot_a_factory(derive_seed(seed, [current_seed_idx, 0]));
                let b = bot_b_factory(derive_seed(seed, [current_seed_idx, 1]));
                (a, b)
            };

            let stats = run_match_series_with_limits(
                make_bots,
                games,
                config,
                limits,
                print_move_progress,
                |i, mr| {
                    print_game_result(i, games, mr);
                    if let Some(dir) = &replay_dir {
                        let path = dir.join(format!("match_{:03}.json", i + 1));
                        if let Ok(json) = mr.replay.to_json() {
                            std::fs::write(&path, &json)
                                .unwrap_or_else(|e| eprintln!("Failed to write replay: {e}"));
                        }
                    }
                },
            );

            println!("\n--- Results ---");
            println!("{} wins: {}", bot_a, stats.bot_a_wins);
            println!("{} wins: {}", bot_b, stats.bot_b_wins);
            println!("Draws: {}", stats.draws);

            let a_avg_time = if stats.bot_a_moves > 0 {
                stats.bot_a_time_ms as f64 / stats.bot_a_moves as f64
            } else {
                0.0
            };
            let b_avg_time = if stats.bot_b_moves > 0 {
                stats.bot_b_time_ms as f64 / stats.bot_b_moves as f64
            } else {
                0.0
            };

            println!("\nAvg time per move:");
            println!("{}: {:.2} ms", bot_a, a_avg_time);
            println!("{}: {:.2} ms", bot_b, b_avg_time);
        }
        Commands::SelfPlay {
            options,
            bot,
            games,
            replay_dir,
        } => {
            let EvalContext {
                config,
                rule_label,
                limits,
                search_time_ms,
                search_cpu_time_ms,
                seed,
            } = eval_context(&options);
            println!("--- Self-Play: {} vs {} ({} games) ---", bot, bot, games);
            println!("Rule: {rule_label}");
            if let Some(ms) = search_time_ms {
                println!("Search time budget: {ms} ms/move");
            }
            if let Some(ms) = search_cpu_time_ms {
                println!("Search CPU-time budget: {ms} ms/move");
            }

            if let Some(dir) = &replay_dir {
                std::fs::create_dir_all(dir).unwrap();
            }

            let bot_factory = match make_bot_factory(&bot, search_time_ms, search_cpu_time_ms) {
                Ok(factory) => factory,
                Err(err) => exit_with_error(err),
            };
            let mut game_seed_idx = 0u64;
            let make_bots = move || {
                let current_seed_idx = game_seed_idx;
                game_seed_idx += 1;
                let a = bot_factory(derive_seed(seed, [current_seed_idx, 0]));
                let b = bot_factory(derive_seed(seed, [current_seed_idx, 1]));
                (a, b)
            };

            let stats = run_match_series_with_limits(
                make_bots,
                games,
                config,
                limits,
                print_move_progress,
                |i, mr| {
                    print_game_result(i, games, mr);
                    if let Some(dir) = &replay_dir {
                        let path = dir.join(format!("selfplay_{:03}.json", i + 1));
                        if let Ok(json) = mr.replay.to_json() {
                            std::fs::write(&path, &json)
                                .unwrap_or_else(|e| eprintln!("Failed to write replay: {e}"));
                        }
                    }
                },
            );

            let black_wins = stats.bot_a_wins;
            let white_wins = stats.bot_b_wins;

            println!("\n--- Results ---");
            println!("Black wins: {}", black_wins);
            println!("White wins: {}", white_wins);
            println!("Draws: {}", stats.draws);

            let avg_time = if stats.bot_a_moves + stats.bot_b_moves > 0 {
                (stats.bot_a_time_ms + stats.bot_b_time_ms) as f64
                    / (stats.bot_a_moves + stats.bot_b_moves) as f64
            } else {
                0.0
            };

            println!("\nAvg time per move: {:.2} ms", avg_time);
        }
        Commands::Tournament {
            options,
            schedule,
            bots,
            candidate,
            candidates,
            anchors,
            anchor_report,
            games_per_pair,
            replay_dir,
            report_json,
            opening_plies,
            opening_policy,
            threads,
        } => {
            let EvalContext {
                config,
                rule_label,
                limits,
                search_time_ms,
                search_cpu_time_ms,
                seed,
            } = eval_context(&options);
            let anchor_report_display = anchor_report
                .as_ref()
                .map(|path| path.to_string_lossy().into_owned());
            let TournamentPlan {
                bot_names,
                anchor_names,
                anchor_report: planned_anchor_report,
                pairs,
            } = tournament_plan(
                schedule,
                bots.as_deref(),
                candidate.as_deref(),
                candidates.as_deref(),
                anchors.as_deref(),
                anchor_report_display.as_deref(),
            )
            .unwrap_or_else(|err| exit_with_error(err));
            if games_per_pair % 2 != 0 {
                eprintln!(
                    "Warning: odd games-per-pair leaves each pair with uneven color coverage."
                );
            }
            let opening_policy: OpeningPolicy = opening_policy.into();
            if opening_policy == OpeningPolicy::CenteredSuite
                && opening_plies > CENTERED_SUITE_MAX_PLIES
            {
                exit_with_error(format!(
                    "centered-suite openings support at most {CENTERED_SUITE_MAX_PLIES} plies"
                ));
            }

            println!("--- Tournament ---");
            println!("Schedule: {}", schedule.label());
            println!("Bots: {:?}", bot_names);
            println!("Rule: {rule_label}");
            println!("Pairings: {}", pairs.len());
            println!("Games per pair: {}", games_per_pair);
            println!("Seed: {}", seed);
            println!(
                "Opening: {}, {} plies",
                opening_policy.label(),
                opening_plies
            );
            let threads = threads.unwrap_or_else(default_thread_count);
            println!("Threads: {}", threads);
            if let Some(ms) = search_time_ms {
                println!("Search time budget: {ms} ms/move");
            }
            if let Some(ms) = search_cpu_time_ms {
                println!("Search CPU-time budget: {ms} ms/move");
            }
            if let Some(max_moves) = limits.max_moves {
                println!("Max moves: {max_moves}");
            }
            if let Some(max_game_ms) = limits.max_game_ms {
                println!("Max game time: {max_game_ms} ms");
            }
            let mut run_report = TournamentRunReport {
                bots: bot_names.clone(),
                schedule: schedule.label().to_string(),
                rules: config.clone(),
                games_per_pair,
                seed,
                opening_plies,
                opening_policy: opening_policy.label().to_string(),
                threads,
                search_time_ms,
                search_cpu_time_ms,
                max_moves: limits.max_moves,
                max_game_ms: limits.max_game_ms,
                total_wall_time_ms: None,
            };
            let reference_anchors = match (&anchor_report, planned_anchor_report) {
                (Some(path), Some(source_path)) => {
                    println!("Anchor report: {source_path}");
                    let reference = load_anchor_reference(path, source_path, &anchor_names)
                        .unwrap_or_else(|err| exit_with_error(err));
                    reference
                        .validate_compatible_run(&run_report)
                        .unwrap_or_else(|err| exit_with_error(err));
                    Some(reference)
                }
                _ => None,
            };
            println!();

            if let Some(dir) = &replay_dir {
                std::fs::create_dir_all(dir).unwrap();
            }

            let mut factories: Vec<NamedBotFactory> = vec![];
            for name in &bot_names {
                match make_bot_factory(name, search_time_ms, search_cpu_time_ms) {
                    Ok(factory) => factories.push((name.clone(), factory)),
                    Err(err) => exit_with_error(err),
                }
            }

            let mut match_idx = 0;
            let tournament_start = Instant::now();
            let results = run_scheduled_pairs_parallel(
                &factories,
                &pairs,
                games_per_pair,
                config.clone(),
                TournamentOptions {
                    limits,
                    seed,
                    opening_plies,
                    opening_policy,
                    threads,
                },
                |black_name, white_name, mr| {
                    match_idx += 1;
                    print!(
                        "Match {:3}: {} (B) vs {} (W) - ",
                        match_idx, black_name, white_name
                    );
                    let suffix = end_reason_suffix(mr.end_reason);
                    match mr.result {
                        GameResult::Winner(Color::Black) => {
                            println!("{} wins{}", black_name, suffix)
                        }
                        GameResult::Winner(Color::White) => {
                            println!("{} wins{}", white_name, suffix)
                        }
                        GameResult::Draw => println!("Draw{}", suffix),
                        GameResult::Ongoing => unreachable!(),
                    }

                    if let Some(dir) = &replay_dir {
                        let path = dir.join(format!("match_{:03}.json", match_idx));
                        if let Ok(json) = mr.replay.to_json() {
                            std::fs::write(&path, &json)
                                .unwrap_or_else(|e| eprintln!("Failed to write replay: {e}"));
                        }
                    }
                },
            );
            let total_wall_time_ms = Some(tournament_start.elapsed().as_millis() as u64);
            run_report.total_wall_time_ms = total_wall_time_ms;

            let mut report = match TournamentReport::from_results(run_report, &results) {
                Ok(report) => report,
                Err(err) => exit_with_error(format!("Failed to build tournament report: {err}")),
            };
            report.reference_anchors = reference_anchors;

            if let Some(path) = &report_json {
                let json = report.to_json().unwrap_or_else(|err| {
                    exit_with_error(format!("Failed to serialize report: {err}"))
                });
                std::fs::write(path, json).unwrap_or_else(|err| {
                    exit_with_error(format!("Failed to write report: {err}"))
                });
                println!("\nReport JSON: {}", path.display());
            }

            println!("\n--- Standings ---");
            for row in &report.standings {
                println!(
                    "{:<15} | Seq: {:>6.1} | Avg: {:>6.1} | W: {:>3} | D: {:>3} | L: {:>3} | ms: {:>7.2} | nodes: {:>9.0} | depth: {:>4.2} | budget: {:>5.1}%",
                    row.bot,
                    row.sequential_elo,
                    row.shuffled_elo_avg,
                    row.wins,
                    row.draws,
                    row.losses,
                    row.avg_search_time_ms,
                    row.avg_nodes,
                    row.avg_depth,
                    row.budget_exhausted_rate * 100.0
                );
            }

            println!("\n--- Pairwise ---");
            for row in &report.pairwise {
                println!(
                    "{:<15} / {:<15} | {:>3}-{:>3}-{:>3} | score {:>5.1}-{:>5.1}",
                    row.bot_a,
                    row.bot_b,
                    row.wins_a,
                    row.draws,
                    row.wins_b,
                    row.score_a,
                    row.score_b
                );
            }

            println!("\n--- Color split ---");
            for row in &report.color_splits {
                println!(
                    "{:<15} (B) vs {:<15} (W) | B: {:>3} | W: {:>3} | D: {:>3}",
                    row.black, row.white, row.black_wins, row.white_wins, row.draws
                );
            }

            println!("\n--- End reasons ---");
            for reason in &report.end_reasons {
                println!("{:<15} {}", reason.key, reason.count);
            }
        }
        Commands::ReportHtml {
            input,
            output,
            json_href,
        } => {
            let json = std::fs::read_to_string(&input)
                .unwrap_or_else(|err| exit_with_error(format!("Failed to read report: {err}")));
            let report = TournamentReport::from_json(&json)
                .unwrap_or_else(|err| exit_with_error(format!("Failed to parse report: {err}")));
            let raw_json_href = json_href.or_else(|| {
                input
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(str::to_string)
            });
            let html = render_tournament_report_html_with_options(
                &report,
                &ReportRenderOptions { raw_json_href },
            );
            std::fs::write(&output, html).unwrap_or_else(|err| {
                exit_with_error(format!("Failed to write HTML report: {err}"))
            });
            println!("HTML report: {}", output.display());
        }
        Commands::TacticalScenarios {
            bots,
            search_time_ms,
            search_cpu_time_ms,
            report_json,
        } => {
            let configs = parse_search_config_specs(&bots, search_time_ms, search_cpu_time_ms)
                .unwrap_or_else(|err| exit_with_error(err));

            println!("--- Tactical Scenarios ---");
            println!(
                "Configs: {:?}",
                configs
                    .iter()
                    .map(|config| config.id.as_str())
                    .collect::<Vec<_>>()
            );
            println!("Cases: {}", TACTICAL_SCENARIO_CASES.len());
            if let Some(ms) = search_time_ms {
                println!("Search time budget: {ms} ms/move");
            }
            if let Some(ms) = search_cpu_time_ms {
                println!("Search CPU-time budget: {ms} ms/move");
            }
            println!();

            let report = run_tactical_scenarios(&configs, TACTICAL_SCENARIO_CASES);
            for result in &report.results {
                print_tactical_scenario_result(result);
            }

            print_tactical_report_summary(&report);

            if let Some(path) = &report_json {
                let json = report.to_json().unwrap_or_else(|err| {
                    exit_with_error(format!("Failed to serialize tactical report: {err}"))
                });
                std::fs::write(path, json).unwrap_or_else(|err| {
                    exit_with_error(format!("Failed to write tactical report: {err}"))
                });
                println!("Report JSON: {}", path.display());
            }
        }
        Commands::AnalyzeReplay {
            input,
            output,
            max_depth,
            max_scan_plies,
        } => {
            let json = std::fs::read_to_string(&input)
                .unwrap_or_else(|err| exit_with_error(format!("Failed to read replay: {err}")));
            let replay = Replay::from_json(&json)
                .unwrap_or_else(|err| exit_with_error(format!("Failed to parse replay: {err}")));
            let analysis = analyze_replay(
                &replay,
                AnalysisOptions {
                    max_depth,
                    max_scan_plies: Some(max_scan_plies),
                    ..AnalysisOptions::default()
                },
            )
            .unwrap_or_else(|err| exit_with_error(format!("Failed to analyze replay: {err}")));
            let output_json = serde_json::to_string_pretty(&analysis).unwrap_or_else(|err| {
                exit_with_error(format!("Failed to serialize analysis: {err}"))
            });

            if let Some(output) = output {
                std::fs::write(&output, output_json).unwrap_or_else(|err| {
                    exit_with_error(format!("Failed to write analysis: {err}"))
                });
                println!("Analysis JSON: {}", output.display());
            } else {
                println!("{output_json}");
            }
        }
        Commands::AnalyzeReplayBatch {
            replay_dir,
            report_json,
            report_html,
            max_depth,
            max_scan_plies,
            include_proof_details,
        } => {
            let report = run_analysis_batch_with_options(
                &replay_dir,
                AnalysisBatchRunOptions {
                    analysis: AnalysisOptions {
                        max_depth,
                        max_scan_plies: Some(max_scan_plies),
                        ..AnalysisOptions::default()
                    },
                    include_proof_details,
                },
            )
            .unwrap_or_else(|err| {
                exit_with_error(format!("Failed to run replay analysis batch: {err}"))
            });

            print_analysis_batch_report_summary(&report);

            if let Some(path) = report_json {
                let json = serde_json::to_string_pretty(&report).unwrap_or_else(|err| {
                    exit_with_error(format!("Failed to serialize analysis batch report: {err}"))
                });
                std::fs::write(&path, json).unwrap_or_else(|err| {
                    exit_with_error(format!("Failed to write analysis batch report: {err}"))
                });
                println!("Report JSON: {}", path.display());
            }
            if let Some(path) = report_html {
                let html = render_analysis_batch_report_html(&report);
                std::fs::write(&path, html).unwrap_or_else(|err| {
                    exit_with_error(format!("Failed to write analysis batch HTML: {err}"))
                });
                println!("Report HTML: {}", path.display());
            }
            if report.failed > 0 {
                std::process::exit(1);
            }
        }
        Commands::AnalyzeReportReplays {
            report,
            entrant_a,
            entrant_b,
            sample_size,
            report_json,
            report_html,
            max_depth,
            max_scan_plies,
            include_proof_details,
        } => {
            let default_top_two = entrant_a.is_none() && entrant_b.is_none();
            let json = std::fs::read_to_string(&report).unwrap_or_else(|err| {
                exit_with_error(format!("Failed to read tournament report: {err}"))
            });
            let tournament_report = TournamentReport::from_json(&json).unwrap_or_else(|err| {
                exit_with_error(format!("Failed to parse tournament report: {err}"))
            });
            let standing_bots = tournament_report
                .standings
                .iter()
                .map(|standing| standing.bot.clone())
                .collect::<Vec<_>>();
            let (entrant_a, entrant_b) =
                resolve_report_replay_entrants(&standing_bots, entrant_a, entrant_b)
                    .unwrap_or_else(|err| exit_with_error(err));
            let selections =
                select_report_matches(&tournament_report, &entrant_a, &entrant_b, sample_size)
                    .unwrap_or_else(|err| exit_with_error(err));
            let mut inputs = Vec::with_capacity(selections.len());
            for selection in selections {
                let replay = report_match_to_replay(&tournament_report, selection.match_report)
                    .unwrap_or_else(|err| {
                        exit_with_error(format!(
                            "Failed to convert match {} to replay: {err}",
                            selection.match_report.match_index
                        ))
                    });
                inputs.push(ReplayAnalysisInput {
                    label: format!(
                        "match_{:04}__{}__vs__{}",
                        selection.match_report.match_index,
                        selection.match_report.black.replace('+', "_"),
                        selection.match_report.white.replace('+', "_")
                    ),
                    replay,
                });
            }
            let batch_report = run_analysis_batch_replays_with_options(
                report_replay_source_label(&report, &entrant_a, &entrant_b, default_top_two),
                inputs,
                AnalysisBatchRunOptions {
                    analysis: AnalysisOptions {
                        max_depth,
                        max_scan_plies: Some(max_scan_plies),
                        ..AnalysisOptions::default()
                    },
                    include_proof_details,
                },
            );

            print_analysis_batch_report_summary(&batch_report);

            if let Some(path) = report_json {
                let json = serde_json::to_string_pretty(&batch_report).unwrap_or_else(|err| {
                    exit_with_error(format!("Failed to serialize analysis batch report: {err}"))
                });
                std::fs::write(&path, json).unwrap_or_else(|err| {
                    exit_with_error(format!("Failed to write analysis batch report: {err}"))
                });
                println!("Report JSON: {}", path.display());
            }
            if let Some(path) = report_html {
                let html = render_analysis_batch_report_html(&batch_report);
                std::fs::write(&path, html).unwrap_or_else(|err| {
                    exit_with_error(format!("Failed to write analysis batch HTML: {err}"))
                });
                println!("Report HTML: {}", path.display());
            }
            if batch_report.failed > 0 {
                std::process::exit(1);
            }
        }
        Commands::AnalysisFixtures {
            report_json,
            report_html,
            max_depth,
            max_scan_plies,
        } => {
            let report = run_analysis_fixtures(AnalysisOptions {
                max_depth,
                max_scan_plies: Some(max_scan_plies),
                ..AnalysisOptions::default()
            })
            .unwrap_or_else(|err| {
                exit_with_error(format!("Failed to run analysis fixtures: {err}"))
            });

            for result in &report.results {
                print_analysis_fixture_result(result);
            }
            print_analysis_fixture_report_summary(&report);

            if let Some(path) = report_json {
                let json = serde_json::to_string_pretty(&report).unwrap_or_else(|err| {
                    exit_with_error(format!(
                        "Failed to serialize analysis fixture report: {err}"
                    ))
                });
                std::fs::write(&path, json).unwrap_or_else(|err| {
                    exit_with_error(format!("Failed to write analysis fixture report: {err}"))
                });
                println!("Report JSON: {}", path.display());
            }
            if let Some(path) = report_html {
                let html = render_analysis_fixture_report_html(&report);
                std::fs::write(&path, html).unwrap_or_else(|err| {
                    exit_with_error(format!("Failed to write analysis fixture HTML: {err}"))
                });
                println!("Report HTML: {}", path.display());
            }
            if report.failed > 0 {
                std::process::exit(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tournament_plan_builds_round_robin_pairs() {
        let plan = tournament_plan(
            CliTournamentSchedule::RoundRobin,
            Some("a,b,c"),
            None,
            None,
            None,
            None,
        )
        .expect("round robin plan should parse");

        assert_eq!(plan.bot_names, vec!["a", "b", "c"]);
        assert!(plan.anchor_names.is_empty());
        assert_eq!(plan.anchor_report, None);
        assert_eq!(
            plan.pairs,
            vec![
                TournamentPair {
                    bot_a_idx: 0,
                    bot_b_idx: 1,
                },
                TournamentPair {
                    bot_a_idx: 0,
                    bot_b_idx: 2,
                },
                TournamentPair {
                    bot_a_idx: 1,
                    bot_b_idx: 2,
                },
            ]
        );
    }

    #[test]
    fn tournament_plan_requires_exactly_two_head_to_head_bots() {
        let plan = tournament_plan(
            CliTournamentSchedule::HeadToHead,
            Some("d5,d7"),
            None,
            None,
            None,
            None,
        )
        .expect("head-to-head plan should parse");

        assert_eq!(plan.bot_names, vec!["d5", "d7"]);
        assert_eq!(
            plan.pairs,
            vec![TournamentPair {
                bot_a_idx: 0,
                bot_b_idx: 1,
            }]
        );

        let err = tournament_plan(
            CliTournamentSchedule::HeadToHead,
            Some("d3,d5,d7"),
            None,
            None,
            None,
            None,
        )
        .unwrap_err();
        assert!(err.contains("exactly 2 bots"));
    }

    #[test]
    fn make_bot_factory_accepts_corridor_lab_aliases() {
        for spec in ["corridor-random", "corridor-d1"] {
            let factory = make_bot_factory(spec, None, None)
                .unwrap_or_else(|err| panic!("{spec} should parse: {err}"));
            let bot = factory(42);
            assert_eq!(bot.name(), spec);
        }
    }

    #[test]
    fn make_bot_factory_applies_budget_to_corridor_search_fallback() {
        let factory = make_bot_factory("corridor-d1", None, Some(123))
            .expect("corridor-d1 should parse with a CPU budget");
        let mut bot = factory(42);
        let board = gomoku_core::Board::new(RuleConfig::default());

        let _ = bot.choose_move(&board);
        let trace = bot
            .trace()
            .expect("corridor-d1 fallback should preserve the search trace");

        assert_eq!(trace["source"], "corridor-fallback");
        assert_eq!(trace["config"]["max_depth"], 1);
        assert_eq!(trace["config"]["cpu_time_budget_ms"], 123);
    }

    #[test]
    fn tournament_plan_builds_candidate_vs_anchor_gauntlet() {
        let plan = tournament_plan(
            CliTournamentSchedule::Gauntlet,
            None,
            Some("candidate"),
            None,
            Some("anchor-a,anchor-b"),
            Some("reports/latest.json"),
        )
        .expect("gauntlet plan should parse");

        assert_eq!(plan.bot_names, vec!["candidate", "anchor-a", "anchor-b"]);
        assert_eq!(plan.anchor_names, vec!["anchor-a", "anchor-b"]);
        assert_eq!(plan.anchor_report.as_deref(), Some("reports/latest.json"));
        assert_eq!(
            plan.pairs,
            vec![
                TournamentPair {
                    bot_a_idx: 0,
                    bot_b_idx: 1,
                },
                TournamentPair {
                    bot_a_idx: 0,
                    bot_b_idx: 2,
                },
            ]
        );
    }

    #[test]
    fn tournament_plan_builds_batch_candidate_gauntlet() {
        let plan = tournament_plan(
            CliTournamentSchedule::Gauntlet,
            None,
            None,
            Some("candidate-a,candidate-b"),
            Some("anchor-a,anchor-b"),
            Some("reports/latest.json"),
        )
        .expect("batch gauntlet plan should parse");

        assert_eq!(
            plan.bot_names,
            vec!["candidate-a", "candidate-b", "anchor-a", "anchor-b"]
        );
        assert_eq!(plan.anchor_names, vec!["anchor-a", "anchor-b"]);
        assert_eq!(plan.anchor_report.as_deref(), Some("reports/latest.json"));
        assert_eq!(
            plan.pairs,
            vec![
                TournamentPair {
                    bot_a_idx: 0,
                    bot_b_idx: 2,
                },
                TournamentPair {
                    bot_a_idx: 0,
                    bot_b_idx: 3,
                },
                TournamentPair {
                    bot_a_idx: 1,
                    bot_b_idx: 2,
                },
                TournamentPair {
                    bot_a_idx: 1,
                    bot_b_idx: 3,
                },
            ]
        );
    }

    #[test]
    fn tournament_plan_rejects_mixed_gauntlet_candidate_args() {
        let err = tournament_plan(
            CliTournamentSchedule::Gauntlet,
            None,
            Some("candidate-a"),
            Some("candidate-b"),
            Some("anchor-a"),
            None,
        )
        .unwrap_err();

        assert!(err.contains("either --candidate or --candidates"));
    }

    #[test]
    fn tournament_plan_rejects_candidates_outside_gauntlet() {
        let err = tournament_plan(
            CliTournamentSchedule::RoundRobin,
            Some("a,b"),
            None,
            Some("candidate-a,candidate-b"),
            None,
            None,
        )
        .unwrap_err();

        assert!(err.contains("--candidate/--candidates/--anchors"));
    }

    #[test]
    fn tournament_plan_rejects_anchor_report_outside_gauntlet() {
        let err = tournament_plan(
            CliTournamentSchedule::RoundRobin,
            Some("a,b"),
            None,
            None,
            None,
            Some("reports/latest.json"),
        )
        .unwrap_err();

        assert!(err.contains("--anchor-report"));
    }

    #[test]
    fn analyze_replay_command_parses_input_output_and_model_limits() {
        let cli = Cli::try_parse_from([
            "gomoku-eval",
            "analyze-replay",
            "--input",
            "replays/match.json",
            "--output",
            "outputs/analysis.json",
            "--max-depth",
            "3",
            "--max-scan-plies",
            "12",
        ])
        .expect("analyze-replay command should parse");

        let Commands::AnalyzeReplay {
            input,
            output,
            max_depth,
            max_scan_plies,
        } = cli.command
        else {
            panic!("expected analyze-replay command");
        };

        assert_eq!(input, PathBuf::from("replays/match.json"));
        assert_eq!(output, Some(PathBuf::from("outputs/analysis.json")));
        assert_eq!(max_depth, 3);
        assert_eq!(max_scan_plies, 12);
    }

    #[test]
    fn analyze_replay_command_defaults_to_bounded_scan_cap() {
        let cli = Cli::try_parse_from([
            "gomoku-eval",
            "analyze-replay",
            "--input",
            "replays/match.json",
        ])
        .expect("analyze-replay command should parse");

        let Commands::AnalyzeReplay { max_scan_plies, .. } = cli.command else {
            panic!("expected analyze-replay command");
        };

        assert_eq!(max_scan_plies, DEFAULT_MAX_SCAN_PLIES);
    }

    #[test]
    fn analyze_replay_command_rejects_retired_reply_policy_flag() {
        let err = Cli::try_parse_from([
            "gomoku-eval",
            "analyze-replay",
            "--input",
            "replays/match.json",
            "--defense-policy",
            "all-legal-defense",
        ])
        .unwrap_err();

        assert!(err.to_string().contains("unexpected argument"));
    }

    #[test]
    fn analysis_fixtures_command_parses_report_and_model_limits() {
        let cli = Cli::try_parse_from([
            "gomoku-eval",
            "analysis-fixtures",
            "--report-json",
            "outputs/analysis-fixtures.json",
            "--report-html",
            "outputs/analysis-fixtures.html",
            "--max-depth",
            "4",
            "--max-scan-plies",
            "16",
        ])
        .expect("analysis-fixtures command should parse");

        let Commands::AnalysisFixtures {
            report_json,
            report_html,
            max_depth,
            max_scan_plies,
        } = cli.command
        else {
            panic!("expected analysis-fixtures command");
        };

        assert_eq!(
            report_json,
            Some(PathBuf::from("outputs/analysis-fixtures.json"))
        );
        assert_eq!(
            report_html,
            Some(PathBuf::from("outputs/analysis-fixtures.html"))
        );
        assert_eq!(max_depth, 4);
        assert_eq!(max_scan_plies, 16);
    }

    #[test]
    fn analyze_replay_batch_command_parses_reports_and_model_limits() {
        let cli = Cli::try_parse_from([
            "gomoku-eval",
            "analyze-replay-batch",
            "--replay-dir",
            "outputs/replays",
            "--report-json",
            "outputs/analysis-batch.json",
            "--report-html",
            "outputs/analysis-batch.html",
            "--max-depth",
            "3",
            "--max-scan-plies",
            "12",
        ])
        .expect("analyze-replay-batch command should parse");

        let Commands::AnalyzeReplayBatch {
            replay_dir,
            report_json,
            report_html,
            max_depth,
            max_scan_plies,
            include_proof_details,
        } = cli.command
        else {
            panic!("expected analyze-replay-batch command");
        };

        assert_eq!(replay_dir, PathBuf::from("outputs/replays"));
        assert_eq!(
            report_json,
            Some(PathBuf::from("outputs/analysis-batch.json"))
        );
        assert_eq!(
            report_html,
            Some(PathBuf::from("outputs/analysis-batch.html"))
        );
        assert_eq!(max_depth, 3);
        assert_eq!(max_scan_plies, 12);
        assert!(!include_proof_details);
    }

    #[test]
    fn analyze_report_replays_command_parses_matchup_sample_and_model_limits() {
        let cli = Cli::try_parse_from([
            "gomoku-eval",
            "analyze-report-replays",
            "--report",
            "reports/latest.json",
            "--entrant-a",
            "search-d7+tactical-cap-8+pattern-eval",
            "--entrant-b",
            "search-d5+tactical-cap-8+pattern-eval",
            "--sample-size",
            "8",
            "--report-json",
            "outputs/analysis/top2-smoke.json",
            "--report-html",
            "outputs/analysis/top2-smoke.html",
            "--max-depth",
            "4",
            "--max-scan-plies",
            "8",
            "--include-proof-details",
        ])
        .expect("analyze-report-replays command should parse");

        let Commands::AnalyzeReportReplays {
            report,
            entrant_a,
            entrant_b,
            sample_size,
            report_json,
            report_html,
            max_depth,
            max_scan_plies,
            include_proof_details,
        } = cli.command
        else {
            panic!("expected analyze-report-replays command");
        };

        assert_eq!(report, PathBuf::from("reports/latest.json"));
        assert_eq!(
            entrant_a.as_deref(),
            Some("search-d7+tactical-cap-8+pattern-eval")
        );
        assert_eq!(
            entrant_b.as_deref(),
            Some("search-d5+tactical-cap-8+pattern-eval")
        );
        assert_eq!(sample_size, 8);
        assert_eq!(
            report_json,
            Some(PathBuf::from("outputs/analysis/top2-smoke.json"))
        );
        assert_eq!(
            report_html,
            Some(PathBuf::from("outputs/analysis/top2-smoke.html"))
        );
        assert_eq!(max_depth, 4);
        assert_eq!(max_scan_plies, 8);
        assert!(include_proof_details);
    }

    #[test]
    fn report_replay_default_entrant_selection_avoids_self_match() {
        let standings = vec![
            "search-d7+tactical-cap-8+pattern-eval".to_string(),
            "search-d5+tactical-cap-8+pattern-eval".to_string(),
            "search-d3+pattern-eval".to_string(),
        ];

        let (entrant_a, entrant_b) = resolve_report_replay_entrants(
            &standings,
            Some("search-d5+tactical-cap-8+pattern-eval".to_string()),
            None,
        )
        .expect("missing entrant should default to the highest different standing");

        assert_eq!(entrant_a, "search-d5+tactical-cap-8+pattern-eval");
        assert_eq!(entrant_b, "search-d7+tactical-cap-8+pattern-eval");
    }

    #[test]
    fn report_replay_source_label_keeps_default_selector_readable() {
        let report = PathBuf::from("reports/latest.json");

        assert_eq!(
            report_replay_source_label(&report, "search-d7", "search-d5", true),
            "reports/latest.json:Top 2 entrants"
        );
        assert_eq!(
            report_replay_source_label(&report, "search-d7", "search-d5", false),
            "reports/latest.json:search-d7 vs search-d5"
        );
    }
}
