use anyhow::{anyhow, Result};
use chrono::{Duration, Local, NaiveDateTime};
use clap::{Parser, ValueEnum};
use colored::Colorize;
use dialoguer::{Confirm, Select};
use diesel::{PgConnection, ExpressionMethods, QueryDsl, RunQueryDsl, JoinOnDsl, BoolExpressionMethods};
use diesel::r2d2::{ConnectionManager, Pool};
use std::io::Write;
use dotenvy::dotenv;
use uuid::Uuid;
use bigdecimal::BigDecimal;

use cradle_back_end::order_book::db_types::OrderBookTradeRecord;
use cradle_back_end::market_time_series::db_types::{CreateMarketTimeSeriesRecord, TimeSeriesInterval, DataProviderType};
use cradle_back_end::market_time_series::processor_enum::MarketTimeSeriesProcessorInput;
use cradle_back_end::cli_helper::{initialize_app_config, call_action_router, execute_with_retry};
use cradle_back_end::action_router::{ActionRouterInput, ActionRouterOutput};

/// OHLC data structure for a single time bucket
#[derive(Clone, Debug)]
struct OhlcBar {
    pub open: BigDecimal,
    pub high: BigDecimal,
    pub low: BigDecimal,
    pub close: BigDecimal,
    pub volume: BigDecimal,
}

#[derive(Parser, Debug)]
#[command(
    name = "timeseries-aggregator",
    about = "OHLC Time Series Aggregator for Cradle Markets",
    long_about = "Aggregates orderbook trades into OHLC bars with checkpoint/resume support"
)]
struct CliArgs {
    /// Market UUID to aggregate (if not provided, interactive mode)
    #[arg(long)]
    market: Option<Uuid>,

    /// Asset UUID to aggregate (if not provided, interactive mode)
    #[arg(long)]
    asset: Option<Uuid>,

    /// Time interval for aggregation
    #[arg(long, value_enum)]
    interval: Option<IntervalArg>,

    /// Operation mode
    #[arg(long, value_enum, default_value = "backfill")]
    mode: ModeArg,

    /// Time range duration
    #[arg(long, value_enum)]
    duration: Option<DurationArg>,

    /// Start timestamp (overrides duration if provided)
    #[arg(long)]
    start: Option<String>,

    /// End timestamp (optional, defaults to now)
    #[arg(long)]
    end: Option<String>,

    /// Scope: single market/asset, all assets in market, or all markets
    #[arg(long, value_enum, default_value = "single")]
    scope: ScopeArg,

    /// Run in parallel (only for all-scope operations)
    #[arg(long)]
    parallel: bool,

    /// Skip confirmation prompts
    #[arg(long)]
    confirm: bool,
}

#[derive(Clone, Debug, ValueEnum)]
enum ModeArg {
    /// Start fresh backfill (clears checkpoint)
    #[value(name = "backfill")]
    Backfill,
    /// Resume from checkpoint
    #[value(name = "resume")]
    Resume,
    /// Single time window aggregation
    #[value(name = "single")]
    Single,
    /// Continuous realtime aggregation
    #[value(name = "realtime")]
    Realtime,
    /// List available markets and assets
    #[value(name = "list")]
    List,
}

#[derive(Clone, Debug, ValueEnum)]
enum IntervalArg {
    #[value(name = "15secs")]
    FifteenSecs,
    #[value(name = "30secs")]
    ThirtySecs,
    #[value(name = "45secs")]
    FortyFiveSecs,
    #[value(name = "1min")]
    OneMinute,
    #[value(name = "5min")]
    FiveMinutes,
    #[value(name = "15min")]
    FifteenMinutes,
    #[value(name = "30min")]
    ThirtyMinutes,
    #[value(name = "1hr")]
    OneHour,
    #[value(name = "4hr")]
    FourHours,
    #[value(name = "1day")]
    OneDay,
    #[value(name = "1week")]
    OneWeek,
}

#[derive(Clone, Debug, ValueEnum)]
enum DurationArg {
    #[value(name = "24h")]
    OneDayAgo,
    #[value(name = "7d")]
    SevenDaysAgo,
    #[value(name = "30d")]
    ThirtyDaysAgo,
    #[value(name = "90d")]
    NinetyDaysAgo,
    #[value(name = "all")]
    AllTime,
}

