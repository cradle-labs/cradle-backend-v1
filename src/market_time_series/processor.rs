use anyhow::anyhow;
use bigdecimal::ToPrimitive;
use chrono::{Duration, Utc};
use diesel::{ExpressionMethods, PgConnection, RunQueryDsl};
use diesel::r2d2::{ConnectionManager, PooledConnection};
use uuid::Uuid;
use diesel::prelude::*;
use crate::market_time_series::config::MarketTimeSeriesConfig;
use crate::market_time_series::db_types::MarketTimeSeriesRecord;
use crate::market_time_series::processor_enum::{MarketTimeSeriesProcessorInput, MarketTimeSeriesProcessorOutput};
use crate::utils::app_config::AppConfig;
use crate::utils::traits::ActionProcessor;
use crate::schema::markets_time_series as MarketTimeSeriesTable;

impl ActionProcessor<MarketTimeSeriesConfig, MarketTimeSeriesProcessorOutput> for MarketTimeSeriesProcessorInput {
    async fn process(&self, app_config: &mut AppConfig, local_config: &mut MarketTimeSeriesConfig, conn: Option<&mut PooledConnection<ConnectionManager<PgConnection>>>) -> anyhow::Result<MarketTimeSeriesProcessorOutput> {
        let app_conn = conn.ok_or_else(||anyhow!("Failed to get conn"))?;

        match self {
            MarketTimeSeriesProcessorInput::AddRecord(args) => {
                // TODO: add publishing to websocket for realtime tracking
                use crate::schema::markets_time_series::dsl::*;

                let bar_id = diesel::insert_into(MarketTimeSeriesTable::table).values(args).returning(id).get_result::<Uuid>(app_conn)?;

                Ok(MarketTimeSeriesProcessorOutput::AddRecord(bar_id))
            }
            MarketTimeSeriesProcessorInput::GetHistory(args) => {
                let duration = Duration::seconds(args.duration_secs.to_i64().ok_or_else(||anyhow!("Failed to unwrap duration"))?);
                let start = Utc::now().naive_utc() - duration;

                use crate::schema::markets_time_series::dsl::*;

                let bars = markets_time_series.filter(
                        market_id.eq(args.market_id.clone()).and(
                            asset.eq(args.asset.clone()).and(
                                interval.eq(args.interval.clone()).and(
                                    start_time.ge(start)
                                )
                            )
                        )
                ).get_results::<MarketTimeSeriesRecord>(app_conn)?;

                Ok(MarketTimeSeriesProcessorOutput::GetHistory(bars))
            }
        }
    }
}