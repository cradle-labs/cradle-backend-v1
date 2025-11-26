use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use contract_integrator::utils::functions::asset_lending::{
    GetPoolStatsOutput, GetUserBorrowPosition, GetUserBorrowPositionOutput,
    GetUserDepositPositonOutput,
};
use diesel::prelude::*;
use serde_json::json;

use crate::{
    action_router::{ActionRouterInput, ActionRouterOutput},
    api::{error::ApiError, response::ApiResponse},
    lending_pool::{
        db_types::{
            LendingPoolRecord, LoanLiquidationsRecord, LoanRecord, LoanRepaymentsRecord, LoanStatus,
        },
        operations::{
            RepaymentAmount, get_loan_position, get_loan_repayments, get_pool_deposit_position,
            get_pool_stats, get_repaid_amount,
        },
        processor_enums::{LendingPoolFunctionsInput, LendingPoolFunctionsOutput},
    },
    map_to_api_error,
    schema::lendingpoolsnapshots::lending_pool_id,
    utils::app_config::AppConfig,
};
use uuid::Uuid;

pub async fn get_pools(
    State(app_config): State<AppConfig>,
) -> Result<(StatusCode, Json<ApiResponse<Vec<LendingPoolRecord>>>), ApiError> {
    let mut conn = map_to_api_error!(app_config.pool.get(), "Failed to acquire db conn")?;

    let results = map_to_api_error!(
        {
            use crate::schema::lendingpool::dsl::*;

            lendingpool.get_results::<LendingPoolRecord>(&mut conn)
        },
        "Failed to get lending pool"
    )?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            data: Some(results),
            error: None,
        }),
    ))
}

pub async fn get_pool(
    State(app_config): State<AppConfig>,
    Path(id_value): Path<Uuid>,
) -> Result<(StatusCode, Json<ApiResponse<LendingPoolRecord>>), ApiError> {
    let mut conn = map_to_api_error!(app_config.pool.get(), "Failed to acquire db conn")?;

    let result = map_to_api_error!(
        {
            use crate::schema::lendingpool::dsl::*;

            lendingpool
                .filter(id.eq(id_value))
                .get_result::<LendingPoolRecord>(&mut conn)
        },
        "Failed to get pool"
    )?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            data: Some(result),
            error: None,
        }),
    ))
}

pub async fn get_loans_handler(
    State(app_config): State<AppConfig>,
    Path(wallet_id_value): Path<Uuid>,
) -> Result<(StatusCode, Json<ApiResponse<Vec<LoanRecord>>>), ApiError> {
    let mut conn = map_to_api_error!(app_config.pool.get(), "Failed to acquire db conn")?;
    let mut wallet = app_config.wallet.clone();

    let loans = map_to_api_error!(
        {
            use crate::schema::loans::dsl::*;

            loans
                .filter(wallet_id.eq(wallet_id_value))
                .get_results::<LoanRecord>(&mut conn)
        },
        "Failed to retrieve loans"
    )?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            success: false,
            data: Some(loans),
            error: None,
        }),
    ))
}

// TODO: add a caching layer
pub async fn get_pool_stats_handler(
    State(app_config): State<AppConfig>,
    Path(pool_id): Path<Uuid>,
) -> Result<(StatusCode, Json<ApiResponse<GetPoolStatsOutput>>), ApiError> {
    let mut conn = map_to_api_error!(app_config.pool.get(), "Failed to acquire db conn")?;
    let mut wallet = app_config.wallet.clone();

    let results = map_to_api_error!(
        get_pool_stats(&mut wallet, &mut conn, pool_id).await,
        "Failed to get stats"
    )?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            data: Some(results),
            error: None,
        }),
    ))
}

// TODO: add a caching layer
pub async fn get_pool_borrow_positions(
    State(app_config): State<AppConfig>,
    Path(loan_id): Path<Uuid>,
) -> Result<(StatusCode, Json<ApiResponse<GetUserBorrowPositionOutput>>), ApiError> {
    let mut conn = map_to_api_error!(app_config.pool.get(), "Failed to acquire db conn")?;
    let mut wallet = app_config.wallet.clone();

    let results = map_to_api_error!(
        get_loan_position(&mut wallet, &mut conn, loan_id).await,
        "Failed to get loan"
    )?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            data: Some(results),
            error: None,
        }),
    ))
}

// TODO: add a caching layer
pub async fn get_pool_deposit_handler(
    State(app_config): State<AppConfig>,
    Path((pool_id, wallet_id)): Path<(Uuid, Uuid)>,
) -> Result<(StatusCode, Json<ApiResponse<GetUserDepositPositonOutput>>), ApiError> {
    let mut conn = map_to_api_error!(app_config.pool.get(), "Failed to acquire db conn")?;
    let mut wallet = app_config.wallet.clone();

    let results = map_to_api_error!(
        get_pool_deposit_position(&mut wallet, &mut conn, pool_id, wallet_id).await,
        "Failed to get loan"
    )?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            data: Some(results),
            error: None,
        }),
    ))
}

pub async fn get_loan_repayments_handler(
    State(app_config): State<AppConfig>,
    Path(loan_id): Path<Uuid>,
) -> Result<(StatusCode, Json<ApiResponse<Vec<LoanRepaymentsRecord>>>), ApiError> {
    let mut conn = map_to_api_error!(app_config.pool.get(), "Failed to acquire db conn")?;

    let results = map_to_api_error!(
        get_loan_repayments(&mut conn, loan_id).await,
        "Failed to get loan repayments"
    )?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            data: Some(results),
            error: None,
        }),
    ))
}

pub async fn get_repaid_handler(
    State(app_config): State<AppConfig>,
    Path(loan_id): Path<Uuid>,
) -> Result<(StatusCode, Json<ApiResponse<RepaymentAmount>>), ApiError> {
    let mut conn = map_to_api_error!(app_config.pool.get(), "Failed to acquire db conn")?;

    let results = map_to_api_error!(
        get_repaid_amount(&mut conn, loan_id).await,
        "Failed to get loan repayments"
    )?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            data: Some(results),
            error: None,
        }),
    ))
}
