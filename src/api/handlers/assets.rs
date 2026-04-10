use crate::asset_book::db_types::AssetBookRecord;
use crate::schema::asset_book::dsl::asset_book;
use crate::{
    accounts::db_types::CradleWalletAccountRecord,
    accounts_ledger::sql_queries::get_deductions,
    action_router::{ActionRouterInput, ActionRouterOutput},
    api::{error::ApiError, response::ApiResponse},
    asset_book::processor_enums::{
        AssetBookProcessorInput, AssetBookProcessorOutput, GetAssetInputArgs,
    },
    utils::{app_config::AppConfig, cache},
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use bigdecimal::{BigDecimal, ToPrimitive};
use contract_integrator::{hedera::TokenId, utils::functions::commons};
use diesel::RunQueryDsl;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// GET /assets/{id} - Get asset by UUID
pub async fn get_asset_by_id(
    State(app_config): State<AppConfig>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), ApiError> {
    let asset_id =
        uuid::Uuid::parse_str(&id).map_err(|_| ApiError::bad_request("Invalid asset ID format"))?;

    let cache_key = format!("asset:{}", asset_id);

    // Check cache first
    if let Some(redis) = &app_config.redis {
        if let Some(cached) = cache::cache_get::<serde_json::Value>(redis, &cache_key).await {
            return Ok((StatusCode::OK, Json(ApiResponse::success(cached))));
        }
    }

    let action = ActionRouterInput::AssetBook(AssetBookProcessorInput::GetAsset(
        GetAssetInputArgs::ById(asset_id),
    ));

    let result = action
        .process(app_config.clone())
        .await
        .map_err(|_| ApiError::not_found("Asset"))?;

    match result {
        ActionRouterOutput::AssetBook(output) => match output {
            AssetBookProcessorOutput::GetAsset(asset) => {
                let json = serde_json::to_value(&asset)
                    .map_err(|e| ApiError::internal_error(format!("Failed to serialize: {}", e)))?;

                // Cache for 1 hour — asset metadata rarely changes
                if let Some(redis) = &app_config.redis {
                    cache::cache_set(redis, &cache_key, &json, 3600).await;
                }

                Ok((StatusCode::OK, Json(ApiResponse::success(json))))
            }
            _ => Err(ApiError::internal_error("Unexpected response type")),
        },
        _ => Err(ApiError::internal_error("Unexpected response type")),
    }
}

/// GET /assets/token/{token} - Get asset by token
pub async fn get_asset_by_token(
    State(app_config): State<AppConfig>,
    Path(token): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), ApiError> {
    let action = ActionRouterInput::AssetBook(AssetBookProcessorInput::GetAsset(
        GetAssetInputArgs::ByToken(token),
    ));

    let result = action
        .process(app_config)
        .await
        .map_err(|_| ApiError::not_found("Asset"))?;

    match result {
        ActionRouterOutput::AssetBook(output) => match output {
            AssetBookProcessorOutput::GetAsset(asset) => {
                let json = serde_json::to_value(&asset)
                    .map_err(|e| ApiError::internal_error(format!("Failed to serialize: {}", e)))?;
                Ok((StatusCode::OK, Json(ApiResponse::success(json))))
            }
            _ => Err(ApiError::internal_error("Unexpected response type")),
        },
        _ => Err(ApiError::internal_error("Unexpected response type")),
    }
}

/// GET /assets/manager/{manager} - Get asset by manager
pub async fn get_asset_by_manager(
    State(app_config): State<AppConfig>,
    Path(manager): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), ApiError> {
    let action = ActionRouterInput::AssetBook(AssetBookProcessorInput::GetAsset(
        GetAssetInputArgs::ByAssetManager(manager),
    ));

    let result = action
        .process(app_config)
        .await
        .map_err(|_| ApiError::not_found("Asset"))?;

    match result {
        ActionRouterOutput::AssetBook(output) => match output {
            AssetBookProcessorOutput::GetAsset(asset) => {
                let json = serde_json::to_value(&asset)
                    .map_err(|e| ApiError::internal_error(format!("Failed to serialize: {}", e)))?;
                Ok((StatusCode::OK, Json(ApiResponse::success(json))))
            }
            _ => Err(ApiError::internal_error("Unexpected response type")),
        },
        _ => Err(ApiError::internal_error("Unexpected response type")),
    }
}

pub async fn get_assets(
    State(app_config): State<AppConfig>,
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), ApiError> {
    let cache_key = "assets:all";

    // Check cache first
    if let Some(redis) = &app_config.redis {
        if let Some(cached) = cache::cache_get::<serde_json::Value>(redis, cache_key).await {
            return Ok((StatusCode::OK, Json(ApiResponse::success(cached))));
        }
    }

    let pool = app_config.pool.clone();
    let results = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get()?;
        crate::schema::asset_book::dsl::asset_book
            .get_results::<AssetBookRecord>(&mut conn)
            .map_err(anyhow::Error::from)
    })
    .await
    .map_err(|e| ApiError::internal_error(format!("Task join error: {}", e)))?
    .map_err(|e| ApiError::internal_error(format!("Error::{}", e)))?;

    let jsonified = serde_json::to_value(&results)
        .map_err(|e| ApiError::internal_error(format!("Failed to serialize: {}", e)))?;

    // Cache for 1 hour
    if let Some(redis) = &app_config.redis {
        cache::cache_set(redis, cache_key, &jsonified, 3600).await;
    }

    Ok((StatusCode::OK, Json(ApiResponse::success(jsonified))))
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AssetBalance {
    pub balance: u64,
    pub before_deductions: u64,
    pub deductions: u64,
    pub decimals: u64,
}

