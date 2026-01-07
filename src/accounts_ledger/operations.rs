use crate::accounts_ledger::db_types::{AccountLedgerTransactionType, CreateLedgerEntry};
use anyhow::{Result, anyhow};
use bigdecimal::{BigDecimal, ToPrimitive};
use contract_integrator::utils::functions::{
    ContractCallOutput, asset_lending::AssetLendingPoolFunctionsOutput,
    cradle_account::CradleAccountFunctionOutput,
    cradle_native_listing::CradleNativeListingFunctionsOutput,
    orderbook_settler::OrderBookSettlerFunctionOutput,
};
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, PooledConnection},
};
use uuid::Uuid;

pub fn create_ledger_entry(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    input: CreateLedgerEntry,
) -> Result<Uuid> {
    let row = input.insert(conn)?;
    Ok(row.id)
}

pub struct BorrowAssets {
    pub collateral: Uuid,
    pub borrowed: Uuid,
}

pub struct Deposit {
    pub deposited: Uuid,
    pub yield_asset: Uuid,
}

pub struct Withdraw {
    pub yield_asset: Uuid,
    pub underlying_asset: Uuid,
}

pub struct ListingPurchase {
    pub purchased: Uuid,
    pub paying_with: Uuid,
}

pub struct ListingSell {
    pub sold: Uuid,
    pub received: Uuid,
}

pub struct LiquidateLoan {
    pub reserve: Uuid,
    pub collateral: Uuid,
}

pub enum RecordTransactionAssets {
    Single(Uuid),
    Borrow(BorrowAssets),
    Repay(BorrowAssets),
    Deposit(Deposit),
    Withdraw(Withdraw),
    ListingPurchase(ListingPurchase),
    ListingSell(ListingSell),
    LiquidateLoan(LiquidateLoan),
}

