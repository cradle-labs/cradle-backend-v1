use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use diesel::{Identifiable, Insertable, Queryable, QueryableByName, Selectable};
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::schema::orderbook as OrderBookTable;
use crate::schema::orderbooktrades as OrderBookTrades;

#[derive(Deserialize, Serialize, DbEnum, Debug, Clone)]
#[ExistingTypePath = "crate::schema::sql_types::FillMode"]
pub enum FillMode {
    #[serde(rename = "fill-or-kill")]
    FillOrKill,
    #[serde(rename = "immediate-or-cancel")]
    ImmediateOrCancel,
    #[serde(rename = "good-till-cancel")]
    GoodTillCancel
}


#[derive(Deserialize, Serialize, DbEnum, Clone, Debug)]
#[ExistingTypePath = "crate::schema::sql_types::OrderStatus"]
#[serde(rename_all = "lowercase")]
pub enum OrderStatus {
    Open,
    Closed,
    Cancelled
}


#[derive(Deserialize,Serialize, DbEnum, Clone, Debug)]
#[ExistingTypePath = "crate::schema::sql_types::OrderType"]
#[serde(rename_all = "lowercase")]
pub enum OrderType {
    Limit,
    Market
}



#[derive(Deserialize,Serialize, Clone, Debug, Queryable, Identifiable, Selectable, QueryableByName)]
#[diesel(table_name = OrderBookTable)]
pub struct OrderBookRecord {
    pub id: Uuid,
    pub wallet: Uuid,
    pub market_id: Uuid,
    pub bid_asset: Uuid,
    pub ask_asset: Uuid,
    pub bid_amount: BigDecimal,
    pub ask_amount: BigDecimal,
    pub price: BigDecimal,
    pub filled_bid_amount: BigDecimal,
    pub filled_ask_amount: BigDecimal,
    pub mode: FillMode,
    pub status: OrderStatus,
    pub created_at: NaiveDateTime,
    pub filled_at: Option<NaiveDateTime>,
    pub cancelled_at: Option<NaiveDateTime>,
    pub expires_at: Option<NaiveDateTime>,
    pub order_type: OrderType
}


#[derive(Deserialize,Serialize, Clone, Debug, Insertable)]
#[diesel(table_name = OrderBookTable)]
pub struct NewOrderBookRecord {
    pub wallet: Uuid,
    pub market_id: Uuid,
    pub bid_asset: Uuid,
    pub ask_asset: Uuid,
    pub bid_amount: BigDecimal,
    pub ask_amount: BigDecimal,
    pub price: BigDecimal,
    pub mode: Option<FillMode>,
    pub expires_at: Option<NaiveDateTime>,
    pub order_type: Option<OrderType>
}


#[derive(Deserialize, Serialize, Clone, Debug, DbEnum)]
#[ExistingTypePath="crate::schema::sql_types::SettlementStatus"]
pub enum SettlementStatus {
    Matched,
    Settled,
    Failed
}


#[derive(Deserialize, Serialize, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = OrderBookTrades)]
pub struct OrderBookTradeRecord {
    pub id: Uuid,
    pub maker_order_id: Uuid,
    pub taker_order_id: Uuid,
    pub maker_filled_amount: BigDecimal,
    pub taker_filled_amount: BigDecimal,
    pub settlement_tx: Option<String>,
    pub settlement_status: SettlementStatus,
    pub created_at: NaiveDateTime,
    pub settled_at: Option<NaiveDateTime>
}


#[derive(Deserialize,Serialize, Clone, Insertable, Debug)]
#[diesel(table_name = OrderBookTrades)]
pub struct CreateOrderBookTrade {
    pub maker_order_id: Uuid,
    pub taker_order_id: Uuid,
    pub maker_filled_amount: BigDecimal,
    pub taker_filled_amount: BigDecimal
}


#[derive(Deserialize,Serialize, Clone, Debug, QueryableByName)]
pub struct MatchingOrderResult {
    #[sql_type = "diesel::sql_types::Uuid"]
    pub id: Uuid,
    #[sql_type = "diesel::sql_types::Uuid"]
    pub wallet: Uuid,
    #[sql_type = "diesel::sql_types::Uuid"]
    pub bid_asset: Uuid,
    #[sql_type = "diesel::sql_types::Uuid"]
    pub ask_asset: Uuid,
    #[sql_type = "diesel::sql_types::Numeric"]
    pub price: BigDecimal,
    #[sql_type = "crate::schema::sql_types::OrderType"]
    pub order_type: OrderType,
    #[sql_type = "crate::schema::sql_types::FillMode"]
    pub mode: FillMode,
    #[sql_type = "diesel::sql_types::Timestamp"]
    pub created_at: NaiveDateTime,
    #[sql_type = "diesel::sql_types::Numeric"]
    pub remaining_bid_amount: BigDecimal,
    #[sql_type = "diesel::sql_types::Numeric"]
    pub remaining_ask_amount: BigDecimal,
    #[sql_type = "diesel::sql_types::Numeric"]
    pub execution_price: BigDecimal
}