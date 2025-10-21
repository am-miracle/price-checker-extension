//! Price Checker Extension - Main entry point.
//!
//! Sets up the Axum HTTP server with tracing, graceful shutdown,
//! and all API routes.

use axum::middleware;
use price_checker_extension::{cache, db, observability, routes, utils, AppState, Config};
use std::{net::SocketAddr, sync::Arc};
use tokio::signal;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Initialize observability (tracing and metrics)
    observability::init_tracing()?;
    observability::init_metrics();
    let metrics_handle = routes::setup_metrics_recorder();

    tracing::info!("Starting Price Checker Extension");

    // Load application configuration
    let config = Config::from_env().map_err(|e| format!("Configuration error: {}", e))?;

    tracing::info!(
        host = %config.server.host,
        port = config.server.port,
        "Server configuration loaded"
    );

    // Initialize database connection pool
    tracing::info!("Connecting to database");
    let db_pool = db::create_pool(&config.database.url).await?;

    // Run database migrations
    tracing::info!("Running database migrations");
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .map_err(|e| format!("Migration failed: {}", e))?;

    // Initialize Redis connection pool
    tracing::info!("Connecting to Redis");
    let redis_pool = cache::create_redis_pool(&config.redis.url)?;

    // Create HTTP client for scrapers
    let http_client = utils::create_http_client(
        &config.scraper.user_agent,
        config.scraper.request_timeout_seconds,
    )?;

    // Create shared application state
    let state = Arc::new(AppState::new(
        db_pool,
        redis_pool,
        http_client,
        config.clone(),
    ));

    tracing::info!("Application state initialized");

    // Create application router with middleware
    let app = routes::create_router(state, metrics_handle)
        .layer(middleware::from_fn(observability::track_metrics))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    // Parse socket address
    let addr = SocketAddr::from((
        config.server.host.parse::<std::net::IpAddr>()?,
        config.server.port,
    ));

    tracing::info!(addr = %addr, "Starting HTTP server");

    // Start the server with graceful shutdown
    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("Server shutdown complete");

    // Shutdown tracing to flush remaining spans
    observability::tracing_setup::shutdown_tracing();

    Ok(())
}

/// Handles graceful shutdown on SIGTERM or SIGINT.
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C signal");
        },
        _ = terminate => {
            tracing::info!("Received SIGTERM signal");
        },
    }

    tracing::info!("Initiating graceful shutdown");
}
