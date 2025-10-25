//! Product matching logic for cross-site price comparison.
//!
//! Implements algorithms to determine if products from different sites
//! are the same item based on identifiers, model numbers, and titles.

use crate::{ProductIdentifiers, SitePrice};
use strsim::jaro_winkler;

/// Minimum confidence score to consider a product match valid.
pub const MIN_MATCH_CONFIDENCE: u8 = 60;

/// Checks if an identifier (UPC/EAN/GTIN) appears as an exact match in text.
///
/// Ensures the identifier is not just part of a longer number or URL.
/// For example, UPC "12345" should match "UPC: 12345" but not "item/12345678"
///
/// # Arguments
/// * `text` - Text to search in (usually product title)
/// * `identifier` - Identifier to look for
///
/// # Returns
/// * `true` if identifier appears as standalone value
fn is_exact_identifier_match(text: &str, identifier: &str) -> bool {
    if identifier.len() < 8 {
        // Identifiers should be at least 8 digits (short ones cause false matches)
        return false;
    }

    // Split text by non-alphanumeric characters and check for exact match

    for word in text.split(|c: char| !c.is_alphanumeric()) {
        if word == identifier {
            return true;
        }
    }

    false
}

/// Calculates match confidence between product identifiers.
///
/// Uses a tiered approach:
/// - Exact UPC/EAN/GTIN match: 100% confidence
/// - ASIN/eBay ID match: 100% confidence
/// - Model number + brand + specs match: 95% confidence
/// - Model number + brand match: 90% confidence
/// - Partial match (model or brand): 75% confidence
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
    // IMPORTANT: Only match in title/description, NOT in URLs (URLs might contain random numbers)
    if let Some(upc) = &source.upc {
        // UPC should be in title/description, surrounded by spaces or special chars
        let title_lower = candidate.title.to_lowercase();
        let upc_lower = upc.to_lowercase();

        // Check if UPC appears as a separate word (not part of other numbers)
        if is_exact_identifier_match(&title_lower, &upc_lower) {
            tracing::debug!(
                site = %candidate.site,
                title = %candidate.title,
                upc = %upc,
                confidence = 100,
                "Exact UPC match found in title"
            );
            return 100;
        }
    }

    if let Some(ean) = &source.ean {
        let title_lower = candidate.title.to_lowercase();
        let ean_lower = ean.to_lowercase();

        if is_exact_identifier_match(&title_lower, &ean_lower) {
            tracing::debug!(
                site = %candidate.site,
                ean = %ean,
                confidence = 100,
                "Exact EAN match found in title"
            );
            return 100;
        }
    }

    if let Some(gtin) = &source.gtin {
        let title_lower = candidate.title.to_lowercase();
        let gtin_lower = gtin.to_lowercase();

        if is_exact_identifier_match(&title_lower, &gtin_lower) {
            tracing::debug!(
                site = %candidate.site,
                gtin = %gtin,
                confidence = 100,
                "Exact GTIN match found in title"
            );
            return 100;
        }
    }

    // ASIN match for Amazon products (only in links, ASINs are Amazon-specific)
    if let Some(asin) = &source.asin {
        if candidate.site.to_lowercase().contains("amazon") && candidate.link.contains(asin) {
            tracing::debug!(
                site = %candidate.site,
                asin = %asin,
                confidence = 100,
                "Exact ASIN match found"
            );
            return 100;
        }
    }

    // eBay item ID match (only for eBay site)
    if let Some(ebay_id) = &source.ebay_item_id {
        if candidate.site.to_lowercase().contains("ebay") && candidate.link.contains(ebay_id) {
            tracing::debug!(
                site = %candidate.site,
                ebay_item_id = %ebay_id,
                confidence = 100,
                "Exact eBay item ID match found"
            );
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

        // Check specification match if available
        let spec_match = check_specification_match(&source.specifications, &candidate.title);

        if model_match && brand_match && spec_match {
            tracing::debug!(
                site = %candidate.site,
                model = %model,
                brand = %brand,
                confidence = 95,
                "Model + brand + specs match"
            );
            return 95; // All match
        } else if model_match && brand_match {
            tracing::debug!(
                site = %candidate.site,
                model = %model,
                brand = %brand,
                confidence = 90,
                "Model + brand match (specs may differ)"
            );
            return 90; // Model and brand match
        } else if model_match || brand_match {
            tracing::debug!(
                site = %candidate.site,
                model_match = %model_match,
                brand_match = %brand_match,
                confidence = 75,
                "Partial match (model or brand only)"
            );
            return 75; // Partial match
        }
    }

    // Fallback to fuzzy title similarity
    let title_confidence = calculate_title_similarity(&source, &candidate);

    tracing::debug!(
        site = %candidate.site,
        candidate_title = %candidate.title,
        confidence = title_confidence,
        match_type = "fuzzy_title",
        "Fuzzy title similarity match"
    );

    title_confidence
}

