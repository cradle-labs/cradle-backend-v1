use super::processor_enums::*;
use crate::accounts::config::AccountProcessorConfig;
use crate::accounts::db_types::{
    AccountAssetBookRecord, CradleAccountRecord, CradleWalletAccountRecord, CreateAccountAssetBook,
};
use crate::accounts::operations::{create_account_wallet, delete_account};
use crate::action_router::{ActionRouterInput, ActionRouterOutput};
use crate::asset_book::db_types::AssetBookRecord;
use crate::schema::asset_book::dsl as AssetBookDsl;
use crate::schema::cradleaccounts as CradleAccounts;
use crate::schema::cradlewalletaccounts as CradleWalletAccounts;
use crate::schema::cradlewalletaccounts::dsl::cradlewalletaccounts;
use crate::utils::app_config::AppConfig;
use crate::utils::traits::ActionProcessor;
use anyhow::anyhow;
use bigdecimal::ToPrimitive;
use chrono::Utc;
use contract_integrator::hedera::ContractId;
use contract_integrator::utils::functions::asset_manager::{
    AssetManagerFunctionInput, AssetManagerFunctionOutput,
};
use contract_integrator::utils::functions::cradle_account::{
    AssociateTokenArgs, CradleAccountFunctionInput, CradleAccountFunctionOutput, WithdrawArgs,
};
use contract_integrator::utils::functions::cradle_account_factory::{
    CradleAccountFactoryFunctionsInput, CradleAccountFactoryFunctionsOutput,
    CreateAccountInputArgs, GetAccountByControllerInputArgs,
};
use contract_integrator::utils::functions::{ContractCallInput, ContractCallOutput, commons};
use diesel::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use uuid::Uuid;

