use super::*;

#[derive(Parser, Debug)]
#[command(name = "gomoku-eval", about = "Evaluation harness for Gomoku bots")]
pub(super) struct Cli {
    #[command(subcommand)]
    pub(super) command: Commands,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub(super) enum CliOpeningPolicy {
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
pub(super) enum CliTournamentSchedule {
    RoundRobin,
    HeadToHead,
    Gauntlet,
}

impl CliTournamentSchedule {
    pub(super) fn label(self) -> &'static str {
        match self {
            CliTournamentSchedule::RoundRobin => "round-robin",
            CliTournamentSchedule::HeadToHead => "head-to-head",
            CliTournamentSchedule::Gauntlet => "gauntlet",
        }
    }
}

pub(super) fn tournament_progress_interval(total_games: usize) -> Option<usize> {
    if total_games == 0 {
        None
    } else {
        Some((total_games / 20).max(1))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub(super) enum CliSearchBudgetMode {
    Strict,
    Pooled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub(super) enum CliReportReplaySelector {
    HeadToHead,
    PresetTriangle,
}

impl CliReportReplaySelector {
    pub(super) fn label(self) -> &'static str {
        match self {
            CliReportReplaySelector::HeadToHead => "Head-to-head",
            CliReportReplaySelector::PresetTriangle => "Preset triangle",
        }
    }
}

impl CliSearchBudgetMode {
    pub(super) fn label(self) -> &'static str {
        match self {
            CliSearchBudgetMode::Strict => "strict",
            CliSearchBudgetMode::Pooled => "pooled",
        }
    }
}

#[derive(Args, Debug, Clone)]
pub(super) struct EvalOptions {
    /// Rule variant: "renju" (default) or "freestyle"
    #[arg(long, default_value = "renju")]
    pub(super) rule: String,

    /// Per-move search budget for search bots, in milliseconds
    #[arg(long)]
    pub(super) search_time_ms: Option<u64>,

    /// Per-move Linux thread CPU-time budget for search bots, in milliseconds
    #[arg(long)]
    pub(super) search_cpu_time_ms: Option<u64>,

    /// Search budget policy for search bots
    #[arg(long, value_enum, default_value = "strict")]
    pub(super) search_budget_mode: CliSearchBudgetMode,

    /// Max CPU-time reserve for pooled search budgeting, in milliseconds
    #[arg(long, default_value_t = 4_000)]
    pub(super) search_cpu_reserve_ms: u64,

    /// Max CPU-time budget for a single pooled move, in milliseconds
    #[arg(long)]
    pub(super) search_cpu_max_move_ms: Option<u64>,

    /// Stop a game after this many moves and record it as a draw
    #[arg(long)]
    pub(super) max_moves: Option<usize>,

    /// Stop a game after this wall-clock duration and record it as a draw
    #[arg(long)]
    pub(super) max_game_ms: Option<u64>,

    /// Base seed for reproducible random bots and tournament openings
    #[arg(long, default_value_t = 0)]
    pub(super) seed: u64,
}

pub(super) struct EvalContext {
    pub(super) config: RuleConfig,
    pub(super) rule_label: &'static str,
    pub(super) limits: MatchLimits,
    pub(super) search_time_ms: Option<u64>,
    pub(super) search_cpu_time_ms: Option<u64>,
    pub(super) search_budget_mode: CliSearchBudgetMode,
    pub(super) search_cpu_reserve_ms: u64,
    pub(super) search_cpu_max_move_ms: Option<u64>,
    pub(super) seed: u64,
}

#[derive(Subcommand, Debug)]
pub(super) enum Commands {
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
