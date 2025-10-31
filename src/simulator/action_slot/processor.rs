use anyhow::Result;
use uuid::Uuid;
use std::io::{self, Write};

use super::models::{ActionSlot, RecoveryAction, SlotExecutionResult, ActionSlotState};
use crate::simulator::shared::retry::ExponentialBackoffRetry;

/// Processes action slots with retry and recovery logic
pub struct SlotProcessor {
    base_delay_ms: u64,
}

impl SlotProcessor {
    pub fn new(base_delay_ms: u64) -> Self {
        Self { base_delay_ms }
    }

    /// Process a single slot with retry logic
    pub async fn process_slot<F>(&self, slot: &mut ActionSlot, executor: F) -> Result<SlotExecutionResult>
    where
        F: Fn(&ActionSlot) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Uuid>>>>,
    {
        loop {
            slot.start_execution();

            // Create retry handler
            let mut retry = ExponentialBackoffRetry::new(
                self.base_delay_ms,
                slot.max_retries - 1, // max_retries - 1 because we've already started
            );

            match retry.execute(|| executor(slot)).await {
                Ok(order_id) => {
                    slot.complete_execution(order_id);
                    return Ok(SlotExecutionResult {
                        slot_id: slot.id,
                        state: slot.state,
                        order_id: Some(order_id),
                        duration_ms: slot.execution_duration_ms(),
                        error: None,
                    });
                }
                Err(e) => {
                    slot.fail_execution(e.to_string());

                    if slot.should_retry() {
                        // Ask user what to do
                        match self.ask_recovery_action(slot)? {
                            RecoveryAction::Retry => {
                                // Loop continues, will retry
                                continue;
                            }
                            RecoveryAction::Skip => {
                                slot.skip();
                                return Ok(SlotExecutionResult {
                                    slot_id: slot.id,
                                    state: slot.state,
                                    order_id: None,
                                    duration_ms: slot.execution_duration_ms(),
                                    error: Some(e.to_string()),
                                });
                            }
                            RecoveryAction::Continue => {
                                return Ok(SlotExecutionResult {
                                    slot_id: slot.id,
                                    state: slot.state,
                                    order_id: None,
                                    duration_ms: slot.execution_duration_ms(),
                                    error: Some(e.to_string()),
                                });
                            }
                            RecoveryAction::Quit => {
                                return Err(anyhow::anyhow!("User quit simulation"));
                            }
                        }
                    } else {
                        // No more retries, return error
                        return Ok(SlotExecutionResult {
                            slot_id: slot.id,
                            state: slot.state,
                            order_id: None,
                            duration_ms: slot.execution_duration_ms(),
                            error: Some(e.to_string()),
                        });
                    }
                }
            }
        }
    }

    /// Interactive prompt for recovery action
    fn ask_recovery_action(&self, slot: &ActionSlot) -> Result<RecoveryAction> {
        println!("\n{}", "─".repeat(60));
        println!("Slot #{} (Account: {}) encountered an error:", slot.sequence, slot.account_id);
        println!("Error: {}", slot.last_error.as_ref().unwrap_or(&"Unknown".to_string()));
        println!("Attempts: {}/{}", slot.attempt_count, slot.max_retries);
        println!("{}", "─".repeat(60));

        loop {
            println!("\nWhat would you like to do?");
            println!("  [1] Retry      - Attempt the slot again");
            println!("  [2] Skip       - Skip this slot and mark as completed");
            println!("  [3] Continue   - Continue without marking as completed");
            println!("  [4] Quit       - Quit the entire simulation");
            print!("\nEnter choice (1-4): ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            match input.trim() {
                "1" => return Ok(RecoveryAction::Retry),
                "2" => return Ok(RecoveryAction::Skip),
                "3" => return Ok(RecoveryAction::Continue),
                "4" => return Ok(RecoveryAction::Quit),
                _ => println!("Invalid choice. Please enter 1, 2, 3, or 4."),
            }
        }
    }

    /// Process multiple slots sequentially with recovery
    pub async fn process_slots<F>(
        &self,
        slots: &mut [ActionSlot],
        executor: F,
    ) -> Result<Vec<SlotExecutionResult>>
    where
        F: Fn(&ActionSlot) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Uuid>>>>,
    {
        let mut results = Vec::new();
        let slots_len = slots.len();

        for (idx, slot) in slots.iter_mut().enumerate() {
            println!("\n[{}/{}] Processing slot #{} (Account: {})",
                idx + 1, slots_len, slot.sequence, slot.account_id);

            match self.process_slot(slot, &executor).await {
                Ok(result) => {
                    match result.state {
                        ActionSlotState::Completed => {
                            println!("✓ Slot #{} completed successfully", slot.sequence);
                        }
                        ActionSlotState::Skipped => {
                            println!("⊘ Slot #{} skipped", slot.sequence);
                        }
                        ActionSlotState::Failed => {
                            println!("✗ Slot #{} failed: {}", slot.sequence,
                                result.error.as_ref().unwrap_or(&"Unknown error".to_string()));
                        }
                        _ => {}
                    }
                    results.push(result);
                }
                Err(e) if e.to_string().contains("User quit") => {
                    println!("\nSimulation quit by user at slot #{}", slot.sequence);
                    return Err(e);
                }
                Err(e) => {
                    println!("✗ Unexpected error processing slot #{}: {}", slot.sequence, e);
                    results.push(SlotExecutionResult {
                        slot_id: slot.id,
                        state: ActionSlotState::Failed,
                        order_id: None,
                        duration_ms: None,
                        error: Some(e.to_string()),
                    });
                }
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use bigdecimal::BigDecimal;
    use crate::simulator::action_slot::models::{OrderAction, OrderActionSide, OrderMatchingStrategy};

    #[tokio::test]
    async fn test_slot_processor_success() {
        let processor = SlotProcessor::new(10);
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

        let mut slot = ActionSlot::new(1, account_id, action, 3);
        let order_id = Uuid::new_v4();

        let result = processor
            .process_slot(&mut slot, |_| {
                Box::pin(async move { Ok(order_id) })
            })
            .await
            .unwrap();

        assert_eq!(result.state, ActionSlotState::Completed);
        assert_eq!(result.order_id, Some(order_id));
        assert_eq!(slot.state, ActionSlotState::Completed);
    }
}
