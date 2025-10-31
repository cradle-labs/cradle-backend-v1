use std::env;
use anyhow::anyhow;
use bigdecimal::{BigDecimal, ToPrimitive};
use chrono::{NaiveDateTime, Utc};
use contract_integrator::utils::functions::{ContractCallInput, ContractCallOutput};
use contract_integrator::utils::functions::cradle_account::{CradleAccountFunctionInput, CradleAccountFunctionOutput, LockAssetArgs, UnLockAssetArgs};
use contract_integrator::utils::functions::orderbook_settler::{OrderBookSettlerFunctionInput, OrderBookSettlerFunctionOutput, SettleOrderInputArgs};
use diesel::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use serde_json::json;
use uuid::Uuid;
use crate::accounts::db_types::CradleWalletAccountRecord;
use crate::action_router::ActionRouterInput;
use crate::asset_book::db_types::AssetBookRecord;
use crate::order_book::config::OrderBookConfig;
use crate::order_book::db_types::{FillMode, OrderBookRecord, OrderStatus};
use crate::order_book::processor_enums::{OrderBookProcessorInput, OrderBookProcessorOutput, OrderFillResult, OrderFillStatus};
use crate::order_book::sql_queries::{get_matching_orders, get_order_fill_trades};
use crate::utils::app_config::AppConfig;
use crate::utils::traits::ActionProcessor;

