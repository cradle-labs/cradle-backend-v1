use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use diesel::RunQueryDsl;
use crate::{
    asset_book::processor_enums::{AssetBookProcessorInput, AssetBookProcessorOutput, GetAssetInputArgs},
    action_router::{ActionRouterInput, ActionRouterOutput},
    api::{error::ApiError, response::ApiResponse},
    utils::app_config::AppConfig,
};
use crate::asset_book::db_types::AssetBookRecord;
use crate::schema::asset_book::dsl::asset_book;

/// GET /assets/{id} - Get asset by UUID
pub async fn get_asset_by_id(
    State(app_config): State<AppConfig>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), ApiError> {
    let asset_id = uuid::Uuid::parse_str(&id)
        .map_err(|_| ApiError::bad_request("Invalid asset ID format"))?;

    let action = ActionRouterInput::AssetBook(AssetBookProcessorInput::GetAsset(
        GetAssetInputArgs::ById(asset_id),
    ));

    let result = action
        .process(app_config)
        .await
        .map_err(|_| ApiError::not_found("Asset"))?;

    match result {
        ActionRouterOutput::AssetBook(output) => {
            match output {
                AssetBookProcessorOutput::GetAsset(asset) => {
                    let json = serde_json::to_value(&asset)
                        .map_err(|e| ApiError::internal_error(format!("Failed to serialize: {}", e)))?;
                    Ok((StatusCode::OK, Json(ApiResponse::success(json))))
                }
                _ => Err(ApiError::internal_error("Unexpected response type")),
            }
        }
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
        ActionRouterOutput::AssetBook(output) => {
            match output {
                AssetBookProcessorOutput::GetAsset(asset) => {
                    let json = serde_json::to_value(&asset)
                        .map_err(|e| ApiError::internal_error(format!("Failed to serialize: {}", e)))?;
                    Ok((StatusCode::OK, Json(ApiResponse::success(json))))
                }
                _ => Err(ApiError::internal_error("Unexpected response type")),
            }
        }
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
        ActionRouterOutput::AssetBook(output) => {
            match output {
                AssetBookProcessorOutput::GetAsset(asset) => {
                    let json = serde_json::to_value(&asset)
                        .map_err(|e| ApiError::internal_error(format!("Failed to serialize: {}", e)))?;
                    Ok((StatusCode::OK, Json(ApiResponse::success(json))))
                }
                _ => Err(ApiError::internal_error("Unexpected response type")),
            }
        }
        _ => Err(ApiError::internal_error("Unexpected response type")),
    }
}

pub async fn get_assets(
    State(app_config): State<AppConfig> 
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), ApiError> {
    use crate::schema::asset_book::dsl::*;
    let mut conn = app_config.pool.get().map_err(|_|ApiError::internal_error(format!("Failed to acquire connection")))?;
    let results = crate::schema::asset_book::dsl::asset_book.get_results::<AssetBookRecord>(&mut conn).map_err(|e|ApiError::internal_error(format!("Error::{}",e)))?;
    let jsonified = serde_json::to_value(&results).map_err(|e| ApiError::internal_error(format!("Failed to serialize: {}", e)))?;
    
    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(jsonified))
        ))
    
}
