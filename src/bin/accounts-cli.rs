use anyhow::{anyhow, Result};
use colored::Colorize;
use std::io::Write;
use bigdecimal::ToPrimitive;
use contract_integrator::utils::functions::asset_manager::{AirdropArgs, AssetManagerFunctionInput, AssetManagerFunctionOutput};
use contract_integrator::utils::functions::commons::ContractFunctionProcessor;
use contract_integrator::utils::functions::{ContractCallInput, ContractCallOutput};
use diesel::RunQueryDsl;
use diesel::prelude::*;
use uuid::Uuid;

use cradle_back_end::accounts::db_types::{CradleAccountRecord, CradleAccountStatus, CradleAccountType, CradleWalletAccountRecord, CreateCradleAccount};
use cradle_back_end::accounts::processor_enums::{AccountsProcessorInput, AccountsProcessorOutput, GetAccountInputArgs, UpdateAccountStatusInputArgs, DeleteAccountInputArgs, GrantKYCInputArgs};
use cradle_back_end::cli_utils::{menu::Operation, input::Input, formatting::{format_table, format_record, print_header, print_section}, print_success, print_info, print_error};
use cradle_back_end::cli_helper::{initialize_app_config, call_action_router, execute_with_retry};
use cradle_back_end::action_router::{ActionRouterInput, ActionRouterOutput};
use cradle_back_end::asset_book::db_types::AssetBookRecord;

#[tokio::main]
async fn main() -> Result<()> {
    eprintln!("{}", "╔═══════════════════════════════════════════════════════╗".bright_cyan());
    eprintln!("{}", "║         Cradle Accounts Management CLI               ║".bright_cyan());
    eprintln!("{}", "╚═══════════════════════════════════════════════════════╝".bright_cyan());
    eprintln!();

    // Initialize app config
    eprint!("Initializing app config... ");
    std::io::stderr().flush().ok();

    let app_config = match initialize_app_config() {
        Ok(config) => {
            eprintln!("{}", "✓ Ready".green());
            config
        }
        Err(e) => {
            eprintln!("{}", "✗ Failed".red());
            eprintln!("Error: {}", e);
            return Err(e);
        }
    };

    eprintln!();

    // Main loop
    loop {
        match Operation::select() {
            Ok(op) => match op {
                Operation::List => list_accounts(&app_config).await?,
                Operation::View => view_account(&app_config).await?,
                Operation::Create => create_account(&app_config).await?,
                Operation::Update => update_account(&app_config).await?,
                Operation::Delete => delete_account(&app_config).await?,
                Operation::Other => do_other(&app_config).await?,
                Operation::Cancel => {
                    eprintln!("{}", "Goodbye!".bright_cyan());
                    break;
                }
            },
            Err(e) => {
                eprintln!("{}", format!("Error: {}", e).red());
                break;
            }
        }

        eprintln!();
    }

    Ok(())
}

async fn list_accounts(app_config: &cradle_back_end::utils::app_config::AppConfig) -> Result<()> {
    print_header("List Accounts");

    // Get optional status filter
    let status_opts = vec!["All", "UnVerified", "Verified", "Suspended", "Closed"];
    let selected_status = Input::select_from_list("Filter by status", status_opts)?;

    // For now, return info about listing being templated
    // TODO: Implement GetAccounts when available in processor
    print_info("Account listing template (full implementation pending - GetAccounts processor TODO)");

    Ok(())
}

async fn view_account(app_config: &cradle_back_end::utils::app_config::AppConfig) -> Result<()> {
    print_header("View Account");

    println!("Query by:");
    let query_opts = vec!["Account ID", "Linked Account"];
    let query_type = Input::select_from_list("", query_opts)?;

    let account_query = match query_type {
        0 => {
            let id = Input::get_uuid("Enter account ID")?;
            GetAccountInputArgs::ByID(id)
        }
        1 => {
            let linked = Input::get_string("Enter linked account ID")?;
            GetAccountInputArgs::ByLinkedAccount(linked)
        }
        _ => {
            let id = Input::get_uuid("Enter account ID")?;
            GetAccountInputArgs::ByID(id)
        }
    };

    execute_with_retry(
        || async {
            let input = AccountsProcessorInput::GetAccount(account_query.clone());
            let router_input = ActionRouterInput::Accounts(input);

            match call_action_router(router_input, app_config.clone()).await? {
                ActionRouterOutput::Accounts(AccountsProcessorOutput::GetAccount(record)) => {
                    println!("Data :: {:?}", record);
                    print_success("Account retrieved successfully");
                    Ok(())
                }
                _ => Err(anyhow::anyhow!("Unexpected output type")),
            }
        },
        "view_account",
    )
    .await?;

    Ok(())
}

