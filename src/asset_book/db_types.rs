use std::str::FromStr;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::schema::asset_book as AssetBook;


#[derive(DbEnum, Deserialize, Serialize, Debug, Clone)]
#[ExistingTypePath="crate::schema::sql_types::AssetType"]
#[serde(rename_all = "lowercase")]
pub enum AssetType {
    Bridged,
    Native,
    Yield_Breaking,
    Chain_Native,
    StableCoin,
    Volatile
}

#[derive(Serialize, Deserialize, Debug, Clone, Queryable, Identifiable)]
#[diesel(table_name = AssetBook)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AssetBookRecord {
    pub id: Uuid,
    pub asset_manager: String,
    pub token: String,
    pub created_at: NaiveDateTime,
    pub asset_type: AssetType,
    pub name: String,
    pub symbol: String,
    pub decimals: i32,
    pub icon: Option<String>
}


#[derive(Serialize,Deserialize, Debug, Clone, Insertable)]
#[diesel(table_name = AssetBook)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CreateAssetOnBook {
    pub asset_manager: String,
    pub token: String,
    pub asset_type: Option<AssetType>,
    pub name: String,
    pub symbol: String,
    pub decimals: i32,
    pub icon: Option<String>
}