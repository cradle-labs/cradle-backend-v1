use std::{env, str::FromStr};

use anyhow::{Result, anyhow};
use bigdecimal::BigDecimal;
use contract_integrator::{
    hedera::ContractId,
    utils::functions::{
        ContractCallInput, ContractCallOutput,
        access_controller::{
            AccessControllerArgs, AccessControllerFunctionsInput, AccessControllerFunctionsOutput,
            GrantAccessBatchArgs,
        },
        cradle_native_listing::WithdrawToBeneficiaryInputArgs,
    },
    wallet::wallet::ActionWallet,
};
use cradle_back_end::{
    asset_book::{db_types::AssetType, processor_enums::CreateNewAssetInputArgs},
    choose,
    cli_utils::{print_error, print_success},
    collect_input,
    listing::{
        db_types::ListingStatus,
        operations::{
            AssetDetails, CreateCompanyInputArgs, CreateListingInputArgs, GetPurchaseFeeInputArgs,
            PurchaseListingAssetInputArgs, ReturnAssetListingInputArgs,
            WithdrawToBeneficiaryInputArgsBody, create_company, create_listing, get_listing,
            get_listing_stats, get_purchase_fee, purchase, return_asset, update_listing_status,
            withdraw_to_beneficiary,
        },
    },
    perr,
    utils::app_config::AppConfig,
};
use dialoguer::{Input, Select};
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, PooledConnection},
};
use serde::de::EnumAccess;
use uuid::Uuid;

#[tokio::main]
pub async fn main() -> Result<()> {
    dotenvy::dotenv()?;

    let config = AppConfig::from_env()?;

    let mut conn = config.pool.get()?;
    let mut wallet = config.wallet;
    println!("initializing");

    let action = choose!(
        "Select Action",
        "Create Company",
        "Create Listing",
        "Purchase From Listing",
        "Return Assets To Listing",
        "Withdraw to Beneficiary",
        "Get Stats",
        "Get Fees",
        "Update Listing Status",
        "Update access level",
        "Exit"
    );
    match action {
        0 => {
            create_company_cli(&mut conn, &mut wallet).await?;
        }
        1 => {
            create_listing_cli(&mut conn, &mut wallet).await?;
        }
        2 => {
            purchase_from_listing(&mut conn, &mut wallet).await?;
        }
        3 => {
            return_asset_to_listing(&mut conn, &mut wallet).await?;
        }
        4 => {
            withdraw_to_beneficiary_cli(&mut conn, &mut wallet).await?;
        }
        5 => {
            get_stats(&mut conn, &mut wallet).await?;
        }
        6 => {
            get_purchase_fee_cli(&mut conn, &mut wallet).await?;
        }
        7 => {
            update_listing_status_cli(&mut conn, &mut wallet).await?;
        }
        8 => {
            update_access_level_cli(&mut conn, &mut wallet).await?;
        }
        _ => {
            return Ok(());
        }
    };

    return Ok(());
}

pub async fn create_company_cli(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    wallet: &mut ActionWallet,
) -> Result<Uuid> {
    let name: String = Input::new()
        .with_prompt("Provide the Company Name")
        .interact()?;
    let description: String = Input::new()
        .with_prompt("Provide a description")
        .default("A Company".to_string())
        .interact()?;
    let legal_documents: String = Input::new()
        .default("Some docs".to_string())
        .with_prompt("Ipfs link to legal documents")
        .interact()?;
    match create_company(
        conn,
        wallet,
        CreateCompanyInputArgs {
            name,
            description,
            legal_documents,
        },
    )
    .await
    {
        Ok(company_id) => {
            print_success(&format!("Created new company {}", company_id.clone()));
            unsafe {
                env::set_var("LISTING_COMPANY_ID_TEMP", &company_id.to_string());
            };
            Ok(company_id)
        }
        Err(e) => {
            println!("An error occured :: {:?}", e);
            print_error("Failed to create company");
            Err(anyhow!("Failed to create company "))
        }
    }
}

