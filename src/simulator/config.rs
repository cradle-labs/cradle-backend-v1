use serde::{Deserialize, Serialize};
use bigdecimal::BigDecimal;
use uuid::Uuid;

/// Configuration for the entire simulator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatorConfig {
    /// Scheduler configuration
    pub scheduler: SchedulerConfig,

    /// Processor configuration
    pub processor: ProcessorConfig,

    /// Budget configuration per account/asset
    pub budget: BudgetConfig,

    /// State persistence directory
    pub state_dir: String,
}

/// Scheduler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    /// Minimum trade amount
    pub min_amount: BigDecimal,

    /// Maximum trade amount
    pub max_amount: BigDecimal,

    /// Trades per account
    pub trades_per_account: u32,

    /// Price offset for bid orders
    pub bid_price_offset: f64,

    /// Price offset for ask orders
    pub ask_price_offset: f64,

    /// Whether to alternate buy/sell
    pub alternate_sides: bool,
}

/// Processor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessorConfig {
    /// Base delay for exponential backoff (milliseconds)
    pub retry_base_delay_ms: u64,

    /// Maximum retries per slot
    pub max_retries: u32,

    /// Whether to save state after each slot
    pub save_after_each_slot: bool,
}

/// Budget configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConfig {
    /// Initial budget per asset per account
    pub budgets: Vec<BudgetSpec>,
}

/// Budget specification for an account/asset pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetSpec {
    /// Account ID (or "all" for all accounts)
    pub account_id: Option<Uuid>,

    /// Asset ID (or "all" for all assets)
    pub asset_id: Option<Uuid>,

    /// Budget amount
    pub amount: BigDecimal,
}

impl Default for SimulatorConfig {
    fn default() -> Self {
        Self {
            scheduler: SchedulerConfig {
                min_amount: BigDecimal::from(10),
                max_amount: BigDecimal::from(1000),
                trades_per_account: 5,
                bid_price_offset: 1.0,
                ask_price_offset: 1.0,
                alternate_sides: true,
            },
            processor: ProcessorConfig {
                retry_base_delay_ms: 500,
                max_retries: 3,
                save_after_each_slot: true,
            },
            budget: BudgetConfig {
                budgets: Vec::new(),
            },
            state_dir: "./simulator_state".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SimulatorConfig::default();
        assert_eq!(config.scheduler.min_amount, BigDecimal::from(10));
        assert_eq!(config.processor.max_retries, 3);
    }

    #[test]
    fn test_config_serialization() {
        let config = SimulatorConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: SimulatorConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.scheduler.min_amount, config.scheduler.min_amount);
    }
}
