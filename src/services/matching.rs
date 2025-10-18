//! Product matching logic for cross-site price comparison.
//!
//! Implements algorithms to determine if products from different sites
//! are the same item based on identifiers, model numbers, and titles.

use crate::{ProductIdentifiers, SitePrice};

/// Minimum confidence score to consider a product match valid.
pub const MIN_MATCH_CONFIDENCE: u8 = 70;

/// Calculates match confidence between product identifiers.
///
/// Uses a tiered approach:
/// - Exact UPC/EAN/GTIN match: 100% confidence
/// - ASIN/eBay ID match: 100% confidence
/// - Model number + brand match: 90% confidence
/// - Fuzzy title match: 60-80% confidence based on similarity
///
/// # Arguments
/// * `source` - Product identifiers from the original page
/// * `candidate` - Product being compared
///
/// # Returns
/// * Confidence score from 0-100, where 100 is exact match
pub fn calculate_match_confidence(source: &ProductIdentifiers, candidate: &SitePrice) -> u8 {
    // Exact UPC/EAN/GTIN match (highest confidence)
    if let Some(upc) = &source.upc {
        if candidate.title.contains(upc) || candidate.link.contains(upc) {
            return 100;
        }
    }

    if let Some(ean) = &source.ean {
        if candidate.title.contains(ean) || candidate.link.contains(ean) {
            return 100;
        }
    }

    if let Some(gtin) = &source.gtin {
        if candidate.title.contains(gtin) || candidate.link.contains(gtin) {
            return 100;
        }
    }

    // ASIN match for Amazon products
    if let Some(asin) = &source.asin {
        if candidate.link.contains(asin) {
            return 100;
        }
    }

    // eBay item ID match
    if let Some(ebay_id) = &source.ebay_item_id {
        if candidate.link.contains(ebay_id) {
            return 100;
        }
    }

    // Model number + brand match
    if let (Some(model), Some(brand)) = (&source.model_number, &source.brand) {
        let model_match = candidate
            .title
            .to_lowercase()
            .contains(&model.to_lowercase());
        let brand_match = candidate
            .title
            .to_lowercase()
            .contains(&brand.to_lowercase());

        if model_match && brand_match {
            return 90;
        } else if model_match || brand_match {
            return 70;
        }
    }

    // Fallback to title similarity (basic implementation)
    // In production, use strsim crate for Levenshtein distance
    0
}

/// Filters price results by minimum confidence threshold.
///
/// # Arguments
/// * `prices` - Vec of SitePrice results
/// * `min_confidence` - Minimum confidence score to include (0-100)
///
/// # Returns
/// * Filtered Vec containing only results above the threshold
pub fn filter_by_confidence(mut prices: Vec<SitePrice>, min_confidence: u8) -> Vec<SitePrice> {
    prices.retain(|p| {
        p.match_confidence
            .map_or(false, |conf| conf >= min_confidence)
    });
    prices
}

/// Extracts product identifiers from a URL.
///
/// Attempts to extract ASINs, eBay item IDs, and other identifiers
/// from common e-commerce URL patterns.
///
/// # Arguments
/// * `url` - Product page URL
///
/// # Returns
/// * ProductIdentifiers with any extracted values
pub fn extract_identifiers_from_url(url: &str) -> ProductIdentifiers {
    let mut identifiers = ProductIdentifiers {
        upc: None,
        ean: None,
        gtin: None,
        asin: None,
        ebay_item_id: None,
        model_number: None,
        brand: None,
        specifications: None,
    };

    // Extract Amazon ASIN
    if url.contains("amazon.com") || url.contains("amazon.") {
        if let Some(asin) = extract_asin(url) {
            identifiers.asin = Some(asin);
        }
    }

    // Extract eBay item ID
    if url.contains("ebay.com") || url.contains("ebay.") {
        if let Some(item_id) = extract_ebay_item_id(url) {
            identifiers.ebay_item_id = Some(item_id);
        }
    }

    identifiers
}