async fn create_account(app_config: &cradle_back_end::utils::app_config::AppConfig) -> Result<()> {
    print_header("Create Account");

    let linked_id = Input::get_string("Linked Account ID")?;

    // Select account type
    let types = vec!["Retail", "Institutional"];
    let selected_type = Input::select_from_list("Account type", types)?;
    let account_type = match selected_type {
        0 => CradleAccountType::Retail,
        1 => CradleAccountType::Institutional,
        _ => CradleAccountType::Retail,
    };

    // Select status
    let statuses = vec!["UnVerified", "Verified"];
    let selected_status = Input::select_from_list("Initial status", statuses)?;
    let status = match selected_status {
        0 => CradleAccountStatus::UnVerified,
        1 => CradleAccountStatus::Verified,
        _ => CradleAccountStatus::UnVerified,
    };

    execute_with_retry(
        || async {
            let create_input = CreateCradleAccount {
                linked_account_id: linked_id.clone(),
                account_type: Some(account_type.clone()),
                status: Some(status.clone()),
            };

            let input = AccountsProcessorInput::CreateAccount(create_input);
            let router_input = ActionRouterInput::Accounts(input);

            match call_action_router(router_input, app_config.clone()).await? {
                ActionRouterOutput::Accounts(AccountsProcessorOutput::CreateAccount(output)) => {
                    println!("Create account args :: {:?}", output);

                    print_success("Account created successfully");
                    Ok(())
                }
                _ => Err(anyhow::anyhow!("Unexpected output type")),
            }
        },
        "create_account",
    )
    .await?;

    Ok(())
}

async fn update_account(app_config: &cradle_back_end::utils::app_config::AppConfig) -> Result<()> {
    print_header("Update Account");

    let account_id = Input::get_uuid("Enter account ID")?;

    // Select new status
    let statuses = vec!["UnVerified", "Verified", "Suspended", "Closed"];
    let selected_status = Input::select_from_list("New status", statuses)?;
    let new_status = match selected_status {
        0 => CradleAccountStatus::UnVerified,
        1 => CradleAccountStatus::Verified,
        2 => CradleAccountStatus::Suspended,
        3 => CradleAccountStatus::Closed,
        _ => CradleAccountStatus::UnVerified,
    };

    execute_with_retry(
        || async {
            let update_input = UpdateAccountStatusInputArgs {
                cradle_account_id: account_id,
                status: new_status.clone(),
            };

            let input = AccountsProcessorInput::UpdateAccountStatus(update_input);
            let router_input = ActionRouterInput::Accounts(input);

            match call_action_router(router_input, app_config.clone()).await? {
                ActionRouterOutput::Accounts(res) => {

                    print_success(&format!("Account updated successfully"));
                    Ok(())
                }
                _ => Err(anyhow::anyhow!("Unexpected output type")),
            }
        },
        "update_account",
    )
    .await?;

    Ok(())
}

async fn delete_account(app_config: &cradle_back_end::utils::app_config::AppConfig) -> Result<()> {
    print_header("Delete Account");

    println!("Delete by:");
    let delete_opts = vec!["Account ID", "Linked Account"];
    let delete_type = Input::select_from_list("", delete_opts)?;

    let delete_input = match delete_type {
        0 => {
            let id = Input::get_uuid("Enter account ID to delete")?;
            DeleteAccountInputArgs::ById(id)
        }
        1 => {
            let linked = Input::get_string("Enter linked account ID to delete")?;
            DeleteAccountInputArgs::ByLinkedAccount(linked)
        }
        _ => {
            let id = Input::get_uuid("Enter account ID to delete")?;
            DeleteAccountInputArgs::ById(id)
        }
    };

    let confirmed = cradle_back_end::cli_utils::confirm(
        "Are you sure you want to delete this account? This cannot be undone."
    )?;

    if confirmed {
        execute_with_retry(
            || async {
                let input = AccountsProcessorInput::DeleteAccount(delete_input.clone());
                let router_input = ActionRouterInput::Accounts(input);

                match call_action_router(router_input, app_config.clone()).await? {
                    ActionRouterOutput::Accounts(AccountsProcessorOutput::DeleteAccount) => {
                        print_success("Account deleted successfully");
                        Ok(())
                    }
                    _ => Err(anyhow::anyhow!("Unexpected output type")),
                }
            },
            "delete_account",
        )
        .await?;
    } else {
        print_info("Deletion cancelled");
    }

    Ok(())
}


