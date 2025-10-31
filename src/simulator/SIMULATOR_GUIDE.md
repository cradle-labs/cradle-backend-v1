# Orderbook Simulator Guide

This simulator enables testing of the orderbook system with multiple test accounts performing realistic trading operations across markets with budget constraints.

## Architecture Overview

The simulator consists of several key components:

### 1. Budget Module (`budget/`)
Tracks and enforces trading budgets per account/asset pair.

**Features:**
- Lock/unlock amounts during order placement/cancellation
- Spend tracking when orders settle
- Budget depletion detection
- History and snapshots

**Key Types:**
- `AccountBudget`: Tracks available, locked, and spent amounts
- `BudgetStore`: In-memory storage for all budgets

### 2. Action Slot Module (`action_slot/`)
Represents scheduled trading operations that will be executed sequentially.

**Components:**
- **Models**: `ActionSlot`, `OrderAction`, `OrderActionSide`, `OrderMatchingStrategy`
  - Each slot has a sequence number for ordering
  - Tracks retry attempts and execution state
  - Can specify counterparty for trade matching

- **Scheduler**: Generates trading sequences
  - Creates slots for multiple accounts across markets
  - Ensures orders will cross (odd/even account matching)
  - Random amount generation within constraints
  - Supports different market distribution strategies

- **Processor**: Executes slots with recovery
  - Exponential backoff retry logic
  - Interactive recovery on failures
  - Options: Retry, Skip, Continue, or Quit

### 3. Market Discipline Module (`market_discipline/`)
Enforces price constraints based on market regulation status.

**Features:**
- Regulated markets: Enforce price bands
- Unregulated markets: No price constraints
- Validation before order placement

### 4. State Module (`state/`)
Persists simulation progress to disk.

**Features:**
- Save/load complete simulation state
- Resume from checkpoint
- Track statistics (completed, failed, skipped slots)

## Usage Example

### Basic Setup

```rust
use uuid::Uuid;
use bigdecimal::BigDecimal;
use cradle_back_end::simulator::{
    action_slot::{SlotScheduler, SlotProcessor},
    action_slot::scheduler::SchedulerConfig,
    budget::storage::BudgetStore,
    state::SimulationState,
};

// 1. Load test accounts from database
let accounts = vec![
    account_1_id,
    account_2_id,
    account_3_id,
];

// 2. Load markets
let markets = vec![
    market_1_id,
    market_2_id,
];

// 3. Initialize budgets
let mut budget_store = BudgetStore::new();
for account in &accounts {
    for market in &markets {
        budget_store.set_budget(
            *account,
            market.asset_one,
            BigDecimal::from(10000),
        )?;
    }
}

// 4. Generate schedule
let scheduler_config = SchedulerConfig {
    min_amount: BigDecimal::from(10),
    max_amount: BigDecimal::from(1000),
    trades_per_account: 5,
    ..Default::default()
};
let scheduler = SlotScheduler::new(scheduler_config);
let mut slots = scheduler.generate_schedule(&accounts, &markets)?;

// 5. Create simulation state
let mut state = SimulationState::new(slots);

// 6. Process slots
let processor = SlotProcessor::new(500); // 500ms base delay

let results = processor.process_slots(&mut state.slots, |slot| {
    Box::pin(async {
        // Execute order placement logic here
        // - Check budget
        // - Validate price discipline
        // - Place order on exchange
        // - Return order ID

        todo!("Implement order execution")
    })
}).await?;

// 7. Update statistics and persist
state.update_stats();
state_persistence.save(&state)?;
```

## Configuration

Create a `simulator_config.json`:

```json
{
  "scheduler": {
    "min_amount": 10,
    "max_amount": 1000,
    "trades_per_account": 5,
    "bid_price_offset": 1.0,
    "ask_price_offset": 1.0,
    "alternate_sides": true
  },
  "processor": {
    "retry_base_delay_ms": 500,
    "max_retries": 3,
    "save_after_each_slot": true
  },
  "budget": {
    "budgets": [
      {
        "account_id": null,
        "asset_id": null,
        "amount": 10000
      }
    ]
  },
  "state_dir": "./simulator_state"
}
```

