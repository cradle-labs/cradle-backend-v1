use crate::{
    accounts::{
        db_types::{
            AccountAssetBookRecord, CradleWalletAccountRecord, CreateAccountAssetBook,
            CreateCradleAccount, CreateCradleWalletAccount,
        },
        processor_enums::{
            AssociateTokenToWalletInputArgs, CreateCradleWalletInputArgs, DeleteAccountInputArgs,
            GrantKYCInputArgs,
        },
    },
    asset_book::db_types::AssetBookRecord,
    schema::accountassetbook,
};
use anyhow::{Result, anyhow};
use chrono::Utc;
use contract_integrator::utils::functions::{
    asset_manager::AssetManagerFunctionOutput,
    cradle_account::{AssociateTokenArgs, CradleAccountFunctionInput, CradleAccountFunctionOutput},
    *,
};
use contract_integrator::{
    utils::functions::cradle_account_factory::{
        CradleAccountFactoryFunctionsOutput, CreateAccountInputArgs,
    },
    wallet::wallet::ActionWallet,
};
use diesel::prelude::*;
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, PooledConnection},
};
use uuid::Uuid;

pub async fn create_account(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    args: CreateCradleAccount,
) -> Result<Uuid> {
    use crate::schema::cradleaccounts::{dsl::*, table as ctable};
    let new_id = diesel::insert_into(ctable)
        .values(&args)
        .returning(id)
        .get_result::<Uuid>(conn)?;
    Ok(new_id)
}

pub async fn create_account_wallet(
    action_wallet: &mut ActionWallet,
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    args: CreateCradleWalletInputArgs,
) -> Result<CradleWalletAccountRecord> {
    use crate::schema::cradlewalletaccounts::table as CradleWalletAccountsTable;

    let res = action_wallet
        .execute(ContractCallInput::CradleAccountFactory(
            cradle_account_factory::CradleAccountFactoryFunctionsInput::CreateAccount(
                CreateAccountInputArgs {
                    account_allow_list: 1.to_string(),
                    controller: args.cradle_account_id.to_string(),
                },
            ),
        ))
        .await?;

    match res {
        ContractCallOutput::CradleAccountFactory(
            CradleAccountFactoryFunctionsOutput::CreateAccount(output),
        ) => {
            let wallet_address = output.output.ok_or_else(|| anyhow!("Missing address"))?;

            let wallet_contract_id =
                commons::get_contract_id_from_evm_address(&wallet_address.account_address).await?;

            let res = diesel::insert_into(CradleWalletAccountsTable)
                .values(&CreateCradleWalletAccount {
                    contract_id: wallet_contract_id.to_string(),
                    address: wallet_address.account_address,
                    cradle_account_id: args.cradle_account_id,
                    status: args.status,
                })
                .get_result::<CradleWalletAccountRecord>(conn)?;

            Ok(res)
        }
        _ => Err(anyhow!("Failed to create account")),
    }
}

pub async fn delete_account(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    instruction: DeleteAccountInputArgs,
) -> Result<()> {
    use crate::schema::cradleaccounts::dsl::*;
    use crate::schema::cradleaccounts::table as CradleAccountsTable;

    match instruction {
        DeleteAccountInputArgs::ById(account_id) => {
            let _ = diesel::delete(CradleAccountsTable)
                .filter(id.eq(account_id))
                .execute(conn)?;
        }
        DeleteAccountInputArgs::ByLinkedAccount(id_value) => {
            let _ = diesel::delete(CradleAccountsTable)
                .filter(linked_account_id.eq(id_value))
                .execute(conn)?;
        }
    }

    Ok(())
}

pub enum AssetRecordAction {
    Associate,
    KYC,
}

