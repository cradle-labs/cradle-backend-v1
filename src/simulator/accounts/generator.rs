use anyhow::{anyhow, Result};
use chrono::Utc;
use uuid::Uuid;
use colored::Colorize;
use std::fmt::Write as FmtWrite;

use crate::utils::app_config::AppConfig;
use crate::accounts::db_types::CreateCradleAccount;
use crate::accounts::processor_enums::{
    AccountsProcessorInput, AccountsProcessorOutput,
};
use crate::action_router::ActionRouterInput;
use crate::cli_helper::call_action_router;
use crate::simulator::shared::ExponentialBackoffRetry;

use super::config::GeneratorConfig;
use super::models::{GeneratedAccount, GeneratedBatch};

/// Account generator for batch creation with retry logic
pub struct AccountGenerator {
    config: GeneratorConfig,
    app_config: AppConfig,
}

impl AccountGenerator {
    /// Create a new account generator
    pub fn new(config: GeneratorConfig, app_config: AppConfig) -> Self {
        Self { config, app_config }
    }

    /// Generate a batch of accounts with automatic asset association and KYC
    pub async fn generate_batch(&self) -> Result<GeneratedBatch> {
        let mut batch = GeneratedBatch::new(self.config.clone());
        batch.stats.total_requested = self.config.batch_size;

        if self.config.assets_to_associate.len() > 0 {
            batch.stats.total_associations =
                self.config.batch_size * self.config.assets_to_associate.len() as u32;

            if self.config.apply_kyc {
                batch.stats.total_kyc_grants =
                    self.config.batch_size * self.config.assets_to_associate.len() as u32;
            }
        }

        println!(
            "{}",
            format!(
                "\n╔═══════════════════════════════════════════════════════╗",
            )
            .bright_cyan()
        );
        println!(
            "{}",
            format!(
                "║  Starting Account Generation Batch ({})",
                self.config.batch_size
            )
            .bright_cyan()
        );
        println!(
            "{}",
            format!(
                "╚═══════════════════════════════════════════════════════╝\n"
            )
            .bright_cyan()
        );

        for i in 0..self.config.batch_size {
            eprint!("[{}/{}] Creating account... ", i + 1, self.config.batch_size);
            std::io::Write::flush(&mut std::io::stderr()).ok();

            // Generate unique linked account ID
            let linked_id = format!(
                "test-account-{}-{}",
                Utc::now().timestamp_millis(),
                Uuid::new_v4().to_string()[..8].to_string()
            );

            match self.create_account_with_retry(&linked_id).await {
                Ok(account_output) => {
                    eprintln!("{}", "✓".green());

                    let mut generated_account = GeneratedAccount::new(
                        account_output.id,
                        linked_id.clone(),
                        account_output.wallet_id,
                        "".to_string(), // Will be filled from processor
                        "".to_string(), // Will be filled from processor
                        self.config.account_type.clone(),
                        self.config.initial_status.clone(),
                    );

                    // TODO: Extract wallet address and contract_id from processor output
                    // For now, we'll need to refactor the processor to return these values

                    batch.stats.successfully_created += 1;

                    // Associate assets if configured
                    if !self.config.assets_to_associate.is_empty() {
                        eprint!("  └─ Associating assets... ");
                        std::io::Write::flush(&mut std::io::stderr()).ok();

                        match self
                            .associate_assets_with_retry(account_output.id)
                            .await
                        {
                            Ok(_) => {
                                eprintln!("{}", "✓".green());
                                generated_account.associated_assets =
                                    self.config.assets_to_associate.clone();
                                batch.stats.successful_associations +=
                                    self.config.assets_to_associate.len() as u32;

                                // Grant KYC if configured
                                if self.config.apply_kyc {
                                    eprint!("     └─ Granting KYC... ");
                                    std::io::Write::flush(&mut std::io::stderr()).ok();

                                    match self.grant_kyc_with_retry(account_output.id).await {
                                        Ok(_) => {
                                            eprintln!("{}", "✓".green());
                                            generated_account.kyc_assets =
                                                self.config.assets_to_associate.clone();
                                            batch.stats.successful_kyc_grants +=
                                                self.config.assets_to_associate.len() as u32;
                                        }
                                        Err(e) => {
                                            eprintln!("{} {}", "✗".red(), e);
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("{} {}", "✗".red(), e);
                            }
                        }
                    }

                    batch.add_account(generated_account);
                }
                Err(e) => {
                    eprintln!("{} {}", "✗".red(), e);
                    batch.stats.failed_count += 1;
                }
            }
        }

        batch.mark_completed();

        // Print summary
        self.print_batch_summary(&batch);

        Ok(batch)
    }

    /// Create an account with exponential backoff retry
    async fn create_account_with_retry(&self, linked_id: &str) -> Result<AccountCreateOutput> {
        let mut retry = ExponentialBackoffRetry::new(
            self.config.retry_delay_ms,
            self.config.retry_limit,
        );

        let linked_id = linked_id.to_string();
        let app_config = self.app_config.clone();
        let account_type = self.config.account_type.clone();
        let initial_status = self.config.initial_status.clone();

        retry
            .execute(|| async {
                let input = AccountsProcessorInput::CreateAccount(CreateCradleAccount {
                    linked_account_id: linked_id.clone(),
                    account_type: Some(account_type.clone()),
                    status: Some(initial_status.clone()),
                });

                let router_input = ActionRouterInput::Accounts(input);

                match call_action_router(router_input, app_config.clone()).await {
                    Ok(output) => {
                        use crate::action_router::ActionRouterOutput;
                        use crate::accounts::processor_enums::AccountsProcessorOutput;

                        if let ActionRouterOutput::Accounts(
                            AccountsProcessorOutput::CreateAccount(result),
                        ) = output
                        {
                            Ok(AccountCreateOutput {
                                id: result.id,
                                wallet_id: result.wallet_id,
                            })
                        } else {
                            Err(anyhow!("Unexpected output type from processor"))
                        }
                    }
                    Err(e) => Err(e),
                }
            })
            .await
    }

    /// Associate assets with an account wallet
    async fn associate_assets_with_retry(&self, account_id: Uuid) -> Result<()> {
        let mut retry = ExponentialBackoffRetry::new(
            self.config.retry_delay_ms,
            self.config.retry_limit,
        );

        let app_config = self.app_config.clone();

        retry
            .execute(|| async {
                let input =
                    AccountsProcessorInput::HandleAssociateAssets(account_id);

                let router_input = ActionRouterInput::Accounts(input);

                match call_action_router(router_input, app_config.clone()).await {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e),
                }
            })
            .await
    }

    /// Grant KYC for assets associated with an account
    async fn grant_kyc_with_retry(&self, account_id: Uuid) -> Result<()> {
        let mut retry = ExponentialBackoffRetry::new(
            self.config.retry_delay_ms,
            self.config.retry_limit,
        );

        let app_config = self.app_config.clone();

        retry
            .execute(|| async {

                let handle_associate = AccountsProcessorInput::HandleAssociateAssets(account_id);

                match call_action_router(ActionRouterInput::Accounts(handle_associate), app_config.clone()).await {
                    Ok(_)=>{
                        let input = AccountsProcessorInput::HandleKYCAssets(account_id);

                        let router_input = ActionRouterInput::Accounts(input);

                        match call_action_router(router_input, app_config.clone()).await {
                            Ok(_) => Ok(()),
                            Err(e) => Err(e),
                        }
                    },
                    Err(e)=>Err(e)
                }


            })
            .await
    }

    /// Print a formatted summary of the batch results
    fn print_batch_summary(&self, batch: &GeneratedBatch) {
        let mut summary = String::new();

        writeln!(summary, "{}", "").ok();
        writeln!(
            summary,
            "{}",
            format!("╔═══════════════════════════════════════════════════════╗",)
                .bright_cyan()
        )
        .ok();
        writeln!(
            summary,
            "{}",
            format!("║  Batch Generation Complete                            ║",)
                .bright_cyan()
        )
        .ok();
        writeln!(
            summary,
            "{}",
            format!("╚═══════════════════════════════════════════════════════╝",)
                .bright_cyan()
        )
        .ok();

        writeln!(summary, "{}", "").ok();
        writeln!(summary, "  {} Account Creation", "├─".bright_cyan()).ok();
        writeln!(
            summary,
            "  │  ├─ Requested: {}",
            batch.stats.total_requested
        )
        .ok();
        writeln!(
            summary,
            "  │  ├─ Created: {} {}",
            batch.stats.successfully_created,
            format!("({:.1}%)", batch.stats.success_rate_accounts()).bright_green()
        )
        .ok();
        if batch.stats.failed_count > 0 {
            writeln!(
                summary,
                "  │  └─ Failed: {} {}",
                batch.stats.failed_count,
                "⚠".bright_yellow()
            )
            .ok();
        }

        if batch.stats.total_associations > 0 {
            writeln!(summary, "  {} Asset Association", "├─".bright_cyan()).ok();
            writeln!(
                summary,
                "  │  ├─ Attempted: {}",
                batch.stats.total_associations
            )
            .ok();
            writeln!(
                summary,
                "  │  └─ Successful: {} {}",
                batch.stats.successful_associations,
                format!("({:.1}%)", batch.stats.success_rate_associations())
                    .bright_green()
            )
            .ok();
        }

        if batch.stats.total_kyc_grants > 0 {
            writeln!(summary, "  {} KYC Grants", "├─".bright_cyan()).ok();
            writeln!(
                summary,
                "  │  ├─ Attempted: {}",
                batch.stats.total_kyc_grants
            )
            .ok();
            writeln!(
                summary,
                "  │  └─ Successful: {} {}",
                batch.stats.successful_kyc_grants,
                format!("({:.1}%)", batch.stats.success_rate_kyc())
                    .bright_green()
            )
            .ok();
        }

        writeln!(
            summary,
            "  {} Duration: {:.2}s",
            "└─".bright_cyan(),
            batch.duration_seconds()
        )
        .ok();

        writeln!(summary, "{}", "").ok();
        writeln!(
            summary,
            "  {} Batch ID: {}",
            "ℹ".bright_blue(),
            batch.batch_id
        )
        .ok();
        writeln!(
            summary,
            "  {} Accounts: {}",
            "ℹ".bright_blue(),
            batch.accounts.len()
        )
        .ok();

        println!("{}", summary);
    }
}

/// Internal struct to hold account creation output
struct AccountCreateOutput {
    id: Uuid,
    wallet_id: Uuid,
}
