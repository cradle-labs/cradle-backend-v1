use bigdecimal::BigDecimal;
use bigdecimal::ToPrimitive;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Tracks budget for a specific account and asset pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountBudget {
    /// Account ID
    pub account_id: Uuid,

    /// Asset ID
    pub asset_id: Uuid,

    /// Initial budget for this asset
    pub initial_budget: BigDecimal,

    /// Remaining available budget
    pub available: BigDecimal,

    /// Amount locked in active orders
    pub locked: BigDecimal,

    /// Total amount spent so far
    pub spent: BigDecimal,

    /// When this budget was created
    pub created_at: DateTime<Utc>,

    /// When budget was last updated
    pub updated_at: DateTime<Utc>,
}

impl AccountBudget {
    /// Create a new budget
    pub fn new(account_id: Uuid, asset_id: Uuid, initial_budget: BigDecimal) -> Self {
        let now = Utc::now();
        Self {
            account_id,
            asset_id,
            initial_budget: initial_budget.clone(),
            available: initial_budget,
            locked: BigDecimal::from(0),
            spent: BigDecimal::from(0),
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if enough budget is available for an amount
    pub fn has_available(&self, amount: &BigDecimal) -> bool {
        amount <= &self.available
    }

    /// Check if amount + locked is within initial budget
    pub fn can_lock(&self, amount: &BigDecimal) -> bool {
        let total_needed = self.locked.clone() + amount;
        total_needed <= self.initial_budget
    }

    /// Lock an amount (when placing order)
    /// Returns error if insufficient budget
    pub fn lock(&mut self, amount: BigDecimal) -> Result<(), String> {
        if !self.can_lock(&amount) {
            return Err(format!(
                "Cannot lock {} - total would be {}, but budget is {}",
                amount,
                &self.locked + &amount,
                self.initial_budget
            ));
        }
        self.locked = self.locked.clone() + amount;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Unlock an amount (when canceling order)
    pub fn unlock(&mut self, amount: BigDecimal) -> Result<(), String> {
        if amount > self.locked {
            return Err(format!("Cannot unlock {} - only {} locked", amount, self.locked));
        }
        self.locked = self.locked.clone() - amount.clone();
        self.available = self.available.clone() + amount;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Spend an amount (when order settles)
    pub fn spend(&mut self, amount: BigDecimal) -> Result<(), String> {
        if amount > self.locked {
            return Err(format!("Cannot spend {} - only {} locked", amount, self.locked));
        }
        self.locked = self.locked.clone() - amount.clone();
        self.available = self.available.clone() - amount.clone();
        self.spent = self.spent.clone() + amount;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Get the total amount still available or locked
    pub fn total_remaining(&self) -> BigDecimal {
        self.available.clone() + self.locked.clone()
    }

    /// Check if budget is depleted
    pub fn is_depleted(&self) -> bool {
        self.total_remaining() <= BigDecimal::from(0)
    }

    /// Get budget utilization percentage
    pub fn utilization_percent(&self) -> f64 {
        if self.initial_budget == BigDecimal::from(0) {
            return 0.0;
        }
        (self.spent.to_f64().unwrap_or(0.0) / self.initial_budget.to_f64().unwrap_or(1.0)) * 100.0
    }
}

/// Snapshot of budget state at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetSnapshot {
    pub account_id: Uuid,
    pub asset_id: Uuid,
    pub initial_budget: BigDecimal,
    pub available: BigDecimal,
    pub locked: BigDecimal,
    pub spent: BigDecimal,
    pub timestamp: DateTime<Utc>,
}

impl From<&AccountBudget> for BudgetSnapshot {
    fn from(budget: &AccountBudget) -> Self {
        Self {
            account_id: budget.account_id,
            asset_id: budget.asset_id,
            initial_budget: budget.initial_budget.clone(),
            available: budget.available.clone(),
            locked: budget.locked.clone(),
            spent: budget.spent.clone(),
            timestamp: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_creation() {
        let budget = AccountBudget::new(Uuid::new_v4(), Uuid::new_v4(), BigDecimal::from(1000));
        assert_eq!(budget.available, BigDecimal::from(1000));
        assert_eq!(budget.locked, BigDecimal::from(0));
        assert_eq!(budget.spent, BigDecimal::from(0));
        assert!(!budget.is_depleted());
    }

    #[test]
    fn test_lock_and_unlock() {
        let mut budget = AccountBudget::new(Uuid::new_v4(), Uuid::new_v4(), BigDecimal::from(1000));

        assert!(budget.lock(BigDecimal::from(500)).is_ok());
        assert_eq!(budget.available, BigDecimal::from(1000));
        assert_eq!(budget.locked, BigDecimal::from(500));

        assert!(budget.unlock(BigDecimal::from(300)).is_ok());
        assert_eq!(budget.available, BigDecimal::from(1300));
        assert_eq!(budget.locked, BigDecimal::from(200));
    }

    #[test]
    fn test_spend() {
        let mut budget = AccountBudget::new(Uuid::new_v4(), Uuid::new_v4(), BigDecimal::from(1000));

        assert!(budget.lock(BigDecimal::from(500)).is_ok());
        assert!(budget.spend(BigDecimal::from(500)).is_ok());
        assert_eq!(budget.available, BigDecimal::from(500));
        assert_eq!(budget.locked, BigDecimal::from(0));
        assert_eq!(budget.spent, BigDecimal::from(500));
    }

    #[test]
    fn test_insufficient_budget() {
        let mut budget = AccountBudget::new(Uuid::new_v4(), Uuid::new_v4(), BigDecimal::from(1000));

        assert!(budget.lock(BigDecimal::from(1500)).is_err());
    }

    #[test]
    fn test_utilization() {
        let mut budget = AccountBudget::new(Uuid::new_v4(), Uuid::new_v4(), BigDecimal::from(1000));
        assert_eq!(budget.utilization_percent(), 0.0);

        budget.lock(BigDecimal::from(500)).unwrap();
        budget.spend(BigDecimal::from(500)).unwrap();
        assert_eq!(budget.utilization_percent(), 50.0);
    }
}
