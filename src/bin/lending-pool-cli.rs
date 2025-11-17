use anyhow::Result;
use bigdecimal::BigDecimal;
use colored::Colorize;
use std::io::Write;
use std::str::FromStr;

use cradle_back_end::action_router::{ActionRouterInput, ActionRouterOutput};
use cradle_back_end::cli_helper::{call_action_router, execute_with_retry, initialize_app_config};
use cradle_back_end::cli_utils::{
    formatting::{format_record, format_table, print_header, print_section},
    input::Input,
    menu::Operation,
    print_info, print_success,
};
use cradle_back_end::lending_pool::db_types::CreateLendingPoolRecord;
use cradle_back_end::lending_pool::processor_enums::{
    GetLendingPoolInput, LendingPoolFunctionsInput, LiquidatePositionInputArgs, RepayLoanInputArgs,
    SupplyLiquidityInputArgs, TakeLoanInputArgs, WithdrawLiquidityInputArgs,
};

#[tokio::main]
async fn main() -> Result<()> {
    eprintln!(
        "{}",
        "╔═══════════════════════════════════════════════════════╗".bright_cyan()
    );
    eprintln!(
        "{}",
        "║     Cradle Lending Pool Management CLI                ║".bright_cyan()
    );
    eprintln!(
        "{}",
        "╚═══════════════════════════════════════════════════════╝".bright_cyan()
    );
    eprintln!();

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

    loop {
        print_header("Lending Pool Management");
        let sections = vec![
            "Pools",
            "Loans",
            "Pool Transactions",
            "Snapshots",
            "Liquidity Operations",
            "Exit",
        ];
        match Input::select_from_list("Select section", sections)? {
            0 => pools_menu(&app_config).await?,
            1 => loans_menu(&app_config).await?,
            2 => pool_transactions_menu(&app_config).await?,
            3 => snapshots_menu(&app_config).await?,
            4 => liquidity_operations_menu(&app_config).await?,
            _ => {
                eprintln!("{}", "Goodbye!".bright_cyan());
                break;
            }
        }

        eprintln!();
    }

    Ok(())
}

// ========== POOLS MENU ==========

async fn pools_menu(app_config: &cradle_back_end::utils::app_config::AppConfig) -> Result<()> {
    print_header("Pools Management");

    let options = vec![
        "List All Pools",
        "Get Pool by ID",
        "Get Pool by Name",
        "Get Pool by Address",
        "Create Pool",
        "Back",
    ];
    match Input::select_from_list("Action", options)? {
        0 => list_pools(app_config).await?,
        1 => get_pool_by_id(app_config).await?,
        2 => get_pool_by_name(app_config).await?,
        3 => get_pool_by_address(app_config).await?,
        4 => create_pool(app_config).await?,
        _ => {}
    }

    Ok(())
}

async fn list_pools(app_config: &cradle_back_end::utils::app_config::AppConfig) -> Result<()> {
    print_header("List All Pools");

    // TODO: Implement GetAllPools when available in processor
    print_info("Pool listing (full implementation pending - GetAllPools processor TODO)");

    Ok(())
}

async fn get_pool_by_id(app_config: &cradle_back_end::utils::app_config::AppConfig) -> Result<()> {
    print_header("Get Pool by ID");

    let pool_id = Input::get_uuid("Enter pool ID")?;

    execute_with_retry(
        || async {
            let input = LendingPoolFunctionsInput::GetLendingPool(GetLendingPoolInput::ById(pool_id));
            let router_input = ActionRouterInput::Pool(input);

            match call_action_router(router_input, app_config.clone()).await? {
                ActionRouterOutput::Pool(cradle_back_end::lending_pool::processor_enums::LendingPoolFunctionsOutput::GetLendingPool(pool)) => {
                    println!("Pool Details :: {:?}", pool);
                    print_success("Pool retrieved successfully");
                    Ok(())
                }
                _ => Err(anyhow::anyhow!("Unexpected output type")),
            }
        },
        "get_pool_by_id",
    )
    .await?;

    Ok(())
}

