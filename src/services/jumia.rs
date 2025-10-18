//! Jumia product search using ZenRows API.
//!
//! Uses ZenRows for Jumia Nigeria product scraping.

use super::zenrows::{ProductSelectors, ZenRowsConfig, search_product};
use crate::{AppError, ProductIdentifiers, SitePrice, config::JumiaConfig};
use reqwest::Client;

/// Fetches price information for a product from Jumia.
///
/// Uses ZenRows to search Jumia Nigeria and extract product information.
///
/// # Arguments
/// * `identifiers` - Product identifiers
/// * `search_query` - Search query for the product
/// * `client` - HTTP client
/// * `config` - Jumia configuration
/// * `zenrows_config` - ZenRows API configuration
///
/// # Returns
/// * `Ok(SitePrice)` - Product details with price
/// * `Err(AppError)` - Network, parsing, or missing field errors
pub async fn fetch_price(
    _identifiers: &ProductIdentifiers,
    search_query: &str,
    client: &Client,
    config: &JumiaConfig,
    zenrows_config: Option<&ZenRowsConfig>,
) -> Result<SitePrice, AppError> {
    tracing::info!(site = "Jumia", query = %search_query, "Fetching price from Jumia");

    if !config.enabled {
        return Err(AppError::Internal(
            "Jumia integration not enabled. Set JUMIA_ENABLED=true".to_string(),
        ));
    }

    let zenrows = zenrows_config.ok_or_else(|| {
        AppError::Internal(
            "ZenRows API key required for Jumia scraping. Set ZENROWS_API_KEY".to_string(),
        )
    })?;

    let search_url = format!(
        "https://www.jumia.com.ng/catalog/?q={}",
        urlencoding::encode(search_query)
    );

    let selectors = ProductSelectors {
        container: "article.prd".to_string(),
        title: ".name".to_string(),
        price: ".prc".to_string(),
        link: "a.core".to_string(),
        image: "img.img".to_string(),
    };

    let mut result = search_product(client, zenrows, &search_url, &selectors).await?;
    result.site = "Jumia".to_string();

    Ok(result)
}
