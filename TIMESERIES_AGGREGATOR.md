# Cradle OHLC Time Series Aggregator Tool

## Overview

The `timeseries-aggregator` is a comprehensive command-line tool for aggregating orderbook trades into OHLC (Open, High, Low, Close) bars at multiple time intervals. It supports both interactive and command-line modes, with built-in checkpoint/resume functionality for fault-tolerant backfill operations.

## Quick Start

### Interactive Mode (Recommended for First Use)

Simply run the tool without arguments:

```bash
cargo run --bin timeseries-aggregator
```

This launches an interactive menu system that guides you through:
1. Selecting an operation (Backfill, Resume, Single Run, Realtime, or List)
2. Choosing scope (Single market/asset, all assets in a market, or all markets)
3. Selecting specific market(s) and asset(s)
4. Choosing aggregation interval
5. Setting time range
6. Confirming configuration before execution

### Command-Line Mode

Run with arguments for non-interactive operation:

```bash
cargo run --bin timeseries-aggregator -- --market <uuid> --asset <uuid> --interval 1day --duration 30d
```

## Modes of Operation

### 1. Backfill Mode

**Description:** Start a fresh aggregation from the beginning, clearing any existing checkpoints.

**When to use:** First-time setup, historical data backfill, or restarting aggregation from scratch.

**Interactive:**
```
? Select operation
> Backfill
  Resume
  Single Run
  Realtime
  List Markets
```

**Command-line:**
```bash
cargo run --bin timeseries-aggregator -- \
  --market 550e8400-e29b-41d4-a716-446655440000 \
  --asset 550e8400-e29b-41d4-a716-446655440001 \
  --interval 1day \
  --duration all \
  --mode backfill
```

**What happens:**
- Clears any existing checkpoint for the market/asset/interval combination
- Processes entire time range specified
- Saves checkpoint after each interval
- Can be interrupted and resumed later with `--mode resume`

**Example: Backfill last 90 days of hourly bars**
```bash
cargo run --bin timeseries-aggregator -- \
  --market 550e8400-e29b-41d4-a716-446655440000 \
  --asset 550e8400-e29b-41d4-a716-446655440001 \
  --interval 1hr \
  --duration 90d \
  --mode backfill
```

---

### 2. Resume Mode

**Description:** Continue a backfill operation from the last checkpoint.

**When to use:** After a backfill was interrupted (crash, service restart, manual stop), resume from where it left off without losing progress.

**Interactive:**
```
? Select operation
  Backfill
> Resume
  Single Run
  Realtime
  List Markets
```

**Command-line:**
```bash
cargo run --bin timeseries-aggregator -- \
  --market 550e8400-e29b-41d4-a716-446655440000 \
  --asset 550e8400-e29b-41d4-a716-446655440001 \
  --interval 1day \
  --duration all \
  --mode resume
```

**What happens:**
- Reads last checkpoint from kvstore
- Resumes aggregation from that timestamp
- Continues to end time specified
- Updates checkpoint as it progresses
- Safe to interrupt and resume again later

**Example: Resume interrupted daily aggregation**
```bash
cargo run --bin timeseries-aggregator -- \
  --market 550e8400-e29b-41d4-a716-446655440000 \
  --asset 550e8400-e29b-41d4-a716-446655440001 \
  --interval 1day \
  --start "2024-01-01 00:00:00" \
  --end "2024-12-31 23:59:59" \
  --mode resume
```

---

### 3. Single Run Mode

**Description:** Process a single time window without checkpoints - one-time aggregation.

**When to use:** Live aggregation, computing a specific time period, or testing without affecting checkpoints.

**Interactive:**
```
? Select operation
  Backfill
  Resume
> Single Run
  Realtime
  List Markets
```

**Command-line:**
```bash
cargo run --bin timeseries-aggregator -- \
  --market 550e8400-e29b-41d4-a716-446655440000 \
  --asset 550e8400-e29b-41d4-a716-446655440001 \
  --interval 15min \
  --duration 24h \
  --mode single
```

**What happens:**
- Aggregates the specified time range
- Does NOT save or read checkpoints
- Does NOT affect existing checkpoint state
- Safe for testing without side effects

