use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::{
    lending_pool::processor_enums::{LendingPoolFunctionsInput, LendingPoolFunctionsOutput},
    action_router::{ActionRouterInput, ActionRouterOutput},
    api::{error::ApiError, response::ApiResponse},
    utils::app_config::AppConfig,
};

/// GET /pools/{id} - Get lending pool by UUID
pub async fn get_pool_by_id(
    State(app_config): State<AppConfig>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), ApiError> {
    let pool_id = uuid::Uuid::parse_str(&id)
        .map_err(|_| ApiError::bad_request("Invalid pool ID format"))?;

    let action = ActionRouterInput::Pool(LendingPoolFunctionsInput::GetLendingPool(
        crate::lending_pool::processor_enums::GetLendingPoolInput::ById(pool_id),
    ));

    let result = action
        .process(app_config)
        .await
        .map_err(|_| ApiError::not_found("Lending pool"))?;

    match result {
        ActionRouterOutput::Pool(output) => {
            match output {
                LendingPoolFunctionsOutput::GetLendingPool(pool) => {
                    let json = serde_json::to_value(&pool)
                        .map_err(|e| ApiError::internal_error(format!("Failed to serialize: {}", e)))?;
                    Ok((StatusCode::OK, Json(ApiResponse::success(json))))
                }
                _ => Err(ApiError::internal_error("Unexpected response type")),
            }
        }
        _ => Err(ApiError::internal_error("Unexpected response type")),
    }
}

/// GET /pools/name/{name} - Get lending pool by name
pub async fn get_pool_by_name(
    State(app_config): State<AppConfig>,
    Path(name): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), ApiError> {
    let action = ActionRouterInput::Pool(LendingPoolFunctionsInput::GetLendingPool(
        crate::lending_pool::processor_enums::GetLendingPoolInput::ByName(name),
    ));

    let result = action
        .process(app_config)
        .await
        .map_err(|_| ApiError::not_found("Lending pool"))?;

    match result {
        ActionRouterOutput::Pool(output) => {
            match output {
                LendingPoolFunctionsOutput::GetLendingPool(pool) => {
                    let json = serde_json::to_value(&pool)
                        .map_err(|e| ApiError::internal_error(format!("Failed to serialize: {}", e)))?;
                    Ok((StatusCode::OK, Json(ApiResponse::success(json))))
                }
                _ => Err(ApiError::internal_error("Unexpected response type")),
            }
        }
        _ => Err(ApiError::internal_error("Unexpected response type")),
    }
}

/// GET /pools/address/{address} - Get lending pool by address
pub async fn get_pool_by_address(
    State(app_config): State<AppConfig>,
    Path(address): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), ApiError> {
    let action = ActionRouterInput::Pool(LendingPoolFunctionsInput::GetLendingPool(
        crate::lending_pool::processor_enums::GetLendingPoolInput::ByAddress(address),
    ));

    let result = action
        .process(app_config)
        .await
        .map_err(|_| ApiError::not_found("Lending pool"))?;

    match result {
        ActionRouterOutput::Pool(output) => {
            match output {
                LendingPoolFunctionsOutput::GetLendingPool(pool) => {
                    let json = serde_json::to_value(&pool)
                        .map_err(|e| ApiError::internal_error(format!("Failed to serialize: {}", e)))?;
                    Ok((StatusCode::OK, Json(ApiResponse::success(json))))
                }
                _ => Err(ApiError::internal_error("Unexpected response type")),
            }
        }
        _ => Err(ApiError::internal_error("Unexpected response type")),
    }
}

/// GET /pools/{id}/snapshot - Get latest snapshot for a pool
pub async fn get_pool_snapshot(
    State(app_config): State<AppConfig>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), ApiError> {
    let pool_id = uuid::Uuid::parse_str(&id)
        .map_err(|_| ApiError::bad_request("Invalid pool ID format"))?;

    let action = ActionRouterInput::Pool(LendingPoolFunctionsInput::GetSnapShot(pool_id));

    let result = action
        .process(app_config)
        .await
        .map_err(|_| ApiError::not_found("Pool snapshot"))?;

    match result {
        ActionRouterOutput::Pool(output) => {
            match output {
                LendingPoolFunctionsOutput::GetSnapShot(snapshot) => {
                    let json = serde_json::to_value(&snapshot)
                        .map_err(|e| ApiError::internal_error(format!("Failed to serialize: {}", e)))?;
                    Ok((StatusCode::OK, Json(ApiResponse::success(json))))
                }
                _ => Err(ApiError::internal_error("Unexpected response type")),
            }
        }
        _ => Err(ApiError::internal_error("Unexpected response type")),
    }
}
