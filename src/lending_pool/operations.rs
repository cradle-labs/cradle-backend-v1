use bigdecimal::ToPrimitive;
use diesel::{prelude::*, sql_types};
use std::{str::FromStr, time::Duration};
use tokio::time::sleep;

use crate::{
    accounts::{
        db_types::{
            CradleAccountStatus, CradleAccountType, CradleWalletStatus, CreateCradleAccount,
        },
        operations::{
            associate_token, create_account, grant_access_to_level, kyc_token,
            register_account_wallet,
        },
        processor_enums::{AssociateTokenToWalletInputArgs, GrantKYCInputArgs},
    },
    asset_book::{
        db_types::AssetType,
        operations::{create_asset, get_asset, get_wallet},
        processor_enums::CreateNewAssetInputArgs,
    },
    big_to_u64, extract_option,
    lending_pool::db_types::{
        CreateLendingPoolRecord, CreateLendingPoolSnapShotRecord, CreateLoanRepaymentRecord,
        LendingPoolRecord, LoanRecord, LoanRepaymentsRecord, LoanStatus,
    },
    utils::commons::{DbConn, TaskWallet},
};
use anyhow::{Result, anyhow};
use bigdecimal::BigDecimal;
use contract_integrator::{
    hedera::ContractId,
    id_to_address, id_to_evm_address,
    utils::functions::{
        ContractCallInput, ContractCallOutput,
        asset_lending::{
            AssetLendingPoolFunctionsInput, AssetLendingPoolFunctionsOutput, GetPoolStatsOutput,
            GetUserBorrowPosition, GetUserBorrowPositionOutput, GetUserDepositPositon,
            GetUserDepositPositonOutput,
        },
        asset_lending_pool_factory::{
            AssetLendingPoolFactoryFunctionInput, AssetLendingPoolFactoryFunctionOutput,
            CreatePoolArgs,
        },
        commons::{get_contract_addresses, get_contract_id_from_evm_address},
    },
};
use diesel::r2d2::PooledConnection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateLendingPoolArgs {
    pub reserve_asset: Uuid,
    pub ltv: u64,
    pub optimal_utilization: u64,
    pub base_rate: u64,
    pub slope_1: u64,
    pub slope_2: u64,
    pub liquidation_threshold: u64,
    pub liquidation_discount: u64,
    pub reserve_factor: u64,
    pub name: String,
}

pub struct CreateNewYieldAsset {
    pub name: String,
    pub symbol: String,
    pub decimals: Option<i32>,
    pub icon: Option<String>,
}

pub enum YieldAsset {
    New(CreateNewYieldAsset),
    Existing(Uuid),
}