#[derive(Clone, Debug, ValueEnum)]
enum ScopeArg {
    /// Single market/asset combination
    #[value(name = "single")]
    Single,
    /// All assets in a specific market
    #[value(name = "market-all")]
    MarketAll,
    /// All markets and all assets
    #[value(name = "all")]
    All,
}

fn duration_arg_to_offset(arg: &DurationArg) -> Duration {
    match arg {
        DurationArg::OneDayAgo => Duration::days(1),
        DurationArg::SevenDaysAgo => Duration::days(7),
        DurationArg::ThirtyDaysAgo => Duration::days(30),
        DurationArg::NinetyDaysAgo => Duration::days(90),
        DurationArg::AllTime => Duration::days(36500), // ~100 years back
    }
}

/// Convert IntervalArg to Duration for time calculations
fn interval_arg_to_duration(arg: &IntervalArg) -> Duration {
    match arg {
        IntervalArg::FifteenSecs => Duration::seconds(15),
        IntervalArg::ThirtySecs => Duration::seconds(30),
        IntervalArg::FortyFiveSecs => Duration::seconds(45),
        IntervalArg::OneMinute => Duration::minutes(1),
        IntervalArg::FiveMinutes => Duration::minutes(5),
        IntervalArg::FifteenMinutes => Duration::minutes(15),
        IntervalArg::ThirtyMinutes => Duration::minutes(30),
        IntervalArg::OneHour => Duration::hours(1),
        IntervalArg::FourHours => Duration::hours(4),
        IntervalArg::OneDay => Duration::days(1),
        IntervalArg::OneWeek => Duration::weeks(1),
    }
}

/// Convert IntervalArg to TimeSeriesInterval enum
fn interval_arg_to_enum(arg: &IntervalArg) -> TimeSeriesInterval {
    match arg {
        IntervalArg::FifteenSecs => TimeSeriesInterval::FifteenSecs,
        IntervalArg::ThirtySecs => TimeSeriesInterval::ThirtySecs,
        IntervalArg::FortyFiveSecs => TimeSeriesInterval::FortyFiveSecs,
        IntervalArg::OneMinute => TimeSeriesInterval::OneMinute,
        IntervalArg::FiveMinutes => TimeSeriesInterval::FiveMinutes,
        IntervalArg::FifteenMinutes => TimeSeriesInterval::FifteenMinutes,
        IntervalArg::ThirtyMinutes => TimeSeriesInterval::ThirtyMinutes,
        IntervalArg::OneHour => TimeSeriesInterval::OneHour,
        IntervalArg::FourHours => TimeSeriesInterval::FourHours,
        IntervalArg::OneDay => TimeSeriesInterval::OneDay,
        IntervalArg::OneWeek => TimeSeriesInterval::OneWeek,
    }
}

/// Calculate OHLC bars from trades grouped by time interval
/// Returns Vec of (start_time, end_time, OhlcBar) tuples
fn calculate_ohlc_bars(
    trades: Vec<OrderBookTradeRecord>,
    start_time: NaiveDateTime,
    _end_time: NaiveDateTime,
    interval: Duration,
) -> Vec<(NaiveDateTime, NaiveDateTime, OhlcBar)> {
    if trades.is_empty() {
        return Vec::new();
    }

    let mut bars = Vec::new();
    let mut current_bucket_start = start_time;
    let mut current_bucket_trades = Vec::new();

    for trade in trades {
        let bucket_start = current_bucket_start;
        let bucket_end = bucket_start + interval;

        // Check if trade falls in current bucket
        if trade.created_at >= bucket_start && trade.created_at < bucket_end {
            current_bucket_trades.push(trade);
        } else {
            // Close current bucket and start new one
            if !current_bucket_trades.is_empty() {
                let bar = aggregate_trades_to_ohlc(&current_bucket_trades);
                bars.push((bucket_start, bucket_end, bar));
            }

            // Move to new bucket containing this trade
            while trade.created_at >= current_bucket_start + interval {
                current_bucket_start = current_bucket_start + interval;
            }
            current_bucket_trades = vec![trade];
        }
    }

    // Don't forget the last bucket
    if !current_bucket_trades.is_empty() {
        let bucket_start = current_bucket_start;
        let bucket_end = bucket_start + interval;
        let bar = aggregate_trades_to_ohlc(&current_bucket_trades);
        bars.push((bucket_start, bucket_end, bar));
    }

    bars
}

