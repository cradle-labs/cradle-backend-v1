# Orderbook Simulator

A comprehensive testing system for simulating realistic trading behavior across multiple accounts, markets, and assets with built-in budget constraints and trade matching.

## Overview

The orderbook simulator allows you to:

- **Schedule trades in advance** - Define trading sequences that will be executed sequentially
- **Enforce budget constraints** - Each account has a budget per asset that depletes as trades execute
- **Ensure trade matching** - Orders are pre-scheduled to cross with specific counterparties
- **Handle failures gracefully** - Exponential backoff retry with interactive recovery options
- **Persist progress** - Save and resume simulations from checkpoints
- **Respect market discipline** - Enforce price constraints for regulated vs unregulated markets

## Architecture

The simulator is built around 4 core modules:

### 1. Budget Management (`src/simulator/budget/`)

Tracks and enforces trading budgets for each account/asset pair.

```
BudgetStore
├── AccountBudget (per account/asset)
│   ├── initial_budget: Total allocated
│   ├── available: Not yet committed
│   ├── locked: Committed in active orders
│   └── spent: Completed transactions
└── BudgetSnapshot (historical)
    └── Point-in-time record
```

**Operations:**
- `lock()` - Reserve budget when placing order
- `unlock()` - Release budget on cancellation
- `spend()` - Finalize budget on settlement
- `has_available()` - Check if budget exists
- `is_depleted()` - Check if budget exhausted

### 2. Action Slots (`src/simulator/action_slot/`)

Represents individual scheduled trading actions executed sequentially.

```
ActionSlot
├── sequence: Order in execution
├── account_id: Account executing
├── action: OrderAction
│   ├── market_id
│   ├── bid/ask_asset
│   ├── bid/ask_amount
│   ├── price
│   ├── side: Bid or Ask
│   └── matching_strategy: Which counterparty
├── state: Pending → InProgress → Completed/Failed
└── retry: attempt_count, max_retries, last_error
```

**State Machine:**
```
Pending
  ↓
InProgress
  ├→ Completed (success)
  ├→ Failed (max retries exceeded)
  └→ Skipped (user choice)
```

**Scheduler (`SlotScheduler`):**
- Generates trading sequences for all accounts
- Ensures orders will cross (matching strategy)
- Randomizes amounts within configured range
- Supports different market distribution modes

**Processor (`SlotProcessor`):**
- Executes slots sequentially
- Retries failed slots with exponential backoff
- Prompts user interactively on max retry failure
- Options: Retry, Skip, Continue, Quit

### 3. Market Discipline (`src/simulator/market_discipline/`)

Enforces price constraints based on market regulation.

```
MarketDiscipline
├── Regulated: Price bands enforced (e.g., ±10%)
└── Unregulated: No price constraints
```

### 4. State Persistence (`src/simulator/state/`)

Saves and restores simulation progress.

```
SimulationState
├── slots: All action slots
├── current_slot_index: Resume point
├── stats: Completed/failed/skipped counts
└── timestamps: Created, saved, duration
```

## Usage Flow

### 1. Initialize Accounts and Markets

```rust
// Load test accounts from database
let accounts = vec![
    account_id_1,
    account_id_2,
    account_id_3,
];

// Load trading markets
let markets = vec![
    market_btc_usd,
    market_eth_usd,
];
```

### 2. Set Up Budgets

```rust
let mut budget_store = BudgetStore::new();

// Set initial budget for each account/asset
for account in &accounts {
    for asset in &assets {
        budget_store.set_budget(
            account,
            asset,
            BigDecimal::from(10000), // $10k per asset
        )?;
    }
}
```

### 3. Generate Trading Schedule

```rust
let scheduler_config = SchedulerConfig {
    min_amount: BigDecimal::from(100),
    max_amount: BigDecimal::from(5000),
    trades_per_account: 10,
    alternate_sides: true,
    market_distribution: MarketDistribution::RoundRobin,
    ..Default::default()
};

let scheduler = SlotScheduler::new(scheduler_config);
let slots = scheduler.generate_schedule(&accounts, &markets)?;
```

### 4. Create Simulation State

```rust
let mut state = SimulationState::new(slots);
let persistence = StatePersistence::new("./simulator_state")?;
```

### 5. Execute Slots

