use crate::schema::asset_book as AssetBook;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

#[derive(DbEnum, Deserialize, Serialize, Debug, Clone)]
#[ExistingTypePath = "crate::schema::sql_types::AssetType"]
#[serde(rename_all = "lowercase")]
pub enum AssetType {
    Bridged,
    Native,
    #[serde(rename = "yield_bearing")]
    #[db_rename = "yield_bearing"]
    YieldBearing,
    #[serde(rename = "chain_native")]
    #[db_rename = "chain_native"]
    ChainNative,
    #[serde(rename = "stablecoin")]
    #[db_rename = "stablecoin"]
    StableCoin,
    Volatile,
}

impl From<usize> for AssetType {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::Bridged,
            1 => Self::Native,
            2 => Self::YieldBearing,
            3 => Self::ChainNative,
            4 => Self::StableCoin,
            5 => Self::Volatile,
            _ => Self::Volatile,
        }
    }
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
    pub icon: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Insertable)]
#[diesel(table_name = AssetBook)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CreateAssetOnBook {
    pub asset_manager: String,
    pub token: String,
    pub asset_type: Option<AssetType>,
    pub name: String,
    pub symbol: String,
    pub decimals: i32,
    pub icon: Option<String>,
}
