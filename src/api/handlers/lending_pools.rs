use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use diesel::prelude::*;
use serde_json::json;

use crate::{
    lending_pool::{
        processor_enums::{LendingPoolFunctionsInput, LendingPoolFunctionsOutput},
        db_types::{LendingPoolRecord, LoanRecord, LoanRepaymentsRecord, LoanLiquidationsRecord, LoanStatus},
    },
    action_router::{ActionRouterInput, ActionRouterOutput},
    api::{error::ApiError, response::ApiResponse},
    utils::app_config::AppConfig,
};

/// GET /pools/{id} - Get lending pool by UUID
pub async fn get_pool_by_id(
    State(app_config): State<AppConfig>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), ApiError> {
    let pool_id = uuid::Uuid::parse_str(&id)
        .map_err(|_| ApiError::bad_request("Invalid pool ID format"))?;

    let action = ActionRouterInput::Pool(LendingPoolFunctionsInput::GetLendingPool(
        crate::lending_pool::processor_enums::GetLendingPoolInput::ById(pool_id),
    ));

    let result = action
        .process(app_config)
        .await
        .map_err(|_| ApiError::not_found("Lending pool"))?;

    match result {
        ActionRouterOutput::Pool(output) => {
            match output {
                LendingPoolFunctionsOutput::GetLendingPool(pool) => {
                    let json = serde_json::to_value(&pool)
                        .map_err(|e| ApiError::internal_error(format!("Failed to serialize: {}", e)))?;
                    Ok((StatusCode::OK, Json(ApiResponse::success(json))))
                }
                _ => Err(ApiError::internal_error("Unexpected response type")),
            }
        }
        _ => Err(ApiError::internal_error("Unexpected response type")),
    }
}

/// GET /pools/name/{name} - Get lending pool by name
pub async fn get_pool_by_name(
    State(app_config): State<AppConfig>,
    Path(name): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), ApiError> {
    let action = ActionRouterInput::Pool(LendingPoolFunctionsInput::GetLendingPool(
        crate::lending_pool::processor_enums::GetLendingPoolInput::ByName(name),
    ));

    let result = action
        .process(app_config)
        .await
        .map_err(|_| ApiError::not_found("Lending pool"))?;

    match result {
        ActionRouterOutput::Pool(output) => {
            match output {
                LendingPoolFunctionsOutput::GetLendingPool(pool) => {
                    let json = serde_json::to_value(&pool)
                        .map_err(|e| ApiError::internal_error(format!("Failed to serialize: {}", e)))?;
                    Ok((StatusCode::OK, Json(ApiResponse::success(json))))
                }
                _ => Err(ApiError::internal_error("Unexpected response type")),
            }
        }
        _ => Err(ApiError::internal_error("Unexpected response type")),
    }
}

/// GET /pools/address/{address} - Get lending pool by address
pub async fn get_pool_by_address(
    State(app_config): State<AppConfig>,
    Path(address): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), ApiError> {
    let action = ActionRouterInput::Pool(LendingPoolFunctionsInput::GetLendingPool(
        crate::lending_pool::processor_enums::GetLendingPoolInput::ByAddress(address),
    ));

    let result = action
        .process(app_config)
        .await
        .map_err(|_| ApiError::not_found("Lending pool"))?;

    match result {
        ActionRouterOutput::Pool(output) => {
            match output {
                LendingPoolFunctionsOutput::GetLendingPool(pool) => {
                    let json = serde_json::to_value(&pool)
                        .map_err(|e| ApiError::internal_error(format!("Failed to serialize: {}", e)))?;
                    Ok((StatusCode::OK, Json(ApiResponse::success(json))))
                }
                _ => Err(ApiError::internal_error("Unexpected response type")),
            }
        }
        _ => Err(ApiError::internal_error("Unexpected response type")),
    }
}

