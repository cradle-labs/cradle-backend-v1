use anyhow::{Result, anyhow};
use bigdecimal::BigDecimal;
use chrono::Utc;
use std::io::{self, Write};
use uuid::Uuid;

use crate::simulator::{
    action_slot::{SlotScheduler, SlotProcessor, scheduler::SchedulerConfig},
    budget::storage::BudgetStore,
    state::{SimulationState, StatePersistence},
};

/// Statistics accumulated across multiple schedules
#[derive(Debug, Clone)]
pub struct CumulativeStats {
    pub total_schedules: u32,
    pub total_slots_processed: usize,
    pub total_slots_completed: usize,
    pub total_slots_failed: usize,
    pub total_slots_skipped: usize,
    pub total_orders_created: usize,
    pub total_matches: usize,
    pub started_at: chrono::DateTime<Utc>,
}

impl CumulativeStats {
    pub fn new() -> Self {
        Self {
            total_schedules: 0,
            total_slots_processed: 0,
            total_slots_completed: 0,
            total_slots_failed: 0,
            total_slots_skipped: 0,
            total_orders_created: 0,
            total_matches: 0,
            started_at: Utc::now(),
        }
    }

    pub fn update_from_state(&mut self, state: &SimulationState) {
        self.total_schedules += 1;
        self.total_slots_processed += state.stats.total_slots;
        self.total_slots_completed += state.stats.completed_slots;
        self.total_slots_failed += state.stats.failed_slots;
        self.total_slots_skipped += state.stats.skipped_slots;
        self.total_orders_created += state.stats.total_orders_created;
        self.total_matches += state.stats.total_matches;
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_slots_processed == 0 {
            return 0.0;
        }
        (self.total_slots_completed as f64 / self.total_slots_processed as f64) * 100.0
    }

    pub fn print_summary(&self) {
        let elapsed = Utc::now() - self.started_at;
        let elapsed_secs = elapsed.num_seconds() as f64;

        println!("\n{}", "=".repeat(70));
        println!("CUMULATIVE SIMULATION SUMMARY");
        println!("{}", "=".repeat(70));
        println!("Schedules run: {}", self.total_schedules);
        println!("Total slots processed: {}", self.total_slots_processed);
        println!("  Completed: {} ({:.1}%)",
            self.total_slots_completed,
            self.success_rate()
        );
        println!("  Failed: {}", self.total_slots_failed);
        println!("  Skipped: {}", self.total_slots_skipped);
        println!("Orders created: {}", self.total_orders_created);
        println!("Total matches: {}", self.total_matches);
        println!("Elapsed time: {:.2}s", elapsed_secs);
        println!("{}", "=".repeat(70));
    }
}

/// Manages continuous simulation with budget tracking
pub struct SimulatorRunner {
    scheduler_config: SchedulerConfig,
    processor: SlotProcessor,
    budget_store: BudgetStore,
    persistence: StatePersistence,
    stats: CumulativeStats,
    accounts: Vec<Uuid>,
    markets: Vec<Uuid>,
    markets_info: Vec<(Uuid, Uuid, Uuid)>, // (market_id, asset_one, asset_two)
    auto_continue: bool,
    max_iterations: Option<u32>,
    trades_per_second: f64,
    interactive: bool,
}

impl SimulatorRunner {
    pub fn new(
        accounts: Vec<Uuid>,
        markets: Vec<Uuid>,
        markets_info: Vec<(Uuid, Uuid, Uuid)>,
        scheduler_config: SchedulerConfig,
        budget_store: BudgetStore,
        persistence: StatePersistence,
        auto_continue: bool,
        max_iterations: Option<u32>,
        trades_per_second: f64,
        interactive: bool,
    ) -> Self {
        Self {
            scheduler_config,
            processor: SlotProcessor::new(500), // 500ms base delay
            budget_store,
            persistence,
            stats: CumulativeStats::new(),
            accounts,
            markets,
            markets_info,
            auto_continue,
            max_iterations,
            trades_per_second,
            interactive,
        }
    }