## Trading Strategies

### Matching Strategy
The scheduler ensures orders will cross by:
- Having odd-indexed accounts place BID orders
- Having even-indexed accounts place ASK orders
- Both trading the same market pair
- Prices allowing the orders to cross

### Budget Constraints
- Each account has a budget per asset
- Orders lock budget when placed
- Budget unlocks on cancellation
- Budget spends on settlement

### Recovery on Failure
If an order fails:
1. User is prompted interactively
2. Options:
   - **Retry**: Attempt same order again (up to max_retries)
   - **Skip**: Mark as completed and move to next
   - **Continue**: Don't mark as completed, move to next
   - **Quit**: Stop simulator

## State Persistence

Simulations are automatically persisted:
- After each slot if `save_after_each_slot` is true
- State includes all slots and their execution status
- Can resume from checkpoint by loading the simulation ID

## Market Price Discipline

The system enforces different price rules by market type:
- **Regulated Markets**: Price bands enforced (±X%)
- **Unregulated Markets**: Any price allowed

Price validation happens before order placement.

## Integration Points

To integrate with the actual orderbook:

### 1. OrderAction Execution
In the `SlotProcessor::process_slots` executor closure:

```rust
|slot| {
    Box::pin(async {
        // Validate budget
        if !budget_store.has_available(slot.account_id, slot.action.bid_asset, &slot.action.bid_amount) {
            return Err(anyhow!("Insufficient budget"));
        }

        // Validate price discipline
        let market = /* fetch market */;
        let discipline = MarketDiscipline::new(market);
        discipline.validate_price(&slot.action.price)?;

        // Lock budget
        budget_store.lock(slot.account_id, slot.action.bid_asset, slot.action.bid_amount.clone())?;

        // Place order
        let order_result = place_order(
            slot.account_id,
            &slot.action,
        ).await?;

        Ok(order_result.id)
    })
}
```

### 2. Order Settlement
When orders settle in the orderbook processor:
```rust
// On settlement, spend the budget
budget_store.spend(account_id, asset_id, amount)?;
```

### 3. Order Cancellation
When orders are cancelled:
```rust
// Unlock the budget
budget_store.unlock(account_id, asset_id, amount)?;
```

## Statistics and Reporting

After simulation completes:

```rust
println!("Simulation Results:");
println!("Total Slots: {}", state.stats.total_slots);
println!("Completed: {}", state.stats.completed_slots);
println!("Failed: {}", state.stats.failed_slots);
println!("Skipped: {}", state.stats.skipped_slots);
println!("Orders Created: {}", state.stats.total_orders_created);
println!("Matches: {}", state.stats.total_matches);
```

## Advanced Features

### Custom Market Distribution
Choose how accounts trade markets:
- `RoundRobin`: Each account trades each market in sequence
- `SameMarket`: All accounts trade the same market
- `Sequential`: Accounts staggered across markets

### Custom Price Offsets
Control bid/ask spreads:
```rust
scheduler_config.bid_price_offset = 1.05; // 5% premium on bids
scheduler_config.ask_price_offset = 0.95; // 5% discount on asks
```

### Resume Interrupted Simulations
```rust
// Load previous state
let state = persistence.load(simulation_id)?;

// Continue from current_slot_index
let remaining_slots = &mut state.slots[state.current_slot_index..];
let results = processor.process_slots(remaining_slots, executor).await?;
```

## Testing

The simulator includes comprehensive unit tests:

```bash
cargo test simulator::budget
cargo test simulator::action_slot
cargo test simulator::state
```

Each module is thoroughly tested with:
- Basic operations
- Edge cases
- Error conditions
- State transitions