/// GET /pools/{id}/snapshot - Get latest snapshot for a pool
pub async fn get_pool_snapshot(
    State(app_config): State<AppConfig>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), ApiError> {
    let pool_id = uuid::Uuid::parse_str(&id)
        .map_err(|_| ApiError::bad_request("Invalid pool ID format"))?;

    let create_snapshot_action = ActionRouterInput::Pool(LendingPoolFunctionsInput::CreateSnapShot(pool_id));

    let _ = create_snapshot_action.process(app_config.clone()).await.map_err(|_|ApiError::internal_error("Failed to create snapshot"))?;

    let action = ActionRouterInput::Pool(LendingPoolFunctionsInput::GetSnapShot(pool_id));

    let result = action
        .process(app_config)
        .await
        .map_err(|_| ApiError::not_found("Pool snapshot"))?;

    match result {
        ActionRouterOutput::Pool(output) => {
            match output {
                LendingPoolFunctionsOutput::GetSnapShot(snapshot) => {
                    let json = serde_json::to_value(&snapshot)
                        .map_err(|e| ApiError::internal_error(format!("Failed to serialize: {}", e)))?;
                    Ok((StatusCode::OK, Json(ApiResponse::success(json))))
                }
                _ => Err(ApiError::internal_error("Unexpected response type")),
            }
        }
        _ => Err(ApiError::internal_error("Unexpected response type")),
    }
}

/// GET /pools - Get all lending pools
pub async fn get_pools(
    State(app_config): State<AppConfig>,
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), ApiError> {
    let mut conn = app_config
        .pool
        .get()
        .map_err(|_| ApiError::internal_error("Failed to acquire database connection"))?;

    let results = crate::schema::lendingpool::dsl::lendingpool
        .get_results::<LendingPoolRecord>(&mut conn)
        .map_err(|e| ApiError::internal_error(format!("Database error: {}", e)))?;

    let json = serde_json::to_value(&results)
        .map_err(|e| ApiError::internal_error(format!("Failed to serialize: {}", e)))?;

    Ok((StatusCode::OK, Json(ApiResponse::success(json))))
}

// ============================================================================
// LOAN QUERY HANDLERS
// ============================================================================

/// GET /loans - Get all loans
pub async fn get_all_loans(
    State(app_config): State<AppConfig>,
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), ApiError> {
    let mut conn = app_config
        .pool
        .get()
        .map_err(|_| ApiError::internal_error("Failed to acquire database connection"))?;

    let results = crate::schema::loans::dsl::loans
        .get_results::<LoanRecord>(&mut conn)
        .map_err(|e| ApiError::internal_error(format!("Database error: {}", e)))?;

    let json = serde_json::to_value(&results)
        .map_err(|e| ApiError::internal_error(format!("Failed to serialize: {}", e)))?;

    Ok((StatusCode::OK, Json(ApiResponse::success(json))))
}

/// GET /loans/pool/{id} - Get loans by pool
pub async fn get_loans_by_pool(
    State(app_config): State<AppConfig>,
    Path(pool_id): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), ApiError> {
    let pool_uuid = uuid::Uuid::parse_str(&pool_id)
        .map_err(|_| ApiError::bad_request("Invalid pool ID format"))?;

    let mut conn = app_config
        .pool
        .get()
        .map_err(|_| ApiError::internal_error("Failed to acquire database connection"))?;

    let results = crate::schema::loans::dsl::loans
        .filter(crate::schema::loans::dsl::pool.eq(pool_uuid))
        .get_results::<LoanRecord>(&mut conn)
        .map_err(|e| ApiError::internal_error(format!("Database error: {}", e)))?;

    let json = serde_json::to_value(&results)
        .map_err(|e| ApiError::internal_error(format!("Failed to serialize: {}", e)))?;

    Ok((StatusCode::OK, Json(ApiResponse::success(json))))
}

