use contract_integrator::utils::functions::{
    FunctionCallOutput, cradle_native_listing::ListingStats,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::listing::operations::{
    CreateCompanyInputArgs, CreateListingInputArgs, GetPurchaseFeeInputArgs,
    PurchaseListingAssetInputArgs, ReturnAssetListingInputArgs, WithdrawToBeneficiaryInputArgsBody,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CradleNativeListingFunctionsInput {
    CreateCompany(CreateCompanyInputArgs),
    CreateListing(CreateListingInputArgs),
    Purchase(PurchaseListingAssetInputArgs),
    ReturnAsset(ReturnAssetListingInputArgs),
    WithdrawToBeneficiary(WithdrawToBeneficiaryInputArgsBody),
    GetStats(Uuid),
    GetFee(GetPurchaseFeeInputArgs),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum CradleNativeListingFunctionsOutput {
    CreateCompany(Uuid),
    CreateListing(Uuid),
    Purchase,
    ReturnAsset,
    WithdrawToBeneficiary,
    GetStats(ListingStats),
    GetFee(u64),
}
