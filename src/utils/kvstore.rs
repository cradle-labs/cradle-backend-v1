use diesel::Queryable;
use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use diesel::r2d2::{PooledConnection, ConnectionManager};
use crate::schema::kvstore as KvStoreTable;
use anyhow::Result;

#[derive(Serialize, Deserialize, Queryable)]
#[diesel(table_name = KvStoreTable)]
pub struct KvStoreRecord {
    pub key: String,
    pub value: Option<String>
}


#[derive(Serialize, Deserialize, Insertable) ]
#[diesel(table_name = KvStoreTable)]
pub struct KvStoreInsertion {
    pub key: String,
    pub value: Option<String>
}


pub async fn set_value_kv(conn: &mut PooledConnection<ConnectionManager<PgConnection>>, key_v: &str, value_v: &str)->Result<()> {
    use crate::schema::kvstore::dsl::*;

    let v = KvStoreInsertion {
        key: String::from(key_v),
        value: Some(String::from(value_v))
    };

    // Try to update first, if no rows affected, insert
    let rows_updated = diesel::update(kvstore.filter(key.eq(key_v)))
        .set(value.eq(Some(String::from(value_v))))
        .execute(conn)?;

    if rows_updated == 0 {
        diesel::insert_into(KvStoreTable::table).values(&v).execute(conn)?;
    }

    Ok(())
}


pub async fn get_value_kv(conn: &mut PooledConnection<ConnectionManager<PgConnection>>, key_v: &str)->Result<Option<String>> {
    use crate::schema::kvstore::dsl::*;
    let res = kvstore.filter(key.eq(key_v)).get_result::<KvStoreRecord>(conn)?;
    Ok(res.value)
}
