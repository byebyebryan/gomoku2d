use crate::seed::derive_seed;
use gomoku_bot::{Bot, RandomBot};
use gomoku_core::{Board, Move, RuleConfig};

pub const CENTERED_SUITE_LABEL: &str = "centered-suite";
pub const RANDOM_LEGAL_LABEL: &str = "random-legal";
pub const CENTERED_SUITE_MAX_PLIES: usize = 4;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum OpeningPolicy {
    #[default]
    CenteredSuite,
    RandomLegal,
}

impl OpeningPolicy {
    pub fn label(self) -> &'static str {
        match self {
            OpeningPolicy::CenteredSuite => CENTERED_SUITE_LABEL,
            OpeningPolicy::RandomLegal => RANDOM_LEGAL_LABEL,
        }
    }
}

const BASE_CENTERED_OPENINGS: [[(isize, isize); CENTERED_SUITE_MAX_PLIES]; 4] = [
    [(0, 0), (0, 1), (1, 0), (1, 1)],
    [(0, 0), (1, 0), (0, 2), (-1, 1)],
    [(0, 0), (1, 1), (-1, 1), (2, 0)],
    [(0, 0), (-1, 1), (1, 0), (0, -2)],
];

type CenteredTransform = fn((isize, isize)) -> (isize, isize);

const CENTERED_TRANSFORMS: [CenteredTransform; 8] = [
    |(r, c)| (r, c),
    |(r, c)| (c, -r),
    |(r, c)| (-r, -c),
    |(r, c)| (-c, r),
    |(r, c)| (r, -c),
    |(r, c)| (-r, c),
    |(r, c)| (c, r),
    |(r, c)| (-c, -r),
];

pub fn opening_moves_for_game(
    policy: OpeningPolicy,
    config: &RuleConfig,
    plies: usize,
    base_seed: u64,
    opening_index: u32,
) -> Vec<Move> {
    match policy {
        OpeningPolicy::CenteredSuite => {
            centered_suite_opening(config.board_size, plies, base_seed, opening_index)
        }
        OpeningPolicy::RandomLegal => random_legal_opening(config, plies, base_seed, opening_index),
    }
}

fn centered_suite_opening(
    board_size: usize,
    plies: usize,
    base_seed: u64,
    opening_index: u32,
) -> Vec<Move> {
    assert!(
        plies <= CENTERED_SUITE_MAX_PLIES,
        "centered opening suite supports at most {CENTERED_SUITE_MAX_PLIES} plies"
    );
    assert!(
        board_size >= 7,
        "centered opening suite requires board >= 7"
    );

    let suite_len = BASE_CENTERED_OPENINGS.len() * CENTERED_TRANSFORMS.len();
    let seed_offset = derive_seed(base_seed, [0]) as usize % suite_len;
    let suite_index = (seed_offset + opening_index as usize) % suite_len;
    let base = BASE_CENTERED_OPENINGS[suite_index / CENTERED_TRANSFORMS.len()];
    let transform = CENTERED_TRANSFORMS[suite_index % CENTERED_TRANSFORMS.len()];
    let center = board_size as isize / 2;

    base.iter()
        .take(plies)
        .map(|&relative| {
            let (row_offset, col_offset) = transform(relative);
            Move {
                row: (center + row_offset) as usize,
                col: (center + col_offset) as usize,
            }
        })
        .collect()
}

fn random_legal_opening(
    config: &RuleConfig,
    plies: usize,
    base_seed: u64,
    opening_index: u32,
) -> Vec<Move> {
    let mut board = Board::new(config.clone());
    let mut bot = RandomBot::seeded(derive_seed(base_seed, [opening_index as u64]));
    let mut moves = Vec::with_capacity(plies);

    for _ in 0..plies {
        let mv = bot.choose_move(&board);
        let _ = board
            .apply_move(mv)
            .expect("opening bot played illegal move");
        moves.push(mv);
    }

    moves
}

#[cfg(test)]
mod tests {
    use super::*;
    use gomoku_core::Variant;

    #[test]
    fn centered_suite_starts_at_board_center() {
        let config = RuleConfig {
            variant: Variant::Renju,
            ..Default::default()
        };

        let opening = opening_moves_for_game(OpeningPolicy::CenteredSuite, &config, 4, 7, 0);

        assert_eq!(opening[0], Move { row: 7, col: 7 });
    }

    #[test]
    fn centered_suite_openings_are_legal_under_renju() {
        let config = RuleConfig {
            variant: Variant::Renju,
            ..Default::default()
        };

        for opening_index in 0..32 {
            let mut board = Board::new(config.clone());
            for mv in opening_moves_for_game(
                OpeningPolicy::CenteredSuite,
                &config,
                CENTERED_SUITE_MAX_PLIES,
                7,
                opening_index,
            ) {
                assert!(board.is_legal(mv), "opening {opening_index} move {mv:?}");
                board.apply_move(mv).unwrap();
            }
        }
    }

    #[test]
    fn centered_suite_cycles_through_all_templates_before_repeating() {
        let config = RuleConfig {
            variant: Variant::Renju,
            ..Default::default()
        };
        let mut seen = std::collections::HashSet::new();

        for opening_index in 0..32 {
            let opening = opening_moves_for_game(
                OpeningPolicy::CenteredSuite,
                &config,
                CENTERED_SUITE_MAX_PLIES,
                7,
                opening_index,
            );
            seen.insert(
                opening
                    .iter()
                    .map(|mv| (mv.row, mv.col))
                    .collect::<Vec<_>>(),
            );
        }

        assert_eq!(seen.len(), 32);
    }
}