pub fn record_transaction(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    from: Option<String>,
    to: Option<String>,
    assets: RecordTransactionAssets,
    amount: Option<u64>,
    transaction: Option<ContractCallOutput>,
    transaction_type: Option<AccountLedgerTransactionType>,
    tx_id: Option<String>,
    secondary_participant: Option<String>,
) -> Result<Uuid> {
    let from_address = from.clone().unwrap_or("system".to_string());
    let to_address = to.clone().unwrap_or("system".to_string());

    let asset = match &assets {
        RecordTransactionAssets::Single(v) => v.clone(),
        RecordTransactionAssets::Borrow(v) => v.borrowed,
        RecordTransactionAssets::Repay(v)=>v.borrowed,
        RecordTransactionAssets::Deposit(v) => v.deposited,
        RecordTransactionAssets::ListingPurchase(v) => v.purchased,
        RecordTransactionAssets::ListingSell(v) => v.sold,
        RecordTransactionAssets::Withdraw(v) => v.underlying_asset,
        RecordTransactionAssets::LiquidateLoan(v) => v.reserve,
    };

    let secondary_asset = match &assets {
        RecordTransactionAssets::Single(v) => v.clone(),
        RecordTransactionAssets::Borrow(v) => v.collateral,
        RecordTransactionAssets::Deposit(v) => v.yield_asset,
        RecordTransactionAssets::ListingPurchase(v) => v.paying_with,
        RecordTransactionAssets::ListingSell(v) => v.received,
        RecordTransactionAssets::Withdraw(v) => v.yield_asset,
        RecordTransactionAssets::LiquidateLoan(v) => v.collateral,
        RecordTransactionAssets::Repay(v)=>v.collateral
    };

    let amount = BigDecimal::from(amount.unwrap_or(0));

    let mut ledger_entry = CreateLedgerEntry {
        from_address: from_address.clone(),
        to_address: to_address.clone(),
        transaction: tx_id,
        asset,
        transaction_type: transaction_type.unwrap_or(AccountLedgerTransactionType::Lock),
        amount: amount.clone(),
        refference: None,
    };

    if let Some(tx) = transaction {
        match tx {
            ContractCallOutput::CradleAccount(CradleAccountFunctionOutput::LockAsset(output)) => {
                ledger_entry.transaction = Some(output.transaction_id);
            }
            ContractCallOutput::CradleAccount(CradleAccountFunctionOutput::UnLockAsset(output)) => {
                ledger_entry.transaction = Some(output.transaction_id);
                ledger_entry.transaction_type = AccountLedgerTransactionType::UnLock;
            }
            ContractCallOutput::AssetLendingPool(AssetLendingPoolFunctionsOutput::Deposit(
                output,
            )) => {
                let (_, yield_tokens_minted) = output.output.unwrap_or((0, 0));

                ledger_entry.transaction = Some(output.transaction_id.clone());
                ledger_entry.transaction_type = AccountLedgerTransactionType::Lend;

                let _ = record_transaction(
                    conn,
                    to, // this becomes system, so system transfers yield assets to user
                    from,
                    RecordTransactionAssets::Single(secondary_asset),
                    Some(yield_tokens_minted),
                    None,
                    Some(AccountLedgerTransactionType::Transfer),
                    Some(output.transaction_id),
                    None,
                )?;
            }
            ContractCallOutput::AssetLendingPool(AssetLendingPoolFunctionsOutput::Withdraw(
                output,
            )) => {
                let (_, withdraw_amount) = output.output.unwrap_or((0, 0));
                ledger_entry.transaction = Some(output.transaction_id.clone());
                ledger_entry.transaction_type = AccountLedgerTransactionType::Withdraw;
                ledger_entry.amount = BigDecimal::from(withdraw_amount);
                ledger_entry.to_address = from_address;
                ledger_entry.from_address = to_address; // withdraw goes from system to user

                let _ = record_transaction(
                    conn,
                    from, // this becomes system, so system transfers yield assets to user
                    to,
                    RecordTransactionAssets::Single(secondary_asset),
                    amount.to_u64(), // amount of yield tokens
                    None,
                    Some(AccountLedgerTransactionType::Transfer),
                    Some(output.transaction_id),
                    None,
                )?;
            }
            ContractCallOutput::AssetLendingPool(AssetLendingPoolFunctionsOutput::Borrow(
                output,
            )) => {
                let res = output.output.ok_or_else(|| anyhow!("Failed to extract"))?;

                ledger_entry.transaction = Some(output.transaction_id.clone());
                ledger_entry.transaction_type = AccountLedgerTransactionType::Borrow;
                ledger_entry.amount = BigDecimal::from(res.borrowed_amount);
                // should trigger lock

                let _ = record_transaction(
                    conn,
                    to, // this becomes system, so system locks assets
                    from,
                    RecordTransactionAssets::Single(secondary_asset),
                    amount.to_u64(), // collateral amount
                    None,
                    Some(AccountLedgerTransactionType::Lock),
                    Some(output.transaction_id),
                    None,
                )?;
            }
            ContractCallOutput::AssetLendingPool(AssetLendingPoolFunctionsOutput::Repay(
                output,
            )) => {
                let res = output.output.ok_or_else(|| anyhow!("Failed to extract"))?;

                ledger_entry.transaction = Some(output.transaction_id.clone());
                ledger_entry.transaction_type = AccountLedgerTransactionType::Repay;
                // should trigger unlock

                let _ = record_transaction(
                    conn,
                    to, // this becomes system
                    from,
                    RecordTransactionAssets::Single(secondary_asset),
                    Some(res.collateral_unlocked),
                    None,
                    Some(AccountLedgerTransactionType::UnLock),
                    Some(output.transaction_id),
                    None,
                )?;
            }
            ContractCallOutput::AssetLendingPool(AssetLendingPoolFunctionsOutput::Liquidate(
                output,
            )) => {
                let res = output.output.ok_or_else(|| anyhow!("Failed to extract"))?;

                ledger_entry.transaction = Some(output.transaction_id.clone());
                ledger_entry.transaction_type = AccountLedgerTransactionType::Liquidate;
                // todo: additional transaction for obtained collateral
                let _ = record_transaction(
                    conn,
                    to,                    // this becomes system
                    secondary_participant, // this is the user we're liquidating
                    RecordTransactionAssets::Single(secondary_asset),
                    Some(res.obtained_collateral),
                    None,
                    Some(AccountLedgerTransactionType::UnLock),
                    Some(output.transaction_id),
                    None,
                )?;
            }
            ContractCallOutput::OrderBookSettler(OrderBookSettlerFunctionOutput::SettleOrder(
                output,
            )) => {
                // the transaction result should probably include how much gets transfered

                ledger_entry.transaction = Some(output.transaction_id);
                ledger_entry.transaction_type = AccountLedgerTransactionType::FillOrder;

                // TODO: trigger unlocks on both sides of the transaction this will have to be done separately by the caller
            }
            ContractCallOutput::CradleNativeListing(
                CradleNativeListingFunctionsOutput::Purchase(output),
            ) => {
                ledger_entry.transaction = Some(output.transaction_id.clone());
                ledger_entry.transaction_type = AccountLedgerTransactionType::BuyListed;
                ledger_entry.from_address = to_address; // from system
                ledger_entry.to_address = from_address; // to user

                // amount bought already set

                let _ = record_transaction(
                    conn,
                    from, // this becomes user
                    to,   // this becomes system
                    RecordTransactionAssets::Single(secondary_asset),
                    output.output,
                    None,
                    Some(AccountLedgerTransactionType::Transfer),
                    Some(output.transaction_id),
                    None,
                )?;
            }
            ContractCallOutput::CradleNativeListing(
                CradleNativeListingFunctionsOutput::ReturnAsset(output),
            ) => {
                ledger_entry.transaction = Some(output.transaction_id.clone());
                ledger_entry.transaction_type = AccountLedgerTransactionType::SellListed;

                let _ = record_transaction(
                    conn,
                    to,   // this becomes sytem
                    from, // this becomes user receiving this asset
                    RecordTransactionAssets::Single(secondary_asset),
                    output.output,
                    None,
                    Some(AccountLedgerTransactionType::Transfer),
                    Some(output.transaction_id),
                    None,
                )?;
            }
            ContractCallOutput::CradleNativeListing(
                CradleNativeListingFunctionsOutput::WithdrawToBeneficiary(output),
            ) => {
                ledger_entry.transaction = Some(output.transaction_id);
                ledger_entry.transaction_type = AccountLedgerTransactionType::BuyListed;
            }
            _ => return Err(anyhow!("unsupported transaction")),
        };
    }

    let res = ledger_entry.insert(conn)?;

    Ok(res.id)
}
