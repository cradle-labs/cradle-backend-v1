use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use crate::{
    market::processor_enums::{MarketProcessorInput, MarketProcessorOutput},
    action_router::{ActionRouterInput, ActionRouterOutput},
    api::{error::ApiError, response::ApiResponse},
    utils::app_config::AppConfig,
};

/// Query parameters for filtering markets
#[derive(Debug, Deserialize)]
pub struct MarketFilterParams {
    #[serde(rename = "market_type")]
    pub market_type: Option<String>,
    pub status: Option<String>,
    pub regulation: Option<String>,
}

/// GET /markets/{id} - Get market by UUID
pub async fn get_market_by_id(
    State(app_config): State<AppConfig>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), ApiError> {
    let market_id = uuid::Uuid::parse_str(&id)
        .map_err(|_| ApiError::bad_request("Invalid market ID format"))?;

    let action = ActionRouterInput::Markets(MarketProcessorInput::GetMarket(market_id));

    let result = action
        .process(app_config)
        .await
        .map_err(|_| ApiError::not_found("Market"))?;

    match result {
        ActionRouterOutput::Markets(output) => {
            match output {
                MarketProcessorOutput::GetMarket(market) => {
                    let json = serde_json::to_value(&market)
                        .map_err(|e| ApiError::internal_error(format!("Failed to serialize: {}", e)))?;
                    Ok((StatusCode::OK, Json(ApiResponse::success(json))))
                }
                _ => Err(ApiError::internal_error("Unexpected response type")),
            }
        }
        _ => Err(ApiError::internal_error("Unexpected response type")),
    }
}

/// GET /markets - Get markets with optional filters
pub async fn get_markets(
    State(app_config): State<AppConfig>,
    Query(params): Query<MarketFilterParams>,
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), ApiError> {
    // For now, return all markets without filtering
    // Full filtering implementation would require parsing enum values
    let action = ActionRouterInput::Markets(MarketProcessorInput::GetMarkets(
        crate::market::processor_enums::GetMarketsFilter {
            status: None,
            market_type: None,
            regulation: None,
        },
    ));

    let result = action
        .process(app_config)
        .await
        .map_err(|e| ApiError::database_error(format!("Failed to fetch markets: {}", e)))?;

    match result {
        ActionRouterOutput::Markets(output) => {
            match output {
                MarketProcessorOutput::GetMarkets(markets) => {
                    let json = serde_json::to_value(&markets)
                        .map_err(|e| ApiError::internal_error(format!("Failed to serialize: {}", e)))?;
                    Ok((StatusCode::OK, Json(ApiResponse::success(json))))
                }
                _ => Err(ApiError::internal_error("Unexpected response type")),
            }
        }
        _ => Err(ApiError::internal_error("Unexpected response type")),
    }
}