async fn get_pool_by_name(
    app_config: &cradle_back_end::utils::app_config::AppConfig,
) -> Result<()> {
    print_header("Get Pool by Name");

    let pool_name = Input::get_string("Enter pool name")?;

    execute_with_retry(
        || async {
            let input = LendingPoolFunctionsInput::GetLendingPool(GetLendingPoolInput::ByName(pool_name.clone()));
            let router_input = ActionRouterInput::Pool(input);

            match call_action_router(router_input, app_config.clone()).await? {
                ActionRouterOutput::Pool(cradle_back_end::lending_pool::processor_enums::LendingPoolFunctionsOutput::GetLendingPool(pool)) => {
                    println!("Pool Details :: {:?}", pool);
                    print_success("Pool retrieved successfully");
                    Ok(())
                }
                _ => Err(anyhow::anyhow!("Unexpected output type")),
            }
        },
        "get_pool_by_name",
    )
    .await?;

    Ok(())
}

async fn get_pool_by_address(
    app_config: &cradle_back_end::utils::app_config::AppConfig,
) -> Result<()> {
    print_header("Get Pool by Address");

    let pool_address = Input::get_string("Enter pool address")?;

    execute_with_retry(
        || async {
            let input = LendingPoolFunctionsInput::GetLendingPool(GetLendingPoolInput::ByAddress(pool_address.clone()));
            let router_input = ActionRouterInput::Pool(input);

            match call_action_router(router_input, app_config.clone()).await? {
                ActionRouterOutput::Pool(cradle_back_end::lending_pool::processor_enums::LendingPoolFunctionsOutput::GetLendingPool(pool)) => {
                    println!("Pool Details :: {:?}", pool);
                    print_success("Pool retrieved successfully");
                    Ok(())
                }
                _ => Err(anyhow::anyhow!("Unexpected output type")),
            }
        },
        "get_pool_by_address",
    )
    .await?;

    Ok(())
}

async fn create_pool(app_config: &cradle_back_end::utils::app_config::AppConfig) -> Result<()> {
    print_header("Create Pool");

    let pool_address = Input::get_string("Pool address")?;
    let contract_id = Input::get_string("Pool contract ID")?;
    let reserve_asset = Input::get_uuid("Reserve asset ID")?;
    let yield_asset = Input::get_uuid("Yiled Asset ID")?;

    let ltv_str = Input::get_string("Loan to value ratio")?;
    let ltv = BigDecimal::from_str(&ltv_str)?;

    let base_rate_str = Input::get_string("Base rate")?;
    let base_rate = BigDecimal::from_str(&base_rate_str)?;

    let slope1_str = Input::get_string("Slope1")?;
    let slope1 = BigDecimal::from_str(&slope1_str)?;

    let slope2_str = Input::get_string("Slope2")?;
    let slope2 = BigDecimal::from_str(&slope2_str)?;

    let liquidation_threshold_str = Input::get_string("Liquidation threshold")?;
    let liquidation_threshold = BigDecimal::from_str(&liquidation_threshold_str)?;

    let liquidation_discount_str = Input::get_string("Liquidation discount")?;
    let liquidation_discount = BigDecimal::from_str(&liquidation_discount_str)?;

    let reserve_factor_str = Input::get_string("Reserve factor")?;
    let reserve_factor = BigDecimal::from_str(&reserve_factor_str)?;

    let name = Input::get_string("Pool name (optional)")?;
    let title = Input::get_string("Pool title (optional - press Enter to skip)")?;
    let description = Input::get_string("Pool description (optional - press Enter to skip)")?;

    execute_with_retry(
        || async {
            let create_input = CreateLendingPoolRecord {
                pool_address: pool_address.clone(),
                pool_contract_id: contract_id.clone(),
                reserve_asset,
                loan_to_value: ltv.clone(),
                base_rate: base_rate.clone(),
                slope1: slope1.clone(),
                slope2: slope2.clone(),
                liquidation_threshold: liquidation_threshold.clone(),
                liquidation_discount: liquidation_discount.clone(),
                reserve_factor: reserve_factor.clone(),
                name: if name.is_empty() { None } else { Some(name.clone()) },
                title: if title.is_empty() { None } else { Some(title.clone()) },
                description: if description.is_empty() { None } else { Some(description.clone()) },
                yield_asset
            };

            let input = LendingPoolFunctionsInput::CreateLendingPool(create_input);
            let router_input = ActionRouterInput::Pool(input);

            match call_action_router(router_input, app_config.clone()).await? {
                ActionRouterOutput::Pool(cradle_back_end::lending_pool::processor_enums::LendingPoolFunctionsOutput::CreateLendingPool(pool_id)) => {
                    println!("Created pool with ID: {}", pool_id);
                    print_success("Pool created successfully");
                    Ok(())
                }
                _ => Err(anyhow::anyhow!("Unexpected output type")),
            }
        },
        "create_pool",
    )
    .await?;

    Ok(())
}

