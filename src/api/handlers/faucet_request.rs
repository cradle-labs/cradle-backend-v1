use contract_integrator::utils::functions::{ContractCallInput, asset_manager::{AirdropArgs, AssetManagerFunctionInput}, commons::ContractFunctionProcessor};
use diesel::prelude::*;
use axum::{Json, extract::State, response::IntoResponse};
use hyper::StatusCode;
use serde::{Serialize, Deserialize};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    accounts::db_types::CradleWalletAccountRecord, 
    api::{error::ApiError, response::ApiResponse}, 
    asset_book::db_types::{AssetBookRecord, AssetType}, 
    schema::orderbook::wallet, 
    utils::app_config::AppConfig};


#[derive(Deserialize , Serialize )]
pub struct AirdropRequestFields {
    pub asset: Uuid,
    pub account: Uuid
}

#[axum::debug_handler]
pub async fn airdrop_request(
    State(app_config): State<AppConfig>,
    Json(fields): Json<AirdropRequestFields>
) -> impl IntoResponse {

    let mut conn = app_config.pool.get().unwrap();
    let mut action_wallet = app_config.wallet.clone();

    let wallet_data = {
        use crate::schema::cradlewalletaccounts::dsl::*;

        cradlewalletaccounts.filter(
            cradle_account_id.eq(fields.account.clone())
        ).get_result::<CradleWalletAccountRecord>(&mut conn)
    };


    let token_data = {
        use crate::schema::asset_book::dsl::*;

        asset_book.filter(
            id.eq(fields.asset)
        ).get_result::<AssetBookRecord>(&mut conn)
    };

    let airdrop_request = ContractCallInput::AssetManager(
        AssetManagerFunctionInput::Airdrop(AirdropArgs {
            amount: 1_000_000_0000_0000, // A mullion of the asset
            asset_contract: token_data.unwrap().asset_manager,
            target: wallet_data.unwrap().address
        })
    );


    match airdrop_request.process(&mut action_wallet).await {
        Ok(v)=>{
            (
                StatusCode::OK,
                Json(ApiResponse::success(json!({})))
            ).into_response()
        },
        Err(e)=>{
            println!("Something went wrong:: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "message": "something went wrong"
                }))
            ).into_response()
        }
    }

}