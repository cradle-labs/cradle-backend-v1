use bigdecimal::{BigDecimal, ToPrimitive};
use chrono::{NaiveDateTime, Utc};
use contract_integrator::utils::functions::asset_lending::UpdateOracleArgs;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::{asset_book::operations::get_asset, big_to_u64, schema::lending_pool_oracle_prices as lpop, utils::commons::{DbConn, TaskWallet}};
use anyhow::{Result, anyhow};

#[derive(Serialize, Deserialize, Queryable, Identifiable, Debug, Clone)]
#[diesel(table_name= lpop)]
pub struct PriceOracle {
    pub id: Uuid,
    pub lending_pool_id: Uuid,
    pub asset_id: Uuid,
    pub price: BigDecimal,
    pub recorded_at: NaiveDateTime
}


#[derive(Serialize, Deserialize, Debug, Insertable)]
#[diesel(table_name=lpop)]
pub struct CreatePriceOracle {
    pub lending_pool_id: Uuid,
    pub asset_id: Uuid,
    pub price: BigDecimal,
    pub recorded_at: NaiveDateTime
}


pub fn create_price_oracle<'a>(conn: DbConn<'a>, args: CreatePriceOracle)->Result<Uuid> {

    let res_id = diesel::insert_into(lpop::table).values(&args).returning(lpop::dsl::id).get_result::<Uuid>(conn)?;

    Ok(res_id)
}

pub fn update_price_oracle<'a>(conn: DbConn<'a>, lending_pool: Uuid, asset: Uuid, price: BigDecimal)->Result<()> {

    let new_oracle = CreatePriceOracle {
        lending_pool_id: lending_pool,
        asset_id: asset,
        price,
        recorded_at: Utc::now().naive_utc()
    };


    diesel::insert_into(lpop::table)
        .values(&new_oracle)
        .on_conflict((lpop::dsl::lending_pool_id, lpop::dsl::asset_id))
        .do_update()
        .set(lpop::dsl::price.eq(&new_oracle.price))
        .execute(conn)?;

    Ok(())
}

pub fn get_price_oracle<'a>(conn: DbConn<'a>, lending_pool: Uuid, asset: Uuid)->Result<PriceOracle> {
    let res = lpop::dsl::lending_pool_oracle_prices.filter(
        lpop::lending_pool_id.eq(lending_pool).and(
            lpop::asset_id.eq(asset)
        )
    ).get_result::<PriceOracle>(conn)?;

    Ok(res)
}

pub async fn publish_price<'a>(conn: DbConn<'a>, wallet: TaskWallet<'a>, lending_pool: Uuid, asset_id: Uuid, price: BigDecimal) -> Result<()>{

    let pool = crate::lending_pool::operations::get_pool(conn, lending_pool).await?;
    let asset = get_asset(conn, asset_id).await?;
    let as_u64 = big_to_u64!(price)?; 

    let res = contract_integrator::operations::asset_lending::update_oracle(UpdateOracleArgs {
        asset: asset.token,
        contract_id: pool.pool_contract_id,
        multiplier: as_u64
    }, wallet).await?;

    println!("TX :: {:?}", res.transaction_id);

    update_price_oracle(conn, lending_pool, asset_id, price)?;

    Ok(())
}