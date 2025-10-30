use anyhow::anyhow;
use bigdecimal::BigDecimal;
use chrono::{NaiveDateTime, Duration};
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::{PgConnection, RunQueryDsl, ExpressionMethods};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::aggregators::aggregation_block::AggregationBlock;
use crate::aggregators::checkpoint;
use crate::aggregators::config::AggregatorsConfig;
use crate::aggregators::OHLCBlock;
use crate::market_time_series::db_types::{CreateMarketTimeSeriesRecord, DataProviderType, TimeSeriesInterval};
use crate::utils::app_config::AppConfig;
use crate::utils::traits::ActionProcessor;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AggregateTradesInputArgs {
    pub market_id: Uuid,
    pub asset_id: Uuid,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    pub interval: TimeSeriesInterval,
}

#[derive(Serialize, Deserialize)]
pub struct BackfillInputArgs {
    pub market_id: Uuid,
    pub asset_id: Uuid,
    pub interval: TimeSeriesInterval,
pub backfill_start: NaiveDateTime,
    pub backfill_end: NaiveDateTime,
}

#[derive(Serialize, Deserialize)]
pub enum AggregatorsProcessorInput {
    /// Single aggregation for a time window
    AggregateTrades(AggregateTradesInputArgs),
    /// Backfill historical data with checkpoint support
    BackfillTrades(BackfillInputArgs),
    /// Resume backfill from last checkpoint
    ResameBackfill(BackfillInputArgs),
    /// Clear checkpoint to restart from scratch
    ClearCheckpoint {
        market_id: Uuid,
        asset_id: Uuid,
        interval: TimeSeriesInterval,
    },
}

#[derive(Serialize, Deserialize)]
pub enum AggregatorsProcessorOutput {
    /// Single aggregation - returns created record ID
    AggregateTrades(Uuid),
    /// Backfill result - returns count of records created
    BackfillTrades(u32),
    /// Resume result - returns count of records created
    ResumeBackfill(u32),
    /// Checkpoint cleared
    ClearCheckpoint,
}

impl ActionProcessor<AggregatorsConfig, AggregatorsProcessorOutput> for AggregatorsProcessorInput {
    async fn process(
        &self,
        _app_config: &mut AppConfig,
        local_config: &mut AggregatorsConfig,
        conn: Option<&mut PooledConnection<ConnectionManager<PgConnection>>>,
    ) -> anyhow::Result<AggregatorsProcessorOutput> {
        let app_conn = conn.ok_or_else(|| anyhow!("Failed to get conn"))?;

        match self {
            AggregatorsProcessorInput::AggregateTrades(args) => {
                // Create an aggregation block that will fetch and aggregate trades
                let aggregation_block = create_aggregation_block(
                    &args.interval,
                    args.market_id,
                    args.asset_id,
                    args.start_time,
                    args.end_time,
                )?;

                // Process the aggregation block to get OHLC data
                let ohlc_block = aggregation_block.process(app_conn)?;

                // Persist the result to the markets_time_series table
                let record = CreateMarketTimeSeriesRecord {
                    market_id: args.market_id,
                    asset: args.asset_id,
                    open: ohlc_block.open,
                    high: ohlc_block.high,
                    low: ohlc_block.low,
                    close: ohlc_block.close,
                    volume: ohlc_block.volume,
                    start_time: args.start_time,
                    end_time: args.end_time,
                    interval: Some(args.interval.clone()),
                    data_provider_type: Some(DataProviderType::OrderBook),
                    data_provider: Some("orderbook_trades".to_string()),
                };

                let bar_id = diesel::insert_into(crate::schema::markets_time_series::table)
                    .values(&record)
                    .returning(crate::schema::markets_time_series::id)
                    .get_result::<Uuid>(app_conn)?;

                Ok(AggregatorsProcessorOutput::AggregateTrades(bar_id))
            }
            AggregatorsProcessorInput::BackfillTrades(args) => {
                backfill_trades(args, app_conn, local_config).await
            }
            AggregatorsProcessorInput::ResameBackfill(args) => {
                resume_backfill(args, app_conn, local_config).await
            }
            AggregatorsProcessorInput::ClearCheckpoint {
                market_id,
                asset_id,
                interval,
            } => {
                checkpoint::clear_checkpoint(*market_id, *asset_id, interval, app_conn).await?;
                Ok(AggregatorsProcessorOutput::ClearCheckpoint)
            }
        }
    }
}

