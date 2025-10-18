//! Utility functions for HTTP requests, retry logic, rate limiting, and parsing.

use reqwest::{Client, header};
use std::time::Duration;
use tokio::time::sleep;

use crate::AppError;

/// Creates a configured HTTP client for web scraping.
///
/// # Arguments
/// * `user_agent` - User-Agent string to identify the bot
/// * `timeout_seconds` - Request timeout in seconds
///
/// # Returns
/// * `Ok(Client)` - Configured HTTP client
/// * `Err(AppError)` - Client build error
pub fn create_http_client(user_agent: &str, timeout_seconds: u64) -> Result<Client, AppError> {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::USER_AGENT,
        header::HeaderValue::from_str(user_agent)
            .map_err(|e| AppError::Internal(format!("Invalid user agent: {}", e)))?,
    );
    headers.insert(
        header::ACCEPT,
        header::HeaderValue::from_static("text/html,application/json,application/xhtml+xml"),
    );
    headers.insert(
        header::ACCEPT_LANGUAGE,
        header::HeaderValue::from_static("en-US,en;q=0.9"),
    );

    Client::builder()
        .timeout(Duration::from_secs(timeout_seconds))
        .default_headers(headers)
        .build()
        .map_err(|e| AppError::Internal(format!("Failed to build HTTP client: {}", e)))
}

/// Performs an HTTP GET request with exponential backoff retry logic.
///
/// # Arguments
/// * `client` - HTTP client
/// * `url` - URL to fetch
/// * `max_retries` - Maximum number of retry attempts
///
/// # Returns
/// * `Ok(String)` - Response body as text
/// * `Err(AppError)` - Network or HTTP error
pub async fn fetch_with_retry(
    client: &Client,
    url: &str,
    max_retries: u32,
) -> Result<String, AppError> {
    let mut attempt = 0;

    loop {
        match client.get(url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    return response
                        .text()
                        .await
                        .map_err(|e| AppError::Network(format!("Failed to read response: {}", e)));
                } else if response.status().as_u16() == 429 {
                    // Rate limited - apply exponential backoff
                    if attempt >= max_retries {
                        return Err(AppError::Network(format!(
                            "Rate limited after {} retries",
                            max_retries
                        )));
                    }
                    let backoff_ms = 1000 * 2_u64.pow(attempt);
                    tracing::warn!(
                        url = %url,
                        attempt = attempt + 1,
                        backoff_ms = backoff_ms,
                        "Rate limited, retrying with exponential backoff"
                    );
                    sleep(Duration::from_millis(backoff_ms)).await;
                } else {
                    return Err(AppError::Network(format!(
                        "HTTP error {}: {}",
                        response.status(),
                        url
                    )));
                }
            }
            Err(e) => {
                if attempt >= max_retries {
                    return Err(AppError::Network(format!(
                        "Request failed after {} retries: {}",
                        max_retries, e
                    )));
                }
                let backoff_ms = 500 * 2_u64.pow(attempt);
                tracing::warn!(
                    url = %url,
                    attempt = attempt + 1,
                    backoff_ms = backoff_ms,
                    error = %e,
                    "Request failed, retrying"
                );
                sleep(Duration::from_millis(backoff_ms)).await;
            }
        }

        attempt += 1;
    }
}

/// Rate limiter to enforce delays between requests.
pub struct RateLimiter {
    delay_ms: u64,
}

impl RateLimiter {
    /// Creates a new rate limiter.
    ///
    /// # Arguments
    /// * `requests_per_second` - Maximum requests per second
    pub fn new(requests_per_second: u32) -> Self {
        let delay_ms = if requests_per_second > 0 {
            1000 / requests_per_second as u64
        } else {
            1000
        };

        Self { delay_ms }
    }

    /// Applies rate limiting by sleeping for the configured delay.
    pub async fn wait(&self) {
        sleep(Duration::from_millis(self.delay_ms)).await;
    }
}

/// DEPRECATED: Use currency::parse_price_with_currency() instead.
/// This function is kept for backward compatibility but will be removed.
///
/// For new code, use:
/// ```ignore
/// use crate::services::currency::parse_price_with_currency;
/// let (amount, currency) = parse_price_with_currency(price_str, site_hint)?;
/// ```
#[deprecated(
    since = "0.1.0",
    note = "Use currency::parse_price_with_currency() for proper currency handling"
)]
pub fn parse_price(price_str: &str) -> Result<f64, AppError> {
    use crate::services::currency::parse_price_with_currency;
    let (amount, _) = parse_price_with_currency(price_str, None)?;
    Ok(amount.to_string().parse::<f64>().unwrap_or(0.0))
}

/// Extracts text content from an HTML element selector.
///
/// # Arguments
/// * `html` - HTML document string
/// * `selector` - CSS selector string
///
/// # Returns
/// * `Ok(String)` - Extracted text content
/// * `Err(AppError)` - Parse error or element not found
pub fn extract_text(html: &str, selector: &str) -> Result<String, AppError> {
    use scraper::{Html, Selector};

    let document = Html::parse_document(html);
    let css_selector = Selector::parse(selector)
        .map_err(|_| AppError::Parse(format!("Invalid CSS selector: {}", selector)))?;

    document
        .select(&css_selector)
        .next()
        .map(|element| element.text().collect::<String>().trim().to_string())
        .ok_or_else(|| AppError::MissingField(format!("Element not found: {}", selector)))
}

/// Extracts an attribute value from an HTML element selector.
///
/// # Arguments
/// * `html` - HTML document string
/// * `selector` - CSS selector string
/// * `attribute` - Attribute name (e.g., "href", "src")
///
/// # Returns
/// * `Ok(String)` - Extracted attribute value
/// * `Err(AppError)` - Parse error or element/attribute not found
pub fn extract_attr(html: &str, selector: &str, attribute: &str) -> Result<String, AppError> {
    use scraper::{Html, Selector};

    let document = Html::parse_document(html);
    let css_selector = Selector::parse(selector)
        .map_err(|_| AppError::Parse(format!("Invalid CSS selector: {}", selector)))?;

    document
        .select(&css_selector)
        .next()
        .and_then(|element| element.value().attr(attribute))
        .map(|s| s.to_string())
        .ok_or_else(|| {
            AppError::MissingField(format!("Attribute '{}' not found: {}", attribute, selector))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_creation() {
        let limiter = RateLimiter::new(2);
        assert_eq!(limiter.delay_ms, 500);
    }

    #[test]
    fn test_extract_text() {
        let html = r#"<div class="price">$19.99</div>"#;
        let result = extract_text(html, ".price").unwrap();
        assert_eq!(result, "$19.99");
    }

    #[test]
    fn test_extract_attr() {
        let html = r#"<a href="https://example.com" class="link">Click</a>"#;
        let result = extract_attr(html, ".link", "href").unwrap();
        assert_eq!(result, "https://example.com");
    }
}
