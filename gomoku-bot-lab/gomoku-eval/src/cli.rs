use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use gomoku_bot::{lab_spec, RandomBot, SearchBot};
use gomoku_core::{Color, GameResult, Move, Replay, RuleConfig, Variant};
use gomoku_eval::analysis::{analyze_replay, AnalysisOptions, DEFAULT_MAX_SCAN_PLIES};
use gomoku_eval::analysis_batch::{
    published_analysis_report_from_batch, run_analysis_batch_replays_with_progress,
    run_analysis_batch_with_options, AnalysisBatchReport, AnalysisBatchRunOptions,
    PublishedAnalysisMatchSummary, PublishedAnalysisSectionInput, ReplayAnalysisInput,
};
use gomoku_eval::analysis_fixture::{
    run_analysis_fixtures, AnalysisFixtureReport, AnalysisFixtureResult,
};
use gomoku_eval::analysis_report::{
    report_match_to_replay, select_report_matches, ReportReplayMatch, ReportReplaySelection,
    ReportReplaySource,
};
use gomoku_eval::arena::{run_match_series_with_limits, MatchEndReason, MatchLimits, MatchResult};
use gomoku_eval::budget::{PooledCpuBudgetConfig, PooledSearchBot};
use gomoku_eval::opening::{OpeningPolicy, CENTERED_SUITE_MAX_PLIES};
use gomoku_eval::renju_rules::{
    all_renju_rule_fixtures, run_renju_rule_fixtures, RenjuRuleFixtureResult, RenjuRuleReport,
};
use gomoku_eval::report::{
    AnchorReferenceReport, PublishedTournamentReport, TournamentReport, TournamentRunReport,
};
use gomoku_eval::seed::derive_seed;
use gomoku_eval::tournament::{
    default_thread_count, round_robin_pairs, run_scheduled_pairs_parallel, TournamentBotFactory,
    TournamentOptions, TournamentPair,
};

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

