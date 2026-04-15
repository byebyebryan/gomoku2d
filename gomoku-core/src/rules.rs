use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleConfig {
    pub board_size: usize,
    pub win_length: usize,
}

impl Default for RuleConfig {
    fn default() -> Self {
        Self {
            board_size: 15,
            win_length: 5,
        }
    }
}