pub async fn get_loan_by_id(
    State(app_config): State<AppConfig>,
    Path(loan_id): Path<String>
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), ApiError> {
    let loan_uuid = uuid::Uuid::parse_str(&loan_id)
        .map_err(|_| ApiError::bad_request("Invalid pool ID format"))?;

    let mut conn = app_config
        .pool
        .get()
        .map_err(|_| ApiError::internal_error("Failed to acquire database connection"))?;

    let results = crate::schema::loans::dsl::loans
        .filter(crate::schema::loans::dsl::id.eq(&loan_uuid))
        .get_result::<LoanRecord>(&mut conn)
        .map_err(|e| ApiError::internal_error(format!("Database error: {}", e)))?;

    let json = serde_json::to_value(&results)
        .map_err(|e| ApiError::internal_error(format!("Failed to serialize: {}", e)))?;

    Ok((StatusCode::OK, Json(ApiResponse::success(json))))
}

/// GET /loans/wallet/{id} - Get loans by wallet
pub async fn get_loans_by_wallet(
    State(app_config): State<AppConfig>,
    Path(wallet_id): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), ApiError> {
    let wallet_uuid = uuid::Uuid::parse_str(&wallet_id)
        .map_err(|_| ApiError::bad_request("Invalid wallet ID format"))?;

    let mut conn = app_config
        .pool
        .get()
        .map_err(|_| ApiError::internal_error("Failed to acquire database connection"))?;

    let results = crate::schema::loans::dsl::loans
        .filter(crate::schema::loans::dsl::wallet_id.eq(wallet_uuid))
        .get_results::<LoanRecord>(&mut conn)
        .map_err(|e| ApiError::internal_error(format!("Database error: {}", e)))?;

    let json = serde_json::to_value(&results)
        .map_err(|e| ApiError::internal_error(format!("Failed to serialize: {}", e)))?;

    Ok((StatusCode::OK, Json(ApiResponse::success(json))))
}

/// GET /loans/status/{status} - Get loans by status
pub async fn get_loans_by_status(
    State(app_config): State<AppConfig>,
    Path(status_str): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), ApiError> {
    let status = match status_str.to_lowercase().as_str() {
        "active" => LoanStatus::Active,
        "repaid" => LoanStatus::Repaid,
        "liquidated" => LoanStatus::Liquidated,
        _ => return Err(ApiError::bad_request("Invalid loan status. Use: active, repaid, or liquidated")),
    };

    let mut conn = app_config
        .pool
        .get()
        .map_err(|_| ApiError::internal_error("Failed to acquire database connection"))?;

    let results = crate::schema::loans::dsl::loans
        .filter(crate::schema::loans::dsl::status.eq(status))
        .get_results::<LoanRecord>(&mut conn)
        .map_err(|e| ApiError::internal_error(format!("Database error: {}", e)))?;

    let json = serde_json::to_value(&results)
        .map_err(|e| ApiError::internal_error(format!("Failed to serialize: {}", e)))?;

    Ok((StatusCode::OK, Json(ApiResponse::success(json))))
}

// ============================================================================
// LOAN REPAYMENT QUERY HANDLERS
// ============================================================================

/// GET /loan-repayments - Get all loan repayments
pub async fn get_all_repayments(
    State(app_config): State<AppConfig>,
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), ApiError> {
    let mut conn = app_config
        .pool
        .get()
        .map_err(|_| ApiError::internal_error("Failed to acquire database connection"))?;

    let results = crate::schema::loanrepayments::dsl::loanrepayments
        .order(crate::schema::loanrepayments::dsl::repayment_date.desc())
        .get_results::<LoanRepaymentsRecord>(&mut conn)
        .map_err(|e| ApiError::internal_error(format!("Database error: {}", e)))?;

    let json = serde_json::to_value(&results)
        .map_err(|e| ApiError::internal_error(format!("Failed to serialize: {}", e)))?;

    Ok((StatusCode::OK, Json(ApiResponse::success(json))))
}

