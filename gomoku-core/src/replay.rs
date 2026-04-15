use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{board::Move, rules::RuleConfig, GameResult, Color, ZOBRIST_SEED, ZOBRIST_ALGORITHM};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReplayResult {
    BlackWins,
    WhiteWins,
    Draw,
    Ongoing,
}

impl From<&GameResult> for ReplayResult {
    fn from(r: &GameResult) -> Self {
        match r {
            GameResult::Winner(Color::Black) => ReplayResult::BlackWins,
            GameResult::Winner(Color::White) => ReplayResult::WhiteWins,
            GameResult::Draw => ReplayResult::Draw,
            GameResult::Ongoing => ReplayResult::Ongoing,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayMove {
    /// Move in display notation, e.g. "H8".
    pub mv: String,
    /// Wall-clock thinking time for this move in milliseconds.
    pub time_ms: u64,
    /// Zobrist hash of the position after this move.
    pub hash: u64,
    /// Optional freeform bot trace output (depth, nodes, score, etc.).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HashAlgo {
    /// PRNG algorithm used to generate the Zobrist table, e.g. "xorshift64".
    pub algorithm: String,
    /// Seed passed to the PRNG. Together with `algorithm` and `rules.board_size`,
    /// this fully determines the table and allows any stored hash to be verified.
    pub seed: u64,
}

impl HashAlgo {
    pub fn current() -> Self {
        Self {
            algorithm: ZOBRIST_ALGORITHM.to_string(),
            seed: ZOBRIST_SEED,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Replay {
    /// Zobrist hash parameters. Fully describes how per-move hashes were produced.
    pub hash_algo: HashAlgo,
    pub rules: RuleConfig,
    pub black: String,
    pub white: String,
    pub moves: Vec<ReplayMove>,
    pub result: ReplayResult,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

impl Replay {
    pub fn new(rules: RuleConfig, black: impl Into<String>, white: impl Into<String>) -> Self {
        Self {
            hash_algo: HashAlgo::current(),
            rules,
            black: black.into(),
            white: white.into(),
            moves: Vec::new(),
            result: ReplayResult::Ongoing,
            duration_ms: None,
        }
    }

    pub fn push_move(&mut self, mv: Move, time_ms: u64, hash: u64, trace: Option<Value>) {
        self.moves.push(ReplayMove {
            mv: mv.to_notation(),
            time_ms,
            hash,
            trace,
        });
    }

    pub fn finish(&mut self, result: &GameResult, duration_ms: Option<u64>) {
        self.result = result.into();
        self.duration_ms = duration_ms;
    }

    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(s: &str) -> serde_json::Result<Self> {
        serde_json::from_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::RuleConfig;

    #[test]
    fn replay_round_trip() {
        let mut r = Replay::new(RuleConfig::default(), "Alice", "Bob");
        r.push_move(Move { row: 7, col: 7 }, 100, 0xdeadbeef, None);
        r.push_move(Move { row: 3, col: 3 }, 5, 0xcafebabe, Some(serde_json::json!({"depth": 3})));
        r.finish(&GameResult::Winner(Color::Black), Some(1500));

        let json = r.to_json().unwrap();
        let r2 = Replay::from_json(&json).unwrap();

        assert_eq!(r2.black, "Alice");
        assert_eq!(r2.white, "Bob");
        assert_eq!(r2.moves[0].mv, "H8");
        assert_eq!(r2.moves[1].mv, "D4");
        assert_eq!(r2.moves[1].trace, Some(serde_json::json!({"depth": 3})));
        assert_eq!(r2.result, ReplayResult::BlackWins);
        assert_eq!(r2.duration_ms, Some(1500));
    }

    #[test]
    fn move_notation_round_trip() {
        let mv = Move { row: 7, col: 7 };
        assert_eq!(mv.to_notation(), "H8");
        assert_eq!(Move::from_notation("H8").unwrap(), mv);

        let mv2 = Move { row: 0, col: 0 };
        assert_eq!(mv2.to_notation(), "A1");
        assert_eq!(Move::from_notation("A1").unwrap(), mv2);

        let mv3 = Move { row: 14, col: 14 };
        assert_eq!(mv3.to_notation(), "O15");
        assert_eq!(Move::from_notation("O15").unwrap(), mv3);
    }
}
