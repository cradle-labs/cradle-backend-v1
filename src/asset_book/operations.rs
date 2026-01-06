use anyhow::{Result, anyhow};
use contract_integrator::{
    id_to_evm_address,
    utils::functions::{
        ContractCallInput, ContractCallOutput,
        asset_factory::{AssetFactoryFunctionInput, AssetFactoryFunctionOutput},
        asset_issuer::{
            AssetIssuerFunctionsInput, AssetIssuerFunctionsOutput, CreateAssetArgs,
            CreateAssetResult,
        },
        asset_manager::{
            AirdropArgs, AssetManagerFunctionInput, AssetManagerFunctionOutput, MintArgs,
        },
        commons::{get_contract_addresses, get_contract_id_from_evm_address},
    },
    wallet::wallet::ActionWallet,
};
use diesel::prelude::*;
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, PooledConnection},
};
use uuid::Uuid;

use crate::{
    accounts::db_types::{AccountAssetBookRecord, CradleWalletAccountRecord},
    api::handlers::assets::get_asset_by_id,
    asset_book::{
        db_types::{AssetBookRecord, AssetType, CreateAssetOnBook},
        processor_enums::CreateNewAssetInputArgs,
    },
    extract_option,
};

pub async fn create_asset(
    wallet: &mut ActionWallet,
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    args: CreateNewAssetInputArgs,
) -> Result<Uuid> {
    let contract_ids = wallet.get_contract_ids()?;

    let acl_evm_add = get_contract_addresses(
        contract_ids
            .access_controller_contract_id
            .to_string()
            .as_str(),
    )
    .await?;

    println!(
        "Address {:?}",
        contract_ids
            .access_controller_contract_id
            .to_solidity_address()?
    );

    let result = match args.asset_type.clone() {
        AssetType::Bridged => {
            let input = ContractCallInput::BridgedAssetIssuer(
                AssetIssuerFunctionsInput::CreateAsset(CreateAssetArgs {
                    contract_id: contract_ids.bridged_asset_issuer_contract_id.to_string(),
                    symbol: args.symbol.clone(),
                    name: args.name.clone(),
                    acl_contract: acl_evm_add.clone(),
                    allow_list: 1,
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
                    acl_contract: acl_evm_add.clone(),
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
            let input = ContractCallInput::AssetFactory(AssetFactoryFunctionInput::CreateAsset(
                contract_integrator::utils::functions::asset_factory::CreateAssetArgs {
                    name: args.name.clone(),
                    symbol: args.symbol.clone(),
                    acl_contract: acl_evm_add.clone(),
                    allow_list: 1,
                },
            ));

            let output = wallet.execute(input).await?;

            match output {
                ContractCallOutput::AssetFactory(AssetFactoryFunctionOutput::CreateAsset(res)) => {
                    let o = extract_option!(res.output)?;
                    CreateAssetResult {
                        asset_manager: o.asset_manager,
                        token: o.token,
                    }
                }
                _ => return Err(anyhow!("Failed to retreive asset result")),
            }
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

pub async fn get_asset(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    asset_id: Uuid,
) -> Result<AssetBookRecord> {
    use crate::schema::asset_book::dsl::*;

    let record = asset_book
        .filter(id.eq(asset_id))
        .get_result::<AssetBookRecord>(conn)?;

    Ok(record)
}

pub async fn get_wallet(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    wallet_id: Uuid,
) -> Result<CradleWalletAccountRecord> {
    use crate::schema::cradlewalletaccounts::dsl::*;

    let record = cradlewalletaccounts
        .filter(id.eq(wallet_id))
        .get_result::<CradleWalletAccountRecord>(conn)?;

    Ok(record)
}

pub async fn mint_asset(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    wallet: &mut ActionWallet,
    asset_id: Uuid,
    amount: u64,
) -> Result<()> {
    let asset = get_asset(conn, asset_id).await?;

    let mint_req_input =
        ContractCallInput::AssetManager(AssetManagerFunctionInput::Mint(MintArgs {
            asset_contract: asset.asset_manager,
            amount,
        }));

    let mint_res = wallet.execute(mint_req_input).await?;

    match mint_res {
        ContractCallOutput::AssetManager(AssetManagerFunctionOutput::Mint(o)) => {
            println!("Transaction successful :: {:?}", o.transaction_id); // TODO: save minting event
            Ok(())
        }
        _ => Err(anyhow!("Failed to mint")),
    }
}

pub async fn airdrop_asset(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    wallet: &mut ActionWallet,
    asset_id: Uuid,
    wallet_id: Uuid,
    amount: u64,
) -> Result<()> {
    let asset = get_asset(conn, asset_id).await?;
    let account_wallet = get_wallet(conn, wallet_id).await?;

    let airdrop_req =
        ContractCallInput::AssetManager(AssetManagerFunctionInput::Airdrop(AirdropArgs {
            asset_contract: asset.asset_manager,
            target: account_wallet.address,
            amount,
        }));

    let res = wallet.execute(airdrop_req).await?;

    match res {
        ContractCallOutput::AssetManager(AssetManagerFunctionOutput::Airdrop(o)) => {
            println!("Transaction successful :: {:?}", o.transaction_id);
            Ok(()) // TODO: record airdrops to ledger
        }
        _ => Err(anyhow!("Failed to airdrop")),
    }
}
