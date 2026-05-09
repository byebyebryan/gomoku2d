use crate::arena::{
    run_match_with_limits, run_match_with_setup, MatchEndReason, MatchLimits, MatchResult,
    MatchSetup,
};
use crate::elo::RatingTracker;
use crate::opening::{opening_metadata_for_game, opening_moves_for_game, OpeningPolicy};
use crate::seed::derive_seed;
use gomoku_bot::Bot;
use gomoku_core::{Color, GameResult, Move, Replay, RuleConfig};
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

pub type TournamentBotFactory = Arc<dyn Fn(u64) -> Box<dyn Bot> + Send + Sync>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TournamentOptions {
    pub limits: MatchLimits,
    pub seed: u64,
    pub opening_plies: usize,
    pub opening_policy: OpeningPolicy,
    pub threads: usize,
    pub progress_interval: Option<usize>,
}

impl Default for TournamentOptions {
    fn default() -> Self {
        Self {
            limits: MatchLimits::default(),
            seed: 0,
            opening_plies: 0,
            opening_policy: OpeningPolicy::default(),
            threads: default_thread_count(),
            progress_interval: None,
        }
    }
}

pub fn default_thread_count() -> usize {
    std::thread::available_parallelism()
        .map(|threads| threads.get())
        .map(|threads| threads.saturating_sub(2).max(1))
        .unwrap_or(1)
}

pub struct TournamentResults {
    pub elo_tracker: RatingTracker,
    pub wins: HashMap<String, u32>,
    pub draws: HashMap<String, u32>,
    pub losses: HashMap<String, u32>,
    pub time_ms: HashMap<String, u128>,
    pub moves: HashMap<String, u32>,
    pub nodes: HashMap<String, u64>,
    pub node_samples: HashMap<String, u32>,
    pub end_reasons: HashMap<MatchEndReason, u32>,
    pub matches: Vec<TournamentMatchRecord>,
}

#[derive(Debug, Clone)]
pub struct TournamentMatchRecord {
    pub match_idx: usize,
    pub black_name: String,
    pub white_name: String,
    pub result: GameResult,
    pub replay: Replay,
    pub timing: crate::arena::MatchTiming,
    pub end_reason: MatchEndReason,
    pub opening: Option<TournamentOpeningRecord>,
}

#[derive(Debug, Clone)]
pub struct TournamentOpeningRecord {
    pub policy: String,
    pub index: u32,
    pub suite_index: Option<usize>,
    pub template_index: Option<usize>,
    pub transform_index: Option<usize>,
    pub ply_count: usize,
}

struct MatchRecordInput<'a> {
    match_idx: usize,
    name_a: &'a str,
    name_b: &'a str,
    black_name: &'a str,
    white_name: &'a str,
    a_is_black: bool,
    opening: Option<TournamentOpeningRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TournamentPair {
    pub bot_a_idx: usize,
    pub bot_b_idx: usize,
}

impl Default for TournamentResults {
    fn default() -> Self {
        Self::new()
    }
}

impl TournamentResults {
    pub fn new() -> Self {
        Self {
            elo_tracker: RatingTracker::new(crate::elo::DEFAULT_K_FACTOR),
            wins: HashMap::new(),
            draws: HashMap::new(),
            losses: HashMap::new(),
            time_ms: HashMap::new(),
            moves: HashMap::new(),
            nodes: HashMap::new(),
            node_samples: HashMap::new(),
            end_reasons: HashMap::new(),
            matches: Vec::new(),
        }
    }

    fn record_result(&mut self, player: &str, is_win: bool, is_draw: bool) {
        if is_draw {
            *self.draws.entry(player.to_string()).or_insert(0) += 1;
        } else if is_win {
            *self.wins.entry(player.to_string()).or_insert(0) += 1;
        } else {
            *self.losses.entry(player.to_string()).or_insert(0) += 1;
        }
    }

