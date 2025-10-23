//! ZenRows API client for web scraping with anti-bot protection.
//!
//! Provides integration with ZenRows API for scraping e-commerce sites
//! with automatic proxy rotation, JavaScript rendering, and CAPTCHA solving.

use crate::services::currency::parse_price_with_currency;
use crate::{AppError, SitePrice};
use reqwest::Client;
use scraper::{Html, Selector};
use serde::Deserialize;

/// ZenRows API configuration.
#[derive(Debug, Clone)]
pub struct ZenRowsConfig {
    pub api_key: String,
    pub api_url: String,
}

impl ZenRowsConfig {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            api_url: "https://api.zenrows.com/v1/".to_string(),
        }
    }
}

/// ZenRows E-Commerce API response for Amazon products.
#[derive(Debug, Deserialize)]
pub struct AmazonProductResponse {
    pub title: Option<String>,
    pub price: Option<String>,
    pub image: Option<String>,
    pub product_url: Option<String>,
    pub asin: Option<String>,
}

/// Scrapes a URL using ZenRows Universal Scraper API.
///
/// # Arguments
/// * `client` - HTTP client
/// * `config` - ZenRows configuration
/// * `url` - Target URL to scrape
/// * `render_js` - Whether to enable JavaScript rendering
///
/// # Returns
/// * `Ok(String)` - HTML content of the page
/// * `Err(AppError)` - Network or API error
pub async fn scrape_url(
    client: &Client,
    config: &ZenRowsConfig,
    url: &str,
    render_js: bool,
) -> Result<String, AppError> {
    let mut params = vec![("apikey", config.api_key.clone()), ("url", url.to_string())];

    if render_js {
        params.push(("js_render", "true".to_string()));
    }

    let response = client
        .get(&config.api_url)
        .query(&params)
        .send()
        .await
        .map_err(|e| AppError::Network(format!("ZenRows API request failed: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        return Err(AppError::Network(format!(
            "ZenRows API error {}: {}",
            status, error_text
        )));
    }

    response
        .text()
        .await
        .map_err(|e| AppError::Network(format!("Failed to read ZenRows response: {}", e)))
}

/// Fetches Amazon product details using ZenRows E-Commerce API.
///
/// Uses the specialized Amazon endpoint for structured data extraction.
///
/// # Arguments
/// * `client` - HTTP client
/// * `config` - ZenRows configuration
/// * `asin` - Amazon Standard Identification Number
///
/// # Returns
/// * `Ok(SitePrice)` - Product details with price
/// * `Err(AppError)` - API or parsing error
pub async fn fetch_amazon_product(
    client: &Client,
    config: &ZenRowsConfig,
    asin: &str,
) -> Result<SitePrice, AppError> {
    let api_endpoint = format!(
        "https://ecommerce.api.zenrows.com/v1/targets/amazon/products/{}",
        asin
    );

    let response = client
        .get(&api_endpoint)
        .query(&[("apikey", &config.api_key)])
        .send()
        .await
        .map_err(|e| AppError::Network(format!("ZenRows Amazon API request failed: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        return Err(AppError::Network(format!(
            "ZenRows Amazon API error: {}",
            status
        )));
    }

    let product: AmazonProductResponse = response
        .json()
        .await
        .map_err(|e| AppError::Parse(format!("Failed to parse Amazon response: {}", e)))?;

    let title = product
        .title
        .ok_or_else(|| AppError::MissingField("Amazon product title".to_string()))?;

    let price_str = product
        .price
        .ok_or_else(|| AppError::MissingField("Amazon product price".to_string()))?;

    let (price, currency) = parse_price_with_currency(&price_str, Some("amazon.com"))?;
    // For now, assume USD for Amazon.com - would need marketplace detection for other regions
    let price_usd = price; // Since we detected USD

    let link = product
        .product_url
        .unwrap_or_else(|| format!("https://www.amazon.com/dp/{}", asin));

    Ok(SitePrice {
        site: "Amazon".to_string(),
        title,
        price,
        currency: currency.code().to_string(),
        price_usd,
        price_converted: None,
        target_currency: None,
        link,
        image: product.image,
        match_confidence: Some(100), // ASIN is exact match
    })
}

