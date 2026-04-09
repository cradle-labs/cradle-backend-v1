use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use diesel::PgConnection;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use uuid::Uuid;
use std::ops::Add;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum TimeSeriesAggregatorIntervals {
    #[serde(rename="15secs")]
    FifteenSeconds,
    #[serde(rename="30secs")]
    ThirtySeconds,
    #[serde(rename="45secs")]
    FortyFiveSeconds,
    #[serde(rename="1min")]
    AMinute,
    #[serde(rename="5min")]
    FiveMinutes,
    #[serde(rename="15min")]
    FifteenMinutes,
    #[serde(rename="30min")]
    ThirtyMinutes,
    #[serde(rename="1hr")]
    OneHour,
    #[serde(rename="4hr")]
    FourHours,
    #[serde(rename="1day")]
    OneDay,
    #[serde(rename="1week")]
    OneWeek
}

#[derive(Clone, Debug)]
pub struct OHLCBlock {
    pub open: BigDecimal,
    pub high: BigDecimal,
    pub low: BigDecimal,
    pub close: BigDecimal,
    pub volume: BigDecimal,
    pub market: String,
    pub asset: String,
    pub start_time: Option<NaiveDateTime>,
}

impl Default for OHLCBlock {
    fn default() -> Self {
        Self {
            open: BigDecimal::from(0),
            high: BigDecimal::from(0),
            low: BigDecimal::from(0),
            close: BigDecimal::from(0),
            volume: BigDecimal::from(0),
            market: String::new(),
            asset: String::new(),
            start_time: None,
        }
    }
}

impl OHLCBlock {

    pub fn sum(blocks: Vec<OHLCBlock>) -> OHLCBlock {
        if blocks.is_empty() {
            return OHLCBlock::default();
        }

        // For OHLC aggregation from sub-blocks:
        // - Open: open price of first block (by time)
        // - High: max high price across all blocks
        // - Low: min low price across all blocks
        // - Close: close price of last block (by time)
        // - Volume: sum of all volumes
        let mut sorted_blocks = blocks.clone();
        sorted_blocks.sort_by(|a, b| {
            a.start_time.cmp(&b.start_time)
        });

        let open = sorted_blocks.first().map(|b| b.open.clone()).unwrap_or_default();
        let close = sorted_blocks.last().map(|b| b.close.clone()).unwrap_or_default();

        let high = sorted_blocks.iter()
            .map(|b| b.high.clone())
            .max()
            .unwrap_or_default();

        let low = sorted_blocks.iter()
            .map(|b| b.low.clone())
            .min()
            .unwrap_or_default();

        let volume = sorted_blocks.iter().fold(BigDecimal::from(0), |acc, x| acc.add(&x.volume));

        OHLCBlock {
            open,
            high,
            low,
            close,
            volume,
            market: sorted_blocks.first().map(|b| b.market.clone()).unwrap_or_default(),
            asset: sorted_blocks.first().map(|b| b.asset.clone()).unwrap_or_default(),
            start_time: sorted_blocks.first().and_then(|b| b.start_time),
        }
    }
}


#[derive(Clone, Debug)]
pub struct AggregationBlock {
    pub start: NaiveDateTime,
    pub end: NaiveDateTime,
    pub index: u64,
    pub interval: TimeSeriesAggregatorIntervals,
    pub sub_blocks: Box<Vec<AggregationBlock>>,
    pub market_id: Uuid,
    pub asset_id: Uuid,
}

impl AggregationBlock {

    pub fn process(&self, conn: &mut PooledConnection<ConnectionManager<PgConnection>>) -> anyhow::Result<OHLCBlock> {
        // Query raw trades directly for this block's time range, regardless of interval.
        // This is correct and efficient: the DB filters trades by [start, end),
        // and we compute OHLC over whatever trades fall in that window.
        use crate::aggregators::ohlc_queries;

        let trades = ohlc_queries::get_trades_for_market_asset(
            self.market_id,
            self.asset_id,
            self.start,
            self.end,
            conn,
        )?;

        if trades.is_empty() {
            return Ok(OHLCBlock {
                open: BigDecimal::from(0),
                high: BigDecimal::from(0),
                low: BigDecimal::from(0),
                close: BigDecimal::from(0),
                volume: BigDecimal::from(0),
                market: self.market_id.to_string(),
                asset: self.asset_id.to_string(),
                start_time: Some(self.start),
            });
        }

        let (open, high, low, close, volume) = ohlc_queries::calculate_ohlc(&trades)?;

        Ok(OHLCBlock {
            open,
            high,
            low,
            close,
            volume,
            market: self.market_id.to_string(),
            asset: self.asset_id.to_string(),
            start_time: Some(self.start),
        })
    }

}