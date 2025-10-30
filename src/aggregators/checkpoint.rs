use anyhow::{anyhow, Result};
use chrono::NaiveDateTime;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::PgConnection;
use uuid::Uuid;
use crate::market_time_series::db_types::TimeSeriesInterval;
use crate::utils::kvstore;

/// Builds a checkpoint key for storing last processed timestamp
fn build_checkpoint_key(market_id: Uuid, asset_id: Uuid, interval: &TimeSeriesInterval) -> String {
    format!(
        "aggregator:{}:{}:{}:last_processed",
        market_id, asset_id, interval_to_string(interval)
    )
}

/// Converts TimeSeriesInterval to string for checkpoint key
fn interval_to_string(interval: &TimeSeriesInterval) -> String {
    match interval {
        TimeSeriesInterval::FifteenSecs => "15secs".to_string(),
        TimeSeriesInterval::ThirtySecs => "30secs".to_string(),
        TimeSeriesInterval::FortyFiveSecs => "45secs".to_string(),
        TimeSeriesInterval::OneMinute => "1min".to_string(),
        TimeSeriesInterval::FiveMinutes => "5min".to_string(),
        TimeSeriesInterval::FifteenMinutes => "15min".to_string(),
        TimeSeriesInterval::ThirtyMinutes => "30min".to_string(),
        TimeSeriesInterval::OneHour => "1hr".to_string(),
        TimeSeriesInterval::FourHours => "4hr".to_string(),
        TimeSeriesInterval::OneDay => "1day".to_string(),
        TimeSeriesInterval::OneWeek => "1week".to_string(),
    }
}

/// Retrieves the last processed timestamp for a market/asset/interval combination
///
/// Returns None if no checkpoint exists yet
pub async fn get_last_checkpoint(
    market_id: Uuid,
    asset_id: Uuid,
    interval: &TimeSeriesInterval,
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
) -> Result<Option<NaiveDateTime>> {
    let key = build_checkpoint_key(market_id, asset_id, interval);

    match kvstore::get_value_kv(conn, &key).await {
        Ok(Some(timestamp_str)) => {
            // Parse the timestamp string back to NaiveDateTime
            let parsed = NaiveDateTime::parse_from_str(&timestamp_str, "%Y-%m-%d %H:%M:%S")
                .map_err(|e| anyhow!("Failed to parse checkpoint timestamp: {}", e))?;
            Ok(Some(parsed))
        }
        Ok(None) => Ok(None),
        Err(e) => {
            // If key doesn't exist in kvstore, return None (not an error)
            match e.source() {
                Some(_) => Ok(None),
                None => Err(e),
            }
        }
    }
}

/// Saves the last processed timestamp for a market/asset/interval combination
pub async fn save_checkpoint(
    market_id: Uuid,
    asset_id: Uuid,
    interval: &TimeSeriesInterval,
    last_processed: NaiveDateTime,
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
) -> Result<()> {
    let key = build_checkpoint_key(market_id, asset_id, interval);
    let timestamp_str = last_processed.format("%Y-%m-%d %H:%M:%S").to_string();

    kvstore::set_value_kv(conn, &key, &timestamp_str).await?;
    Ok(())
}

/// Clears a checkpoint (useful for restarting aggregation from scratch)
pub async fn clear_checkpoint(
    market_id: Uuid,
    asset_id: Uuid,
    interval: &TimeSeriesInterval,
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
) -> Result<()> {
    let key = build_checkpoint_key(market_id, asset_id, interval);

    // Set value to empty string to effectively delete it
    kvstore::set_value_kv(conn, &key, "").await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkpoint_key_generation() {
        let market_id = Uuid::nil();
        let asset_id = Uuid::nil();
        let interval = TimeSeriesInterval::FifteenSecs;

        let key = build_checkpoint_key(market_id, asset_id, &interval);
        assert!(key.contains("aggregator:"));
        assert!(key.contains("15secs"));
        assert!(key.contains("last_processed"));
    }

    #[test]
    fn test_interval_to_string() {
        assert_eq!(interval_to_string(&TimeSeriesInterval::FifteenSecs), "15secs");
        assert_eq!(interval_to_string(&TimeSeriesInterval::OneDay), "1day");
        assert_eq!(interval_to_string(&TimeSeriesInterval::OneWeek), "1week");
    }
}
