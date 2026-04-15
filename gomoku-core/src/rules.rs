use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Variant {
    #[default]
    Freestyle,
    Renju,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleConfig {
    pub board_size: usize,
    pub win_length: usize,
    #[serde(default)]
    pub variant: Variant,
}

impl Default for RuleConfig {
    fn default() -> Self {
        Self {
            board_size: 15,
            win_length: 5,
            variant: Variant::Freestyle,
        }
    }
}