/// Aggregate a group of trades into OHLC data
/// Price is calculated as: filled_amount_of_one_side / filled_amount_of_other_side
fn aggregate_trades_to_ohlc(trades: &[OrderBookTradeRecord]) -> OhlcBar {
    if trades.is_empty() {
        return OhlcBar {
            open: BigDecimal::from(0),
            high: BigDecimal::from(0),
            low: BigDecimal::from(0),
            close: BigDecimal::from(0),
            volume: BigDecimal::from(0),
        };
    }

    // Calculate prices for each trade
    let mut prices = Vec::new();
    let mut volume = BigDecimal::from(0);

    for trade in trades {
        // Price = taker_filled / maker_filled (one side's amount / other side's amount)
        let price = if trade.maker_filled_amount != BigDecimal::from(0) {
            &trade.taker_filled_amount / &trade.maker_filled_amount
        } else {
            BigDecimal::from(0)
        };
        prices.push(price);
        volume = volume + &trade.maker_filled_amount + &trade.taker_filled_amount;
    }

    // Open is first trade's price, Close is last trade's price
    let open = prices[0].clone();
    let close = prices[prices.len() - 1].clone();

    // High and Low from all prices
    let high = prices.iter().max().cloned().unwrap_or_else(|| BigDecimal::from(0));
    let low = prices.iter().min().cloned().unwrap_or_else(|| BigDecimal::from(0));

    OhlcBar {
        open,
        high,
        low,
        close,
        volume,
    }
}

/// Query OrderBookTrades for a specific market/asset within time range
fn query_trades_for_market_asset(
    conn: &mut diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<PgConnection>>,
    market_id: Uuid,
    _asset_id: Uuid,
    start_time: NaiveDateTime,
    end_time: NaiveDateTime,
) -> Result<Vec<OrderBookTradeRecord>> {
    use cradle_back_end::schema::orderbooktrades;
    use cradle_back_end::schema::orderbook;

    // Query trades for the given time range
    // Note: OrderBookTrades doesn't directly reference market_id, we need to join with OrderBook
    let trades = orderbooktrades::table
        .inner_join(orderbook::table.on(
            orderbooktrades::maker_order_id.eq(orderbook::id)
        ))
        .filter(
            orderbook::market_id.eq(market_id)
                .and(orderbooktrades::created_at.ge(start_time))
                .and(orderbooktrades::created_at.le(end_time))
        )
        .select(orderbooktrades::all_columns)
        .order_by(orderbooktrades::created_at.asc())
        .load::<OrderBookTradeRecord>(conn)?;

    Ok(trades)
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    // Force unbuffered stdout for immediate output
    eprintln!("{}", "╔═══════════════════════════════════════════════════════╗".bright_cyan());
    eprintln!("{}", "║     Cradle OHLC Time Series Aggregator Tool          ║".bright_cyan());
    eprintln!("{}", "╚═══════════════════════════════════════════════════════╝".bright_cyan());
    eprintln!();

    let args = CliArgs::parse();

    // Initialize database connection pool
    eprint!("Connecting to database... ");
    std::io::stderr().flush()?;

    let database_url = std::env::var("DATABASE_URL")
        .or_else(|_| std::env::var("DB_URL"))
        .unwrap_or_else(|_| {
            eprintln!("{}", "⚠ DATABASE_URL not set, using default".yellow());
            "postgres://localhost/cradle".to_string()
        });

    eprintln!("Using database: {}", database_url.dimmed());

    match std::time::Instant::now().elapsed().as_secs() {
        _ => {}
    }

    let manager = ConnectionManager::<PgConnection>::new(&database_url);
    let pool = match Pool::builder()
        .max_size(5)
        .build(manager)
    {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{}", format!("✗ Failed to create connection pool: {}", e).red());
            eprintln!("{}", "Check that:".yellow());
            eprintln!("  1. PostgreSQL server is running");
            eprintln!("  2. DATABASE_URL is set correctly");
            eprintln!("  3. Database user exists and has correct permissions");
            return Err(e.into());
        }
    };

    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", format!("✗ Failed to get database connection: {}", e).red());
            eprintln!("{}", "The database server may not be responding.".yellow());
            return Err(e.into());
        }
    };

    eprintln!("{}", "✓ Connected".green());
    eprintln!();

    // Initialize app config for database writes
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

    // Determine if we're in interactive or CLI mode
    let interactive = args.market.is_none() && args.asset.is_none();

    if interactive {
        run_interactive_mode(&mut conn, &app_config).await?;
    } else {
        run_cli_mode(&args, &mut conn, &app_config).await?;
    }

    eprintln!();
    eprintln!("{}", "✓ Operation completed successfully".green());
    Ok(())
}