/// Helper function to create an AggregationBlock from interval and time range
fn create_aggregation_block(
    interval: &TimeSeriesInterval,
    market_id: Uuid,
    asset_id: Uuid,
    start_time: NaiveDateTime,
    end_time: NaiveDateTime,
) -> anyhow::Result<AggregationBlock> {
    let interval_enum = match interval {
        TimeSeriesInterval::FifteenSecs => crate::aggregators::TimeSeriesAggregatorIntervals::FifteenSeconds,
        TimeSeriesInterval::ThirtySecs => crate::aggregators::TimeSeriesAggregatorIntervals::ThirtySeconds,
        TimeSeriesInterval::FortyFiveSecs => crate::aggregators::TimeSeriesAggregatorIntervals::FortyFiveSeconds,
        TimeSeriesInterval::OneMinute => crate::aggregators::TimeSeriesAggregatorIntervals::AMinute,
        TimeSeriesInterval::FiveMinutes => crate::aggregators::TimeSeriesAggregatorIntervals::FiveMinutes,
        TimeSeriesInterval::FifteenMinutes => crate::aggregators::TimeSeriesAggregatorIntervals::FifteenMinutes,
        TimeSeriesInterval::ThirtyMinutes => crate::aggregators::TimeSeriesAggregatorIntervals::ThirtyMinutes,
        TimeSeriesInterval::OneHour => crate::aggregators::TimeSeriesAggregatorIntervals::OneHour,
        TimeSeriesInterval::FourHours => crate::aggregators::TimeSeriesAggregatorIntervals::FourHours,
        TimeSeriesInterval::OneDay => crate::aggregators::TimeSeriesAggregatorIntervals::OneDay,
        TimeSeriesInterval::OneWeek => crate::aggregators::TimeSeriesAggregatorIntervals::OneDay,
    };

    Ok(AggregationBlock {
        start: start_time,
        end: end_time,
        index: 0,
        interval: interval_enum,
        sub_blocks: Box::new(Vec::new()),
        market_id,
        asset_id,
    })
}

/// Helper function to get duration from interval for backfill iteration
fn interval_to_duration(interval: &TimeSeriesInterval) -> Duration {
    match interval {
        TimeSeriesInterval::FifteenSecs => Duration::seconds(15),
        TimeSeriesInterval::ThirtySecs => Duration::seconds(30),
        TimeSeriesInterval::FortyFiveSecs => Duration::seconds(45),
        TimeSeriesInterval::OneMinute => Duration::minutes(1),
        TimeSeriesInterval::FiveMinutes => Duration::minutes(5),
        TimeSeriesInterval::FifteenMinutes => Duration::minutes(15),
        TimeSeriesInterval::ThirtyMinutes => Duration::minutes(30),
        TimeSeriesInterval::OneHour => Duration::hours(1),
        TimeSeriesInterval::FourHours => Duration::hours(4),
        TimeSeriesInterval::OneDay => Duration::days(1),
        TimeSeriesInterval::OneWeek => Duration::days(7),
    }
}

