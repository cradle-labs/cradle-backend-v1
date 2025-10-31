use bigdecimal::BigDecimal;
use bigdecimal::ToPrimitive;
use rand::Rng;
use uuid::Uuid;
use anyhow::{anyhow, Result};

use super::models::{ActionSlot, OrderAction, OrderActionSide, OrderMatchingStrategy};

/// Configuration for the scheduler
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    /// Maximum amount per trade (will be randomized up to this)
    pub max_amount: BigDecimal,

    /// Minimum amount per trade
    pub min_amount: BigDecimal,

    /// Price multiplier for bid orders (how much to ask relative to market)
    pub bid_price_offset: f64,

    /// Price multiplier for ask orders
    pub ask_price_offset: f64,

    /// How many trades to schedule per account
    pub trades_per_account: u32,

    /// How to distribute accounts across markets
    pub market_distribution: MarketDistribution,

    /// Whether to alternate buy/sell per account
    pub alternate_sides: bool,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            max_amount: BigDecimal::from(1000),
            min_amount: BigDecimal::from(10),
            bid_price_offset: 1.0,
            ask_price_offset: 1.0,
            trades_per_account: 5,
            market_distribution: MarketDistribution::RoundRobin,
            alternate_sides: true,
        }
    }
}

/// How to distribute trading across markets
#[derive(Debug, Clone, Copy)]
pub enum MarketDistribution {
    /// Each account trades each market in order
    RoundRobin,
    /// Each account trades the same market
    SameMarket,
    /// Accounts spread across markets sequentially
    Sequential,
}

/// Generates trading action slots
pub struct SlotScheduler {
    config: SchedulerConfig,
}

impl SlotScheduler {
    pub fn new(config: SchedulerConfig) -> Self {
        Self { config }
    }

    /// Generate slots for all accounts trading multiple markets
    /// markets_info: Vec of (market_id, asset_one, asset_two) tuples
    pub fn generate_schedule(
        &self,
        accounts: &[Uuid],
        markets: &[Uuid],
        markets_info: &[(Uuid, Uuid, Uuid)],
    ) -> Result<Vec<ActionSlot>> {
        if accounts.is_empty() {
            return Err(anyhow!("No accounts provided"));
        }
        if markets.is_empty() {
            return Err(anyhow!("No markets provided"));
        }

        let mut slots = Vec::new();
        let mut sequence = 0u32;
        let mut rng = rand::thread_rng();

        // For each account
        for (account_idx, &account_id) in accounts.iter().enumerate() {
            // Schedule N trades for this account
            for trade_idx in 0..self.config.trades_per_account {
                // Determine which market to trade
                let market_idx = match self.config.market_distribution {
                    MarketDistribution::RoundRobin => (trade_idx as usize) % markets.len(),
                    MarketDistribution::SameMarket => account_idx % markets.len(),
                    MarketDistribution::Sequential => (account_idx + trade_idx as usize) % markets.len(),
                };
                let market_id = markets[market_idx];

                // Get assets from market info and alternate based on account index for complementary orders
                let (bid_asset, ask_asset) = if let Some((_, asset_one, asset_two)) =
                    markets_info.iter().find(|(m_id, _, _)| *m_id == market_id) {
                    // Alternate asset pairs: even accounts go asset_one→asset_two, odd accounts go asset_two→asset_one
                    // This ensures complementary orders can match
                    if account_idx % 2 == 0 {
                        (*asset_one, *asset_two)
                    } else {
                        (*asset_two, *asset_one)
                    }
                } else {
                    return Err(anyhow!("Market {} not found in markets_info", market_id));
                };

                // Determine side (bid/ask)
                let side = if self.config.alternate_sides {
                    if account_idx % 2 == 0 {
                        if trade_idx % 2 == 0 {
                            OrderActionSide::Bid
                        } else {
                            OrderActionSide::Ask
                        }
                    } else {
                        if trade_idx % 2 == 0 {
                            OrderActionSide::Ask
                        } else {
                            OrderActionSide::Bid
                        }
                    }
                } else {
                    if account_idx % 2 == 0 {
                        OrderActionSide::Bid
                    } else {
                        OrderActionSide::Ask
                    }
                };

                // Determine matching strategy
                // Even accounts match with next odd account, odd accounts match with previous even
                let matching_strategy = if account_idx < accounts.len() - 1 {
                    if side == OrderActionSide::Bid {
                        // Bidders match with next asker
                        let asker_idx = (account_idx + 1) % accounts.len();
                        OrderMatchingStrategy::MatchWith(accounts[asker_idx])
                    } else {
                        // Askers match with previous bidder
                        let bidder_idx = if account_idx == 0 {
                            accounts.len() - 1
                        } else {
                            account_idx - 1
                        };
                        OrderMatchingStrategy::MatchWith(accounts[bidder_idx])
                    }
                } else {
                    OrderMatchingStrategy::SequentialNext
                };

                // Generate random amounts
                let bid_amount = self.random_amount(&mut rng);
                let ask_amount = (bid_amount.to_f64().unwrap_or(0.0) * 0.95) as i32; // 5% price difference
                let ask_amount = BigDecimal::from(ask_amount.max(1));

                // Calculate price: for odd accounts (swapped assets), invert the price ratio
                let price = if account_idx % 2 == 0 {
                    ask_amount.clone() / bid_amount.clone()
                } else {
                    // Swapped assets: invert the price to maintain correct pricing relationship
                    bid_amount.clone() / ask_amount.clone()
                };

                // Create action with actual assets from the market
                let action = OrderAction {
                    market_id,
                    bid_asset,
                    ask_asset,
                    bid_amount,
                    ask_amount,
                    side,
                    price,
                    matching_strategy,
                };

                let slot = ActionSlot::new(sequence, account_id, action, 3);
                slots.push(slot);
                sequence += 1;
            }
        }

        Ok(slots)
    }