/// GET /loan-repayments/loan/{id} - Get repayments by loan
pub async fn get_repayments_by_loan(
    State(app_config): State<AppConfig>,
    Path(loan_id): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), ApiError> {
    let loan_uuid = uuid::Uuid::parse_str(&loan_id)
        .map_err(|_| ApiError::bad_request("Invalid loan ID format"))?;

    let mut conn = app_config
        .pool
        .get()
        .map_err(|_| ApiError::internal_error("Failed to acquire database connection"))?;

    let results = crate::schema::loanrepayments::dsl::loanrepayments
        .filter(crate::schema::loanrepayments::dsl::loan_id.eq(loan_uuid))
        .order(crate::schema::loanrepayments::dsl::repayment_date.desc())
        .get_results::<LoanRepaymentsRecord>(&mut conn)
        .map_err(|e| ApiError::internal_error(format!("Database error: {}", e)))?;

    let json = serde_json::to_value(&results)
        .map_err(|e| ApiError::internal_error(format!("Failed to serialize: {}", e)))?;

    Ok((StatusCode::OK, Json(ApiResponse::success(json))))
}

// ============================================================================
// LOAN LIQUIDATION QUERY HANDLERS
// ============================================================================

/// GET /loan-liquidations - Get all loan liquidations
pub async fn get_all_liquidations(
    State(app_config): State<AppConfig>,
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), ApiError> {
    let mut conn = app_config
        .pool
        .get()
        .map_err(|_| ApiError::internal_error("Failed to acquire database connection"))?;

    let results = crate::schema::loanliquidations::dsl::loanliquidations
        .order(crate::schema::loanliquidations::dsl::liquidation_date.desc())
        .get_results::<LoanLiquidationsRecord>(&mut conn)
        .map_err(|e| ApiError::internal_error(format!("Database error: {}", e)))?;

    let json = serde_json::to_value(&results)
        .map_err(|e| ApiError::internal_error(format!("Failed to serialize: {}", e)))?;

    Ok((StatusCode::OK, Json(ApiResponse::success(json))))
}

/// GET /loan-liquidations/loan/{id} - Get liquidations by loan
pub async fn get_liquidations_by_loan(
    State(app_config): State<AppConfig>,
    Path(loan_id): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), ApiError> {
    let loan_uuid = uuid::Uuid::parse_str(&loan_id)
        .map_err(|_| ApiError::bad_request("Invalid loan ID format"))?;

    let mut conn = app_config
        .pool
        .get()
        .map_err(|_| ApiError::internal_error("Failed to acquire database connection"))?;

    let results = crate::schema::loanliquidations::dsl::loanliquidations
        .filter(crate::schema::loanliquidations::dsl::loan_id.eq(loan_uuid))
        .order(crate::schema::loanliquidations::dsl::liquidation_date.desc())
        .get_results::<LoanLiquidationsRecord>(&mut conn)
        .map_err(|e| ApiError::internal_error(format!("Database error: {}", e)))?;

    let json = serde_json::to_value(&results)
        .map_err(|e| ApiError::internal_error(format!("Failed to serialize: {}", e)))?;

    Ok((StatusCode::OK, Json(ApiResponse::success(json))))
}

// ============================================================================
// ASSET LENDING POOL CONTRACT GETTER HANDLERS
// ============================================================================

/// GET /pools/{id}/interest-rates - Get pool interest rate configuration
pub async fn get_interest_rates(
    State(app_config): State<AppConfig>,
    Path(pool_id): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), ApiError> {
    let pool_uuid = uuid::Uuid::parse_str(&pool_id)
        .map_err(|_| ApiError::bad_request("Invalid pool ID format"))?;

    let mut conn = app_config
        .pool
        .get()
        .map_err(|_| ApiError::internal_error("Failed to acquire database connection"))?;

    let pool = crate::schema::lendingpool::dsl::lendingpool
        .filter(crate::schema::lendingpool::dsl::id.eq(pool_uuid))
        .first::<LendingPoolRecord>(&mut conn)
        .map_err(|_| ApiError::not_found("Lending pool"))?;

    let json = json!({
        "pool_id": pool.id,
        "base_rate": pool.base_rate,
        "slope1": pool.slope1,
        "slope2": pool.slope2,
        "reserve_factor": pool.reserve_factor,
        "interest_rate_model": {
            "description": "Two-slope interest rate model",
            "slope1_threshold": "Kink point where slope changes",
            "slope1_rate": pool.slope1,
            "slope2_rate": pool.slope2,
        }
    });

    Ok((StatusCode::OK, Json(ApiResponse::success(json))))
}