pub async fn create_lending_pool<'a>(
    conn: DbConn<'a>,
    wallet: TaskWallet<'a>,
    input: CreateLendingPoolArgs,
    yield_asset_input: YieldAsset,
) -> Result<Uuid> {
    let reserve_asset = get_asset(conn, input.reserve_asset).await?;
    let yield_asset_data = match yield_asset_input {
        YieldAsset::Existing(id) => get_asset(conn, id).await?,
        YieldAsset::New(create_args) => {
            let asset_id = create_asset(
                wallet,
                conn,
                CreateNewAssetInputArgs {
                    asset_type: AssetType::YieldBearing,
                    name: create_args.name,
                    symbol: create_args.symbol,
                    decimals: create_args.decimals.unwrap_or(reserve_asset.decimals),
                    icon: create_args.icon.unwrap_or("".to_string()),
                },
            )
            .await?;

            get_asset(conn, asset_id).await?
        }
    };

    let yield_contract_asset_manager =
        get_contract_addresses(&yield_asset_data.asset_manager).await?;

    let tx_instruction = ContractCallInput::AssetLendingPoolFactory(
        AssetLendingPoolFactoryFunctionInput::CreatePool(CreatePoolArgs {
            ltv: input.ltv,
            optimal_utilization: input.optimal_utilization,
            base_rate: input.base_rate,
            slope1: input.slope_1,
            slope2: input.slope_2,
            liquidation_threshold: input.liquidation_threshold,
            liquidation_discount: input.liquidation_discount,
            reserve_factor: input.reserve_factor,
            lending: reserve_asset.token,
            yield_contract: yield_contract_asset_manager,
            lending_pool: input.name.clone(),
        }),
    );

    let tx_res = wallet.execute(tx_instruction).await?;

    let tx_output = match tx_res {
        ContractCallOutput::AssetLendingPoolFactory(
            AssetLendingPoolFactoryFunctionOutput::CreatePool(res),
        ) => res,
        _ => return Err(anyhow!("Failed to create pool")),
    };

    let results = extract_option!(tx_output.output)?;
    let treasury = get_pool_treasury(wallet, results.contract_id.clone()).await?;
    let reserve = get_pool_reserve(wallet, results.contract_id.clone()).await?;

    let pool_account = create_account(
        conn,
        CreateCradleAccount {
            linked_account_id: results.contract_id.clone(),
            account_type: Some(CradleAccountType::System),
            status: Some(CradleAccountStatus::Verified),
        },
    )
    .await?;
    let treasury_wallet = register_account_wallet(
        conn,
        pool_account,
        treasury.clone(),
        Some(CradleWalletStatus::Active),
    )
    .await?;
    let reserve_wallet = register_account_wallet(
        conn,
        pool_account,
        reserve.clone(),
        Some(CradleWalletStatus::Active),
    )
    .await?;

    // associate and kyc both accounts to the reserve asset
    associate_token(
        conn,
        wallet,
        AssociateTokenToWalletInputArgs {
            wallet_id: treasury_wallet,
            token: reserve_asset.id,
        },
    )
    .await?;
    associate_token(
        conn,
        wallet,
        AssociateTokenToWalletInputArgs {
            wallet_id: reserve_wallet,
            token: reserve_asset.id,
        },
    )
    .await?;
    kyc_token(
        conn,
        wallet,
        GrantKYCInputArgs {
            wallet_id: treasury_wallet,
            token: reserve_asset.id,
        },
    )
    .await?;
    kyc_token(
        conn,
        wallet,
        GrantKYCInputArgs {
            wallet_id: reserve_wallet,
            token: reserve_asset.id,
        },
    )
    .await?;

    // grant level 1 access to the pool so that it can initiate yield minting
    grant_access_to_level(wallet, results.address.clone(), 0).await?;
    grant_access_to_level(wallet, results.address.clone(), 1).await?;

    let pool_record = CreateLendingPoolRecord {
        pool_address: results.address,
        pool_contract_id: results.contract_id.to_string(),
        reserve_asset: reserve_asset.id,
        loan_to_value: BigDecimal::from(input.ltv),
        base_rate: BigDecimal::from(input.base_rate),
        slope1: BigDecimal::from(input.slope_1),
        slope2: BigDecimal::from(input.slope_2),
        liquidation_threshold: BigDecimal::from(input.liquidation_threshold),
        liquidation_discount: BigDecimal::from(input.liquidation_discount),
        reserve_factor: BigDecimal::from(input.reserve_factor),
        name: Some(input.name.clone()),
        title: Some(input.name),
        description: None,
        yield_asset: yield_asset_data.id,
        treasury_wallet,
        reserve_wallet,
        pool_account_id: pool_account,
    };

    use crate::schema::lendingpool as lpool;

    let created_id = diesel::insert_into(lpool::table)
        .values(&pool_record)
        .returning(lpool::id)
        .get_result::<Uuid>(conn)?;

    Ok(created_id)
}

pub async fn get_pool_treasury<'a>(wallet: TaskWallet<'a>, contract_id: String) -> Result<String> {
    let tx_input = ContractCallInput::AssetLendingPool(
        AssetLendingPoolFunctionsInput::GetTreasuryAccount(contract_id),
    );

    let tx_res = wallet.execute(tx_input).await?;

    let tx_output = match tx_res {
        ContractCallOutput::AssetLendingPool(
            AssetLendingPoolFunctionsOutput::GetTreasuryAccount(o),
        ) => o,
        _ => return Err(anyhow!("Failed to get pool treasury")),
    };

    let results = extract_option!(tx_output.output)?;

    Ok(results.account)
}

pub async fn get_pool_reserve<'a>(wallet: TaskWallet<'a>, contract_id: String) -> Result<String> {
    let tx_input = ContractCallInput::AssetLendingPool(
        AssetLendingPoolFunctionsInput::GetReserveAccount(contract_id),
    );

    let tx_res = wallet.execute(tx_input).await?;

    let tx_output = match tx_res {
        ContractCallOutput::AssetLendingPool(
            AssetLendingPoolFunctionsOutput::GetReserveAccount(o),
        ) => o,
        _ => return Err(anyhow!("Failed to get pool reserve")),
    };

    let results = extract_option!(tx_output.output)?;

    Ok(results.account)
}

pub async fn get_pool<'a>(conn: DbConn<'a>, pool_id: Uuid) -> Result<LendingPoolRecord> {
    use crate::schema::lendingpool::dsl::*;

    let res = lendingpool
        .filter(id.eq(pool_id))
        .get_result::<LendingPoolRecord>(conn)?;

    Ok(res)
}

pub async fn get_pool_stats<'a>(
    wallet: TaskWallet<'a>,
    conn: DbConn<'a>,
    pool_id: Uuid,
) -> Result<GetPoolStatsOutput> {
    let pool = get_pool(conn, pool_id).await?;
    let tx_instruction = ContractCallInput::AssetLendingPool(
        AssetLendingPoolFunctionsInput::GetPoolStats(pool.pool_contract_id),
    );
    let res = wallet.execute(tx_instruction).await?;

    match res {
        ContractCallOutput::AssetLendingPool(AssetLendingPoolFunctionsOutput::GetPoolStats(o)) => {
            extract_option!(o.output)
        }
        _ => Err(anyhow!("Failed to obtain pool, stats")),
    }
}

