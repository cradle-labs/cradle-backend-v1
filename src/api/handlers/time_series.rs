use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use bigdecimal::BigDecimal;
use serde::Deserialize;
use std::str::FromStr;

use crate::{
    market_time_series::processor_enum::{
        MarketTimeSeriesProcessorInput, MarketTimeSeriesProcessorOutput,
    },
    action_router::{ActionRouterInput, ActionRouterOutput},
    api::{error::ApiError, response::ApiResponse},
    utils::app_config::AppConfig,
};

/// Query parameters for time series history
#[derive(Debug, Deserialize)]
pub struct TimeSeriesParams {
    pub market: String,
    pub duration_secs: String,
    pub interval: String,
}

/// GET /time-series/history - Get time series data with filters
pub async fn get_time_series_history(
    State(app_config): State<AppConfig>,
    Query(params): Query<TimeSeriesParams>,
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), ApiError> {
    // Parse market UUID
    let market_id = uuid::Uuid::parse_str(&params.market)
        .map_err(|_| ApiError::bad_request("Invalid market UUID format"))?;

    // Parse duration in seconds
    let duration_secs = BigDecimal::from_str(&params.duration_secs)
        .map_err(|_| ApiError::bad_request("Invalid duration_secs format. Must be a number"))?;

    // Parse interval
    let interval = parse_time_series_interval(&params.interval)?;

    let action = ActionRouterInput::MarketTimeSeries(
        MarketTimeSeriesProcessorInput::GetHistory(
            crate::market_time_series::processor_enum::GetHistoryInputArgs {
                market_id,
                duration_secs,
                interval,
            },
        ),
    );

    let result = action
        .process(app_config)
        .await
        .map_err(|e| ApiError::database_error(format!("Failed to fetch time series data: {}", e)))?;

    match result {
        ActionRouterOutput::MarketTimeSeries(output) => {
            match output {
                MarketTimeSeriesProcessorOutput::GetHistory(records) => {
                    let json = serde_json::to_value(&records)
                        .map_err(|e| ApiError::internal_error(format!("Failed to serialize: {}", e)))?;
                    Ok((StatusCode::OK, Json(ApiResponse::success(json))))
                }
                _ => Err(ApiError::internal_error("Unexpected response type")),
            }
        }
        _ => Err(ApiError::internal_error("Unexpected response type")),
    }
}

/// Parse time series interval from string
fn parse_time_series_interval(
    s: &str,
) -> Result<crate::market_time_series::db_types::TimeSeriesInterval, ApiError> {
    use crate::market_time_series::db_types::TimeSeriesInterval;
    match s.to_lowercase().as_str() {
        "1min" => Ok(TimeSeriesInterval::OneMinute),
        "5min" => Ok(TimeSeriesInterval::FiveMinutes),
        "15min" => Ok(TimeSeriesInterval::FifteenMinutes),
        "30min" => Ok(TimeSeriesInterval::ThirtyMinutes),
        "1hr" => Ok(TimeSeriesInterval::OneHour),
        "4hr" => Ok(TimeSeriesInterval::FourHours),
        "1day" => Ok(TimeSeriesInterval::OneDay),
        "1week" => Ok(TimeSeriesInterval::OneWeek),
        _ => Err(ApiError::bad_request(
            "Invalid interval. Expected: 1min, 5min, 15min, 30min, 1hr, 4hr, 1day, or 1week",
        )),
    }
}
