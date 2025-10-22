//! API route handlers.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::get,
    Router,
};
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{cache, services, AppError, AppState, PriceComparisonResult, ProductMatchRequest};

/// Query parameters for price comparison endpoint.
#[derive(Debug, Deserialize)]
pub struct CompareQuery {
    /// Product search query
    pub item: String,
}

/// Health check response.
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

/// Readiness check response.
#[derive(Serialize)]
pub struct ReadyResponse {
    pub ready: bool,
}

/// Initializes Prometheus metrics exporter.
pub fn setup_metrics_recorder() -> PrometheusHandle {
    PrometheusBuilder::new()
        .set_buckets_for_metric(
            Matcher::Full("http_request_duration_seconds".to_string()),
            &[0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0],
        )
        .unwrap()
        .set_buckets_for_metric(
            Matcher::Full("price_comparison_duration_seconds".to_string()),
            &[0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0],
        )
        .unwrap()
        .install_recorder()
        .expect("Failed to install Prometheus recorder")
}

/// Creates the main application router with all routes.
pub fn create_router(state: Arc<AppState>, metrics_handle: PrometheusHandle) -> Router {
    Router::new()
        .route("/api/health", get(health_handler))
        .route("/api/ready", get(ready_handler))
        .route(
            "/metrics",
            get(move || metrics_handler(metrics_handle.clone())),
        )
        .route(
            "/api/compare",
            get(compare_handler).post(compare_post_handler),
        )
        .route("/api/currencies", get(currencies_handler))
        .with_state(state)
}

/// Prometheus metrics endpoint.
///
/// Returns metrics in Prometheus text format for scraping.
async fn metrics_handler(handle: PrometheusHandle) -> Response {
    handle.render().into_response()
}

/// Health check endpoint.
///
/// Returns 200 OK if the service is running.
async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Readiness check endpoint.
///
/// Returns 200 OK if the service is ready to handle requests.
async fn ready_handler(State(state): State<Arc<AppState>>) -> (StatusCode, Json<ReadyResponse>) {
    // Check database connectivity
    let db_ready = sqlx::query("SELECT 1")
        .fetch_one(&state.db_pool)
        .await
        .is_ok();

    // Check Redis connectivity
    let redis_ready = state.redis_pool.get().await.is_ok();

    let ready = db_ready && redis_ready;
    let status = if ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (status, Json(ReadyResponse { ready }))
}

/// Price comparison endpoint.
///
/// Fetches prices from all supported platforms and returns comparison results.
/// Uses Redis cache with TTL to improve performance.
///
/// # Query Parameters
/// * `item` - Product search query (required)
///
/// # Returns
/// * `200 OK` - Comparison results
/// * `400 Bad Request` - Missing or invalid query parameter
/// * `500 Internal Server Error` - All scrapers failed
async fn compare_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<CompareQuery>,
) -> Result<Json<PriceComparisonResult>, AppError> {
    tracing::info!(item = %params.item, "Received price comparison request");

    // Check cache first
    if let Ok(Some(cached_result)) = cache::get_cached_result(&state.redis_pool, &params.item).await
    {
        tracing::info!(item = %params.item, "Returning cached result");
        return Ok(Json(cached_result));
    }

    // Cache miss - fetch fresh data
    let result = services::compare_all(&params.item, &state).await?;

    // Cache the result
    if let Err(e) = cache::set_cached_result(
        &state.redis_pool,
        &params.item,
        &result,
        state.config.cache.ttl_seconds,
    )
    .await
    {
        tracing::warn!(
            item = %params.item,
            error = %e,
            "Failed to cache result"
        );
    }

    tracing::info!(
        item = %params.item,
        results_count = result.all_prices.len(),
        "Price comparison completed"
    );

    Ok(Json(result))
}

/// Currency information response.
#[derive(Serialize)]
pub struct CurrencyInfo {
    pub code: String,
    pub symbol: String,
    pub name: String,
}

