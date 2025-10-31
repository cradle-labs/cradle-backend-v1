use bigdecimal::BigDecimal;
use bigdecimal::ToPrimitive;
use uuid::Uuid;
use std::collections::HashMap;
use super::models::{AccountBudget, BudgetSnapshot};

/// In-memory storage for account budgets
/// Maps (account_id, asset_id) -> AccountBudget
#[derive(Debug, Clone)]
pub struct BudgetStore {
    budgets: HashMap<(Uuid, Uuid), AccountBudget>,
    history: Vec<BudgetSnapshot>,
}

impl BudgetStore {
    /// Create a new empty budget store
    pub fn new() -> Self {
        Self {
            budgets: HashMap::new(),
            history: Vec::new(),
        }
    }

    /// Initialize budget for an account/asset pair
    pub fn set_budget(
        &mut self,
        account_id: Uuid,
        asset_id: Uuid,
        initial_budget: BigDecimal,
    ) -> Result<(), String> {
        let key = (account_id, asset_id);

        if self.budgets.contains_key(&key) {
            return Err(format!(
                "Budget already exists for account {} and asset {}",
                account_id, asset_id
            ));
        }

        self.budgets.insert(key, AccountBudget::new(account_id, asset_id, initial_budget));
        Ok(())
    }

    /// Get a budget (immutable)
    pub fn get(&self, account_id: Uuid, asset_id: Uuid) -> Option<AccountBudget> {
        self.budgets.get(&(account_id, asset_id)).cloned()
    }

    /// Get a budget (mutable) for operations
    pub fn get_mut(&mut self, account_id: Uuid, asset_id: Uuid) -> Option<&mut AccountBudget> {
        self.budgets.get_mut(&(account_id, asset_id))
    }

    /// Lock budget for an order
    pub fn lock(&mut self, account_id: Uuid, asset_id: Uuid, amount: BigDecimal) -> Result<(), String> {
        match self.get_mut(account_id, asset_id) {
            Some(budget) => {
                budget.lock(amount)?;
                self.record_snapshot(account_id, asset_id);
                Ok(())
            }
            None => Err(format!(
                "No budget found for account {} and asset {}",
                account_id, asset_id
            )),
        }
    }

    /// Unlock budget when order is cancelled
    pub fn unlock(&mut self, account_id: Uuid, asset_id: Uuid, amount: BigDecimal) -> Result<(), String> {
        match self.get_mut(account_id, asset_id) {
            Some(budget) => {
                budget.unlock(amount)?;
                self.record_snapshot(account_id, asset_id);
                Ok(())
            }
            None => Err(format!(
                "No budget found for account {} and asset {}",
                account_id, asset_id
            )),
        }
    }

    /// Spend budget when order settles
    pub fn spend(&mut self, account_id: Uuid, asset_id: Uuid, amount: BigDecimal) -> Result<(), String> {
        match self.get_mut(account_id, asset_id) {
            Some(budget) => {
                budget.spend(amount)?;
                self.record_snapshot(account_id, asset_id);
                Ok(())
            }
            None => Err(format!(
                "No budget found for account {} and asset {}",
                account_id, asset_id
            )),
        }
    }

    /// Check if budget is available
    pub fn has_available(&self, account_id: Uuid, asset_id: Uuid, amount: &BigDecimal) -> bool {
        self.get(account_id, asset_id)
            .map(|b| b.has_available(amount))
            .unwrap_or(false)
    }

    /// Check if asset budget is depleted
    pub fn is_depleted(&self, account_id: Uuid, asset_id: Uuid) -> bool {
        self.get(account_id, asset_id)
            .map(|b| b.is_depleted())
            .unwrap_or(true)
    }

    /// Get all budgets for an account
    pub fn get_account_budgets(&self, account_id: Uuid) -> Vec<AccountBudget> {
        self.budgets
            .values()
            .filter(|b| b.account_id == account_id)
            .cloned()
            .collect()
    }

    /// Get all assets with budget for an account
    pub fn get_account_assets(&self, account_id: Uuid) -> Vec<Uuid> {
        self.get_account_budgets(account_id)
            .into_iter()
            .map(|b| b.asset_id)
            .collect()
    }

    /// Get all accounts with budget for an asset
    pub fn get_asset_accounts(&self, asset_id: Uuid) -> Vec<Uuid> {
        self.budgets
            .values()
            .filter(|b| b.asset_id == asset_id)
            .map(|b| b.account_id)
            .collect()
    }