/// Backfill trades from backfill_start, saving checkpoints as we go
async fn backfill_trades(
    args: &BackfillInputArgs,
    app_conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    config: &AggregatorsConfig,
) -> anyhow::Result<AggregatorsProcessorOutput> {
    let interval_duration = interval_to_duration(&args.interval);
    let mut records_created = 0u32;
    let mut current_time = args.backfill_start;

    while current_time < args.backfill_end {
        let end_time = std::cmp::min(current_time + interval_duration, args.backfill_end);

        // Create and process aggregation block
        let aggregation_block = create_aggregation_block(
            &args.interval,
            args.market_id,
            args.asset_id,
            current_time,
            end_time,
        )?;

        let ohlc_block = aggregation_block.process(app_conn)?;

        // Only insert if there's data
        if ohlc_block.volume > BigDecimal::from(0) {
            let record = CreateMarketTimeSeriesRecord {
                market_id: args.market_id,
                asset: args.asset_id,
                open: ohlc_block.open,
                high: ohlc_block.high,
                low: ohlc_block.low,
                close: ohlc_block.close,
                volume: ohlc_block.volume,
                start_time: current_time,
                end_time,
                interval: Some(args.interval.clone()),
                data_provider_type: Some(DataProviderType::OrderBook),
                data_provider: Some("orderbook_trades_backfill".to_string()),
            };

            let _ = diesel::insert_into(crate::schema::markets_time_series::table)
                .values(&record)
                .returning(crate::schema::markets_time_series::id)
                .get_result::<Uuid>(app_conn)?;

            records_created += 1;
        }

        // Save checkpoint periodically
        if config.enable_checkpoints {
            checkpoint::save_checkpoint(
                args.market_id,
                args.asset_id,
                &args.interval,
                end_time,
                app_conn,
            )
            .await?;
        }

        current_time = end_time;
    }

    Ok(AggregatorsProcessorOutput::BackfillTrades(records_created))
}

/// Resume backfill from last checkpoint
async fn resume_backfill(
    args: &BackfillInputArgs,
    app_conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    config: &AggregatorsConfig,
) -> anyhow::Result<AggregatorsProcessorOutput> {
    // Get the last checkpoint
    let last_checkpoint = checkpoint::get_last_checkpoint(
        args.market_id,
        args.asset_id,
        &args.interval,
        app_conn,
    )
    .await?;

    // Start from checkpoint or beginning
    let actual_start = last_checkpoint.unwrap_or(args.backfill_start);

    if actual_start >= args.backfill_end {
        // Already completed
        return Ok(AggregatorsProcessorOutput::ResumeBackfill(0));
    }

    let interval_duration = interval_to_duration(&args.interval);
    let mut records_created = 0u32;
    let mut current_time = actual_start;

    while current_time < args.backfill_end {
        let end_time = std::cmp::min(current_time + interval_duration, args.backfill_end);

        // Create and process aggregation block
        let aggregation_block = create_aggregation_block(
            &args.interval,
            args.market_id,
            args.asset_id,
            current_time,
            end_time,
        )?;

        let ohlc_block = aggregation_block.process(app_conn)?;

        // Only insert if there's data
        if ohlc_block.volume > BigDecimal::from(0) {
            let record = CreateMarketTimeSeriesRecord {
                market_id: args.market_id,
                asset: args.asset_id,
                open: ohlc_block.open,
                high: ohlc_block.high,
                low: ohlc_block.low,
                close: ohlc_block.close,
                volume: ohlc_block.volume,
                start_time: current_time,
                end_time,
                interval: Some(args.interval.clone()),
                data_provider_type: Some(DataProviderType::OrderBook),
                data_provider: Some("orderbook_trades_resume".to_string()),
            };

            let _ = diesel::insert_into(crate::schema::markets_time_series::table)
                .values(&record)
                .returning(crate::schema::markets_time_series::id)
                .get_result::<Uuid>(app_conn)?;

            records_created += 1;
        }

        // Save checkpoint periodically
        if config.enable_checkpoints {
            checkpoint::save_checkpoint(
                args.market_id,
                args.asset_id,
                &args.interval,
                end_time,
                app_conn,
            )
            .await?;
        }

        current_time = end_time;
    }

    Ok(AggregatorsProcessorOutput::ResumeBackfill(records_created))
}
