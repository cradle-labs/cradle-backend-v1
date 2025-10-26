use chrono::NaiveDateTime;
use diesel::{Identifiable, Insertable, Queryable};
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::schema::markets as MarketsTable;

#[derive(DbEnum, Deserialize, Serialize, Debug, Clone)]
#[ExistingTypePath="crate::schema::sql_types::MarketStatus"]
#[serde(rename_all = "lowercase")]
pub enum MarketStatus {
    Active,
    InActive,
    Suspended
}

#[derive(DbEnum, Deserialize, Serialize, Debug, Clone)]
#[ExistingTypePath="crate::schema::sql_types::MarketType"]
#[serde(rename_all = "lowercase")]
pub enum MarketType {
    Spot,
    Derivative,
    Futures
}

#[derive(DbEnum, Deserialize, Serialize, Debug, Clone)]
#[ExistingTypePath="crate::schema::sql_types::MarketRegulation"]
#[serde(rename_all = "lowercase")]
pub enum MarketRegulation {
    Regulated,
    UnRegulated
}

#[derive(Serialize,Deserialize, Debug, Clone, Queryable, Identifiable)]
#[diesel(table_name = MarketsTable)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MarketRecord {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub asset_one: Uuid,
    pub asset_two: Uuid,
    pub created_at: NaiveDateTime,
    pub market_type: MarketType,
    pub market_status: MarketStatus,
    pub market_regulation: MarketRegulation
}


#[derive(Serialize,Deserialize, Debug, Clone, Insertable)]
#[diesel(table_name = MarketsTable)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CreateMarket {
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub asset_one: Uuid,
    pub asset_two: Uuid,
    pub market_type: Option<MarketType>,
    pub market_status: Option<MarketStatus>,
    pub market_regulation: Option<MarketRegulation>
}