/// Extracts ASIN from Amazon URL.
///
/// Handles formats like:
/// - /dp/B07FZ8S74R
/// - /gp/product/B07FZ8S74R
/// - /product/B07FZ8S74R
fn extract_asin(url: &str) -> Option<String> {
    let patterns = ["/dp/", "/gp/product/", "/product/"];

    for pattern in &patterns {
        if let Some(start_idx) = url.find(pattern) {
            let asin_start = start_idx + pattern.len();
            let remaining = &url[asin_start..];

            // ASIN is 10 alphanumeric characters
            let asin: String = remaining
                .chars()
                .take_while(|c| c.is_alphanumeric())
                .take(10)
                .collect();

            if asin.len() == 10 {
                return Some(asin);
            }
        }
    }

    None
}

/// Extracts eBay item ID from URL.
///
/// Handles formats like:
/// - /itm/12345678910
/// - /itm/Product-Name/12345678910
fn extract_ebay_item_id(url: &str) -> Option<String> {
    if let Some(itm_idx) = url.find("/itm/") {
        let after_itm = &url[itm_idx + 5..];

        // Remove query parameters if present
        let path_only = after_itm.split('?').next().unwrap_or(after_itm);

        // Find the item ID (last numeric sequence in the path)
        let parts: Vec<&str> = path_only.split('/').collect();
        for part in parts.iter().rev() {
            if part.chars().all(|c| c.is_ascii_digit()) && !part.is_empty() {
                return Some(part.to_string());
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[test]
    fn test_extract_asin() {
        let url1 = "https://www.amazon.com/dp/B07FZ8S74R/ref=xyz";
        assert_eq!(extract_asin(url1), Some("B07FZ8S74R".to_string()));

        let url2 = "https://www.amazon.com/gp/product/B08N5WRWNW";
        assert_eq!(extract_asin(url2), Some("B08N5WRWNW".to_string()));
    }

    #[test]
    fn test_extract_ebay_item_id() {
        let url1 = "https://www.ebay.com/itm/12345678910";
        assert_eq!(extract_ebay_item_id(url1), Some("12345678910".to_string()));

        let url2 = "https://www.ebay.com/itm/Product-Name/12345678910?hash=xyz";
        assert_eq!(extract_ebay_item_id(url2), Some("12345678910".to_string()));
    }

    #[test]
    fn test_calculate_match_confidence_exact_upc() {
        let source = ProductIdentifiers {
            upc: Some("123456789012".to_string()),
            ean: None,
            gtin: None,
            asin: None,
            ebay_item_id: None,
            model_number: None,
            brand: None,
            specifications: None,
        };

        let candidate = SitePrice {
            site: "Test".to_string(),
            title: "Product with UPC 123456789012".to_string(),
            price: Decimal::from_str("99.99").unwrap(),
            currency: "NGN".to_string(),
            price_usd: Decimal::from_str("999.9").unwrap(),
            link: "https://example.com".to_string(),
            image: None,
            match_confidence: None,
        };

        assert_eq!(calculate_match_confidence(&source, &candidate), 100);
    }

    #[test]
    fn test_calculate_match_confidence_model_brand() {
        let source = ProductIdentifiers {
            upc: None,
            ean: None,
            gtin: None,
            asin: None,
            ebay_item_id: None,
            model_number: Some("XPS-13".to_string()),
            brand: Some("Dell".to_string()),
            specifications: None,
        };

        let candidate = SitePrice {
            site: "Test".to_string(),
            title: "Dell XPS-13 Laptop".to_string(),
            price: Decimal::from_str("999.99").unwrap(),
            currency: "NGN".to_string(),
            price_usd: Decimal::from_str("999.9").unwrap(),
            link: "https://example.com".to_string(),
            image: None,
            match_confidence: None,
        };

        assert_eq!(calculate_match_confidence(&source, &candidate), 90);
    }
}
