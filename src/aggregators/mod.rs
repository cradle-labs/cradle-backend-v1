pub mod time_series;
pub mod aggregation_block;
pub mod ohlc_queries;
pub mod processor;
pub mod checkpoint;
pub mod config;

// Re-export commonly used types
pub use aggregation_block::{AggregationBlock, OHLCBlock, TimeSeriesAggregatorIntervals};
pub use ohlc_queries::{get_trades_for_market_asset, calculate_ohlc, TradeDataForAggregation};
pub use config::AggregatorsConfig;
pub use processor::{AggregatorsProcessorInput, AggregatorsProcessorOutput, AggregateTradesInputArgs, BackfillInputArgs};