use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use anyhow::Error;

/// Execution state of an action slot
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActionSlotState {
    /// Waiting to be executed
    Pending,
    /// Currently executing
    InProgress,
    /// Successfully completed
    Completed,
    /// Failed and waiting for recovery decision
    Failed,
    /// Skipped by user
    Skipped,
}

/// Side of the order (buy or sell)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderActionSide {
    /// Buying (bidding)
    Bid,
    /// Selling (asking)
    Ask,
}

/// Strategy for matching this order with another
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderMatchingStrategy {
    /// Match with a specific account ID
    MatchWith(Uuid),
    /// Match with next account in sequence
    SequentialNext,
    /// Match with any account trading same market
    Any,
}

/// A trading action to place an order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderAction {
    /// Market being traded
    pub market_id: Uuid,

    /// Asset being bid
    pub bid_asset: Uuid,

    /// Asset being asked
    pub ask_asset: Uuid,

    /// Amount of bid asset
    pub bid_amount: BigDecimal,

    /// Amount of ask asset
    pub ask_amount: BigDecimal,

    /// Which side this account is on
    pub side: OrderActionSide,

    /// Price (ask_amount / bid_amount)
    pub price: BigDecimal,

    /// How to match this order
    pub matching_strategy: OrderMatchingStrategy,
}

/// An action slot represents a single scheduled trading action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionSlot {
    /// Unique ID for this slot
    pub id: Uuid,

    /// Sequence number (determines execution order)
    pub sequence: u32,

    /// Account executing this action
    pub account_id: Uuid,

    /// The action to perform
    pub action: OrderAction,

    /// Current state of the slot
    pub state: ActionSlotState,

    /// Number of execution attempts
    pub attempt_count: u32,

    /// Maximum retry attempts before asking user
    pub max_retries: u32,

    /// Last error if any
    pub last_error: Option<String>,

    /// The order ID if successfully created
    pub created_order_id: Option<Uuid>,

    /// Matched order ID(s) if any
    pub matched_order_ids: Vec<Uuid>,

    /// When the slot was created
    pub created_at: DateTime<Utc>,

    /// When execution started
    pub execution_started_at: Option<DateTime<Utc>>,

    /// When execution completed
    pub execution_completed_at: Option<DateTime<Utc>>,
}

impl ActionSlot {
    /// Create a new action slot
    pub fn new(
        sequence: u32,
        account_id: Uuid,
        action: OrderAction,
        max_retries: u32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            sequence,
            account_id,
            action,
            state: ActionSlotState::Pending,
            attempt_count: 0,
            max_retries,
            last_error: None,
            created_order_id: None,
            matched_order_ids: Vec::new(),
            created_at: Utc::now(),
            execution_started_at: None,
            execution_completed_at: None,
        }
    }

    /// Mark as in-progress
    pub fn start_execution(&mut self) {
        self.state = ActionSlotState::InProgress;
        self.execution_started_at = Some(Utc::now());
        self.attempt_count += 1;
    }

    /// Mark as completed
    pub fn complete_execution(&mut self, order_id: Uuid) {
        self.state = ActionSlotState::Completed;
        self.execution_completed_at = Some(Utc::now());
        self.created_order_id = Some(order_id);
    }

    /// Mark as failed with error
    pub fn fail_execution(&mut self, error: String) {
        self.state = ActionSlotState::Failed;
        self.last_error = Some(error);
    }

    /// Mark as skipped
    pub fn skip(&mut self) {
        self.state = ActionSlotState::Skipped;
        self.execution_completed_at = Some(Utc::now());
    }

    /// Check if we should retry
    pub fn should_retry(&self) -> bool {
        self.state == ActionSlotState::Failed && self.attempt_count <= self.max_retries
    }

    /// Get execution duration in milliseconds if completed
    pub fn execution_duration_ms(&self) -> Option<u128> {
        match (self.execution_started_at, self.execution_completed_at) {
            (Some(start), Some(end)) => Some((end - start).num_milliseconds() as u128),
            _ => None,
        }
    }

    /// Record a matched order
    pub fn add_matched_order(&mut self, order_id: Uuid) {
        self.matched_order_ids.push(order_id);
    }
}

/// Result of slot execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotExecutionResult {
    pub slot_id: Uuid,
    pub state: ActionSlotState,
    pub order_id: Option<Uuid>,
    pub duration_ms: Option<u128>,
    pub error: Option<String>,
}

/// Types of errors during execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SlotExecutionError {
    /// Budget insufficient for the trade
    InsufficientBudget {
        account_id: Uuid,
        asset_id: Uuid,
        required: BigDecimal,
        available: BigDecimal,
    },
    /// Market constraint violation
    PriceOutOfRange {
        market_id: Uuid,
        price: BigDecimal,
        min_price: BigDecimal,
        max_price: BigDecimal,
    },
    /// Database error
    DatabaseError(String),
    /// Contract execution error
    ContractError(String),
    /// Other error
    Other(String),
}

impl std::fmt::Display for SlotExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InsufficientBudget { required, available, .. } => {
                write!(f, "Insufficient budget: need {}, have {}", required, available)
            }
            Self::PriceOutOfRange { market_id: _, price, min_price, max_price } => {
                write!(f, "Price {} outside range [{}, {}]", price, min_price, max_price)
            }
            Self::DatabaseError(e) => write!(f, "Database error: {}", e),
            Self::ContractError(e) => write!(f, "Contract error: {}", e),
            Self::Other(e) => write!(f, "Error: {}", e),
        }
    }
}

/// Actions user can take on a failed slot
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RecoveryAction {
    /// Retry the same slot
    Retry,
    /// Skip this slot and continue
    Skip,
    /// Quit the entire simulation
    Quit,
    /// Continue to next slot without marking this one
    Continue,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_slot_creation() {
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

        let slot = ActionSlot::new(1, account_id, action, 3);

        assert_eq!(slot.sequence, 1);
        assert_eq!(slot.account_id, account_id);
        assert_eq!(slot.state, ActionSlotState::Pending);
        assert_eq!(slot.attempt_count, 0);
        assert_eq!(slot.max_retries, 3);
    }

    #[test]
    fn test_slot_state_transitions() {
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

        assert_eq!(slot.state, ActionSlotState::Pending);

        slot.start_execution();
        assert_eq!(slot.state, ActionSlotState::InProgress);
        assert_eq!(slot.attempt_count, 1);

        let order_id = Uuid::new_v4();
        slot.complete_execution(order_id);
        assert_eq!(slot.state, ActionSlotState::Completed);
        assert_eq!(slot.created_order_id, Some(order_id));
    }

    #[test]
    fn test_retry_logic() {
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

        slot.start_execution();
        assert!(!slot.should_retry());

        slot.fail_execution("Test error".to_string());
        assert!(slot.should_retry());
        assert_eq!(slot.attempt_count, 1);

        for _ in 1..3 {
            slot.attempt_count += 1;
            assert!(slot.should_retry());
        }

        slot.attempt_count = 4;
        assert!(!slot.should_retry());
    }
}
