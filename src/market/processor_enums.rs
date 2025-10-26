use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::market::db_types::{CreateMarket, MarketRecord, MarketRegulation, MarketStatus, MarketType};



#[derive(Deserialize,Serialize)]
pub struct UpdateMarketStatusInputArgs {
    pub market_id: Uuid,
    pub status: MarketStatus
}

#[derive(Deserialize,Serialize)]
pub struct UpdateMarketTypeInputArgs {
    pub market_id: Uuid,
    pub market_type: MarketType
}

#[derive(Deserialize,Serialize)]
pub struct UpdateMarketRegulationInputArgs {
    pub market_id: Uuid,
    pub regulation: MarketRegulation
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GetMarketsFilter {
    pub status: Option<MarketStatus>,
    pub market_type: Option<MarketType>,
    pub regulation: Option<MarketRegulation>
}

pub enum MarketProcessorInput {
    CreateMarket(CreateMarket),
    UpdateMarketStatus(UpdateMarketStatusInputArgs),
    UpdateMarketType(UpdateMarketTypeInputArgs),
    UpdateMarketRegulation(UpdateMarketRegulationInputArgs),
    GetMarket(Uuid),
    GetMarkets(GetMarketsFilter)
}



pub enum MarketProcessorOutput {
    CreateMarket(Uuid),
    UpdateMarketStatus,
    UpdateMarketType,
    UpdateMarketRegulation,
    GetMarket(MarketRecord),
    GetMarkets(Vec<MarketRecord>)
}