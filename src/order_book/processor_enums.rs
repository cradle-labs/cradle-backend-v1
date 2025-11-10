use crate::order_book::db_types::{
    CreateOrderBookTrade, FillMode, NewOrderBookRecord, OrderBookRecord, OrderStatus, OrderType,
};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GetOrdersFilter {
    pub wallet: Option<Uuid>,
    pub market_id: Option<Uuid>,
    pub status: Option<OrderStatus>,
    pub order_type: Option<OrderType>,
    pub mode: Option<FillMode>,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum OrderBookProcessorInput {
    PlaceOrder(NewOrderBookRecord),
    GetOrder(Uuid),
    GetOrders(GetOrdersFilter),
}

#[derive(Deserialize, Serialize, Debug)]
pub enum OrderFillStatus {
    Partial,
    Filled,
    Cancelled,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct OrderFillResult {
    pub id: Uuid,
    pub status: OrderFillStatus,
    pub bid_amount_filled: BigDecimal,
    pub ask_amount_filled: BigDecimal,
    pub matched_trades: Vec<Uuid>,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum OrderBookProcessorOutput {
    PlaceOrder(OrderFillResult),
    GetOrder(OrderBookRecord),
    GetOrders(Vec<OrderBookRecord>),
}
