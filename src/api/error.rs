use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::api::response::ApiResponse;

#[derive(Debug)]
pub enum ApiError {
    BadRequest(String),
    Unauthorized(String),
    NotFound(String),
    InternalError(String),
    DatabaseError(String),
}

impl ApiError {
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::BadRequest(msg.into())
    }

    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self::Unauthorized(msg.into())
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    pub fn internal_error(msg: impl Into<String>) -> Self {
        Self::InternalError(msg.into())
    }

    pub fn database_error(msg: impl Into<String>) -> Self {
        Self::DatabaseError(msg.into())
    }

    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn message(&self) -> String {
        match self {
            ApiError::BadRequest(msg) => msg.clone(),
            ApiError::Unauthorized(msg) => msg.clone(),
            ApiError::NotFound(msg) => format!("{} not found", msg),
            ApiError::InternalError(msg) => msg.clone(),
            ApiError::DatabaseError(msg) => msg.clone(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_response = ApiResponse::<serde_json::Value>::error(self.message());

        (status, Json(error_response)).into_response()
    }
}