async fn run_interactive_mode(
    conn: &mut diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<PgConnection>>,
    app_config: &cradle_back_end::utils::app_config::AppConfig,
) -> Result<()> {
    // Main menu
    let menu_options = vec!["Backfill", "Resume", "Single Run", "Realtime", "List Markets"];
    let selection = Select::new()
        .with_prompt("Select operation")
        .items(&menu_options)
        .default(0)
        .interact()?;

    let mode = match selection {
        0 => ModeArg::Backfill,
        1 => ModeArg::Resume,
        2 => ModeArg::Single,
        3 => ModeArg::Realtime,
        4 => ModeArg::List,
        _ => ModeArg::Backfill,
    };

    // Handle List Markets mode separately
    if matches!(mode, ModeArg::List) {
        println!();
        println!("{}", "Available Markets and Assets:".bold().bright_cyan());
        list_markets_and_assets(conn)?;
        return Ok(());
    }

    // Select scope
    let scope_options = vec!["Single Market/Asset", "All Assets in Market", "All Markets and Assets"];
    let scope_selection = Select::new()
        .with_prompt("Select scope")
        .items(&scope_options)
        .default(0)
        .interact()?;

    let scope = match scope_selection {
        0 => ScopeArg::Single,
        1 => ScopeArg::MarketAll,
        2 => ScopeArg::All,
        _ => ScopeArg::Single,
    };

    // Select market(s) and asset(s)
    let markets = get_markets(conn)?;
    if markets.is_empty() {
        println!("{}", "No markets found in database".red());
        return Err(anyhow!("No markets available"));
    }

    let (market_ids, asset_ids) = match scope {
        ScopeArg::Single => {
            let market_names: Vec<String> = markets.iter().map(|(_, name, _)| name.clone()).collect();
            let market_idx = Select::new()
                .with_prompt("Select market")
                .items(&market_names)
                .interact()?;

            let (market_id, _, assets) = &markets[market_idx];

            if assets.is_empty() {
                println!("{}", "Selected market has no assets".red());
                return Err(anyhow!("Market has no assets"));
            }

            let asset_names: Vec<String> = assets.iter().map(|(_, symbol)| symbol.clone()).collect();
            let asset_idx = Select::new()
                .with_prompt("Select asset")
                .items(&asset_names)
                .interact()?;

            let (asset_id, _) = &assets[asset_idx];
            (vec![*market_id], vec![*asset_id])
        }
        ScopeArg::MarketAll => {
            let market_names: Vec<String> = markets.iter().map(|(_, name, _)| name.clone()).collect();
            let market_idx = Select::new()
                .with_prompt("Select market")
                .items(&market_names)
                .interact()?;

            let (market_id, _, assets) = &markets[market_idx];

            if assets.is_empty() {
                println!("{}", "Selected market has no assets".red());
                return Err(anyhow!("Market has no assets"));
            }

            let asset_ids: Vec<Uuid> = assets.iter().map(|(id, _)| *id).collect();

            // Ask about sequential vs parallel
            let process_options = vec!["Sequential", "Parallel"];
            let process_idx = Select::new()
                .with_prompt("Process assets how?")
                .items(&process_options)
                .interact()?;

            if process_idx == 1 {
                println!("{}", "Note: Parallel processing not yet implemented, using sequential".yellow());
            }

            (vec![*market_id], asset_ids)
        }
        ScopeArg::All => {
            let all_market_ids: Vec<Uuid> = markets.iter().map(|(id, _, _)| *id).collect();
            let all_asset_ids: Vec<Uuid> = markets
                .iter()
                .flat_map(|(_, _, assets)| assets.iter().map(|(id, _)| *id))
                .collect();

            // Ask about sequential vs parallel
            let process_options = vec!["Sequential", "Parallel"];
            let process_idx = Select::new()
                .with_prompt("Process markets how?")
                .items(&process_options)
                .interact()?;

            if process_idx == 1 {
                println!("{}", "Note: Parallel processing not yet implemented, using sequential".yellow());
            }

            (all_market_ids, all_asset_ids)
        }
    };

    // Select interval
    let interval_options = vec![
        "15 seconds", "30 seconds", "45 seconds", "1 minute", "5 minutes",
        "15 minutes", "30 minutes", "1 hour", "4 hours", "1 day", "1 week"
    ];
    let interval_idx = Select::new()
        .with_prompt("Select aggregation interval")
        .items(&interval_options)
        .interact()?;

    let interval = match interval_idx {
        0 => IntervalArg::FifteenSecs,
        1 => IntervalArg::ThirtySecs,
        2 => IntervalArg::FortyFiveSecs,
        3 => IntervalArg::OneMinute,
        4 => IntervalArg::FiveMinutes,
        5 => IntervalArg::FifteenMinutes,
        6 => IntervalArg::ThirtyMinutes,
        7 => IntervalArg::OneHour,
        8 => IntervalArg::FourHours,
        9 => IntervalArg::OneDay,
        10 => IntervalArg::OneWeek,
        _ => IntervalArg::FifteenMinutes,
    };

    // Skip duration/time for realtime mode
    let (start_time, end_time) = if matches!(mode, ModeArg::Realtime) {
        println!("{}", "Running in realtime mode - will continuously aggregate new bars".bright_cyan());
        (Local::now().naive_local(), Local::now().naive_local())
    } else {
        // Select duration
        let duration_options = vec!["Last 24 hours", "Last 7 days", "Last 30 days", "Last 90 days", "All time"];
        let duration_idx = Select::new()
            .with_prompt("Select time range")
            .items(&duration_options)
            .interact()?;

        let duration_arg = match duration_idx {
            0 => DurationArg::OneDayAgo,
            1 => DurationArg::SevenDaysAgo,
            2 => DurationArg::ThirtyDaysAgo,
            3 => DurationArg::NinetyDaysAgo,
            4 => DurationArg::AllTime,
            _ => DurationArg::SevenDaysAgo,
        };

        let now = Local::now().naive_local();
        let offset = duration_arg_to_offset(&duration_arg);
        (now - offset, now)
    };

    // Summary and confirmation
    println!();
    println!("{}", "╔═══════════════════════════════════════════════════════╗".bright_cyan());
    println!("{}", "║                    Configuration Summary              ║".bright_cyan());
    println!("{}", "╚═══════════════════════════════════════════════════════╝".bright_cyan());
    println!("  {} {}", "Mode:".bold(), format!("{:?}", mode).bright_white());
    println!("  {} {}", "Markets:".bold(), format!("{}", market_ids.len()).bright_white());
    println!("  {} {}", "Assets:".bold(), format!("{}", asset_ids.len()).bright_white());
    println!("  {} {}", "Start:".bold(), format!("{}", start_time).bright_white());
    println!("  {} {}", "End:".bold(), format!("{}", end_time).bright_white());
    println!();

    if !Confirm::new().with_prompt("Proceed with aggregation?").interact()? {
        println!("{}", "Aggregation cancelled".yellow());
        return Ok(());
    }

    // Execute aggregation
    println!();
    println!("{}", "Starting aggregation...".bright_green());

    let mut total_records = 0;

    for market_id in &market_ids {
        for asset_id in &asset_ids {
            print!("  {} {} ",
                format!("[{}]", market_id).bright_cyan(),
                format!("[{}]", asset_id).bright_cyan()
            );
            std::io::stdout().flush()?;

            // Query trades for this market/asset
            match query_trades_for_market_asset(conn, *market_id, *asset_id, start_time, end_time) {
                Ok(trades) => {
                    if trades.is_empty() {
                        println!("{}", "✓ no trades".dimmed());
                        continue;
                    }

                    // Calculate OHLC bars
                    let interval_duration = interval_arg_to_duration(&interval);
                    let bars = calculate_ohlc_bars(trades, start_time, end_time, interval_duration);

                    let interval_enum = interval_arg_to_enum(&interval);
                    let mut bar_count = 0;

                    // Write each OHLC bar to database via ActionRouter
                    for (bar_start, bar_end, bar) in bars {
                        let bar_clone = bar.clone();
                        let interval_clone = interval_enum.clone();

                        let result = execute_with_retry(
                            || {
                                let app_config = app_config.clone();
                                let bar_data = bar_clone.clone();
                                let interval_data = interval_clone.clone();
                                async move {
                                    let create_input = CreateMarketTimeSeriesRecord {
                                        market_id: *market_id,
                                        asset: *asset_id,
                                        open: bar_data.open.clone(),
                                        high: bar_data.high.clone(),
                                        low: bar_data.low.clone(),
                                        close: bar_data.close.clone(),
                                        volume: bar_data.volume.clone(),
                                        start_time: bar_start,
                                        end_time: bar_end,
                                        interval: Some(interval_data),
                                        data_provider_type: Some(DataProviderType::OrderBook),
                                        data_provider: None,
                                    };

                                    let input = MarketTimeSeriesProcessorInput::AddRecord(create_input);
                                    let router_input = ActionRouterInput::MarketTimeSeries(input);

                                    match call_action_router(router_input, app_config).await? {
                                        ActionRouterOutput::MarketTimeSeries(_) => Ok(()),
                                        _ => Err(anyhow!("Unexpected action router output type")),
                                    }
                                }
                            },
                            "create_ohlc_record",
                        ).await;

                        match result {
                            Ok(_) => {
                                bar_count += 1;
                                total_records += 1;
                            }
                            Err(e) => {
                                eprintln!("Failed to create OHLC record: {}", e);
                            }
                        }
                    }

                    println!("{}", format!("✓ {} bars created", bar_count).green());
                }
                Err(e) => {
                    println!("{}", format!("✗ error: {}", e).red());
                }
            }
        }
    }

    println!();
    println!("{}", format!("✓ Total records created: {}", total_records).green());

    Ok(())
}

