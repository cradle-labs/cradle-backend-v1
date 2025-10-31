use anyhow::anyhow;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use bigdecimal::{BigDecimal, FromPrimitive, ToPrimitive};
use contract_integrator::utils::functions::commons::get_account_balances;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
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
    State(_app_config): State<AppConfig>,
    Path(_account_id): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), ApiError> {
    Err(ApiError::internal_error(
        "GET /accounts/{account_id}/wallets endpoint not yet implemented",
    ))
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

pub async fn api_get_account_balances(
    State(app_state): State<AppConfig>,
    Path(wallet_id): Path<String>
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), ApiError> {

    #[derive(Serialize, Deserialize)]
    struct Balance {
        pub token: String,
        pub balance: BigDecimal
    }

    let mut  all_balances: Vec<Balance > = vec![];

    let data = get_account_balances(&app_state.wallet.client, wallet_id.as_str()).await.map_err(|_|ApiError::internal_error("Failed to fetch balances "))?;

    let v = data.hbars.get_value().to_i64().unwrap_or(0);

    all_balances.push(Balance {
        token: "HBAR".to_string(),
        balance: BigDecimal::from(v)
    });

    for (token, balance) in data.tokens {
        all_balances.push(Balance {
            token: token.to_string(),
            balance: BigDecimal::from(balance)
        })
    }



    let data_value = serde_json::to_value(&all_balances).unwrap_or(serde_json::to_value::<Vec<Balance>>(Vec::new()).map_err(|_|ApiError::internal_error("Unable to get data"))?);
    Ok((StatusCode::OK, Json(ApiResponse::success(json!(data_value)))))
}