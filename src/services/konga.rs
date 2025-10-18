//! Konga product search using ZenRows API.
//!
//! Uses ZenRows for Konga Nigeria product scraping.

use super::zenrows::{ProductSelectors, ZenRowsConfig, search_product};
use crate::{AppError, ProductIdentifiers, SitePrice, config::KongaConfig};
use reqwest::Client;

/// Fetches price information for a product from Konga.
///
/// Uses ZenRows to search Konga Nigeria and extract product information.
///
/// # Arguments
/// * `identifiers` - Product identifiers
/// * `search_query` - Search query for the product
/// * `client` - HTTP client
/// * `config` - Konga configuration
/// * `zenrows_config` - ZenRows API configuration
///
/// # Returns
/// * `Ok(SitePrice)` - Product details with price
/// * `Err(AppError)` - Network, parsing, or missing field errors
pub async fn fetch_price(
    _identifiers: &ProductIdentifiers,
    search_query: &str,
    client: &Client,
    config: &KongaConfig,
    zenrows_config: Option<&ZenRowsConfig>,
) -> Result<SitePrice, AppError> {
    tracing::info!(site = "Konga", query = %search_query, "Fetching price from Konga");

    if !config.enabled {
        return Err(AppError::Internal(
            "Konga integration not enabled. Set KONGA_ENABLED=true".to_string(),
        ));
    }

    let zenrows = zenrows_config.ok_or_else(|| {
        AppError::Internal(
            "ZenRows API key required for Konga scraping. Set ZENROWS_API_KEY".to_string(),
        )
    })?;

    let search_url = format!(
        "https://www.konga.com/search?search={}",
        urlencoding::encode(search_query)
    );

    let selectors = ProductSelectors {
        container: "div._0a8d6_3FrP8".to_string(),
        title: "div._0a8d6_2v3u7".to_string(),
        price: "span._0a8d6_1nrBS".to_string(),
        link: "a._0a8d6_3pJo1".to_string(),
        image: "img._0a8d6_8jVS9".to_string(),
    };

    let mut result = search_product(client, zenrows, &search_url, &selectors).await?;
    result.site = "Konga".to_string();

    Ok(result)
}
