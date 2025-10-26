use anyhow::anyhow;
use diesel::prelude::*;
use diesel::{ExpressionMethods, PgConnection, RunQueryDsl};
use diesel::r2d2::{ConnectionManager, PooledConnection};
use uuid::Uuid;
use crate::market::config::MarketsConfig;
use crate::market::db_types::MarketRecord;
use crate::market::processor_enums::{MarketProcessorInput, MarketProcessorOutput};
use crate::utils::app_config::AppConfig;
use crate::utils::traits::ActionProcessor;
use crate::schema::markets as MarketsTable;
impl ActionProcessor<MarketsConfig, MarketProcessorOutput> for MarketProcessorInput {
    async fn process(&self, app_config: &mut AppConfig, local_config: &mut MarketsConfig, conn: Option<&mut PooledConnection<ConnectionManager<PgConnection>>>) -> anyhow::Result<MarketProcessorOutput> {
        let app_conn = conn.ok_or_else(||anyhow!("Db Connection not found"))?;
        match self {
            MarketProcessorInput::CreateMarket(create_args) => {
                use crate::schema::markets::dsl::*;
                let res = diesel::insert_into(MarketsTable::table).values(create_args).returning(id).get_result::<Uuid>(app_conn)?;
                Ok(MarketProcessorOutput::CreateMarket(res))
            }
            MarketProcessorInput::UpdateMarketStatus(update_args ) => {
                use crate::schema::markets::dsl::*;
                
                let _ = diesel::update(MarketsTable::table).filter(
                    id.eq(update_args.market_id)
                ).set(
                    market_status.eq(update_args.status.clone())
                ).execute(app_conn)?;
                
                Ok(MarketProcessorOutput::UpdateMarketStatus)
            }
            MarketProcessorInput::UpdateMarketType(update_args) => {
                use crate::schema::markets::dsl::*;

                let _ = diesel::update(MarketsTable::table).filter(
                    id.eq(update_args.market_id)
                ).set(
                    market_type.eq(update_args.market_type.clone())
                ).execute(app_conn)?;

                Ok(MarketProcessorOutput::UpdateMarketType)
            }
            MarketProcessorInput::UpdateMarketRegulation(update_args) => {
                use crate::schema::markets::dsl::*;

                let _ = diesel::update(MarketsTable::table).filter(
                    id.eq(update_args.market_id)
                ).set(
                    market_regulation.eq(update_args.regulation.clone())
                ).execute(app_conn)?;

                Ok(MarketProcessorOutput::UpdateMarketRegulation)
            }
            MarketProcessorInput::GetMarket(market_id) => {
                use crate::schema::markets::dsl::*;
                
                let result = markets.filter(
                    id.eq(market_id)
                ).get_result::<MarketRecord>(app_conn)?;
                
                
                Ok(MarketProcessorOutput::GetMarket(result))
            }
            MarketProcessorInput::GetMarkets(filter) => {
                use crate::schema::markets::dsl::*;
                let mut query = markets.into_boxed();
                if let Some(status_filter) = &filter.status {
                    query = query.filter(market_status.eq(status_filter.clone()));
                }
                if let Some(type_filter) = &filter.market_type {
                    query = query.filter(market_type.eq(type_filter.clone()));
                }
                if let Some(regulation_filter) = &filter.regulation {
                    query = query.filter(market_regulation.eq(regulation_filter.clone()));
                }

                let results = query.get_results::<MarketRecord>(app_conn)?;

                Ok(MarketProcessorOutput::GetMarkets(results) )
            }
        }
    }
}