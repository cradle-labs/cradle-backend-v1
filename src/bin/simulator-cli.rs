use anyhow::{Result, anyhow};
use bigdecimal::BigDecimal;
use clap::Parser;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use std::path::PathBuf;
use dotenvy::dotenv;
use uuid::Uuid;

use cradle_back_end::simulator::{
    action_slot::scheduler::SchedulerConfig,
    budget::storage::BudgetStore,
    cli::{discovery, SimulatorRunner},
    state::StatePersistence,
};
use cradle_back_end::cli_helper::call_action_router;
use cradle_back_end::action_router::{ActionRouterInput, ActionRouterOutput};
use cradle_back_end::order_book::db_types::NewOrderBookRecord;
use cradle_back_end::order_book::processor_enums::OrderBookProcessorInput;
use cradle_back_end::order_book::db_types::{OrderType, FillMode};

#[derive(Parser, Debug)]
#[command(name = "Simulator CLI")]
#[command(about = "Run continuous orderbook trading simulations", long_about = None)]
struct Args {
    /// Override trades per account (default: 10)
    #[arg(long, short = 't')]
    trades_per_account: Option<u32>,

    /// Minimum trade amount (default: 100)
    #[arg(long)]
    min_amount: Option<f64>,

    /// Maximum trade amount (default: 5000)
    #[arg(long)]
    max_amount: Option<f64>,

    /// Initial budget per account/asset in tokens (default: 1000000)
    #[arg(long)]
    initial_budget: Option<f64>,

    /// State directory for checkpoints (default: ./simulator_state)
    #[arg(long)]
    state_dir: Option<PathBuf>,

    /// Account prefix filter (default: test-account)
    #[arg(long)]
    account_filter: Option<String>,

    /// Require user prompt between schedules instead of auto-continue
    #[arg(long)]
    no_auto_continue: bool,

    /// Maximum number of schedules to run (default: unlimited)
    #[arg(long)]
    iterations: Option<u32>,

    /// Bid price offset multiplier (default: 1.0)
    #[arg(long)]
    bid_price_offset: Option<f64>,

    /// Ask price offset multiplier (default: 1.0)
    #[arg(long)]
    ask_price_offset: Option<f64>,

    /// Trades per second execution speed (default: 10)
    #[arg(long)]
    trades_per_second: Option<f64>,

    /// Enable interactive CLI prompts during execution
    #[arg(long)]
    interactive: bool,
}

