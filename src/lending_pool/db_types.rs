use anyhow::Result;
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::{
    ExpressionMethods, Identifiable, Insertable, PgConnection, Queryable, QueryableByName,
    RunQueryDsl, Selectable,
};
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug, QueryableByName, Queryable, Identifiable)]
#[diesel(table_name = crate::schema::lendingpool)]
pub struct LendingPoolRecord {
    pub id: Uuid,
    pub pool_address: String,
    pub pool_contract_id: String,
    pub reserve_asset: Uuid,
    pub loan_to_value: BigDecimal,
    pub base_rate: BigDecimal,
    pub slope1: BigDecimal,
    pub slope2: BigDecimal,
    pub liquidation_threshold: BigDecimal,
    pub liquidation_discount: BigDecimal,
    pub reserve_factor: BigDecimal,
    pub name: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub yield_asset: Uuid,
}

impl LendingPoolRecord {
    pub fn get(
        conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
        value_id: Uuid,
    ) -> Result<Self> {
        use crate::schema::lendingpool::dsl::*;

        let value = crate::schema::lendingpool::dsl::lendingpool
            .filter(id.eq(value_id))
            .get_result::<Self>(conn)?;

        Ok(value)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Insertable)]
#[diesel(table_name = crate::schema::lendingpool)]
pub struct CreateLendingPoolRecord {
    pub pool_address: String,
    pub pool_contract_id: String,
    pub reserve_asset: Uuid,
    pub loan_to_value: BigDecimal,
    pub base_rate: BigDecimal,
    pub slope1: BigDecimal,
    pub slope2: BigDecimal,
    pub liquidation_threshold: BigDecimal,
    pub liquidation_discount: BigDecimal,
    pub reserve_factor: BigDecimal,
    pub name: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub yield_asset: Uuid,
}

#[derive(
    Serialize, Deserialize, Clone, Debug, Queryable, QueryableByName, Selectable, Identifiable,
)]
#[diesel(table_name = crate::schema::lendingpoolsnapshots)]
pub struct LendingPoolSnapShotRecord {
    pub id: Uuid,
    pub lending_pool_id: Uuid,
    pub total_supply: BigDecimal,
    pub total_borrow: BigDecimal,
    pub available_liquidity: BigDecimal,
    pub utilization_rate: BigDecimal,
    pub supply_apy: BigDecimal,
    pub borrow_apy: BigDecimal,
    pub created_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Clone, Debug, Insertable)]
#[diesel(table_name = crate::schema::lendingpoolsnapshots)]
pub struct CreateLendingPoolSnapShotRecord {
    pub lending_pool_id: Uuid,
    pub total_supply: BigDecimal,
    pub total_borrow: BigDecimal,
    pub available_liquidity: BigDecimal,
    pub utilization_rate: BigDecimal,
    pub supply_apy: BigDecimal,
    pub borrow_apy: BigDecimal,
}

// Loans
#[derive(Serialize, Deserialize, Clone, Debug, DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::LoanStatus"]
#[serde(rename_all = "lowercase")]
pub enum LoanStatus {
    Active,
    Repaid,
    Liquidated,
}

#[derive(Serialize, Deserialize, Clone, Debug, Queryable, Identifiable, QueryableByName)]
#[diesel(table_name = crate::schema::loans)]
pub struct LoanRecord {
    pub id: Uuid,
    pub account_id: Uuid,
    pub wallet_id: Uuid,
    pub pool: Uuid,
    pub borrow_index: BigDecimal,
    pub principal_amount: BigDecimal,
    pub created_at: NaiveDateTime,
    pub status: LoanStatus,
    pub transaction: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Insertable)]
#[diesel(table_name = crate::schema::loans)]
pub struct CreateLoanRecord {
    pub account_id: Uuid,
    pub wallet_id: Uuid,
    pub pool: Uuid,
    pub borrow_index: BigDecimal,
    pub principal_amount: BigDecimal,
    pub status: LoanStatus,
    pub transaction: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Queryable, Identifiable, QueryableByName)]
#[diesel(table_name = crate::schema::loanrepayments)]
pub struct LoanRepaymentsRecord {
    pub id: Uuid,
    pub loan_id: Uuid,
    pub repayment_amount: BigDecimal,
    pub repayment_date: NaiveDateTime,
    pub transaction: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Insertable)]
#[diesel(table_name = crate::schema::loanrepayments)]
pub struct CreateLoanRepaymentRecord {
    pub loan_id: Uuid,
    pub repayment_amount: BigDecimal,
    pub transaction: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Queryable, Identifiable, QueryableByName)]
#[diesel(table_name = crate::schema::loanliquidations)]
pub struct LoanLiquidationsRecord {
    pub id: Uuid,
    pub loan_id: Uuid,
    pub liquidator_wallet_id: Uuid,
    pub liquidation_amount: BigDecimal,
    pub liquidation_date: NaiveDateTime,
    pub transaction: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Insertable)]
#[diesel(table_name = crate::schema::loanliquidations)]
pub struct CreateLoanLiquidationRecord {
    pub loan_id: Uuid,
    pub liquidator_wallet_id: Uuid,
    pub liquidation_amount: BigDecimal,
    pub transaction: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::PoolTransactionType"]
#[serde(rename_all = "lowercase")]
pub enum PoolTransactionType {
    Supply,
    Withdraw,
}

#[derive(Serialize, Deserialize, Debug, Clone, Queryable, Identifiable, QueryableByName)]
#[diesel(table_name = crate::schema::pooltransactions)]
pub struct PoolTransactionRecord {
    pub id: Uuid,
    pub pool_id: Uuid,
    pub wallet_id: Uuid,
    pub amount: BigDecimal,
    pub supply_index: BigDecimal,
    pub transaction_type: PoolTransactionType,
    pub yield_token_amount: BigDecimal,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub transaction: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Insertable)]
#[diesel(table_name = crate::schema::pooltransactions)]
pub struct CreatePoolTransactionRecord {
    pub pool_id: Uuid,
    pub wallet_id: Uuid,
    pub amount: BigDecimal,
    pub supply_index: BigDecimal,
    pub transaction_type: PoolTransactionType,
    pub yield_token_amount: BigDecimal,
    pub transaction: String,
}
