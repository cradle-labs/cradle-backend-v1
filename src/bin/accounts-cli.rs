use anyhow::{anyhow, Result};
use colored::Colorize;
use std::io::Write;
use diesel::RunQueryDsl;
use uuid::Uuid;

use cradle_back_end::accounts::db_types::{CradleAccountRecord, CradleAccountStatus, CradleAccountType, CreateCradleAccount};
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


async fn airdrop_tokens(app_config: &cradle_back_end::utils::app_config::AppConfig)-> Result<()> {

    let mut conn = app_config.pool.get()?;
    let tokens = cradle_back_end::schema::asset_book::dsl::asset_book.get_results::<AssetBookRecord>(&mut conn)?;

    // let selection_list = tokens.iter().map(|v| v.name).collect();

    Ok(())
}