/// Currencies list response.
#[derive(Serialize)]
pub struct CurrenciesResponse {
    pub currencies: Vec<CurrencyInfo>,
}

/// Currencies endpoint - returns list of supported currencies.
///
/// GET /api/currencies
async fn currencies_handler() -> Json<CurrenciesResponse> {
    use crate::services::currency::Currency;

    let currencies = vec![
        CurrencyInfo {
            code: Currency::USD.code().to_string(),
            symbol: Currency::USD.symbol().to_string(),
            name: "US Dollar".to_string(),
        },
        CurrencyInfo {
            code: Currency::EUR.code().to_string(),
            symbol: Currency::EUR.symbol().to_string(),
            name: "Euro".to_string(),
        },
        CurrencyInfo {
            code: Currency::GBP.code().to_string(),
            symbol: Currency::GBP.symbol().to_string(),
            name: "British Pound".to_string(),
        },
        CurrencyInfo {
            code: Currency::NGN.code().to_string(),
            symbol: Currency::NGN.symbol().to_string(),
            name: "Nigerian Naira".to_string(),
        },
        CurrencyInfo {
            code: Currency::INR.code().to_string(),
            symbol: Currency::INR.symbol().to_string(),
            name: "Indian Rupee".to_string(),
        },
        CurrencyInfo {
            code: Currency::CAD.code().to_string(),
            symbol: Currency::CAD.symbol().to_string(),
            name: "Canadian Dollar".to_string(),
        },
        CurrencyInfo {
            code: Currency::AUD.code().to_string(),
            symbol: Currency::AUD.symbol().to_string(),
            name: "Australian Dollar".to_string(),
        },
        CurrencyInfo {
            code: Currency::JPY.code().to_string(),
            symbol: Currency::JPY.symbol().to_string(),
            name: "Japanese Yen".to_string(),
        },
    ];

    Json(CurrenciesResponse { currencies })
}

/// Product comparison endpoint with detailed identifiers (POST).
///
/// Accepts detailed product information including identifiers (UPC, ASIN, model)
/// for more accurate cross-site matching. Uses product matching algorithms
/// to ensure results are for the same product.
///
/// # Request Body
/// * `ProductMatchRequest` - Product details and identifiers (JSON)
///
/// # Returns
/// * `200 OK` - Comparison results with match confidence scores
/// * `400 Bad Request` - Invalid request body
/// * `500 Internal Server Error` - All scrapers failed or no matches found
async fn compare_post_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ProductMatchRequest>,
) -> Result<Json<PriceComparisonResult>, AppError> {
    tracing::info!(
        title = %request.title,
        current_site = ?request.current_site,
        has_upc = request.identifiers.upc.is_some(),
        has_asin = request.identifiers.asin.is_some(),
        "Received product match request"
    );

    // Use title as search query
    let search_query = &request.title;

    // Check cache first (using title as key)
    if let Ok(Some(cached_result)) = cache::get_cached_result(&state.redis_pool, search_query).await
    {
        tracing::info!(title = %request.title, "Returning cached result");
        return Ok(Json(cached_result));
    }

    // Cache miss - fetch fresh data with identifiers
    let target_currency = request.target_currency.as_deref();
    let result = services::compare_with_identifiers(
        &request.identifiers,
        search_query,
        &state,
        target_currency,
    )
    .await?;

    // Cache the result
    if let Err(e) = cache::set_cached_result(
        &state.redis_pool,
        search_query,
        &result,
        state.config.cache.ttl_seconds,
    )
    .await
    {
        tracing::warn!(
            title = %request.title,
            error = %e,
            "Failed to cache result"
        );
    }

    tracing::info!(
        title = %request.title,
        results_count = result.all_prices.len(),
        best_price = ?result.best_deal.as_ref().map(|p| p.price),
        "Product comparison completed"
    );

    Ok(Json(result))
}
