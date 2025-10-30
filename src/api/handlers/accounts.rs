use anyhow::anyhow;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::{
    accounts::processor_enums::{AccountsProcessorInput, AccountsProcessorOutput, GetAccountInputArgs, GetWalletInputArgs},
    action_router::{ActionRouterInput, ActionRouterOutput},
    api::{error::ApiError, response::ApiResponse},
    utils::app_config::AppConfig,
};

/// GET /accounts/{id} - Get account by UUID
pub async fn get_account_by_id(
    State(app_config): State<AppConfig>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), ApiError> {
    let account_id = uuid::Uuid::parse_str(&id)
        .map_err(|_| ApiError::bad_request("Invalid account ID format"))?;

    let action = ActionRouterInput::Accounts(AccountsProcessorInput::GetAccount(
        GetAccountInputArgs::ByID(account_id),
    ));

    let result = action
        .process(app_config)
        .await
        .map_err(|_| ApiError::not_found("Account"))?;

    match result {
        ActionRouterOutput::Accounts(output) => {
            match output {
                AccountsProcessorOutput::GetAccount(account) => {
                    let json = serde_json::to_value(&account)
                        .map_err(|e| ApiError::internal_error(format!("Failed to serialize: {}", e)))?;
                    Ok((StatusCode::OK, Json(ApiResponse::success(json))))
                }
                _ => Err(ApiError::internal_error("Unexpected response type")),
            }
        }
        _ => Err(ApiError::internal_error("Unexpected response type")),
    }
}

/// GET /accounts/linked/{linked_id} - Get account by linked account ID
pub async fn get_account_by_linked_id(
    State(app_config): State<AppConfig>,
    Path(linked_id): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), ApiError> {
    let action = ActionRouterInput::Accounts(AccountsProcessorInput::GetAccount(
        GetAccountInputArgs::ByLinkedAccount(linked_id),
    ));

    let result = action
        .process(app_config)
        .await
        .map_err(|_| ApiError::not_found("Account"))?;

    match result {
        ActionRouterOutput::Accounts(output) => {
            match output {
                AccountsProcessorOutput::GetAccount(account) => {
                    let json = serde_json::to_value(&account)
                        .map_err(|e| ApiError::internal_error(format!("Failed to serialize: {}", e)))?;
                    Ok((StatusCode::OK, Json(ApiResponse::success(json))))
                }
                _ => Err(ApiError::internal_error("Unexpected response type")),
            }
        }
        _ => Err(ApiError::internal_error("Unexpected response type")),
    }
}

/// GET /accounts/{account_id}/wallets - Get wallets for account (not implemented)
pub async fn get_account_wallets(
    State(app_config): State<AppConfig>,
    Path(_account_id): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), ApiError> {
    let action = ActionRouterInput::Accounts(
        AccountsProcessorInput::GetWallet(
            GetWalletInputArgs::ByCradleAccount(_account_id.parse().map_err(|_|ApiError::internal_error("Unable to convert account id"))?)
        )
    );

    let result = action
        .process(app_config)
        .await
        .map_err(|_| ApiError::not_found("Account"))?;

    match result {
        ActionRouterOutput::Accounts(output) => {
            match output {
                AccountsProcessorOutput::GetWallet(account) => {
                    let json = serde_json::to_value(&account)
                        .map_err(|e| ApiError::internal_error(format!("Failed to serialize: {}", e)))?;
                    Ok((StatusCode::OK, Json(ApiResponse::success(json))))
                }
                _ => Err(ApiError::internal_error("Unexpected response type")),
            }
        }
        _ => Err(ApiError::internal_error("Unexpected response type")),
    }
}

/// GET /wallets/{id} - Get wallet by UUID
pub async fn get_wallet_by_id(
    State(app_config): State<AppConfig>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), ApiError> {
    let wallet_id = uuid::Uuid::parse_str(&id)
        .map_err(|_| ApiError::bad_request("Invalid wallet ID format"))?;

    let action = ActionRouterInput::Accounts(AccountsProcessorInput::GetWallet(
        GetWalletInputArgs::ById(wallet_id),
    ));

    let result = action
        .process(app_config)
        .await
        .map_err(|_| ApiError::not_found("Wallet"))?;

    match result {
        ActionRouterOutput::Accounts(output) => {
            match output {
                AccountsProcessorOutput::GetWallet(wallet) => {
                    let json = serde_json::to_value(&wallet)
                        .map_err(|e| ApiError::internal_error(format!("Failed to serialize: {}", e)))?;
                    Ok((StatusCode::OK, Json(ApiResponse::success(json))))
                }
                _ => Err(ApiError::internal_error("Unexpected response type")),
            }
        }
        _ => Err(ApiError::internal_error("Unexpected response type")),
    }
}

/// GET /wallets/account/{account_id} - Get wallet by account ID
pub async fn get_wallet_by_account_id(
    State(app_config): State<AppConfig>,
    Path(account_id): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), ApiError> {
    let acc_id = uuid::Uuid::parse_str(&account_id)
        .map_err(|_| ApiError::bad_request("Invalid account ID format"))?;

    let action = ActionRouterInput::Accounts(AccountsProcessorInput::GetWallet(
        GetWalletInputArgs::ByCradleAccount(acc_id),
    ));

    let result = action
        .process(app_config)
        .await
        .map_err(|_| ApiError::not_found("Wallet"))?;

    match result {
        ActionRouterOutput::Accounts(output) => {
            match output {
                AccountsProcessorOutput::GetWallet(wallet) => {
                    let json = serde_json::to_value(&wallet)
                        .map_err(|e| ApiError::internal_error(format!("Failed to serialize: {}", e)))?;
                    Ok((StatusCode::OK, Json(ApiResponse::success(json))))
                }
                _ => Err(ApiError::internal_error("Unexpected response type")),
            }
        }
        _ => Err(ApiError::internal_error("Unexpected response type")),
    }
}
