use anyhow::Result;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde::{de::DeserializeOwned, Serialize};

pub type RedisPool = ConnectionManager;

/// Initialize a Redis connection manager from the REDIS_URL env var.
/// Falls back to "redis://127.0.0.1:6379" if not set.
pub async fn init_redis() -> Result<RedisPool> {
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let client = redis::Client::open(redis_url)?;
    let manager = ConnectionManager::new(client).await?;
    Ok(manager)
}

/// Get a value from Redis cache, deserializing from JSON.
/// Returns None on cache miss or any error (fail-open).
pub async fn cache_get<T: DeserializeOwned>(conn: &RedisPool, key: &str) -> Option<T> {
    let mut conn = conn.clone();
    let result: Option<String> = conn.get(key).await.ok()?;
    let json_str = result?;
    serde_json::from_str(&json_str).ok()
}

/// Set a value in Redis cache with a TTL in seconds.
/// Errors are silently ignored (fail-open).
pub async fn cache_set<T: Serialize>(conn: &RedisPool, key: &str, value: &T, ttl_secs: u64) {
    if let Ok(json) = serde_json::to_string(value) {
        let mut conn = conn.clone();
        let _: Result<(), _> = conn.set_ex(key, json, ttl_secs).await;
    }
}

/// Delete a key from Redis cache. Errors are silently ignored.
pub async fn cache_del(conn: &RedisPool, key: &str) {
    let mut conn = conn.clone();
    let _: Result<(), _> = conn.del(key).await;
}