    /// Execute slots with trade speed control and optional interactive prompts
    async fn execute_slots_with_speed<F>(
        &self,
        state: &mut SimulationState,
        executor: &F,
    ) -> Result<Vec<crate::simulator::action_slot::models::SlotExecutionResult>>
    where
        F: Fn(&crate::simulator::action_slot::models::ActionSlot)
            -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Uuid>>>>,
    {
        let delay_ms = if self.trades_per_second > 0.0 {
            (1000.0 / self.trades_per_second) as u64
        } else {
            100 // default 100ms if invalid
        };

        let mut results = Vec::new();
        let slots_len = state.slots.len();

        for (idx, slot) in state.slots.iter_mut().enumerate() {
            println!("\n[{}/{}] Processing slot #{} (Account: {})",
                idx + 1, slots_len, slot.sequence, slot.account_id);

            match self.processor.process_slot(slot, executor).await {
                Ok(result) => {
                    match result.state {
                        crate::simulator::action_slot::models::ActionSlotState::Completed => {
                            println!("✓ Slot #{} completed successfully", slot.sequence);
                        }
                        crate::simulator::action_slot::models::ActionSlotState::Skipped => {
                            println!("⊘ Slot #{} skipped", slot.sequence);
                        }
                        crate::simulator::action_slot::models::ActionSlotState::Failed => {
                            println!("✗ Slot #{} failed: {}", slot.sequence,
                                result.error.as_ref().unwrap_or(&"Unknown error".to_string()));
                        }
                        _ => {}
                    }
                    results.push(result);
                }
                Err(e) if e.to_string().contains("User quit") => {
                    return Err(e);
                }
                Err(e) => {
                    println!("✗ Unexpected error processing slot #{}: {}", slot.sequence, e);
                    results.push(crate::simulator::action_slot::models::SlotExecutionResult {
                        slot_id: slot.id,
                        state: slot.state,
                        order_id: None,
                        duration_ms: slot.execution_duration_ms(),
                        error: Some(e.to_string()),
                    });
                }
            }

            // Add delay between trades for speed control
            if idx < slots_len - 1 {
                tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
            }

            // Show interactive menu if enabled and every 10 trades
            if self.interactive && (idx + 1) % 10 == 0 {
                if !self.show_interactive_menu(&results)? {
                    break; // User chose to skip rest of schedule
                }
            }
        }

        Ok(results)
    }

    /// Show interactive menu during execution
    fn show_interactive_menu(&self, results: &[crate::simulator::action_slot::models::SlotExecutionResult]) -> Result<bool> {
        use std::io::Write;
        loop {
            println!("\n{}", "─".repeat(60));
            println!("Interactive Menu (after {} trades)", results.len());
            println!("  [V] View current stats");
            println!("  [C] Continue execution");
            println!("  [S] Skip rest of schedule");
            println!("  [P] Pause execution (coming soon)");
            print!("\nEnter choice (V/C/S/P): ");
            std::io::stdout().flush()?;

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;

            match input.trim().to_uppercase().as_str() {
                "V" => {
                    let completed = results.iter().filter(|r| r.state == crate::simulator::action_slot::models::ActionSlotState::Completed).count();
                    let failed = results.iter().filter(|r| r.state == crate::simulator::action_slot::models::ActionSlotState::Failed).count();
                    let skipped = results.iter().filter(|r| r.state == crate::simulator::action_slot::models::ActionSlotState::Skipped).count();
                    println!("Trades processed: {} (Completed: {}, Failed: {}, Skipped: {})",
                        results.len(), completed, failed, skipped);
                }
                "C" => return Ok(true),
                "S" => return Ok(false),
                "P" => println!("Pause feature coming soon!"),
                _ => println!("Invalid choice. Please enter V, C, S, or P."),
            }
        }
    }