    /// Generate random amount within configured range
    fn random_amount(&self, rng: &mut rand::rngs::ThreadRng) -> BigDecimal {
        let min = self.config.min_amount.to_f64().unwrap_or(10.0);
        let max = self.config.max_amount.to_f64().unwrap_or(1000.0);
        let amount = rng.gen_range(min..=max);
        BigDecimal::from(amount as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduler_generation() {
        let config = SchedulerConfig::default();
        let scheduler = SlotScheduler::new(config);

        let accounts = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];
        let market_1 = Uuid::new_v4();
        let market_2 = Uuid::new_v4();
        let asset_1 = Uuid::new_v4();
        let asset_2 = Uuid::new_v4();
        let asset_3 = Uuid::new_v4();

        let markets = vec![market_1, market_2];
        let markets_info = vec![
            (market_1, asset_1, asset_2),
            (market_2, asset_2, asset_3),
        ];

        let slots = scheduler.generate_schedule(&accounts, &markets, &markets_info).unwrap();

        // Should have 3 accounts * 5 trades each = 15 slots
        assert_eq!(slots.len(), 15);

        // Check sequences are sequential
        for (i, slot) in slots.iter().enumerate() {
            assert_eq!(slot.sequence, i as u32);
        }

        // Check that assets come from the market, not random
        for slot in slots.iter() {
            let market_assets = markets_info.iter()
                .find(|(m_id, _, _)| *m_id == slot.action.market_id)
                .map(|(_, a1, a2)| (a1, a2));
            assert!(market_assets.is_some(), "Market {} not found in markets_info", slot.action.market_id);
            let (asset_one, asset_two) = market_assets.unwrap();
            // Assets should match the market, not be random
            assert!(
                (slot.action.bid_asset == *asset_one && slot.action.ask_asset == *asset_two) ||
                (slot.action.bid_asset == *asset_two && slot.action.ask_asset == *asset_one),
                "Assets for market {} don't match market info",
                slot.action.market_id
            );
        }
    }

    #[test]
    fn test_alternating_sides() {
        let config = SchedulerConfig {
            alternate_sides: true,
            ..Default::default()
        };
        let scheduler = SlotScheduler::new(config);

        let accounts = vec![Uuid::new_v4(), Uuid::new_v4()];
        let market = Uuid::new_v4();
        let asset_1 = Uuid::new_v4();
        let asset_2 = Uuid::new_v4();

        let markets = vec![market];
        let markets_info = vec![(market, asset_1, asset_2)];

        let slots = scheduler.generate_schedule(&accounts, &markets, &markets_info).unwrap();

        // First account should have alternating sides
        let first_account_slots: Vec<_> = slots.iter().filter(|s| s.account_id == accounts[0]).collect();
        assert!(first_account_slots[0].action.side == OrderActionSide::Bid);
        assert!(first_account_slots[1].action.side == OrderActionSide::Ask);
    }

    #[test]
    fn test_no_accounts_error() {
        let config = SchedulerConfig::default();
        let scheduler = SlotScheduler::new(config);

        let accounts = vec![];
        let market = Uuid::new_v4();
        let markets = vec![market];
        let markets_info = vec![(market, Uuid::new_v4(), Uuid::new_v4())];

        let result = scheduler.generate_schedule(&accounts, &markets, &markets_info);
        assert!(result.is_err());
    }

    #[test]
    fn test_no_markets_error() {
        let config = SchedulerConfig::default();
        let scheduler = SlotScheduler::new(config);

        let accounts = vec![Uuid::new_v4()];
        let markets = vec![];
        let markets_info = vec![];

        let result = scheduler.generate_schedule(&accounts, &markets, &markets_info);
        assert!(result.is_err());
    }
}
