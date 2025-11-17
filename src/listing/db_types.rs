use crate::schema::cradlelistedcompanies as CradleCompanyTable;
use crate::schema::cradlenativelistings as CradleNativeListingTable;
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Queryable, QueryableByName, Identifiable)]
#[diesel(table_name = CradleCompanyTable)]
pub struct CompanyRow {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub listed_at: Option<NaiveDateTime>,
    pub legal_documents: String,
    pub beneficiary_wallet: Uuid,
}

#[derive(Serialize, Deserialize, Insertable)]
#[diesel(table_name = CradleCompanyTable)]
pub struct CreateCompany {
    pub name: String,
    pub description: String,
    pub legal_documents: String,
    pub beneficiary_wallet: Uuid,
}

#[derive(Serialize, Deserialize, DbEnum, Debug, Clone)]
#[ExistingTypePath = "crate::schema::sql_types::ListingStatus"]
#[serde(rename_all = "lowercase")]
pub enum ListingStatus {
    Pending,
    Open,
    Closed,
    Paused,
    Cancelled,
}

#[derive(Serialize, Deserialize, Debug, Clone, Queryable, QueryableByName, Identifiable)]
#[diesel(table_name = CradleNativeListingTable)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CradleNativeListingRow {
    pub id: Uuid,
    pub listing_contract_id: String,
    pub name: String,
    pub description: String,
    pub documents: String,
    pub company: Uuid,
    pub status: ListingStatus,
    pub created_at: NaiveDateTime,
    pub opened_at: Option<NaiveDateTime>,
    pub stopped_at: Option<NaiveDateTime>,
    pub listed_asset: Uuid,
    pub purchase_with_asset: Uuid,
    pub purchase_price: BigDecimal,
    pub max_supply: BigDecimal,
    pub treasury: Uuid,
    pub shadow_asset: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone, Insertable)]
#[diesel(table_name = CradleNativeListingTable)]
pub struct CreateCraldeNativeListing {
    pub listing_contract_id: String,
    pub name: String,
    pub description: String,
    pub documents: String,
    pub company: Uuid,
    pub status: ListingStatus,
    pub opened_at: Option<NaiveDateTime>,
    pub stopped_at: Option<NaiveDateTime>,
    pub listed_asset: Uuid,
    pub purchase_with_asset: Uuid,
    pub purchase_price: BigDecimal,
    pub max_supply: BigDecimal,
    pub treasury: Uuid,
    pub shadow_asset: Uuid,
}
