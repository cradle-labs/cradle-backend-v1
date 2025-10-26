use anyhow::anyhow;
use bigdecimal::ToPrimitive;
use diesel::prelude::*;
use contract_integrator::utils::functions::{ContractCallInput, ContractCallOutput};
use contract_integrator::utils::functions::cradle_account::{AssociateTokenArgs, CradleAccountFunctionInput, CradleAccountFunctionOutput, WithdrawArgs};
use contract_integrator::utils::functions::cradle_account_factory::{CradleAccountFactoryFunctionsInput, CradleAccountFactoryFunctionsOutput, CreateAccountInputArgs, GetAccountByControllerInputArgs};
use diesel::{PgConnection};
use diesel::r2d2::{ConnectionManager, PooledConnection};
use uuid::Uuid;
use crate::accounts::config::AccountProcessorConfig;
use crate::accounts::db_types::{CradleAccountRecord, CradleWalletAccountRecord};
use crate::action_router::{ActionRouterInput, ActionRouterOutput};
use crate::utils::app_config::AppConfig;
use crate::utils::traits::ActionProcessor;
use super::processor_enums::*;
use crate::schema::cradleaccounts as CradleAccounts;
use crate::schema::cradlewalletaccounts as CradleWalletAccounts;

impl ActionProcessor<AccountProcessorConfig, AccountsProcessorOutput> for AccountsProcessorInput {
    async fn process(&self, app_config: &mut AppConfig, local_config: &mut AccountProcessorConfig, conn: Option<&mut PooledConnection<ConnectionManager<PgConnection>>>) -> anyhow::Result<AccountsProcessorOutput> {

        match self {
            AccountsProcessorInput::CreateAccount(args) => {
                if let  Some(action_conn) = conn {

                    use crate::schema::cradleaccounts::dsl::*;

                    let account_id = diesel::insert_into(CradleAccounts::table).values(args).returning(id).get_result::<Uuid>(action_conn)?;

                    let request = ActionRouterInput::Accounts(
                        AccountsProcessorInput::CreateAccountWallet(CreateCradleWalletInputArgs{
                            cradle_account_id: account_id.clone(),
                            status: None
                        })
                    );

                    let output = Box::pin(request.process(app_config.clone())).await?;

                    return if let ActionRouterOutput::Accounts(AccountsProcessorOutput::CreateAccountWallet(res)) = output {
                        Ok(AccountsProcessorOutput::CreateAccount(CreateAccountOutputArgs {
                            id: account_id.clone(),
                            wallet_id: res.id
                        }))
                    } else {
                        let deletion_req = ActionRouterInput::Accounts(
                            AccountsProcessorInput::DeleteAccount(DeleteAccountInputArgs::ById(account_id.clone()))
                        );

                        // ignore result, should fail if something went wrong
                        let _ = Box::pin(deletion_req.process(app_config.clone())).await?;


                        Err(anyhow!("Failed to create contract id"))
                    }

                }

                Err(anyhow!("Connection not provided"))
            }
            AccountsProcessorInput::CreateAccountWallet(args) => {

                if let Some(action_conn) = conn {
                    use crate::schema::cradlewalletaccounts::dsl::*;

                    let res = local_config.wallet.execute(
                        ContractCallInput::CradleAccountFactory(
                            CradleAccountFactoryFunctionsInput::CreateAccount(
                                CreateAccountInputArgs {
                                    account_allow_list: 1.to_string(),
                                    // TODO: may need to figure out a way to proxy this so it doesnt point directly to the user's id
                                    controller: args.cradle_account_id.to_string()
                                }
                            )
                        )
                    ).await?;


                    if let ContractCallOutput::CradleAccountFactory(CradleAccountFactoryFunctionsOutput::CreateAccount(_)) = res {
                        // TODO: do something with the result

                        let call_res = local_config.wallet.execute(
                            ContractCallInput::CradleAccountFactory(
                                CradleAccountFactoryFunctionsInput::GetAccountByController(
                                    GetAccountByControllerInputArgs {
                                        controller: args.cradle_account_id.to_string()
                                    }
                                )
                            )
                        ).await?;


                        if let ContractCallOutput::CradleAccountFactory(CradleAccountFactoryFunctionsOutput::GetAccountByController(output)) = call_res {

                            let wallet_address = output.output.expect("Address not found");

                            let action_data = super::db_types::CreateCradleWalletAccount {
                              cradle_account_id: args.cradle_account_id.clone(),
                                contract_id: contract_integrator::hedera::ContractId::from_solidity_address(&wallet_address.account_address)?.to_string(),
                                address: wallet_address.account_address,
                                status: args.status.clone()
                            };

                            let wallet_id = diesel::insert_into(CradleWalletAccounts::table).values(&action_data).returning(id).get_result::<Uuid>(action_conn)?;


                            return Ok(AccountsProcessorOutput::CreateAccountWallet(CreateAccountWalletOutputArgs {
                                id: wallet_id
                            }));


                        }

                    }else {
                        return Err(anyhow!("Failed to  create account with factory contract"));
                    }
                }

                Err(anyhow!("Unable to get connection"))
            }
            AccountsProcessorInput::UpdateAccountStatus(args) => {

                if let Some (action_conn) = conn {
                    use crate::schema::cradleaccounts::dsl::*;

                    let _ = diesel::update(CradleAccounts::table).filter(
                        id.eq(args.cradle_account_id)
                    )
                        .set(
                            status.eq(&args.status)
                        ).execute(action_conn)?;

                    return Ok(AccountsProcessorOutput::UpdateAccountStatus);
                }
                Err(anyhow!("Something went wrong"))
            }
            AccountsProcessorInput::UpdateAccountType(args) => {
                if let Some(action_conn) = conn {
                    use crate::schema::cradleaccounts::dsl::*;

                    let _ = diesel::update(CradleAccounts::table).filter(
                        id.eq(args.cradle_account_id)
                    )
                        .set(
                            account_type.eq(&args.account_type)
                        ).execute(action_conn)?;

                    return Ok(AccountsProcessorOutput::UpdateAccountType);
                }
                Err(anyhow!("Unable to update account type cause can't get conn"))
            }
            AccountsProcessorInput::UpdateAccountWalletStatusById(args) => {
                if let Some(action_conn) = conn {
                    use crate::schema::cradlewalletaccounts::dsl::*;

                    let _ = diesel::update(CradleWalletAccounts::table).filter(
                        id.eq(args.wallet_id)
                    )
                        .set(
                            status.eq(&args.status)
                        ).execute(action_conn)?;

                    return Ok(AccountsProcessorOutput::UpdateAccountType);
                }
                Err(anyhow!("Unable to update account status cause can't get conn"))
            }
            AccountsProcessorInput::UpdateAccountWalletStatusByAccount(args) => {
                if let Some(action_conn) = conn {
                    use crate::schema::cradlewalletaccounts::dsl::*;

                    let _ = diesel::update(CradleWalletAccounts::table).filter(
                        cradle_account_id.eq(args.cradle_account_id)
                    )
                        .set(
                            status.eq(&args.status)
                        ).execute(action_conn)?;

                    return Ok(AccountsProcessorOutput::UpdateAccountType);
                }
                Err(anyhow!("Unable to update account status cause can't get conn"))
            }
            AccountsProcessorInput::GetAccount(args) => {
                if let Some(action_conn) = conn {
                    use crate::schema::cradleaccounts::dsl::*;

                    let mut query = cradleaccounts.into_boxed();
                    match args {
                        GetAccountInputArgs::ByID(account_id) => {
                            query = query.filter(
                                id.eq(account_id)
                            );
                        }
                        GetAccountInputArgs::ByLinkedAccount(linked_account_id_value) => {
                            query = query.filter(
                                linked_account_id.eq(linked_account_id_value)
                            );
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
                            query = query.filter(
                                id.eq(id_value)
                            );
                        }
                        GetWalletInputArgs::ByCradleAccount(account_id_value) => {
                            query = query.filter(
                                cradle_account_id.eq(account_id_value)
                            );
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
            },
            AccountsProcessorInput::DeleteAccount(instructions) => {
                use crate::schema::cradleaccounts::dsl::*;

                if let Some(action_conn) = conn {
                    match instructions {
                        DeleteAccountInputArgs::ById(account_id) => {
                            let _ = diesel::delete(CradleAccounts::table).filter(
                                id.eq(account_id)
                            ).execute(action_conn)?;
                        }
                        DeleteAccountInputArgs::ByLinkedAccount(id_value) => {
                            let _ = diesel::delete(CradleAccounts::table).filter(
                                linked_account_id.eq(id_value)
                            ).execute(action_conn)?;
                        }
                    }
                }

                Ok(AccountsProcessorOutput::DeleteAccount)
            },
            AccountsProcessorInput::DeleteWallet(instructions)=> {
                use crate::schema::cradlewalletaccounts::dsl::*;

                if let Some(action_conn) = conn {
                    match instructions {
                        DeleteWalletInputArgs::ById(id_value) => {
                            let _ = diesel::delete(CradleWalletAccounts::table)
                                .filter(
                                    id.eq(id_value)
                                ).execute(action_conn)?;
                        }
                        DeleteWalletInputArgs::ByOwner(owner) => {
                            let _ = diesel::delete(CradleWalletAccounts::table)
                                .filter(
                                    cradle_account_id.eq(owner)
                                ).execute(action_conn)?;
                        }
                    }
                }

                Ok(AccountsProcessorOutput::DeleteWallet)
            },
            AccountsProcessorInput::AssociateTokenToWallet(args)=>{

                    let wallet_req = ActionRouterInput::Accounts(
                        AccountsProcessorInput::GetWallet(
                            GetWalletInputArgs::ById(args.wallet_id)
                        )
                    );

                    let res = Box::pin(wallet_req.process(app_config.clone())).await?;

                    if let ActionRouterOutput::Accounts(AccountsProcessorOutput::GetWallet(wallet)) = res {

                        let res = local_config.wallet.execute(
                            ContractCallInput::CradleAccount(
                                CradleAccountFunctionInput::AssociateToken(
                                    AssociateTokenArgs {
                                        account_contract_id:wallet.contract_id,
                                        token: args.token.clone()
                                    }
                                )
                            )
                        ).await?;


                        return if let ContractCallOutput::CradleAccount(CradleAccountFunctionOutput::AssociateToken(out)) = res {
                            // TODO: record token somewhere

                            Ok(AccountsProcessorOutput::AssociateTokenToWallet)
                        } else {
                            Err(anyhow!("Unable to associate account"))
                        }


                    }else{
                        return Err(anyhow!("Unable to find wallet"));
                    }

            },
            AccountsProcessorInput::GrantKYC(args)=>{
                let wallet_req = ActionRouterInput::Accounts(
                    AccountsProcessorInput::GetWallet(
                        GetWalletInputArgs::ById(args.wallet_id)
                    )
                );

                let res = Box::pin(wallet_req.process(app_config.clone())).await?;

                if let ActionRouterOutput::Accounts(AccountsProcessorOutput::GetWallet(wallet)) = res {

                    todo!("Yet to figure out how to resolve tokens and token managers")


                }else{
                    return Err(anyhow!("Unable to find wallet"));
                }
            },
            AccountsProcessorInput::WithdrawTokens(args)=>{
                let wallet_req = ActionRouterInput::Accounts(
                    AccountsProcessorInput::GetWallet(
                        GetWalletInputArgs::ById(args.from.clone())
                    )
                );

                let res = Box::pin(wallet_req.process(app_config.clone())).await?;

                if let ActionRouterOutput::Accounts(AccountsProcessorOutput::GetWallet(wallet)) = res {

                    match args.withdrawal_type {
                        WithdrawalType::Fiat => {
                            unimplemented!("TODO: Fiat support will be added with opretium later")
                        }
                        WithdrawalType::Crypto => {
                            let res = local_config.wallet.execute(
                                ContractCallInput::CradleAccount(
                                    CradleAccountFunctionInput::Withdraw(
                                        WithdrawArgs {
                                            account_contract_id: wallet.contract_id.clone(),
                                            amount: args.amount.to_u64().unwrap(),
                                            to: args.to.clone(),
                                            asset: args.token.clone()
                                        }
                                    )
                                )
                            ).await?;

                            if let ContractCallOutput::CradleAccount(CradleAccountFunctionOutput::Withdraw(o)) = res {
                                // TODO: record this in the ledger

                                Ok(AccountsProcessorOutput::WithdrawTokens)
                            }else{
                                Err(anyhow!("Failed to withdraw tokens"))
                            }

                        }
                    }
                }else{
                    Err(anyhow!("Unable to find wallet"))
                }
            }
        }
    }
}