// ========== LOANS MENU ==========

async fn loans_menu(app_config: &cradle_back_end::utils::app_config::AppConfig) -> Result<()> {
    print_header("Loans Management");

    let options = vec![
        "List Loans",
        "View Loan (TODO)",
        "Create Loan (Borrow)",
        "Back",
    ];
    match Input::select_from_list("Action", options)? {
        0 => list_loans(app_config).await?,
        1 => view_loan(app_config).await?,
        2 => create_loan(app_config).await?,
        _ => {}
    }

    Ok(())
}

async fn list_loans(app_config: &cradle_back_end::utils::app_config::AppConfig) -> Result<()> {
    print_header("List Loans");

    // TODO: Implement GetAllLoans when available in processor
    print_info("Loan listing template (query pending - GetAllLoans processor TODO)");

    Ok(())
}

async fn view_loan(app_config: &cradle_back_end::utils::app_config::AppConfig) -> Result<()> {
    print_header("View Loan");

    // TODO: Implement GetLoan in processor
    print_info("Loan viewing template (query pending - GetLoan processor TODO)");

    Ok(())
}

async fn create_loan(app_config: &cradle_back_end::utils::app_config::AppConfig) -> Result<()> {
    print_header("Create Loan (Borrow)");

    let wallet = Input::get_uuid("Wallet ID")?;
    let pool = Input::get_uuid("Pool ID")?;
    let amount = Input::get_i64("Borrow amount")? as u64;
    let collateral = Input::get_uuid("Collateral asset ID")?;

    execute_with_retry(
        || async {
            let borrow_input = TakeLoanInputArgs {
                wallet,
                pool,
                amount,
                collateral,
            };

            let input = LendingPoolFunctionsInput::BorrowAsset(borrow_input);
            let router_input = ActionRouterInput::Pool(input);

            match call_action_router(router_input, app_config.clone()).await? {
                ActionRouterOutput::Pool(cradle_back_end::lending_pool::processor_enums::LendingPoolFunctionsOutput::BorrowAsset(loan_id)) => {
                    println!("Created loan with ID: {}", loan_id);
                    print_success("Loan created successfully");
                    Ok(())
                }
                _ => Err(anyhow::anyhow!("Unexpected output type")),
            }
        },
        "create_loan",
    )
    .await?;

    Ok(())
}

// ========== POOL TRANSACTIONS MENU ==========

async fn pool_transactions_menu(
    app_config: &cradle_back_end::utils::app_config::AppConfig,
) -> Result<()> {
    print_header("Pool Transactions");

    let options = vec![
        "List Transactions (TODO)",
        "Supply Liquidity",
        "Withdraw Liquidity",
        "Back",
    ];
    match Input::select_from_list("Action", options)? {
        0 => list_transactions(app_config).await?,
        1 => supply_liquidity(app_config).await?,
        2 => withdraw_liquidity(app_config).await?,
        _ => {}
    }

    Ok(())
}

async fn list_transactions(
    app_config: &cradle_back_end::utils::app_config::AppConfig,
) -> Result<()> {
    print_header("List Pool Transactions");

    // TODO: Implement GetPoolTransactions when available in processor
    print_info("Transaction listing (query pending - GetPoolTransactions processor TODO)");

    Ok(())
}

