use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::asset_book::db_types::{AssetBookRecord, AssetType};

#[derive(Deserialize, Serialize)]
pub struct CreateNewAssetInputArgs {
    pub asset_type: AssetType,
    pub name: String,
    pub symbol: String,
    pub decimals: i32,
    pub icon: String
}

#[derive(Deserialize,Serialize)]
pub struct CreateExistingAssetInputArgs {
    pub asset_manager: Option<String>,
    pub token: String,
    pub asset_type: AssetType,
    pub name: String,
    pub symbol: String,
    pub decimals: i32,
    pub icon: String
}
#[derive(Deserialize, Serialize)]
pub enum GetAssetInputArgs {
    ById(Uuid),
    ByToken(String),
    ByAssetManager(String)
}
#[derive(Deserialize, Serialize)]
pub enum AssetBookProcessorInput {
    CreateNewAsset(CreateNewAssetInputArgs),
    CreateExistingAsset(CreateExistingAssetInputArgs),
    GetAsset(GetAssetInputArgs)
}

#[derive(Deserialize, Serialize)]
pub enum AssetBookProcessorOutput {
    CreateNewAsset(Uuid),
    CreateExistingAsset(Uuid),
    GetAsset(AssetBookRecord)
}

