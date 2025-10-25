use diesel::{r2d2, PgConnection};
use diesel::r2d2::{ConnectionManager, PooledConnection};
use anyhow::Result;

pub fn get_conn(pool: r2d2::Pool<ConnectionManager<PgConnection>>)->Result<PooledConnection<ConnectionManager<PgConnection>>> {
    // TODO: add additional checks around this
    let conn = pool.get()?;
    
    Ok(conn)
}