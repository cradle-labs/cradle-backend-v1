use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;
use crate::accounts::db_types::{CradleAccountType, CradleAccountStatus};

/// Configuration for batch account generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorConfig {
    /// Number of accounts to generate
    pub batch_size: u32,

    /// Account type for all generated accounts
    pub account_type: CradleAccountType,

    /// Asset IDs to automatically associate with each account
    pub assets_to_associate: Vec<Uuid>,

    /// Whether to grant KYC for associated assets
    pub apply_kyc: bool,

    /// Whether to airdrop tokens to generated accounts
    pub apply_airdrops: bool,

    /// Amount of each asset to airdrop per account
    pub airdrop_amount: u64,

    /// Output file path for generated accounts
    pub output_file: PathBuf,

    /// Initial status for created accounts
    pub initial_status: CradleAccountStatus,

    /// Maximum retry attempts per operation
    pub retry_limit: u32,

    /// Base delay in milliseconds for exponential backoff
    pub retry_delay_ms: u64,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            batch_size: 10,
            account_type: CradleAccountType::Retail,
            assets_to_associate: Vec::new(),
            apply_kyc: false,
            apply_airdrops: false,
            airdrop_amount: 1_000_000,
            output_file: PathBuf::from("simulated_accounts.json"),
            initial_status: CradleAccountStatus::Unverified,
            retry_limit: 3,
            retry_delay_ms: 500,
        }
    }
}

impl GeneratorConfig {
    /// Create a new generator config with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set batch size
    pub fn with_batch_size(mut self, size: u32) -> Self {
        self.batch_size = size;
        self
    }

    /// Set account type
    pub fn with_account_type(mut self, account_type: CradleAccountType) -> Self {
        self.account_type = account_type;
        self
    }

    /// Set assets to associate
    pub fn with_assets_to_associate(mut self, assets: Vec<Uuid>) -> Self {
        self.assets_to_associate = assets;
        self
    }

    /// Enable KYC granting
    pub fn with_apply_kyc(mut self, apply: bool) -> Self {
        self.apply_kyc = apply;
        self
    }

    /// Enable token airdrops
    pub fn with_apply_airdrops(mut self, apply: bool) -> Self {
        self.apply_airdrops = apply;
        self
    }

    /// Set airdrop amount per asset per account
    pub fn with_airdrop_amount(mut self, amount: u64) -> Self {
        self.airdrop_amount = amount;
        self
    }

    /// Set output file path
    pub fn with_output_file(mut self, path: PathBuf) -> Self {
        self.output_file = path;
        self
    }

    /// Set initial account status
    pub fn with_initial_status(mut self, status: CradleAccountStatus) -> Self {
        self.initial_status = status;
        self
    }

    /// Set retry configuration
    pub fn with_retry(mut self, limit: u32, delay_ms: u64) -> Self {
        self.retry_limit = limit;
        self.retry_delay_ms = delay_ms;
        self
    }
}
