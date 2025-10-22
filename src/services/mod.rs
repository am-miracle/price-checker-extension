//! Price scraping and comparison services.
//!
//! This module coordinates fetching prices from multiple e-commerce platforms
//! concurrently and aggregates results with product matching.

pub mod amazon;
pub mod currency;
pub mod ebay;
pub mod jumia;
pub mod konga;
pub mod matching;
pub mod mock;
pub mod zenrows;

use crate::{AppError, AppState, PriceComparisonResult, ProductIdentifiers, SitePrice};
use std::sync::Arc;

/// Compares prices across all supported platforms with product identifiers.
///
/// Uses product identifiers (UPC, ASIN, model number) for accurate matching
/// across different e-commerce sites. Fetches from all platforms concurrently,
/// calculates match confidence, and filters by minimum threshold.
///
/// # Arguments
/// * `identifiers` - Product identifiers for matching
/// * `search_query` - Search query text
/// * `state` - Application state with configuration and HTTP client
/// * `target_currency` - Optional target currency for price conversion
///
/// # Returns
/// * `Ok(PriceComparisonResult)` - Comparison results with confidence scores
/// * `Err(AppError)` - Only fails if all scrapers fail
pub async fn compare_with_identifiers(
    identifiers: &ProductIdentifiers,
    search_query: &str,
    state: &Arc<AppState>,
    target_currency: Option<&str>,
) -> Result<PriceComparisonResult, AppError> {
    tracing::info!(
        query = %search_query,
        has_upc = identifiers.upc.is_some(),
        has_asin = identifiers.asin.is_some(),
        has_model = identifiers.model_number.is_some(),
        target_currency = ?target_currency,
        "Starting product comparison with identifiers"
    );

    let mut all_prices: Vec<SitePrice> = Vec::new();

    // Use mock data if configured
    if state.config.scraper.use_mock_data {
        tracing::info!("Using mock data for demonstration");

        if let Ok(price) = mock::generate_mock_price(search_query, "Amazon") {
            all_prices.push(price);
        }
        if let Ok(price) = mock::generate_mock_price(search_query, "eBay") {
            all_prices.push(price);
        }
        if let Ok(price) = mock::generate_mock_price(search_query, "Jumia") {
            all_prices.push(price);
        }
        if let Ok(price) = mock::generate_mock_price(search_query, "Konga") {
            all_prices.push(price);
        }
    } else {
        // Use ZenRows for scraping
        let zenrows_config = state
            .config
            .scraper
            .zenrows_api_key
            .as_ref()
            .map(|key| zenrows::ZenRowsConfig::new(key.clone()));

        // Launch all scrapers concurrently
        let (amazon_result, ebay_result, jumia_result, konga_result) = tokio::join!(
            amazon::fetch_price(
                identifiers,
                search_query,
                &state.http_client,
                &state.config.scraper.amazon,
                zenrows_config.as_ref()
            ),
            ebay::fetch_price(
                identifiers,
                search_query,
                &state.http_client,
                &state.config.scraper.ebay,
                zenrows_config.as_ref()
            ),
            jumia::fetch_price(
                identifiers,
                search_query,
                &state.http_client,
                &state.config.scraper.jumia,
                zenrows_config.as_ref()
            ),
            konga::fetch_price(
                identifiers,
                search_query,
                &state.http_client,
                &state.config.scraper.konga,
                zenrows_config.as_ref()
            ),
        );

        // Collect successful results
        if let Ok(mut price) = amazon_result {
            // Calculate match confidence
            if price.match_confidence.is_none() {
                price.match_confidence =
                    Some(matching::calculate_match_confidence(identifiers, &price));
            }
            all_prices.push(price);
        } else if let Err(e) = amazon_result {
            tracing::debug!(error = %e, "Amazon fetch failed");
        }

        if let Ok(mut price) = ebay_result {
            if price.match_confidence.is_none() {
                price.match_confidence =
                    Some(matching::calculate_match_confidence(identifiers, &price));
            }
            all_prices.push(price);
        } else if let Err(e) = ebay_result {
            tracing::debug!(error = %e, "eBay fetch failed");
        }

        if let Ok(mut price) = jumia_result {
            if price.match_confidence.is_none() {
                price.match_confidence =
                    Some(matching::calculate_match_confidence(identifiers, &price));
            }
            all_prices.push(price);
        } else if let Err(e) = jumia_result {
            tracing::debug!(error = %e, "Jumia fetch failed");
        }

        if let Ok(mut price) = konga_result {
            if price.match_confidence.is_none() {
                price.match_confidence =
                    Some(matching::calculate_match_confidence(identifiers, &price));
            }
            all_prices.push(price);
        } else if let Err(e) = konga_result {
            tracing::debug!(error = %e, "Konga fetch failed");
        }
    }

    // Filter by minimum confidence threshold
    let min_confidence = state.config.scraper.product_match_min_confidence;
    all_prices = matching::filter_by_confidence(all_prices, min_confidence);

    // Return error if all scrapers failed or no matches above threshold
    if all_prices.is_empty() {
        tracing::error!(query = %search_query, "No products found above confidence threshold");
        return Err(AppError::Internal(format!(
            "No products found matching confidence threshold of {}%. \
                Check credentials, enable platforms, or lower PRODUCT_MATCH_MIN_CONFIDENCE. \
                Set USE_MOCK_DATA=true for demonstration.",
            min_confidence
        )));
    }

    // Convert prices to target currency if specified
    if let Some(target_curr) = target_currency {
        all_prices = convert_prices_to_currency(all_prices, target_curr, state).await?;
    }

    // Sort by price ascending (lowest price first)
    all_prices.sort_by(|a, b| {
        a.price_usd
            .partial_cmp(&b.price_usd)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let best_deal = all_prices.first().cloned();

    tracing::info!(
        query = %search_query,
        total_results = all_prices.len(),
        best_price = ?best_deal.as_ref().map(|p| p.price),
        "Price comparison completed"
    );

    Ok(PriceComparisonResult {
        best_deal,
        all_prices,
    })
}

/// Converts all prices in the result to a target currency.
///
/// # Arguments
/// * `prices` - Vector of site prices to convert
/// * `target_currency` - Target currency code (e.g., "GBP", "EUR")
/// * `state` - Application state with currency service
///
/// # Returns
/// * `Ok(Vec<SitePrice>)` - Prices with conversion applied
/// * `Err(AppError)` - If currency conversion fails
async fn convert_prices_to_currency(
    mut prices: Vec<SitePrice>,
    target_currency: &str,
    state: &Arc<AppState>,
) -> Result<Vec<SitePrice>, AppError> {
    use currency::Currency;
    use std::str::FromStr;

    let target_curr = Currency::from_str(target_currency)?;

    tracing::debug!(
        target_currency = %target_currency,
        price_count = prices.len(),
        "Converting prices to target currency"
    );

    for price in &mut prices {
        // Parse the original currency
        let source_curr = Currency::from_str(&price.currency).unwrap_or(Currency::USD);

        // Convert from source currency to target currency
        let converted = state
            .currency_service
            .convert(price.price, &source_curr, &target_curr)
            .await
            .unwrap_or(price.price); // Fallback to original if conversion fails

        // Round to 2 decimal places for display
        let rounded = converted.round_dp(2);

        price.price_converted = Some(rounded);
        price.target_currency = Some(target_currency.to_string());
    }

    Ok(prices)
}

/// Simple price comparison using just a search query (backward compatibility).
///
/// Creates basic ProductIdentifiers from the search query and calls
/// compare_with_identifiers. Used by the GET /api/compare?item= endpoint.
///
/// # Arguments
/// * `item` - Search query for the product
/// * `state` - Application state with configuration and HTTP client
///
/// # Returns
/// * `Ok(PriceComparisonResult)` - Comparison results with best deal and all prices
/// * `Err(AppError)` - Only fails if all scrapers fail
pub async fn compare_all(
    item: &str,
    state: &Arc<AppState>,
) -> Result<PriceComparisonResult, AppError> {
    // Create basic identifiers from search query
    let identifiers = ProductIdentifiers {
        upc: None,
        ean: None,
        gtin: None,
        asin: None,
        mpn: None,
        ebay_item_id: None,
        model_number: None,
        brand: None,
        specifications: None,
    };

    compare_with_identifiers(&identifiers, item, state, None).await
}