    fn record_timing(&mut self, player: &str, time_ms: u128, moves: u32) {
        *self.time_ms.entry(player.to_string()).or_insert(0) += time_ms;
        *self.moves.entry(player.to_string()).or_insert(0) += moves;
    }

    fn record_end_reason(&mut self, reason: MatchEndReason) {
        *self.end_reasons.entry(reason).or_insert(0) += 1;
    }

    fn record_nodes(&mut self, player: &str, nodes: u64) {
        *self.nodes.entry(player.to_string()).or_insert(0) += nodes;
        *self.node_samples.entry(player.to_string()).or_insert(0) += 1;
    }

    pub fn avg_time_per_move_ms(&self, player: &str) -> f64 {
        let moves = *self.moves.get(player).unwrap_or(&0);
        if moves == 0 {
            return 0.0;
        }
        *self.time_ms.get(player).unwrap_or(&0) as f64 / moves as f64
    }

    pub fn avg_nodes_per_search_move(&self, player: &str) -> f64 {
        let samples = *self.node_samples.get(player).unwrap_or(&0);
        if samples == 0 {
            return 0.0;
        }
        *self.nodes.get(player).unwrap_or(&0) as f64 / samples as f64
    }

    fn initialize_players<F>(&mut self, bot_factories: &[(String, F)]) {
        for (name, _) in bot_factories {
            self.wins.entry(name.clone()).or_insert(0);
            self.draws.entry(name.clone()).or_insert(0);
            self.losses.entry(name.clone()).or_insert(0);
            self.time_ms.entry(name.clone()).or_insert(0);
            self.moves.entry(name.clone()).or_insert(0);
            self.nodes.entry(name.clone()).or_insert(0);
            self.node_samples.entry(name.clone()).or_insert(0);
        }
    }

    fn record_match(&mut self, input: MatchRecordInput<'_>, mr: &MatchResult) {
        self.elo_tracker
            .update(input.name_a, input.name_b, &mr.result, input.a_is_black);

        match mr.result {
            GameResult::Winner(Color::Black) => {
                self.record_result(input.name_a, input.a_is_black, false);
                self.record_result(input.name_b, !input.a_is_black, false);
            }
            GameResult::Winner(Color::White) => {
                self.record_result(input.name_a, !input.a_is_black, false);
                self.record_result(input.name_b, input.a_is_black, false);
            }
            GameResult::Draw => {
                self.record_result(input.name_a, false, true);
                self.record_result(input.name_b, false, true);
            }
            GameResult::Ongoing => unreachable!(),
        }

        self.record_timing(
            input.black_name,
            mr.timing.black_time_ms,
            mr.timing.black_moves,
        );
        self.record_timing(
            input.white_name,
            mr.timing.white_time_ms,
            mr.timing.white_moves,
        );
        for (idx, replay_move) in mr.replay.moves.iter().enumerate() {
            let Some(trace) = &replay_move.trace else {
                continue;
            };
            let Some(nodes) = trace
                .get("total_nodes")
                .or_else(|| trace.get("nodes"))
                .and_then(|value| value.as_u64())
            else {
                continue;
            };
            let player = if idx % 2 == 0 {
                input.black_name
            } else {
                input.white_name
            };
            self.record_nodes(player, nodes);
        }
        self.record_end_reason(mr.end_reason);
        self.matches.push(TournamentMatchRecord {
            match_idx: input.match_idx,
            black_name: input.black_name.to_string(),
            white_name: input.white_name.to_string(),
            result: mr.result.clone(),
            replay: mr.replay.clone(),
            timing: mr.timing,
            end_reason: mr.end_reason,
            opening: input.opening,
        });
    }
}