impl ActionProcessor<AccountProcessorConfig, AccountsProcessorOutput> for AccountsProcessorInput {
    async fn process(
        &self,
        app_config: &mut AppConfig,
        local_config: &mut AccountProcessorConfig,
        conn: Option<&mut PooledConnection<ConnectionManager<PgConnection>>>,
    ) -> anyhow::Result<AccountsProcessorOutput> {
        match self {
            AccountsProcessorInput::CreateAccount(args) => {
                if let Some(action_conn) = conn {
                    use crate::schema::cradleaccounts::dsl::*;

                    let account_id = diesel::insert_into(CradleAccounts::table)
                        .values(args)
                        .returning(id)
                        .get_result::<Uuid>(action_conn)?;

                    match create_account_wallet(
                        &mut local_config.wallet,
                        action_conn,
                        CreateCradleWalletInputArgs {
                            cradle_account_id: account_id,
                            status: None,
                        },
                    )
                    .await
                    {
                        Ok(wallet_data) => Ok(AccountsProcessorOutput::CreateAccount(
                            CreateAccountOutputArgs {
                                id: account_id.clone(),
                                wallet_id: wallet_data.id,
                            },
                        )),
                        Err(e) => {
                            println!("Failed to create wallet");
                            match delete_account(
                                action_conn,
                                DeleteAccountInputArgs::ById(account_id),
                            )
                            .await
                            {
                                Ok(_) => Err(anyhow!("Failed to create account")),
                                Err(_) => Err(anyhow!("Failed to create contract id")),
                            }
                        }
                    }
                } else {
                    Err(anyhow!("Failed to get conn"))
                }
            }
            AccountsProcessorInput::CreateAccountWallet(args) => {
                if let Some(action_conn) = conn {
                    use crate::schema::cradlewalletaccounts::dsl::*;

                    let res = local_config
                        .wallet
                        .execute(ContractCallInput::CradleAccountFactory(
                            CradleAccountFactoryFunctionsInput::CreateAccount(
                                CreateAccountInputArgs {
                                    account_allow_list: 1.to_string(),
                                    // TODO: may need to figure out a way to proxy this so it doesnt point directly to the user's id
                                    controller: args.cradle_account_id.to_string(),
                                },
                            ),
                        ))
                        .await?;

                    if let ContractCallOutput::CradleAccountFactory(
                        CradleAccountFactoryFunctionsOutput::CreateAccount(output),
                    ) = res
                    {
                        // TODO: do something with the result

                        let wallet_contract_address = output
                            .output
                            .ok_or_else(|| anyhow!("Failed to get wallet address"))?
                            .account_address;
                        println!(
                            "Wallet contract address:: {}",
                            wallet_contract_address.clone()
                        );
                        let contract_id_value = commons::get_contract_id_from_evm_address(
                            wallet_contract_address.as_str(),
                        )
                        .await?;
                        println!("Contract ID: {:?}", contract_id_value.clone());
                        let as_str_value = contract_id_value.to_string();
                        println!("Contract ID as String: {}", as_str_value);
                        let action_data = super::db_types::CreateCradleWalletAccount {
                            cradle_account_id: args.cradle_account_id.clone(),
                            contract_id: as_str_value,
                            address: wallet_contract_address,
                            status: args.status.clone(),
                        };

                        let wallet_id = diesel::insert_into(CradleWalletAccounts::table)
                            .values(&action_data)
                            .returning(id)
                            .get_result::<Uuid>(action_conn)?;

                        let associate_req = ActionRouterInput::Accounts(
                            AccountsProcessorInput::HandleAssociateAssets(wallet_id),
                        );

                        let kyc_req = ActionRouterInput::Accounts(
                            AccountsProcessorInput::HandleKYCAssets(wallet_id),
                        );

                        let _ = Box::pin(associate_req.process(app_config.clone())).await?;
                        let _ = Box::pin(kyc_req.process(app_config.clone())).await?;

                        return Ok(AccountsProcessorOutput::CreateAccountWallet(
                            CreateAccountWalletOutputArgs { id: wallet_id },
                        ));
                    } else {
                        return Err(anyhow!("Failed to  create account with factory contract"));
                    }
                }

                Err(anyhow!("Unable to get connection"))
            }
            AccountsProcessorInput::UpdateAccountStatus(args) => {
                if let Some(action_conn) = conn {
                    use crate::schema::cradleaccounts::dsl::*;

                    let _ = diesel::update(CradleAccounts::table)
                        .filter(id.eq(args.cradle_account_id))
                        .set(status.eq(&args.status))
                        .execute(action_conn)?;

                    return Ok(AccountsProcessorOutput::UpdateAccountStatus);
                }
                Err(anyhow!("Something went wrong"))
            }
            AccountsProcessorInput::UpdateAccountType(args) => {
                if let Some(action_conn) = conn {
                    use crate::schema::cradleaccounts::dsl::*;

                    let _ = diesel::update(CradleAccounts::table)
                        .filter(id.eq(args.cradle_account_id))
                        .set(account_type.eq(&args.account_type))
                        .execute(action_conn)?;

                    return Ok(AccountsProcessorOutput::UpdateAccountType);
                }
                Err(anyhow!(
                    "Unable to update account type cause can't get conn"
                ))
            }
            AccountsProcessorInput::UpdateAccountWalletStatusById(args) => {
                if let Some(action_conn) = conn {
                    use crate::schema::cradlewalletaccounts::dsl::*;

                    let _ = diesel::update(CradleWalletAccounts::table)
                        .filter(id.eq(args.wallet_id))
                        .set(status.eq(&args.status))
                        .execute(action_conn)?;

                    return Ok(AccountsProcessorOutput::UpdateAccountType);
                }
                Err(anyhow!(
                    "Unable to update account status cause can't get conn"
                ))
            }
            AccountsProcessorInput::UpdateAccountWalletStatusByAccount(args) => {
                if let Some(action_conn) = conn {
                    use crate::schema::cradlewalletaccounts::dsl::*;

                    let _ = diesel::update(CradleWalletAccounts::table)
                        .filter(cradle_account_id.eq(args.cradle_account_id))
                        .set(status.eq(&args.status))
                        .execute(action_conn)?;

                    return Ok(AccountsProcessorOutput::UpdateAccountType);
                }
                Err(anyhow!(
                    "Unable to update account status cause can't get conn"
                ))
            }
            AccountsProcessorInput::GetAccount(args) => {
                if let Some(action_conn) = conn {
                    use crate::schema::cradleaccounts::dsl::*;

                    let mut query = cradleaccounts.into_boxed();
                    match args {
                        GetAccountInputArgs::ByID(account_id) => {
                            query = query.filter(id.eq(account_id));
                        }
                        GetAccountInputArgs::ByLinkedAccount(linked_account_id_value) => {
                            query = query.filter(linked_account_id.eq(linked_account_id_value));
                        }
                    }

                    let res = query.get_result::<CradleAccountRecord>(action_conn)?;

                    return Ok(AccountsProcessorOutput::GetAccount(res));
                }
                Err(anyhow!("Unable to get account cause can't get conn"))
            }
            AccountsProcessorInput::GetWallet(args) => {
                if let Some(action_conn) = conn {
                    use crate::schema::cradlewalletaccounts::dsl::*;

                    let mut query = cradlewalletaccounts.into_boxed();
                    match args {
                        GetWalletInputArgs::ById(id_value) => {
                            query = query.filter(id.eq(id_value));
                        }
                        GetWalletInputArgs::ByCradleAccount(account_id_value) => {
                            query = query.filter(cradle_account_id.eq(account_id_value));
                        }
                    }

                    let res = query.get_result::<CradleWalletAccountRecord>(action_conn)?;

                    return Ok(AccountsProcessorOutput::GetWallet(res));
                }
                Err(anyhow!("Unable to get wallet cause can't get conn"))
            }
            AccountsProcessorInput::GetAccounts => {
                unimplemented!()
            }
            AccountsProcessorInput::GetWallets => {
                unimplemented!()
            }
            AccountsProcessorInput::DeleteAccount(instructions) => {
                use crate::schema::cradleaccounts::dsl::*;

                if let Some(action_conn) = conn {
                    match instructions {
                        DeleteAccountInputArgs::ById(account_id) => {
                            let _ = diesel::delete(CradleAccounts::table)
                                .filter(id.eq(account_id))
                                .execute(action_conn)?;
                        }
                        DeleteAccountInputArgs::ByLinkedAccount(id_value) => {
                            let _ = diesel::delete(CradleAccounts::table)
                                .filter(linked_account_id.eq(id_value))
                                .execute(action_conn)?;
                        }
                    }
                }

                Ok(AccountsProcessorOutput::DeleteAccount)
            }
            AccountsProcessorInput::DeleteWallet(instructions) => {
                use crate::schema::cradlewalletaccounts::dsl::*;

                if let Some(action_conn) = conn {
                    match instructions {
                        DeleteWalletInputArgs::ById(id_value) => {
                            let _ = diesel::delete(CradleWalletAccounts::table)
                                .filter(id.eq(id_value))
                                .execute(action_conn)?;
                        }
                        DeleteWalletInputArgs::ByOwner(owner) => {
                            let _ = diesel::delete(CradleWalletAccounts::table)
                                .filter(cradle_account_id.eq(owner))
                                .execute(action_conn)?;
                        }
                    }
                }

                Ok(AccountsProcessorOutput::DeleteWallet)
            }
            AccountsProcessorInput::AssociateTokenToWallet(args) => {
                let wallet_req = ActionRouterInput::Accounts(AccountsProcessorInput::GetWallet(
                    GetWalletInputArgs::ById(args.wallet_id),
                ));

                let token = AssetBookDsl::asset_book
                    .filter(AssetBookDsl::id.eq(args.token))
                    .get_result::<AssetBookRecord>(
                        conn.ok_or_else(|| anyhow!("Unable to get connection"))?,
                    )?;

                let res = Box::pin(wallet_req.process(app_config.clone())).await?;

                if let ActionRouterOutput::Accounts(AccountsProcessorOutput::GetWallet(wallet)) =
                    res
                {
                    let res = local_config
                        .wallet
                        .execute(ContractCallInput::CradleAccount(
                            CradleAccountFunctionInput::AssociateToken(AssociateTokenArgs {
                                account_contract_id: wallet.contract_id,
                                token: token.token,
                            }),
                        ))
                        .await?;

                    return if let ContractCallOutput::CradleAccount(
                        CradleAccountFunctionOutput::AssociateToken(out),
                    ) = res
                    {
                        println!("Out :: {:?}", out.transaction_id);
                        // TODO: record token somewhere

                        Ok(AccountsProcessorOutput::AssociateTokenToWallet)
                    } else {
                        Err(anyhow!("Unable to associate account"))
                    };
                } else {
                    return Err(anyhow!("Unable to find wallet"));
                }
            }
            AccountsProcessorInput::GrantKYC(args) => {
                let wallet_req = ActionRouterInput::Accounts(AccountsProcessorInput::GetWallet(
                    GetWalletInputArgs::ById(args.wallet_id),
                ));

                let app_conn = conn.ok_or_else(|| anyhow!("Unable to get connection"))?;

                let token_record = AssetBookDsl::asset_book
                    .filter(AssetBookDsl::id.eq(args.token))
                    .get_result::<AssetBookRecord>(app_conn)?;

                let res = Box::pin(wallet_req.process(app_config.clone())).await?;

                if let ActionRouterOutput::Accounts(AccountsProcessorOutput::GetWallet(wallet)) =
                    res
                {
                    let res = app_config
                        .wallet
                        .execute(ContractCallInput::AssetManager(
                            AssetManagerFunctionInput::GrantKYC(
                                token_record.asset_manager,
                                wallet.address.clone(),
                            ),
                        ))
                        .await?;

                    if let ContractCallOutput::AssetManager(AssetManagerFunctionOutput::GrantKYC(
                        output,
                    )) = res
                    {
                        println!("Grant kyc:: {}", output.transaction_id);
                        Ok(AccountsProcessorOutput::GrantKYC)
                    } else {
                        Err(anyhow!("Unable to grant kyc"))
                    }
                } else {
                    return Err(anyhow!("Unable to find wallet"));
                }
            }
            AccountsProcessorInput::WithdrawTokens(args) => {
                let wallet_req = ActionRouterInput::Accounts(AccountsProcessorInput::GetWallet(
                    GetWalletInputArgs::ById(args.from.clone()),
                ));

                let res = Box::pin(wallet_req.process(app_config.clone())).await?;

                if let ActionRouterOutput::Accounts(AccountsProcessorOutput::GetWallet(wallet)) =
                    res
                {
                    match args.withdrawal_type {
                        WithdrawalType::Fiat => {
                            unimplemented!("TODO: Fiat support will be added with opretium later")
                        }
                        WithdrawalType::Crypto => {
                            let res = local_config
                                .wallet
                                .execute(ContractCallInput::CradleAccount(
                                    CradleAccountFunctionInput::Withdraw(WithdrawArgs {
                                        account_contract_id: wallet.contract_id.clone(),
                                        amount: args.amount.to_u64().unwrap(),
                                        to: args.to.clone(),
                                        asset: args.token.clone(),
                                    }),
                                ))
                                .await?;

                            if let ContractCallOutput::CradleAccount(
                                CradleAccountFunctionOutput::Withdraw(o),
                            ) = res
                            {
                                // TODO: record this in the ledger

                                Ok(AccountsProcessorOutput::WithdrawTokens)
                            } else {
                                Err(anyhow!("Failed to withdraw tokens"))
                            }
                        }
                    }
                } else {
                    Err(anyhow!("Unable to find wallet"))
                }
            }
            AccountsProcessorInput::HandleAssociateAssets(wallet_id) => {
                use crate::schema::accountassetbook;
                use crate::schema::asset_book;
                use crate::schema::cradlewalletaccounts;

                if let Some(action_conn) = conn {
                    let wallet = cradlewalletaccounts::dsl::cradlewalletaccounts
                        .filter(cradlewalletaccounts::dsl::id.eq(wallet_id.clone()))
                        .first::<CradleWalletAccountRecord>(action_conn)?;

                    // find all assets in the assetbook table that the user has not associated yet
                    let unassociated_tokens = asset_book::dsl::asset_book
                        .left_join(
                            accountassetbook::table.on(accountassetbook::dsl::asset_id
                                .eq(asset_book::dsl::id)
                                .and(accountassetbook::dsl::associated.eq(true))
                                .and(accountassetbook::dsl::account_id.eq(wallet_id.clone()))),
                        )
                        .filter(accountassetbook::dsl::id.is_null())
                        .select(asset_book::all_columns)
                        .get_results::<AssetBookRecord>(action_conn)?;

                    for token in unassociated_tokens {
                        let res = local_config
                            .wallet
                            .execute(ContractCallInput::CradleAccount(
                                CradleAccountFunctionInput::AssociateToken(AssociateTokenArgs {
                                    account_contract_id: wallet.contract_id.clone(),
                                    token: token.token.clone(),
                                }),
                            ))
                            .await?;

                        if let ContractCallOutput::CradleAccount(
                            CradleAccountFunctionOutput::AssociateToken(_),
                        ) = res
                        {
                            // insert or update the accountassetbook to reflect the association
                            let now = Utc::now().naive_utc();
                            let asset_book_entry = CreateAccountAssetBook {
                                asset_id: token.id.clone(),
                                account_id: wallet_id.clone(),
                                associated: Some(true),
                                kyced: None,
                                associated_at: Some(now),
                                kyced_at: None,
                            };

                            diesel::insert_into(accountassetbook::table)
                                .values(&asset_book_entry)
                                .on_conflict((
                                    accountassetbook::dsl::asset_id,
                                    accountassetbook::dsl::account_id,
                                ))
                                .do_update()
                                .set((
                                    accountassetbook::dsl::associated.eq(true),
                                    accountassetbook::dsl::associated_at.eq(now),
                                ))
                                .execute(action_conn)?;
                        } else {
                            return Err(anyhow!(
                                "Unable to associate token {}",
                                token.token.clone()
                            ));
                        }
                    }
                    return Ok(AccountsProcessorOutput::HandleAssociateAssets);
                }

                Err(anyhow!("Unable to get connection"))
            }
            AccountsProcessorInput::HandleKYCAssets(wallet_id) => {
                use crate::schema::accountassetbook;
                use crate::schema::asset_book;
                use crate::schema::cradlewalletaccounts;

                if let Some(action_conn) = conn {
                    let wallet = cradlewalletaccounts::dsl::cradlewalletaccounts
                        .filter(cradlewalletaccounts::dsl::id.eq(wallet_id.clone()))
                        .first::<CradleWalletAccountRecord>(action_conn)?;

                    // find all assets in the assetbook table that the user has not registered yet
                    let unassociated_tokens = asset_book::dsl::asset_book
                        .left_join(
                            accountassetbook::table.on(accountassetbook::dsl::asset_id
                                .eq(asset_book::dsl::id)
                                .and(accountassetbook::dsl::kyced.eq(true))
                                .and(accountassetbook::dsl::account_id.eq(wallet_id.clone()))),
                        )
                        .filter(accountassetbook::dsl::id.is_null())
                        .select(asset_book::all_columns)
                        .get_results::<AssetBookRecord>(action_conn)?;

                    for token in unassociated_tokens {
                        let res = app_config
                            .wallet
                            .execute(ContractCallInput::AssetManager(
                                AssetManagerFunctionInput::GrantKYC(
                                    token.asset_manager,
                                    wallet.address.clone(),
                                ),
                            ))
                            .await?;
                        if let ContractCallOutput::AssetManager(
                            AssetManagerFunctionOutput::GrantKYC(_),
                        ) = res
                        {
                            // update the accountassetbook to reflect the KYC grant
                            let now = Utc::now().naive_utc();
                            let asset_book_entry = CreateAccountAssetBook {
                                asset_id: token.id.clone(),
                                account_id: wallet_id.clone(),
                                associated: None,
                                kyced: Some(true),
                                associated_at: None,
                                kyced_at: Some(now),
                            };

                            diesel::insert_into(accountassetbook::table)
                                .values(&asset_book_entry)
                                .on_conflict((
                                    accountassetbook::dsl::asset_id,
                                    accountassetbook::dsl::account_id,
                                ))
                                .do_update()
                                .set((
                                    accountassetbook::dsl::kyced.eq(true),
                                    accountassetbook::dsl::kyced_at.eq(now),
                                ))
                                .execute(action_conn)?;
                        } else {
                            return Err(anyhow!(
                                "Unable to grant kyc for token {}",
                                token.token.clone()
                            ));
                        }
                    }
                    return Ok(AccountsProcessorOutput::HandleKYCAssets);
                }

                Err(anyhow!("Unable to get connection"))
            }
        }
    }
}
