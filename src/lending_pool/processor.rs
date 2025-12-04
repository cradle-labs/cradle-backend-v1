use crate::accounts::db_types::CradleWalletAccountRecord;
use crate::accounts::operations::{associate_token, kyc_token};
use crate::accounts::processor_enums::{AssociateTokenToWalletInputArgs, GrantKYCInputArgs};
use crate::accounts_ledger::operations::{
    BorrowAssets, Deposit, LiquidateLoan, RecordTransactionAssets, Withdraw, record_transaction,
};
use crate::asset_book::db_types::AssetBookRecord;
use crate::lending_pool::config::LendingPoolConfig;
use crate::lending_pool::db_types::{
    CreateLendingPoolSnapShotRecord, CreateLoanRecord, CreatePoolTransactionRecord,
    LendingPoolRecord, LendingPoolSnapShotRecord, LoanStatus, PoolTransactionType,
};
use crate::lending_pool::operations::{UpdateRepaymentArgs, update_repayment};
use crate::lending_pool::processor_enums::{
    GetLendingPoolInput, LendingPoolFunctionsInput, LendingPoolFunctionsOutput,
};
use crate::schema::accountassetbook::dsl::accountassetbook;
use crate::schema::asset_book::dsl::asset_book;
use crate::schema::cradlewalletaccounts::dsl::cradlewalletaccounts;
use crate::utils::app_config::AppConfig;
use crate::utils::traits::ActionProcessor;
use anyhow::anyhow;
use bigdecimal::BigDecimal;
use contract_integrator::utils::functions::asset_lending::{
    AssetLendingPoolFunctionsInput, AssetLendingPoolFunctionsOutput, BorrowArgs, DepositArgs,
    WithdrawArgs,
};
use contract_integrator::utils::functions::{ContractCallInput, ContractCallOutput};
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::{AggregateExpressionMethods, ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};
use uuid::Uuid;