/// GET /pools/{id}/collateral-info - Get pool collateral configuration
pub async fn get_collateral_info(
    State(app_config): State<AppConfig>,
    Path(pool_id): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), ApiError> {
    let pool_uuid = uuid::Uuid::parse_str(&pool_id)
        .map_err(|_| ApiError::bad_request("Invalid pool ID format"))?;

    let mut conn = app_config
        .pool
        .get()
        .map_err(|_| ApiError::internal_error("Failed to acquire database connection"))?;

    let pool = crate::schema::lendingpool::dsl::lendingpool
        .filter(crate::schema::lendingpool::dsl::id.eq(pool_uuid))
        .first::<LendingPoolRecord>(&mut conn)
        .map_err(|_| ApiError::not_found("Lending pool"))?;

    let json = json!({
        "pool_id": pool.id,
        "loan_to_value": pool.loan_to_value,
        "liquidation_threshold": pool.liquidation_threshold,
        "liquidation_discount": pool.liquidation_discount,
        "risk_parameters": {
            "ltv": pool.loan_to_value,
            "liquidation_threshold": pool.liquidation_threshold,
            "liquidation_penalty": pool.liquidation_discount,
        }
    });

    Ok((StatusCode::OK, Json(ApiResponse::success(json))))
}

/// GET /pools/{id}/pool-stats - Get pool statistics and metrics
pub async fn get_pool_stats(
    State(app_config): State<AppConfig>,
    Path(pool_id): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), ApiError> {
    let pool_uuid = uuid::Uuid::parse_str(&pool_id)
        .map_err(|_| ApiError::bad_request("Invalid pool ID format"))?;

    let mut conn = app_config
        .pool
        .get()
        .map_err(|_| ApiError::internal_error("Failed to acquire database connection"))?;

    // Get pool configuration
    let pool = crate::schema::lendingpool::dsl::lendingpool
        .filter(crate::schema::lendingpool::dsl::id.eq(pool_uuid))
        .first::<LendingPoolRecord>(&mut conn)
        .map_err(|_| ApiError::not_found("Lending pool"))?;

    // Get latest snapshot with pool metrics
    let snapshot = crate::schema::lendingpoolsnapshots::dsl::lendingpoolsnapshots
        .filter(crate::schema::lendingpoolsnapshots::dsl::lending_pool_id.eq(pool_uuid))
        .order(crate::schema::lendingpoolsnapshots::dsl::created_at.desc())
        .first::<crate::lending_pool::db_types::LendingPoolSnapShotRecord>(&mut conn)
        .ok();

    let json = if let Some(snap) = snapshot {
        json!({
            "pool_id": pool.id,
            "pool_name": pool.name,
            "pool_address": pool.pool_address,
            "reserve_asset": pool.reserve_asset,
            "metrics": {
                "total_supply": snap.total_supply,
                "total_borrow": snap.total_borrow,
                "available_liquidity": snap.available_liquidity,
                "utilization_rate": snap.utilization_rate,
                "supply_apy": snap.supply_apy,
                "borrow_apy": snap.borrow_apy,
            },
            "last_updated": snap.created_at,
            "rate_configuration": {
                "base_rate": pool.base_rate,
                "slope1": pool.slope1,
                "slope2": pool.slope2,
            }
        })
    } else {
        json!({
            "pool_id": pool.id,
            "pool_name": pool.name,
            "pool_address": pool.pool_address,
            "reserve_asset": pool.reserve_asset,
            "metrics": {
                "total_supply": null,
                "total_borrow": null,
                "available_liquidity": null,
                "utilization_rate": null,
                "supply_apy": null,
                "borrow_apy": null,
            },
            "note": "No snapshots available yet",
            "rate_configuration": {
                "base_rate": pool.base_rate,
                "slope1": pool.slope1,
                "slope2": pool.slope2,
            }
        })
    };

    Ok((StatusCode::OK, Json(ApiResponse::success(json))))
}