**Example: Quick aggregation of last hour at 15-minute intervals**
```bash
cargo run --bin timeseries-aggregator -- \
  --market 550e8400-e29b-41d4-a716-446655440000 \
  --asset 550e8400-e29b-41d4-a716-446655440001 \
  --interval 15min \
  --duration 24h \
  --mode single
```

---

### 4. Realtime Mode

**Description:** Continuously create new OHLC bars as time passes, indefinitely.

**When to use:** Live market data aggregation, continuous bar generation, production aggregation service.

**Interactive:**
```
? Select operation
  Backfill
  Resume
  Single Run
> Realtime
  List Markets
```

**Command-line:**
```bash
cargo run --bin timeseries-aggregator -- \
  --market 550e8400-e29b-41d4-a716-446655440000 \
  --asset 550e8400-e29b-41d4-a716-446655440001 \
  --interval 1min \
  --mode realtime
```

**What happens:**
- Runs indefinitely
- Creates new bar for each interval as time passes
- For interval `1min`: creates new bar every minute at [now-1min, now]
- Can be stopped with Ctrl+C
- Safe to restart (doesn't duplicate bars)

**Example: Live 5-minute bars**
```bash
cargo run --bin timeseries-aggregator -- \
  --market 550e8400-e29b-41d4-a716-446655440000 \
  --asset 550e8400-e29b-41d4-a716-446655440001 \
  --interval 5min \
  --mode realtime
```

**Production Usage:**
```bash
# Run in background with logging
nohup cargo run --bin timeseries-aggregator -- \
  --market 550e8400-e29b-41d4-a716-446655440000 \
  --asset 550e8400-e29b-41d4-a716-446655440001 \
  --interval 1min \
  --mode realtime \
  > aggregator.log 2>&1 &
```

---

### 5. List Mode

**Description:** Display all available markets and their assets.

**When to use:** Discovering market UUIDs and asset IDs without needing the database directly.

**Interactive:**
```
? Select operation
  Backfill
  Resume
  Single Run
  Realtime
> List Markets
```

**Command-line:**
```bash
cargo run --bin timeseries-aggregator -- --mode list
```

**Output Example:**
```
╔═══════════════════════════════════════════════════════╗
║     Cradle OHLC Time Series Aggregator Tool          ║
╚═══════════════════════════════════════════════════════╝

Available Markets and Assets:
  Market: BTC/USD
    UUID: 550e8400-e29b-41d4-a716-446655440000
    ├─ BTC (550e8400-e29b-41d4-a716-446655440001)
    ├─ USD (550e8400-e29b-41d4-a716-446655440002)

  Market: ETH/USD
    UUID: 550e8400-e29b-41d4-a716-446655440003
    ├─ ETH (550e8400-e29b-41d4-a716-446655440004)
    ├─ USD (550e8400-e29b-41d4-a716-446655440005)
```

---

## Scope Options

### Single Market/Asset

Process one specific market and one specific asset.

**Interactive:**
```
? Select scope
> Single Market/Asset
  All Assets in Market
  All Markets and Assets

? Select market
> BTC/USD
  ETH/USD

? Select asset
> BTC
  USD
```

**Command-line:**
```bash
# Default scope is "single"
cargo run --bin timeseries-aggregator -- \
  --market 550e8400-e29b-41d4-a716-446655440000 \
  --asset 550e8400-e29b-41d4-a716-446655440001 \
  --interval 1day \
  --duration 30d
```

**Use case:** Fine-grained control, testing single market pairs, specific data aggregation.

---

### Market All (All Assets in Market)

Process all assets within a specific market.

**Interactive:**
```
? Select scope
  Single Market/Asset
> All Assets in Market
  All Markets and Assets

? Select market
> BTC/USD
  ETH/USD

? Process assets how?
> Sequential
  Parallel
```

**Command-line:**
```bash
cargo run --bin timeseries-aggregator -- \
  --market 550e8400-e29b-41d4-a716-446655440000 \
  --scope market-all \
  --interval 1day \
  --duration 30d
```

**What happens:**
- Processes BTC asset, then USD asset in sequence
- Creates OHLC bars for each asset
- Can show counts for each market/asset pair

**Use case:** Aggregating both sides of a market pair, comprehensive market analysis.

---

### Global All (All Markets and Assets)

Process every market and every asset in the system.

**Interactive:**
```
? Select scope
  Single Market/Asset
  All Assets in Market
> All Markets and Assets

? Process markets how?
> Sequential
  Parallel
```

**Command-line:**
```bash
cargo run --bin timeseries-aggregator -- \
  --scope all \
  --interval 1day \
  --duration 7d
```

**What happens:**
- Iterates through every market
- For each market, aggregates all assets
- Shows progress for each combination
- Final summary with total records created

**Use case:** System-wide backfill, comprehensive historical aggregation.

**Example: Backfill all markets with 1-week of daily bars**
```bash
cargo run --bin timeseries-aggregator -- \
  --scope all \
  --interval 1day \
  --duration 7d \
  --mode backfill
```

---

## Time Range Options

### Duration Presets

#### Last 24 Hours (24h)

Aggregates data from 24 hours ago to now.

```bash
cargo run --bin timeseries-aggregator -- \
  --market 550e8400-e29b-41d4-a716-446655440000 \
  --asset 550e8400-e29b-41d4-a716-446655440001 \
  --interval 1hr \
  --duration 24h
```

**Interactive Selection:** Select "Last 24 hours" from time range menu

---

#### Last 7 Days (7d)

Aggregates data from 7 days ago to now.

```bash
cargo run --bin timeseries-aggregator -- \
  --market 550e8400-e29b-41d4-a716-446655440000 \
  --asset 550e8400-e29b-41d4-a716-446655440001 \
  --interval 4hr \
  --duration 7d
```

**Interactive Selection:** Select "Last 7 days" from time range menu

---

#### Last 30 Days (30d)

Aggregates data from 30 days ago to now.

```bash
cargo run --bin timeseries-aggregator -- \
  --market 550e8400-e29b-41d4-a716-446655440000 \
  --asset 550e8400-e29b-41d4-a716-446655440001 \
  --interval 1day \
  --duration 30d
```

**Interactive Selection:** Select "Last 30 days" from time range menu

---

#### Last 90 Days (90d)

Aggregates data from 90 days ago to now.

```bash
cargo run --bin timeseries-aggregator -- \
  --market 550e8400-e29b-41d4-a716-446655440000 \
  --asset 550e8400-e29b-41d4-a716-446655440001 \
  --interval 1day \
  --duration 90d
```

**Interactive Selection:** Select "Last 90 days" from time range menu

---

#### All Time

Aggregates all available historical data (approximately 100 years back).

```bash
cargo run --bin timeseries-aggregator -- \
  --market 550e8400-e29b-41d4-a716-446655440000 \
  --asset 550e8400-e29b-41d4-a716-446655440001 \
  --interval 1day \
  --duration all
```

**Interactive Selection:** Select "All time" from time range menu

**Note:** This triggers a full historical backfill. Combine with `--mode backfill` for checkpoint support.

---

### Custom Time Range

Specify exact start and end timestamps.

```bash
cargo run --bin timeseries-aggregator -- \
  --market 550e8400-e29b-41d4-a716-446655440000 \
  --asset 550e8400-e29b-41d4-a716-446655440001 \
  --interval 1day \
  --start "2024-01-01 00:00:00" \
  --end "2024-12-31 23:59:59"
```

**Format:** `YYYY-MM-DD HH:MM:SS`

**Examples:**

```bash
# Q1 2024
cargo run --bin timeseries-aggregator -- \
  --market 550e8400-e29b-41d4-a716-446655440000 \
  --asset 550e8400-e29b-41d4-a716-446655440001 \
  --interval 1day \
  --start "2024-01-01 00:00:00" \
  --end "2024-03-31 23:59:59"

# Specific week
cargo run --bin timeseries-aggregator -- \
  --market 550e8400-e29b-41d4-a716-446655440000 \
  --asset 550e8400-e29b-41d4-a716-446655440001 \
  --interval 1hr \
  --start "2024-10-01 00:00:00" \
  --end "2024-10-07 23:59:59"

# One trading day
cargo run --bin timeseries-aggregator -- \
  --market 550e8400-e29b-41d4-a716-446655440000 \
  --asset 550e8400-e29b-41d4-a716-446655440001 \
  --interval 15min \
  --start "2024-10-25 09:30:00" \
  --end "2024-10-25 16:00:00"
```

**Use case:** Analyzing specific periods, testing, reproducing issues.

---

## Interval Options

### Sub-Minute Intervals

#### 15 Seconds (15secs)
```bash
cargo run --bin timeseries-aggregator -- \
  --interval 15secs \
  --market <uuid> --asset <uuid> --duration 24h
```
- Use for: High-frequency trading data, tick analysis
- Data density: 3,600 bars per day

#### 30 Seconds (30secs)
```bash
cargo run --bin timeseries-aggregator -- \
  --interval 30secs \
  --market <uuid> --asset <uuid> --duration 24h
```
- Use for: Real-time monitoring, technical indicators
- Data density: 1,440 bars per day

#### 45 Seconds (45secs)
```bash
cargo run --bin timeseries-aggregator -- \
  --interval 45secs \
  --market <uuid> --asset <uuid> --duration 24h
```
- Use for: Medium-frequency analysis
- Data density: 1,920 bars per day

---

### Minute Intervals

#### 1 Minute (1min)
```bash
cargo run --bin timeseries-aggregator -- \
  --interval 1min \
  --market <uuid> --asset <uuid> --duration 24h
```
- Use for: Short-term trading, scalping strategies
- Data density: 1,440 bars per day

#### 5 Minutes (5min)
```bash
cargo run --bin timeseries-aggregator -- \
  --interval 5min \
  --market <uuid> --asset <uuid> --duration 30d
```
- Use for: Intraday trading, swing signals
- Data density: 288 bars per day

#### 15 Minutes (15min)
```bash
cargo run --bin timeseries-aggregator -- \
  --interval 15min \
  --market <uuid> --asset <uuid> --duration 30d
```
- Use for: Standard intraday charts, technical analysis
- Data density: 96 bars per day

#### 30 Minutes (30min)
```bash
cargo run --bin timeseries-aggregator -- \
  --interval 30min \
  --market <uuid> --asset <uuid> --duration 30d
```
- Use for: Flexible intraday analysis
- Data density: 48 bars per day

---

### Hourly Intervals

#### 1 Hour (1hr)
```bash
cargo run --bin timeseries-aggregator -- \
  --interval 1hr \
  --market <uuid> --asset <uuid> --duration 90d
```
- Use for: Daily trading, position analysis
- Data density: 24 bars per day

#### 4 Hours (4hr)
```bash
cargo run --bin timeseries-aggregator -- \
  --interval 4hr \
  --market <uuid> --asset <uuid> --duration 90d
```
- Use for: Position traders, swing traders
- Data density: 6 bars per day

---

### Daily and Weekly Intervals

#### 1 Day (1day)
```bash
cargo run --bin timeseries-aggregator -- \
  --interval 1day \
  --market <uuid> --asset <uuid> --duration all
```
- Use for: Long-term trends, historical analysis
- Data density: 1 bar per day

#### 1 Week (1week)
```bash
cargo run --bin timeseries-aggregator -- \
  --interval 1week \
  --market <uuid> --asset <uuid> --duration all
```
- Use for: Weekly trends, macro analysis
- Data density: 1 bar per week

---

## Complete Command Examples

### Example 1: Quick 24-Hour Backfill

Backfill the last 24 hours of 15-minute bars for a single market/asset pair.

```bash
cargo run --bin timeseries-aggregator -- \
  --market 550e8400-e29b-41d4-a716-446655440000 \
  --asset 550e8400-e29b-41d4-a716-446655440001 \
  --interval 15min \
  --duration 24h \
  --mode backfill
```

**Expected output:**
```
╔═══════════════════════════════════════════════════════╗
║     Cradle OHLC Time Series Aggregator Tool          ║
╚═══════════════════════════════════════════════════════╝

  [550e8400-e29b-41d4-a716-446655440000] [550e8400-e29b-41d4-a716-446655440001] ✓ completed

✓ Total records created: 1
✓ Operation completed successfully
```

---

### Example 2: Month-Long Daily Aggregation with Resume Support

Backfill a full month of daily bars with checkpoint support.

```bash
# Start backfill
cargo run --bin timeseries-aggregator -- \
  --market 550e8400-e29b-41d4-a716-446655440000 \
  --asset 550e8400-e29b-41d4-a716-446655440001 \
  --interval 1day \
  --duration 30d \
  --mode backfill
```

If interrupted, resume with:

```bash
# Resume from checkpoint
cargo run --bin timeseries-aggregator -- \
  --market 550e8400-e29b-41d4-a716-446655440000 \
  --asset 550e8400-e29b-41d4-a716-446655440001 \
  --interval 1day \
  --duration 30d \
  --mode resume
```

---

### Example 3: Multi-Market Backfill

Backfill all assets in all markets with 1-week of daily bars.

```bash
cargo run --bin timeseries-aggregator -- \
  --scope all \
  --interval 1day \
  --duration 7d \
  --mode backfill \
  --confirm
```

The `--confirm` flag skips the interactive confirmation prompt.

---

### Example 4: Production Realtime Aggregation

Run continuous 1-minute bar generation in production.

```bash
# Start background service
nohup cargo run --bin timeseries-aggregator -- \
  --market 550e8400-e29b-41d4-a716-446655440000 \
  --asset 550e8400-e29b-41d4-a716-446655440001 \
  --interval 1min \
  --mode realtime \
  > /var/log/aggregator.log 2>&1 &
```

To monitor:
```bash
tail -f /var/log/aggregator.log
```

To stop:
```bash
pkill -f "timeseries-aggregator"
```

---

### Example 5: Custom Date Range Analysis

Aggregate specific trading week at 15-minute intervals.

```bash
cargo run --bin timeseries-aggregator -- \
  --market 550e8400-e29b-41d4-a716-446655440000 \
  --asset 550e8400-e29b-41d4-a716-446655440001 \
  --interval 15min \
  --start "2024-10-21 00:00:00" \
  --end "2024-10-25 23:59:59" \
  --mode single
```

---

### Example 6: All Markets Quick Test

Test aggregation across all markets for last 24 hours without affecting checkpoints.

```bash
cargo run --bin timeseries-aggregator -- \
  --scope all \
  --interval 4hr \
  --duration 24h \
  --mode single
```

---

### Example 7: Market-Specific Asset Pair Aggregation

Backfill all assets in BTC/USD market from scratch.

```bash
cargo run --bin timeseries-aggregator -- \
  --market 550e8400-e29b-41d4-a716-446655440000 \
  --scope market-all \
  --interval 1day \
  --duration all \
  --mode backfill
```

---

## Interactive Mode Walkthrough

### Starting the Tool

```bash
cargo run --bin timeseries-aggregator
```

### Step 1: Operation Selection

```
? Select operation
> Backfill
  Resume
  Single Run
  Realtime
  List Markets
```

Press **Up/Down arrows** to navigate, **Enter** to select.

### Step 2: Scope Selection

```
? Select scope
> Single Market/Asset
  All Assets in Market
  All Markets and Assets
```

### Step 3: Market Selection (if applicable)

```
? Select market
> BTC/USD
  ETH/USD
  ADA/USDT
```

### Step 4: Asset Selection (if single scope)

```
? Select asset
> BTC
  USD
```

### Step 5: Interval Selection

```
? Select aggregation interval
> 15 seconds
  30 seconds
  45 seconds
  1 minute
  5 minutes
  15 minutes
  30 minutes
  1 hour
  4 hours
  1 day
  1 week
```

### Step 6: Time Range Selection (unless Realtime mode)

```
? Select time range
> Last 24 hours
  Last 7 days
  Last 30 days
  Last 90 days
  All time
```

### Step 7: Configuration Summary

```
╔═══════════════════════════════════════════════════════╗
║                    Configuration Summary              ║
╚═══════════════════════════════════════════════════════╝
  Mode: Backfill
  Markets: 1
  Assets: 1
  Start: 2024-10-23 13:45:32
  End: 2024-10-30 13:45:32

? Proceed with aggregation?
> Yes
  No
```

### Step 8: Execution

```
Starting aggregation...
  [550e8400-e29b-41d4-a716-446655440000] [550e8400-e29b-41d4-a716-446655440001] ✓ completed

✓ Total records created: 1
✓ Operation completed successfully
```

---

## Checkpoint Management

### Understanding Checkpoints

Checkpoints are saved to the `kvstore` table with keys in format:
```
aggregator:{market_id}:{asset_id}:{interval}:last_processed
```

They store the timestamp of the last successfully processed interval.

### Clearing a Checkpoint

To restart aggregation from scratch (clears saved progress):

**Interactive:**
1. Select operation: Choose "List Markets" first to get UUIDs if needed
2. Run tool again with `--mode backfill` (automatically clears checkpoint)

**Command-line:**
```bash
# Backfill mode automatically clears the checkpoint
cargo run --bin timeseries-aggregator -- \
  --market <uuid> \
  --asset <uuid> \
  --interval 1day \
  --duration all \
  --mode backfill
```

### Checking Checkpoint Status

Query the kvstore table directly:

```sql
SELECT key, value FROM kvstore
WHERE key LIKE 'aggregator:%'
ORDER BY key;
```

Example output:
```
key                                                                           | value
aggregator:550e8400-e29b-41d4-a716-446655440000:550e8400-e29b-41d4-a716... | 2024-10-28 14:30:00
```

---

## Advanced Usage

### Scripting and Automation

#### Bash Script: Backfill All Markets Daily

```bash
#!/bin/bash
# daily_backfill.sh

BINARY="cargo run --bin timeseries-aggregator --"

# Backfill all markets with last 24 hours of daily bars
$BINARY \
  --scope all \
  --interval 1day \
  --duration 24h \
  --mode backfill \
  --confirm

if [ $? -eq 0 ]; then
  echo "Backfill completed successfully"
  # Send notification, log to monitoring system, etc.
else
  echo "Backfill failed"
  # Alert operators
  exit 1
fi
```

Run daily via cron:
```cron
0 2 * * * /path/to/daily_backfill.sh >> /var/log/backfill.log 2>&1
```

#### Bash Script: Resume Interrupted Backfill

```bash
#!/bin/bash
# resume_backfill.sh

MARKET_UUID="550e8400-e29b-41d4-a716-446655440000"
ASSET_UUID="550e8400-e29b-41d4-a716-446655440001"
INTERVAL="1day"
DURATION="all"

while true; do
  echo "Resuming aggregation at $(date)"

  cargo run --bin timeseries-aggregator -- \
    --market "$MARKET_UUID" \
    --asset "$ASSET_UUID" \
    --interval "$INTERVAL" \
    --duration "$DURATION" \
    --mode resume \
    --confirm

  if [ $? -eq 0 ]; then
    echo "Aggregation completed successfully"
    break
  else
    echo "Aggregation failed, retrying in 60 seconds..."
    sleep 60
  fi
done
```

---

### Docker Container Usage

If running in Docker:

```dockerfile
FROM rust:latest

WORKDIR /app
COPY . .

RUN cargo build --bin timeseries-aggregator --release

CMD ["cargo", "run", "--bin", "timeseries-aggregator", "--", \
     "--market", "$MARKET_UUID", \
     "--asset", "$ASSET_UUID", \
     "--interval", "$INTERVAL", \
     "--mode", "realtime"]
```

Run with environment variables:
```bash
docker run -e MARKET_UUID=<uuid> \
           -e ASSET_UUID=<uuid> \
           -e INTERVAL=1min \
           -e DATABASE_URL=postgres://... \
           cradle-aggregator:latest
```

---

### Kubernetes Deployment

```yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: timeseries-aggregator-backfill
spec:
  schedule: "0 2 * * *"  # Daily at 2 AM
  jobTemplate:
    spec:
      template:
        spec:
          containers:
          - name: aggregator
            image: cradle-aggregator:latest
            args:
            - "--scope=all"
            - "--interval=1day"
            - "--duration=24h"
            - "--mode=backfill"
            - "--confirm"
            env:
            - name: DATABASE_URL
              valueFrom:
                secretKeyRef:
                  name: db-credentials
                  key: connection-url
          restartPolicy: OnFailure
```

---

## Troubleshooting

### Issue: "No markets found in database"

**Cause:** Database has no market data.

**Solution:**
1. Verify database connection: `echo $DATABASE_URL`
2. Check if markets exist:
   ```sql
   SELECT COUNT(*) FROM markets;
   ```
3. If empty, seed test data

---

### Issue: "Connection refused"

**Cause:** Database server not running or wrong connection URL.

**Solution:**
```bash
# Check connection string
echo $DATABASE_URL

# Test connection
psql $DATABASE_URL -c "SELECT 1"

# Verify server is running
ps aux | grep postgres
```

---

### Issue: Out of Memory During Large Backfill

**Cause:** Processing too many markets at once with high-frequency intervals.

**Solution:**
1. Process by scope: Use `--scope single` instead of `--scope all`
2. Reduce interval: Process daily bars instead of minutely
3. Process in smaller chunks: Split time ranges

```bash
# Instead of:
# cargo run --bin timeseries-aggregator -- --scope all --interval 1min

# Do:
cargo run --bin timeseries-aggregator -- \
  --scope single \
  --interval 1day \
  --start "2024-01-01" --end "2024-03-31"
```

---

### Issue: Checkpoint Not Being Saved

**Cause:** Checkpoints only save in `backfill` and `resume` modes.

**Solution:**
Verify you're using correct mode:
```bash
# Checkpoints ARE saved:
cargo run --bin timeseries-aggregator -- --mode backfill
cargo run --bin timeseries-aggregator -- --mode resume

# Checkpoints are NOT saved:
cargo run --bin timeseries-aggregator -- --mode single
cargo run --bin timeseries-aggregator -- --mode realtime
```

---

## Performance Considerations

### Recommended Configurations

**High-Frequency Trading (5-min or less):**
```bash
cargo run --bin timeseries-aggregator -- \
  --scope single \
  --interval 5min \
  --duration 7d
```

**Daily Analysis (1-day bars):**
```bash
cargo run --bin timeseries-aggregator -- \
  --scope all \
  --interval 1day \
  --duration 90d
```

**Real-Time Monitoring (1-min bars):**
```bash
cargo run --bin timeseries-aggregator -- \
  --scope single \
  --interval 1min \
  --mode realtime
```

### Database Connection Pool

The tool uses a connection pool with max 5 connections. For large-scale operations, adjust in the source code or request higher limits from DBA.

---

## FAQ

**Q: Can I run multiple aggregators simultaneously?**
A: Yes! Use different `--market` and `--asset` combinations. Checkpoints prevent duplicate processing.

**Q: How long does a full backfill take?**
A: Depends on:
- Number of trades in database
- Interval chosen (smaller = more bars)
- System resources
- Estimate: ~1-5 minutes per million trades per interval

**Q: Can I modify checkpoints manually?**
A: Yes, via direct database query:
```sql
UPDATE kvstore SET value = '2024-01-01 00:00:00'
WHERE key = 'aggregator:...';
```

**Q: What if I need multiple markets in one run?**
A: Use `--scope market-all` or `--scope all` to process multiple assets/markets.

**Q: Can I run realtime and backfill simultaneously?**
A: Yes, on different markets/assets. Use separate terminal windows or processes.

**Q: Where are the results stored?**
A: In the `markets_time_series` table, with `data_provider_type` set to `OrderBook`.

---

## Support and Monitoring

### Enabling Debug Output

```bash
RUST_LOG=debug cargo run --bin timeseries-aggregator -- ...
```

### Log File Configuration

```bash
# Redirect to file
cargo run --bin timeseries-aggregator -- ... >> aggregator.log 2>&1

# With timestamps
cargo run --bin timeseries-aggregator -- ... | while read line; do echo "[$(date '+%Y-%m-%d %H:%M:%S')] $line"; done >> aggregator.log
```

### Health Checks

```bash
# Verify recent checkpoint updates
SELECT key, value FROM kvstore
WHERE key LIKE 'aggregator:%'
AND value > (NOW() - INTERVAL '1 hour')::timestamp;
```

---

## Summary Table

| Use Case | Command | Mode | Scope |
|----------|---------|------|-------|
| Quick test | `cargo run --bin timeseries-aggregator -- --market <> --asset <> --interval 1hr --duration 24h` | single | single |
| Full backfill | `cargo run --bin timeseries-aggregator -- --market <> --asset <> --interval 1day --duration all --mode backfill` | backfill | single |
| Resume failed backfill | `cargo run --bin timeseries-aggregator -- --market <> --asset <> --interval 1day --duration all --mode resume` | resume | single |
| Live aggregation | `cargo run --bin timeseries-aggregator -- --market <> --asset <> --interval 1min --mode realtime` | realtime | single |
| All markets backfill | `cargo run --bin timeseries-aggregator -- --scope all --interval 1day --duration 7d --mode backfill` | backfill | all |
| List markets | `cargo run --bin timeseries-aggregator -- --mode list` | list | N/A |
| Interactive mode | `cargo run --bin timeseries-aggregator` | (prompt) | (prompt) |

