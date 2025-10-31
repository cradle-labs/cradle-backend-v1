use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::schema::cradleaccounts as CradleAccountsTable;
use crate::schema::cradlewalletaccounts as CradleWalletAccountsTable;

#[derive(DbEnum, Deserialize, Serialize, Debug, Clone)]
#[ExistingTypePath="crate::schema::sql_types::Cradleaccounttype"]
#[serde(rename_all = "lowercase")]
pub enum CradleAccountType {
    Retail,
    Institutional
}


#[derive(DbEnum, Deserialize, Serialize, Debug, Clone)]
#[ExistingTypePath="crate::schema::sql_types::Cradlewalletstatus"]
#[serde(rename_all = "lowercase")]
pub enum CradleWalletStatus {
    Active,
    #[serde(rename = "inactive")]
    Inactive,
    Suspended
}


#[derive(DbEnum, Deserialize, Serialize, Debug, Clone)]
#[ExistingTypePath="crate::schema::sql_types::Cradleaccountstatus"]
#[serde(rename_all = "lowercase")]
pub enum CradleAccountStatus {
    #[serde(rename="unverified")]
    Unverified,
    Verified,
    Suspended,
    Closed
}


#[derive(Serialize, Deserialize, Queryable, Debug, Clone, Identifiable, QueryableByName)]
#[diesel(table_name = CradleAccountsTable)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CradleAccountRecord {
    pub id: Uuid,
    pub linked_account_id: String,
    pub created_at: NaiveDateTime,
    pub account_type: CradleAccountType,
    pub status: CradleAccountStatus
}


#[derive(Serialize, Deserialize, Insertable, Debug, Clone)]
#[diesel(table_name = CradleAccountsTable)]
pub struct CreateCradleAccount {
    pub linked_account_id: String,
    pub account_type: Option<CradleAccountType>,
    pub status: Option<CradleAccountStatus>
}

#[derive(Serialize, Deserialize, QueryableByName, Debug, Clone, Identifiable, Queryable)]
#[diesel(table_name = CradleWalletAccountsTable)]
pub struct CradleWalletAccountRecord {
    pub id: Uuid,
    pub cradle_account_id: Uuid,
    pub address: String,
    pub contract_id: String,
    pub created_at: NaiveDateTime,
    pub status: CradleWalletStatus
}


#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[diesel(table_name = CradleWalletAccountsTable)]
pub struct CreateCradleWalletAccount {
    pub cradle_account_id: Uuid,
    pub address: String,
    pub contract_id: String,
    pub status: Option<CradleWalletStatus>
}


#[derive(Serialize,Deserialize, Queryable, Identifiable, QueryableByName, Clone, Debug)]
#[diesel(table_name = crate::schema::accountassetbook)]
pub struct AccountAssetBookRecord {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub account_id: Uuid,
    pub associated: bool,
    pub kyced: bool,
    pub associated_at: Option<NaiveDateTime>,
    pub kyced_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

#[derive(Serialize,Deserialize, Insertable, Clone, Debug)]
#[diesel(table_name = crate::schema::accountassetbook)]
pub struct CreateAccountAssetBook {
    pub asset_id: Uuid,
    pub account_id: Uuid,
    pub associated: Option<bool>,
    pub kyced: Option<bool>,
    pub associated_at: Option<NaiveDateTime>,
    pub kyced_at: Option<NaiveDateTime>,
}