use bigdecimal::BigDecimal;
use uuid::Uuid;
use crate::market::db_types::MarketRecord;

/// Enforces price discipline rules for markets
pub struct MarketDiscipline {
    market: MarketRecord,
    min_price: Option<BigDecimal>,
    max_price: Option<BigDecimal>,
}

impl MarketDiscipline {
    /// Create discipline rules for a market
    pub fn new(market: MarketRecord) -> Self {
        // Regulated markets have tighter price bands
        let (min_price, max_price) = match market.market_regulation {
            crate::market::db_types::MarketRegulation::Regulated => {
                // Example: regulated markets allow ±10% from some reference price
                // In practice, this might come from the market config or time series data
                (Some(BigDecimal::from(90)), Some(BigDecimal::from(110)))
            }
            crate::market::db_types::MarketRegulation::Unregulated => {
                // Unregulated markets have no price restrictions
                (None, None)
            }
        };

        Self {
            market,
            min_price,
            max_price,
        }
    }

    /// Check if price is allowed
    pub fn is_price_valid(&self, price: &BigDecimal) -> bool {
        match (&self.min_price, &self.max_price) {
            (Some(min), Some(max)) => price >= min && price <= max,
            (Some(min), None) => price >= min,
            (None, Some(max)) => price <= max,
            (None, None) => true,
        }
    }

    /// Get error if price is invalid
    pub fn validate_price(&self, price: &BigDecimal) -> Result<(), String> {
        if !self.is_price_valid(price) {
            return Err(format!(
                "Price {} outside allowed range [{:?}, {:?}] for market {}",
                price, self.min_price, self.max_price, self.market.id
            ));
        }
        Ok(())
    }

    /// Get the market ID
    pub fn market_id(&self) -> Uuid {
        self.market.id
    }

    /// Get the market
    pub fn market(&self) -> &MarketRecord {
        &self.market
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::market::db_types::{MarketStatus, MarketType};
    use chrono::Utc;

    fn create_test_market(regulation: crate::market::db_types::MarketRegulation) -> MarketRecord {
        MarketRecord {
            id: Uuid::new_v4(),
            name: "Test Market".to_string(),
            description: None,
            icon: None,
            asset_one: Uuid::new_v4(),
            asset_two: Uuid::new_v4(),
            created_at: Utc::now().naive_utc(),
            market_type: MarketType::Spot,
            market_status: MarketStatus::Active,
            market_regulation: regulation,
        }
    }

    #[test]
    fn test_regulated_market_price_bounds() {
        let market = create_test_market(crate::market::db_types::MarketRegulation::Regulated);
        let discipline = MarketDiscipline::new(market);

        assert!(discipline.is_price_valid(&BigDecimal::from(100)));
        assert!(discipline.is_price_valid(&BigDecimal::from(90)));
        assert!(discipline.is_price_valid(&BigDecimal::from(110)));
        assert!(!discipline.is_price_valid(&BigDecimal::from(80)));
        assert!(!discipline.is_price_valid(&BigDecimal::from(120)));
    }

    #[test]
    fn test_unregulated_market_no_bounds() {
        let market = create_test_market(crate::market::db_types::MarketRegulation::Unregulated);
        let discipline = MarketDiscipline::new(market);

        assert!(discipline.is_price_valid(&BigDecimal::from(1)));
        assert!(discipline.is_price_valid(&BigDecimal::from(1000)));
        assert!(discipline.is_price_valid(&BigDecimal::from(999999)));
    }
}