    /// Record a snapshot of current state
    fn record_snapshot(&mut self, account_id: Uuid, asset_id: Uuid) {
        if let Some(budget) = self.get(account_id, asset_id) {
            self.history.push(BudgetSnapshot::from(&budget));
        }
    }

    /// Get budget history
    pub fn get_history(&self) -> &[BudgetSnapshot] {
        &self.history
    }

    /// Clear history (useful for memory management)
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// Get summary for all budgets
    pub fn get_summary(&self) -> BudgetSummary {
        let mut summary = BudgetSummary::default();

        for budget in self.budgets.values() {
            summary.total_initial += budget.initial_budget.clone();
            summary.total_spent += budget.spent.clone();
            summary.total_available += budget.available.clone();
            summary.total_locked += budget.locked.clone();
            summary.budget_count += 1;

            if budget.is_depleted() {
                summary.depleted_count += 1;
            }
        }

        summary
    }
}

impl Default for BudgetStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of all budgets in the store
#[derive(Debug, Clone, Default)]
pub struct BudgetSummary {
    pub total_initial: BigDecimal,
    pub total_spent: BigDecimal,
    pub total_available: BigDecimal,
    pub total_locked: BigDecimal,
    pub budget_count: usize,
    pub depleted_count: usize,
}

impl BudgetSummary {
    pub fn total_utilization_percent(&self) -> f64 {
        if self.total_initial == BigDecimal::from(0) {
            return 0.0;
        }
        (self.total_spent.to_f64().unwrap_or(0.0) / self.total_initial.to_f64().unwrap_or(1.0)) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_store_creation() {
        let store = BudgetStore::new();
        assert_eq!(store.budgets.len(), 0);
    }

    #[test]
    fn test_set_and_get_budget() {
        let mut store = BudgetStore::new();
        let account_id = Uuid::new_v4();
        let asset_id = Uuid::new_v4();

        assert!(store
            .set_budget(account_id, asset_id, BigDecimal::from(1000))
            .is_ok());
        assert!(store.get(account_id, asset_id).is_some());
    }

    #[test]
    fn test_duplicate_budget_rejected() {
        let mut store = BudgetStore::new();
        let account_id = Uuid::new_v4();
        let asset_id = Uuid::new_v4();

        assert!(store
            .set_budget(account_id, asset_id, BigDecimal::from(1000))
            .is_ok());
        assert!(store
            .set_budget(account_id, asset_id, BigDecimal::from(2000))
            .is_err());
    }

    #[test]
    fn test_lock_and_unlock_operations() {
        let mut store = BudgetStore::new();
        let account_id = Uuid::new_v4();
        let asset_id = Uuid::new_v4();

        store
            .set_budget(account_id, asset_id, BigDecimal::from(1000))
            .unwrap();

        assert!(store.lock(account_id, asset_id, BigDecimal::from(500)).is_ok());
        assert!(store.has_available(account_id, asset_id, &BigDecimal::from(500)));

        assert!(store.unlock(account_id, asset_id, BigDecimal::from(300)).is_ok());
        assert!(store.has_available(account_id, asset_id, &BigDecimal::from(800)));
    }

    #[test]
    fn test_get_account_budgets() {
        let mut store = BudgetStore::new();
        let account_id = Uuid::new_v4();
        let asset_1 = Uuid::new_v4();
        let asset_2 = Uuid::new_v4();

        store.set_budget(account_id, asset_1, BigDecimal::from(1000)).unwrap();
        store.set_budget(account_id, asset_2, BigDecimal::from(2000)).unwrap();

        let budgets = store.get_account_budgets(account_id);
        assert_eq!(budgets.len(), 2);
    }

    #[test]
    fn test_summary() {
        let mut store = BudgetStore::new();
        let account_id = Uuid::new_v4();
        let asset_id = Uuid::new_v4();

        store.set_budget(account_id, asset_id, BigDecimal::from(1000)).unwrap();
        store.lock(account_id, asset_id, BigDecimal::from(500)).unwrap();
        store.spend(account_id, asset_id, BigDecimal::from(500)).unwrap();

        let summary = store.get_summary();
        assert_eq!(summary.total_initial, BigDecimal::from(1000));
        assert_eq!(summary.total_spent, BigDecimal::from(500));
        assert_eq!(summary.budget_count, 1);
    }
}
