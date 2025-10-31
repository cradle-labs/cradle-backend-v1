# Simulator CLI - User Guide

The Simulator CLI is a standalone command-line tool for running continuous, long-running trading simulations with persistent budget tracking across multiple schedules.

## Overview

The CLI automatically:
1. **Discovers test accounts** - All accounts with `linked_account_id` starting with "test-account"
2. **Discovers markets** - All available markets from the database
3. **Initializes budgets** - 1M tokens per account per market asset
4. **Generates trading schedules** - 10 trades per account per schedule (configurable)
5. **Tracks budgets across schedules** - Budgets persist and deplete across iterations
6. **Handles failures gracefully** - Interactive recovery options (Retry/Skip/Continue/Quit)
7. **Saves checkpoints** - Full state saved after each schedule for resumption

## Quick Start

### Basic Usage
```bash
cargo run --release --bin simulator-cli
```

This will:
- Connect to your database
- Find all test accounts
- Load all markets
- Initialize 1M token budget for each account/asset
- Start running continuous trading schedules
- Auto-continue to next schedule when completed
- Exit on Ctrl+C (after saving state)

### With Custom Parameters
```bash
cargo run --release --bin simulator-cli -- \
  --trades-per-account 20 \
  --min-amount 50 \
  --max-amount 10000 \
  --initial-budget 5000000 \
  --no-auto-continue \
  --iterations 5
```

## CLI Arguments

All arguments are optional with sensible defaults.

### Budget & Trading
```
--initial-budget <N>        Initial budget per account/asset in tokens
                            Default: 1,000,000 (1M)

--trades-per-account <N>    Number of trades per account per schedule
                            Default: 10

--min-amount <N>           Minimum single trade amount
                            Default: 100

--max-amount <N>           Maximum single trade amount
                            Default: 5,000
```

### Price Offsets
```
--bid-price-offset <F>      Bid price multiplier
                            Default: 1.0 (no offset)

--ask-price-offset <F>      Ask price multiplier
                            Default: 1.0 (no offset)
```

### Account & Market Discovery
```
--account-filter <PREFIX>   Account prefix to match
                            Default: "test-account"
                            (matches test-account, test-account-1, etc.)
```

### Execution Control
```
--no-auto-continue         Require user confirmation between schedules
                            Default: auto-continue enabled

--iterations <N>           Maximum number of schedules to run
                            Default: unlimited (continue until Ctrl+C)

--state-dir <PATH>         Directory for checkpoint files
                            Default: ./simulator_state
```

### Help
```bash
cargo run --bin simulator-cli -- --help
```

## Configuration Examples

### Example 1: Light Testing
```bash
cargo run --release --bin simulator-cli -- \
  --trades-per-account 3 \
  --min-amount 10 \
  --max-amount 100 \
  --initial-budget 50000 \
  --no-auto-continue
```

### Example 2: Heavy Load Testing
```bash
cargo run --release --bin simulator-cli -- \
  --trades-per-account 50 \
  --min-amount 1000 \
  --max-amount 100000 \
  --initial-budget 10000000 \
  --iterations 100
```

### Example 3: Specific Test Accounts
```bash
cargo run --release --bin simulator-cli -- \
  --account-filter "test-bot" \
  --trades-per-account 5
```

(Only accounts with `linked_account_id` starting with "test-bot" will trade)

### Example 4: Price Discipline Testing
```bash
cargo run --release --bin simulator-cli -- \
  --bid-price-offset 1.05 \
  --ask-price-offset 0.95
```

(5% premium on bids, 5% discount on asks)

## Understanding the Output

### Startup
```
Orderbook Simulator CLI
======================

Connecting to database...
Discovering accounts matching 'test-account'...
Found 10 test accounts
Discovering markets...
Found 5 markets
Initialized 30 budgets (10x3 = 1000000 per account/asset)

State checkpoints will be saved to: ./simulator_state

Configuration:
  Trades per account: 10
  Trade amount range: 100 - 5000
  Auto-continue: true

Starting continuous simulation...
Press Ctrl+C to stop
```

### During Schedule
```
──────────────────────────────────────────────────────────────────────────
SCHEDULE 1
──────────────────────────────────────────────────────────────────────────
Generated 100 slots for 10 accounts across 5 markets
✓ [1/100] Processing slot #0...
✓ [2/100] Processing slot #1...
...
[99/100] Processing slot #98...
[100/100] Processing slot #99...

Schedule Summary:
  Slots: 98 completed, 1 failed, 1 skipped
  Orders: 98 created
  Matches: 96
  Budget utilization: 4.90%
    Spent: 49,000 / 1,000,000
    Remaining: 951,000 (available) + 0 (locked) = 951,000
✓ State saved to checkpoint
```

### On Failure
```
───────────────────────────────────────────────
Slot #47 (Account: 12345678-...) encountered an error:
Error: Insufficient budget for asset XYZ
Attempts: 1/3
───────────────────────────────────────────────

What would you like to do?
  [1] Retry      - Attempt the slot again
  [2] Skip       - Skip this slot and mark as completed
  [3] Continue   - Continue without marking as completed
  [4] Quit       - Quit the entire simulation

Enter choice (1-4):
```

