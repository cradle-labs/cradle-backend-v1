use crate::asset_book::config::AssetBookConfig;
use crate::asset_book::db_types::{AssetBookRecord, AssetType, CreateAssetOnBook};
use crate::asset_book::operations::create_asset;
use crate::asset_book::processor_enums::{
    AssetBookProcessorInput, AssetBookProcessorOutput, GetAssetInputArgs,
};
use crate::utils::app_config::AppConfig;
use crate::utils::traits::ActionProcessor;
use anyhow::anyhow;
use contract_integrator::utils::functions::asset_issuer::{
    AssetIssuerFunctionsInput, AssetIssuerFunctionsOutput, CreateAssetArgs,
};
use contract_integrator::utils::functions::commons::get_contract_id_from_evm_address;
use contract_integrator::utils::functions::{ContractCallInput, ContractCallOutput};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::{PgConnection, QueryDsl, RunQueryDsl};
use uuid::Uuid;

impl ActionProcessor<AssetBookConfig, AssetBookProcessorOutput> for AssetBookProcessorInput {
    async fn process(
        &self,
        app_config: &mut AppConfig,
        local_config: &mut AssetBookConfig,
        conn: Option<&mut PooledConnection<ConnectionManager<PgConnection>>>,
    ) -> anyhow::Result<AssetBookProcessorOutput> {
        let contract_ids = app_config.wallet.get_contract_ids()?;
        let app_conn = conn.ok_or_else(|| anyhow!("Unable to retrieve connection"))?;

        match self {
            AssetBookProcessorInput::CreateNewAsset(args) => {
                let mut wallet = app_config.wallet.clone();
                let asset_id = create_asset(&mut wallet, app_conn, args.clone()).await?;

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
                    token: args.token.clone(),
                };

                use crate::schema::asset_book as AssetBookTable;
                use crate::schema::asset_book::dsl::*;

                let asset_id = diesel::insert_into(AssetBookTable::table)
                    .values(&input)
                    .returning(id)
                    .get_result::<Uuid>(app_conn)?;

                Ok(AssetBookProcessorOutput::CreateExistingAsset(asset_id))
            }
            AssetBookProcessorInput::GetAsset(args) => {
                use crate::schema::asset_book::dsl::*;

                let mut query = asset_book.into_boxed();

                query = match args {
                    GetAssetInputArgs::ById(asset_id) => query.filter(id.eq(asset_id)),
                    GetAssetInputArgs::ByToken(token_value) => query.filter(token.eq(token_value)),
                    GetAssetInputArgs::ByAssetManager(manager_value) => {
                        query.filter(asset_manager.eq(manager_value))
                    }
                };

                let result = query.get_result::<AssetBookRecord>(app_conn)?;

                Ok(AssetBookProcessorOutput::GetAsset(result))
            }
        }
    }
}
