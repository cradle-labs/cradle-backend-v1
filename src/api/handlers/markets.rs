use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use diesel::RunQueryDsl;
use serde::Deserialize;

use crate::{
    market::{
        processor_enums::{MarketProcessorInput, MarketProcessorOutput},
        db_types::MarketRecord,
    },
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

/// GET /markets - Get all markets
pub async fn get_markets(
    State(app_config): State<AppConfig>,
    Query(_params): Query<MarketFilterParams>,
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), ApiError> {
    let mut conn = app_config
        .pool
        .get()
        .map_err(|_| ApiError::internal_error("Failed to acquire database connection"))?;

    let results = crate::schema::markets::dsl::markets
        .get_results::<MarketRecord>(&mut conn)
        .map_err(|e| ApiError::internal_error(format!("Database error: {}", e)))?;

    let json = serde_json::to_value(&results)
        .map_err(|e| ApiError::internal_error(format!("Failed to serialize: {}", e)))?;

    Ok((StatusCode::OK, Json(ApiResponse::success(json))))
}