async fn do_other(app_config: &cradle_back_end::utils::app_config::AppConfig) -> Result<()> {

    let action = Input::select_from_list("Choose an Action", vec!["Associate", "Airdrop", "Setup All"])?;

    match action {
        0=>associate_and_kyc(app_config).await,
        1=>airdrop_tokens(app_config).await,
        2=>setup_all_accounts(app_config).await,
        _=>unimplemented!()
    }

}

async fn associate_and_kyc(app_config: &cradle_back_end::utils::app_config::AppConfig)-> Result<()> {
    
    let account_id = Input::get_uuid("Enter account id")?;
    
    let request = ActionRouterInput::Accounts(
        AccountsProcessorInput::HandleAssociateAssets(account_id.clone())
    );
    
    match request.process(app_config.clone()).await {
        Ok(_)=>{
            let request = ActionRouterInput::Accounts(
                AccountsProcessorInput::HandleKYCAssets(account_id.clone())
            );
            print_success("association complete");
            let _ = request.process(app_config.clone()).await?;
            print_success("kyc granted");
            print_info("Done processing token associations and kyc ")
        },
        Err(e)=>{
            print_error(&format!("Failed {}", e));
            return Err(anyhow!(e))
        }
    }
    
    
    Ok(())
}


async fn setup_all_accounts(app_config: &cradle_back_end::utils::app_config::AppConfig) -> Result<()> {
    print_header("Setup All Accounts");

    // Fetch all wallet accounts from database
    let mut conn = app_config.pool.get()?;
    let wallets = cradle_back_end::schema::cradlewalletaccounts::dsl::cradlewalletaccounts
        .get_results::<CradleWalletAccountRecord>(&mut conn)
        .map_err(|e| anyhow!("Failed to fetch wallet accounts: {}", e))?;

    if wallets.is_empty() {
        print_info("No wallet accounts found in the database");
        return Ok(());
    }

    eprintln!();
    print_info(&format!("Found {} wallet accounts to process", wallets.len()));

    // Ask for confirmation
    let confirmed = cradle_back_end::cli_utils::confirm(
        "Continue with setup for all accounts? This may take a while."
    )?;

    if !confirmed {
        print_info("Operation cancelled");
        return Ok(());
    }

    eprintln!();

    // Statistics tracking
    let mut stats = SetupStats::new(wallets.len());

    // Process each account
    for (index, wallet) in wallets.iter().enumerate() {
        let account_num = index + 1;
        let total = wallets.len();

        eprintln!("[{}/{}] Processing account {}... ", account_num, total, wallet.id);
        std::io::stderr().flush().ok();

        // Step 1: Associate Assets
        eprint!("  └─ Associating assets... ");
        std::io::Write::flush(&mut std::io::stderr()).ok();

        match associate_account_with_retry(wallet.id, app_config).await {
            Ok(_) => {
                eprintln!("{}", "✓".green());
                stats.successful_associations += 1;
            }
            Err(e) => {
                eprintln!("{} {}", "✗".red(), e);
                stats.failed_associations += 1;

                // Ask user what to do
                match handle_step_failure("association").await {
                    StepAction::Skip => {
                        print_info(&format!("  Skipped account {}", wallet.id));
                        stats.skipped_accounts += 1;
                        continue;
                    }
                    StepAction::Exit => {
                        print_error("Setup aborted by user");
                        break;
                    }
                }
            }
        }

        // Step 2: Grant KYC
        eprint!("     └─ Granting KYC... ");
        std::io::Write::flush(&mut std::io::stderr()).ok();

        match grant_kyc_account_with_retry(wallet.id, app_config).await {
            Ok(_) => {
                eprintln!("{}", "✓".green());
                stats.successful_kyc += 1;
            }
            Err(e) => {
                eprintln!("{} {}", "✗".red(), e);
                stats.failed_kyc += 1;

                match handle_step_failure("KYC grant").await {
                    StepAction::Skip => {
                        print_info(&format!("  Skipped account {}", wallet.id));
                        stats.skipped_accounts += 1;
                        continue;
                    }
                    StepAction::Exit => {
                        print_error("Setup aborted by user");
                        break;
                    }
                }
            }
        }

        // Step 3: Airdrop Tokens
        eprint!("        └─ Airdropping tokens... ");
        std::io::Write::flush(&mut std::io::stderr()).ok();

        match airdrop_account_with_retry(wallet.id, app_config).await {
            Ok(count) => {
                eprintln!("{}", "✓".green());
                stats.successful_airdrops += count;
            }
            Err(e) => {
                eprintln!("{} {}", "✗".red(), e);
                stats.failed_airdrops += 1;

                match handle_step_failure("airdrop").await {
                    StepAction::Skip => {
                        print_info(&format!("  Skipped account {}", wallet.id));
                        stats.skipped_accounts += 1;
                        continue;
                    }
                    StepAction::Exit => {
                        print_error("Setup aborted by user");
                        break;
                    }
                }
            }
        }

        stats.completed_accounts += 1;
        eprintln!("     {} Account setup complete", "✓".green());
        eprintln!();
    }

    // Print summary
    print_setup_summary(&stats);

    Ok(())
}

