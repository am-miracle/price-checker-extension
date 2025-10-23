//! Prometheus metrics for monitoring application performance.
//!
//! Tracks key business and technical metrics including:
//! - Request rates and latencies
//! - Price comparison success/failure rates
//! - Database and cache performance
//! - Currency conversion metrics

use metrics::{counter, describe_counter, describe_gauge, describe_histogram, gauge, histogram};
use std::time::Instant;

/// Initializes all metric descriptions for Prometheus.
///
/// Call this once at application startup to register metric metadata.
pub fn init_metrics() {
    // HTTP Request Metrics
    describe_counter!(
        "http_requests_total",
        "Total number of HTTP requests received"
    );
    describe_histogram!("http_request_duration_seconds", "HTTP request duration");
    describe_counter!(
        "http_requests_errors_total",
        "Total number of HTTP request errors"
    );

    // Price Comparison Metrics
    describe_counter!(
        "price_comparisons_total",
        "Total number of price comparison requests"
    );
    describe_counter!(
        "price_comparisons_cache_hits",
        "Number of cache hits for price comparisons"
    );
    describe_counter!(
        "price_comparisons_cache_misses",
        "Number of cache misses for price comparisons"
    );
    describe_histogram!(
        "price_comparison_duration_seconds",
        "Time taken to complete price comparisons"
    );

    // Scraper Metrics
    describe_counter!("scraper_requests_total", "Total scraper requests by site");
    describe_counter!(
        "scraper_requests_success",
        "Successful scraper requests by site"
    );
    describe_counter!(
        "scraper_requests_failures",
        "Failed scraper requests by site"
    );
    describe_histogram!(
        "scraper_duration_seconds",
        "Time taken for scraper requests"
    );

    // Database Metrics
    describe_gauge!(
        "database_connections_active",
        "Number of active database connections"
    );
    describe_counter!("database_queries_total", "Total number of database queries");
    describe_histogram!("database_query_duration_seconds", "Database query duration");

    // Cache Metrics
    describe_gauge!(
        "cache_connections_active",
        "Number of active Redis connections"
    );
    describe_counter!("cache_operations_total", "Total cache operations");
    describe_histogram!(
        "cache_operation_duration_seconds",
        "Cache operation duration"
    );

    // Currency Conversion Metrics
    describe_counter!(
        "currency_conversions_total",
        "Total number of currency conversions"
    );
    describe_counter!(
        "currency_api_calls_total",
        "Total calls to exchange rate API"
    );
    describe_counter!("currency_api_cache_hits", "Exchange rate cache hits");

    // Business Metrics
    describe_gauge!("active_searches", "Number of active price searches");
    describe_histogram!(
        "prices_found_per_search",
        "Number of prices found per search"
    );
    describe_histogram!("price_savings_usd", "Savings found in USD");
}

/// Records an HTTP request.
pub fn record_http_request(method: &str, path: &str, status: u16, duration: std::time::Duration) {
    counter!("http_requests_total", "method" => method.to_string(), "path" => path.to_string(), "status" => status.to_string()).increment(1);
    histogram!("http_request_duration_seconds", "method" => method.to_string(), "path" => path.to_string())
        .record(duration.as_secs_f64());

    if status >= 400 {
        counter!("http_requests_errors_total", "method" => method.to_string(), "path" => path.to_string(), "status" => status.to_string()).increment(1);
    }
}

/// Records a price comparison request.
pub fn record_price_comparison(cached: bool, results_count: usize, duration: std::time::Duration) {
    counter!("price_comparisons_total").increment(1);

    if cached {
        counter!("price_comparisons_cache_hits").increment(1);
    } else {
        counter!("price_comparisons_cache_misses").increment(1);
        histogram!("price_comparison_duration_seconds").record(duration.as_secs_f64());
    }

    histogram!("prices_found_per_search").record(results_count as f64);
}

/// Records a scraper request.
pub fn record_scraper_request(site: &str, success: bool, duration: std::time::Duration) {
    counter!("scraper_requests_total", "site" => site.to_string()).increment(1);

    if success {
        counter!("scraper_requests_success", "site" => site.to_string()).increment(1);
    } else {
        counter!("scraper_requests_failures", "site" => site.to_string()).increment(1);
    }

    histogram!("scraper_duration_seconds", "site" => site.to_string())
        .record(duration.as_secs_f64());
}

/// Records a database query.
pub fn record_database_query(query_type: &str, duration: std::time::Duration) {
    counter!("database_queries_total", "type" => query_type.to_string()).increment(1);
    histogram!("database_query_duration_seconds", "type" => query_type.to_string())
        .record(duration.as_secs_f64());
}

/// Updates database connection pool metrics.
pub fn update_database_connections(active: usize) {
    gauge!("database_connections_active").set(active as f64);
}

/// Records a cache operation.
pub fn record_cache_operation(operation: &str, duration: std::time::Duration) {
    counter!("cache_operations_total", "operation" => operation.to_string()).increment(1);
    histogram!("cache_operation_duration_seconds", "operation" => operation.to_string())
        .record(duration.as_secs_f64());
}

/// Updates Redis connection pool metrics.
pub fn update_cache_connections(active: usize) {
    gauge!("cache_connections_active").set(active as f64);
}

/// Records a currency conversion.
pub fn record_currency_conversion(from: &str, to: &str) {
    counter!("currency_conversions_total", "from" => from.to_string(), "to" => to.to_string())
        .increment(1);
}

/// Records an exchange rate API call.
pub fn record_currency_api_call(cached: bool) {
    counter!("currency_api_calls_total").increment(1);

    if cached {
        counter!("currency_api_cache_hits").increment(1);
    }
}

/// Records price savings found.
pub fn record_price_savings(savings_usd: f64) {
    histogram!("price_savings_usd").record(savings_usd);
}

/// Helper to track active searches.
pub struct SearchTracker {
    _start: Instant,
}

impl SearchTracker {
    pub fn new() -> Self {
        gauge!("active_searches").increment(1.0);
        Self {
            _start: Instant::now(),
        }
    }
}

impl Drop for SearchTracker {
    fn drop(&mut self) {
        gauge!("active_searches").decrement(1.0);
    }
}
