//! Amazon product search using ZenRows API.
//!
//! Uses ZenRows E-Commerce API for Amazon product scraping.
//! Supports ASIN-based lookups and search queries.

use super::zenrows::{fetch_amazon_product, search_product, ProductSelectors, ZenRowsConfig};
use crate::{config::AmazonConfig, AppError, ProductIdentifiers, SitePrice};
use reqwest::Client;

/// Fetches price information for a product from Amazon.
///
///
/// # Arguments
/// * `identifiers` - Product identifiers (ASIN preferred)
/// * `search_query` - Fallback search query if no ASIN
/// * `client` - HTTP client
/// * `config` - Amazon API configuration
/// * `zenrows_config` - ZenRows API configuration
///
/// # Returns
/// * `Ok(SitePrice)` - Product details with price
/// * `Err(AppError)` - Network, parsing, or missing field errors
pub async fn fetch_price(
    identifiers: &ProductIdentifiers,
    search_query: &str,
    client: &Client,
    config: &AmazonConfig,
    zenrows_config: Option<&ZenRowsConfig>,
) -> Result<SitePrice, AppError> {
    tracing::info!(site = "Amazon", query = %search_query, "Fetching price from Amazon");

    if !config.enabled {
        return Err(AppError::Internal(
            "Amazon integration not enabled. Set AMAZON_ENABLED=true".to_string(),
        ));
    }

    let zenrows = zenrows_config.ok_or_else(|| {
        AppError::Internal(
            "ZenRows API key required for Amazon scraping. Set ZENROWS_API_KEY".to_string(),
        )
    })?;

    if let Some(asin) = &identifiers.asin {
        tracing::info!(asin = %asin, "Using ASIN for Amazon lookup");
        match fetch_amazon_product(client, zenrows, asin).await {
            Ok(price) => return Ok(price),
            Err(e) => {
                tracing::warn!(asin = %asin, error = %e, "ASIN lookup failed, falling back to search");
            }
        }
    }

    // Priority 2: Search Amazon
    let search_url = format!(
        "https://www.amazon.com/s?k={}",
        urlencoding::encode(search_query)
    );

    let selectors = ProductSelectors {
        container: "div[data-component-type='s-search-result']".to_string(),
        title: "h2 a span".to_string(),
        price: "span.a-price span.a-offscreen".to_string(),
        link: "h2 a".to_string(),
        image: "img.s-image".to_string(),
    };

    let mut result = search_product(client, zenrows, &search_url, &selectors).await?;
    result.site = "Amazon".to_string();

    Ok(result)
}