/// Runs a round-robin tournament among the provided bot factories.
/// Every pair of bots plays `games_per_pair` matches.
/// A factory is used to instantiate a new bot per game, ensuring clean state.
pub fn run_round_robin<F>(
    bot_factories: &[(String, F)],
    games_per_pair: u32,
    config: RuleConfig,
    mut on_game_end: impl FnMut(&str, &str, &GameResult, &Replay),
) -> TournamentResults
where
    F: Fn() -> Box<dyn Bot>,
{
    run_round_robin_with_limits(
        bot_factories,
        games_per_pair,
        config,
        MatchLimits::default(),
        |black_name, white_name, mr| on_game_end(black_name, white_name, &mr.result, &mr.replay),
    )
}

pub fn run_round_robin_with_limits<F>(
    bot_factories: &[(String, F)],
    games_per_pair: u32,
    config: RuleConfig,
    limits: MatchLimits,
    mut on_game_end: impl FnMut(&str, &str, &MatchResult),
) -> TournamentResults
where
    F: Fn() -> Box<dyn Bot>,
{
    let mut results = TournamentResults::new();
    let num_bots = bot_factories.len();

    results.initialize_players(bot_factories);

    let mut match_idx = 0usize;
    for i in 0..num_bots {
        for j in (i + 1)..num_bots {
            let (name_a, factory_a) = &bot_factories[i];
            let (name_b, factory_b) = &bot_factories[j];

            for game in 0..games_per_pair {
                match_idx += 1;
                let mut bot_a = factory_a();
                let mut bot_b = factory_b();

                // Alternate who plays black
                let a_is_black = game % 2 == 0;

                let mr = if a_is_black {
                    run_match_with_limits(
                        bot_a.as_mut(),
                        bot_b.as_mut(),
                        config.clone(),
                        limits,
                        |_, _, _, _| {},
                    )
                } else {
                    run_match_with_limits(
                        bot_b.as_mut(),
                        bot_a.as_mut(),
                        config.clone(),
                        limits,
                        |_, _, _, _| {},
                    )
                };
                let black_name = if a_is_black { name_a } else { name_b };
                let white_name = if a_is_black { name_b } else { name_a };

                results.record_match(
                    MatchRecordInput {
                        match_idx,
                        name_a,
                        name_b,
                        black_name,
                        white_name,
                        a_is_black,
                        opening: None,
                    },
                    &mr,
                );

                on_game_end(black_name, white_name, &mr);
            }
        }
    }

    results
}

struct TournamentJob {
    match_idx: usize,
    bot_a_idx: usize,
    bot_b_idx: usize,
    a_is_black: bool,
    opening_moves: Vec<Move>,
    opening: TournamentOpeningRecord,
    bot_a_seed: u64,
    bot_b_seed: u64,
}

struct TournamentOutcome {
    match_idx: usize,
    name_a: String,
    name_b: String,
    black_name: String,
    white_name: String,
    a_is_black: bool,
    opening: TournamentOpeningRecord,
    mr: MatchResult,
}

pub fn run_round_robin_parallel(
    bot_factories: &[(String, TournamentBotFactory)],
    games_per_pair: u32,
    config: RuleConfig,
    options: TournamentOptions,
    mut on_game_end: impl FnMut(&str, &str, &MatchResult),
) -> TournamentResults {
    let pairs = round_robin_pairs(bot_factories.len());
    run_scheduled_pairs_parallel(
        bot_factories,
        &pairs,
        games_per_pair,
        config,
        options,
        &mut on_game_end,
    )
}

pub fn round_robin_pairs(bot_count: usize) -> Vec<TournamentPair> {
    let mut pairs = Vec::new();
    for i in 0..bot_count {
        for j in (i + 1)..bot_count {
            pairs.push(TournamentPair {
                bot_a_idx: i,
                bot_b_idx: j,
            });
        }
    }
    pairs
}