fn tournament_progress_interval(total_games: usize) -> Option<usize> {
    if total_games == 0 {
        None
    } else {
        Some((total_games / 20).max(1))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[clap(rename_all = "kebab-case")]
enum CliSearchBudgetMode {
    Strict,
    Pooled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[clap(rename_all = "kebab-case")]
enum CliReportReplaySelector {
    HeadToHead,
    PresetTriangle,
}

impl CliReportReplaySelector {
    fn label(self) -> &'static str {
        match self {
            CliReportReplaySelector::HeadToHead => "Head-to-head",
            CliReportReplaySelector::PresetTriangle => "Preset triangle",
        }
    }
}

impl CliSearchBudgetMode {
    fn label(self) -> &'static str {
        match self {
            CliSearchBudgetMode::Strict => "strict",
            CliSearchBudgetMode::Pooled => "pooled",
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

    /// Search budget policy for search bots
    #[arg(long, value_enum, default_value = "strict")]
    search_budget_mode: CliSearchBudgetMode,

    /// Max CPU-time reserve for pooled search budgeting, in milliseconds
    #[arg(long, default_value_t = 4_000)]
    search_cpu_reserve_ms: u64,

    /// Max CPU-time budget for a single pooled move, in milliseconds
    #[arg(long)]
    search_cpu_max_move_ms: Option<u64>,

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
    search_budget_mode: CliSearchBudgetMode,
    search_cpu_reserve_ms: u64,
    search_cpu_max_move_ms: Option<u64>,
    seed: u64,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run N games between two different bots
    Versus {
        #[command(flatten)]
        options: EvalOptions,

        #[arg(long, default_value = "search-d3")]
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

        #[arg(long, default_value = "search-d3")]
        bot: String,

        #[arg(long, default_value_t = 10)]
        games: u32,

        #[arg(long)]
        replay_dir: Option<PathBuf>,
    },
    /// Export compact published tournament report JSON from a full tournament report
    ReportJson {
        #[arg(long)]
        input: PathBuf,

        #[arg(long)]
        output: PathBuf,
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

        /// Write compact published tournament report JSON
        #[arg(long)]
        published_report_json: Option<PathBuf>,

        /// Number of opening plies before bots take over
        #[arg(long, default_value_t = 4)]
        opening_plies: usize,

        /// Opening policy used before bots take over
        #[arg(long, value_enum, default_value = "centered-suite")]
        opening_policy: CliOpeningPolicy,

        /// Worker threads used to run tournament games
        #[arg(long)]
        threads: Option<usize>,

        /// Exit nonzero after writing the report if rolling-frontier shadow checks mismatch
        #[arg(long)]
        fail_on_shadow_mismatch: bool,
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
    /// Run lethal-threat classifier scenarios
    LethalScenarios {
        /// Write reusable lethal scenario report JSON
        #[arg(long)]
        report_json: Option<PathBuf>,

        /// Print exact scenario boards after each result
        #[arg(long)]
        show_boards: bool,
    },
    /// Run focused Renju forbidden-move rule fixtures
    RenjuRules {
        /// Write reusable Renju rule fixture report JSON
        #[arg(long)]
        report_json: Option<PathBuf>,

        /// Print exact fixture boards after each result
        #[arg(long)]
        show_boards: bool,
    },
    /// Analyze a saved replay and emit bounded proof/classification JSON
    AnalyzeReplay {
        /// Replay JSON to analyze
        #[arg(long)]
        input: PathBuf,

        /// Write analysis JSON to this path instead of stdout
        #[arg(long)]
        output: Option<PathBuf>,

        /// Maximum corridor proof depth
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

        /// Maximum corridor proof depth
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
        /// Tournament report JSON containing replay move cells
        #[arg(long)]
        report: PathBuf,

        /// Match selection mode for report replays
        #[arg(long, value_enum, default_value = "head-to-head")]
        selector: CliReportReplaySelector,

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

        /// Write compact published analysis report JSON
        #[arg(long)]
        published_report_json: Option<PathBuf>,

        /// Maximum corridor proof depth
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

        /// Maximum corridor proof depth
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
    let value: serde_json::Value = serde_json::from_str(&json)
        .map_err(|err| format!("Failed to parse anchor report {}: {err}", path.display()))?;
    match value.get("report_kind").and_then(serde_json::Value::as_str) {
        Some("tournament") => {
            let source_report = TournamentReport::from_json(&json).map_err(|err| {
                format!("Failed to parse anchor report {}: {err}", path.display())
            })?;
            AnchorReferenceReport::from_report(Some(source_path), &source_report, anchor_names)
        }
        Some("published_tournament") => {
            let source_report = PublishedTournamentReport::from_json(&json).map_err(|err| {
                format!("Failed to parse anchor report {}: {err}", path.display())
            })?;
            AnchorReferenceReport::from_published_report(
                Some(source_path),
                &source_report,
                anchor_names,
            )
        }
        Some(other) => Err(format!("unsupported anchor report kind: {other}")),
        None => Err("anchor report is missing report_kind".to_string()),
    }
}

fn make_bot_factory(
    spec: &str,
    search_time_ms: Option<u64>,
    search_cpu_time_ms: Option<u64>,
    search_budget_mode: CliSearchBudgetMode,
    search_cpu_reserve_ms: u64,
    search_cpu_max_move_ms: Option<u64>,
) -> Result<BotFactory, String> {
    let spec = spec.to_string();
    if spec == "random" {
        return Ok(Arc::new(|seed| Box::new(RandomBot::seeded(seed))));
    }
    if let Some(config) =
        lab_spec::search_config_from_lab_spec(&spec, search_time_ms, search_cpu_time_ms)
    {
        return match search_budget_mode {
            CliSearchBudgetMode::Strict => {
                Ok(Arc::new(move |_| Box::new(SearchBot::with_config(config))))
            }
            CliSearchBudgetMode::Pooled => {
                if search_time_ms.is_some() {
                    return Err(
                        "Pooled search budgeting currently supports --search-cpu-time-ms, not --search-time-ms."
                            .to_string(),
                    );
                }
                let Some(base_ms) = search_cpu_time_ms else {
                    return Err(
                        "Pooled search budgeting requires --search-cpu-time-ms.".to_string()
                    );
                };
                if let Some(max_move_ms) = search_cpu_max_move_ms {
                    if max_move_ms < base_ms {
                        return Err(
                            "--search-cpu-max-move-ms must be greater than or equal to --search-cpu-time-ms."
                                .to_string(),
                        );
                    }
                }
                Ok(Arc::new(move |_| {
                    Box::new(PooledSearchBot::new(
                        config,
                        PooledCpuBudgetConfig {
                            base_ms,
                            reserve_cap_ms: search_cpu_reserve_ms,
                            max_move_ms: search_cpu_max_move_ms,
                        },
                    ))
                }))
            }
        };
    }

    Err(format!(
        "Unknown bot type: '{spec}'. Use random, search-dN, or search-dN+suffixes."
    ))
}

fn exit_with_error(message: impl AsRef<str>) -> ! {
    eprintln!("{}", message.as_ref());
    std::process::exit(2);
}

fn pooled_budget_label(reserve_cap_ms: u64, max_move_ms: Option<u64>) -> String {
    match max_move_ms {
        Some(max_move_ms) => format!(
            "Search budget mode: pooled (reserve cap {reserve_cap_ms} ms, max move {max_move_ms} ms)"
        ),
        None => format!("Search budget mode: pooled (reserve cap {reserve_cap_ms} ms)"),
    }
}

fn variant_label(variant: &Variant) -> &'static str {
    match variant {
        Variant::Freestyle => "freestyle",
        Variant::Renju => "renju",
    }
}

fn eval_context(options: &EvalOptions) -> EvalContext {
    if options.search_budget_mode != CliSearchBudgetMode::Pooled
        && options.search_cpu_max_move_ms.is_some()
    {
        exit_with_error("--search-cpu-max-move-ms requires --search-budget-mode pooled.");
    }

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
        search_budget_mode: options.search_budget_mode,
        search_cpu_reserve_ms: options.search_cpu_reserve_ms,
        search_cpu_max_move_ms: options.search_cpu_max_move_ms,
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

fn print_renju_rule_fixture_result(result: &RenjuRuleFixtureResult) {
    let status = if result.passed { "PASS" } else { "FAIL" };
    let expected = if result.expected_legal {
        "legal"
    } else {
        "forbidden"
    };
    let actual = if result.actual_legal {
        "legal"
    } else {
        "forbidden"
    };
    println!(
        "{:<5} {:<44} {:?} {:<3} expect {:<9} actual {:<9} source {}",
        status, result.id, result.color, result.candidate, expected, actual, result.source
    );
}

fn print_renju_rule_fixture_board(result: &RenjuRuleFixtureResult) {
    println!();
    for row in &result.board {
        println!("{row}");
    }
}

fn print_renju_rule_report_summary(report: &RenjuRuleReport) {
    println!(
        "\n--- Summary ---\nRenju rule fixtures: {}/{} passed, {} failed",
        report.passed, report.total, report.failed
    );
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
        "{:<5} {:<34} winner {:<7} root {:<14} forced {:>2}..{:<2} chance {:<4} loser {:<4} notes {}",
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
            .critical_loser_ply
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
        "summary: total {}, unclear {}, ongoing/draw {}, errors {}",
        report.total,
        report.summary.unclear,
        report.summary.ongoing_or_draw,
        report.summary.analysis_error
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

struct ReportReplaySectionPlan<'a> {
    label: String,
    entrant_a: String,
    entrant_b: String,
    selections: Vec<ReportReplaySelection<'a>>,
}

const PRESET_EASY_BOT: &str = "search-d1";
const PRESET_NORMAL_BOT: &str = "search-d3+pattern-eval";
const PRESET_HARD_BOT: &str = "search-d7+tactical-cap-8+pattern-eval+corridor-proof-c16-d8-w4";

fn report_replay_section_plans<'a>(
    report_source: &'a ReportReplaySource,
    selector: CliReportReplaySelector,
    entrant_a: Option<String>,
    entrant_b: Option<String>,
    sample_size: usize,
) -> Result<Vec<ReportReplaySectionPlan<'a>>, String> {
    match selector {
        CliReportReplaySelector::HeadToHead => {
            let (entrant_a, entrant_b) =
                resolve_report_replay_entrants(&report_source.standings, entrant_a, entrant_b)?;
            let selections =
                select_report_matches(report_source, &entrant_a, &entrant_b, sample_size)?;
            Ok(vec![ReportReplaySectionPlan {
                label: if sample_size == usize::MAX {
                    format!("{entrant_a} vs {entrant_b}")
                } else {
                    report_replay_selector_label(&entrant_a, &entrant_b, false)
                },
                entrant_a,
                entrant_b,
                selections,
            }])
        }
        CliReportReplaySelector::PresetTriangle => {
            if entrant_a.is_some() || entrant_b.is_some() {
                return Err(
                    "preset-triangle selector does not accept --entrant-a or --entrant-b"
                        .to_string(),
                );
            }

            [
                ("Easy vs Normal", PRESET_EASY_BOT, PRESET_NORMAL_BOT),
                ("Easy vs Hard", PRESET_EASY_BOT, PRESET_HARD_BOT),
                ("Normal vs Hard", PRESET_NORMAL_BOT, PRESET_HARD_BOT),
            ]
            .into_iter()
            .map(|(label, entrant_a, entrant_b)| {
                let selections =
                    select_report_matches(report_source, entrant_a, entrant_b, usize::MAX)?;
                Ok(ReportReplaySectionPlan {
                    label: label.to_string(),
                    entrant_a: entrant_a.to_string(),
                    entrant_b: entrant_b.to_string(),
                    selections,
                })
            })
            .collect()
        }
    }
}

fn flatten_report_replay_sections(
    report_source: &ReportReplaySource,
    sections: &[ReportReplaySectionPlan<'_>],
) -> Vec<ReplayAnalysisInput> {
    sections
        .iter()
        .flat_map(|section| {
            section.selections.iter().map(|selection| {
                let replay = report_match_to_replay(report_source, selection.match_report)
                    .unwrap_or_else(|err| {
                        exit_with_error(format!(
                            "Failed to convert match {} to replay: {err}",
                            selection.match_report.match_index
                        ))
                    });
                ReplayAnalysisInput {
                    label: report_replay_input_label(selection.match_report),
                    replay,
                }
            })
        })
        .collect()
}

fn published_analysis_sections_from_plans(
    sections: &[ReportReplaySectionPlan<'_>],
) -> Vec<PublishedAnalysisSectionInput> {
    sections
        .iter()
        .map(|section| PublishedAnalysisSectionInput {
            label: section.label.clone(),
            entrant_a: section.entrant_a.clone(),
            entrant_b: section.entrant_b.clone(),
            matches: section
                .selections
                .iter()
                .map(|selection| published_analysis_match_summary(selection.match_report))
                .collect(),
        })
        .collect()
}

fn published_analysis_match_summary(
    match_report: &ReportReplayMatch,
) -> PublishedAnalysisMatchSummary {
    PublishedAnalysisMatchSummary {
        match_index: match_report.match_index,
        black: match_report.black.clone(),
        white: match_report.white.clone(),
        result: match_report.result.clone(),
        winner: match_report.winner.clone(),
        end_reason: match_report.end_reason.clone(),
        move_cells: match_report.move_cells.clone(),
        move_count: match_report.move_count,
    }
}

fn report_replay_input_label(match_report: &ReportReplayMatch) -> String {
    format!(
        "match_{:04}__{}__vs__{}",
        match_report.match_index,
        match_report.black.replace('+', "_"),
        match_report.white.replace('+', "_")
    )
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
    format!(
        "{}:{}",
        report.display(),
        report_replay_selector_label(entrant_a, entrant_b, default_top_two)
    )
}

fn report_replay_selector_label(entrant_a: &str, entrant_b: &str, default_top_two: bool) -> String {
    if default_top_two {
        "Top 2 entrants".to_string()
    } else {
        format!("{entrant_a} vs {entrant_b}")
    }
}

pub fn run() {
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
                search_budget_mode,
                search_cpu_reserve_ms,
                search_cpu_max_move_ms,
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
            if search_budget_mode == CliSearchBudgetMode::Pooled {
                println!(
                    "{}",
                    pooled_budget_label(search_cpu_reserve_ms, search_cpu_max_move_ms)
                );
            }

            if let Some(dir) = &replay_dir {
                std::fs::create_dir_all(dir).unwrap();
            }

            let bot_a_factory = match make_bot_factory(
                &bot_a,
                search_time_ms,
                search_cpu_time_ms,
                search_budget_mode,
                search_cpu_reserve_ms,
                search_cpu_max_move_ms,
            ) {
                Ok(factory) => factory,
                Err(err) => exit_with_error(err),
            };
            let bot_b_factory = match make_bot_factory(
                &bot_b,
                search_time_ms,
                search_cpu_time_ms,
                search_budget_mode,
                search_cpu_reserve_ms,
                search_cpu_max_move_ms,
            ) {
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
                search_budget_mode,
                search_cpu_reserve_ms,
                search_cpu_max_move_ms,
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
            if search_budget_mode == CliSearchBudgetMode::Pooled {
                println!(
                    "{}",
                    pooled_budget_label(search_cpu_reserve_ms, search_cpu_max_move_ms)
                );
            }

            if let Some(dir) = &replay_dir {
                std::fs::create_dir_all(dir).unwrap();
            }

            let bot_factory = match make_bot_factory(
                &bot,
                search_time_ms,
                search_cpu_time_ms,
                search_budget_mode,
                search_cpu_reserve_ms,
                search_cpu_max_move_ms,
            ) {
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
            published_report_json,
            opening_plies,
            opening_policy,
            threads,
            fail_on_shadow_mismatch,
        } => {
            let EvalContext {
                config,
                rule_label,
                limits,
                search_time_ms,
                search_cpu_time_ms,
                search_budget_mode,
                search_cpu_reserve_ms,
                search_cpu_max_move_ms,
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
            let total_games = pairs.len() * games_per_pair as usize;
            let progress_interval = tournament_progress_interval(total_games);
            if let Some(progress_interval) = progress_interval {
                println!("Progress: every {progress_interval} completed game(s)");
            }
            if let Some(ms) = search_time_ms {
                println!("Search time budget: {ms} ms/move");
            }
            if let Some(ms) = search_cpu_time_ms {
                println!("Search CPU-time budget: {ms} ms/move");
            }
            if search_budget_mode == CliSearchBudgetMode::Pooled {
                println!(
                    "{}",
                    pooled_budget_label(search_cpu_reserve_ms, search_cpu_max_move_ms)
                );
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
                search_budget_mode: search_budget_mode.label().to_string(),
                search_cpu_reserve_ms: (search_budget_mode == CliSearchBudgetMode::Pooled)
                    .then_some(search_cpu_reserve_ms),
                search_cpu_max_move_ms: (search_budget_mode == CliSearchBudgetMode::Pooled)
                    .then_some(search_cpu_max_move_ms)
                    .flatten(),
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
                match make_bot_factory(
                    name,
                    search_time_ms,
                    search_cpu_time_ms,
                    search_budget_mode,
                    search_cpu_reserve_ms,
                    search_cpu_max_move_ms,
                ) {
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
                    progress_interval,
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
            if let Some(path) = &published_report_json {
                let published_report = PublishedTournamentReport::from_tournament_report(&report);
                let json = published_report.to_json().unwrap_or_else(|err| {
                    exit_with_error(format!("Failed to serialize published report: {err}"))
                });
                std::fs::write(path, json).unwrap_or_else(|err| {
                    exit_with_error(format!("Failed to write published report: {err}"))
                });
                println!("\nPublished report JSON: {}", path.display());
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

            let shadow_mismatches = report.shadow_mismatch_count();
            if fail_on_shadow_mismatch && shadow_mismatches > 0 {
                eprintln!(
                    "Rolling frontier shadow guard failed: {shadow_mismatches} mismatch(es)."
                );
                std::process::exit(1);
            }
        }
        Commands::ReportJson { input, output } => {
            let json = std::fs::read_to_string(&input)
                .unwrap_or_else(|err| exit_with_error(format!("Failed to read report: {err}")));
            let value: serde_json::Value = serde_json::from_str(&json)
                .unwrap_or_else(|err| exit_with_error(format!("Failed to parse report: {err}")));
            let published_report =
                match value.get("report_kind").and_then(serde_json::Value::as_str) {
                    Some("tournament") => {
                        let report = TournamentReport::from_json(&json).unwrap_or_else(|err| {
                            exit_with_error(format!("Failed to parse tournament report: {err}"))
                        });
                        PublishedTournamentReport::from_tournament_report(&report)
                    }
                    Some("published_tournament") => {
                        let report =
                            PublishedTournamentReport::from_json(&json).unwrap_or_else(|err| {
                                exit_with_error(format!("Failed to parse published report: {err}"))
                            });
                        PublishedTournamentReport::from_published_report(&report)
                    }
                    Some(other) => exit_with_error(format!("Unsupported report kind: {other}")),
                    None => exit_with_error("Report is missing report_kind"),
                };
            let json = published_report.to_json().unwrap_or_else(|err| {
                exit_with_error(format!("Failed to serialize published report: {err}"))
            });
            std::fs::write(&output, json).unwrap_or_else(|err| {
                exit_with_error(format!("Failed to write published report: {err}"))
            });
            println!("Published report JSON: {}", output.display());
        }
        Commands::TacticalScenarios {
            bots,
            search_time_ms,
            search_cpu_time_ms,
            report_json,
        } => {
            let failed = crate::scenario_cli::run_tactical_scenarios_command(
                &bots,
                search_time_ms,
                search_cpu_time_ms,
                report_json.as_deref(),
            )
            .unwrap_or_else(|err| exit_with_error(err));
            if failed {
                std::process::exit(1);
            }
        }
        Commands::LethalScenarios {
            report_json,
            show_boards,
        } => {
            let failed = crate::scenario_cli::run_lethal_scenarios_command(
                report_json.as_deref(),
                show_boards,
            )
            .unwrap_or_else(|err| exit_with_error(err));
            if failed {
                std::process::exit(1);
            }
        }
        Commands::RenjuRules {
            report_json,
            show_boards,
        } => {
            println!("--- Renju Rule Fixtures ---");
            let fixtures = all_renju_rule_fixtures()
                .unwrap_or_else(|err| exit_with_error(format!("Failed to load fixtures: {err}")));
            println!("Cases: {}", fixtures.len());
            println!();

            let report = run_renju_rule_fixtures(&fixtures)
                .unwrap_or_else(|err| exit_with_error(format!("Failed to run fixtures: {err}")));
            for result in &report.results {
                print_renju_rule_fixture_result(result);
                if show_boards {
                    print_renju_rule_fixture_board(result);
                    println!();
                }
            }

            print_renju_rule_report_summary(&report);

            if let Some(path) = &report_json {
                let json = report.to_json().unwrap_or_else(|err| {
                    exit_with_error(format!("Failed to serialize Renju rule report: {err}"))
                });
                std::fs::write(path, json).unwrap_or_else(|err| {
                    exit_with_error(format!("Failed to write Renju rule report: {err}"))
                });
                println!("Report JSON: {}", path.display());
            }

            if report.failed > 0 {
                std::process::exit(1);
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
            if report.failed > 0 {
                std::process::exit(1);
            }
        }
        Commands::AnalyzeReportReplays {
            report,
            selector,
            entrant_a,
            entrant_b,
            sample_size,
            report_json,
            published_report_json,
            max_depth,
            max_scan_plies,
            include_proof_details,
        } => {
            let json = std::fs::read_to_string(&report).unwrap_or_else(|err| {
                exit_with_error(format!("Failed to read tournament report: {err}"))
            });
            let report_source = ReportReplaySource::from_json(&json).unwrap_or_else(|err| {
                exit_with_error(format!("Failed to parse tournament report: {err}"))
            });
            let default_top_two = selector == CliReportReplaySelector::HeadToHead
                && entrant_a.is_none()
                && entrant_b.is_none();
            let section_plans = report_replay_section_plans(
                &report_source,
                selector,
                entrant_a,
                entrant_b,
                sample_size,
            )
            .unwrap_or_else(|err| exit_with_error(err));
            let published_sections = published_analysis_sections_from_plans(&section_plans);
            let inputs = flatten_report_replay_sections(&report_source, &section_plans);
            let source = match selector {
                CliReportReplaySelector::HeadToHead => {
                    let section = section_plans
                        .first()
                        .unwrap_or_else(|| exit_with_error("missing head-to-head section"));
                    report_replay_source_label(
                        &report,
                        &section.entrant_a,
                        &section.entrant_b,
                        default_top_two,
                    )
                }
                CliReportReplaySelector::PresetTriangle => {
                    format!("{}:{}", report.display(), selector.label())
                }
            };
            let progress_interval = tournament_progress_interval(inputs.len());
            let batch_report = run_analysis_batch_replays_with_progress(
                source,
                inputs,
                AnalysisBatchRunOptions {
                    analysis: AnalysisOptions {
                        max_depth,
                        max_scan_plies: Some(max_scan_plies),
                        ..AnalysisOptions::default()
                    },
                    include_proof_details: include_proof_details || published_report_json.is_some(),
                },
                progress_interval,
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
            if let Some(path) = published_report_json {
                let published_report = published_analysis_report_from_batch(
                    report.display().to_string(),
                    Some(&report_source.provenance),
                    selector.label().to_string(),
                    &batch_report,
                    &published_sections,
                )
                .unwrap_or_else(|err| {
                    exit_with_error(format!("Failed to build published analysis report: {err}"))
                });
                let json = published_report.to_json().unwrap_or_else(|err| {
                    exit_with_error(format!(
                        "Failed to serialize published analysis report: {err}"
                    ))
                });
                std::fs::write(&path, json).unwrap_or_else(|err| {
                    exit_with_error(format!("Failed to write published analysis report: {err}"))
                });
                println!("Published report JSON: {}", path.display());
            }
            if batch_report.failed > 0 {
                std::process::exit(1);
            }
        }
        Commands::AnalysisFixtures {
            report_json,
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
    fn make_bot_factory_rejects_retired_corridor_lab_aliases() {
        for spec in ["corridor-random", "corridor-d1"] {
            let err = match make_bot_factory(spec, None, None, CliSearchBudgetMode::Strict, 0, None)
            {
                Ok(_) => panic!("retired corridor bot alias should not parse: {spec}"),
                Err(err) => err,
            };
            assert!(err.contains("search-dN+suffixes"));
        }
    }

    #[test]
    fn make_bot_factory_rejects_retired_corridor_quiescence_suffixes() {
        for spec in ["search-d1+corridor-q", "search-d1+corridor-qd4"] {
            let err =
                match make_bot_factory(spec, None, Some(123), CliSearchBudgetMode::Strict, 0, None)
                {
                    Ok(_) => panic!("retired corridor quiescence suffix should not parse: {spec}"),
                    Err(err) => err,
                };
            assert!(err.contains("Unknown bot"));
        }
    }

    #[test]
    fn make_bot_factory_rejects_pooled_max_move_below_base_budget() {
        let err = match make_bot_factory(
            "search-d1",
            None,
            Some(2_000),
            CliSearchBudgetMode::Pooled,
            8_000,
            Some(1_000),
        ) {
            Ok(_) => panic!("pooled max move below base budget should not parse"),
            Err(err) => err,
        };

        assert!(err.contains("--search-cpu-max-move-ms"));
    }

    #[test]
    fn tournament_plan_builds_candidate_vs_anchor_gauntlet() {
        let plan = tournament_plan(
            CliTournamentSchedule::Gauntlet,
            None,
            Some("candidate"),
            None,
            Some("anchor-a,anchor-b"),
            Some("../reports/lab/bot-report.json"),
        )
        .expect("gauntlet plan should parse");

        assert_eq!(plan.bot_names, vec!["candidate", "anchor-a", "anchor-b"]);
        assert_eq!(plan.anchor_names, vec!["anchor-a", "anchor-b"]);
        assert_eq!(
            plan.anchor_report.as_deref(),
            Some("../reports/lab/bot-report.json")
        );
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
            Some("../reports/lab/bot-report.json"),
        )
        .expect("batch gauntlet plan should parse");

        assert_eq!(
            plan.bot_names,
            vec!["candidate-a", "candidate-b", "anchor-a", "anchor-b"]
        );
        assert_eq!(plan.anchor_names, vec!["anchor-a", "anchor-b"]);
        assert_eq!(
            plan.anchor_report.as_deref(),
            Some("../reports/lab/bot-report.json")
        );
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
            Some("../reports/lab/bot-report.json"),
        )
        .unwrap_err();

        assert!(err.contains("--anchor-report"));
    }

    #[test]
    fn tournament_command_parses_pooled_search_budget() {
        let cli = Cli::try_parse_from([
            "gomoku-eval",
            "tournament",
            "--bots",
            "search-d3,search-d5",
            "--search-cpu-time-ms",
            "1000",
            "--search-budget-mode",
            "pooled",
            "--search-cpu-reserve-ms",
            "8000",
            "--search-cpu-max-move-ms",
            "4000",
        ])
        .expect("tournament command should parse");

        let Commands::Tournament { options, .. } = cli.command else {
            panic!("expected tournament command");
        };

        assert_eq!(options.search_cpu_time_ms, Some(1000));
        assert_eq!(options.search_budget_mode, CliSearchBudgetMode::Pooled);
        assert_eq!(options.search_cpu_reserve_ms, 8000);
        assert_eq!(options.search_cpu_max_move_ms, Some(4000));
    }

    #[test]
    fn tournament_command_parses_shadow_mismatch_guard() {
        let cli = Cli::try_parse_from([
            "gomoku-eval",
            "tournament",
            "--bots",
            "search-d3,search-d3+rolling-frontier-shadow",
            "--fail-on-shadow-mismatch",
        ])
        .expect("tournament command should parse");

        let Commands::Tournament {
            fail_on_shadow_mismatch,
            ..
        } = cli.command
        else {
            panic!("expected tournament command");
        };

        assert!(fail_on_shadow_mismatch);
    }

    #[test]
    fn report_json_command_parses_input_and_output() {
        let cli = Cli::try_parse_from([
            "gomoku-eval",
            "report-json",
            "--input",
            "outputs/full-report.json",
            "--output",
            "outputs/report.json",
        ])
        .expect("report-json command should parse");

        let Commands::ReportJson { input, output } = cli.command else {
            panic!("expected report-json command");
        };

        assert_eq!(input, PathBuf::from("outputs/full-report.json"));
        assert_eq!(output, PathBuf::from("outputs/report.json"));
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
    fn renju_rules_command_parses_report_and_board_flag() {
        let cli = Cli::try_parse_from([
            "gomoku-eval",
            "renju-rules",
            "--report-json",
            "outputs/renju-rule-fixtures.json",
            "--show-boards",
        ])
        .expect("renju-rules command should parse");

        let Commands::RenjuRules {
            report_json,
            show_boards,
        } = cli.command
        else {
            panic!("expected renju-rules command");
        };

        assert_eq!(
            report_json,
            Some(PathBuf::from("outputs/renju-rule-fixtures.json"))
        );
        assert!(show_boards);
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
            "--max-depth",
            "4",
            "--max-scan-plies",
            "16",
        ])
        .expect("analysis-fixtures command should parse");

        let Commands::AnalysisFixtures {
            report_json,
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
            "--max-depth",
            "3",
            "--max-scan-plies",
            "12",
        ])
        .expect("analyze-replay-batch command should parse");

        let Commands::AnalyzeReplayBatch {
            replay_dir,
            report_json,
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
            "../reports/lab/bot-report.json",
            "--entrant-a",
            "search-d7+tactical-cap-8+pattern-eval",
            "--entrant-b",
            "search-d5+tactical-cap-8+pattern-eval",
            "--sample-size",
            "8",
            "--report-json",
            "outputs/analysis/top2-smoke.json",
            "--published-report-json",
            "../reports/lab/analysis-report.json",
            "--max-depth",
            "4",
            "--max-scan-plies",
            "8",
            "--include-proof-details",
        ])
        .expect("analyze-report-replays command should parse");

        let Commands::AnalyzeReportReplays {
            report,
            selector,
            entrant_a,
            entrant_b,
            sample_size,
            report_json,
            published_report_json,
            max_depth,
            max_scan_plies,
            include_proof_details,
        } = cli.command
        else {
            panic!("expected analyze-report-replays command");
        };

        assert_eq!(report, PathBuf::from("../reports/lab/bot-report.json"));
        assert_eq!(selector, CliReportReplaySelector::HeadToHead);
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
            published_report_json,
            Some(PathBuf::from("../reports/lab/analysis-report.json"))
        );
        assert_eq!(max_depth, 4);
        assert_eq!(max_scan_plies, 8);
        assert!(include_proof_details);
    }

    #[test]
    fn analyze_report_replays_command_parses_preset_triangle_selector() {
        let cli = Cli::try_parse_from([
            "gomoku-eval",
            "analyze-report-replays",
            "--report",
            "../reports/lab/bot-report.json",
            "--selector",
            "preset-triangle",
            "--published-report-json",
            "../reports/lab/analysis-report.json",
        ])
        .expect("analyze-report-replays command should parse");

        let Commands::AnalyzeReportReplays {
            selector,
            report,
            published_report_json,
            ..
        } = cli.command
        else {
            panic!("expected analyze-report-replays command");
        };

        assert_eq!(selector, CliReportReplaySelector::PresetTriangle);
        assert_eq!(report, PathBuf::from("../reports/lab/bot-report.json"));
        assert_eq!(
            published_report_json,
            Some(PathBuf::from("../reports/lab/analysis-report.json"))
        );
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
        let report = PathBuf::from("../reports/lab/bot-report.json");

        assert_eq!(
            report_replay_source_label(&report, "search-d7", "search-d5", true),
            "../reports/lab/bot-report.json:Top 2 entrants"
        );
        assert_eq!(
            report_replay_source_label(&report, "search-d7", "search-d5", false),
            "../reports/lab/bot-report.json:search-d7 vs search-d5"
        );
    }
}
