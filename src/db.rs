//! Database connection and repository functions.
//!
//! Manages PostgreSQL connection pool and provides data access methods
//! for price history, metrics, and scraper status tracking.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::{PgPool, postgres::PgPoolOptions};
use uuid::Uuid;

use crate::{AppError, SitePrice};

/// Creates a PostgreSQL connection pool.
///
/// # Arguments
/// * `database_url` - PostgreSQL connection string
///
/// # Returns
/// * `Ok(PgPool)` - Connected database pool
/// * `Err(AppError)` - Connection failure
pub async fn create_pool(database_url: &str) -> Result<PgPool, AppError> {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .map_err(|e| AppError::Internal(format!("Database connection failed: {}", e)))
}

/// Saves a price record to the price_history table.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `price` - Price data to save
/// * `search_query` - Original search query
///
/// # Returns
/// * `Ok(Uuid)` - ID of inserted record
/// * `Err(AppError)` - Database error
pub async fn save_price_history(
    pool: &PgPool,
    price: &SitePrice,
    search_query: &str,
) -> Result<Uuid, AppError> {
    let record: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO price_history (
            site, product_title, price_original, currency, price_usd,
            product_link, image_url, search_query
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id
        "#,
    )
    .bind(&price.site)
    .bind(&price.title)
    .bind(&price.price)
    .bind(&price.currency)
    .bind(&price.price_usd)
    .bind(&price.link)
    .bind(&price.image)
    .bind(search_query)
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::Internal(format!("Failed to save price history: {}", e)))?;

    Ok(record.0)
}

/// Records API metrics for performance monitoring.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `endpoint` - API endpoint path
/// * `search_query` - Search query (optional)
/// * `response_time_ms` - Response time in milliseconds
/// * `status_code` - HTTP status code
/// * `results_count` - Number of results returned
/// * `error_message` - Error message if failed (optional)
pub async fn record_api_metrics(
    pool: &PgPool,
    endpoint: &str,
    search_query: Option<&str>,
    response_time_ms: i32,
    status_code: i32,
    results_count: i32,
    error_message: Option<&str>,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        INSERT INTO api_metrics (endpoint, search_query, response_time_ms, status_code, results_count, error_message)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(endpoint)
    .bind(search_query)
    .bind(response_time_ms)
    .bind(status_code)
    .bind(results_count)
    .bind(error_message)
    .execute(pool)
    .await
    .map_err(|e| AppError::Internal(format!("Failed to record metrics: {}", e)))?;

    Ok(())
}

/// Records scraper status for health monitoring.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `site` - Site name
/// * `status` - Scraper status (success/failure/timeout/blocked)
/// * `search_query` - Search query (optional)
/// * `error_type` - Error category (optional)
/// * `error_message` - Error details (optional)
/// * `response_time_ms` - Response time (optional)
pub async fn record_scraper_status(
    pool: &PgPool,
    site: &str,
    status: &str,
    search_query: Option<&str>,
    error_type: Option<&str>,
    error_message: Option<&str>,
    response_time_ms: Option<i32>,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        INSERT INTO scraper_status (site, status, search_query, error_type, error_message, response_time_ms)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(site)
    .bind(status)
    .bind(search_query)
    .bind(error_type)
    .bind(error_message)
    .bind(response_time_ms)
    .execute(pool)
    .await
    .map_err(|e| AppError::Internal(format!("Failed to record scraper status: {}", e)))?;

    Ok(())
}

/// Retrieves recent price history for a search query.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `search_query` - Search query to filter by
/// * `limit` - Maximum number of results
///
/// # Returns
/// * `Ok(Vec<PriceHistoryRecord>)` - List of historical prices
/// * `Err(AppError)` - Database error
pub async fn get_price_history(
    pool: &PgPool,
    search_query: &str,
    limit: i64,
) -> Result<Vec<PriceHistoryRecord>, AppError> {
    let records = sqlx::query_as::<_, PriceHistoryRecord>(
        r#"
        SELECT id, site, product_title, price_original, currency, price_usd,
               product_link, image_url, search_query, scraped_at
        FROM price_history
        WHERE search_query = $1
        ORDER BY scraped_at DESC
        LIMIT $2
        "#,
    )
    .bind(search_query)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::Internal(format!("Failed to fetch price history: {}", e)))?;

    Ok(records)
}

/// Price history record from database.
#[derive(Debug, sqlx::FromRow)]
pub struct PriceHistoryRecord {
    pub id: Uuid,
    pub site: String,
    pub product_title: String,
    pub price_original: Decimal,
    pub currency: String,
    pub price_usd: Decimal,
    pub product_link: String,
    pub image_url: Option<String>,
    pub search_query: String,
    pub scraped_at: DateTime<Utc>,
}
