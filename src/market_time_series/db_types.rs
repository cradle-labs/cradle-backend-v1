use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use diesel::{Identifiable, Insertable, Queryable, Selectable};
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::schema::markets_time_series as MarketsTimeSeriesTable;

#[derive(Deserialize,Serialize, Debug, Clone, DbEnum)]
#[ExistingTypePath="crate::schema::sql_types::TimeSeriesInterval"]
pub enum TimeSeriesInterval {
    #[serde(rename = "15secs")]
    FifteenSecs,
    #[serde(rename = "30secs")]
    ThirtySecs,
    #[serde(rename = "45secs")]
    FortyFiveSecs,
    #[serde(rename = "1min")]
    OneMinute,
    #[serde(rename = "5min")]
    FiveMinutes,
    #[serde(rename = "15min")]
    FifteenMinutes,
    #[serde(rename = "30min")]
    ThirtyMinutes,
    #[serde(rename = "1hr")]
    OneHour,
    #[serde(rename = "4hr")]
    FourHours,
    #[serde(rename = "1day")]
    OneDay,
    #[serde(rename = "1week")]
    OneWeek
}


#[derive(Deserialize,Serialize, Debug, Clone, DbEnum)]
#[ExistingTypePath="crate::schema::sql_types::DataProviderType"]
pub enum DataProviderType {
    #[serde(rename = "order_book")]
    OrderBook,
    #[serde(rename = "exchange")]
    Exchange,
    #[serde(rename = "aggregated")]
    Aggregated
}

#[derive(Deserialize, Serialize, Queryable, Identifiable, Selectable)]
#[diesel(table_name =  MarketsTimeSeriesTable)]
pub struct MarketTimeSeriesRecord {
    pub id: Uuid,
    pub market_id: Uuid,
    pub asset: Uuid,
    pub open: BigDecimal,
    pub high: BigDecimal,
    pub low: BigDecimal,
    pub close: BigDecimal,
    pub volume: BigDecimal,
    pub created_at: NaiveDateTime,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    pub interval: TimeSeriesInterval,
    pub data_provider_type: DataProviderType,
    pub data_provider: Option<String>
}


#[derive(Deserialize, Serialize,Insertable)]
#[diesel(table_name = MarketsTimeSeriesTable)]
pub struct CreateMarketTimeSeriesRecord {
    pub market_id: Uuid,
    pub asset: Uuid,
    pub open: BigDecimal,
    pub high: BigDecimal,
    pub low: BigDecimal,
    pub close: BigDecimal,
    pub volume: BigDecimal,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    pub interval: Option<TimeSeriesInterval>,
    pub data_provider_type: Option<DataProviderType>,
    pub data_provider: Option<String>,
}