/// GET /pools/{pool_id}/user-positions/{wallet_id} - Get user's position in a pool
pub async fn get_user_positions(
    State(app_config): State<AppConfig>,
    Path((pool_id, wallet_id)): Path<(String, String)>,
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), ApiError> {
    let pool_uuid = uuid::Uuid::parse_str(&pool_id)
        .map_err(|_| ApiError::bad_request("Invalid pool ID format"))?;

    let wallet_uuid = uuid::Uuid::parse_str(&wallet_id)
        .map_err(|_| ApiError::bad_request("Invalid wallet ID format"))?;

    let mut conn = app_config
        .pool
        .get()
        .map_err(|_| ApiError::internal_error("Failed to acquire database connection"))?;

    // Verify pool exists
    let _pool = crate::schema::lendingpool::dsl::lendingpool
        .filter(crate::schema::lendingpool::dsl::id.eq(pool_uuid))
        .first::<LendingPoolRecord>(&mut conn)
        .map_err(|_| ApiError::not_found("Lending pool"))?;

    // Get user's loans in this pool
    let loans = crate::schema::loans::dsl::loans
        .filter(crate::schema::loans::dsl::wallet_id.eq(wallet_uuid))
        .filter(crate::schema::loans::dsl::pool.eq(pool_uuid))
        .get_results::<LoanRecord>(&mut conn)
        .map_err(|e| ApiError::internal_error(format!("Database error: {}", e)))?;

    // Calculate total borrow from active loans
    let total_borrow = loans
        .iter()
        .filter(|loan| matches!(loan.status, LoanStatus::Active))
        .fold(bigdecimal::BigDecimal::from(0), |acc, loan| {
            acc + loan.principal_amount.clone()
        });

    // Get repayments for this wallet in this pool
    let repayments = crate::schema::loanrepayments::dsl::loanrepayments
        .filter(
            crate::schema::loanrepayments::dsl::loan_id.eq_any(
                loans.iter().map(|l| l.id).collect::<Vec<_>>()
            )
        )
        .get_results::<LoanRepaymentsRecord>(&mut conn)
        .map_err(|e| ApiError::internal_error(format!("Database error: {}", e)))?;

    let total_repayments = repayments
        .iter()
        .fold(bigdecimal::BigDecimal::from(0), |acc, rep| {
            acc + rep.repayment_amount.clone()
        });

    let json = json!({
        "pool_id": pool_uuid,
        "wallet_id": wallet_uuid,
        "borrow_position": {
            "active_loans_count": loans.iter().filter(|l| matches!(l.status, LoanStatus::Active)).count(),
            "total_borrow_amount": total_borrow,
            "loans": loans.iter().map(|loan| {
                json!({
                    "loan_id": loan.id,
                    "principal_amount": loan.principal_amount,
                    "status": format!("{:?}", loan.status),
                    "created_at": loan.created_at,
                })
            }).collect::<Vec<_>>(),
        },
        "repayment_history": {
            "total_repaid": total_repayments,
            "repayment_count": repayments.len(),
            "recent_repayments": repayments.iter().rev().take(5).map(|rep| {
                json!({
                    "repayment_amount": rep.repayment_amount,
                    "repayment_date": rep.repayment_date,
                })
            }).collect::<Vec<_>>(),
        }
    });

    Ok((StatusCode::OK, Json(ApiResponse::success(json))))
}
