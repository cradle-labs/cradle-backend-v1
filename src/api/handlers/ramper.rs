use axum::{Json, extract::State};
use hyper::StatusCode;

use crate::{
    api::{error::ApiError, response::ApiResponse},
    map_to_api_error,
    ramper::{CallbackData, OnRampRequest, OnRampResponse, Ramper},
    utils::app_config::AppConfig,
};

pub async fn request_payment(
    State(app_config): State<AppConfig>,
    Json(req): Json<OnRampRequest>,
) -> Result<(StatusCode, Json<ApiResponse<OnRampResponse>>), ApiError> {
    let ramper = map_to_api_error!(Ramper::from_env(), "Failed to get ramper")?;
    let mut conn = map_to_api_error!(app_config.pool.get(), "Unable to obtain")?;
    let mut wallet = app_config.wallet.clone();

    let res = map_to_api_error!(
        ramper.onramp(&mut wallet, &mut conn, req).await,
        "Failed to get ramper"
    )?;

    Ok((StatusCode::OK, Json(ApiResponse::success(res))))
}

pub async fn handle_callback(
    State(app_config): State<AppConfig>,
    Json(req): Json<CallbackData>,
) -> Result<(StatusCode, Json<ApiResponse<()>>), ApiError> {
    let ramper = map_to_api_error!(Ramper::from_env(), "Failed to get ramper")?;
    let mut conn = map_to_api_error!(app_config.pool.get(), "Unable to obtain")?;
    let mut wallet = app_config.wallet.clone();

    map_to_api_error!(
        ramper.callback_handler(&mut conn, req).await,
        "Failed to handle callback"
    )?;

    Ok((StatusCode::OK, Json(ApiResponse::success(()))))
}