async fn supply_liquidity(
    app_config: &cradle_back_end::utils::app_config::AppConfig,
) -> Result<()> {
    print_header("Supply Liquidity to Pool");

    let wallet = Input::get_uuid("Wallet ID")?;
    let pool = Input::get_uuid("Pool ID")?;
    let amount = Input::get_i64("Supply amount")? as u64;

    execute_with_retry(
        || async {
            let supply_input = SupplyLiquidityInputArgs {
                wallet,
                pool,
                amount,
            };

            let input = LendingPoolFunctionsInput::SupplyLiquidity(supply_input);
            let router_input = ActionRouterInput::Pool(input);

            match call_action_router(router_input, app_config.clone()).await? {
                ActionRouterOutput::Pool(cradle_back_end::lending_pool::processor_enums::LendingPoolFunctionsOutput::SupplyLiquidity(tx_id)) => {
                    println!("Supply transaction ID: {}", tx_id);
                    print_success("Liquidity supplied successfully");
                    Ok(())
                }
                _ => Err(anyhow::anyhow!("Unexpected output type")),
            }
        },
        "supply_liquidity",
    )
    .await?;

    Ok(())
}

async fn withdraw_liquidity(
    app_config: &cradle_back_end::utils::app_config::AppConfig,
) -> Result<()> {
    print_header("Withdraw Liquidity from Pool");

    let wallet = Input::get_uuid("Wallet ID")?;
    let pool = Input::get_uuid("Pool ID")?;
    let amount = Input::get_i64("Withdrawal amount (in yield asset)")? as u64;

    execute_with_retry(
        || async {
            let withdraw_input = WithdrawLiquidityInputArgs {
                wallet,
                pool,
                amount,
            };

            let input = LendingPoolFunctionsInput::WithdrawLiquidity(withdraw_input);
            let router_input = ActionRouterInput::Pool(input);

            match call_action_router(router_input, app_config.clone()).await? {
                ActionRouterOutput::Pool(cradle_back_end::lending_pool::processor_enums::LendingPoolFunctionsOutput::WithdrawLiquidity(tx_id)) => {
                    println!("Withdrawal transaction ID: {}", tx_id);
                    print_success("Liquidity withdrawn successfully");
                    Ok(())
                }
                _ => Err(anyhow::anyhow!("Unexpected output type")),
            }
        },
        "withdraw_liquidity",
    )
    .await?;

    Ok(())
}

// ========== SNAPSHOTS MENU ==========

async fn snapshots_menu(app_config: &cradle_back_end::utils::app_config::AppConfig) -> Result<()> {
    print_header("Pool Snapshots");

    let options = vec![
        "List Snapshots (TODO)",
        "Get Latest Snapshot",
        "Create Snapshot",
        "Back",
    ];
    match Input::select_from_list("Action", options)? {
        0 => list_snapshots(app_config).await?,
        1 => get_snapshot(app_config).await?,
        2 => create_snapshot(app_config).await?,
        _ => {}
    }

    Ok(())
}

async fn list_snapshots(app_config: &cradle_back_end::utils::app_config::AppConfig) -> Result<()> {
    print_header("List Pool Snapshots");

    // TODO: Implement GetAllSnapshots when available in processor
    print_info("Snapshot listing (query pending - GetAllSnapshots processor TODO)");

    Ok(())
}

async fn get_snapshot(app_config: &cradle_back_end::utils::app_config::AppConfig) -> Result<()> {
    print_header("Get Latest Snapshot for Pool");

    let pool_id = Input::get_uuid("Enter pool ID")?;

    execute_with_retry(
        || async {
            let input = LendingPoolFunctionsInput::GetSnapShot(pool_id);
            let router_input = ActionRouterInput::Pool(input);

            match call_action_router(router_input, app_config.clone()).await? {
                ActionRouterOutput::Pool(cradle_back_end::lending_pool::processor_enums::LendingPoolFunctionsOutput::GetSnapShot(snapshot)) => {
                    println!("Snapshot Data :: {:?}", snapshot);
                    print_success("Snapshot retrieved successfully");
                    Ok(())
                }
                _ => Err(anyhow::anyhow!("Unexpected output type")),
            }
        },
        "get_snapshot",
    )
    .await?;

    Ok(())
}