pub fn run_scheduled_pairs_parallel(
    bot_factories: &[(String, TournamentBotFactory)],
    pairs: &[TournamentPair],
    games_per_pair: u32,
    config: RuleConfig,
    options: TournamentOptions,
    mut on_game_end: impl FnMut(&str, &str, &MatchResult),
) -> TournamentResults {
    let mut results = TournamentResults::new();
    results.initialize_players(bot_factories);

    let mut jobs = Vec::new();
    let mut match_idx = 0usize;
    for pair in pairs {
        for game in 0..games_per_pair {
            match_idx += 1;
            let paired_game = game / 2;
            let opening_metadata =
                opening_metadata_for_game(options.opening_policy, options.seed, paired_game);
            let opening_moves = opening_moves_for_game(
                options.opening_policy,
                &config,
                options.opening_plies,
                options.seed,
                paired_game,
            );
            jobs.push(TournamentJob {
                match_idx,
                bot_a_idx: pair.bot_a_idx,
                bot_b_idx: pair.bot_b_idx,
                a_is_black: game % 2 == 0,
                opening: TournamentOpeningRecord {
                    policy: opening_metadata.policy.label().to_string(),
                    index: opening_metadata.index,
                    suite_index: opening_metadata.suite_index,
                    template_index: opening_metadata.template_index,
                    transform_index: opening_metadata.transform_index,
                    ply_count: opening_moves.len(),
                },
                opening_moves,
                bot_a_seed: derive_seed(
                    options.seed,
                    [pair.bot_a_idx as u64, pair.bot_b_idx as u64, game as u64, 0],
                ),
                bot_b_seed: derive_seed(
                    options.seed,
                    [pair.bot_a_idx as u64, pair.bot_b_idx as u64, game as u64, 1],
                ),
            });
        }
    }

    let threads = options.threads.max(1);
    let pool = ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .expect("failed to build tournament thread pool");
    let total_jobs = jobs.len();
    let completed_jobs = AtomicUsize::new(0);
    let progress_interval = options
        .progress_interval
        .filter(|interval| total_jobs > 0 && *interval > 0);
    let progress_start = Instant::now();
    if progress_interval.is_some() {
        eprintln!("Progress: 0/{total_jobs} games complete");
    }
    let mut outcomes: Vec<TournamentOutcome> = pool.install(|| {
        jobs.par_iter()
            .map(|job| {
                let (name_a, factory_a) = &bot_factories[job.bot_a_idx];
                let (name_b, factory_b) = &bot_factories[job.bot_b_idx];
                let mut bot_a = factory_a(job.bot_a_seed);
                let mut bot_b = factory_b(job.bot_b_seed);
                let setup = MatchSetup {
                    opening_moves: job.opening_moves.clone(),
                };

                let mr = if job.a_is_black {
                    run_match_with_setup(
                        bot_a.as_mut(),
                        bot_b.as_mut(),
                        config.clone(),
                        options.limits,
                        setup,
                        |_, _, _, _| {},
                    )
                } else {
                    run_match_with_setup(
                        bot_b.as_mut(),
                        bot_a.as_mut(),
                        config.clone(),
                        options.limits,
                        setup,
                        |_, _, _, _| {},
                    )
                };

                if let Some(interval) = progress_interval {
                    let done = completed_jobs.fetch_add(1, Ordering::Relaxed) + 1;
                    if done == total_jobs || done.is_multiple_of(interval) {
                        let elapsed = progress_start.elapsed().as_secs_f64();
                        let progress = done as f64 * 100.0 / total_jobs as f64;
                        let eta_secs = if done > 0 && done < total_jobs && elapsed > 0.0 {
                            let games_per_sec = done as f64 / elapsed;
                            Some((total_jobs - done) as f64 / games_per_sec)
                        } else {
                            None
                        };
                        match eta_secs {
                            Some(eta_secs) => eprintln!(
                                "Progress: {done}/{total_jobs} games complete ({progress:.1}%, elapsed {:.0}s, ETA {:.0}s)",
                                elapsed, eta_secs
                            ),
                            None => eprintln!(
                                "Progress: {done}/{total_jobs} games complete ({progress:.1}%, elapsed {:.0}s)",
                                elapsed
                            ),
                        }
                    }
                }

                TournamentOutcome {
                    match_idx: job.match_idx,
                    name_a: name_a.clone(),
                    name_b: name_b.clone(),
                    black_name: if job.a_is_black {
                        name_a.clone()
                    } else {
                        name_b.clone()
                    },
                    white_name: if job.a_is_black {
                        name_b.clone()
                    } else {
                        name_a.clone()
                    },
                    a_is_black: job.a_is_black,
                    opening: job.opening.clone(),
                    mr,
                }
            })
            .collect()
    });
    outcomes.sort_by_key(|outcome| outcome.match_idx);

    for outcome in outcomes {
        results.record_match(
            MatchRecordInput {
                match_idx: outcome.match_idx,
                name_a: &outcome.name_a,
                name_b: &outcome.name_b,
                black_name: &outcome.black_name,
                white_name: &outcome.white_name,
                a_is_black: outcome.a_is_black,
                opening: Some(outcome.opening.clone()),
            },
            &outcome.mr,
        );
        on_game_end(&outcome.black_name, &outcome.white_name, &outcome.mr);
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use gomoku_bot::RandomBot;
    use gomoku_core::{Move, Variant};

    type TestBotFactory = fn() -> Box<dyn Bot>;

    fn random_bot_factory() -> Box<dyn Bot> {
        Box::new(RandomBot::new())
    }

    #[test]
    fn round_robin_records_capped_draws_and_timing() {
        let factories: Vec<(String, TestBotFactory)> = vec![
            ("random-a".to_string(), random_bot_factory),
            ("random-b".to_string(), random_bot_factory),
        ];
        let config = RuleConfig {
            variant: Variant::Freestyle,
            ..Default::default()
        };

        let results = run_round_robin_with_limits(
            &factories,
            2,
            config,
            MatchLimits {
                max_moves: Some(1),
                max_game_ms: None,
            },
            |_, _, _| {},
        );

        assert_eq!(*results.draws.get("random-a").unwrap(), 2);
        assert_eq!(*results.draws.get("random-b").unwrap(), 2);
        assert_eq!(
            *results.end_reasons.get(&MatchEndReason::MaxMoves).unwrap(),
            2
        );
        assert_eq!(*results.moves.get("random-a").unwrap(), 1);
        assert_eq!(*results.moves.get("random-b").unwrap(), 1);
    }

    #[test]
    fn parallel_round_robin_records_capped_draws_and_timing() {
        let factories: Vec<(String, TournamentBotFactory)> = vec![
            (
                "random-a".to_string(),
                Arc::new(|seed| Box::new(RandomBot::seeded(seed))),
            ),
            (
                "random-b".to_string(),
                Arc::new(|seed| Box::new(RandomBot::seeded(seed))),
            ),
        ];
        let config = RuleConfig {
            variant: Variant::Freestyle,
            ..Default::default()
        };

        let results = run_round_robin_parallel(
            &factories,
            2,
            config,
            TournamentOptions {
                limits: MatchLimits {
                    max_moves: Some(1),
                    max_game_ms: None,
                },
                seed: 7,
                opening_plies: 0,
                opening_policy: OpeningPolicy::CenteredSuite,
                threads: 2,
                progress_interval: None,
            },
            |_, _, _| {},
        );

        assert_eq!(*results.draws.get("random-a").unwrap(), 2);
        assert_eq!(*results.draws.get("random-b").unwrap(), 2);
        assert_eq!(
            *results.end_reasons.get(&MatchEndReason::MaxMoves).unwrap(),
            2
        );
        assert_eq!(*results.moves.get("random-a").unwrap(), 1);
        assert_eq!(*results.moves.get("random-b").unwrap(), 1);
    }

    #[test]
    fn parallel_round_robin_uses_shared_centered_openings_across_pairs() {
        let factories: Vec<(String, TournamentBotFactory)> = vec![
            (
                "random-a".to_string(),
                Arc::new(|seed| Box::new(RandomBot::seeded(seed))),
            ),
            (
                "random-b".to_string(),
                Arc::new(|seed| Box::new(RandomBot::seeded(seed))),
            ),
            (
                "random-c".to_string(),
                Arc::new(|seed| Box::new(RandomBot::seeded(seed))),
            ),
        ];
        let config = RuleConfig {
            variant: Variant::Renju,
            ..Default::default()
        };

        let results = run_round_robin_parallel(
            &factories,
            4,
            config,
            TournamentOptions {
                limits: MatchLimits {
                    max_moves: Some(4),
                    max_game_ms: None,
                },
                seed: 7,
                opening_plies: 4,
                opening_policy: OpeningPolicy::CenteredSuite,
                threads: 2,
                progress_interval: None,
            },
            |_, _, _| {},
        );

        let center = Move { row: 7, col: 7 }.to_notation();
        fn replay_moves(record: &TournamentMatchRecord) -> Vec<String> {
            record.replay.moves.iter().map(|mv| mv.mv.clone()).collect()
        }

        let first_opening = replay_moves(&results.matches[0]);
        let second_opening = replay_moves(&results.matches[2]);
        let first_metadata = results.matches[0]
            .opening
            .as_ref()
            .expect("parallel tournament should record opening metadata");

        assert_eq!(first_opening[0], center);
        assert_ne!(first_opening, second_opening);
        assert_eq!(first_metadata.policy, "centered-suite");
        assert_eq!(first_metadata.index, 0);
        assert_eq!(first_metadata.ply_count, 4);
        assert!(first_metadata.suite_index.is_some());
        assert!(first_metadata.template_index.is_some());
        assert!(first_metadata.transform_index.is_some());
        for pair_games in results.matches.chunks_exact(4) {
            let first_color_opening = replay_moves(&pair_games[0]);
            let first_swapped_opening = replay_moves(&pair_games[1]);
            let second_color_opening = replay_moves(&pair_games[2]);
            let second_swapped_opening = replay_moves(&pair_games[3]);

            assert_eq!(first_color_opening, first_swapped_opening);
            assert_eq!(first_color_opening, first_opening);
            assert_eq!(second_color_opening, second_swapped_opening);
            assert_eq!(second_color_opening, second_opening);
        }
    }

    #[test]
    fn parallel_scheduled_pairs_runs_only_requested_pairs() {
        let factories: Vec<(String, TournamentBotFactory)> = vec![
            (
                "candidate".to_string(),
                Arc::new(|seed| Box::new(RandomBot::seeded(seed))),
            ),
            (
                "anchor-a".to_string(),
                Arc::new(|seed| Box::new(RandomBot::seeded(seed))),
            ),
            (
                "anchor-b".to_string(),
                Arc::new(|seed| Box::new(RandomBot::seeded(seed))),
            ),
        ];
        let config = RuleConfig {
            variant: Variant::Freestyle,
            ..Default::default()
        };

        let results = run_scheduled_pairs_parallel(
            &factories,
            &[TournamentPair {
                bot_a_idx: 0,
                bot_b_idx: 1,
            }],
            2,
            config,
            TournamentOptions {
                limits: MatchLimits {
                    max_moves: Some(1),
                    max_game_ms: None,
                },
                seed: 7,
                opening_plies: 0,
                opening_policy: OpeningPolicy::CenteredSuite,
                threads: 2,
                progress_interval: None,
            },
            |_, _, _| {},
        );

        assert_eq!(results.matches.len(), 2);
        assert!(results
            .matches
            .iter()
            .all(|record| record.black_name != "anchor-b" && record.white_name != "anchor-b"));
        assert_eq!(*results.moves.get("candidate").unwrap(), 1);
        assert_eq!(*results.moves.get("anchor-a").unwrap(), 1);
        assert_eq!(*results.moves.get("anchor-b").unwrap(), 0);
    }
}