pub async fn get_loan<'a>(conn: DbConn<'a>, loan_id: Uuid) -> Result<LoanRecord> {
    use crate::schema::loans::dsl::*;

    let loan_data = loans
        .filter(id.eq(loan_id))
        .get_result::<LoanRecord>(conn)?;

    Ok(loan_data)
}

pub async fn get_loan_position<'a>(
    wallet: TaskWallet<'a>,
    conn: DbConn<'a>,
    loan_id: Uuid,
) -> Result<GetUserBorrowPositionOutput> {
    let loan = get_loan(conn, loan_id).await?;
    let wallet_data = get_wallet(conn, loan.wallet_id).await?;
    let collateral = get_asset(conn, loan.collateral_asset).await?;
    let pool = get_pool(conn, loan.pool).await?;

    let tx_instruction = ContractCallInput::AssetLendingPool(
        AssetLendingPoolFunctionsInput::GetUserBorrowPosition(GetUserBorrowPosition {
            user: wallet_data.address,
            collateral_asset: collateral.token,
            contract_id: pool.pool_contract_id,
        }),
    );

    let res = wallet.execute(tx_instruction).await?;

    match res {
        ContractCallOutput::AssetLendingPool(
            AssetLendingPoolFunctionsOutput::GetUserBorrowPosition(o),
        ) => extract_option!(o.output),
        _ => Err(anyhow!("Failed to retrieve")),
    }
}

pub async fn get_pool_deposit_position<'a>(
    wallet: TaskWallet<'a>,
    conn: DbConn<'a>,
    pool_id: Uuid,
    wallet_id: Uuid,
) -> Result<GetUserDepositPositonOutput> {
    let pool = get_pool(conn, pool_id).await?;
    let wallet_data = get_wallet(conn, wallet_id).await?;

    let tx_instruction = ContractCallInput::AssetLendingPool(
        AssetLendingPoolFunctionsInput::GetUserDepositPosition(GetUserDepositPositon {
            user: wallet_data.address,
            contract_id: pool.pool_contract_id,
        }),
    );

    let res = wallet.execute(tx_instruction).await?;

    match res {
        ContractCallOutput::AssetLendingPool(
            AssetLendingPoolFunctionsOutput::GetUserDepositPosition(o),
        ) => extract_option!(o.output),
        _ => Err(anyhow!("Failed to retrieve")),
    }
}

const REPAYMENT_SQL_QUERY: &str = r"
    select sum(r.repayment_amount) as repaid_amount from loanrepayments as r
    where r.loan_id = $1;
";

#[derive(Serialize, Deserialize, Clone, QueryableByName)]
#[diesel(table_name = crate::schema::loanrepayments)]
pub struct RepaymentAmount {
    #[diesel(sql_type = diesel::sql_types::Numeric)]
    pub repaid_amount: BigDecimal,
}

pub async fn get_repaid_amount<'a>(conn: DbConn<'a>, loan_id: Uuid) -> Result<RepaymentAmount> {
    let result = diesel::sql_query(REPAYMENT_SQL_QUERY)
        .bind::<sql_types::Uuid, _>(loan_id)
        .get_result::<RepaymentAmount>(conn)?;

    Ok(result)
}

pub async fn get_loan_repayments<'a>(
    conn: DbConn<'a>,
    loan_id_value: Uuid,
) -> Result<Vec<LoanRepaymentsRecord>> {
    use crate::schema::loanrepayments::dsl::*;

    let results = loanrepayments
        .filter(loan_id.eq(loan_id_value))
        .get_results::<LoanRepaymentsRecord>(conn)?;

    Ok(results)
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UpdateRepaymentArgs {
    pub loan_id: Uuid,
    pub amount: u64,
    pub transaction: String,
}
pub async fn update_repayment<'a>(conn: DbConn<'a>, args: UpdateRepaymentArgs) -> Result<Uuid> {
    use crate::schema::loanrepayments::table as lptable;
    let loan_data = get_loan(conn, args.loan_id).await?;

    let id = diesel::insert_into(lptable)
        .values(&CreateLoanRepaymentRecord {
            loan_id: args.loan_id,
            repayment_amount: BigDecimal::from(args.amount),
            transaction: args.transaction,
        })
        .returning(crate::schema::loanrepayments::dsl::id)
        .get_result::<Uuid>(conn)?;

    let repaid_amount = get_repaid_amount(conn, args.loan_id).await?;

    let remaining_amount = big_to_u64!(loan_data.principal_amount)? as i64
        - big_to_u64!(repaid_amount.repaid_amount)? as i64;

    let new_status = if remaining_amount <= 0 {
        LoanStatus::Repaid
    } else {
        LoanStatus::Active
    };

    {
        use crate::schema::loans as lps;
        use lps::dsl::*;

        diesel::update(lps::table)
            .filter(id.eq(args.loan_id))
            .set(status.eq(new_status))
            .execute(conn)?;

        0
    };

    Ok(id)
}
