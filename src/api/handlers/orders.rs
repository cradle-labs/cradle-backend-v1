use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use crate::{
    order_book::processor_enums::{OrderBookProcessorInput, OrderBookProcessorOutput},
    action_router::{ActionRouterInput, ActionRouterOutput},
    api::{error::ApiError, response::ApiResponse},
    utils::app_config::AppConfig,
};

/// Query parameters for filtering orders
#[derive(Debug, Deserialize)]
pub struct OrderFilterParams {
    pub wallet: Option<String>,
    pub market_id: Option<String>,
    pub status: Option<String>,
    pub order_type: Option<String>,
    pub mode: Option<String>,
}

/// GET /orders/{id} - Get order by UUID
pub async fn get_order_by_id(
    State(app_config): State<AppConfig>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), ApiError> {
    let order_id = uuid::Uuid::parse_str(&id)
        .map_err(|_| ApiError::bad_request("Invalid order ID format"))?;

    let action = ActionRouterInput::OrderBook(OrderBookProcessorInput::GetOrder(order_id));

    let result = action
        .process(app_config)
        .await
        .map_err(|_| ApiError::not_found("Order"))?;

    match result {
        ActionRouterOutput::OrderBook(output) => {
            match output {
                OrderBookProcessorOutput::GetOrder(order) => {
                    let json = serde_json::to_value(&order)
                        .map_err(|e| ApiError::internal_error(format!("Failed to serialize: {}", e)))?;
                    Ok((StatusCode::OK, Json(ApiResponse::success(json))))
                }
                _ => Err(ApiError::internal_error("Unexpected response type")),
            }
        }
        _ => Err(ApiError::internal_error("Unexpected response type")),
    }
}

/// GET /orders - Get orders with optional filters
pub async fn get_orders(
    State(app_config): State<AppConfig>,
    Query(params): Query<OrderFilterParams>,
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), ApiError> {
    // For now, return all orders without filtering
    let action = ActionRouterInput::OrderBook(OrderBookProcessorInput::GetOrders(
        crate::order_book::processor_enums::GetOrdersFilter {
            wallet: None,
            market_id: None,
            status: None,
            order_type: None,
            mode: None,
        },
    ));

    let result = action
        .process(app_config)
        .await
        .map_err(|e| ApiError::database_error(format!("Failed to fetch orders: {}", e)))?;

    match result {
        ActionRouterOutput::OrderBook(output) => {
            match output {
                OrderBookProcessorOutput::GetOrders(orders) => {
                    let json = serde_json::to_value(&orders)
                        .map_err(|e| ApiError::internal_error(format!("Failed to serialize: {}", e)))?;
                    Ok((StatusCode::OK, Json(ApiResponse::success(json))))
                }
                _ => Err(ApiError::internal_error("Unexpected response type")),
            }
        }
        _ => Err(ApiError::internal_error("Unexpected response type")),
    }
}