impl Args {
    fn to_scheduler_config(&self) -> SchedulerConfig {
        SchedulerConfig {
            min_amount: BigDecimal::from(self.min_amount.unwrap_or(100.0) as i32),
            max_amount: BigDecimal::from(self.max_amount.unwrap_or(5000.0) as i32),
            trades_per_account: self.trades_per_account.unwrap_or(10),
            bid_price_offset: self.bid_price_offset.unwrap_or(1.0),
            ask_price_offset: self.ask_price_offset.unwrap_or(1.0),
            alternate_sides: true,
            market_distribution: cradle_back_end::simulator::action_slot::scheduler::MarketDistribution::RoundRobin,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let args = Args::parse();

    println!("Orderbook Simulator CLI");
    println!("======================\n");

    // Database setup
    println!("Connecting to database...");
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/cradle".to_string());

    let manager = ConnectionManager::<PgConnection>::new(&database_url);
    let pool = Pool::builder()
        .max_size(5)
        .build(manager)
        .map_err(|e| anyhow!("Failed to create connection pool: {}", e))?;

    let mut conn = pool
        .get()
        .map_err(|e| anyhow!("Failed to get database connection: {}", e))?;

    // Account discovery
    let account_filter = args.account_filter.as_deref().unwrap_or("test-account");
    println!("Discovering accounts matching '{}'...", account_filter);

    let accounts_records = discovery::discover_accounts(&mut conn, account_filter)?;

    if accounts_records.is_empty() {
        return Err(anyhow!(
            "No test accounts found matching prefix '{}'",
            account_filter
        ));
    }

    let account_ids: Vec<Uuid> = accounts_records.iter().map(|a| a.id).collect();
    println!("Found {} test accounts", accounts_records.len());

    // Discover wallets for each account
    println!("Discovering wallets for accounts...");
    let mut account_to_wallet = std::collections::HashMap::new();
    for account_record in &accounts_records {
        match discovery::get_wallet_for_account(&mut conn, account_record.id) {
            Ok(wallet) => {
                account_to_wallet.insert(account_record.id, wallet.id);
            }
            Err(e) => {
                eprintln!("Warning: Failed to find wallet for account {}: {}", account_record.id, e);
            }
        }
    }
    println!("Found {} wallets", account_to_wallet.len());

    // Market discovery
    println!("Discovering markets...");
    let markets_records = discovery::discover_markets(&mut conn)?;

    if markets_records.is_empty() {
        return Err(anyhow!("No markets found"));
    }

    let market_ids: Vec<Uuid> = markets_records.iter().map(|m| m.id).collect();
    println!("Found {} markets", markets_records.len());

    // Build markets info (market_id, asset_one, asset_two)
    let markets_info: Vec<(Uuid, Uuid, Uuid)> = markets_records
        .iter()
        .map(|m| (m.id, m.asset_one, m.asset_two))
        .collect();

    // Budget initialization
    let initial_budget = BigDecimal::from(args.initial_budget.unwrap_or(1_000_000.0) as i64);
    let mut budget_store = BudgetStore::new();

    discovery::initialize_budgets(
        &mut conn,
        &mut budget_store,
        &accounts_records,
        initial_budget,
    )?;

    println!();

    // State persistence
    let state_dir = args.state_dir.as_deref().unwrap_or_else(|| std::path::Path::new("./simulator_state"));
    let persistence = StatePersistence::new(state_dir)?;

    println!("State checkpoints will be saved to: {}",state_dir.display());
    println!();

    // Initialize app config for order placement
    println!("Initializing app config...");
    let app_config = cradle_back_end::cli_helper::initialize_app_config()?;
    println!("✓ App config initialized\n");

    // Create runner
    let auto_continue = !args.no_auto_continue;
    let scheduler_config = args.to_scheduler_config();

    // Print configuration (before moving scheduler_config)
    let trades_per_second = args.trades_per_second.unwrap_or(10.0);
    println!("Configuration:");
    println!("  Trades per account: {}", scheduler_config.trades_per_account);
    println!("  Trade amount range: {} - {}", args.min_amount.unwrap_or(100.0), args.max_amount.unwrap_or(5000.0));
    println!("  Auto-continue: {}", auto_continue);
    println!("  Execution speed: {:.1} trades/second", trades_per_second);
    println!("  Interactive mode: {}", if args.interactive { "ON" } else { "OFF" });
    if let Some(iter) = args.iterations {
        println!("  Max iterations: {}", iter);
    }
    println!();

    let mut runner = SimulatorRunner::new(
        account_ids,
        market_ids,
        markets_info,
        scheduler_config,
        budget_store,
        persistence,
        auto_continue,
        args.iterations,
        trades_per_second,
        args.interactive,
    );

    println!("Starting continuous simulation...");
    println!("Press Ctrl+C to stop\n");

    // Executor for processing action slots - places real orders through the OrderBook processor
    let executor = |slot: &cradle_back_end::simulator::action_slot::models::ActionSlot| -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Uuid>>>> {
        let app_config_clone = app_config.clone();
        let account_to_wallet_clone = account_to_wallet.clone();
        // Clone slot data to avoid lifetime issues with borrowed reference in async

        // Get the wallet ID for this account (cloned before async block)
        let account_id = slot.account_id;
        let wallet = account_to_wallet_clone
            .get(&account_id)
            .copied()
            .unwrap_or_else(Uuid::nil); // Use nil UUID if not found; error will be caught in async

        let market_id = slot.action.market_id;
        let bid_asset = slot.action.bid_asset;
        let ask_asset = slot.action.ask_asset;
        let bid_amount = slot.action.bid_amount.clone();
        let ask_amount = slot.action.ask_amount.clone();
        let price = slot.action.price.clone();

        Box::pin(async move {
            // Check if wallet was found
            if wallet == Uuid::nil() {
                return Err(anyhow!("No wallet found for account {}", account_id));
            }

            // Create order record from action slot
            let new_order = NewOrderBookRecord {
                wallet,
                market_id,
                bid_asset,
                ask_asset,
                bid_amount,
                ask_amount,
                price,
                mode: Some(FillMode::GoodTillCancel),
                expires_at: None,
                order_type: Some(OrderType::Limit),
            };

            // Call the order book processor through the action router
            let input = OrderBookProcessorInput::PlaceOrder(new_order);
            let router_input = ActionRouterInput::OrderBook(input);

            match call_action_router(router_input, app_config_clone).await {
                Ok(ActionRouterOutput::OrderBook(output)) => {
                    // Extract order ID from the PlaceOrder result
                    match output {
                        cradle_back_end::order_book::processor_enums::OrderBookProcessorOutput::PlaceOrder(fill_result) => {
                            Ok(fill_result.id)
                        }
                        _ => Err(anyhow!("Unexpected order book output type"))
                    }
                }
                Ok(_) => Err(anyhow!("Unexpected action router output type")),
                Err(e) => Err(e),
            }
        })
    };

    runner.run(executor).await?;

    println!("\nSimulation complete!");
    Ok(())
}