impl ActionProcessor<OrderBookConfig, OrderBookProcessorOutput> for OrderBookProcessorInput {
    async fn process(&self, app_config: &mut AppConfig, local_config: &mut OrderBookConfig, conn: Option<&mut PooledConnection<ConnectionManager<PgConnection>>>) -> anyhow::Result<OrderBookProcessorOutput> {
        let app_conn = conn.ok_or_else(||anyhow!("Unable to get conn"))?;
        // let io_conn = app_config.get_io()?;
        use crate::schema::orderbook;
        use crate::schema::orderbooktrades;
        use crate::schema::cradlewalletaccounts;
        use crate::schema::asset_book;
        match self {
            OrderBookProcessorInput::PlaceOrder(args) => {
                // Lock assets in wallet before anything

                let disable_onchain_settlement = env::var("DISABLE_ONCHAIN_SETTLEMENT")
                    .unwrap_or_default() == "true";

                if !disable_onchain_settlement {
                    let lock_asset_request = ActionRouterInput::OrderBook(
                        OrderBookProcessorInput::LockWalletAssets(
                            args.wallet.clone(),
                            args.bid_asset.clone(),
                            args.bid_amount.clone()
                        )
                    );

                    let _ = Box::pin(lock_asset_request.process(app_config.clone())).await?;
                }

                let order = diesel::insert_into(orderbook::table)
                    .values(args.clone())
                    .get_result::<OrderBookRecord>(app_conn)?;

                // io_conn.to(format!("realtime_market_orders_{}", args.market_id)).emit("new-order", &order).await.map_err(|e|anyhow!("Broadcast error {}",e))?;

                let matching_orders = get_matching_orders(app_conn, order.id.clone()).await?;
                let (remaining_bid, unfilled_ask, trades) = get_order_fill_trades(&order, matching_orders);

                // Handle FillOrKill
                if let Some(FillMode::FillOrKill) = args.mode {
                    if remaining_bid > BigDecimal::from(0) || unfilled_ask > BigDecimal::from(0) {
                        let cancel_request = ActionRouterInput::OrderBook(
                            OrderBookProcessorInput::CancelOrder(order.id.clone())
                        );
                        let _ = Box::pin(cancel_request.process(app_config.clone())).await?;

                        return Ok(OrderBookProcessorOutput::PlaceOrder(
                            OrderFillResult {
                                id: order.id,
                                status: OrderFillStatus::Cancelled,
                                bid_amount_filled: BigDecimal::from(0),
                                ask_amount_filled: BigDecimal::from(0),
                                matched_trades: Vec::new()
                            }
                        ));
                    }
                }

                // Insert trades
                let mut matched_trades: Vec<Uuid> = Vec::new();
                for trade in &trades {
                    let id = diesel::insert_into(orderbooktrades::table)
                        .values(trade)
                        .returning(orderbooktrades::id)
                        .get_result::<Uuid>(app_conn)?;
                    matched_trades.push(id);
                }

                // Settle orders
                let settle_request = ActionRouterInput::OrderBook(
                    OrderBookProcessorInput::SettleOrder(order.id.clone())
                );
                let _ = Box::pin(settle_request.process(app_config.clone())).await?;

                // Update fill amounts
                let order_fill_update_request = ActionRouterInput::OrderBook(
                    OrderBookProcessorInput::UpdateOrderFill(
                        order.id.clone(),
                        remaining_bid.clone(),
                        unfilled_ask.clone(),
                        trades.clone()
                    )
                );

                let _ = Box::pin(order_fill_update_request.process(app_config.clone())).await?;

                // Handle ImmediateOrCancel after settlement
                let final_status = if let Some(FillMode::ImmediateOrCancel) = args.mode {
                    if remaining_bid > BigDecimal::from(0) || unfilled_ask > BigDecimal::from(0) {
                        let cancel_request = ActionRouterInput::OrderBook(
                            OrderBookProcessorInput::CancelOrder(order.id.clone())
                        );
                        let _ = Box::pin(cancel_request.process(app_config.clone())).await?;
                        OrderFillStatus::Partial
                    } else {
                        OrderFillStatus::Filled
                    }
                } else if remaining_bid == BigDecimal::from(0) && unfilled_ask == BigDecimal::from(0) {
                    OrderFillStatus::Filled
                } else {
                    OrderFillStatus::Partial
                };

                Ok(OrderBookProcessorOutput::PlaceOrder(
                    OrderFillResult {
                        id: order.id,
                        status: final_status,
                        bid_amount_filled: order.bid_amount - remaining_bid,
                        ask_amount_filled: order.ask_amount - unfilled_ask,
                        matched_trades
                    }
                ))
            }
            OrderBookProcessorInput::SettleOrder(order_id) => {
                // TODO: handle onchain settle for all trades matched to the provided order
                let trades = orderbooktrades::dsl::orderbooktrades.filter(
                    orderbooktrades::taker_order_id.eq(order_id).and(
                        orderbooktrades::settlement_status.eq(crate::order_book::db_types::SettlementStatus::Matched)
                    )
                ).get_results::<crate::order_book::db_types::OrderBookTradeRecord>(app_conn)?;


                for trade in trades.clone() {
                    let maker_order = orderbook::dsl::orderbook.filter(
                        orderbook::id.eq(trade.maker_order_id.clone())
                    ).get_result::<OrderBookRecord>(app_conn)?;


                    let marker_asset = asset_book::dsl::asset_book.filter(
                        asset_book::id.eq(maker_order.ask_asset.clone())
                    ).get_result::<AssetBookRecord>(app_conn)?;

                    let maket_wallet = cradlewalletaccounts::dsl::cradlewalletaccounts.filter(
                        cradlewalletaccounts::id.eq(maker_order.wallet.clone())
                    ).get_result::<CradleWalletAccountRecord>(app_conn)?;


                    let taker_order = orderbook::dsl::orderbook.filter(
                        orderbook::id.eq(trade.taker_order_id.clone())
                    ).get_result::<OrderBookRecord>(app_conn)?;

                    let taker_asset = asset_book::dsl::asset_book.filter(
                        asset_book::id.eq(taker_order.ask_asset.clone())
                    ).get_result::<AssetBookRecord>(app_conn)?;

                    let taker_wallet = cradlewalletaccounts::dsl::cradlewalletaccounts.filter(
                        cradlewalletaccounts::id.eq(taker_order.wallet.clone())
                    ).get_result::<CradleWalletAccountRecord>(app_conn)?;

                    let disable_onchain_settlement = env::var("DISABLE_ONCHAIN_SETTLEMENT")
                        .unwrap_or_default() == "true";

                    if disable_onchain_settlement {

                        let mut tx_id = Uuid::new_v4().to_string();
                        tx_id = format!("test_transaction_{}", tx_id);
                        let _ = diesel::update(orderbooktrades::table.filter(
                            orderbooktrades::id.eq(trade.id.clone())
                        ))
                            .set((
                                orderbooktrades::settlement_status.eq(crate::order_book::db_types::SettlementStatus::Settled),
                                orderbooktrades::settled_at.eq(Utc::now().naive_utc()),
                                orderbooktrades::settlement_tx.eq(Some(tx_id))
                            ))
                            .execute(app_conn)?;

                        continue;
                    }

                    let res = app_config.wallet.execute(ContractCallInput::OrderBookSettler(
                        OrderBookSettlerFunctionInput::SettleOrder(
                            SettleOrderInputArgs {
                                bid_asset: marker_asset.token,
                                ask_asset: taker_asset.token,
                                ask_asset_amount: trade.taker_filled_amount.to_u64().ok_or_else(||anyhow!("Amount too large to convert to u64"))?,
                                bid_asset_amount: trade.maker_filled_amount.to_u64().ok_or_else(||anyhow!("Amount too large to convert to u64"))?,
                                asker: taker_wallet.contract_id,
                                bidder: maket_wallet.contract_id
                            }
                        )
                    )).await?;

                    if let ContractCallOutput::OrderBookSettler(OrderBookSettlerFunctionOutput::SettleOrder(res)) = res {
                        let _ = diesel::update(orderbooktrades::table.filter(
                            orderbooktrades::id.eq(trade.id.clone())
                        ))
                        .set((
                            orderbooktrades::settlement_status.eq(crate::order_book::db_types::SettlementStatus::Settled),
                            orderbooktrades::settled_at.eq(Utc::now().naive_utc()),
                            orderbooktrades::settlement_tx.eq(Some(res.transaction_id.clone()))
                        ))
                        .execute(app_conn)?;

                        continue
                    }else {
                        return Err(anyhow!("Unexpected contract call output"));
                    }
                }

                // io_conn.to(format!("order_{}", order_id)).emit("settled", &json!({
                //     "trades": trades.clone()
                // })).await.map_err(|e|anyhow!("Failed to send to room {}", e))?;

                Ok(OrderBookProcessorOutput::SettleOrder)
            }
            OrderBookProcessorInput::UpdateOrderFill(order_id, remaining_bid, unfilled_ask, trades)=>{
                let order = orderbook::dsl::orderbook.filter(orderbook::id.eq(order_id.clone())).first::<OrderBookRecord>(app_conn)?;

                let new_filled_bid = &order.filled_bid_amount + (order.bid_amount.clone() - remaining_bid.clone());
                let new_filled_ask = &order.filled_ask_amount + (order.ask_amount.clone() - unfilled_ask.clone());
                let new_status = if remaining_bid.clone() == BigDecimal::from(0) && unfilled_ask.clone() == BigDecimal::from(0) {
                    crate::order_book::db_types::OrderStatus::Closed
                } else {
                    crate::order_book::db_types::OrderStatus::Open
                };

                match new_status {
                    OrderStatus::Closed=>{
                        let _ = diesel::update(orderbook::table.filter(orderbook::id.eq(order_id.clone())))
                            .set((
                                orderbook::filled_bid_amount.eq(new_filled_bid.clone()),
                                orderbook::filled_ask_amount.eq(new_filled_ask.clone()),
                                orderbook::status.eq(new_status),
                                orderbook::filled_at.eq(Utc::now().naive_utc())
                            ))
                            .execute(app_conn)?;
                    },
                    _=>{
                        let _ = diesel::update(orderbook::table.filter(orderbook::id.eq(order_id.clone())))
                            .set((
                                orderbook::filled_bid_amount.eq(new_filled_bid.clone()),
                                orderbook::filled_ask_amount.eq(new_filled_ask.clone()),
                                orderbook::status.eq(new_status)
                            ))
                            .execute(app_conn)?;
                    }
                }


                let unlock_asset_request = ActionRouterInput::OrderBook(
                    OrderBookProcessorInput::UnLockWalletAssets(
                        order.wallet.clone(),
                        order.bid_asset.clone(),
                        new_filled_bid.clone()
                    )
                );

                let _ = Box::pin(unlock_asset_request.process(app_config.clone())).await?;


                // io_conn.to(format!("order_{}", order_id)).emit("order-filled", &json!({})).await.map_err(|e|anyhow!("Unable to send broadcast {} ", e))?;

                for trade in trades {
                    // TODO: Unlock assets in wallet

                    let trade_order = orderbook::dsl::orderbook.filter(
                        orderbook::id.eq(trade.maker_order_id.clone())
                    ).get_result::<OrderBookRecord>(app_conn)?;

                    let remaining_bid = &trade_order.bid_amount - (&trade_order.filled_bid_amount + trade.maker_filled_amount.clone());
                    let remaining_ask = &trade_order.ask_amount - (&trade_order.filled_ask_amount + trade.taker_filled_amount.clone());



                    let request = ActionRouterInput::OrderBook(
                        OrderBookProcessorInput::UpdateOrderFill(
                            trade.maker_order_id.clone(),
                            remaining_bid,
                            remaining_ask,
                            Vec::new()
                        )
                    );

                    let _ = Box::pin(request.process(app_config.clone())).await?;
                }

                Ok(OrderBookProcessorOutput::UpdateOrderFill)
            }
            OrderBookProcessorInput::CancelOrder(order_id) => {
                let order = orderbook::dsl::orderbook
                    .filter(orderbook::id.eq(order_id.clone()))
                    .first::<OrderBookRecord>(app_conn)?;

                let remaining_locked = &order.bid_amount - &order.filled_bid_amount;

                if remaining_locked > BigDecimal::from(0) {
                    let unlock_request = ActionRouterInput::OrderBook(
                        OrderBookProcessorInput::UnLockWalletAssets(
                            order.wallet.clone(),
                            order.bid_asset.clone(),
                            remaining_locked
                        )
                    );
                    let _ = Box::pin(unlock_request.process(app_config.clone())).await?;
                }

                let _ = diesel::update(orderbook::table.filter(orderbook::id.eq(order_id.clone())))
                    .set(orderbook::status.eq(crate::order_book::db_types::OrderStatus::Cancelled))
                    .execute(app_conn)?;

                Ok(OrderBookProcessorOutput::CancelOrder)
            }
            OrderBookProcessorInput::GetOrder(order_id) => {

                let order_record = orderbook::dsl::orderbook.filter(
                    orderbook::dsl::id.eq(order_id.clone())
                ).get_result::<OrderBookRecord>(app_conn)?;

                Ok(OrderBookProcessorOutput::GetOrder(order_record))
            }
            OrderBookProcessorInput::GetOrders(filter) => {

                let mut query = orderbook::dsl::orderbook.into_boxed();

                if let Some(wallet) = &filter.wallet {
                    query = query.filter(orderbook::dsl::wallet.eq(wallet.clone()));
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
            },
            OrderBookProcessorInput::LockWalletAssets(wallet_id, asset, amount) => {

                let wallet = cradlewalletaccounts::dsl::cradlewalletaccounts.filter(
                    cradlewalletaccounts::dsl::id.eq(wallet_id.clone())
                ).get_result::<CradleWalletAccountRecord>(app_conn)?;

                let asset_record = asset_book::dsl::asset_book.filter(
                    asset_book::dsl::id.eq(asset.clone())
                ).get_result::<AssetBookRecord>(app_conn)?;

                let res = app_config.wallet.execute(
                    ContractCallInput::CradleAccount(
                        CradleAccountFunctionInput::LockAsset(
                            LockAssetArgs {
                                asset: asset_record.token,
                                amount: amount.to_u64().ok_or_else(||anyhow!("Amount too large to convert to u64"))?,
                                account_contract_id: wallet.contract_id
                            }
                        )
                    )
                ).await?;


                if let ContractCallOutput::CradleAccount(CradleAccountFunctionOutput::LockAsset(_)) = res {
                    Ok(OrderBookProcessorOutput::LockWalletAssets)
                }else{
                    Err(anyhow!("Unexpected contract call output"))
                }
            },
            OrderBookProcessorInput::UnLockWalletAssets(wallet_id, asset, amount) => {
                let wallet = cradlewalletaccounts::dsl::cradlewalletaccounts.filter(
                    cradlewalletaccounts::dsl::id.eq(wallet_id.clone())
                ).get_result::<CradleWalletAccountRecord>(app_conn)?;

                let asset_record = asset_book::dsl::asset_book.filter(
                    asset_book::dsl::id.eq(asset.clone())
                ).get_result::<AssetBookRecord>(app_conn)?;

                let disable_onchain_settlement = env::var("DISABLE_ONCHAIN_SETTLEMENT")
                    .unwrap_or_default() == "true";

                if disable_onchain_settlement {
                    return Ok(OrderBookProcessorOutput::UnLockWalletAssets);
                }

                let res = app_config.wallet.execute(
                    ContractCallInput::CradleAccount(
                        CradleAccountFunctionInput::UnLockAsset(
                            UnLockAssetArgs {
                                asset: asset_record.token,
                                amount: amount.to_u64().ok_or_else(||anyhow!("Amount too large to convert to u64"))?,
                                account_contract_id: wallet.contract_id
                            }
                        )
                    )
                ).await?;


                if let ContractCallOutput::CradleAccount(CradleAccountFunctionOutput::UnLockAsset(_)) = res {
                    Ok(OrderBookProcessorOutput::UnLockWalletAssets)
                }else{
                    Err(anyhow!("Unexpected contract call output"))
                }
            }
        }

    }
}