/// This file contains a practical integration example showing how to use
/// the orderbook simulator with actual order placement logic.
///
/// NOTE: This is pseudo-code / documentation. Not meant to compile as-is.
/// Adapt the executor closure in SlotProcessor::process_slots to your needs.

#[allow(dead_code)]
mod integration_example {
    use uuid::Uuid;
    use bigdecimal::BigDecimal;
    use anyhow::{Result, anyhow};

    use crate::simulator::{
        action_slot::{SlotScheduler, SlotProcessor, models::ActionSlot},
        action_slot::scheduler::SchedulerConfig,
        budget::storage::BudgetStore,
        state::{SimulationState, StatePersistence},
        market_discipline::MarketDiscipline,
    };

    use crate::action_router::{ActionRouterInput, ActionRouterOutput};
    use crate::order_book::processor_enums::{OrderBookProcessorInput, OrderBookProcessorOutput};

    /// Full simulation workflow example
    pub async fn run_complete_simulation(
        accounts: Vec<Uuid>,
        markets: Vec<Uuid>,
        assets: Vec<Uuid>,
        app_config: &mut crate::utils::app_config::AppConfig,
        conn: Option<&mut diesel::r2d2::PooledConnection<
            diesel::r2d2::ConnectionManager<diesel::PgConnection>,
        >>,
    ) -> Result<()> {
        println!("Starting orderbook simulator...\n");

        // Step 1: Initialize budget store with test data
        let mut budget_store = initialize_budgets(&accounts, &assets)?;

        // Step 2: Generate trading schedule
        let scheduler_config = SchedulerConfig {
            min_amount: BigDecimal::from(100),
            max_amount: BigDecimal::from(1000),
            trades_per_account: 5,
            bid_price_offset: 1.0,
            ask_price_offset: 1.0,
            alternate_sides: true,
        };

        let scheduler = SlotScheduler::new(scheduler_config);
        let slots = scheduler.generate_schedule(&accounts, &markets)?;
        println!("Generated {} slots for {} accounts across {} markets\n",
            slots.len(), accounts.len(), markets.len());

        // Step 3: Create simulation state
        let mut state = SimulationState::new(slots);
        let persistence = StatePersistence::new("./simulator_state")?;

        // Step 4: Create processor with retry settings
        let processor = SlotProcessor::new(500); // 500ms base retry delay

        // Step 5: Process all slots
        let results = processor.process_slots(&mut state.slots, |slot| {
            let budget_store = budget_store.clone();
            let mut app_config = app_config.clone();
            let conn = conn.cloned();

            Box::pin(async move {
                execute_order_slot(slot, &budget_store, &mut app_config, conn).await
            })
        }).await?;

        // Step 6: Update statistics and save
        state.update_stats();
        persistence.save(&state)?;

        // Print results
        print_simulation_results(&state, &budget_store);

        Ok(())
    }

    /// Initialize budgets for all accounts and assets
    fn initialize_budgets(
        accounts: &[Uuid],
        assets: &[Uuid],
    ) -> Result<BudgetStore> {
        let mut budget_store = BudgetStore::new();

        // Set initial budget for each account/asset pair
        for account in accounts {
            for asset in assets {
                budget_store.set_budget(
                    *account,
                    *asset,
                    BigDecimal::from(100000), // $100k per account/asset
                )?;
            }
        }

        println!("Initialized budgets for {} accounts × {} assets", accounts.len(), assets.len());
        Ok(budget_store)
    }

    /// Execute a single order from an action slot
    async fn execute_order_slot(
        slot: &ActionSlot,
        budget_store: &BudgetStore,
        app_config: &mut crate::utils::app_config::AppConfig,
        conn: Option<diesel::r2d2::PooledConnection<
            diesel::r2d2::ConnectionManager<diesel::PgConnection>,
        >>,
    ) -> Result<Uuid> {
        // 1. Validate budget availability
        if !budget_store.has_available(
            slot.account_id,
            slot.action.bid_asset,
            &slot.action.bid_amount,
        ) {
            return Err(anyhow!(
                "Insufficient budget for account {} on asset {}",
                slot.account_id,
                slot.action.bid_asset
            ));
        }

        // 2. Fetch market and validate price discipline
        let market_request = ActionRouterInput::Market(
            crate::market::processor_enums::MarketProcessorInput::GetMarket(
                slot.action.market_id,
            ),
        );

        let market_response = Box::pin(market_request.process(
            app_config.clone(),
        )).await?;

        if let ActionRouterOutput::Market(
            crate::market::processor_enums::MarketProcessorOutput::GetMarket(market),
        ) = market_response
        {
            let discipline = MarketDiscipline::new(market);
            discipline.validate_price(&slot.action.price)?;
        } else {
            return Err(anyhow!("Failed to fetch market {}", slot.action.market_id));
        }

        // 3. Create the order via orderbook processor
        let order_input = crate::order_book::db_types::NewOrderBookRecord {
            wallet: slot.account_id,
            market_id: slot.action.market_id,
            bid_asset: slot.action.bid_asset,
            ask_asset: slot.action.ask_asset,
            bid_amount: slot.action.bid_amount.clone(),
            ask_amount: slot.action.ask_amount.clone(),
            price: slot.action.price.clone(),
            mode: Some(crate::order_book::db_types::FillMode::GoodTillCancel),
            expires_at: None,
            order_type: Some(crate::order_book::db_types::OrderType::Limit),
        };

        let order_request = ActionRouterInput::OrderBook(
            OrderBookProcessorInput::PlaceOrder(order_input),
        );

        let order_response = Box::pin(order_request.process(
            app_config.clone(),
        )).await?;

        // Extract order ID from response
        let order_id = if let ActionRouterOutput::OrderBook(
            OrderBookProcessorOutput::PlaceOrder(result),
        ) = order_response
        {
            result.id
        } else {
            return Err(anyhow!("Invalid response from order placement"));
        };

        println!(
            "Order {} placed by account {} - {} {} @ {}",
            order_id,
            slot.account_id,
            slot.action.bid_amount,
            slot.action.bid_asset,
            slot.action.price
        );

        Ok(order_id)
    }

