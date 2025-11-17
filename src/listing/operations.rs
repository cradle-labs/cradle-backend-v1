use crate::{
    accounts::{
        self,
        db_types::{
            CradleAccountStatus, CradleAccountType, CradleWalletAccountRecord, CradleWalletStatus,
            CreateCradleAccount,
        },
        processor_enums::{
            AssociateTokenToWalletInputArgs, CreateCradleWalletInputArgs, GrantKYCInputArgs,
        },
    },
    accounts_ledger::{
        db_types::AccountLedgerTransactionType,
        operations::{ListingPurchase, ListingSell, RecordTransactionAssets, record_transaction},
    },
    asset_book::{db_types::AssetBookRecord, processor_enums::CreateNewAssetInputArgs},
    listing::db_types::{
        CompanyRow, CradleNativeListingRow, CreateCompany, CreateCraldeNativeListing, ListingStatus,
    },
    schema::cradlenativelistings,
};
use accounts::operations::*;
use anyhow::{Result, anyhow};
use bigdecimal::{BigDecimal, ToPrimitive};
use contract_integrator::{
    utils::functions::{
        ContractCallInput, ContractCallOutput, WithContractId, commons,
        cradle_native_listing::{
            CradleNativeListingFunctionsInput, CradleNativeListingFunctionsOutput, ListingStats,
            PurchaseInputArgs, ReturnAssetInputArgs, WithdrawToBeneficiaryInputArgs,
        },
        listing_factory::{
            CradleListingFactoryFunctionsInput, CradleListingFactoryFunctionsOutput, CreateListing,
        },
    },
    wallet::wallet::ActionWallet,
};
use diesel::prelude::*;
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, PooledConnection},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub async fn get_listing(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    listing_id: Uuid,
) -> Result<CradleNativeListingRow> {
    use crate::schema::cradlenativelistings::dsl::*;

    let res = cradlenativelistings
        .filter(id.eq(listing_id))
        .get_result::<CradleNativeListingRow>(conn)?;
    Ok(res)
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct CreateCompanyInputArgs {
    pub name: String,
    pub description: String,
    pub legal_documents: String,
}

pub async fn create_company(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    wallet: &mut ActionWallet,
    input_args: CreateCompanyInputArgs,
) -> Result<Uuid> {
    use crate::schema::cradlelistedcompanies::{dsl::id, table as CompanyTable};
    let account_id = create_account(
        conn,
        CreateCradleAccount {
            linked_account_id: format!("company-{:?}", input_args.name.clone()),
            account_type: Some(CradleAccountType::Institutional),
            status: None,
        },
    )
    .await?;

    let wallet = create_account_wallet(
        wallet,
        conn,
        CreateCradleWalletInputArgs {
            cradle_account_id: account_id,
            status: None,
        },
    )
    .await?;

    let data = CreateCompany {
        name: input_args.name,
        description: input_args.description,
        legal_documents: input_args.legal_documents,
        beneficiary_wallet: wallet.id,
    };

    let company_id = diesel::insert_into(CompanyTable)
        .values(&data)
        .returning(id)
        .get_result::<Uuid>(conn)?;

    Ok(company_id)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AssetDetails {
    Existing(Uuid),
    New(CreateNewAssetInputArgs),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateListingInputArgs {
    pub name: String,
    pub description: String,
    pub documents: String,
    pub company: Uuid,
    pub asset: AssetDetails,
    pub purchase_asset: Uuid,
    pub purchase_price: BigDecimal,
    pub max_supply: BigDecimal,
}

pub async fn create_listing(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    wallet: &mut ActionWallet,
    input: CreateListingInputArgs,
) -> Result<Uuid> {
    let company = {
        use crate::schema::cradlelistedcompanies::dsl::*;

        cradlelistedcompanies
            .filter(id.eq(input.company.clone()))
            .get_result::<CompanyRow>(conn)
    }?;

    let beneficiary_wallet = {
        use crate::schema::cradlewalletaccounts::dsl::*;

        cradlewalletaccounts
            .filter(id.eq(company.beneficiary_wallet))
            .get_result::<CradleWalletAccountRecord>(conn)
    }?;

    let asset = {
        use crate::schema::asset_book::dsl::*;
        let asset_id = match input.asset {
            AssetDetails::Existing(asset_id) => asset_id,
            AssetDetails::New(args) => {
                let res = crate::asset_book::operations::create_asset(
                    wallet,
                    conn,
                    CreateNewAssetInputArgs {
                        asset_type: crate::asset_book::db_types::AssetType::Native,
                        name: args.name,
                        symbol: args.symbol,
                        decimals: args.decimals,
                        icon: args.icon,
                    },
                )
                .await?;

                res
            }
        };

        asset_book
            .filter(id.eq(asset_id))
            .get_result::<AssetBookRecord>(conn)?
    };

    let shadow_asset_value = {
        use crate::schema::asset_book::dsl::*;
        let res = crate::asset_book::operations::create_asset(
            wallet,
            conn,
            CreateNewAssetInputArgs {
                asset_type: crate::asset_book::db_types::AssetType::Native,
                name: format!("shadow-{:?}", asset.name.clone()),
                symbol: format!("s-{:?}", asset.symbol.clone()),
                decimals: asset.decimals,
                icon: asset.symbol,
            },
        )
        .await?;

        asset_book
            .filter(id.eq(res))
            .get_result::<AssetBookRecord>(conn)?
    };

    let purchase_asset = {
        use crate::schema::asset_book::dsl::*;

        asset_book
            .filter(id.eq(input.purchase_asset.clone()))
            .get_result::<AssetBookRecord>(conn)?
    };

    let treasury = {
        let ta = create_account(
            conn,
            CreateCradleAccount {
                linked_account_id: format!("treasurey-{:?}", Uuid::new_v4().to_string()),
                account_type: Some(CradleAccountType::Institutional),
                status: Some(CradleAccountStatus::Verified),
            },
        )
        .await?;

        let tw = create_account_wallet(
            wallet,
            conn,
            CreateCradleWalletInputArgs {
                cradle_account_id: ta,
                status: Some(CradleWalletStatus::Active),
            },
        )
        .await?;

        // associate
        // associate to purchase asset
        let _ = associate_token(
            conn,
            wallet,
            AssociateTokenToWalletInputArgs {
                wallet_id: tw.id,
                token: input.purchase_asset,
            },
        )
        .await?;
        // associate to listing asset
        let _ = associate_token(
            conn,
            wallet,
            AssociateTokenToWalletInputArgs {
                wallet_id: tw.id,
                token: asset.id,
            },
        )
        .await?;
        // will never hold shadow asset

        // kyc
        // kyc to purchase asset
        let _ = kyc_token(
            conn,
            wallet,
            GrantKYCInputArgs {
                wallet_id: tw.id,
                token: input.purchase_asset,
            },
        )
        .await?;

        // kyc to listing asset
        let _ = kyc_token(
            conn,
            wallet,
            GrantKYCInputArgs {
                wallet_id: tw.id,
                token: asset.id,
            },
        )
        .await?;
        // will never hold shadow asset

        tw
    };

    // create da listing

    let res = wallet
        .execute(ContractCallInput::CradleListingFactory(
            CradleListingFactoryFunctionsInput::CreateListing(CreateListing {
                fee_collector_address: "".to_string(),
                reserve_account: treasury.address,
                max_supply: input
                    .max_supply
                    .clone()
                    .to_u64()
                    .ok_or_else(|| anyhow!("unable to convert"))?,
                listing_asset: asset.asset_manager,
                purchase_asset: purchase_asset.token,
                purchase_price: input
                    .purchase_price
                    .to_u64()
                    .ok_or_else(|| anyhow!("Unable to unwrap"))?,
                beneficiary_address: beneficiary_wallet.address,
                shadow_asset: shadow_asset_value.asset_manager,
            }),
        ))
        .await?;

    let contract_id = {
        let address = match res {
            ContractCallOutput::CradleListingFactory(
                CradleListingFactoryFunctionsOutput::CreateListing(r),
            ) => r
                .output
                .ok_or_else(|| anyhow!("Failed to retrieve contract address"))?,
            _ => return Err(anyhow!("Failed to create listing successfully")),
        };

        let id = commons::get_contract_id_from_evm_address(address.as_str()).await?;

        id.to_string()
    };

    let listing = diesel::insert_into(cradlenativelistings::table)
        .values(CreateCraldeNativeListing {
            name: input.name,
            description: input.description,
            documents: input.documents,
            company: company.id,
            status: ListingStatus::Pending,
            opened_at: None,
            stopped_at: None,
            listed_asset: asset.id,
            purchase_with_asset: input.purchase_asset,
            purchase_price: input.purchase_price,
            max_supply: input.max_supply,
            treasury: treasury.id,
            listing_contract_id: contract_id,
            shadow_asset: shadow_asset_value.id,
        })
        .returning(cradlenativelistings::dsl::id)
        .get_result::<Uuid>(conn)?;

    Ok(listing)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PurchaseListingAssetInputArgs {
    pub wallet: Uuid,
    pub amount: BigDecimal,
    pub listing: Uuid,
}

pub async fn purchase(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    wallet: &mut ActionWallet,
    input: PurchaseListingAssetInputArgs,
) -> Result<()> {
    let listing = {
        use crate::schema::cradlenativelistings::dsl::*;

        cradlenativelistings
            .filter(id.eq(input.listing))
            .get_result::<CradleNativeListingRow>(conn)?
    };

    let account_wallet = {
        use crate::schema::cradlewalletaccounts::dsl::*;

        cradlewalletaccounts
            .filter(id.eq(input.wallet))
            .get_result::<CradleWalletAccountRecord>(conn)?
    };

    let _ = associate_token(
        conn,
        wallet,
        AssociateTokenToWalletInputArgs {
            wallet_id: input.wallet,
            token: listing.listed_asset,
        },
    )
    .await?;
    let _ = associate_token(
        conn,
        wallet,
        AssociateTokenToWalletInputArgs {
            wallet_id: input.wallet,
            token: listing.shadow_asset,
        },
    )
    .await?;

    let _ = kyc_token(
        conn,
        wallet,
        GrantKYCInputArgs {
            wallet_id: input.wallet,
            token: listing.listed_asset,
        },
    )
    .await?;
    let _ = kyc_token(
        conn,
        wallet,
        GrantKYCInputArgs {
            wallet_id: input.wallet,
            token: listing.shadow_asset,
        },
    )
    .await?;

    let transaction_input = ContractCallInput::CradleNativeListing(
        CradleNativeListingFunctionsInput::Purchase(WithContractId {
            contract_id: listing.listing_contract_id,
            rest: Some(PurchaseInputArgs {
                buyer: account_wallet.address.clone(),
                amount: input
                    .amount
                    .clone()
                    .to_u64()
                    .ok_or_else(|| anyhow!("Unable to unwrap"))?,
            }),
        }),
    );

    let transaction = wallet.execute(transaction_input).await?;

    // TODO: record balance updates to accounts ledger
    record_transaction(
        conn,
        Some(account_wallet.address),
        None,
        RecordTransactionAssets::ListingPurchase(ListingPurchase {
            purchased: listing.listed_asset,
            paying_with: listing.purchase_with_asset,
        }),
        input.amount.to_u64(),
        Some(transaction),
        Some(AccountLedgerTransactionType::BuyListed),
        None,
        None,
    )?;

    Ok(())
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ReturnAssetListingInputArgs {
    pub wallet: Uuid,
    pub amount: BigDecimal,
    pub listing: Uuid,
}

pub async fn return_asset(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    wallet: &mut ActionWallet,
    input: ReturnAssetListingInputArgs,
) -> Result<()> {
    let listing = {
        use crate::schema::cradlenativelistings::dsl::*;

        cradlenativelistings
            .filter(id.eq(input.listing))
            .get_result::<CradleNativeListingRow>(conn)?
    };

    let account_wallet = {
        use crate::schema::cradlewalletaccounts::dsl::*;

        cradlewalletaccounts
            .filter(id.eq(input.wallet))
            .get_result::<CradleWalletAccountRecord>(conn)?
    };

    let _ = associate_token(
        conn,
        wallet,
        AssociateTokenToWalletInputArgs {
            wallet_id: input.wallet,
            token: listing.listed_asset,
        },
    )
    .await?;
    let _ = associate_token(
        conn,
        wallet,
        AssociateTokenToWalletInputArgs {
            wallet_id: input.wallet,
            token: listing.shadow_asset,
        },
    )
    .await?;

    let _ = kyc_token(
        conn,
        wallet,
        GrantKYCInputArgs {
            wallet_id: input.wallet,
            token: listing.listed_asset,
        },
    )
    .await?;
    let _ = kyc_token(
        conn,
        wallet,
        GrantKYCInputArgs {
            wallet_id: input.wallet,
            token: listing.shadow_asset,
        },
    )
    .await?;

    let transaction_input = ContractCallInput::CradleNativeListing(
        CradleNativeListingFunctionsInput::ReturnAsset(WithContractId {
            contract_id: listing.listing_contract_id,
            rest: Some(ReturnAssetInputArgs {
                account: account_wallet.address.clone(),
                amount: input
                    .amount
                    .to_u64()
                    .ok_or_else(|| anyhow!("Unable to unwrap"))?,
            }),
        }),
    );

    let transaction = wallet.execute(transaction_input).await?;

    record_transaction(
        conn,
        Some(account_wallet.address),
        None,
        RecordTransactionAssets::ListingSell(ListingSell {
            sold: listing.listed_asset,
            received: listing.purchase_with_asset,
        }),
        input.amount.to_u64(),
        Some(transaction),
        Some(AccountLedgerTransactionType::SellListed),
        None,
        None,
    )?;

    Ok(())
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WithdrawToBeneficiaryInputArgsBody {
    pub amount: BigDecimal,
    pub listing: Uuid,
}

pub async fn withdraw_to_beneficiary(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    wallet: &mut ActionWallet,
    input: WithdrawToBeneficiaryInputArgsBody,
) -> Result<()> {
    let listing = {
        use crate::schema::cradlenativelistings::dsl::*;

        cradlenativelistings
            .filter(id.eq(input.listing))
            .get_result::<CradleNativeListingRow>(conn)?
    };

    let company = {
        use crate::schema::cradlelistedcompanies::dsl::*;

        cradlelistedcompanies
            .filter(id.eq(listing.company))
            .get_result::<CompanyRow>(conn)?
    };

    let company_wallet = {
        use crate::schema::cradlewalletaccounts::dsl::*;

        cradlewalletaccounts
            .filter(id.eq(company.beneficiary_wallet))
            .get_result::<CradleWalletAccountRecord>(conn)?
    };

    // associate and kyc the beneficiary wallet with purchase asset
    let _ = associate_token(
        conn,
        wallet,
        AssociateTokenToWalletInputArgs {
            wallet_id: company.beneficiary_wallet,
            token: listing.purchase_with_asset,
        },
    )
    .await?;
    let _ = kyc_token(
        conn,
        wallet,
        GrantKYCInputArgs {
            wallet_id: company.beneficiary_wallet,
            token: listing.purchase_with_asset,
        },
    )
    .await?;

    let transaction_input = ContractCallInput::CradleNativeListing(
        CradleNativeListingFunctionsInput::WithdrawToBeneficiary(WithContractId {
            contract_id: listing.listing_contract_id,
            rest: Some(WithdrawToBeneficiaryInputArgs {
                amount: input
                    .amount
                    .to_u64()
                    .ok_or_else(|| anyhow!("Failed to get u64"))?,
            }),
        }),
    );

    let transaction = wallet.execute(transaction_input).await?;

    record_transaction(
        conn,
        None,
        Some(company_wallet.address),
        RecordTransactionAssets::Single(listing.purchase_with_asset),
        input.amount.to_u64(),
        Some(transaction),
        Some(AccountLedgerTransactionType::ListingBeneficiaryWithdrawal),
        None,
        None,
    )?;

    Ok(())
}

pub async fn get_listing_stats(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    wallet: &mut ActionWallet,
    listing_id: Uuid,
) -> Result<ListingStats> {
    let listing = get_listing(conn, listing_id).await?;

    let transaction_input = ContractCallInput::CradleNativeListing(
        CradleNativeListingFunctionsInput::GetListingStats(WithContractId {
            contract_id: listing.listing_contract_id,
            rest: None,
        }),
    );

    let transaction = wallet.execute(transaction_input).await?;

    match transaction {
        ContractCallOutput::CradleNativeListing(
            CradleNativeListingFunctionsOutput::GetListingStats(o),
        ) => o.output.ok_or_else(|| anyhow!("Unable to retrieve stats")),
        _ => Err(anyhow!("Unable to get listing stats")),
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetPurchaseFeeInputArgs {
    pub listing_id: Uuid,
    pub amount: BigDecimal,
}

pub async fn get_purchase_fee(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    wallet: &mut ActionWallet,
    args: GetPurchaseFeeInputArgs,
) -> Result<u64> {
    let listing = get_listing(conn, args.listing_id).await?;

    let transaction_input = ContractCallInput::CradleNativeListing(
        CradleNativeListingFunctionsInput::GetFee(WithContractId {
            contract_id: listing.listing_contract_id,
            rest: args.amount.to_u64(),
        }),
    );

    let transaction = wallet.execute(transaction_input).await?;

    match transaction {
        ContractCallOutput::CradleNativeListing(CradleNativeListingFunctionsOutput::GetFee(o)) => {
            o.output.ok_or_else(|| anyhow!("Unable to retrieve stats"))
        }
        _ => Err(anyhow!("Unable to get listing stats")),
    }
}
