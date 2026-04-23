use gomoku_core::GameResult;
use std::collections::HashMap;

/// Standard Elo K-factor. Higher values mean ratings change more per game.
pub const DEFAULT_K_FACTOR: f64 = 32.0;
pub const DEFAULT_INITIAL_RATING: f64 = 1200.0;

/// Calculate the expected probability of Player A winning against Player B.
pub fn expected_score(rating_a: f64, rating_b: f64) -> f64 {
    1.0 / (1.0 + 10.0_f64.powf((rating_b - rating_a) / 400.0))
}

/// Calculate the new rating for Player A given the result and K-factor.
/// Score: 1.0 for win, 0.5 for draw, 0.0 for loss.
pub fn compute_new_rating(rating_a: f64, expected: f64, actual_score: f64, k_factor: f64) -> f64 {
    rating_a + k_factor * (actual_score - expected)
}

pub struct RatingTracker {
    ratings: HashMap<String, f64>,
    k_factor: f64,
}

impl RatingTracker {
    pub fn new(k_factor: f64) -> Self {
        Self {
            ratings: HashMap::new(),
            k_factor,
        }
    }

    pub fn get_rating(&self, name: &str) -> f64 {
        *self.ratings.get(name).unwrap_or(&DEFAULT_INITIAL_RATING)
    }

    pub fn update(
        &mut self,
        player_a: &str,
        player_b: &str,
        result: &GameResult,
        a_is_black: bool,
    ) {
        let r_a = self.get_rating(player_a);
        let r_b = self.get_rating(player_b);

        let e_a = expected_score(r_a, r_b);
        let e_b = expected_score(r_b, r_a);

        let (score_a, score_b) = match result {
            GameResult::Winner(gomoku_core::Color::Black) => {
                if a_is_black {
                    (1.0, 0.0)
                } else {
                    (0.0, 1.0)
                }
            }
            GameResult::Winner(gomoku_core::Color::White) => {
                if !a_is_black {
                    (1.0, 0.0)
                } else {
                    (0.0, 1.0)
                }
            }
            GameResult::Draw => (0.5, 0.5),
            GameResult::Ongoing => return,
        };

        let new_r_a = compute_new_rating(r_a, e_a, score_a, self.k_factor);
        let new_r_b = compute_new_rating(r_b, e_b, score_b, self.k_factor);

        self.ratings.insert(player_a.to_string(), new_r_a);
        self.ratings.insert(player_b.to_string(), new_r_b);
    }

    pub fn get_sorted_ratings(&self) -> Vec<(String, f64)> {
        let mut vec: Vec<_> = self.ratings.clone().into_iter().collect();
        // Sort descending by rating
        vec.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        vec
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expected_score() {
        assert!((expected_score(1200.0, 1200.0) - 0.5).abs() < 1e-6);
        assert!(expected_score(1600.0, 1200.0) > 0.9);
        assert!(expected_score(1200.0, 1600.0) < 0.1);
    }

    #[test]
    fn test_compute_new_rating() {
        // Equal ratings, A wins
        let new_a = compute_new_rating(1200.0, 0.5, 1.0, 32.0);
        assert_eq!(new_a, 1216.0); // 1200 + 32 * (1.0 - 0.5)

        // Equal ratings, B loses (score 0 for B)
        let new_b = compute_new_rating(1200.0, 0.5, 0.0, 32.0);
        assert_eq!(new_b, 1184.0); // 1200 + 32 * (0.0 - 0.5)
    }

    #[test]
    fn test_tracker_update() {
        let mut tracker = RatingTracker::new(32.0);

        // A wins playing Black
        tracker.update(
            "BotA",
            "BotB",
            &GameResult::Winner(gomoku_core::Color::Black),
            true,
        );
        assert_eq!(tracker.get_rating("BotA"), 1216.0);
        assert_eq!(tracker.get_rating("BotB"), 1184.0);

        // B wins playing Black against A
        tracker.update(
            "BotB",
            "BotA",
            &GameResult::Winner(gomoku_core::Color::Black),
            true,
        );
        // B expected to lose more often since rating is lower, so win gives more points.
        let expected_b = expected_score(1184.0, 1216.0); // < 0.5
        let expected_new_b = 1184.0 + 32.0 * (1.0 - expected_b);
        assert!((tracker.get_rating("BotB") - expected_new_b).abs() < 1e-6);
    }
}