async fn create_snapshot(app_config: &cradle_back_end::utils::app_config::AppConfig) -> Result<()> {
    print_header("Create Pool Snapshot");

    let pool_id = Input::get_uuid("Enter pool ID")?;

    execute_with_retry(
        || async {
            let input = LendingPoolFunctionsInput::CreateSnapShot(pool_id);
            let router_input = ActionRouterInput::Pool(input);

            match call_action_router(router_input, app_config.clone()).await? {
                ActionRouterOutput::Pool(cradle_back_end::lending_pool::processor_enums::LendingPoolFunctionsOutput::CreateSnapShot(snapshot_id)) => {
                    println!("Created snapshot with ID: {}", snapshot_id);
                    print_success("Snapshot created successfully");
                    Ok(())
                }
                _ => Err(anyhow::anyhow!("Unexpected output type")),
            }
        },
        "create_snapshot",
    )
    .await?;

    Ok(())
}

// ========== LIQUIDITY OPERATIONS MENU ==========

async fn liquidity_operations_menu(
    app_config: &cradle_back_end::utils::app_config::AppConfig,
) -> Result<()> {
    print_header("Liquidity Operations");

    let options = vec!["Repay Loan", "Liquidate Position", "Back"];
    match Input::select_from_list("Action", options)? {
        0 => repay_loan(app_config).await?,
        1 => liquidate_position(app_config).await?,
        _ => {}
    }

    Ok(())
}

async fn repay_loan(app_config: &cradle_back_end::utils::app_config::AppConfig) -> Result<()> {
    print_header("Repay Loan");

    let wallet = Input::get_uuid("Wallet ID")?;
    let loan_id = Input::get_uuid("Loan ID")?;
    let amount = Input::get_i64("Repayment amount")? as u64;

    execute_with_retry(
        || async {
            let repay_input = RepayLoanInputArgs {
                wallet,
                loan: loan_id,
                amount,
            };

            let input = LendingPoolFunctionsInput::RepayBorrow(repay_input);
            let router_input = ActionRouterInput::Pool(input);

            match call_action_router(router_input, app_config.clone()).await? {
                ActionRouterOutput::Pool(cradle_back_end::lending_pool::processor_enums::LendingPoolFunctionsOutput::RepayBorrow()) => {
                    print_success("Loan repaid successfully");
                    Ok(())
                }
                _ => Err(anyhow::anyhow!("Unexpected output type")),
            }
        },
        "repay_loan",
    )
    .await?;

    Ok(())
}

async fn liquidate_position(
    app_config: &cradle_back_end::utils::app_config::AppConfig,
) -> Result<()> {
    print_header("Liquidate Position");

    let wallet = Input::get_uuid("Liquidator wallet ID")?;
    let loan_id = Input::get_uuid("Loan ID")?;
    let amount = Input::get_i64("Liquidation amount")? as u64;

    let confirmed = cradle_back_end::cli_utils::confirm(
        "Are you sure you want to liquidate this position? This is a significant operation.",
    )?;

    if confirmed {
        execute_with_retry(
            || async {
                let liquidate_input = LiquidatePositionInputArgs {
                    wallet,
                    loan: loan_id,
                    amount,
                };

                let input = LendingPoolFunctionsInput::LiquidatePosition(liquidate_input);
                let router_input = ActionRouterInput::Pool(input);

                match call_action_router(router_input, app_config.clone()).await? {
                    ActionRouterOutput::Pool(cradle_back_end::lending_pool::processor_enums::LendingPoolFunctionsOutput::LiquidatePosition()) => {
                        print_success("Position liquidated successfully");
                        Ok(())
                    }
                    _ => Err(anyhow::anyhow!("Unexpected output type")),
                }
            },
            "liquidate_position",
        )
        .await?;
    } else {
        print_info("Liquidation cancelled");
    }

    Ok(())
}