/// Helper enum for step-level user actions
#[derive(Debug, Clone, Copy)]
enum StepAction {
    Skip,
    Exit,
}

/// Statistics for setup_all_accounts
struct SetupStats {
    total_accounts: usize,
    completed_accounts: usize,
    skipped_accounts: usize,
    successful_associations: usize,
    failed_associations: usize,
    successful_kyc: usize,
    failed_kyc: usize,
    successful_airdrops: u32,
    failed_airdrops: usize,
}

impl SetupStats {
    fn new(total: usize) -> Self {
        Self {
            total_accounts: total,
            completed_accounts: 0,
            skipped_accounts: 0,
            successful_associations: 0,
            failed_associations: 0,
            successful_kyc: 0,
            failed_kyc: 0,
            successful_airdrops: 0,
            failed_airdrops: 0,
        }
    }

    fn success_rate_associations(&self) -> f64 {
        let total = self.successful_associations + self.failed_associations;
        if total == 0 {
            return 100.0;
        }
        (self.successful_associations as f64 / total as f64) * 100.0
    }

    fn success_rate_kyc(&self) -> f64 {
        let total = self.successful_kyc + self.failed_kyc;
        if total == 0 {
            return 100.0;
        }
        (self.successful_kyc as f64 / total as f64) * 100.0
    }
}

/// Handle failure at each step - ask user to skip or exit
async fn handle_step_failure(step_name: &str) -> StepAction {
    let options = vec!["Skip This Account", "Exit Setup"];
    match Input::select_from_list(&format!("{} failed. What next?", step_name), options) {
        Ok(choice) => {
            match choice {
                0 => StepAction::Skip,
                _ => StepAction::Exit,
            }
        }
        Err(_) => StepAction::Exit, // Default to exit on error
    }
}

/// Associate assets for a single account with retry logic
async fn associate_account_with_retry(
    account_id: Uuid,
    app_config: &cradle_back_end::utils::app_config::AppConfig,
) -> Result<()> {
    let request = ActionRouterInput::Accounts(
        AccountsProcessorInput::HandleAssociateAssets(account_id)
    );

    match request.process(app_config.clone()).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!("Association failed: {}", e)),
    }
}

/// Grant KYC for a single account with retry logic
async fn grant_kyc_account_with_retry(
    account_id: Uuid,
    app_config: &cradle_back_end::utils::app_config::AppConfig,
) -> Result<()> {
    let request = ActionRouterInput::Accounts(
        AccountsProcessorInput::HandleKYCAssets(account_id)
    );

    match request.process(app_config.clone()).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!("KYC grant failed: {}", e)),
    }
}