```rust
let processor = SlotProcessor::new(500); // 500ms base retry delay

let results = processor.process_slots(&mut state.slots, |slot| {
    Box::pin(async {
        // Your order placement logic here
        let order_id = place_order(slot).await?;

        // Update budget
        budget_store.lock(
            slot.account_id,
            slot.action.bid_asset,
            slot.action.bid_amount.clone(),
        )?;

        Ok(order_id)
    })
}).await?;

// Persist progress
state.update_stats();
persistence.save(&state)?;
```

### 6. Resume Interrupted Simulation

```rust
// List saved simulations
let simulations = persistence.list_simulations()?;

// Load specific simulation
let mut state = persistence.load(simulation_id)?;

// Resume from current position
let remaining_slots = &mut state.slots[state.current_slot_index..];
processor.process_slots(remaining_slots, executor).await?;
```

## Configuration

### SchedulerConfig

```rust
pub struct SchedulerConfig {
    pub min_amount: BigDecimal,           // Minimum trade size
    pub max_amount: BigDecimal,           // Maximum trade size
    pub trades_per_account: u32,          // How many trades per account
    pub bid_price_offset: f64,            // Price multiplier for bids
    pub ask_price_offset: f64,            // Price multiplier for asks
    pub alternate_sides: bool,            // Alternate buy/sell per account
}
```

### ProcessorConfig

```rust
pub struct ProcessorConfig {
    pub retry_base_delay_ms: u64,         // Initial retry delay
    pub max_retries: u32,                 // Retries before user prompt
    pub save_after_each_slot: bool,       // Auto-save state
}
```

### BudgetConfig

```rust
pub struct BudgetSpec {
    pub account_id: Option<Uuid>,         // Specific account or all
    pub asset_id: Option<Uuid>,           // Specific asset or all
    pub amount: BigDecimal,               // Budget amount
}
```

## Trade Matching Strategy

Orders are scheduled to cross using the `OrderMatchingStrategy`:

### MatchWith(Uuid)
Specific counterparty account:
```rust
// Account A (Bid) matches with Account B (Ask)
OrderMatchingStrategy::MatchWith(account_b_id)
```

### SequentialNext
Next account in sequence:
```rust
// Scheduler determines counterparty automatically
OrderMatchingStrategy::SequentialNext
```

### Any
Any matching account:
```rust
// Order can match with any other order
OrderMatchingStrategy::Any
```

## Execution Flow with Retries

When executing a slot:

1. **Start Execution** - Mark slot as InProgress
2. **Execute Order** - Call user's executor function
3. **Success** - Mark as Completed, move to next slot
4. **Failure** - Check retry count:
   - **Can Retry** (attempt < max_retries)
     - Exponential backoff delay
     - Return to step 2
   - **Max Retries Exceeded**
     - Prompt user interactively
     - Options:
       - **Retry** - Repeat from step 2 (ignoring max_retries)
       - **Skip** - Mark as Skipped, move to next
       - **Continue** - Move to next without marking state
       - **Quit** - Stop simulation

## Budget Flow

### Placing Order (Bid)
```
Account Budget for Asset A:
Before: available=1000, locked=0, spent=0
After:  available=1000, locked=500, spent=0  (lock for bid amount)
```

### Canceling Order
```
Account Budget for Asset A:
Before: available=1000, locked=500, spent=0
After:  available=1500, locked=0, spent=0    (unlock the amount)
```

### Settling Order
```
Account Budget for Asset A:
Before: available=1000, locked=500, spent=0
After:  available=500, locked=0, spent=500   (spend the locked amount)
```

### Depleted Budget
```
Account Budget for Asset A:
available=0, locked=0, spent=1000
is_depleted() = true

// Cannot place new orders
budget_store.has_available(account, asset, &amount) = false
```

## Integration with Orderbook

### Before Placing Order

```rust
// Check budget
if !budget_store.has_available(&slot.account_id, &slot.action.bid_asset, &slot.action.bid_amount) {
    return Err(anyhow!("Insufficient budget"));
}

// Validate price discipline
let market = fetch_market(slot.action.market_id).await?;
let discipline = MarketDiscipline::new(market);
discipline.validate_price(&slot.action.price)?;

// Lock budget
budget_store.lock(
    slot.account_id,
    slot.action.bid_asset,
    slot.action.bid_amount.clone(),
)?;
```

### After Order Placement

```rust
// Update slot with order ID
slot.complete_execution(order_id);
```

### On Order Settlement

```rust
// Spend budget when order settles
budget_store.spend(
    maker_account_id,
    maker_asset_id,
    settlement_amount,
)?;
```

### On Order Cancellation

