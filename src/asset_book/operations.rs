use anyhow::{Result, anyhow};
use contract_integrator::{
    utils::functions::{
        ContractCallInput, ContractCallOutput,
        asset_issuer::{AssetIssuerFunctionsInput, AssetIssuerFunctionsOutput, CreateAssetArgs},
        commons::get_contract_id_from_evm_address,
    },
    wallet::wallet::ActionWallet,
};
use diesel::prelude::*;
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, PooledConnection},
};
use uuid::Uuid;

use crate::asset_book::{
    db_types::{AssetType, CreateAssetOnBook},
    processor_enums::CreateNewAssetInputArgs,
};

pub async fn create_asset(
    wallet: &mut ActionWallet,
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    args: CreateNewAssetInputArgs,
) -> Result<Uuid> {
    let contract_ids = wallet.get_contract_ids()?;

    let result = match args.asset_type.clone() {
        AssetType::Bridged => {
            let input = ContractCallInput::BridgedAssetIssuer(
                AssetIssuerFunctionsInput::CreateAsset(CreateAssetArgs {
                    contract_id: contract_ids.bridged_asset_issuer_contract_id.to_string(),
                    symbol: args.symbol.clone(),
                    name: args.name.clone(),
                    acl_contract: contract_ids
                        .access_controller_contract_id
                        .to_solidity_address()?,
                    allow_list: 2,
                }),
            );

            let output = wallet.execute(input).await?;

            match output {
                ContractCallOutput::BridgedAssetIssuer(
                    AssetIssuerFunctionsOutput::CreateAsset(res),
                ) => res
                    .output
                    .ok_or_else(|| anyhow!("Failed to retrieve result"))?,
                _ => return Err(anyhow!("Unable to find asset result")),
            }
        }
        AssetType::Native => {
            let input = ContractCallInput::NativeAssetIssuer(
                AssetIssuerFunctionsInput::CreateAsset(CreateAssetArgs {
                    contract_id: contract_ids.native_asset_issuer_contract_id.to_string(),
                    symbol: args.symbol.clone(),
                    name: args.name.clone(),
                    acl_contract: contract_ids
                        .access_controller_contract_id
                        .to_solidity_address()?,
                    allow_list: 1,
                }),
            );

            let output = wallet.execute(input).await?;

            match output {
                ContractCallOutput::NativeAssetIssuer(AssetIssuerFunctionsOutput::CreateAsset(
                    res,
                )) => res
                    .output
                    .ok_or_else(|| anyhow!("Failed to retrieve result"))?,
                _ => return Err(anyhow!("Failed to retrieve result")),
            }
        }
        _ => {
            unimplemented!("asset type not supported")
        }
    };

    let asset_manager_contract_id = get_contract_id_from_evm_address(&result.asset_manager).await?;

    let input = CreateAssetOnBook {
        asset_manager: asset_manager_contract_id.to_string(),
        token: result.token,
        name: args.name.clone(),
        symbol: args.symbol.clone(),
        asset_type: Some(args.asset_type.clone()),
        decimals: args.decimals,
        icon: Some(args.icon.clone()),
    };

    use crate::schema::asset_book as AssetBookTable;
    use crate::schema::asset_book::dsl::*;

    let asset_id = diesel::insert_into(AssetBookTable::table)
        .values(&input)
        .returning(id)
        .get_result::<Uuid>(conn)?;

    Ok(asset_id)
}