/// Airdrop tokens for a single account
async fn airdrop_account_with_retry(
    account_id: Uuid,
    app_config: &cradle_back_end::utils::app_config::AppConfig,
) -> Result<u32> {
    let mut conn = app_config.pool.get()?;

    // Fetch wallet
    let wallet = cradle_back_end::schema::cradlewalletaccounts::dsl::cradlewalletaccounts
        .find(account_id)
        .get_result::<CradleWalletAccountRecord>(&mut conn)
        .map_err(|e| anyhow!("Failed to fetch wallet: {}", e))?;

    // Fetch all assets
    let assets = cradle_back_end::schema::asset_book::dsl::asset_book
        .get_results::<AssetBookRecord>(&mut conn)
        .map_err(|e| anyhow!("Failed to fetch assets: {}", e))?;

    if assets.is_empty() {
        return Ok(0); // No assets to airdrop
    }

    let mut successful_airdrops = 0;

    // Airdrop each asset
    for asset in assets {
        let airdrop_input = ContractCallInput::AssetManager(
            AssetManagerFunctionInput::Airdrop(
                AirdropArgs {
                    amount: 1_000_000, // 1 million tokens
                    asset_contract: asset.asset_manager,
                    target: wallet.address.clone(),
                }
            )
        );

        let mut wallet_clone = app_config.wallet.clone();
        match airdrop_input.process(&mut wallet_clone).await {
            Ok(_) => {
                successful_airdrops += 1;
            }
            Err(_e) => {
                // Log but continue with other assets
            }
        }
    }

    Ok(successful_airdrops)
}

/// Print setup summary
fn print_setup_summary(stats: &SetupStats) {
    eprintln!();
    eprintln!("{}", "╔═══════════════════════════════════════════════════════╗".bright_cyan());
    eprintln!("{}", "║  Account Setup Complete                               ║".bright_cyan());
    eprintln!("{}", "╚═══════════════════════════════════════════════════════╝".bright_cyan());
    eprintln!();

    eprintln!("  {} Account Processing", "├─".bright_cyan());
    eprintln!("  │  ├─ Total Accounts: {}", stats.total_accounts);
    eprintln!(
        "  │  ├─ Completed: {} {}",
        stats.completed_accounts,
        if stats.completed_accounts == stats.total_accounts {
            "✓".green().to_string()
        } else {
            format!("(⚠ {} incomplete)", stats.total_accounts - stats.completed_accounts).yellow().to_string()
        }
    );
    if stats.skipped_accounts > 0 {
        eprintln!("  │  └─ Skipped: {} {}", stats.skipped_accounts, "⚠".yellow());
    }

    eprintln!("  {} Asset Associations", "├─".bright_cyan());
    eprintln!("  │  ├─ Successful: {}", stats.successful_associations);
    eprintln!(
        "  │  └─ Failed: {} {}",
        stats.failed_associations,
        format!("({:.1}% success)", stats.success_rate_associations()).bright_green()
    );

    eprintln!("  {} KYC Grants", "├─".bright_cyan());
    eprintln!("  │  ├─ Successful: {}", stats.successful_kyc);
    eprintln!(
        "  │  └─ Failed: {} {}",
        stats.failed_kyc,
        format!("({:.1}% success)", stats.success_rate_kyc()).bright_green()
    );

    eprintln!("  {} Token Airdrops", "└─".bright_cyan());
    eprintln!("  │  ├─ Successful: {}", stats.successful_airdrops);
    eprintln!("  │  └─ Failed: {}", stats.failed_airdrops);

    eprintln!();
}

async fn airdrop_tokens(app_config: &cradle_back_end::utils::app_config::AppConfig)-> Result<()> {
    let mut runs = 0;
    loop {

        if runs > 0 {
            let confirm = Input::get_bool("Continue")?;

            if(!confirm) { break; };
        }


        let mut conn = app_config.pool.get()?;
        let tokens = cradle_back_end::schema::asset_book::dsl::asset_book.get_results::<AssetBookRecord>(&mut conn)?;

        let selection_list: Vec<&str> = tokens.iter().map(|v| v.name.as_str()).collect();

        let wallet_id = Input::get_uuid("Provide wallet ID")?;

        let wallet =  cradle_back_end::schema::cradlewalletaccounts::dsl::cradlewalletaccounts.find(wallet_id).get_result::<CradleWalletAccountRecord>(&mut conn)?;

        let selection = Input::select_from_list("Select an asset to handle", selection_list)?;

        let selected = tokens[selection].clone();

        let amount = Input::get_decimal("Amount to airdrop")?;


        let request = ContractCallInput::AssetManager(
            AssetManagerFunctionInput::Airdrop(
                AirdropArgs{
                    amount: amount.to_u64().unwrap(),
                    asset_contract: selected.asset_manager,
                    target: wallet.address
                }
            )
        );
        let mut wallet = app_config.wallet.clone();

        let res = request.process(&mut wallet).await?;

        if let ContractCallOutput::AssetManager(AssetManagerFunctionOutput::Airdrop(output)) = res {

            println!("Transaction {:?}", output.transaction_id);

            print_success("Completed airdrop");
        }

        runs +=1;
    }


    Ok(())
}