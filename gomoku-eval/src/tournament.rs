use crate::arena::run_match;
use crate::elo::RatingTracker;
use gomoku_bot::Bot;
use gomoku_core::{Color, GameResult, Replay, RuleConfig};
use std::collections::HashMap;

pub struct TournamentResults {
    pub elo_tracker: RatingTracker,
    pub wins: HashMap<String, u32>,
    pub draws: HashMap<String, u32>,
    pub losses: HashMap<String, u32>,
}

impl TournamentResults {
    pub fn new() -> Self {
        Self {
            elo_tracker: RatingTracker::new(crate::elo::DEFAULT_K_FACTOR),
            wins: HashMap::new(),
            draws: HashMap::new(),
            losses: HashMap::new(),
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
}

/// Runs a round-robin tournament among the provided bot factories.
/// Every pair of bots plays `games_per_pair` matches.
/// A factory is used to instantiate a new bot per game, ensuring clean state.
pub fn run_round_robin<F>(
    bot_factories: &Vec<(String, F)>,
    games_per_pair: u32,
    config: RuleConfig,
    mut on_game_end: impl FnMut(&str, &str, &GameResult, &Replay),
) -> TournamentResults
where
    F: Fn() -> Box<dyn Bot>,
{
    let mut results = TournamentResults::new();
    let num_bots = bot_factories.len();

    // Ensure all bots are initialized in the trackers
    for (name, _) in bot_factories {
        results.wins.entry(name.clone()).or_insert(0);
        results.draws.entry(name.clone()).or_insert(0);
        results.losses.entry(name.clone()).or_insert(0);
    }

    for i in 0..num_bots {
        for j in (i + 1)..num_bots {
            let (name_a, factory_a) = &bot_factories[i];
            let (name_b, factory_b) = &bot_factories[j];

            for game in 0..games_per_pair {
                let mut bot_a = factory_a();
                let mut bot_b = factory_b();

                // Alternate who plays black
                let a_is_black = game % 2 == 0;

                let mr = if a_is_black {
                    run_match(
                        bot_a.as_mut(),
                        bot_b.as_mut(),
                        config.clone(),
                        |_, _, _, _| {},
                    )
                } else {
                    run_match(
                        bot_b.as_mut(),
                        bot_a.as_mut(),
                        config.clone(),
                        |_, _, _, _| {},
                    )
                };

                // Update Elo
                results
                    .elo_tracker
                    .update(name_a, name_b, &mr.result, a_is_black);

                // Update W/L/D
                match mr.result {
                    GameResult::Winner(Color::Black) => {
                        results.record_result(name_a, a_is_black, false);
                        results.record_result(name_b, !a_is_black, false);
                    }
                    GameResult::Winner(Color::White) => {
                        results.record_result(name_a, !a_is_black, false);
                        results.record_result(name_b, a_is_black, false);
                    }
                    GameResult::Draw => {
                        results.record_result(name_a, false, true);
                        results.record_result(name_b, false, true);
                    }
                    GameResult::Ongoing => unreachable!(),
                }

                on_game_end(
                    if a_is_black { name_a } else { name_b },
                    if a_is_black { name_b } else { name_a },
                    &mr.result,
                    &mr.replay,
                );
            }
        }
    }

    results
}
