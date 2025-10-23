use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Product identifiers used for matching across sites.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProductIdentifiers {
    /// Universal Product Code
    pub upc: Option<String>,
    /// European Article Number
    pub ean: Option<String>,
    /// Global Trade Item Number
    pub gtin: Option<String>,
    /// Amazon Standard Identification Number
    pub asin: Option<String>,
    /// eBay Item Number
    pub ebay_item_id: Option<String>,
    /// Manufacturer Part Number
    pub mpn: Option<String>,
    /// Manufacturer model number
    pub model_number: Option<String>,
    /// Product brand name
    pub brand: Option<String>,
    /// Product specifications (e.g., RAM, storage, color)
    pub specifications: Option<HashMap<String, String>>,
}

/// Request body for product comparison with detailed identifiers.
#[derive(Debug, Deserialize)]
pub struct ProductMatchRequest {
    /// Product title from the current page
    pub title: String,
    /// Current price on the page being viewed (in original currency)
    pub current_price: Option<Decimal>,
    /// Currency code of current price (e.g., "USD", "GBP", "NGN")
    pub currency: Option<String>,
    /// Current site name (e.g., "amazon", "ebay")
    pub current_site: Option<String>,
    /// Product URL from the current page
    pub url: Option<String>,
    /// Target currency for price conversion (optional)
    pub target_currency: Option<String>,
    /// Product identifiers
    #[serde(default)]
    pub identifiers: ProductIdentifiers,
}

/// Price information from a specific site with match confidence.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SitePrice {
    pub site: String,
    pub title: String,
    /// Price in original currency (use Decimal for precision)
    pub price: Decimal,
    /// ISO 4217 currency code (e.g., "USD", "GBP", "NGN")
    pub currency: String,
    /// Price converted to USD for comparison
    pub price_usd: Decimal,
    /// Price converted to target currency (if requested)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_converted: Option<Decimal>,
    /// Target currency code (if conversion was requested)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_currency: Option<String>,
    pub link: String,
    pub image: Option<String>,
    /// Match confidence score (0-100), where 100 is exact match
    pub match_confidence: Option<u8>,
}

impl SitePrice {
    /// Creates a new SitePrice with all required fields.
    pub fn new(
        site: String,
        title: String,
        price: Decimal,
        currency: String,
        price_usd: Decimal,
        link: String,
        image: Option<String>,
        match_confidence: Option<u8>,
    ) -> Self {
        Self {
            site,
            title,
            price,
            currency,
            price_usd,
            price_converted: None,
            target_currency: None,
            link,
            image,
            match_confidence,
        }
    }

    /// Sets the converted price and target currency.
    pub fn with_conversion(mut self, price_converted: Decimal, target_currency: String) -> Self {
        self.price_converted = Some(price_converted);
        self.target_currency = Some(target_currency);
        self
    }
}

/// Result of price comparison across multiple sites.
#[derive(Serialize, Deserialize, Debug)]
pub struct PriceComparisonResult {
    pub best_deal: Option<SitePrice>,
    pub all_prices: Vec<SitePrice>,
}

impl PriceComparisonResult {
    /// Creates a new comparison result and finds the best deal.
    pub fn new(mut prices: Vec<SitePrice>) -> Self {
        // Sort by USD price to find best deal
        prices.sort_by(|a, b| a.price_usd.cmp(&b.price_usd));

        let best_deal = prices.first().cloned();

        Self {
            best_deal,
            all_prices: prices,
        }
    }
}
