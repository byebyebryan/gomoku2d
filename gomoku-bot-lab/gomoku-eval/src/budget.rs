use gomoku_bot::{Bot, SearchBot, SearchBotConfig};
use gomoku_core::{Board, Move};
use serde_json::{json, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PooledCpuBudgetConfig {
    pub base_ms: u64,
    pub reserve_cap_ms: u64,
    pub max_move_ms: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PooledCpuBudgetSnapshot {
    pub base_ms: u64,
    pub move_budget_ms: u64,
    pub reserve_cap_ms: u64,
    pub max_move_ms: Option<u64>,
    pub reserve_before_ms: u64,
    pub reserve_after_ms: u64,
    pub consumed_ms: u64,
    pub budget_exhausted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PooledCpuBudget {
    config: PooledCpuBudgetConfig,
    reserve_ms: u64,
}

impl PooledCpuBudget {
    pub fn new(config: PooledCpuBudgetConfig) -> Self {
        Self {
            config,
            reserve_ms: 0,
        }
    }

    pub fn reserve_ms(self) -> u64 {
        self.reserve_ms
    }

    pub fn next_budget_ms(self) -> u64 {
        let pooled_budget = self.config.base_ms.saturating_add(self.reserve_ms);
        self.config
            .max_move_ms
            .map_or(pooled_budget, |max_move_ms| pooled_budget.min(max_move_ms))
    }

    pub fn record_move(&mut self, consumed_ms: u64) -> PooledCpuBudgetSnapshot {
        let reserve_before_ms = self.reserve_ms;
        let move_budget_ms = self.next_budget_ms();
        let reserve_after_ms = if consumed_ms <= self.config.base_ms {
            reserve_before_ms
                .saturating_add(self.config.base_ms - consumed_ms)
                .min(self.config.reserve_cap_ms)
        } else {
            reserve_before_ms.saturating_sub(consumed_ms - self.config.base_ms)
        };

        self.reserve_ms = reserve_after_ms;

        PooledCpuBudgetSnapshot {
            base_ms: self.config.base_ms,
            move_budget_ms,
            reserve_cap_ms: self.config.reserve_cap_ms,
            max_move_ms: self.config.max_move_ms,
            reserve_before_ms,
            reserve_after_ms,
            consumed_ms,
            budget_exhausted: consumed_ms >= move_budget_ms && move_budget_ms > 0,
        }
    }
}

pub struct PooledSearchBot {
    bot: SearchBot,
    budget: PooledCpuBudget,
    last_trace: Option<Value>,
}

impl PooledSearchBot {
    pub fn new(config: SearchBotConfig, budget_config: PooledCpuBudgetConfig) -> Self {
        Self {
            bot: SearchBot::with_config(config),
            budget: PooledCpuBudget::new(budget_config),
            last_trace: None,
        }
    }
}

impl Bot for PooledSearchBot {
    fn name(&self) -> &str {
        self.bot.name()
    }

    fn choose_move(&mut self, board: &Board) -> Move {
        let move_budget_ms = self.budget.next_budget_ms();
        self.bot.set_time_budgets(None, Some(move_budget_ms));
        let mv = self.bot.choose_move(board);

        let mut trace = self.bot.trace().unwrap_or_else(|| json!({}));
        let consumed_ms = trace
            .get("cpu_time_ms")
            .and_then(Value::as_u64)
            .or_else(|| trace.get("elapsed_ms").and_then(Value::as_u64))
            .unwrap_or(0);
        let snapshot = self.budget.record_move(consumed_ms);

        if let Some(obj) = trace.as_object_mut() {
            obj.insert(
                "budget_pool".to_string(),
                json!({
                    "mode": "pooled_cpu",
                    "base_ms": snapshot.base_ms,
                    "move_budget_ms": snapshot.move_budget_ms,
                    "reserve_cap_ms": snapshot.reserve_cap_ms,
                    "max_move_ms": snapshot.max_move_ms,
                    "reserve_before_ms": snapshot.reserve_before_ms,
                    "reserve_after_ms": snapshot.reserve_after_ms,
                    "consumed_ms": snapshot.consumed_ms,
                    "budget_exhausted": snapshot.budget_exhausted,
                    "reserve_exhausted": snapshot.budget_exhausted,
                }),
            );
        }
        self.last_trace = Some(trace);

        mv
    }

    fn trace(&self) -> Option<Value> {
        self.last_trace.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pooled_cpu_budget_banks_cheap_moves_and_spends_on_slow_moves() {
        let mut budget = PooledCpuBudget::new(PooledCpuBudgetConfig {
            base_ms: 1_000,
            reserve_cap_ms: 4_000,
            max_move_ms: None,
        });

        assert_eq!(budget.next_budget_ms(), 1_000);

        let first = budget.record_move(250);
        assert_eq!(first.reserve_before_ms, 0);
        assert_eq!(first.reserve_after_ms, 750);
        assert!(!first.budget_exhausted);
        assert_eq!(budget.next_budget_ms(), 1_750);

        let second = budget.record_move(1_500);
        assert_eq!(second.reserve_before_ms, 750);
        assert_eq!(second.reserve_after_ms, 250);
        assert!(!second.budget_exhausted);
        assert_eq!(budget.next_budget_ms(), 1_250);
    }

    #[test]
    fn pooled_cpu_budget_caps_reserve_and_reports_exhaustion() {
        let mut budget = PooledCpuBudget::new(PooledCpuBudgetConfig {
            base_ms: 1_000,
            reserve_cap_ms: 1_500,
            max_move_ms: None,
        });

        budget.record_move(0);
        budget.record_move(0);
        assert_eq!(budget.reserve_ms(), 1_500);
        assert_eq!(budget.next_budget_ms(), 2_500);

        let slow = budget.record_move(3_000);
        assert_eq!(slow.reserve_before_ms, 1_500);
        assert_eq!(slow.reserve_after_ms, 0);
        assert!(slow.budget_exhausted);
    }

    #[test]
    fn pooled_cpu_budget_caps_single_move_budget_independently_from_reserve() {
        let mut budget = PooledCpuBudget::new(PooledCpuBudgetConfig {
            base_ms: 2_000,
            reserve_cap_ms: 8_000,
            max_move_ms: Some(4_000),
        });

        budget.record_move(0);
        budget.record_move(0);
        budget.record_move(0);
        budget.record_move(0);
        assert_eq!(budget.reserve_ms(), 8_000);
        assert_eq!(budget.next_budget_ms(), 4_000);

        let capped = budget.record_move(4_000);
        assert_eq!(capped.reserve_before_ms, 8_000);
        assert_eq!(capped.move_budget_ms, 4_000);
        assert_eq!(capped.max_move_ms, Some(4_000));
        assert_eq!(capped.reserve_after_ms, 6_000);
        assert!(capped.budget_exhausted);
    }

    #[test]
    fn pooled_search_bot_emits_budget_trace() {
        let board = gomoku_core::Board::new(gomoku_core::RuleConfig::default());
        let mut bot = PooledSearchBot::new(
            gomoku_bot::SearchBotConfig::custom_depth(1),
            PooledCpuBudgetConfig {
                base_ms: 100,
                reserve_cap_ms: 400,
                max_move_ms: Some(250),
            },
        );

        let _ = bot.choose_move(&board);
        let trace = bot.trace().expect("pooled search bot should emit trace");

        assert_eq!(trace["config"]["cpu_time_budget_ms"], 100);
        assert_eq!(trace["budget_pool"]["mode"], "pooled_cpu");
        assert_eq!(trace["budget_pool"]["base_ms"], 100);
        assert_eq!(trace["budget_pool"]["move_budget_ms"], 100);
        assert_eq!(trace["budget_pool"]["reserve_cap_ms"], 400);
        assert_eq!(trace["budget_pool"]["max_move_ms"], 250);
        assert_eq!(trace["budget_pool"]["budget_exhausted"], false);
    }
}
