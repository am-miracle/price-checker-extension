//! Axum middleware for automatic metrics collection.

use axum::{extract::Request, middleware::Next, response::Response};
use std::time::Instant;

use super::metrics;

/// Middleware that tracks HTTP request metrics.
///
/// Records:
/// - Request count by method, path, and status
/// - Request duration histogram
/// - Error count for 4xx and 5xx responses
pub async fn track_metrics(req: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = req.method().to_string();
    let path = req.uri().path().to_string();

    // Process the request
    let response = next.run(req).await;

    let duration = start.elapsed();
    let status = response.status().as_u16();

    // Record metrics
    metrics::record_http_request(&method, &path, status, duration);

    response
}

/// Extracts a clean path for metrics (removes IDs and query params).
///
/// Example: "/api/compare?item=laptop" -> "/api/compare"
pub fn normalize_path(path: &str) -> String {
    // Remove query parameters
    let path = path.split('?').next().unwrap_or(path);

    // Remove trailing slashes
    path.trim_end_matches('/').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_path() {
        assert_eq!(normalize_path("/api/compare?item=test"), "/api/compare");
        assert_eq!(normalize_path("/api/health/"), "/api/health");
        assert_eq!(normalize_path("/metrics"), "/metrics");
    }
}