pub async fn create_listing_cli(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    wallet: &mut ActionWallet,
) -> Result<Uuid> {
    let name: String = collect_input!("Name", String); // let description: String = Input::new().with_prompt
    let description = collect_input!("Description", String);
    let documents = collect_input!("IPFS link to listing documents", String);
    let default_value = env::var("LISTING_COMPANY_ID_TEMP").ok();

    let company = match default_value {
        Some(v) => {
            let stringified = Uuid::from_str(v.as_str())?;
            collect_input!("Company UUID", stringified, Uuid)
        }
        None => collect_input!("Company UUID", Uuid),
    };

    let asset_kind = choose!(
        "What kind of asset will this listing use",
        "New",
        "Existing"
    );

    let asset_details = match asset_kind {
        0 => {
            let name = collect_input!("Name", String);
            let symbol = collect_input!("Symbol", String);
            let asset_type_id = choose!(
                "Select Asset Type",
                "Bridged",
                "Native",
                "Yield Bearing",
                "Chain Native",
                "StableCoin",
                "Volatile"
            );
            let asset_type = AssetType::from(asset_type_id);
            let decimals = collect_input!("Decimals", 8i32, i32);
            let icon = collect_input!("Icon", "somewhere_in_ipfs".to_string(), String);

            AssetDetails::New(CreateNewAssetInputArgs {
                asset_type,
                name,
                symbol,
                decimals,
                icon,
            })
        }
        1 => AssetDetails::Existing(collect_input!("Existing asset UUID", Uuid)),
        _ => return Err(anyhow!("Unexpected asset kind")),
    };

    let purchase_asset = collect_input!("Purchase asset", Uuid);
    let purchase_price = collect_input!("Listed Asset Price", u64);
    let max_supply = collect_input!("Max Supply", u64);

    match create_listing(
        conn,
        wallet,
        CreateListingInputArgs {
            name,
            description,
            documents,
            company,
            asset: asset_details,
            purchase_asset,
            purchase_price: BigDecimal::from(purchase_price),
            max_supply: BigDecimal::from(max_supply),
        },
    )
    .await
    {
        Ok(res) => {
            println!("Success :: {:?}", res.clone());

            Ok(res)
        }
        Err(e) => {
            print_error(&format!("Failed with error {e}"));
            Err(anyhow!("Unable to create listing"))
        }
    }
}

pub async fn purchase_from_listing(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    wallet: &mut ActionWallet,
) -> Result<Uuid> {
    let wallet_id = collect_input!("User Wallet UUID::", Uuid);
    let amount = collect_input!("Amount of tokens to purchase::", u64);
    let listing = collect_input!("Listing UUID::", Uuid);

    match purchase(
        conn,
        wallet,
        PurchaseListingAssetInputArgs {
            wallet: wallet_id,
            amount: BigDecimal::from(amount),
            listing,
        },
    )
    .await
    {
        Ok(tx_id) => {
            print_success(&format!("Purchase successful {:?}", tx_id));
            Ok(tx_id)
        }
        Err(e) => {
            perr!(e);
            print_error("Failed to complete purchase");
            Err(anyhow!("Failed to complete purchase"))
        }
    }
}

pub async fn return_asset_to_listing(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    wallet: &mut ActionWallet,
) -> Result<Uuid> {
    let wallet_id = collect_input!("User Wallet UUID::", Uuid);
    let amount = collect_input!("Amount of tokens to purchase::", u64);
    let listing = collect_input!("Listing UUID::", Uuid);
    match return_asset(
        conn,
        wallet,
        ReturnAssetListingInputArgs {
            wallet: wallet_id,
            amount: BigDecimal::from(amount),
            listing,
        },
    )
    .await
    {
        Ok(tx_id) => {
            print_success(&format!("Return  successful {:?}", tx_id));
            Ok(tx_id)
        }
        Err(e) => {
            print_error("Failed to complete return");
            Err(anyhow!("Failed to complete return"))
        }
    }
}

