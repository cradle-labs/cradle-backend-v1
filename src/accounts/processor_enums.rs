use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::accounts::db_types::{CradleAccountRecord, CradleAccountStatus, CradleAccountType, CradleWalletAccountRecord, CradleWalletStatus, CreateCradleAccount, CreateCradleWalletAccount};

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateCradleWalletInputArgs {
    pub cradle_account_id: Uuid,
    pub status: Option<CradleWalletStatus>
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UpdateAccountStatusInputArgs {
    pub cradle_account_id: Uuid,
    pub status: CradleAccountStatus
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UpdateAccountTypeInputArgs {
    pub cradle_account_id: Uuid,
    pub account_type: CradleAccountType
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UpdateWalletStatusByIdInputArgs {
    pub wallet_id: Uuid,
    pub status: CradleWalletStatus
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UpdateWalletStatusByAccountIdInputArgs {
    pub cradle_account_id: Uuid,
    pub status: CradleWalletStatus
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum GetAccountInputArgs {
    ByID(Uuid),
    ByLinkedAccount(String)
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum GetWalletInputArgs {
    ById(Uuid),
    ByCradleAccount(Uuid)
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum DeleteAccountInputArgs {
    ById(Uuid),
    ByLinkedAccount(String)
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum DeleteWalletInputArgs {
    ById(Uuid),
    ByOwner(Uuid)
}

#[derive(Deserialize,Serialize, Debug)]
pub struct AssociateTokenToWalletInputArgs {
    pub wallet_id: Uuid,
    pub token: Uuid
}

#[derive(Deserialize,Serialize, Debug)]
pub struct GrantKYCInputArgs {

    pub wallet_id: Uuid,
    pub token: Uuid // for backward comp with current client sdk
}


#[derive(Deserialize,Serialize, Debug)]
pub enum WithdrawalType {
    Fiat, // TODO: will enhance once I need to bring in Pretium
    Crypto
}

#[derive(Deserialize,Serialize, Debug)]
pub struct WithdrawTokensInputArgs {
    pub withdrawal_type: WithdrawalType,
    pub to: String, // if fiat is figured out, this can be the phone number
    pub amount: BigDecimal,
    pub token: String,
    pub from: Uuid
}

#[derive(Deserialize, Serialize, Debug)]
pub enum AccountsProcessorInput {
    CreateAccount(CreateCradleAccount),
    CreateAccountWallet(CreateCradleWalletInputArgs),
    UpdateAccountStatus(UpdateAccountStatusInputArgs),
    UpdateAccountType(UpdateAccountTypeInputArgs),
    UpdateAccountWalletStatusById(UpdateWalletStatusByIdInputArgs),
    UpdateAccountWalletStatusByAccount(UpdateWalletStatusByAccountIdInputArgs),
    DeleteAccount(DeleteAccountInputArgs),
    DeleteWallet(DeleteWalletInputArgs),
    GetAccount(GetAccountInputArgs),
    GetWallet(GetWalletInputArgs),
    GetAccounts, // TODO: add implementation later
    GetWallets, // TODO: implementations later
    AssociateTokenToWallet(AssociateTokenToWalletInputArgs),
    GrantKYC(GrantKYCInputArgs),
    WithdrawTokens(WithdrawTokensInputArgs),
    HandleAssociateAssets(Uuid),
    HandleKYCAssets(Uuid)
}


#[derive(Deserialize, Serialize, Debug )]
pub struct CreateAccountOutputArgs {
    pub id: Uuid,
    pub wallet_id: Uuid
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateAccountWalletOutputArgs {
    pub id: Uuid
}

#[derive(Deserialize, Serialize, Debug)]
pub enum AccountsProcessorOutput {
    CreateAccount(CreateAccountOutputArgs),
    CreateAccountWallet(CreateAccountWalletOutputArgs),
    UpdateAccountStatus,
    UpdateAccountType,
    UpdateAccountWalletStatus,
    UpdateAccountWalletStatusById,
    UpdateAccountWalletStatusByAccount,
    GetAccount(CradleAccountRecord),
    GetWallet(CradleWalletAccountRecord),
    GetAccounts,
    GetWallets,
    DeleteAccount,
    DeleteWallet,
    AssociateTokenToWallet,
    GrantKYC,
    WithdrawTokens,
    HandleAssociateAssets,
    HandleKYCAssets
}