/// Searches for a product on a site and returns the first matching result.
///
/// Uses ZenRows to scrape the search results page and extract product info.
///
/// # Arguments
/// * `client` - HTTP client
/// * `config` - ZenRows configuration
/// * `search_url` - Full search URL
/// * `selectors` - CSS selectors for extracting product data
///
/// # Returns
/// * `Ok(SitePrice)` - First matching product
/// * `Err(AppError)` - Scraping or parsing error
pub async fn search_product(
    client: &Client,
    config: &ZenRowsConfig,
    search_url: &str,
    selectors: &ProductSelectors,
) -> Result<SitePrice, AppError> {
    let html = scrape_url(client, config, search_url, true).await?;
    let document = Html::parse_document(&html);

    // Extract base URL for converting relative links to absolute
    let base_url = extract_base_url(search_url)?;

    extract_first_product(&document, selectors, &base_url)
}

/// Extracts the base URL from a full URL (e.g., "https://www.jumia.com.ng/...")
/// Returns "eg: https://www.jumia.com.ng"
fn extract_base_url(url: &str) -> Result<String, AppError> {
    // Find the position after the scheme (http:// or https://)
    let scheme_end = url
        .find("://")
        .ok_or_else(|| AppError::Internal("Invalid URL: no scheme found".to_string()))?;

    let after_scheme = &url[scheme_end + 3..];

    // Find the end of the host (first '/' or '?' after scheme)
    let host_end = after_scheme
        .find('/')
        .or_else(|| after_scheme.find('?'))
        .unwrap_or(after_scheme.len());

    Ok(url[..scheme_end + 3 + host_end].to_string())
}

/// CSS selectors for extracting product data from search results.
#[derive(Debug, Clone)]
pub struct ProductSelectors {
    pub container: String,
    pub title: String,
    pub price: String,
    pub link: String,
    pub image: String,
}

/// Extracts the first product from search results HTML.
fn extract_first_product(
    document: &Html,
    selectors: &ProductSelectors,
    base_url: &str,
) -> Result<SitePrice, AppError> {
    let container_selector = Selector::parse(&selectors.container)
        .map_err(|e| AppError::Internal(format!("Invalid container selector: {}", e)))?;

    let title_selector = Selector::parse(&selectors.title)
        .map_err(|e| AppError::Internal(format!("Invalid title selector: {}", e)))?;

    let price_selector = Selector::parse(&selectors.price)
        .map_err(|e| AppError::Internal(format!("Invalid price selector: {}", e)))?;

    let link_selector = Selector::parse(&selectors.link)
        .map_err(|e| AppError::Internal(format!("Invalid link selector: {}", e)))?;

    let image_selector = Selector::parse(&selectors.image)
        .map_err(|e| AppError::Internal(format!("Invalid image selector: {}", e)))?;

    let container = document
        .select(&container_selector)
        .next()
        .ok_or_else(|| AppError::MissingField("No product container found".to_string()))?;

    let title = container
        .select(&title_selector)
        .next()
        .ok_or_else(|| AppError::MissingField("Product title".to_string()))?
        .text()
        .collect::<String>()
        .trim()
        .to_string();

    let price_text = container
        .select(&price_selector)
        .next()
        .ok_or_else(|| AppError::MissingField("Product price".to_string()))?
        .text()
        .collect::<String>();

    let (price, currency) = parse_price_with_currency(&price_text, None)?;
    let price_usd = price; // Default to same as price - caller should convert if needed

    let link = container
        .select(&link_selector)
        .next()
        .and_then(|el| el.value().attr("href"))
        .ok_or_else(|| AppError::MissingField("Product link".to_string()))?;

    // Convert relative link to absolute URL
    let link = if link.starts_with("http://") || link.starts_with("https://") {
        link.to_string()
    } else {
        format!(
            "{}{}",
            base_url,
            if link.starts_with('/') {
                link
            } else {
                &format!("/{}", link)
            }
        )
    };

    let image = container
        .select(&image_selector)
        .next()
        .and_then(|el| {
            // Try data-src first (for lazy-loaded images), then fall back to src
            el.value()
                .attr("data-src")
                .filter(|src| !src.contains("data:image/svg") && !src.is_empty())
                .or_else(|| el.value().attr("src"))
        })
        .map(|s| s.to_string());

    Ok(SitePrice {
        site: "Unknown".to_string(),
        title,
        price,
        currency: currency.code().to_string(),
        price_usd,
        price_converted: None,
        target_currency: None,
        link,
        image,
        match_confidence: Some(70), // Search-based match has lower confidence
    })
}
