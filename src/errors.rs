//! Error types and handling for the price checker API.
//!
//! Defines a unified error type using `thiserror` with variants for common
//! failure scenarios. Includes Axum integration for HTTP responses.

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

/// Application error type with variants for different failure scenarios.
#[derive(Debug, Error)]
pub enum AppError {
    /// Network-related errors (timeouts, connection failures, etc.)
    #[error("Network error: {0}")]
    Network(String),

    /// HTML/JSON parsing errors
    #[error("Parse error: {0}")]
    Parse(String),

    /// Missing required field in scraped data
    #[error("Missing field: {0}")]
    MissingField(String),

    /// Cache-related errors (Redis connection, serialization, etc.)
    #[error("Cache error: {0}")]
    Cache(String),

    /// Internal server errors (unexpected failures)
    #[error("Internal error: {0}")]
    Internal(String),
}

impl AppError {
    /// Maps error variants to appropriate HTTP status codes.
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::Network(_) => StatusCode::BAD_GATEWAY,
            AppError::Parse(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::MissingField(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::Cache(_) => StatusCode::SERVICE_UNAVAILABLE,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// JSON error response structure
#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_message = self.to_string();

        let body = Json(ErrorResponse {
            error: error_message,
        });

        (status, body).into_response()
    }
}
