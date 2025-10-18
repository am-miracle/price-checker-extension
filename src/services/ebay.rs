//! eBay product search using ZenRows API.
//!
//! Uses ZenRows for eBay product scraping with search-based matching.

use super::zenrows::{ProductSelectors, ZenRowsConfig, search_product};
use crate::{AppError, ProductIdentifiers, SitePrice, config::EbayConfig};
use reqwest::Client;

/// Fetches price information for a product from eBay.
///
/// Uses ZenRows to search eBay and extract the first matching product.
///
/// # Arguments
/// * `identifiers` - Product identifiers (used for building search query)
/// * `search_query` - Search query for the product
/// * `client` - HTTP client
/// * `config` - eBay configuration
/// * `zenrows_config` - ZenRows API configuration
///
/// # Returns
/// * `Ok(SitePrice)` - Product details with price
/// * `Err(AppError)` - Network, parsing, or missing field errors
pub async fn fetch_price(
    identifiers: &ProductIdentifiers,
    search_query: &str,
    client: &Client,
    config: &EbayConfig,
    zenrows_config: Option<&ZenRowsConfig>,
) -> Result<SitePrice, AppError> {
    tracing::info!(site = "eBay", query = %search_query, "Fetching price from eBay");

    if !config.enabled {
        return Err(AppError::Internal(
            "eBay integration not enabled. Set EBAY_ENABLED=true".to_string(),
        ));
    }

    let zenrows = zenrows_config.ok_or_else(|| {
        AppError::Internal(
            "ZenRows API key required for eBay scraping. Set ZENROWS_API_KEY".to_string(),
        )
    })?;

    // Build enhanced search query with identifiers
    let mut query_parts = vec![search_query.to_string()];

    if let Some(brand) = &identifiers.brand {
        query_parts.push(brand.clone());
    }

    if let Some(model) = &identifiers.model_number {
        query_parts.push(model.clone());
    }

    let enhanced_query = query_parts.join(" ");
    let search_url = format!(
        "https://www.ebay.com/sch/i.html?_nkw={}",
        urlencoding::encode(&enhanced_query)
    );

    let selectors = ProductSelectors {
        container: "li.s-item".to_string(),
        title: ".s-item__title".to_string(),
        price: ".s-item__price".to_string(),
        link: ".s-item__link".to_string(),
        image: ".s-item__image-img".to_string(),
    };

    let mut result = search_product(client, zenrows, &search_url, &selectors).await?;
    result.site = "eBay".to_string();

    Ok(result)
}
