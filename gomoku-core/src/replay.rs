use serde::{Deserialize, Serialize};

use crate::{board::Move, rules::RuleConfig, GameResult, Color};

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
pub struct Replay {
    pub rules: RuleConfig,
    pub black: String,
    pub white: String,
    pub moves: Vec<[usize; 2]>,
    pub result: ReplayResult,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

impl Replay {
    pub fn new(rules: RuleConfig, black: impl Into<String>, white: impl Into<String>) -> Self {
        Self {
            rules,
            black: black.into(),
            white: white.into(),
            moves: Vec::new(),
            result: ReplayResult::Ongoing,
            duration_ms: None,
        }
    }

    pub fn push_move(&mut self, mv: Move) {
        self.moves.push([mv.row, mv.col]);
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
        r.push_move(Move { row: 7, col: 7 });
        r.push_move(Move { row: 3, col: 3 });
        r.finish(&GameResult::Winner(Color::Black), Some(1500));

        let json = r.to_json().unwrap();
        let r2 = Replay::from_json(&json).unwrap();

        assert_eq!(r2.black, "Alice");
        assert_eq!(r2.white, "Bob");
        assert_eq!(r2.moves, vec![[7, 7], [3, 3]]);
        assert_eq!(r2.result, ReplayResult::BlackWins);
        assert_eq!(r2.duration_ms, Some(1500));
    }
}