### Final Summary
```
==============================================================================
CUMULATIVE SIMULATION SUMMARY
==============================================================================
Schedules run: 10
Total slots processed: 1000
  Completed: 980 (98.0%)
  Failed: 15
  Skipped: 5
Orders created: 980
Total matches: 960
Elapsed time: 45.23s
==============================================================================
```

## Budget Behavior

### Initial State
```
Schedule 1: 10 accounts × 3 assets = 30 budgets
Each budget: 1,000,000 tokens
Total: 30,000,000 tokens allocated
```

### After Schedule 1
```
Average trade size: 1,000 tokens
Average per account: 10,000 tokens spent per asset
Remaining: 990,000 per budget

Total remaining: ~29,700,000 tokens
Utilization: 1%
```

### After Multiple Schedules
```
Schedule 2: Uses same BudgetStore
  Budgets continue from Schedule 1 state
  No reset between schedules

Schedule 5:
  Some accounts may have depleted assets
  Only non-depleted assets can be traded
  Depletion warning shown: "⚠ 2 accounts/assets depleted"

Schedule 10:
  Most budgets significantly depleted
  Fewer available assets to trade
  Simulation naturally winds down
```

## Stopping the Simulation

### Graceful Stop (Recommended)
```
Press Ctrl+C
```
The simulator will:
1. Finish current slot execution
2. Save state checkpoint
3. Print final summary
4. Exit cleanly

### Resume from Checkpoint
After stopping, restart with same or updated parameters:
```bash
cargo run --release --bin simulator-cli
```

The CLI will automatically load and resume from the last checkpoint in `./simulator_state/`.

## Database Setup

### Required
- PostgreSQL database with Cradle schema
- Test accounts with `linked_account_id` starting with your filter prefix
- At least one market with two assets
- DATABASE_URL environment variable set

### Example Database Setup
```bash
export DATABASE_URL="postgres://user:password@localhost/cradle_testdb"
```

## Performance Tuning

### For Quick Tests
```bash
--trades-per-account 3 \
--min-amount 10 \
--max-amount 100 \
--iterations 2
```

### For Stress Testing
```bash
--trades-per-account 100 \
--min-amount 100 \
--max-amount 100000 \
--iterations 1000
```

### For Load Testing (Multiple Instances)
```bash
# Terminal 1
cargo run --release --bin simulator-cli -- \
  --account-filter "test-account-batch-1"

# Terminal 2
cargo run --release --bin simulator-cli -- \
  --account-filter "test-account-batch-2"
```

## Troubleshooting

### "No test accounts found"
- Verify accounts exist in database
- Check `linked_account_id` format matches `--account-filter`
- Try more permissive filter: `--account-filter ""`  (matches all)

### "No markets found"
- Verify markets exist in database
- Check that markets have valid asset references

### "Insufficient budget for asset"
- Budget depleted for that account/asset
- Reduce trade amounts or increase initial budget
- Run with higher `--initial-budget`

### "Checkpoint load failed"
- Delete `./simulator_state/` directory
- Restart from fresh state
- Previous checkpoints will be lost

### Slow Performance
- Reduce `--trades-per-account`
- Increase `--min-amount` and `--max-amount` (fewer smaller orders)
- Reduce number of accounts via `--account-filter`
- Check database performance

## Advanced Usage

### Analyzing Results
Checkpoints are saved as JSON in `./simulator_state/`:
```json
{
  "simulation_id": "...",
  "slots": [...],
  "current_slot_index": 1000,
  "stats": {
    "total_slots": 1000,
    "completed_slots": 980,
    "failed_slots": 15,
    "...": "..."
  }
}
```

### Custom Budget Allocation
Edit budget initialization in code or use highest `--initial-budget`:
```bash
--initial-budget 10000000  # 10M per account/asset
```

### Budget Depletion Analysis
Monitor the "Budget utilization" line during execution:
```
Budget utilization: 45.23%
  Spent: 452,300 / 1,000,000
  Remaining: 547,700 (available) + 0 (locked) = 547,700
⚠ 2 accounts/assets depleted
```

When utilization approaches 100%, trading slows as budgets deplete.

## Files & Directories

```
simulator-cli                   # Binary executable
./simulator_state/              # Checkpoint directory
  ├── {simulation_id_1}.json   # Schedule 1 checkpoint
  ├── {simulation_id_2}.json   # Schedule 2 checkpoint
  └── ...
```

## Integration Notes

The simulator is a standalone tool for testing the orderbook. To integrate actual order placement:

1. Implement the executor closure in `bin/simulator-cli.rs`
2. Replace the stub `place_order()` logic with actual orderbook calls
3. Use `ActionRouter.OrderBook.PlaceOrder` from the main application

See `INTEGRATION_EXAMPLE.rs` for implementation details.

## Support

- Check `ORDERBOOK_SIMULATOR.md` for architecture details
- See `SIMULATOR_GUIDE.md` for programmatic API
- Review `bin/simulator-cli.rs` for CLI implementation

---

**Ready to test!** Start with:
```bash
cargo run --release --bin simulator-cli
```
