# Account Generator - Batch Account Creation Simulator

A powerful batch account generation tool for the Cradle platform that creates multiple accounts with automatic token association, KYC handling, and token airdrops, with built-in retry logic and JSON persistence.

## Table of Contents

- [Overview](#overview)
- [Installation & Setup](#installation--setup)
- [Usage](#usage)
- [Configuration Options](#configuration-options)
- [Examples](#examples)
- [Output Format](#output-format)
- [Architecture](#architecture)
- [Advanced Features](#advanced-features)
- [Troubleshooting](#troubleshooting)
- [Next Steps: Airdrop Phase](#next-steps-airdrop-phase)

## Overview

The Account Generator (`account_generator`) is a standalone binary that enables efficient batch creation of Cradle accounts with the following capabilities:

- **Batch Account Creation**: Create 1 to thousands of accounts in a single operation
- **Automatic Token Association**: Optionally associate a list of tokens with each account
- **KYC Processing**: Optionally grant KYC for all associated tokens
- **Automatic Token Airdrops**: Optionally airdrop a configurable amount of each associated token to every account
- **Exponential Backoff Retries**: Intelligent retry mechanism with jitter to handle transient failures
- **Progress Tracking**: Real-time console output showing creation progress
- **JSON Export**: All generated accounts saved to a JSON file for use in subsequent phases
- **Comprehensive Statistics**: Success/failure rates for each operation type
- **Configurable via CLI Arguments**: Full control through command-line flags

## Installation & Setup

### Prerequisites

- Rust 1.90.0 or later
- `DATABASE_URL` environment variable pointing to PostgreSQL database
- Hedera contract credentials (handled via environment - see `cli_helper`)
- Valid asset UUIDs for association (if using asset features)

### Building the Binary

```bash
cd cradle-back-end
cargo build --bin account_generator --release
```

The executable will be available at:
```
target/release/account_generator
```

### Running the Binary

```bash
# Quick start - create 10 accounts
./target/release/account_generator

# With full options including airdrops
./target/release/account_generator \
  --batch-size 50 \
  --account-type retail \
  --assets uuid1,uuid2 \
  --apply-kyc \
  --apply-airdrops \
  --airdrop-amount 1000000 \
  --output my_accounts.json
```

## Usage

### Basic Syntax

```bash
account_generator [OPTIONS]
```

### Minimal Example

```bash
# Creates 10 retail accounts with default settings
account_generator
```

### Full Example

```bash
account_generator \
  --batch-size 100 \
  --account-type retail \
  --assets a1b2c3d4-e5f6-4a5b-6c7d-8e9f0a1b2c3d,b2c3d4e5-f6a7-5b6c-7d8e-9f0a1b2c3d4e \
  --apply-kyc \
  --apply-airdrops \
  --airdrop-amount 1000000 \
  --output test_accounts.json \
  --initial-status verified \
  --retry-limit 5 \
  --retry-delay 1000
```

## Configuration Options

### `--batch-size <COUNT>`

Number of accounts to generate in this batch.

- **Type**: Integer
- **Default**: 10
- **Range**: 1-∞ (practical limit depends on database and contract performance)

**Example**:
```bash
account_generator --batch-size 100
```

### `--account-type <TYPE>`

Account type for all generated accounts. Must be either `retail` or `institutional`.

- **Type**: String (enum)
- **Default**: `retail`
- **Options**: `retail`, `institutional`

**Example**:
```bash
account_generator --account-type institutional
```

### `--assets <UUID1>,<UUID2>,...`

Comma-separated list of asset UUIDs to associate with each generated account. These must be valid Cradle asset IDs that exist in the database.

- **Type**: CSV of UUIDs
- **Default**: (empty - no automatic association)
- **Format**: `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` (standard UUID format)

**Example**:
```bash
account_generator --assets a1b2c3d4-e5f6-4a5b-6c7d-8e9f0a1b2c3d,b2c3d4e5-f6a7-5b6c-7d8e-9f0a1b2c3d4e
```

### `--apply-kyc`

Enable KYC (Know Your Customer) granting for all associated assets. Only relevant if `--assets` is specified.

- **Type**: Boolean flag
- **Default**: false (KYC disabled)

**Example**:
```bash
account_generator --assets uuid1,uuid2 --apply-kyc
```

### `--no-kyc`

Explicitly disable KYC granting (useful when you want to override a config file setting).

- **Type**: Boolean flag
- **Default**: (overrides `--apply-kyc`)

**Example**:
```bash
account_generator --assets uuid1 --no-kyc
```

### `--apply-airdrops`

Enable automatic token airdrops to all generated accounts. Each account receives the configured amount of each associated asset. Only relevant if `--assets` is specified.

- **Type**: Boolean flag
- **Default**: false (airdrops disabled)

**Example**:
```bash
account_generator --assets uuid1,uuid2 --apply-kyc --apply-airdrops
```

### `--airdrop-amount <AMOUNT>`

Amount of each asset to airdrop to each account. Specified as an integer in the smallest unit of the token.

- **Type**: Integer (u64)
- **Default**: 1000000 (1 million in smallest unit)
- **Note**: Airdrops only occur when `--apply-airdrops` is enabled

**Example**:
```bash
account_generator --assets uuid1,uuid2 --apply-airdrops --airdrop-amount 5000000
```

**Common Values**:
- `1000000` - 1 million (default)
- `10000000` - 10 million
- `100000000` - 100 million
- `1000000000` - 1 billion

### `--output <FILE>` or `--output-file <FILE>`

Path where the generated accounts JSON file will be saved. If the file already exists, it will be overwritten.

- **Type**: File path (string)
- **Default**: `simulated_accounts.json`

**Example**:
```bash
account_generator --output ./output/batch_1.json
```

### `--initial-status <STATUS>`

Initial account status for all created accounts. Must be one of: `unverified`, `verified`, `suspended`, or `closed`.

- **Type**: String (enum)
- **Default**: `unverified`
- **Options**: `unverified`, `verified`, `suspended`, `closed`

**Example**:
```bash
account_generator --initial-status verified
```

### `--retry-limit <COUNT>`

Maximum number of retry attempts for each failed operation. Higher values increase resilience to transient failures but also increase execution time on permanent failures.

- **Type**: Integer
- **Default**: 3
- **Recommended**: 3-5 for most use cases

**Example**:
```bash
account_generator --retry-limit 5
```

### `--retry-delay <MILLISECONDS>`

Base delay in milliseconds for exponential backoff. The actual delay increases exponentially with each retry: `delay = base_delay * 2^(attempt-1)` with ±10% jitter.

- **Type**: Integer (milliseconds)
- **Default**: 500 (0.5 seconds)
- **Recommended**: 500-2000

**Example**:
```bash
account_generator --retry-delay 1000
```

**Retry Delay Calculation**:
- Attempt 1: ~500ms
- Attempt 2: ~1000ms
- Attempt 3: ~2000ms
- Attempt 4: ~4000ms
- (capped at 30 seconds)

### `--help` or `-h`

Display help message and exit.

**Example**:
```bash
account_generator --help
```

## Examples

### Example 1: Quick Test - 10 Accounts

```bash
account_generator
```

Creates 10 retail accounts with no asset association, saves to `simulated_accounts.json`.

**Output**:
```
╔═══════════════════════════════════════════════════════╗
║     Cradle Account Generator (Batch Simulator)        ║
╚═══════════════════════════════════════════════════════╝

Initializing app config... ✓ Ready

╔═══════════════════════════════════════════════════════╗
║  Starting Account Generation Batch (10)              ║
╚═══════════════════════════════════════════════════════╝

[1/10] Creating account... ✓
[2/10] Creating account... ✓
...
[10/10] Creating account... ✓

╔═══════════════════════════════════════════════════════╗
║  Batch Generation Complete                            ║
╚═══════════════════════════════════════════════════════╝

  ├─ Account Creation
  │  ├─ Requested: 10
  │  ├─ Created: 10 (100.0%)
  │
  └─ Duration: 15.23s

  ℹ Batch ID: 550e8400-e29b-41d4-a716-446655440000
  ℹ Accounts: 10

Saving batch to JSON... ✓ Done
  ℹ Saved to: simulated_accounts.json
```

### Example 2: Asset Association and KYC

```bash
account_generator \
  --batch-size 20 \
  --account-type retail \
  --assets a1b2c3d4-e5f6-4a5b-6c7d-8e9f0a1b2c3d,b2c3d4e5-f6a7-5b6c-7d8e-9f0a1b2c3d4e \
  --apply-kyc
```

Creates 20 retail accounts, associates 2 assets with each (40 associations total), and grants KYC for all 40 assets.

**Expected Console Output**:
```
[1/20] Creating account... ✓
  └─ Associating assets... ✓
     └─ Granting KYC... ✓
[2/20] Creating account... ✓
  └─ Associating assets... ✓
     └─ Granting KYC... ✓
...

  ├─ Account Creation
  │  ├─ Requested: 20
  │  ├─ Created: 20 (100.0%)
  │
  ├─ Asset Association
  │  ├─ Attempted: 40
  │  └─ Successful: 40 (100.0%)
  │
  ├─ KYC Grants
  │  ├─ Attempted: 40
  │  └─ Successful: 40 (100.0%)
  │
  └─ Duration: 45.67s
```

### Example 2b: Asset Association, KYC, and Airdrops

```bash
account_generator \
  --batch-size 20 \
  --account-type retail \
  --assets a1b2c3d4-e5f6-4a5b-6c7d-8e9f0a1b2c3d,b2c3d4e5-f6a7-5b6c-7d8e-9f0a1b2c3d4e \
  --apply-kyc \
  --apply-airdrops \
  --airdrop-amount 1000000
```

Creates 20 retail accounts, associates 2 assets (40 total), grants KYC (40 total), and airdrops 1 million of each asset to each account (40 airdrops total).

**Expected Console Output**:
```
[1/20] Creating account... ✓
  └─ Associating assets... ✓
     └─ Granting KYC... ✓
        └─ Airdropping tokens... ✓
[2/20] Creating account... ✓
  └─ Associating assets... ✓
     └─ Granting KYC... ✓
        └─ Airdropping tokens... ✓
...

  ├─ Account Creation
  │  ├─ Requested: 20
  │  ├─ Created: 20 (100.0%)
  │
  ├─ Asset Association
  │  ├─ Attempted: 40
  │  └─ Successful: 40 (100.0%)
  │
  ├─ KYC Grants
  │  ├─ Attempted: 40
  │  └─ Successful: 40 (100.0%)
  │
  ├─ Token Airdrops
  │  ├─ Attempted: 40
  │  └─ Successful: 40 (100.0%)
  │
  └─ Duration: 78.45s
```

### Example 3: Institutional Accounts with Custom Status

```bash
account_generator \
  --batch-size 5 \
  --account-type institutional \
  --initial-status verified \
  --output ./institutional_accounts.json
```

Creates 5 institutional accounts with "verified" status.

### Example 4: Large Batch with Retry Configuration

```bash
account_generator \
  --batch-size 1000 \
  --account-type retail \
  --assets uuid1,uuid2,uuid3 \
  --apply-kyc \
  --retry-limit 5 \
  --retry-delay 2000 \
  --output batch_1000.json
```

Creates 1000 accounts with aggressive retry settings for stability on slower networks.

### Example 5: Asset Association Without KYC

```bash
account_generator \
  --batch-size 50 \
  --assets uuid1,uuid2 \
  --no-kyc
```

Creates 50 accounts and associates assets, but does NOT grant KYC.

### Example 6: Full Seeding Pipeline with Airdrops

```bash
account_generator \
  --batch-size 1000 \
  --account-type retail \
  --assets a1b2c3d4-e5f6-4a5b-6c7d-8e9f0a1b2c3d,b2c3d4e5-f6a7-5b6c-7d8e-9f0a1b2c3d4e,c3d4e5f6-a7b8-6c7d-8e9f-0a1b2c3d4e5f \
  --apply-kyc \
  --apply-airdrops \
  --airdrop-amount 1000000 \
  --initial-status verified \
  --retry-limit 5 \
  --retry-delay 1000 \
  --output production_accounts.json
```

Creates 1000 accounts with 3 assets each, fully seeded with:
- All assets associated (3000 associations)
- KYC granted for all assets (3000 grants)
- 1 million of each asset airdropped (3000 airdrops)
- All accounts set to "verified" status
- Aggressive retry configuration for reliability

## Output Format

### JSON File Structure

The generated JSON file contains a complete batch record with all metadata:

```json
{
  "batch_id": "550e8400-e29b-41d4-a716-446655440000",
  "created_at": "2025-10-30T12:34:56.789Z",
  "completed_at": "2025-10-30T12:35:42.123Z",
  "config": {
    "batch_size": 20,
    "account_type": "retail",
    "assets_to_associate": [
      "a1b2c3d4-e5f6-4a5b-6c7d-8e9f0a1b2c3d",
      "b2c3d4e5-f6a7-5b6c-7d8e-9f0a1b2c3d4e"
    ],
    "apply_kyc": true,
    "apply_airdrops": true,
    "airdrop_amount": 1000000,
    "output_file": "simulated_accounts.json",
    "initial_status": "unverified",
    "retry_limit": 3,
    "retry_delay_ms": 500
  },
  "stats": {
    "total_requested": 20,
    "successfully_created": 20,
    "failed_count": 0,
    "total_associations": 40,
    "successful_associations": 40,
    "total_kyc_grants": 40,
    "successful_kyc_grants": 40,
    "total_airdrops": 40,
    "successful_airdrops": 40
  },
  "accounts": [
    {
      "cradle_account_id": "11111111-1111-1111-1111-111111111111",
      "linked_account_id": "test-account-1698681296-abc12345",
      "wallet_id": "22222222-2222-2222-2222-222222222222",
      "wallet_address": "0x1234567890abcdef1234567890abcdef12345678",
      "contract_id": "0.0.12345",
      "account_type": "retail",
      "status": "unverified",
      "associated_assets": [
        "a1b2c3d4-e5f6-4a5b-6c7d-8e9f0a1b2c3d",
        "b2c3d4e5-f6a7-5b6c-7d8e-9f0a1b2c3d4e"
      ],
      "kyc_assets": [
        "a1b2c3d4-e5f6-4a5b-6c7d-8e9f0a1b2c3d",
        "b2c3d4e5-f6a7-5b6c-7d8e-9f0a1b2c3d4e"
      ],
      "airdropped_assets": [
        "a1b2c3d4-e5f6-4a5b-6c7d-8e9f0a1b2c3d",
        "b2c3d4e5-f6a7-5b6c-7d8e-9f0a1b2c3d4e"
      ],
      "created_at": "2025-10-30T12:34:57.100Z",
      "association_completed_at": "2025-10-30T12:35:02.250Z",
      "kyc_completed_at": "2025-10-30T12:35:08.400Z",
      "airdrop_completed_at": "2025-10-30T12:35:15.600Z"
    },
    {
      "cradle_account_id": "33333333-3333-3333-3333-333333333333",
      "linked_account_id": "test-account-1698681298-def67890",
      ...
    }
  ]
}
```

### JSON Field Descriptions

#### Root Level

- `batch_id` (UUID): Unique identifier for this batch
- `created_at` (ISO8601 DateTime): When the batch generation started
- `completed_at` (ISO8601 DateTime): When the batch generation completed
- `config` (object): The configuration used for generation
- `stats` (object): Statistics about the batch results
- `accounts` (array): All generated account records

#### Config Object

Mirrors the command-line configuration that was used:
- `batch_size`: Number of accounts requested
- `account_type`: "retail" or "institutional"
- `assets_to_associate`: Array of asset UUIDs to associate
- `apply_kyc`: Whether KYC was granted
- `output_file`: Where this file was saved
- `initial_status`: Initial account status
- `retry_limit`: Retry configuration
- `retry_delay_ms`: Base retry delay

#### Stats Object

- `total_requested`: Total accounts requested
- `successfully_created`: Number of successfully created accounts
- `failed_count`: Number of failed account creations
- `total_associations`: Total asset associations attempted
- `successful_associations`: Number of successful associations
- `total_kyc_grants`: Total KYC operations attempted
- `successful_kyc_grants`: Number of successful KYC grants

#### Account Object

Per-account details:
- `cradle_account_id`: UUID of the Cradle account in database
- `linked_account_id`: Unique identifier used during creation
- `wallet_id`: UUID of the account's wallet
- `wallet_address`: EVM wallet contract address (0x format)
- `contract_id`: Hedera contract ID format (0.0.xxxxx)
- `account_type`: "retail" or "institutional"
- `status`: Current account status
- `associated_assets`: Array of asset UUIDs that were successfully associated
- `kyc_assets`: Array of asset UUIDs for which KYC was granted
- `airdropped_assets`: Array of asset UUIDs that were airdropped
- `created_at`: When this account was created
- `association_completed_at`: When asset association completed (null if none)
- `kyc_completed_at`: When KYC was completed (null if not applied)
- `airdrop_completed_at`: When airdrops were completed (null if not applied)

## Architecture

### Directory Structure

```
src/simulator/
├── mod.rs                                # Module exports
├── accounts/                             # Accounts submodule
│   ├── mod.rs                           # Exports
│   ├── config.rs (GeneratorConfig)      # Configuration structures
│   ├── models.rs (GeneratedAccount)     # Output data models
│   ├── storage.rs                       # JSON file I/O operations
│   └── generator.rs (AccountGenerator)  # Core batch generation logic
└── shared/                               # Shared utilities
    ├── mod.rs                           # Exports
    └── retry.rs (ExponentialBackoffRetry) # Exponential backoff retry
```

### Key Components

#### GeneratorConfig

Located in `src/simulator/accounts/config.rs`

Holds all configuration parameters with sensible defaults and builder pattern support.

```rust
pub struct GeneratorConfig {
    pub batch_size: u32,
    pub account_type: CradleAccountType,
    pub assets_to_associate: Vec<Uuid>,
    pub apply_kyc: bool,
    pub apply_airdrops: bool,
    pub airdrop_amount: u64,
    pub output_file: PathBuf,
    pub initial_status: CradleAccountStatus,
    pub retry_limit: u32,
    pub retry_delay_ms: u64,
}
```

#### GeneratedAccount & GeneratedBatch

Located in `src/simulator/accounts/models.rs`

Serializable data structures for storing generated accounts and batch metadata.

#### ExponentialBackoffRetry

Located in `src/simulator/shared/retry.rs`

Implements intelligent retry logic with:
- Exponential backoff formula
- Random jitter (±10%)
- Configurable max retries
- 30-second maximum delay cap

#### AccountGenerator

Located in `src/simulator/accounts/generator.rs`

Orchestrates the entire batch generation process:
1. Creates accounts in a loop
2. Automatically associates assets per account
3. Grants KYC for associated assets (if enabled)
4. Airdrops tokens to each account (if enabled)
5. Tracks statistics throughout
6. Provides formatted progress output

#### Storage Functions

Located in `src/simulator/accounts/storage.rs`

- `save_batch_to_json()`: Serialize batch to JSON file
- `load_batch_from_json()`: Deserialize from JSON file
- `append_to_existing_batch()`: Extend existing batch files

## Advanced Features

### Exponential Backoff with Jitter

The retry mechanism uses exponential backoff to handle transient failures gracefully:

```
Attempt 1: ~500ms   (base delay ± jitter)
Attempt 2: ~1000ms  (2x base delay ± jitter)
Attempt 3: ~2000ms  (4x base delay ± jitter)
Attempt 4: ~4000ms  (8x base delay ± jitter)
...
Maximum:  ~30000ms  (30 second cap)
```

The jitter (±10% random variance) prevents "thundering herd" problems where all retries happen at the same time.

### Progress Tracking

The generator provides real-time console output showing:
- Current account number and total
- Success/failure indicators
- Asset association progress
- KYC granting progress
- Airdrop progress (if enabled)
- Overall batch duration
- Final statistics with success percentages for all operations

### Graceful Error Handling

If individual operations fail:
- Account creation failures skip that account but continue with the next
- Asset association failures for one asset don't block other assets
- KYC failures log the error but continue with the batch
- Airdrop failures for one asset don't block other airdrops
- Final statistics show all success/failure counts for all operation types

### JSON Persistence

All generated accounts are automatically saved to a JSON file that can be:
- Loaded for validation
- Used in the airdrop phase
- Backed up or shared with other systems
- Processed by external tools

## Troubleshooting

### Issue: "DATABASE_URL must be set" Error

**Cause**: Database connection environment variable not configured.

**Solution**:
```bash
export DATABASE_URL=postgres://user:password@localhost/cradle_db
account_generator --batch-size 10
```

### Issue: "Invalid UUID" Error

**Cause**: Asset UUID in wrong format.

**Solution**: Ensure UUIDs are in standard format: `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`

```bash
# Wrong:
account_generator --assets 12345678

# Correct:
account_generator --assets 12345678-1234-1234-1234-123456789012
```

### Issue: "Unexpected output type from processor"

**Cause**: Processor returned unexpected result type (likely a bug or version mismatch).

**Solution**:
1. Check that the database is accessible
2. Verify the action router is correctly configured
3. Check logs for more detailed error information
4. Try with `--retry-limit 0` to see immediate error

### Issue: Slow Account Creation

**Cause**: High network latency or heavy database load.

**Solution**:
1. Reduce `--batch-size` to process fewer accounts at once
2. Increase `--retry-delay` to give the system more recovery time
3. Run during off-peak hours
4. Monitor database and contract service health

### Issue: "Failed to write JSON file"

**Cause**: Insufficient permissions or invalid output path.

**Solution**:
```bash
# Check directory exists and is writable
mkdir -p ./accounts
account_generator --output ./accounts/batch.json

# Or use absolute path
account_generator --output /tmp/batch.json
```

### Issue: Assets Not Associating

**Cause**: Asset UUIDs don't exist or belong to different account type.

**Solution**:
1. Verify asset UUIDs exist in the database
2. Check that assets are compatible with account type
3. Try with `--no-kyc` to isolate asset association
4. Check database logs for permission/constraint issues

## Integrated Airdrop System

Airdrops are now fully integrated into the account generation pipeline. Rather than needing a separate airdrop phase, accounts are automatically seeded with tokens during batch generation when `--apply-airdrops` is enabled.

### Complete Seeding Workflow

```bash
account_generator \
  --batch-size 1000 \
  --assets uuid1,uuid2,uuid3 \
  --apply-kyc \
  --apply-airdrops \
  --airdrop-amount 1000000 \
  --output seeded_accounts.json
```

This single command:
1. Creates 1000 accounts
2. Associates 3 assets with each (3000 associations)
3. Grants KYC for all assets (3000 KYC grants)
4. Airdrops 1 million of each asset to each account (3000 airdrops)
5. Saves complete batch metadata to JSON

### Data Available After Generation

```json
{
  "wallet_id": "UUID for future reference",
  "wallet_address": "0x... EVM address (now has tokens)",
  "contract_id": "0.0.xxxxx Hedera ID",
  "associated_assets": [all associated assets],
  "kyc_assets": [all KYC'd assets],
  "airdropped_assets": [all airdropped assets with amounts]
}
```

### Usage After Generation

1. All accounts are fully seeded and ready for use
2. The JSON file documents what was completed for each account
3. Verify success rates in the batch statistics
4. Accounts can be immediately used for testing or trading workflows

## Performance Notes

### Typical Performance

- **Account Creation**: ~1-5 seconds per account (depends on network/contract)
- **Asset Association**: ~1-2 seconds per asset per account
- **KYC Granting**: ~1-2 seconds per asset per account
- **Token Airdrop**: ~1-2 seconds per airdrop (per asset per account)

### Performance with Full Pipeline

For 100 accounts with 3 assets each, fully seeded:
- Account creation: ~100-500 seconds
- Asset associations: ~300-600 seconds
- KYC granting: ~300-600 seconds
- Airdrops: ~300-600 seconds
- **Total estimated time**: ~1000-2300 seconds (~17-38 minutes)

### Batch Size Recommendations

- **Small batches** (1-10): Testing and validation
- **Medium batches** (11-100): Development and staging
- **Large batches** (100-10000): Production workloads
- **Massive batches** (10000+): Consider splitting into multiple runs

### Resource Requirements

- **Memory**: ~10-50MB for typical batches (depends on size)
- **CPU**: Single-threaded, minimal CPU usage
- **Network**: Significant network I/O for contract calls
- **Database**: Read/write operations for account persistence

## Support and Documentation

- Main documentation: See `README.md`
- Lending pool CLI: See `src/bin/lending-pool-cli.rs`
- Accounts module: See `src/accounts/`
- Action router: See `src/action_router.rs`

## License and Attribution

Part of the Cradle ecosystem. Built with Rust, Tokio, Diesel, and Hedera SDK.

---

**Last Updated**: 2025-10-31
**Version**: 1.1.0 (Added integrated airdrop system)
**Status**: Ready for production use

### Changelog v1.1.0
- Added `--apply-airdrops` flag for automatic token airdrops
- Added `--airdrop-amount` configuration for custom airdrop amounts
- Integrated airdrops into account generation pipeline (no separate phase needed)
- Updated GeneratorConfig with `apply_airdrops` and `airdrop_amount` fields
- Enhanced GeneratedAccount model with `airdropped_assets` tracking
- Added airdrop statistics to BatchStats
- Updated batch summary output to show airdrop progress and success rates
- Comprehensive documentation and examples for airdrop functionality
