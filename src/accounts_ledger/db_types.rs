use crate::schema::accountassetsledger as AccountAssetsLedgerTable;
use anyhow::Result;
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::{
    prelude::{Identifiable, Insertable, Queryable},
    r2d2::{ConnectionManager, PooledConnection},
};
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::TransactionType"]
#[serde(rename_all = "lowercase")]
pub enum AccountLedgerTransactionType {
    Lock,
    UnLock,
    Lend,
    Borrow,
    Repay,
    Liquidate,
    FillOrder,
    Withdraw,
    Transfer,
    BuyListed,
    SellListed,
    ListingBeneficiaryWithdrawal,
}

#[derive(Serialize, Deserialize, Debug, Clone, Queryable, Identifiable)]
#[diesel(table_name = AccountAssetsLedgerTable)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct LedgerRow {
    pub id: Uuid,
    pub timestamp: NaiveDateTime,
    pub transaction: Option<String>,
    pub from_address: String,
    pub to_address: String,
    pub asset: Uuid,
    pub transaction_type: AccountLedgerTransactionType,
    pub amount: BigDecimal,
    #[serde(rename = "ref")]
    #[diesel(column_name = "ref")]
    pub ref_value: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Insertable)]
#[diesel(table_name = AccountAssetsLedgerTable)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CreateLedgerEntry {
    pub transaction: Option<String>,
    pub from_address: String,
    pub to_address: String,
    pub asset: Uuid,
    pub transaction_type: AccountLedgerTransactionType,
    pub amount: BigDecimal,
    pub refference: Option<String>,
}

impl CreateLedgerEntry {
    pub fn insert(
        &self,
        conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    ) -> Result<LedgerRow> {
        let data = self.clone();
        let entry = diesel::insert_into(AccountAssetsLedgerTable::table)
            .values(&data)
            .get_result::<LedgerRow>(conn)?;

        Ok(entry)
    }
}
