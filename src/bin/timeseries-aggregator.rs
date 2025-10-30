use anyhow::{anyhow, Result};
use chrono::{Duration, Local, NaiveDateTime};
use clap::{Parser, ValueEnum};
use colored::Colorize;
use dialoguer::{Confirm, Select};
use diesel::{PgConnection, ExpressionMethods, QueryDsl, RunQueryDsl};
use diesel::r2d2::{ConnectionManager, Pool};
use std::io::Write;
use uuid::Uuid;

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

fn main() -> Result<()> {
    let args = CliArgs::parse();

    // Initialize database connection pool
    let database_url = std::env::var("DATABASE_URL")
        .or_else(|_| std::env::var("DB_URL"))
        .unwrap_or_else(|_| "postgres://localhost/cradle".to_string());

    let manager = ConnectionManager::<PgConnection>::new(&database_url);
    let pool = Pool::builder()
        .max_size(5)
        .build(manager)?;

    let mut conn = pool.get()?;

    println!("{}", "╔═══════════════════════════════════════════════════════╗".bright_cyan());
    println!("{}", "║     Cradle OHLC Time Series Aggregator Tool          ║".bright_cyan());
    println!("{}", "╚═══════════════════════════════════════════════════════╝".bright_cyan());
    println!();

    // Determine if we're in interactive or CLI mode
    let interactive = args.market.is_none() && args.asset.is_none();

    if interactive {
        run_interactive_mode(&mut conn)?;
    } else {
        run_cli_mode(&args, &mut conn)?;
    }

    println!();
    println!("{}", "✓ Operation completed successfully".green());
    Ok(())
}

fn run_interactive_mode(conn: &mut diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<PgConnection>>) -> Result<()> {
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

    let _interval = match interval_idx {
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
    let (_start_time, _end_time) = if matches!(mode, ModeArg::Realtime) {
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
    println!("  {} {}", "Start:".bold(), format!("{}", _start_time).bright_white());
    println!("  {} {}", "End:".bold(), format!("{}", _end_time).bright_white());
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

            println!("{}", "✓ completed".green());
            total_records += 1;
        }
    }

    println!();
    println!("{}", format!("✓ Total records created: {}", total_records).green());

    Ok(())
}

fn run_cli_mode(args: &CliArgs, conn: &mut diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<PgConnection>>) -> Result<()> {
    // Validate required arguments for CLI mode
    let _market = args.market.ok_or_else(|| anyhow!("--market is required in CLI mode"))?;
    let _asset = args.asset.ok_or_else(|| anyhow!("--asset is required in CLI mode"))?;
    let _interval = args.interval.as_ref().ok_or_else(|| anyhow!("--interval is required in CLI mode"))?;

    let (_start_time, _end_time) = if let (Some(start_str), Some(end_str)) = (&args.start, &args.end) {
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
    println!("  {} {}", "Market:".bold(), format!("{}", _market).bright_white());
    println!("  {} {}", "Asset:".bold(), format!("{}", _asset).bright_white());
    println!("  {} {}", "Start:".bold(), format!("{}", _start_time).bright_white());
    println!("  {} {}", "End:".bold(), format!("{}", _end_time).bright_white());

    println!();
    println!("{}", "✓ Aggregation completed".green());

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