    /// Resume a previously interrupted simulation
    pub async fn resume_simulation(
        simulation_id: Uuid,
        app_config: &mut crate::utils::app_config::AppConfig,
        conn: Option<&mut diesel::r2d2::PooledConnection<
            diesel::r2d2::ConnectionManager<diesel::PgConnection>,
        >>,
    ) -> Result<()> {
        println!("Resuming simulation {}...\n", simulation_id);

        let persistence = StatePersistence::new("./simulator_state")?;

        // Load saved state
        let mut state = persistence.load(simulation_id)?;
        println!(
            "Loaded simulation at slot {}/{}",
            state.current_slot_index, state.stats.total_slots
        );

        // Reconstruct budget store from current state
        // (In production, might save budget state separately)
        let budget_store = BudgetStore::new();

        // Get remaining slots to process
        let remaining_slots = &mut state.slots[state.current_slot_index..];

        if remaining_slots.is_empty() {
            println!("Simulation already complete!");
            return Ok(());
        }

        // Create processor and continue
        let processor = SlotProcessor::new(500);

        let results = processor.process_slots(remaining_slots, |slot| {
            let budget_store = budget_store.clone();
            let mut app_config = app_config.clone();
            let conn = conn.cloned();

            Box::pin(async move {
                execute_order_slot(slot, &budget_store, &mut app_config, conn).await
            })
        }).await?;

        // Update and save
        state.update_stats();
        persistence.save(&state)?;

        println!("\nSimulation resumed successfully!");
        Ok(())
    }

    /// Handle order settlement (called from orderbook when orders settle)
    pub fn handle_order_settlement(
        maker_account: Uuid,
        taker_account: Uuid,
        maker_asset: Uuid,
        taker_asset: Uuid,
        maker_amount: BigDecimal,
        taker_amount: BigDecimal,
        budget_store: &mut BudgetStore,
    ) -> Result<()> {
        // Spend maker's budget
        budget_store.spend(maker_account, maker_asset, maker_amount)?;

        // Spend taker's budget
        budget_store.spend(taker_account, taker_asset, taker_amount)?;

        println!(
            "Settlement: Maker {} spent {}, Taker {} spent {}",
            maker_account, maker_asset, taker_account, taker_asset
        );

        Ok(())
    }

    /// Handle order cancellation (called when orders are cancelled)
    pub fn handle_order_cancellation(
        account_id: Uuid,
        asset_id: Uuid,
        amount: BigDecimal,
        budget_store: &mut BudgetStore,
    ) -> Result<()> {
        budget_store.unlock(account_id, asset_id, amount)?;
        println!(
            "Cancelled: Account {} unlocked {} of asset {}",
            account_id, amount, asset_id
        );
        Ok(())
    }

    /// Print simulation results and statistics
    fn print_simulation_results(
        state: &SimulationState,
        budget_store: &BudgetStore,
    ) {
        println!("\n{}", "=".repeat(60));
        println!("SIMULATION RESULTS");
        println!("{}", "=".repeat(60));

        // Slot execution statistics
        println!("\nSlot Execution:");
        println!("  Total Slots: {}", state.stats.total_slots);
        println!("  Completed: {} ({:.1}%)",
            state.stats.completed_slots,
            (state.stats.completed_slots as f64 / state.stats.total_slots as f64) * 100.0
        );
        println!("  Failed: {}", state.stats.failed_slots);
        println!("  Skipped: {}", state.stats.skipped_slots);

        // Order statistics
        println!("\nOrders:");
        println!("  Created: {}", state.stats.total_orders_created);
        println!("  Matched: {}", state.stats.total_matches);

        // Budget statistics
        let summary = budget_store.get_summary();
        println!("\nBudgets:");
        println!("  Total Allocated: {}", summary.total_initial);
        println!("  Total Spent: {}", summary.total_spent);
        println!("  Total Available: {}", summary.total_available);
        println!("  Total Locked: {}", summary.total_locked);
        println!("  Utilization: {:.2}%", summary.total_utilization_percent());
        println!("  Depleted Accounts: {}", summary.depleted_count);

        // Timing
        println!("\nTiming:");
        if let Some(duration) = state.started_at.signed_duration_since(state.last_saved_at).to_std().ok() {
            println!("  Duration: {:.2}s", duration.as_secs_f64());
        }

        println!("{}", "=".repeat(60));
    }

    /// Configuration structure for simulation
    #[derive(Debug, Clone)]
    pub struct SimulationConfig {
        pub min_trade_amount: BigDecimal,
        pub max_trade_amount: BigDecimal,
        pub trades_per_account: u32,
        pub retry_base_delay_ms: u64,
        pub max_retries: u32,
        pub save_after_each_slot: bool,
    }

    impl Default for SimulationConfig {
        fn default() -> Self {
            Self {
                min_trade_amount: BigDecimal::from(100),
                max_trade_amount: BigDecimal::from(5000),
                trades_per_account: 10,
                retry_base_delay_ms: 500,
                max_retries: 3,
                save_after_each_slot: true,
            }
        }
    }
}
