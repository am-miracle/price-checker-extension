//! Redis caching layer for price comparison results.
//!
//! Implements cache-aside pattern with TTL for performance optimization.

use deadpool_redis::{Config as RedisConfig, Pool, Runtime};
use redis::AsyncCommands;

use sha2::{Digest, Sha256};

use crate::{AppError, PriceComparisonResult};

/// Creates a Redis connection pool.
///
/// # Arguments
/// * `redis_url` - Redis connection URL
///
/// # Returns
/// * `Ok(Pool)` - Redis connection pool
/// * `Err(AppError)` - Connection error
pub fn create_redis_pool(redis_url: &str) -> Result<Pool, AppError> {
    let cfg = RedisConfig::from_url(redis_url);
    cfg.create_pool(Some(Runtime::Tokio1))
        .map_err(|e| AppError::Internal(format!("Failed to create Redis pool: {}", e)))
}

/// Generates a cache key for a search query.
///
/// Uses SHA-256 hash to create a consistent, collision-resistant key.
///
/// # Arguments
/// * `search_query` - Original search query
///
/// # Returns
/// * Cache key string with prefix
fn generate_cache_key(search_query: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(search_query.as_bytes());
    let hash = format!("{:x}", hasher.finalize());
    format!("price_check:{}", hash)
}

/// Retrieves cached price comparison results.
///
/// # Arguments
/// * `pool` - Redis connection pool
/// * `search_query` - Search query to look up
///
/// # Returns
/// * `Ok(Some(PriceComparisonResult))` - Cached result found
/// * `Ok(None)` - Cache miss
/// * `Err(AppError)` - Redis error
pub async fn get_cached_result(
    pool: &Pool,
    search_query: &str,
) -> Result<Option<PriceComparisonResult>, AppError> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to get Redis connection: {}", e)))?;

    let cache_key = generate_cache_key(search_query);

    let cached_data: Option<String> = conn
        .get(&cache_key)
        .await
        .map_err(|e| AppError::Internal(format!("Redis GET failed: {}", e)))?;

    match cached_data {
        Some(json) => {
            tracing::debug!(
                search_query = %search_query,
                cache_key = %cache_key,
                "Cache hit"
            );
            let result: PriceComparisonResult = serde_json::from_str(&json)
                .map_err(|e| AppError::Parse(format!("Failed to deserialize cache: {}", e)))?;
            Ok(Some(result))
        }
        None => {
            tracing::debug!(
                search_query = %search_query,
                cache_key = %cache_key,
                "Cache miss"
            );
            Ok(None)
        }
    }
}

/// Stores price comparison results in cache with TTL.
///
/// # Arguments
/// * `pool` - Redis connection pool
/// * `search_query` - Search query used as key
/// * `result` - Price comparison result to cache
/// * `ttl_seconds` - Time-to-live in seconds
///
/// # Returns
/// * `Ok(())` - Successfully cached
/// * `Err(AppError)` - Redis error
pub async fn set_cached_result(
    pool: &Pool,
    search_query: &str,
    result: &PriceComparisonResult,
    ttl_seconds: u64,
) -> Result<(), AppError> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to get Redis connection: {}", e)))?;

    let cache_key = generate_cache_key(search_query);
    let json = serde_json::to_string(result)
        .map_err(|e| AppError::Internal(format!("Failed to serialize result: {}", e)))?;

    let _: () = conn
        .set_ex(&cache_key, json, ttl_seconds)
        .await
        .map_err(|e| AppError::Internal(format!("Redis SET failed: {}", e)))?;

    tracing::debug!(
        search_query = %search_query,
        cache_key = %cache_key,
        ttl_seconds = ttl_seconds,
        "Result cached"
    );

    Ok(())
}

/// Invalidates cached result for a search query.
///
/// # Arguments
/// * `pool` - Redis connection pool
/// * `search_query` - Search query to invalidate
///
/// # Returns
/// * `Ok(())` - Successfully invalidated
/// * `Err(AppError)` - Redis error
pub async fn invalidate_cache(pool: &Pool, search_query: &str) -> Result<(), AppError> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to get Redis connection: {}", e)))?;

    let cache_key = generate_cache_key(search_query);

    let _: () = conn
        .del(&cache_key)
        .await
        .map_err(|e| AppError::Internal(format!("Redis DEL failed: {}", e)))?;

    tracing::debug!(
        search_query = %search_query,
        cache_key = %cache_key,
        "Cache invalidated"
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_cache_key_consistency() {
        let key1 = generate_cache_key("laptop");
        let key2 = generate_cache_key("laptop");
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_generate_cache_key_uniqueness() {
        let key1 = generate_cache_key("laptop");
        let key2 = generate_cache_key("desktop");
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_cache_key_format() {
        let key = generate_cache_key("test");
        assert!(key.starts_with("price_check:"));
    }
}
