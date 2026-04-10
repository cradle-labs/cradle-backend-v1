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
    utils::{app_config::AppConfig, cache},
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

    let cache_key = format!("market:{}", market_id);

    if let Some(redis) = &app_config.redis {
        if let Some(cached) = cache::cache_get::<serde_json::Value>(redis, &cache_key).await {
            return Ok((StatusCode::OK, Json(ApiResponse::success(cached))));
        }
    }

    let action = ActionRouterInput::Markets(MarketProcessorInput::GetMarket(market_id));

    let result = action
        .process(app_config.clone())
        .await
        .map_err(|_| ApiError::not_found("Market"))?;

    match result {
        ActionRouterOutput::Markets(output) => {
            match output {
                MarketProcessorOutput::GetMarket(market) => {
                    let json = serde_json::to_value(&market)
                        .map_err(|e| ApiError::internal_error(format!("Failed to serialize: {}", e)))?;

                    if let Some(redis) = &app_config.redis {
                        cache::cache_set(redis, &cache_key, &json, 600).await;
                    }

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
    let cache_key = "markets:all";

    if let Some(redis) = &app_config.redis {
        if let Some(cached) = cache::cache_get::<serde_json::Value>(redis, cache_key).await {
            return Ok((StatusCode::OK, Json(ApiResponse::success(cached))));
        }
    }

    // Move the blocking Diesel query to the blocking thread pool
    // so it doesn't stall the Tokio worker.
    let pool = app_config.pool.clone();
    let results = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get()?;
        crate::schema::markets::dsl::markets
            .get_results::<MarketRecord>(&mut conn)
            .map_err(anyhow::Error::from)
    })
    .await
    .map_err(|e| ApiError::internal_error(format!("Task join error: {}", e)))?
    .map_err(|e| ApiError::internal_error(format!("Database error: {}", e)))?;

    let json = serde_json::to_value(&results)
        .map_err(|e| ApiError::internal_error(format!("Failed to serialize: {}", e)))?;

    if let Some(redis) = &app_config.redis {
        cache::cache_set(redis, cache_key, &json, 600).await;
    }

    Ok((StatusCode::OK, Json(ApiResponse::success(json))))
}