    /// Run the continuous simulation loop
    pub async fn run<F>(&mut self, executor: F) -> Result<()>
    where
        F: Fn(&crate::simulator::action_slot::models::ActionSlot)
            -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Uuid>>>>,
    {
        let mut iteration = 0u32;

        loop {
            // Check iteration limit
            if let Some(max) = self.max_iterations {
                if iteration >= max {
                    println!("\nReached maximum iterations ({})", max);
                    break;
                }
            }

            iteration += 1;

            println!("\n{}", "─".repeat(70));
            println!("SCHEDULE {}", iteration);
            println!("{}", "─".repeat(70));

            // Generate schedule
            let scheduler = SlotScheduler::new(self.scheduler_config.clone());
            let slots = scheduler.generate_schedule(&self.accounts, &self.markets, &self.markets_info)?;

            println!("Generated {} slots for {} accounts across {} markets",
                slots.len(), self.accounts.len(), self.markets.len());

            // Create simulation state
            let mut state = SimulationState::new(slots);

            // Execute slots with trade speed control and interactive prompts
            match self.execute_slots_with_speed(&mut state, &executor).await {
                Ok(_results) => {
                    // Update statistics
                    state.update_stats();
                    self.stats.update_from_state(&state);

                    // Print schedule summary
                    self.print_schedule_summary(&state);

                    // Save checkpoint
                    self.persistence.save(&state)?;
                    println!("✓ State saved to checkpoint");
                }
                Err(e) if e.to_string().contains("User quit") => {
                    println!("\nSimulation quit by user at schedule {}", iteration);
                    self.stats.print_summary();
                    return Ok(());
                }
                Err(e) => {
                    println!("\n✗ Error during schedule execution: {}", e);
                    self.stats.print_summary();
                    return Err(e);
                }
            }

            // Ask to continue
            if !self.auto_continue {
                if !self.ask_continue()? {
                    break;
                }
            }
        }

        self.stats.print_summary();
        Ok(())
    }

    /// Print summary for current schedule
    fn print_schedule_summary(&self, state: &SimulationState) {
        println!("\nSchedule Summary:");
        println!("  Slots: {} completed, {} failed, {} skipped",
            state.stats.completed_slots,
            state.stats.failed_slots,
            state.stats.skipped_slots
        );
        println!("  Orders: {} created", state.stats.total_orders_created);
        println!("  Matches: {}", state.stats.total_matches);

        let budget_summary = self.budget_store.get_summary();
        let utilization = budget_summary.total_utilization_percent();
        println!("  Budget utilization: {:.2}%", utilization);
        println!("    Spent: {} / {}",
            budget_summary.total_spent,
            budget_summary.total_initial
        );
        println!("    Remaining: {} (available) + {} (locked) = {}",
            budget_summary.total_available,
            budget_summary.total_locked,
            budget_summary.total_available.clone() + budget_summary.total_locked.clone()
        );

        if budget_summary.depleted_count > 0 {
            println!("  ⚠ {} accounts/assets depleted", budget_summary.depleted_count);
        }
    }

    /// Ask user if they want to continue
    fn ask_continue(&self) -> Result<bool> {
        loop {
            print!("\nContinue to next schedule? (y/n): ");
            io::stdout().flush().map_err(|e| anyhow!("IO error: {}", e))?;

            let mut input = String::new();
            io::stdin().read_line(&mut input).map_err(|e| anyhow!("IO error: {}", e))?;

            match input.trim().to_lowercase().as_str() {
                "y" | "yes" => return Ok(true),
                "n" | "no" => return Ok(false),
                _ => println!("Please enter 'y' or 'n'"),
            }
        }
    }

    /// Get current cumulative statistics
    pub fn stats(&self) -> &CumulativeStats {
        &self.stats
    }

    /// Get mutable reference to budget store
    pub fn budget_store_mut(&mut self) -> &mut BudgetStore {
        &mut self.budget_store
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cumulative_stats_creation() {
        let stats = CumulativeStats::new();
        assert_eq!(stats.total_schedules, 0);
        assert_eq!(stats.total_slots_processed, 0);
        assert_eq!(stats.success_rate(), 0.0);
    }

    #[test]
    fn test_success_rate_calculation() {
        let mut stats = CumulativeStats::new();
        stats.total_slots_processed = 100;
        stats.total_slots_completed = 80;
        assert_eq!(stats.success_rate(), 80.0);
    }
}
