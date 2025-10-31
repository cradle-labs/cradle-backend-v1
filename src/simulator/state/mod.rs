use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Result, anyhow};
use crate::simulator::action_slot::models::ActionSlot;
use crate::simulator::budget::storage::BudgetStore;

/// Complete simulation state that can be persisted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationState {
    /// Unique simulation ID
    pub simulation_id: Uuid,

    /// All action slots in order
    pub slots: Vec<ActionSlot>,

    /// Budget snapshots
    #[serde(skip)]
    pub budget_store: Option<BudgetStore>,

    /// Current slot being processed (index)
    pub current_slot_index: usize,

    /// When simulation started
    pub started_at: DateTime<Utc>,

    /// When simulation was last saved
    pub last_saved_at: DateTime<Utc>,

    /// Statistics
    pub stats: SimulationStats,
}

/// Statistics about simulation progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationStats {
    pub total_slots: usize,
    pub completed_slots: usize,
    pub failed_slots: usize,
    pub skipped_slots: usize,
    pub pending_slots: usize,
    pub total_orders_created: usize,
    pub total_matches: usize,
}

impl Default for SimulationStats {
    fn default() -> Self {
        Self {
            total_slots: 0,
            completed_slots: 0,
            failed_slots: 0,
            skipped_slots: 0,
            pending_slots: 0,
            total_orders_created: 0,
            total_matches: 0,
        }
    }
}

impl SimulationState {
    /// Create a new simulation state
    pub fn new(slots: Vec<ActionSlot>) -> Self {
        let total = slots.len();
        Self {
            simulation_id: Uuid::new_v4(),
            slots,
            budget_store: None,
            current_slot_index: 0,
            started_at: Utc::now(),
            last_saved_at: Utc::now(),
            stats: SimulationStats {
                total_slots: total,
                pending_slots: total,
                ..Default::default()
            },
        }
    }

    /// Update statistics from current slot states
    pub fn update_stats(&mut self) {
        use crate::simulator::action_slot::models::ActionSlotState;

        let mut stats = SimulationStats {
            total_slots: self.slots.len(),
            ..Default::default()
        };

        for slot in &self.slots {
            match slot.state {
                ActionSlotState::Completed => {
                    stats.completed_slots += 1;
                    stats.total_orders_created += 1;
                    stats.total_matches += slot.matched_order_ids.len();
                }
                ActionSlotState::Failed => {
                    stats.failed_slots += 1;
                }
                ActionSlotState::Skipped => {
                    stats.skipped_slots += 1;
                }
                ActionSlotState::Pending | ActionSlotState::InProgress => {
                    stats.pending_slots += 1;
                }
            }
        }

        self.stats = stats;
    }

    /// Get current slot
    pub fn current_slot(&self) -> Option<&ActionSlot> {
        self.slots.get(self.current_slot_index)
    }

    /// Get current slot (mutable)
    pub fn current_slot_mut(&mut self) -> Option<&mut ActionSlot> {
        self.slots.get_mut(self.current_slot_index)
    }

    /// Move to next slot
    pub fn next_slot(&mut self) {
        if self.current_slot_index < self.slots.len() {
            self.current_slot_index += 1;
        }
    }

    /// Check if simulation is complete
    pub fn is_complete(&self) -> bool {
        self.current_slot_index >= self.slots.len()
    }

    /// Get progress percentage
    pub fn progress_percent(&self) -> f64 {
        if self.slots.is_empty() {
            return 100.0;
        }
        ((self.current_slot_index as f64 / self.slots.len() as f64) * 100.0).min(100.0)
    }
}

/// Persistence manager for simulation state
pub struct StatePersistence {
    state_dir: PathBuf,
}

impl StatePersistence {
    /// Create a new state persistence manager
    pub fn new<P: AsRef<Path>>(state_dir: P) -> Result<Self> {
        let state_dir = state_dir.as_ref().to_path_buf();
        fs::create_dir_all(&state_dir)?;
        Ok(Self { state_dir })
    }

