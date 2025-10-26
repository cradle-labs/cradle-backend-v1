use anyhow::anyhow;
use contract_integrator::utils::functions::asset_issuer::{AssetIssuerFunctionsInput, AssetIssuerFunctionsOutput, CreateAssetArgs};
use contract_integrator::utils::functions::{ContractCallInput, ContractCallOutput};
use diesel::prelude::*;
use diesel::{PgConnection, QueryDsl, RunQueryDsl};
use diesel::r2d2::{ConnectionManager, PooledConnection};
use uuid::Uuid;
use crate::asset_book::config::AssetBookConfig;
use crate::asset_book::db_types::{AssetBookRecord, AssetType, CreateAssetOnBook};
use crate::asset_book::processor_enums::{AssetBookProcessorInput, AssetBookProcessorOutput, GetAssetInputArgs};
use crate::utils::app_config::AppConfig;
use crate::utils::traits::ActionProcessor;

impl ActionProcessor<AssetBookConfig, AssetBookProcessorOutput> for AssetBookProcessorInput {
    async fn process(&self, app_config: &mut AppConfig, local_config: &mut AssetBookConfig, conn: Option<&mut PooledConnection<ConnectionManager<PgConnection>>>) -> anyhow::Result<AssetBookProcessorOutput> {
        let contract_ids = app_config.wallet.get_contract_ids()?;
        let app_conn = conn.ok_or_else(||anyhow!("Unable to retrieve connection"))?;

        match self {
            AssetBookProcessorInput::CreateNewAsset(args) => {

                let result = match args.asset_type.clone() {
                    AssetType::Bridged => {
                        let input = ContractCallInput::BridgedAssetIssuer(
                            AssetIssuerFunctionsInput::CreateAsset(CreateAssetArgs {
                                contract_id: contract_ids.bridged_asset_issuer_contract_id.to_string(),
                                symbol: args.symbol.clone(),
                                name: args.name.clone(),
                                acl_contract: contract_ids.access_controller_contract_id.to_solidity_address()?,
                                allow_list: 1
                            })
                        );

                        let output =  app_config.wallet.execute(
                            input
                        ).await?;

                        match output {
                            ContractCallOutput::BridgedAssetIssuer(
                                AssetIssuerFunctionsOutput::CreateAsset(res)
                            ) => {
                                res.output.ok_or_else(||anyhow!("Failed to retrieve result"))?
                            },
                            _=> return Err(anyhow!("Unable to find asset result"))
                        }

                    }
                    AssetType::Native => {
                        let input = ContractCallInput::NativeAssetIssuer(
                            AssetIssuerFunctionsInput::CreateAsset(CreateAssetArgs {
                                contract_id: contract_ids.native_asset_issuer_contract_id.to_string(),
                                symbol: args.symbol.clone(),
                                name: args.name.clone(),
                                acl_contract: contract_ids.access_controller_contract_id.to_solidity_address()?,
                                allow_list: 1
                            })
                        );

                        let output =  app_config.wallet.execute(
                            input
                        ).await?;

                        match output {
                            ContractCallOutput::NativeAssetIssuer(
                                AssetIssuerFunctionsOutput::CreateAsset(res)
                            ) => {
                                res.output.ok_or_else(||anyhow!("Failed to retrieve result"))?
                            },
                            _=>return Err(anyhow!("Failed to retrieve result"))
                        }
                    }
                    _=>{
                        unimplemented!("asset type not supported")
                    }
                };

                let input = CreateAssetOnBook {
                    asset_manager: result.asset_manager,
                    token: result.token,
                    name: args.name.clone(),
                    symbol: args.symbol.clone(),
                    asset_type: Some(args.asset_type.clone()),
                    decimals: args.decimals,
                    icon: Some(args.icon.clone())
                };

                use crate::schema::asset_book::dsl::*;
                use crate::schema::asset_book as AssetBookTable;


                let asset_id = diesel::insert_into(AssetBookTable::table).values(&input).returning(id).get_result::<Uuid>(app_conn)?;


                Ok(AssetBookProcessorOutput::CreateNewAsset(asset_id))
            }
            AssetBookProcessorInput::CreateExistingAsset(args) => {
                let input = CreateAssetOnBook {
                    asset_manager: args.token.clone(),
                    icon: Some(args.icon.clone()),
                    decimals: args.decimals,
                    asset_type: Some(args.asset_type.clone()),
                    symbol: args.symbol.clone(),
                    name: args.name.clone(),
                    token: args.token.clone()
                };

                use crate::schema::asset_book::dsl::*;
                use crate::schema::asset_book as AssetBookTable;


                let asset_id = diesel::insert_into(AssetBookTable::table).values(&input).returning(id).get_result::<Uuid>(app_conn)?;


                Ok(AssetBookProcessorOutput::CreateExistingAsset(asset_id))
            }
            AssetBookProcessorInput::GetAsset(args) => {

                use crate::schema::asset_book::dsl::*;

                let mut query = asset_book.into_boxed();

                query = match args {
                    GetAssetInputArgs::ById(asset_id)=>{
                        query.filter(
                            id.eq(asset_id)
                        )
                    },
                    GetAssetInputArgs::ByToken(token_value)=>{
                        query.filter(
                            token.eq(token_value)
                        )
                    },
                    GetAssetInputArgs::ByAssetManager(manager_value)=>{
                        query.filter(
                            asset_manager.eq(manager_value)
                        )
                    }
                };

                let result = query.get_result::<AssetBookRecord>(app_conn)?;


                Ok(AssetBookProcessorOutput::GetAsset(result))

            }
        }
    }
}