```rust
// Unlock budget when cancelled
budget_store.unlock(
    account_id,
    bid_asset_id,
    remaining_bid_amount,
)?;
```

## Statistics and Monitoring

After execution:

```rust
state.update_stats();

println!("Total Slots: {}", state.stats.total_slots);
println!("Completed: {}", state.stats.completed_slots);
println!("Failed: {}", state.stats.failed_slots);
println!("Skipped: {}", state.stats.skipped_slots);
println!("Orders Created: {}", state.stats.total_orders_created);
println!("Matches: {}", state.stats.total_matches);

// Budget summary
let summary = budget_store.get_summary();
println!("Total Budget: {}", summary.total_initial);
println!("Total Spent: {}", summary.total_spent);
println!("Utilization: {:.2}%", summary.total_utilization_percent());
```

## Testing

Run all simulator tests:

```bash
cargo test simulator::
```

Run specific module tests:

```bash
cargo test simulator::budget
cargo test simulator::action_slot
cargo test simulator::state
cargo test simulator::market_discipline
```

## Example Configuration File

Create `simulator.json`:

```json
{
  "scheduler": {
    "min_amount": 100,
    "max_amount": 5000,
    "trades_per_account": 10,
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

## Advanced Usage

### Custom Amount Distribution

Instead of random, use deterministic amounts:

```rust
let slots = scheduler.generate_schedule(&accounts, &markets)?;

// Override amounts
for slot in &mut slots {
    slot.action.bid_amount = BigDecimal::from(1000);
    slot.action.ask_amount = BigDecimal::from(950);
}
```

### Filter by Market

Execute only slots for specific market:

```rust
let market_slots: Vec<_> = state.slots
    .iter_mut()
    .filter(|s| s.action.market_id == target_market)
    .collect();

processor.process_slots(&market_slots, executor).await?;
```

### Export Trading History

```rust
let history = budget_store.get_history();

for snapshot in history {
    println!("{:?}: Account {:?}, Spent: {}",
        snapshot.timestamp,
        snapshot.account_id,
        snapshot.spent
    );
}
```

## File Structure

```
src/simulator/
├── mod.rs                  # Module exports
├── config.rs              # Configuration structures
├── SIMULATOR_GUIDE.md     # Detailed guide
│
├── budget/                # Budget tracking
│   ├── mod.rs
│   ├── models.rs          # AccountBudget, BudgetSnapshot
│   └── storage.rs         # BudgetStore
│
├── action_slot/           # Slot scheduling & execution
│   ├── mod.rs
│   ├── models.rs          # ActionSlot, OrderAction
│   ├── scheduler.rs       # SlotScheduler
│   └── processor.rs       # SlotProcessor
│
├── market_discipline/     # Price constraints
│   └── mod.rs             # MarketDiscipline
│
├── state/                 # Persistence
│   └── mod.rs             # SimulationState, StatePersistence
│
├── accounts/              # Account generation (existing)
│   ├── models.rs
│   ├── generator.rs
│   └── storage.rs
│
└── shared/                # Common utilities (existing)
    └── retry.rs           # ExponentialBackoffRetry
```

## Troubleshooting

### Orders Not Crossing

**Check:**
1. Matching strategy correctly set
2. Accounts have opposite sides (Bid/Ask)
3. Same market in both orders
4. Prices allow crossing (bid >= ask)

### Budget Not Updating

**Check:**
1. Lock is called before order placement
2. Spend is called on settlement
3. Unlock is called on cancellation
4. Amount conversions handle precision

### State Not Persisting

**Check:**
1. State directory exists and is writable
2. `state.update_stats()` called before save
3. `save()` error is checked

### Slot Processor Hangs

**Check:**
1. Executor function completes
2. No infinite loops in retry logic
3. User input prompt isn't waiting for input

## Performance Considerations

- Budget store is in-memory (no database hits)
- Scheduler generates slots upfront
- Each slot execution is independent
- State persistence is sequential (can be parallelized)
- Large number of slots may need pagination

## Future Enhancements

- [ ] Parallel slot execution with locking
- [ ] Real-time price feed integration
- [ ] More granular budget tracking (per-order)
- [ ] Portfolio margin calculations
- [ ] Risk limit enforcement
- [ ] WebSocket progress updates
- [ ] Batch operations optimization

## See Also

- `src/order_book/` - Orderbook implementation
- `src/market/` - Market management
- `src/accounts/` - Account management
- `src/simulator/accounts/` - Account generation

---

For detailed integration examples, see `SIMULATOR_GUIDE.md`.