pub async fn get_asset_balance(
    State(app_config): State<AppConfig>,
    Path((wallet_id, asset_id)): Path<(Uuid, Uuid)>,
) -> Result<(StatusCode, Json<ApiResponse<AssetBalance>>), ApiError> {
    let cache_key = format!("balance:{}:{}", wallet_id, asset_id);

    // Check cache first — avoids expensive Hedera call
    if let Some(redis) = &app_config.redis {
        if let Some(cached) = cache::cache_get::<AssetBalance>(redis, &cache_key).await {
            return Ok((StatusCode::OK, Json(ApiResponse { success: true, data: Some(cached), error: None })));
        }
    }

    // TODO: add support for hbar and other native tokens
    let mut conn = app_config
        .pool
        .get()
        .map_err(|_| ApiError::DatabaseError("Failed to obtain connection".to_string()))?;

    let wallet = app_config.wallet.clone();

    let asset = {
        use crate::schema::asset_book::dsl::*;

        asset_book
            .filter(id.eq(asset_id))
            .get_result::<AssetBookRecord>(&mut conn)
    }
    .map_err(|_| ApiError::DatabaseError("Failed to get asset".to_string()))?;

    let wallet_data = {
        use crate::schema::cradlewalletaccounts::dsl::*;

        cradlewalletaccounts
            .filter(id.eq(wallet_id))
            .get_result::<CradleWalletAccountRecord>(&mut conn)
    }
    .map_err(|_| ApiError::DatabaseError("Failed to get wallet".to_string()))?;

    let balance = commons::get_account_balances(&wallet.client, &wallet_data.contract_id)
        .await
        .map_err(|_| ApiError::InternalError("Failed to get balance".to_string()))?;

    let token_id = TokenId::from_solidity_address(&asset.token)
        .map_err(|_| ApiError::InternalError("Failed to extract token id".to_string()))?;

    let token_balance = *balance.tokens.get(&token_id).unwrap_or(&0);

    let deductions = get_deductions(&mut conn, wallet_data.address, asset_id)
        .map_err(|_| ApiError::InternalError("Failed to get deductions".to_string()))?;
    let deductions_u64 = deductions
        .total
        .to_u64()
        .ok_or_else(|| ApiError::InternalError("BigDecimal conversion failed".to_string()))?;
    let net = token_balance - deductions_u64;

    let res = AssetBalance {
        balance: net,
        before_deductions: token_balance,
        deductions: deductions_u64,
        decimals: asset.decimals as u64,
    };

    // Cache for 30 seconds — balances change on transactions
    if let Some(redis) = &app_config.redis {
        cache::cache_set(redis, &cache_key, &res, 30).await;
    }

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            data: Some(res),
            error: None,
        }),
    ))
}
