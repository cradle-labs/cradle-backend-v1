use anyhow::Result;
use colored::Colorize;
use std::io::Write;

use cradle_back_end::market::db_types::{MarketStatus, MarketType, MarketRegulation, CreateMarket};
use cradle_back_end::market::processor_enums::{
    MarketProcessorInput, UpdateMarketStatusInputArgs,
};
use cradle_back_end::cli_utils::{
    menu::Operation,
    input::Input,
    formatting::{print_header, print_section},
    print_success, print_info,
};
use cradle_back_end::cli_helper::{initialize_app_config, call_action_router, execute_with_retry};
use cradle_back_end::action_router::{ActionRouterInput, ActionRouterOutput};

#[tokio::main]
async fn main() -> Result<()> {
    eprintln!("{}", "╔═══════════════════════════════════════════════════════╗".bright_cyan());
    eprintln!("{}", "║         Cradle Markets Management CLI                 ║".bright_cyan());
    eprintln!("{}", "╚═══════════════════════════════════════════════════════╝".bright_cyan());
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
        match Operation::select() {
            Ok(op) => match op {
                Operation::List => list_markets(&app_config).await?,
                Operation::View => view_market(&app_config).await?,
                Operation::Create => create_market(&app_config).await?,
                Operation::Update => update_market(&app_config).await?,
                Operation::Delete => {
                    print_info("Market deletion not supported");
                }
                Operation::Cancel => {
                    eprintln!("{}", "Goodbye!".bright_cyan());
                    break;
                },
                _=>unimplemented!()
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

async fn list_markets(app_config: &cradle_back_end::utils::app_config::AppConfig) -> Result<()> {
    print_header("List Markets");

    let market_types = vec!["All", "Spot", "Derivative", "Futures"];
    let _type_filter = Input::select_from_list("Filter by type", market_types)?;

    let statuses = vec!["All", "Active", "InActive", "Suspended"];
    let _status_filter = Input::select_from_list("Filter by status", statuses)?;

    print_info("Market listing template (filtered query pending)");

    Ok(())
}

async fn view_market(app_config: &cradle_back_end::utils::app_config::AppConfig) -> Result<()> {
    print_header("View Market");

    let market_id = Input::get_uuid("Enter market ID")?;

    execute_with_retry(
        || async {
            let input = MarketProcessorInput::GetMarket(market_id);
            let router_input = ActionRouterInput::Markets(input);

            match call_action_router(router_input, app_config.clone()).await? {
                ActionRouterOutput::Markets(output) => {
                    print_success("Market retrieved successfully");
                    Ok(())
                }
                _ => Err(anyhow::anyhow!("Unexpected output type")),
            }
        },
        "view_market",
    )
    .await?;

    Ok(())
}

async fn create_market(app_config: &cradle_back_end::utils::app_config::AppConfig) -> Result<()> {
    print_header("Create Market");

    let name = Input::get_string("Market name")?;
    let description = Input::get_optional_string("Description")?;
    let asset_one = Input::get_uuid("Asset One ID")?;
    let asset_two = Input::get_uuid("Asset Two ID")?;

    let types = vec!["Spot", "Derivative", "Futures"];
    let selected_type = Input::select_from_list("Market type", types)?;
    let market_type = match selected_type {
        0 => MarketType::Spot,
        1 => MarketType::Derivative,
        2 => MarketType::Futures,
        _ => MarketType::Spot,
    };

    let regulations = vec!["Regulated", "UnRegulated"];
    let selected_regulation = Input::select_from_list("Regulation", regulations)?;
    let regulation = match selected_regulation {
        0 => MarketRegulation::Regulated,
        1 => MarketRegulation::UnRegulated,
        _ => MarketRegulation::UnRegulated,
    };

    execute_with_retry(
        || async {
            let create_input = CreateMarket {
                name: name.clone(),
                description: description.clone(),
                icon: None,
                asset_one,
                asset_two,
                market_type: Some(market_type.clone()),
                market_status: Some(MarketStatus::Active),
                market_regulation: Some(regulation.clone()),
            };

            let input = MarketProcessorInput::CreateMarket(create_input);
            let router_input = ActionRouterInput::Markets(input);

            match call_action_router(router_input, app_config.clone()).await? {
                ActionRouterOutput::Markets(output) => {
                    print_success("Market created successfully");
                    Ok(())
                }
                _ => Err(anyhow::anyhow!("Unexpected output type")),
            }
        },
        "create_market",
    )
    .await?;

    Ok(())
}

async fn update_market(app_config: &cradle_back_end::utils::app_config::AppConfig) -> Result<()> {
    print_header("Update Market");

    let market_id = Input::get_uuid("Enter market ID")?;

    let statuses = vec!["Active", "InActive", "Suspended"];
    let selected_status = Input::select_from_list("New status", statuses)?;
    let new_status = match selected_status {
        0 => MarketStatus::Active,
        1 => MarketStatus::InActive,
        2 => MarketStatus::Suspended,
        _ => MarketStatus::Active,
    };

    execute_with_retry(
        || async {
            let update_input = UpdateMarketStatusInputArgs {
                market_id,
                status: new_status.clone(),
            };

            let input = MarketProcessorInput::UpdateMarketStatus(update_input);
            let router_input = ActionRouterInput::Markets(input);

            match call_action_router(router_input, app_config.clone()).await? {
                ActionRouterOutput::Markets(output) => {
                    print_success("Market updated successfully");
                    Ok(())
                }
                _ => Err(anyhow::anyhow!("Unexpected output type")),
            }
        },
        "update_market",
    )
    .await?;

    Ok(())
}