async fn run_cli_mode(
    args: &CliArgs,
    conn: &mut diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<PgConnection>>,
    app_config: &cradle_back_end::utils::app_config::AppConfig,
) -> Result<()> {
    // Validate required arguments for CLI mode
    let market = args.market.ok_or_else(|| anyhow!("--market is required in CLI mode"))?;
    let asset = args.asset.ok_or_else(|| anyhow!("--asset is required in CLI mode"))?;
    let interval = args.interval.as_ref().ok_or_else(|| anyhow!("--interval is required in CLI mode"))?;

    let (start_time, end_time) = if let (Some(start_str), Some(end_str)) = (&args.start, &args.end) {
        let start = NaiveDateTime::parse_from_str(start_str, "%Y-%m-%d %H:%M:%S")?;
        let end = NaiveDateTime::parse_from_str(end_str, "%Y-%m-%d %H:%M:%S")?;
        (start, end)
    } else if let Some(duration) = &args.duration {
        let now = Local::now().naive_local();
        let offset = duration_arg_to_offset(duration);
        (now - offset, now)
    } else {
        return Err(anyhow!("Either --duration or both --start and --end must be provided"));
    };

    println!();
    println!("{}", "Executing aggregation...".bright_green());
    println!("  {} {}", "Market:".bold(), format!("{}", market).bright_white());
    println!("  {} {}", "Asset:".bold(), format!("{}", asset).bright_white());
    println!("  {} {}", "Start:".bold(), format!("{}", start_time).bright_white());
    println!("  {} {}", "End:".bold(), format!("{}", end_time).bright_white());

    println!();

    // Query trades for this market/asset
    match query_trades_for_market_asset(conn, market, asset, start_time, end_time) {
        Ok(trades) => {
            if trades.is_empty() {
                println!("{}", "No trades found for the specified time range".yellow());
                return Ok(());
            }

            // Calculate OHLC bars
            let interval_duration = interval_arg_to_duration(&interval);
            let bars = calculate_ohlc_bars(trades, start_time, end_time, interval_duration);

            let interval_enum = interval_arg_to_enum(&interval);
            let mut bar_count = 0;

            println!("Creating {} OHLC bars...", bars.len());

            // Write each OHLC bar to database via ActionRouter
            for (bar_start, bar_end, bar) in bars {
                let bar_clone = bar.clone();
                let interval_clone = interval_enum.clone();

                let result = execute_with_retry(
                    || {
                        let app_config = app_config.clone();
                        let bar_data = bar_clone.clone();
                        let interval_data = interval_clone.clone();
                        async move {
                            let create_input = CreateMarketTimeSeriesRecord {
                                market_id: market,
                                asset,
                                open: bar_data.open.clone(),
                                high: bar_data.high.clone(),
                                low: bar_data.low.clone(),
                                close: bar_data.close.clone(),
                                volume: bar_data.volume.clone(),
                                start_time: bar_start,
                                end_time: bar_end,
                                interval: Some(interval_data),
                                data_provider_type: Some(DataProviderType::OrderBook),
                                data_provider: None,
                            };

                            let input = MarketTimeSeriesProcessorInput::AddRecord(create_input);
                            let router_input = ActionRouterInput::MarketTimeSeries(input);

                            match call_action_router(router_input, app_config).await? {
                                ActionRouterOutput::MarketTimeSeries(_) => Ok(()),
                                _ => Err(anyhow!("Unexpected action router output type")),
                            }
                        }
                    },
                    "create_ohlc_record",
                ).await;

                match result {
                    Ok(_) => {
                        bar_count += 1;
                    }
                    Err(e) => {
                        eprintln!("Failed to create OHLC record: {}", e);
                    }
                }
            }

            println!("{}", format!("✓ Created {} OHLC bars", bar_count).green());
        }
        Err(e) => {
            println!("{}", format!("✗ Error querying trades: {}", e).red());
            return Err(e);
        }
    }

    Ok(())
}