    /// Save simulation state to disk
    pub fn save(&self, state: &SimulationState) -> Result<()> {
        let file_path = self.state_dir.join(format!("{}.json", state.simulation_id));
        let json = serde_json::to_string_pretty(state)?;
        fs::write(file_path, json)?;
        Ok(())
    }

    /// Load simulation state from disk
    pub fn load(&self, simulation_id: Uuid) -> Result<SimulationState> {
        let file_path = self.state_dir.join(format!("{}.json", simulation_id));
        let json = fs::read_to_string(file_path)?;
        let state = serde_json::from_str(&json)?;
        Ok(state)
    }

    /// List all saved simulations
    pub fn list_simulations(&self) -> Result<Vec<Uuid>> {
        let mut simulations = Vec::new();

        for entry in fs::read_dir(&self.state_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                if let Some(stem) = path.file_stem() {
                    if let Some(id_str) = stem.to_str() {
                        if let Ok(id) = Uuid::parse_str(id_str) {
                            simulations.push(id);
                        }
                    }
                }
            }
        }

        Ok(simulations)
    }

    /// Delete a saved simulation
    pub fn delete(&self, simulation_id: Uuid) -> Result<()> {
        let file_path = self.state_dir.join(format!("{}.json", simulation_id));
        fs::remove_file(file_path)?;
        Ok(())
    }

    /// Get state directory path
    pub fn state_dir(&self) -> &Path {
        &self.state_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bigdecimal::BigDecimal;
    use crate::simulator::action_slot::models::{OrderAction, OrderActionSide, OrderMatchingStrategy};

    fn create_test_slot() -> ActionSlot {
        let account_id = Uuid::new_v4();
        let action = OrderAction {
            market_id: Uuid::new_v4(),
            bid_asset: Uuid::new_v4(),
            ask_asset: Uuid::new_v4(),
            bid_amount: BigDecimal::from(100),
            ask_amount: BigDecimal::from(50),
            side: OrderActionSide::Bid,
            price: BigDecimal::from(50),
            matching_strategy: OrderMatchingStrategy::SequentialNext,
        };
        ActionSlot::new(1, account_id, action, 3)
    }

    #[test]
    fn test_simulation_state_creation() {
        let slots = vec![create_test_slot()];
        let state = SimulationState::new(slots.clone());

        assert_eq!(state.slots.len(), 1);
        assert_eq!(state.current_slot_index, 0);
        assert!(!state.is_complete());
        assert_eq!(state.progress_percent(), 0.0);
    }

    #[test]
    fn test_state_persistence_save_and_load() {
        let test_dir = "./test_simulator_state";
        let _ = std::fs::remove_dir_all(test_dir); // Clean up from previous test
        let persistence = StatePersistence::new(test_dir).unwrap();

        let slots = vec![create_test_slot()];
        let mut state = SimulationState::new(slots);
        state.update_stats();

        persistence.save(&state).unwrap();

        let loaded = persistence.load(state.simulation_id).unwrap();
        assert_eq!(loaded.simulation_id, state.simulation_id);
        assert_eq!(loaded.slots.len(), state.slots.len());

        // Cleanup
        let _ = std::fs::remove_dir_all(test_dir);
    }

    #[test]
    fn test_list_simulations() {
        let test_dir = "./test_simulator_state_list";
        let _ = std::fs::remove_dir_all(test_dir); // Clean up from previous test
        let persistence = StatePersistence::new(test_dir).unwrap();

        let slots = vec![create_test_slot()];
        let state1 = SimulationState::new(slots.clone());
        let state2 = SimulationState::new(slots);

        persistence.save(&state1).unwrap();
        persistence.save(&state2).unwrap();

        let simulations = persistence.list_simulations().unwrap();
        assert_eq!(simulations.len(), 2);

        // Cleanup
        let _ = std::fs::remove_dir_all(test_dir);
    }
}
