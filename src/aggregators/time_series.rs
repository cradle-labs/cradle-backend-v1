use std::ops::Add;
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use clap::{Parser, ValueEnum};
use diesel::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use serde::{Deserialize, Serialize};
use anyhow::{anyhow, Result};
use uuid::Uuid;
use crate::aggregators::ohlc_queries;

/**
 * Mechanic:
* - Create a time block, consisting of a discreet start and end time
*   divide the time block by the interval, and get all starts and ends
*   for each time interval get the orders that occured during that period
*   carry out aggregation to determine ohlc, and volume values
 */

#[derive(Parser, Clone)]
pub struct TimeSeriesInputArgs {
    #[clap(long,env)]
    pub aggregator_start_time: NaiveDateTime,
}