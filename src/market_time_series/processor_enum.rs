use bigdecimal::BigDecimal;
use chrono::Duration;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::market_time_series::db_types::{CreateMarketTimeSeriesRecord, MarketTimeSeriesRecord, TimeSeriesInterval};


#[derive(Serialize,Deserialize)]
pub struct GetHistoryInputArgs {
    pub market_id: Uuid,
    pub asset: Uuid,
    pub duration_secs: BigDecimal,
    pub interval: TimeSeriesInterval
}

#[derive(Deserialize, Serialize)]
pub enum MarketTimeSeriesProcessorInput {
    AddRecord(CreateMarketTimeSeriesRecord),
    GetHistory(GetHistoryInputArgs)
}

#[derive(Deserialize, Serialize)]
pub enum MarketTimeSeriesProcessorOutput {
    AddRecord(Uuid),
    GetHistory(Vec<MarketTimeSeriesRecord>)
}