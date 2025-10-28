use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::order_book::db_types::{CreateOrderBookTrade, FillMode, NewOrderBookRecord, OrderBookRecord, OrderStatus, OrderType};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GetOrdersFilter {
    pub wallet: Option<Uuid>,
    pub market_id: Option<Uuid>,
    pub status: Option<OrderStatus>,
    pub order_type: Option<OrderType>,
    pub mode: Option<FillMode>
}

#[derive(Deserialize, Serialize)]
pub enum OrderBookProcessorInput {
    PlaceOrder(NewOrderBookRecord),
    UpdateOrderFill(Uuid, BigDecimal, BigDecimal, Vec<CreateOrderBookTrade>),
    SettleOrder(Uuid),
    CancelOrder(Uuid),
    GetOrder(Uuid),
    GetOrders(GetOrdersFilter),
    LockWalletAssets(Uuid, Uuid, BigDecimal),
    UnLockWalletAssets(Uuid, Uuid, BigDecimal)
}

#[derive(Deserialize, Serialize)]
pub enum OrderFillStatus {
    Partial,
    Filled,
    Cancelled
}



#[derive(Deserialize, Serialize)]
pub struct OrderFillResult {
    pub id: Uuid,
    pub status: OrderFillStatus,
    pub bid_amount_filled: BigDecimal,
    pub ask_amount_filled: BigDecimal,
    pub matched_trades: Vec<Uuid>
}


#[derive(Deserialize, Serialize)]
pub enum OrderBookProcessorOutput {
    PlaceOrder(OrderFillResult),
    SettleOrder,
    UpdateOrderFill,
    CancelOrder,
    GetOrder(OrderBookRecord),
    GetOrders(Vec<OrderBookRecord>),
    LockWalletAssets,
    UnLockWalletAssets
}


