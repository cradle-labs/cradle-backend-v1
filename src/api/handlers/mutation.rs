use axum::{extract::State, Json};
use serde_json::Value;
use socketioxide::SocketIo;
use crate::{
    action_router::{ActionRouterInput, ActionRouterOutput},
    api::{error::ApiError, extractors::ActionRouterExtractor, response::ApiResponse},
    utils::app_config::AppConfig,
};

/// POST /process - Main mutation endpoint
/// Accepts ActionRouterInput enum in nested JSON format
///
/// Expected JSON structure:
/// { "Accounts": { "GetAccount": { "ByID": "..." } } }
/// or any other valid ActionRouterInput variant
pub async fn process_mutation(
    State(app_config): State<AppConfig>,
    // State(io): State<SocketIo>,
    ActionRouterExtractor(payload): ActionRouterExtractor,
) -> Result<Json<ApiResponse<Value>>, ApiError> {
    // app_config.set_io(io);
    // Deserialize the JSON into ActionRouterInput
    let action_input: ActionRouterInput = serde_json::from_value(payload)
        .map_err(|e| {
            ApiError::bad_request(format!(
                "Failed to deserialize request into valid action: {}",
                e
            ))
        })?;

    // Process the action through the router
    let result = action_input
        .process(app_config)
        .await
        .map_err(|e| ApiError::database_error(format!("Action processing failed: {}", e)))?;

    // Serialize the result back to JSON
    let result_json = serde_json::to_value(&result)
        .map_err(|e| ApiError::internal_error(format!("Failed to serialize response: {}", e)))?;

    Ok(Json(ApiResponse::success(result_json)))
}
