use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::accounts::db_types::{CradleAccountType, CradleAccountStatus};
use super::config::GeneratorConfig;

/// A single generated account with all associated metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedAccount {
    /// Cradle account ID from database
    pub cradle_account_id: Uuid,

    /// Linked account ID (unique identifier used during creation)
    pub linked_account_id: String,

    /// Wallet ID from database
    pub wallet_id: Uuid,

    /// Wallet contract address (EVM address)
    pub wallet_address: String,

    /// Wallet contract ID (Hedera contract format)
    pub contract_id: String,

    /// Account type (Retail or Institutional)
    pub account_type: CradleAccountType,

    /// Current account status
    pub status: CradleAccountStatus,

    /// Assets successfully associated with this wallet
    pub associated_assets: Vec<Uuid>,

    /// Assets for which KYC was granted
    pub kyc_assets: Vec<Uuid>,

    /// When the account was created
    pub created_at: DateTime<Utc>,

    /// When asset associations were completed (if applicable)
    pub association_completed_at: Option<DateTime<Utc>>,

    /// When KYC was completed (if applicable)
    pub kyc_completed_at: Option<DateTime<Utc>>,
}

impl GeneratedAccount {
    pub fn new(
        cradle_account_id: Uuid,
        linked_account_id: String,
        wallet_id: Uuid,
        wallet_address: String,
        contract_id: String,
        account_type: CradleAccountType,
        status: CradleAccountStatus,
    ) -> Self {
        Self {
            cradle_account_id,
            linked_account_id,
            wallet_id,
            wallet_address,
            contract_id,
            account_type,
            status,
            associated_assets: Vec::new(),
            kyc_assets: Vec::new(),
            created_at: Utc::now(),
            association_completed_at: None,
            kyc_completed_at: None,
        }
    }

    pub fn with_associated_assets(mut self, assets: Vec<Uuid>) -> Self {
        self.associated_assets = assets.clone();
        if !assets.is_empty() {
            self.association_completed_at = Some(Utc::now());
        }
        self
    }

    pub fn with_kyc_assets(mut self, assets: Vec<Uuid>) -> Self {
        self.kyc_assets = assets.clone();
        if !assets.is_empty() {
            self.kyc_completed_at = Some(Utc::now());
        }
        self
    }
}

/// Statistics about a generated batch
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BatchStats {
    /// Total accounts requested
    pub total_requested: u32,

    /// Successfully created accounts
    pub successfully_created: u32,

    /// Failed account creations
    pub failed_count: u32,

    /// Total asset associations attempted
    pub total_associations: u32,

    /// Successful asset associations
    pub successful_associations: u32,

    /// Total KYC grants attempted
    pub total_kyc_grants: u32,

    /// Successful KYC grants
    pub successful_kyc_grants: u32,
}

impl BatchStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn success_rate_accounts(&self) -> f64 {
        if self.total_requested == 0 {
            return 0.0;
        }
        (self.successfully_created as f64 / self.total_requested as f64) * 100.0
    }

    pub fn success_rate_associations(&self) -> f64 {
        if self.total_associations == 0 {
            return 100.0; // No associations attempted
        }
        (self.successful_associations as f64 / self.total_associations as f64) * 100.0
    }

    pub fn success_rate_kyc(&self) -> f64 {
        if self.total_kyc_grants == 0 {
            return 100.0; // No KYC attempted
        }
        (self.successful_kyc_grants as f64 / self.total_kyc_grants as f64) * 100.0
    }
}

/// A complete batch of generated accounts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedBatch {
    /// Unique batch identifier
    pub batch_id: Uuid,

    /// Configuration used for this batch
    pub config: GeneratorConfig,

    /// All generated accounts
    pub accounts: Vec<GeneratedAccount>,

    /// Statistics about the batch
    pub stats: BatchStats,

    /// When the batch was created
    pub created_at: DateTime<Utc>,

    /// When the batch was completed (None if still in progress)
    pub completed_at: Option<DateTime<Utc>>,
}

impl GeneratedBatch {
    pub fn new(config: GeneratorConfig) -> Self {
        Self {
            batch_id: Uuid::new_v4(),
            config,
            accounts: Vec::new(),
            stats: BatchStats::new(),
            created_at: Utc::now(),
            completed_at: None,
        }
    }

    pub fn add_account(&mut self, account: GeneratedAccount) {
        self.accounts.push(account);
    }

    pub fn mark_completed(&mut self) {
        self.completed_at = Some(Utc::now());
    }

    pub fn duration_seconds(&self) -> f64 {
        if let Some(completed) = self.completed_at {
            (completed - self.created_at).num_seconds() as f64
        } else {
            (Utc::now() - self.created_at).num_seconds() as f64
        }
    }
}
