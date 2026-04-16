use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{board::Move, rules::RuleConfig, Color, GameResult, ZOBRIST_ALGORITHM, ZOBRIST_SEED};

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

/// Current replay schema version. Bump when making breaking changes to the format.
pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Replay {
    /// Schema version — increment on breaking format changes. Consumers should
    /// reject or warn on versions they don't recognise.
    pub schema_version: u32,
    /// Zobrist hash parameters. Fully describes how per-move hashes were produced.
    pub hash_algo: HashAlgo,
    pub rules: RuleConfig,
    /// Name or identifier of the black player (human name or bot name).
    pub black: String,
    /// Name or identifier of the white player (human name or bot name).
    pub white: String,
    /// Ordered list of moves; index 0 = first move (Black).
    pub moves: Vec<ReplayMove>,
    pub result: ReplayResult,
    /// Total wall-clock duration of the match in milliseconds. Optional —
    /// may be absent in replays recorded without a match timer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

impl Replay {
    pub fn new(rules: RuleConfig, black: impl Into<String>, white: impl Into<String>) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
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
        let replay: Replay = serde_json::from_str(s)?;
        if replay.schema_version != SCHEMA_VERSION {
            return Err(serde::de::Error::custom(format!(
                "unsupported replay schema version: expected {}, got {}; regenerate replay",
                SCHEMA_VERSION, replay.schema_version
            )));
        }
        Ok(replay)
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
        r.push_move(
            Move { row: 3, col: 3 },
            5,
            0xcafebabe,
            Some(serde_json::json!({"depth": 3})),
        );
        r.finish(&GameResult::Winner(Color::Black), Some(1500));

        let json = r.to_json().unwrap();
        let r2 = Replay::from_json(&json).unwrap();

        assert_eq!(r2.schema_version, SCHEMA_VERSION);
        assert_eq!(r2.black, "Alice");
        assert_eq!(r2.white, "Bob");
        assert_eq!(r2.moves[0].mv, "H8");
        assert_eq!(r2.moves[1].mv, "D4");
        assert_eq!(r2.moves[1].trace, Some(serde_json::json!({"depth": 3})));
        assert_eq!(r2.result, ReplayResult::BlackWins);
        assert_eq!(r2.duration_ms, Some(1500));
    }

    #[test]
    fn from_json_rejects_wrong_schema_version() {
        let mut r = Replay::new(RuleConfig::default(), "Alice", "Bob");
        r.push_move(Move { row: 7, col: 7 }, 100, 0xdeadbeef, None);
        r.finish(&GameResult::Ongoing, None);

        let json = r.to_json().unwrap();
        let tampered = json.replace(r#""schema_version": 1"#, r#""schema_version": 99"#);
        let err = Replay::from_json(&tampered).unwrap_err();
        assert!(err
            .to_string()
            .contains("unsupported replay schema version"));
        assert!(err.to_string().contains("expected 1"));
        assert!(err.to_string().contains("got 99"));
    }

    #[test]
    fn replay_reconstruction() {
        use crate::board::Board;

        let rules = RuleConfig::default();
        let mut board = Board::new(rules.clone());
        let mut replay = Replay::new(rules.clone(), "BlackPlayer", "WhitePlayer");

        let moves = [(7, 7), (3, 3), (7, 8), (3, 4), (7, 9)];

        for (row, col) in moves {
            let mv = Move { row, col };
            board.apply_move(mv).unwrap();
            replay.push_move(mv, 100, board.hash(), None);
        }
        replay.finish(&board.result, Some(500));

        let json = replay.to_json().unwrap();
        let loaded = Replay::from_json(&json).unwrap();
        assert_eq!(loaded.schema_version, SCHEMA_VERSION);

        let mut reconstructed = Board::new(rules.clone());
        for rm in &loaded.moves {
            let mv = Move::from_notation(&rm.mv).unwrap();
            reconstructed.apply_move(mv).unwrap();
        }

        assert_eq!(reconstructed.history.len(), moves.len());
        assert_eq!(reconstructed.result, board.result);

        let mut check_board = Board::new(rules.clone());
        for (i, rm) in loaded.moves.iter().enumerate() {
            let mv = Move::from_notation(&rm.mv).unwrap();
            check_board.apply_move(mv).unwrap();
            assert_eq!(
                check_board.hash(),
                rm.hash,
                "hash mismatch at move {}",
                i + 1
            );
        }
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
