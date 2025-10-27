use axum::{
    async_trait,
    extract::{FromRequest, Request},
    response::IntoResponse,
    Json,
};
use serde_json::Value;

use crate::api::error::ApiError;

/// Custom extractor for ActionRouterInput JSON
pub struct ActionRouterExtractor(pub Value);

#[async_trait]
impl<S> FromRequest<S> for ActionRouterExtractor
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<Value>::from_request(req, state)
            .await
            .map_err(|e| {
                ApiError::bad_request(format!("Failed to parse JSON: {}", e))
            })?;

        // Validate basic structure
        if !value.is_object() {
            return Err(ApiError::bad_request(
                "Request body must be a JSON object",
            ));
        }

        let obj = value.as_object().expect("already checked is_object");
        if obj.len() != 1 {
            return Err(ApiError::bad_request(
                "Request body must contain exactly one top-level enum variant",
            ));
        }

        Ok(ActionRouterExtractor(value))
    }
}
