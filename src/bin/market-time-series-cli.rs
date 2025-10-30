use anyhow::Result;
use colored::Colorize;
use std::io::Write;
use bigdecimal::BigDecimal;
use std::str::FromStr;

use cradle_back_end::market_time_series::db_types::{CreateMarketTimeSeriesRecord, TimeSeriesInterval, DataProviderType};
use cradle_back_end::market_time_series::processor_enum::MarketTimeSeriesProcessorInput;
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
    eprintln!("{}", "║      Cradle Market Time Series Management CLI         ║".bright_cyan());
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
                Operation::List => list_time_series(&app_config).await?,
                Operation::View => view_time_series(&app_config).await?,
                Operation::Create => create_time_series(&app_config).await?,
                Operation::Update | Operation::Delete => {
                    print_info("Time series records are append-only (no update/delete)");
                }
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

async fn list_time_series(app_config: &cradle_back_end::utils::app_config::AppConfig) -> Result<()> {
    print_header("List Time Series");

    let intervals = vec!["15secs", "30secs", "1min", "5min", "15min", "30min", "1hr", "4hr", "1day", "1week"];
    let _interval_filter = Input::select_from_list("Filter by interval", intervals)?;

    print_info("Time series listing template (query pending)");

    Ok(())
}

async fn view_time_series(app_config: &cradle_back_end::utils::app_config::AppConfig) -> Result<()> {
    print_header("View Time Series");

    let record_id = Input::get_uuid("Enter time series record ID")?;

    print_info("Time series record viewing template (direct query pending)");

    Ok(())
}

async fn create_time_series(app_config: &cradle_back_end::utils::app_config::AppConfig) -> Result<()> {
    print_header("Create Time Series Record");

    let market_id = Input::get_uuid("Market ID")?;
    let asset = Input::get_uuid("Asset ID")?;

    let open_str = Input::get_string("Open price")?;
    let open = BigDecimal::from_str(&open_str)?;

    let high_str = Input::get_string("High price")?;
    let high = BigDecimal::from_str(&high_str)?;

    let low_str = Input::get_string("Low price")?;
    let low = BigDecimal::from_str(&low_str)?;

    let close_str = Input::get_string("Close price")?;
    let close = BigDecimal::from_str(&close_str)?;

    let volume_str = Input::get_string("Volume")?;
    let volume = BigDecimal::from_str(&volume_str)?;

    let intervals = vec!["15secs", "30secs", "1min", "5min", "15min", "30min", "1hr", "4hr", "1day", "1week"];
    let selected_interval = Input::select_from_list("Interval", intervals)?;

    let interval = match selected_interval {
        0 => TimeSeriesInterval::FifteenSecs,
        1 => TimeSeriesInterval::ThirtySecs,
        2 => TimeSeriesInterval::OneMinute,
        3 => TimeSeriesInterval::FiveMinutes,
        4 => TimeSeriesInterval::FifteenMinutes,
        5 => TimeSeriesInterval::ThirtyMinutes,
        6 => TimeSeriesInterval::OneHour,
        7 => TimeSeriesInterval::FourHours,
        8 => TimeSeriesInterval::OneDay,
        9 => TimeSeriesInterval::OneWeek,
        _ => TimeSeriesInterval::OneMinute,
    };

    let providers = vec!["OrderBook", "Exchange", "Aggregated"];
    let selected_provider = Input::select_from_list("Data provider", providers)?;

    let data_provider_type = match selected_provider {
        0 => DataProviderType::OrderBook,
        1 => DataProviderType::Exchange,
        2 => DataProviderType::Aggregated,
        _ => DataProviderType::OrderBook,
    };

    let now = chrono::Local::now().naive_local();

    execute_with_retry(
        || async {
            let create_input = CreateMarketTimeSeriesRecord {
                market_id,
                asset,
                open: open.clone(),
                high: high.clone(),
                low: low.clone(),
                close: close.clone(),
                volume: volume.clone(),
                start_time: now,
                end_time: now,
                interval: Some(interval.clone()),
                data_provider_type: Some(data_provider_type.clone()),
                data_provider: None,
            };

            let input = MarketTimeSeriesProcessorInput::AddRecord(create_input);
            let router_input = ActionRouterInput::MarketTimeSeries(input);

            match call_action_router(router_input, app_config.clone()).await? {
                ActionRouterOutput::MarketTimeSeries(output) => {
                    print_success("Time series record created successfully");
                    Ok(())
                }
                _ => Err(anyhow::anyhow!("Unexpected output type")),
            }
        },
        "create_time_series",
    )
    .await?;

    Ok(())
}