impl ActionProcessor<LendingPoolConfig, LendingPoolFunctionsOutput> for LendingPoolFunctionsInput {
    async fn process(
        &self,
        app_config: &mut AppConfig,
        local_config: &mut LendingPoolConfig,
        conn: Option<&mut PooledConnection<ConnectionManager<PgConnection>>>,
    ) -> anyhow::Result<LendingPoolFunctionsOutput> {
        let app_conn = conn.ok_or_else(|| anyhow!("No database connection available"))?;

        match self {
            LendingPoolFunctionsInput::CreateLendingPool(args) => {
                let res = diesel::insert_into(crate::schema::lendingpool::table)
                    .values(args)
                    .returning(crate::schema::lendingpool::dsl::id)
                    .get_result::<Uuid>(app_conn)?;
                Ok(LendingPoolFunctionsOutput::CreateLendingPool(res))
            }
            LendingPoolFunctionsInput::GetLendingPool(filters) => {
                use crate::schema::lendingpool::dsl::*;
                let mut query = lendingpool.into_boxed();
                match filters {
                    GetLendingPoolInput::ByName(name_filter) => {
                        query = query.filter(name.eq(name_filter));
                    }
                    GetLendingPoolInput::ByAddress(address_filter) => {
                        query = query.filter(pool_address.eq(address_filter))
                    }
                    GetLendingPoolInput::ById(id_filter) => query = query.filter(id.eq(id_filter)),
                };
                let res = query.first::<LendingPoolRecord>(app_conn)?;
                Ok(LendingPoolFunctionsOutput::GetLendingPool(res))
            }
            LendingPoolFunctionsInput::CreateSnapShot(pool_id_value) => {
                let pool = LendingPoolRecord::get(app_conn, pool_id_value.clone())?;

                let res = app_config
                    .wallet
                    .execute(ContractCallInput::AssetLendingPool(
                        AssetLendingPoolFunctionsInput::GetPoolStats(pool.pool_contract_id), // TODO: pool id
                    ))
                    .await?;

                if let ContractCallOutput::AssetLendingPool(
                    AssetLendingPoolFunctionsOutput::GetPoolStats(stats),
                ) = res
                {
                    let data = stats
                        .output
                        .ok_or_else(|| anyhow!("No stats returned from contract"))?;
                    let new_snapshot = CreateLendingPoolSnapShotRecord {
                        borrow_apy: BigDecimal::from(data.borrow_rate.clone()),
                        supply_apy: BigDecimal::from(data.supply_rate.clone()),
                        available_liquidity: BigDecimal::from(data.liquidity.clone()),
                        lending_pool_id: pool_id_value.clone(),
                        total_borrow: BigDecimal::from(data.total_borrowed.clone()),
                        total_supply: BigDecimal::from(data.total_supplied.clone()),
                        utilization_rate: BigDecimal::from(data.utilization.clone()),
                    };

                    let snapshot_id =
                        diesel::insert_into(crate::schema::lendingpoolsnapshots::table)
                            .values(&new_snapshot)
                            .returning(crate::schema::lendingpoolsnapshots::dsl::id)
                            .get_result::<Uuid>(app_conn)?;

                    return Ok(LendingPoolFunctionsOutput::CreateSnapShot(snapshot_id));
                }

                Err(anyhow!("Failed to create snapshot"))
            }
            LendingPoolFunctionsInput::GetSnapShot(pool_id) => {
                use crate::schema::lendingpoolsnapshots::dsl::*;

                let res = lendingpoolsnapshots
                    .filter(lending_pool_id.eq(pool_id))
                    .order(created_at.desc())
                    .first::<LendingPoolSnapShotRecord>(app_conn)?;

                Ok(LendingPoolFunctionsOutput::GetSnapShot(res))
            }
            LendingPoolFunctionsInput::SupplyLiquidity(args) => {
                let pool = LendingPoolRecord::get(app_conn, args.pool)?;
                use crate::schema::cradlewalletaccounts;
                let wallet = cradlewalletaccounts::dsl::cradlewalletaccounts
                    .filter(cradlewalletaccounts::dsl::id.eq(args.wallet))
                    .get_result::<CradleWalletAccountRecord>(app_conn)?;

                // auto associate and grant kyc to account for user
                associate_token(
                    app_conn,
                    &mut app_config.wallet,
                    AssociateTokenToWalletInputArgs {
                        wallet_id: wallet.id,
                        token: pool.yield_asset,
                    },
                )
                .await?;

                kyc_token(
                    app_conn,
                    &mut app_config.wallet,
                    GrantKYCInputArgs {
                        wallet_id: wallet.id,
                        token: pool.yield_asset,
                    },
                )
                .await?;

                let result = app_config
                    .wallet
                    .execute(ContractCallInput::AssetLendingPool(
                        AssetLendingPoolFunctionsInput::Deposit(DepositArgs {
                            amount: args.amount.clone(),
                            user: wallet.address.clone(),
                            contract_id: pool.pool_contract_id,
                        }),
                    ))
                    .await?;

                record_transaction(
                    app_conn,
                    Some(wallet.address.clone()),
                    None,
                    RecordTransactionAssets::Deposit(Deposit {
                        deposited: pool.reserve_asset,
                        yield_asset: pool.yield_asset,
                    }),
                    Some(args.amount),
                    Some(result.clone()),
                    None,
                    None,
                    None,
                )?;

                if let ContractCallOutput::AssetLendingPool(
                    AssetLendingPoolFunctionsOutput::Deposit(output),
                ) = result.clone()
                {
                    let (supplyIndex, yieldTokensAmount) = output
                        .output
                        .ok_or_else(|| anyhow!("No output from deposit"))?;
                    let supply = CreatePoolTransactionRecord {
                        amount: BigDecimal::from(args.amount.clone()),
                        pool_id: args.pool.clone(),
                        wallet_id: wallet.id.clone(),
                        supply_index: BigDecimal::from(supplyIndex),
                        transaction: output.transaction_id,
                        transaction_type: PoolTransactionType::Supply,
                        yield_token_amount: BigDecimal::from(yieldTokensAmount),
                    };

                    let res = diesel::insert_into(crate::schema::pooltransactions::table)
                        .values(&supply)
                        .returning(crate::schema::pooltransactions::dsl::id)
                        .get_result::<Uuid>(app_conn)?;

                    return Ok(LendingPoolFunctionsOutput::SupplyLiquidity(res));
                }

                Err(anyhow!("Failed to supply liquidity"))
            }
            LendingPoolFunctionsInput::WithdrawLiquidity(args) => {
                let pool = LendingPoolRecord::get(app_conn, args.pool)?;

                use crate::schema::cradlewalletaccounts::dsl as cwa_dsl;

                let wallet = cradlewalletaccounts
                    .filter(cwa_dsl::id.eq(args.wallet))
                    .get_result::<CradleWalletAccountRecord>(app_conn)?;

                let result = app_config
                    .wallet
                    .execute(ContractCallInput::AssetLendingPool(
                        AssetLendingPoolFunctionsInput::Withdraw(WithdrawArgs {
                            yield_token_amount: args.amount.clone(),
                            user: wallet.address.clone(),
                            contract_id: pool.pool_contract_id,
                        }),
                    ))
                    .await?;

                record_transaction(
                    app_conn,
                    Some(wallet.address.clone()),
                    None,
                    RecordTransactionAssets::Withdraw(Withdraw {
                        underlying_asset: pool.reserve_asset,
                        yield_asset: pool.yield_asset,
                    }),
                    Some(args.amount),
                    Some(result.clone()),
                    None,
                    None,
                    None,
                )?;

                if let ContractCallOutput::AssetLendingPool(
                    AssetLendingPoolFunctionsOutput::Withdraw(output),
                ) = result
                {
                    let (withdrawIndex, underlyingAmount) = output
                        .output
                        .ok_or_else(|| anyhow!("No output from withdraw"))?;
                    let withdraw = CreatePoolTransactionRecord {
                        amount: BigDecimal::from(args.amount),
                        pool_id: args.pool.clone(),
                        wallet_id: wallet.id.clone(),
                        supply_index: BigDecimal::from(withdrawIndex),
                        transaction: output.transaction_id,
                        transaction_type: PoolTransactionType::Withdraw,
                        yield_token_amount: BigDecimal::from(underlyingAmount),
                    };

                    let res = diesel::insert_into(crate::schema::pooltransactions::table)
                        .values(&withdraw)
                        .returning(crate::schema::pooltransactions::dsl::id)
                        .get_result::<Uuid>(app_conn)?;

                    return Ok(LendingPoolFunctionsOutput::WithdrawLiquidity(res));
                }
                Err(anyhow!("Failed to withdraw liquidity"))
            }
            LendingPoolFunctionsInput::BorrowAsset(args) => {
                let pool = LendingPoolRecord::get(app_conn, args.pool)?;

                use crate::schema::asset_book::dsl::*;
                use crate::schema::cradlewalletaccounts::dsl as cwa_dsl;

                let wallet = cradlewalletaccounts
                    .filter(cwa_dsl::id.eq(args.wallet))
                    .get_result::<CradleWalletAccountRecord>(app_conn)?;

                let collateral_record = asset_book
                    .filter(id.eq(args.collateral))
                    .get_result::<AssetBookRecord>(app_conn)?;

                // auto associate and grant kyc to account for user
                associate_token(
                    app_conn,
                    &mut app_config.wallet,
                    AssociateTokenToWalletInputArgs {
                        wallet_id: wallet.id,
                        token: pool.reserve_asset,
                    },
                )
                .await?;

                kyc_token(
                    app_conn,
                    &mut app_config.wallet,
                    GrantKYCInputArgs {
                        wallet_id: wallet.id,
                        token: pool.reserve_asset,
                    },
                )
                .await?;

                let res = app_config
                    .wallet
                    .execute(ContractCallInput::AssetLendingPool(
                        AssetLendingPoolFunctionsInput::Borrow(BorrowArgs {
                            collateral_asset: collateral_record.token.clone(),
                            collateral_amount: args.amount.clone(),
                            user: wallet.address.clone(),
                            contract_id: pool.pool_contract_id.to_string(),
                        }),
                    ))
                    .await?;

                record_transaction(
                    app_conn,
                    Some(wallet.address.clone()),
                    None,
                    RecordTransactionAssets::Borrow(BorrowAssets {
                        collateral: collateral_record.id,
                        borrowed: pool.reserve_asset,
                    }),
                    Some(args.amount),
                    Some(res.clone()),
                    None,
                    None,
                    None,
                )?;

                if let ContractCallOutput::AssetLendingPool(
                    AssetLendingPoolFunctionsOutput::Borrow(output),
                ) = res
                {
                    let data = output
                        .output
                        .ok_or_else(|| anyhow!("No output from borrow"))?;
                    let new_borrow = CreateLoanRecord {
                        account_id: wallet.cradle_account_id.clone(),
                        wallet_id: wallet.id.clone(),
                        pool: args.pool.clone(),
                        transaction: Some(output.transaction_id.clone()),
                        borrow_index: BigDecimal::from(data.borrow_index),
                        principal_amount: BigDecimal::from(data.borrowed_amount),
                        status: LoanStatus::Active,
                        collateral_asset: args.collateral,
                    };

                    let loan_id = diesel::insert_into(crate::schema::loans::table)
                        .values(&new_borrow)
                        .returning(crate::schema::loans::dsl::id)
                        .get_result::<Uuid>(app_conn)?;

                    return Ok(LendingPoolFunctionsOutput::BorrowAsset(loan_id));
                }

                Err(anyhow!("Failed to borrow asset"))
            }
            LendingPoolFunctionsInput::RepayBorrow(args) => {
                use crate::schema::cradlewalletaccounts::dsl as cwa_dsl;
                use crate::schema::loans::dsl as loans_dsl;

                let wallet = cradlewalletaccounts
                    .filter(cwa_dsl::id.eq(args.wallet))
                    .get_result::<CradleWalletAccountRecord>(app_conn)?;

                let loan = crate::schema::loans::table
                    .filter(loans_dsl::id.eq(args.loan))
                    .get_result::<crate::lending_pool::db_types::LoanRecord>(app_conn)?;

                let pool = LendingPoolRecord::get(app_conn, loan.pool)?;

                let collateral_record = asset_book
                    .filter(crate::schema::asset_book::dsl::id.eq(loan.collateral_asset))
                    .get_result::<AssetBookRecord>(app_conn)?;

                let result = app_config
                    .wallet
                    .execute(ContractCallInput::AssetLendingPool(
                        AssetLendingPoolFunctionsInput::Repay(
                            contract_integrator::utils::functions::asset_lending::RepayArgs {
                                user: wallet.address.clone(),
                                collateralized_asset: collateral_record.token.clone(),
                                repay_amount: args.amount,
                                contract_id: pool.pool_contract_id,
                            },
                        ),
                    ))
                    .await?;

                record_transaction(
                    app_conn,
                    Some(wallet.address.clone()),
                    None,
                    RecordTransactionAssets::Single(pool.reserve_asset),
                    Some(args.amount),
                    Some(result.clone()),
                    None,
                    None,
                    None,
                )?;

                if let ContractCallOutput::AssetLendingPool(
                    AssetLendingPoolFunctionsOutput::Repay(output),
                ) = result
                {
                    let repayment = crate::lending_pool::db_types::CreateLoanRepaymentRecord {
                        loan_id: loan.id,
                        repayment_amount: BigDecimal::from(args.amount),
                        transaction: output.transaction_id.clone(),
                    };

                    update_repayment(
                        app_conn,
                        UpdateRepaymentArgs {
                            loan_id: loan.id,
                            amount: args.amount,
                            transaction: output.transaction_id.clone(),
                        },
                    )
                    .await?;

                    return Ok(LendingPoolFunctionsOutput::RepayBorrow());
                }

                Err(anyhow!("Failed to repay borrow"))
            }
            LendingPoolFunctionsInput::LiquidatePosition(args) => {
                use crate::schema::cradlewalletaccounts::dsl as cwa_dsl;
                use crate::schema::lendingpool::dsl as pool_dsl;
                use crate::schema::loans::dsl as loans_dsl;

                let liquidator_wallet = cradlewalletaccounts
                    .filter(cwa_dsl::id.eq(args.wallet))
                    .get_result::<CradleWalletAccountRecord>(app_conn)?;

                let loan = crate::schema::loans::table
                    .filter(loans_dsl::id.eq(args.loan))
                    .get_result::<crate::lending_pool::db_types::LoanRecord>(app_conn)?;

                let borrower_wallet = cradlewalletaccounts
                    .filter(cwa_dsl::id.eq(loan.wallet_id))
                    .get_result::<CradleWalletAccountRecord>(app_conn)?;

                let pool = LendingPoolRecord::get(app_conn, loan.pool)?;

                let collateral_record = asset_book
                    .filter(crate::schema::asset_book::dsl::id.eq(loan.collateral_asset))
                    .get_result::<AssetBookRecord>(app_conn)?;

                // associate collateral asset and kyc before giving the user the asset
                associate_token(
                    app_conn,
                    &mut app_config.wallet,
                    AssociateTokenToWalletInputArgs {
                        wallet_id: args.wallet,
                        token: loan.collateral_asset,
                    },
                )
                .await?;

                kyc_token(
                    app_conn,
                    &mut app_config.wallet,
                    GrantKYCInputArgs {
                        wallet_id: args.wallet,
                        token: loan.collateral_asset,
                    },
                )
                .await?;

                let result = app_config
                    .wallet
                    .execute(ContractCallInput::AssetLendingPool(
                        AssetLendingPoolFunctionsInput::Liquidate(
                            contract_integrator::utils::functions::asset_lending::LiquidateArgs {
                                liquidator: liquidator_wallet.address.clone(),
                                borrower: borrower_wallet.address.clone(),
                                dept_to_cover: args.amount,
                                collateral_asset: collateral_record.token.clone(),
                                contract_id: pool.pool_contract_id,
                            },
                        ),
                    ))
                    .await?;

                record_transaction(
                    app_conn,
                    Some(liquidator_wallet.address.clone()),
                    None,
                    RecordTransactionAssets::LiquidateLoan(LiquidateLoan {
                        reserve: pool.reserve_asset,
                        collateral: collateral_record.id,
                    }),
                    Some(args.amount),
                    Some(result.clone()),
                    None,
                    None,
                    Some(borrower_wallet.address),
                )?;

                if let ContractCallOutput::AssetLendingPool(
                    AssetLendingPoolFunctionsOutput::Liquidate(output),
                ) = result
                {
                    let liquidation = crate::lending_pool::db_types::CreateLoanLiquidationRecord {
                        loan_id: loan.id,
                        liquidator_wallet_id: liquidator_wallet.id,
                        liquidation_amount: BigDecimal::from(args.amount),
                        transaction: output.transaction_id,
                    };

                    let res = diesel::insert_into(crate::schema::loanliquidations::table)
                        .values(&liquidation)
                        .returning(crate::schema::loanliquidations::dsl::id)
                        .get_result::<Uuid>(app_conn)?;

                    return Ok(LendingPoolFunctionsOutput::LiquidatePosition());
                }

                Err(anyhow!("Failed to liquidate position"))
            }
        }
    }
}
