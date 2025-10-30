use axum::http::HeaderMap;

use crate::api::error::ApiError;

/// Extract and validate Bearer token from Authorization header
pub async fn validate_auth(
    headers: &HeaderMap,
    secret_key: &str,
) -> Result<(), ApiError> {
    println!("headers coming in");
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| ApiError::unauthorized("Missing authorization header"))?;

    // Expected format: "Bearer <token>"
    let parts: Vec<&str> = auth_header.split_whitespace().collect();
    if parts.len() != 2 || parts[0] != "Bearer" {
        return Err(ApiError::unauthorized(
            "Invalid authorization header format. Expected: Bearer <token>",
        ));
    }

    let token = parts[1];
    if token != secret_key {
        return Err(ApiError::unauthorized("Invalid authentication token"));
    }

    Ok(())
}
