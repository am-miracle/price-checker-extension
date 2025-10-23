//! Structured logging and tracing setup.
//!
//! Provides JSON-formatted structured logging for production and
//! pretty-printed logs for development.

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

/// Initializes tracing with structured logging.
///
/// Sets up:
/// - JSON-formatted structured logging for production (if LOG_FORMAT=json)
/// - Pretty-printed console logging for development (default)
/// - Configurable log levels via RUST_LOG environment variable
///
/// # Environment Variables
/// - `RUST_LOG`: Log level filter (default: "info,price_checker_extension=debug")
/// - `LOG_FORMAT`: Output format ("json" or "pretty", default: "pretty")
/// - `SERVICE_NAME`: Service name for logs (default: "price-checker-extension")
/// - `ENVIRONMENT`: Deployment environment (dev/staging/prod)
pub fn init_tracing() -> Result<(), Box<dyn std::error::Error>> {
    let service_name =
        std::env::var("SERVICE_NAME").unwrap_or_else(|_| "price-checker-extension".to_string());

    let environment = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());

    let log_format = std::env::var("LOG_FORMAT").unwrap_or_else(|_| "pretty".to_string());

    // Base filter from environment
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,price_checker_extension=debug"));

    if log_format == "json" {
        // Production setup with JSON formatting
        tracing::info!("Initializing JSON structured logging");

        let json_layer = tracing_subscriber::fmt::layer()
            .json()
            .with_current_span(true)
            .with_span_list(true)
            .with_target(true)
            .with_filter(env_filter);

        tracing_subscriber::registry().with(json_layer).init();

        tracing::info!(
            service = %service_name,
            version = env!("CARGO_PKG_VERSION"),
            environment = %environment,
            "Structured logging initialized"
        );
    } else {
        // Development setup with pretty printing
        tracing::info!("Initializing pretty-printed logging");

        let fmt_layer = tracing_subscriber::fmt::layer()
            .pretty()
            .with_target(true)
            .with_thread_ids(true)
            .with_line_number(true)
            .with_filter(env_filter);

        tracing_subscriber::registry().with(fmt_layer).init();

        tracing::info!(
            service = %service_name,
            version = env!("CARGO_PKG_VERSION"),
            environment = %environment,
            "Pretty logging initialized (development mode)"
        );
    }

    Ok(())
}

/// Placeholder for graceful shutdown (no-op without OpenTelemetry).
pub fn shutdown_tracing() {
    tracing::info!("Tracing shutdown complete");
}