fn get_markets(conn: &mut diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<PgConnection>>) -> Result<Vec<(Uuid, String, Vec<(Uuid, String)>)>> {
    use diesel::prelude::*;

    #[derive(Queryable)]
    struct Market {
        id: Uuid,
        name: String,
        asset_one: Uuid,
        asset_two: Uuid,
    }

    #[derive(Queryable)]
    struct Asset {
        id: Uuid,
        symbol: String,
    }

    // Get all markets
    let markets: Vec<Market> = {
        use cradle_back_end::schema::markets::dsl::*;
        markets
            .select((id, name, asset_one, asset_two))
            .load(conn)?
    };

    let mut result = Vec::new();

    for market in markets {
        let mut assets = Vec::new();

        // Get asset_one
        let asset_one: Asset = {
            use cradle_back_end::schema::asset_book::dsl::*;
            asset_book
                .filter(id.eq(market.asset_one))
                .select((id, symbol))
                .first(conn)?
        };
        assets.push((asset_one.id, asset_one.symbol));

        // Get asset_two
        let asset_two: Asset = {
            use cradle_back_end::schema::asset_book::dsl::*;
            asset_book
                .filter(id.eq(market.asset_two))
                .select((id, symbol))
                .first(conn)?
        };
        assets.push((asset_two.id, asset_two.symbol));

        result.push((market.id, market.name, assets));
    }

    Ok(result)
}

fn list_markets_and_assets(conn: &mut diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<PgConnection>>) -> Result<()> {
    let markets = get_markets(conn)?;

    for (market_id, market_name, assets) in markets {
        println!("  {} {}", "Market:".bold().bright_cyan(), market_name.bright_white());
        println!("    {} {}", "UUID:".dimmed(), format!("{}", market_id).dimmed());
        for (asset_id, symbol) in assets {
            println!("    {} {} ({})", "├─".dimmed(), symbol.bright_yellow(), format!("{}", asset_id).dimmed());
        }
        println!();
    }

    Ok(())
}
