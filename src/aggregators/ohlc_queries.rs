use anyhow::{anyhow, Result};
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use uuid::Uuid;

use crate::order_book::db_types::OrderBookRecord;
use crate::schema::{orderbook, orderbooktrades};

/// Represents a trade with relevant market/asset information for OHLC aggregation
#[derive(Debug, Clone)]
pub struct TradeDataForAggregation {
    pub execution_price: BigDecimal,
    pub maker_filled_amount: BigDecimal,
    pub taker_filled_amount: BigDecimal,
    pub created_at: NaiveDateTime,
    pub market_id: Uuid,
    pub asset_id: Uuid, // The asset being aggregated
}

/// Fetches trades for a specific market and asset within a time range
///
/// This function queries orderbooktrades joined with orderbook to get market and asset info.
/// For each trade, it determines which asset to use based on the order details.
///
/// # Arguments
/// * `market_id` - The market to filter trades for
/// * `asset_id` - The asset to aggregate (can be bid or ask asset)
/// * `start_time` - Start of the time window
/// * `end_time` - End of the time window
/// * `conn` - Database connection
pub fn get_trades_for_market_asset(
    market_id: Uuid,
    asset_id: Uuid,
    start_time: NaiveDateTime,
    end_time: NaiveDateTime,
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
) -> Result<Vec<TradeDataForAggregation>> {
    use crate::schema::orderbook::dsl as ob_dsl;
    use crate::schema::orderbooktrades::dsl as ot_dsl;

    // Get all trades within the time window
    let trades = ot_dsl::orderbooktrades
        .inner_join(ob_dsl::orderbook.on(ot_dsl::maker_order_id.eq(ob_dsl::id)))
        .filter(
            ot_dsl::created_at
                .ge(start_time)
                .and(ot_dsl::created_at.lt(end_time))
                .and(ob_dsl::market_id.eq(market_id)),
        )
        .select((
            ot_dsl::id,
            ot_dsl::maker_order_id,
            ot_dsl::taker_order_id,
            ot_dsl::maker_filled_amount,
            ot_dsl::taker_filled_amount,
            ot_dsl::created_at,
            ob_dsl::market_id,
            ob_dsl::bid_asset,
            ob_dsl::ask_asset,
        ))
        .load::<(
            uuid::Uuid,
            uuid::Uuid,
            uuid::Uuid,
            BigDecimal,
            BigDecimal,
            NaiveDateTime,
            uuid::Uuid,
            uuid::Uuid,
            uuid::Uuid,
        )>(conn)?;

    // Now fetch taker order info to determine which asset each trade uses
    let mut aggregation_trades = Vec::new();

    for (
        _trade_id,
        _maker_order_id,
        taker_order_id,
        maker_filled_amount,
        taker_filled_amount,
        created_at,
        market_id_from_maker,
        bid_asset,
        ask_asset,
    ) in trades
    {
        // Get taker order to understand the market context
        let taker_order = ob_dsl::orderbook
            .filter(ob_dsl::id.eq(taker_order_id))
            .first::<OrderBookRecord>(conn)?;

        // Determine if this trade involves our target asset
        let is_trading_asset = bid_asset == asset_id || ask_asset == asset_id ||
                              taker_order.bid_asset == asset_id || taker_order.ask_asset == asset_id;

        if !is_trading_asset {
            continue;
        }

        // For OHLC purposes, we'll use the filled amounts as proxy for volume
        // The execution price would be derived from the order's price field
        // We'll get that from the maker order
        let maker_order = ob_dsl::orderbook
            .filter(ob_dsl::id.eq(_maker_order_id))
            .first::<OrderBookRecord>(conn)?;

        aggregation_trades.push(TradeDataForAggregation {
            execution_price: maker_order.price.clone(),
            maker_filled_amount: maker_filled_amount.clone(),
            taker_filled_amount: taker_filled_amount.clone(),
            created_at,
            market_id: market_id_from_maker,
            asset_id,
        });
    }

    Ok(aggregation_trades)
}

/// Calculates OHLC values from a set of trades
///
/// # Arguments
/// * `trades` - The trades to aggregate
///
/// # Returns
/// A tuple of (open, high, low, close, volume)
pub fn calculate_ohlc(
    trades: &[TradeDataForAggregation],
) -> Result<(BigDecimal, BigDecimal, BigDecimal, BigDecimal, BigDecimal)> {
    if trades.is_empty() {
        return Err(anyhow!("No trades to aggregate"));
    }

    // Sort by timestamp to get proper OHLC order
    let mut sorted_trades = trades.to_vec();
    sorted_trades.sort_by(|a, b| a.created_at.cmp(&b.created_at));

    let open = sorted_trades[0].execution_price.clone();
    let close = sorted_trades[sorted_trades.len() - 1].execution_price.clone();

    let high = sorted_trades
        .iter()
        .map(|t| t.execution_price.clone())
        .max()
        .ok_or_else(|| anyhow!("Failed to calculate high"))?;

    let low = sorted_trades
        .iter()
        .map(|t| t.execution_price.clone())
        .min()
        .ok_or_else(|| anyhow!("Failed to calculate low"))?;

    // Volume is sum of taker filled amounts (could be maker or taker, but taker is the "volume" of the market)
    let volume = sorted_trades
        .iter()
        .fold(BigDecimal::from(0), |acc, t| acc + t.taker_filled_amount.clone());

    Ok((open, high, low, close, volume))
}
