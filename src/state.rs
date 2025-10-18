//! Application shared state.
//!
//! Contains database connection pool, Redis pool, HTTP client,
//! and configuration shared across request handlers.

use deadpool_redis::Pool as RedisPool;
use reqwest::Client;
use sqlx::PgPool;
use std::sync::Arc;

use crate::Config;
use crate::services::currency::CurrencyService;

/// Shared application state passed to all route handlers.
#[derive(Clone)]
pub struct AppState {
    /// PostgreSQL connection pool
    pub db_pool: PgPool,

    /// Redis connection pool for caching
    pub redis_pool: RedisPool,

    /// HTTP client for scraping
    pub http_client: Client,

    /// Application configuration
    pub config: Config,

    /// Currency service for exchange rates and conversions
    pub currency_service: Arc<CurrencyService>,
}

impl AppState {
    /// Creates a new AppState instance.
    ///
    /// # Arguments
    /// * `db_pool` - Database connection pool
    /// * `redis_pool` - Redis connection pool
    /// * `http_client` - HTTP client
    /// * `config` - Application configuration
    pub fn new(
        db_pool: PgPool,
        redis_pool: RedisPool,
        http_client: Client,
        config: Config,
    ) -> Self {
        // Initialize currency service
        let currency_service = Arc::new(CurrencyService::new(
            redis_pool.clone(),
            http_client.clone(),
            config.currency.api_url.clone(),
            config.currency.cache_ttl_hours,
        ));

        Self {
            db_pool,
            redis_pool,
            http_client,
            config,
            currency_service,
        }
    }
}
