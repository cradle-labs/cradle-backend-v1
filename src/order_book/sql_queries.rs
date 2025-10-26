use diesel::PgConnection;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::prelude::*;
use anyhow::Result;
use bigdecimal::BigDecimal;
use uuid::Uuid;
use crate::order_book::db_types::{CreateOrderBookTrade, MatchingOrderResult, OrderBookRecord};

const MATCHING_ORDERS: &str = r"
-- Find orders that can fill an incoming order (supports both market and limit orders)
WITH incoming_order AS (
    SELECT
        id,
        wallet,
        market_id,
        bid_asset,
        ask_asset,
        bid_amount - filled_bid_amount AS remaining_bid_amount,
        ask_amount - filled_ask_amount AS remaining_ask_amount,
        price,
        mode,
        order_type  -- 'market' or 'limit'
    FROM OrderBook
    WHERE id = $1
)
SELECT
    ob.id,
    ob.wallet,
    ob.bid_asset,
    ob.ask_asset,
    ob.price,
    ob.order_type,
    ob.mode,
    ob.created_at,
    (ob.bid_amount - ob.filled_bid_amount) AS remaining_bid_amount,
    (ob.ask_amount - ob.filled_ask_amount) AS remaining_ask_amount,
    -- Execution price: market orders take the limit order's price
    ob.price AS execution_price
FROM OrderBook ob
CROSS JOIN incoming_order io
WHERE
    ob.status = 'open'
    AND ob.market_id = io.market_id
    AND ob.id != io.id
    AND ob.wallet != io.wallet  -- Remove if self-trading allowed
    AND ob.bid_asset = io.ask_asset
    AND ob.ask_asset = io.bid_asset
    AND (ob.bid_amount - ob.filled_bid_amount) > 0
    AND (ob.ask_amount - ob.filled_ask_amount) > 0
    AND (ob.expires_at IS NULL OR ob.expires_at > NOW())

    -- Price compatibility check: only apply for limit orders
    AND (
        -- If incoming order is MARKET, skip price check (match any price)
        io.order_type = 'market'
        OR
        -- If incoming order is LIMIT, enforce price compatibility
        (io.order_type = 'limit' AND io.price * ob.price >= 1)
    )

    AND NOT EXISTS (
        SELECT 1
        FROM OrderBookTrades obt
        WHERE
            obt.settlement_status = 'matched'
            AND (
                (obt.maker_order_id = ob.id AND obt.taker_order_id = io.id)
                OR (obt.maker_order_id = io.id AND obt.taker_order_id = ob.id)
            )
    )

ORDER BY
    -- Best price first, then time priority
    ob.price ASC,  -- Use DESC for the opposite side
    ob.created_at ASC
;
";


pub async fn get_matching_orders(conn: &mut PooledConnection<ConnectionManager<PgConnection>>, incoming_order: Uuid)->Result<Vec<MatchingOrderResult>> {

    let result = diesel::sql_query(MATCHING_ORDERS)
        .bind::<diesel::sql_types::Uuid, _>(&incoming_order)
        .get_results::<MatchingOrderResult>(conn)?;

    Ok(result)
}


pub fn get_order_fill_trades(
    incoming: &OrderBookRecord,
    matches: Vec<MatchingOrderResult>
) -> (BigDecimal, BigDecimal, Vec<CreateOrderBookTrade>) {
    let mut remaining_bid = incoming.bid_amount.clone() - incoming.filled_bid_amount.clone();
    let mut unfilled_ask = incoming.ask_amount.clone() - incoming.filled_ask_amount.clone();
    let mut trades: Vec<CreateOrderBookTrade> = Vec::new();

    let ratio = incoming.bid_amount.clone() / incoming.ask_amount.clone();

    for matching_order in matches.into_iter() {
        if unfilled_ask <= BigDecimal::from(0) || remaining_bid <= BigDecimal::from(0) {
            break;
        }

        let can_fill_ask = unfilled_ask.clone().min(matching_order.remaining_bid_amount.clone());
        let can_fill_bid = can_fill_ask * &ratio;

        let actual_fill_bid = can_fill_bid.min(matching_order.remaining_ask_amount.clone());
        let actual_fill_ask = &actual_fill_bid / &ratio;

        unfilled_ask -= &actual_fill_ask;
        remaining_bid -= &actual_fill_bid;

        trades.push(CreateOrderBookTrade {
            maker_order_id: matching_order.id.clone(),
            taker_order_id: incoming.id.clone(),
            maker_filled_amount: actual_fill_bid,
            taker_filled_amount: actual_fill_ask
        });
    }

    (remaining_bid, unfilled_ask, trades)
}