pub async fn withdraw_to_beneficiary_cli(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    wallet: &mut ActionWallet,
) -> Result<Uuid> {
    let amount = collect_input!("Amount of tokens to purchase::", u64);
    let listing = collect_input!("Listing UUID::", Uuid);

    match withdraw_to_beneficiary(
        conn,
        wallet,
        WithdrawToBeneficiaryInputArgsBody {
            amount: BigDecimal::from(amount),
            listing,
        },
    )
    .await
    {
        Ok(tx_id) => {
            print_success(&format!("Withdrawal  successful {:?}", tx_id));
            Ok(tx_id)
        }
        Err(e) => {
            perr!(e);
            print_error("Failed to complete withdrawal");
            Err(anyhow!("Failed to complete withdrawal"))
        }
    }
}

pub async fn get_stats(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    wallet: &mut ActionWallet,
) -> Result<()> {
    let listing = collect_input!("Listing UUID::", Uuid);

    match get_listing_stats(conn, wallet, listing).await {
        Ok(data) => {
            print_success(&format!("Stats {:?}", data));
            Ok(())
        }
        Err(e) => {
            perr!(e);
            print_error("Failed to get data");
            Err(anyhow!("Failed to get data"))
        }
    }
}

pub async fn get_purchase_fee_cli(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    wallet: &mut ActionWallet,
) -> Result<()> {
    let listing = collect_input!("Listing UUID::", Uuid);
    let amount = collect_input!("Amount ::", u64);
    match get_purchase_fee(
        conn,
        wallet,
        GetPurchaseFeeInputArgs {
            listing_id: listing,
            amount: BigDecimal::from(amount),
        },
    )
    .await
    {
        Ok(data) => {
            print_success(&format!("Net {:?}", data));
            Ok(())
        }
        Err(e) => {
            perr!(e);
            print_error("Failed to get data");
            Err(anyhow!("Failed to get data"))
        }
    }
}

pub async fn update_listing_status_cli(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    wallet: &mut ActionWallet,
) -> Result<()> {
    let listing_id = collect_input!("LISTING UUID::", Uuid);
    let status_idx = choose!(
        "Select New Status",
        "Pending",
        "Open",
        "Closed",
        "Paused",
        "Cancelled"
    );

    let status = match status_idx {
        0 => ListingStatus::Pending,
        1 => ListingStatus::Open,
        2 => ListingStatus::Closed,
        3 => ListingStatus::Paused,
        _ => ListingStatus::Cancelled,
    };

    match update_listing_status(conn, wallet, listing_id, status).await {
        Ok(_) => {
            print_success(&format!("Success"));
            Ok(())
        }
        Err(e) => {
            perr!(e);
            print_error("Failed to get data");
            Err(anyhow!("Failed to get data"))
        }
    }
}

pub async fn update_access_level_cli(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    wallet: &mut ActionWallet,
) -> Result<()> {
    let listing_id = collect_input!("Listing ID ", Uuid);
    let listing = get_listing(conn, listing_id).await?;

    let address = ContractId::from_str(&listing.listing_contract_id)?.to_solidity_address()?;

    let level = collect_input!("Level", 0, u64);

    let transaction = ContractCallInput::AccessController(
        AccessControllerFunctionsInput::GrantAccess(AccessControllerArgs {
            level,
            account: address,
        }),
    );

    let res = wallet.execute(transaction).await?;

    match res {
        ContractCallOutput::AccessController(AccessControllerFunctionsOutput::GrantAccess(
            output,
        )) => {
            print_success(&format!("Result :: {:?}", output.transaction_id));
        }
        _ => {
            print_error("Something went wrong");
        }
    }

    Ok(())
}