/// Calculates title similarity using fuzzy string matching.
///
/// Uses Jaro-Winkler distance and keyword overlap to determine similarity.
///
/// # Arguments
/// * `source` - Source product identifiers
/// * `candidate` - Candidate product to match
///
/// # Returns
/// * Confidence score from 0-80 based on title similarity
fn calculate_title_similarity(source: &ProductIdentifiers, candidate: &SitePrice) -> u8 {
    // Extract brand and model from source if available
    let source_title = format!(
        "{} {}",
        source.brand.as_deref().unwrap_or(""),
        source.model_number.as_deref().unwrap_or("")
    )
    .trim()
    .to_lowercase();

    let candidate_title = candidate.title.to_lowercase();

    // If we have no source title info, we can't match reliably
    if source_title.is_empty() {
        return 0;
    }

    // Calculate Jaro-Winkler similarity (0.0 to 1.0)
    let similarity = jaro_winkler(&source_title, &candidate_title);

    // Extract important keywords from titles
    let source_keywords = extract_keywords(&source_title);
    let candidate_keywords = extract_keywords(&candidate_title);

    // Calculate keyword overlap
    let matching_keywords = source_keywords
        .iter()
        .filter(|k| candidate_keywords.contains(k))
        .count();

    let keyword_overlap = if !source_keywords.is_empty() {
        (matching_keywords as f64 / source_keywords.len() as f64).min(1.0)
    } else {
        0.0
    };

    // Combine fuzzy similarity and keyword overlap
    // Weight: 60% fuzzy similarity + 40% keyword overlap
    let combined_score = (similarity * 0.6) + (keyword_overlap * 0.4);

    // Convert to 0-80 confidence score
    (combined_score * 80.0) as u8
}

/// Extracts important keywords from a product title.
///
/// Filters out common words and keeps meaningful terms like brand,
/// model numbers, specifications, etc.
///
/// # Arguments
/// * `title` - Product title to extract keywords from
///
/// # Returns
/// * Vector of important keywords
fn extract_keywords(title: &str) -> Vec<String> {
    // Common stop words to filter out
    let stop_words = vec![
        "the",
        "a",
        "an",
        "and",
        "or",
        "but",
        "in",
        "on",
        "at",
        "to",
        "for",
        "of",
        "with",
        "by",
        "from",
        "as",
        "is",
        "was",
        "are",
        "were",
        "been",
        "be",
        "have",
        "has",
        "had",
        "new",
        "original",
        "official",
        "genuine",
        "authentic",
        "brand",
        "product",
        "-",
        "|",
        "/",
        ":",
        ";",
    ];

    title
        .split_whitespace()
        .map(|word| {
            word.trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase()
        })
        .filter(|word| !word.is_empty() && word.len() > 2 && !stop_words.contains(&word.as_str()))
        .collect()
}

/// Checks if product specifications match between source and candidate.
///
/// Compares specifications like color, size, storage capacity.
///
/// # Arguments
/// * `source_specs` - Source product specifications
/// * `candidate_title` - Candidate product title to check against
///
/// # Returns
/// * `true` if specifications match or no specs to compare
/// * `false` if specifications conflict
fn check_specification_match(
    source_specs: &Option<std::collections::HashMap<String, String>>,
    candidate_title: &str,
) -> bool {
    let Some(specs) = source_specs else {
        return true; // No specs to compare
    };

    let candidate_lower = candidate_title.to_lowercase();

    // Check key specifications
    for (key, value) in specs.iter() {
        let key_lower = key.to_lowercase();
        let value_lower = value.to_lowercase();

        // Important specs that must match
        if key_lower == "color" || key_lower == "size" || key_lower == "storage" {
            // If the candidate title doesn't contain this specification value,
            // it might be a different variant
            if !value_lower.is_empty() && !candidate_lower.contains(&value_lower) {
                // Check for common variations (e.g., "64gb" vs "64 gb")
                let normalized_value = value_lower.replace(" ", "");
                if !candidate_lower.replace(" ", "").contains(&normalized_value) {
                    return false;
                }
            }
        }
    }

    true
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
        mpn: None,
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
            mpn: None,
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
            price_converted: None,
            target_currency: None,
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
            mpn: None,
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
            price_converted: None,
            target_currency: None,
            link: "https://example.com".to_string(),
            image: None,
            match_confidence: None,
        };

        assert_eq!(calculate_match_confidence(&source, &candidate), 90);
    }
}
