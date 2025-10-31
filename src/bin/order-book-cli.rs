use anyhow::Result;
use colored::Colorize;
use std::io::Write;
use bigdecimal::BigDecimal;
use std::str::FromStr;

use cradle_back_end::order_book::db_types::{NewOrderBookRecord, OrderStatus, OrderType, FillMode};
use cradle_back_end::order_book::processor_enums::{
    OrderBookProcessorInput,
};
use cradle_back_end::cli_utils::{
    menu::Operation,
    input::Input,
    formatting::{print_header},
    print_success, print_info,
};
use cradle_back_end::cli_helper::{initialize_app_config, call_action_router, execute_with_retry};
use cradle_back_end::action_router::{ActionRouterInput, ActionRouterOutput};

#[tokio::main]
async fn main() -> Result<()> {
    eprintln!("{}", "╔═══════════════════════════════════════════════════════╗".bright_cyan());
    eprintln!("{}", "║       Cradle Order Book Management CLI                ║".bright_cyan());
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
                Operation::List => list_orders(&app_config).await?,
                Operation::View => view_order(&app_config).await?,
                Operation::Create => create_order(&app_config).await?,
                Operation::Update => update_order(&app_config).await?,
                Operation::Delete => cancel_order(&app_config).await?,
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

async fn list_orders(app_config: &cradle_back_end::utils::app_config::AppConfig) -> Result<()> {
    print_header("List Orders");

    let statuses = vec!["All", "Open", "Closed", "Cancelled"];
    let _status_filter = Input::select_from_list("Filter by status", statuses)?;

    let types = vec!["All", "Limit", "Market"];
    let _type_filter = Input::select_from_list("Filter by type", types)?;

    print_info("Order listing template (filtered query pending)");

    Ok(())
}

async fn view_order(app_config: &cradle_back_end::utils::app_config::AppConfig) -> Result<()> {
    print_header("View Order");

    let order_id = Input::get_uuid("Enter order ID")?;

    execute_with_retry(
        || async {
            let input = OrderBookProcessorInput::GetOrder(order_id);
            let router_input = ActionRouterInput::OrderBook(input);

            match call_action_router(router_input, app_config.clone()).await? {
                ActionRouterOutput::OrderBook(output) => {
                    print_success("Order retrieved successfully");
                    Ok(())
                }
                _ => Err(anyhow::anyhow!("Unexpected output type")),
            }
        },
        "view_order",
    )
    .await?;

    Ok(())
}

async fn create_order(app_config: &cradle_back_end::utils::app_config::AppConfig) -> Result<()> {
    print_header("Create Order");

    let wallet = Input::get_uuid("Wallet ID")?;
    let market_id = Input::get_uuid("Market ID")?;
    let bid_asset = Input::get_uuid("Bid Asset ID")?;
    let ask_asset = Input::get_uuid("Ask Asset ID")?;

    let price_str = Input::get_string("Price")?;
    let price = BigDecimal::from_str(&price_str)?;

    let bid_amount_str = Input::get_string("Bid Amount")?;
    let bid_amount = BigDecimal::from_str(&bid_amount_str)?;

    let ask_amount_str = Input::get_string("Ask Amount")?;
    let ask_amount = BigDecimal::from_str(&ask_amount_str)?;

    let types = vec!["Limit", "Market"];
    let selected_type = Input::select_from_list("Order type", types)?;
    let order_type = match selected_type {
        0 => OrderType::Limit,
        1 => OrderType::Market,
        _ => OrderType::Limit,
    };

    let modes = vec!["Fill-or-Kill", "Immediate-or-Cancel", "Good-Till-Cancel"];
    let selected_mode = Input::select_from_list("Fill mode", modes)?;
    let fill_mode = match selected_mode {
        0 => FillMode::FillOrKill,
        1 => FillMode::ImmediateOrCancel,
        2 => FillMode::GoodTillCancel,
        _ => FillMode::GoodTillCancel,
    };

    execute_with_retry(
        || async {
            let new_order = NewOrderBookRecord {
                wallet,
                market_id,
                bid_asset,
                ask_asset,
                bid_amount: bid_amount.clone(),
                ask_amount: ask_amount.clone(),
                price: price.clone(),
                mode: Some(fill_mode.clone()),
                expires_at: None,
                order_type: Some(order_type.clone()),
            };

            let input = OrderBookProcessorInput::PlaceOrder(new_order);
            let router_input = ActionRouterInput::OrderBook(input);

            match call_action_router(router_input, app_config.clone()).await? {
                ActionRouterOutput::OrderBook(output) => {
                    print_success("Order placed successfully");
                    Ok(())
                }
                _ => Err(anyhow::anyhow!("Unexpected output type")),
            }
        },
        "create_order",
    )
    .await?;

    Ok(())
}

async fn update_order(app_config: &cradle_back_end::utils::app_config::AppConfig) -> Result<()> {
    print_header("Update Order");

    let order_id = Input::get_uuid("Enter order ID")?;

    let updates = vec!["Settle", "Cancel"];
    let selected_update = Input::select_from_list("Action", updates)?;

    match selected_update {
        0 => settle_order(app_config, order_id).await?,
        1 => cancel_order_by_id(app_config, order_id).await?,
        _ => {}
    }

    Ok(())
}

async fn settle_order(app_config: &cradle_back_end::utils::app_config::AppConfig, order_id: uuid::Uuid) -> Result<()> {
    execute_with_retry(
        || async {
            let input = OrderBookProcessorInput::SettleOrder(order_id);
            let router_input = ActionRouterInput::OrderBook(input);

            match call_action_router(router_input, app_config.clone()).await? {
                ActionRouterOutput::OrderBook(output) => {
                    print_success("Order settled successfully");
                    Ok(())
                }
                _ => Err(anyhow::anyhow!("Unexpected output type")),
            }
        },
        "settle_order",
    )
    .await?;

    Ok(())
}

async fn cancel_order_by_id(app_config: &cradle_back_end::utils::app_config::AppConfig, order_id: uuid::Uuid) -> Result<()> {
    execute_with_retry(
        || async {
            let input = OrderBookProcessorInput::CancelOrder(order_id);
            let router_input = ActionRouterInput::OrderBook(input);

            match call_action_router(router_input, app_config.clone()).await? {
                ActionRouterOutput::OrderBook(output) => {
                    print_success("Order cancelled successfully");
                    Ok(())
                }
                _ => Err(anyhow::anyhow!("Unexpected output type")),
            }
        },
        "cancel_order",
    )
    .await?;

    Ok(())
}

async fn cancel_order(app_config: &cradle_back_end::utils::app_config::AppConfig) -> Result<()> {
    print_header("Cancel Order");

    let order_id = Input::get_uuid("Enter order ID to cancel")?;

    let confirmed = cradle_back_end::cli_utils::confirm(
        &format!("Are you sure you want to cancel order {}?", order_id)
    )?;

    if confirmed {
        cancel_order_by_id(app_config, order_id).await?;
    } else {
        print_info("Cancellation cancelled");
    }

    Ok(())
}
