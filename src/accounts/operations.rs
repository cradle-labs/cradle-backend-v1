use crate::accounts::{
    db_types::{CradleWalletAccountRecord, CreateCradleWalletAccount},
    processor_enums::{CreateCradleWalletInputArgs, DeleteAccountInputArgs},
};
use anyhow::{Result, anyhow};
use contract_integrator::utils::functions::*;
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
