use axum::{Json, extract::State};
use contract_integrator::utils::functions::{
    ContractCallInput,
    asset_manager::{AirdropArgs, AssetManagerFunctionInput},
    commons::ContractFunctionProcessor,
};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    accounts::{
        operations::{associate_token, kyc_token},
        processor_enums::{AssociateTokenToWalletInputArgs, GrantKYCInputArgs},
    },
    api::{error::ApiError, response::ApiResponse},
    asset_book::operations::{get_asset, get_wallet, mint_asset},
    map_to_api_error,
    utils::app_config::AppConfig,
};

#[derive(Deserialize, Serialize)]
pub struct AirdropRequestFields {
    pub asset: Uuid,
    pub account: Uuid,
}

pub async fn airdrop_request(
    State(app_config): State<AppConfig>,
    Json(fields): Json<AirdropRequestFields>,
) -> Result<(StatusCode, Json<ApiResponse<()>>), ApiError> {
    let mut conn = map_to_api_error!(app_config.pool.get(), "Unable to obtain db connection")?;
    let mut action_wallet = app_config.wallet.clone();
    println!("Git acion wallet");

    let wallet_data = map_to_api_error!(
        get_wallet(&mut conn, fields.account).await,
        "Failed to get wallet"
    )?;

    let token_data = map_to_api_error!(
        get_asset(&mut conn, fields.asset).await,
        "Failed to get asset"
    )?;

    map_to_api_error!(
        associate_token(
            &mut conn,
            &mut action_wallet,
            AssociateTokenToWalletInputArgs {
                wallet_id: wallet_data.id,
                token: token_data.id
            }
        )
        .await,
        "Failed to associate token"
    )?;

    map_to_api_error!(
        kyc_token(
            &mut conn,
            &mut action_wallet,
            GrantKYCInputArgs {
                wallet_id: wallet_data.id,
                token: token_data.id
            }
        )
        .await,
        "Failed to grant kyc"
    )?;
    map_to_api_error!(
        mint_asset(
            &mut conn,
            &mut action_wallet,
            token_data.id,
            100_000_000_000_000
        )
        .await,
        "Failed to mint"
    )?;
    let airdrop_request =
        ContractCallInput::AssetManager(AssetManagerFunctionInput::Airdrop(AirdropArgs {
            amount: 100_000_000_000_000, // A mullion of the asset
            asset_contract: token_data.asset_manager.clone(),
            target: wallet_data.address.clone(),
        }));

    match airdrop_request.process(&mut action_wallet).await {
        Ok(v) => Ok((StatusCode::OK, Json(ApiResponse::success(())))),
        Err(e) => {
            println!("Something went wrong:: {}", e);
            Err(ApiError::InternalError(
                "Failed to airdrop tokens".to_string(),
            ))
        }
    }
}
