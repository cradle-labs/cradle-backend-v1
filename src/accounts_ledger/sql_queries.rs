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
SELECT
    COALESCE(
        (
            COALESCE(SUM(
                CASE
                    WHEN to_address   = $1
                     AND asset        = $2
                     AND transaction_type = 'lock'
                    THEN amount
                    ELSE 0
                END
            ), 0)
            +
            COALESCE(SUM(
                CASE
                    WHEN from_address = $1
                     AND asset        = $2
                     AND transaction_type = 'lend'
                    THEN amount
                    ELSE 0
                END
            ), 0)
            -
            COALESCE(SUM(
                CASE
                    WHEN to_address = $1
                     AND asset        = $2
                     AND transaction_type = 'unlock'
                    THEN amount
                    ELSE 0
                END
            ), 0)
        ),
        0
    ) AS total
FROM accountassetsledger;
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
    let mut res = diesel::sql_query(DEDUCTIONS_QUERY)
        .bind::<diesel::sql_types::Text, _>(address)
        .bind::<diesel::sql_types::Uuid, _>(asset)
        .get_result::<DeductionResult>(conn)?;

    res.total = res.total.abs();

    Ok(res)
}
