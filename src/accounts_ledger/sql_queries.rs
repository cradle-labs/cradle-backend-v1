use anyhow::Result;
use bigdecimal::BigDecimal;
use diesel::prelude::*;
use diesel::{
    PgConnection,
    prelude::QueryableByName,
    r2d2::{ConnectionManager, PooledConnection},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const DEDUCTIONS_QUERY: &str = r"
with locked_amount as (
    select sum(amount) as total from accountassetsledger where from_address = $1 and asset = $2 and transaction_type = 'lock'
),
unlocked as (
    select sum(amount) as total from accountassetsledger where to_address = $1 and asset = $2 and transaction_type = 'unlock'
    ),
lent as (
    select sum(amount) as total from accountassetsledger where from_address = $1 and asset = $2 and transaction_type = 'lend'
)
select coalesce(((l.total + le.total) - u.total), 0) as total from locked_amount as l
cross join unlocked as u
cross join lent as le;
";

#[derive(Serialize, Deserialize, QueryableByName)]
#[diesel(table_name=crate::schema::accountassetsledger)]
pub struct DeductionResult {
    #[diesel(sql_type = diesel::sql_types::Numeric)]
    pub total: BigDecimal,
}

pub fn get_deductions(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    address: String,
    asset: Uuid,
) -> Result<DeductionResult> {
    let res = diesel::sql_query(DEDUCTIONS_QUERY)
        .bind::<diesel::sql_types::Text, _>(address)
        .bind::<diesel::sql_types::Uuid, _>(asset)
        .get_result::<DeductionResult>(conn)?;

    Ok(res)
}