pub async fn update_asset_book_record(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    account: Uuid,
    asset: Uuid,
    action: AssetRecordAction,
) -> Result<()> {
    let now = Utc::now().naive_utc();

    let associated_input = match action {
        AssetRecordAction::Associate => Some(true),
        _ => None,
    };

    let kyc_input = match action {
        AssetRecordAction::KYC => Some(true),
        _ => None,
    };

    let entry = CreateAccountAssetBook {
        asset_id: asset,
        account_id: account,
        associated: associated_input,
        kyced: kyc_input,
        associated_at: None,
        kyced_at: None,
    };

    match action {
        AssetRecordAction::Associate => {
            let _ = diesel::insert_into(accountassetbook::table)
                .values(&entry)
                .on_conflict((
                    accountassetbook::dsl::asset_id,
                    accountassetbook::dsl::account_id,
                ))
                .do_update()
                .set((
                    accountassetbook::dsl::associated.eq(true),
                    accountassetbook::associated_at.eq(now),
                ))
                .execute(conn)?;
        }
        AssetRecordAction::KYC => {
            diesel::insert_into(accountassetbook::table)
                .values(&entry)
                .on_conflict((
                    accountassetbook::dsl::asset_id,
                    accountassetbook::dsl::account_id,
                ))
                .do_update()
                .set((
                    accountassetbook::dsl::kyced.eq(true),
                    accountassetbook::kyced_at.eq(now),
                ))
                .execute(conn)?;
        }
    };

    Ok(())
}

pub async fn associate_token(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    wallet: &mut ActionWallet,
    instruction: AssociateTokenToWalletInputArgs,
) -> Result<()> {
    let is_associated = {
        use crate::schema::accountassetbook::dsl::*;

        match accountassetbook
            .filter(
                account_id
                    .eq(instruction.wallet_id)
                    .and(asset_id.eq(instruction.token))
                    .and(associated.eq(true)),
            )
            .get_result::<AccountAssetBookRecord>(conn)
        {
            Ok(res) => res.associated,
            Err(_) => false,
        }
    };

    if is_associated {
        return Ok(());
    };
    let account_wallet = {
        use crate::schema::cradlewalletaccounts::dsl::*;

        let res = cradlewalletaccounts
            .filter(id.eq(instruction.wallet_id))
            .get_result::<CradleWalletAccountRecord>(conn)?;

        res
    };

    let asset = {
        use crate::schema::asset_book::dsl::*;

        let res = asset_book
            .filter(id.eq(instruction.token))
            .get_result::<AssetBookRecord>(conn)?;

        res
    };

    let res = wallet
        .execute(ContractCallInput::CradleAccount(
            CradleAccountFunctionInput::AssociateToken(AssociateTokenArgs {
                token: asset.token,
                account_contract_id: account_wallet.contract_id,
            }),
        ))
        .await?;

    match res {
        ContractCallOutput::CradleAccount(CradleAccountFunctionOutput::AssociateToken(v)) => {
            println!("association tx :: {:?}", v.transaction_id);
            update_asset_book_record(
                conn,
                account_wallet.id,
                asset.id,
                AssetRecordAction::Associate,
            )
            .await
        }
        _ => return Err(anyhow!("Failed to associate token account")),
    }
}

pub async fn kyc_token(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    wallet: &mut ActionWallet,
    instruction: GrantKYCInputArgs,
) -> Result<()> {
    let is_kyced = {
        use crate::schema::accountassetbook::dsl::*;

        match accountassetbook
            .filter(
                account_id
                    .eq(instruction.wallet_id)
                    .and(asset_id.eq(instruction.token))
                    .and(kyced.eq(true)),
            )
            .get_result::<AccountAssetBookRecord>(conn)
        {
            Ok(res) => res.associated,
            Err(_) => false,
        }
    };

    if is_kyced {
        return Ok(());
    };
    let account_wallet = {
        use crate::schema::cradlewalletaccounts::dsl::*;

        let res = cradlewalletaccounts
            .filter(id.eq(instruction.wallet_id))
            .get_result::<CradleWalletAccountRecord>(conn)?;

        res
    };

    let asset = {
        use crate::schema::asset_book::dsl::*;

        let res = asset_book
            .filter(id.eq(instruction.token))
            .get_result::<AssetBookRecord>(conn)?;

        res
    };

    let res = wallet
        .execute(ContractCallInput::AssetManager(
            asset_manager::AssetManagerFunctionInput::GrantKYC(
                asset.asset_manager,
                account_wallet.address,
            ),
        ))
        .await?;

    match res {
        ContractCallOutput::AssetManager(AssetManagerFunctionOutput::GrantKYC(v)) => {
            println!("kyc tx :: {:?}", v.transaction_id);
            update_asset_book_record(conn, account_wallet.id, asset.id, AssetRecordAction::KYC)
                .await
        }
        _ => return Err(anyhow!("Failed to associate token account")),
    }
}
