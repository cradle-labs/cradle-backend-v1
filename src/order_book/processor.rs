use crate::order_book::config::OrderBookConfig;
use crate::order_book::db_types::{FillMode, OrderBookRecord, OrderStatus};
use crate::order_book::operations::{lock_asset, settle_order, update_order_status};
use crate::order_book::processor_enums::{
    OrderBookProcessorInput, OrderBookProcessorOutput, OrderFillResult, OrderFillStatus,
};
use crate::order_book::sql_queries::{get_matching_orders, get_order_fill_trades};
use crate::utils::app_config::AppConfig;
use crate::utils::traits::ActionProcessor;
use anyhow::anyhow;
use bigdecimal::{BigDecimal, ToPrimitive};
use diesel::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use std::env;
use uuid::Uuid;

impl ActionProcessor<OrderBookConfig, OrderBookProcessorOutput> for OrderBookProcessorInput {
    async fn process(
        &self,
        app_config: &mut AppConfig,
        local_config: &mut OrderBookConfig,
        conn: Option<&mut PooledConnection<ConnectionManager<PgConnection>>>,
    ) -> anyhow::Result<OrderBookProcessorOutput> {
        let app_conn = conn.ok_or_else(|| anyhow!("Unable to get conn"))?;
        use crate::schema::orderbook;
        use crate::schema::orderbooktrades;

        let disable_onchain_interactions =
            env::var("DISABLE_ONCHAIN_INTERACTIONS").unwrap_or("false".to_string()) == "true";

        match self {
            OrderBookProcessorInput::PlaceOrder(_args) => {
                // Lock assets in wallet before anything
                let mut args = _args.clone();
                args.ask_amount = args
                    .ask_amount
                    .with_scale_round(0, bigdecimal::RoundingMode::Down);
                args.bid_amount = args
                    .bid_amount
                    .with_scale_round(0, bigdecimal::RoundingMode::Down);

                lock_asset(
                    app_config,
                    app_conn,
                    args.wallet,
                    args.ask_asset,
                    args.ask_amount
                        .to_u64()
                        .ok_or_else(|| anyhow!("Failed to u64"))?,
                )
                .await?;

                let order = diesel::insert_into(orderbook::table)
                    .values(args.clone())
                    .get_result::<OrderBookRecord>(app_conn)?;

                let matching_orders = get_matching_orders(app_conn, order.id).await?;
                let (remaining_bid, unfilled_ask, trades) =
                    get_order_fill_trades(&order, matching_orders);
                println!("Order trades :: {:?}", trades.clone());
                println!(
                    "Remaining Bid {:?}, Unfilled ask {:?}",
                    remaining_bid.clone(),
                    unfilled_ask.clone()
                );
                // Handle FillOrKill
                if let Some(FillMode::FillOrKill) = args.mode
                    && (remaining_bid > BigDecimal::from(0) || unfilled_ask > BigDecimal::from(0))
                {
                    println!("killing order");
                    update_order_status(app_config, app_conn, order.id, OrderStatus::Cancelled)
                        .await?;

                    return Ok(OrderBookProcessorOutput::PlaceOrder(OrderFillResult {
                        id: order.id,
                        status: OrderFillStatus::Cancelled,
                        bid_amount_filled: BigDecimal::from(0),
                        ask_amount_filled: BigDecimal::from(0),
                        matched_trades: Vec::new(),
                    }));
                }

                // Insert trades
                let mut matched_trades: Vec<Uuid> = Vec::new();
                for trade in &trades {
                    let id = diesel::insert_into(orderbooktrades::table)
                        .values(trade)
                        .returning(orderbooktrades::id)
                        .get_result::<Uuid>(app_conn)?;
                    println!("Matched trade :: {:?}", id);
                    matched_trades.push(id);
                }

                println!("about to settle order");
                // Settle orders

                settle_order(&mut app_config.wallet, app_conn, order.id).await?;

                println!("matched and settled trades");

                // Handle ImmediateOrCancel after settlement
                let final_status = if let Some(FillMode::ImmediateOrCancel) = args.mode {
                    if remaining_bid > BigDecimal::from(0) || unfilled_ask > BigDecimal::from(0) {
                        update_order_status(app_config, app_conn, order.id, OrderStatus::Cancelled)
                            .await?;

                        OrderFillStatus::Partial
                    } else {
                        OrderFillStatus::Filled
                    }
                } else if remaining_bid == BigDecimal::from(0)
                    && unfilled_ask == BigDecimal::from(0)
                {
                    OrderFillStatus::Filled
                } else {
                    OrderFillStatus::Partial
                };

                Ok(OrderBookProcessorOutput::PlaceOrder(OrderFillResult {
                    id: order.id,
                    status: final_status,
                    bid_amount_filled: order.bid_amount - remaining_bid,
                    ask_amount_filled: order.ask_amount - unfilled_ask,
                    matched_trades,
                }))
            }
            OrderBookProcessorInput::GetOrder(order_id) => {
                use crate::schema::orderbook::dsl::*;
                let order_record = orderbook
                    .filter(id.eq(*order_id))
                    .get_result::<OrderBookRecord>(app_conn)?;

                Ok(OrderBookProcessorOutput::GetOrder(order_record))
            }
            OrderBookProcessorInput::GetOrders(filter) => {
                let mut query = orderbook::dsl::orderbook.into_boxed();

                if let Some(wallet) = &filter.wallet {
                    query = query.filter(orderbook::dsl::wallet.eq(*wallet));
                }
                if let Some(market_id) = &filter.market_id {
                    query = query.filter(orderbook::dsl::market_id.eq(market_id.clone()));
                }
                if let Some(status) = &filter.status {
                    query = query.filter(orderbook::dsl::status.eq(status.clone()));
                }
                if let Some(order_type) = &filter.order_type {
                    query = query.filter(orderbook::dsl::order_type.eq(order_type.clone()));
                }

                if let Some(mode) = &filter.mode {
                    query = query.filter(orderbook::dsl::mode.eq(mode.clone()));
                }

                let orders = query.get_results::<OrderBookRecord>(app_conn)?;

                Ok(OrderBookProcessorOutput::GetOrders(orders))
            }
        }
    }
}
