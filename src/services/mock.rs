//! Mock data provider for demonstration and testing.
//!
//! Generates realistic product data when official APIs are not configured.

use crate::{AppError, SitePrice};
use rand::Rng;
use rust_decimal::Decimal;

/// Generates mock price data for a product search.
///
/// # Arguments
/// * `item` - Search query
/// * `site` - Site name
///
/// # Returns
/// * `Ok(SitePrice)` - Mock product data
/// * `Err(AppError)` - Random simulation of failures
pub fn generate_mock_price(item: &str, site: &str) -> Result<SitePrice, AppError> {
    let mut rng = rand::thread_rng();

    // Randomly simulate failures (10% chance)
    if rng.r#gen_bool(0.1) {
        return Err(AppError::Network(format!(
            "{} service temporarily unavailable",
            site
        )));
    }

    // Generate base price based on search query hash
    let base_price = calculate_base_price(item);
    let site_multiplier = get_site_multiplier(site);
    let random_factor: f64 = rng.r#gen();
    let price = base_price * site_multiplier * (0.9 + random_factor * 0.2);

    let price_decimal =
        Decimal::from_f64_retain((price * 100.0).round() / 100.0).unwrap_or(Decimal::new(9999, 2)); // Fallback to $99.99

    Ok(SitePrice {
        site: site.to_string(),
        title: generate_product_title(item, site),
        price: price_decimal,
        currency: "USD".to_string(), // Mock data defaults to USD
        price_usd: price_decimal,    // Same as price for mock USD data
        link: format!(
            "https://www.{}.com/product/{}",
            site.to_lowercase(),
            hash_string(item)
        ),
        image: Some(format!(
            "https://www.{}.com/images/{}.jpg",
            site.to_lowercase(),
            hash_string(item)
        )),
        match_confidence: Some(100), // Mock data is always 100% "match"
    })
}

/// Calculates base price from search query.
fn calculate_base_price(item: &str) -> f64 {
    let hash = hash_string(item);
    let base = (hash % 900) as f64 + 100.0;

    // Adjust price based on keywords
    let item_lower = item.to_lowercase();
    if item_lower.contains("laptop") || item_lower.contains("computer") {
        base * 8.0
    } else if item_lower.contains("phone") || item_lower.contains("smartphone") {
        base * 5.0
    } else if item_lower.contains("watch") || item_lower.contains("smartwatch") {
        base * 3.0
    } else if item_lower.contains("headphones") || item_lower.contains("earbuds") {
        base * 1.5
    } else {
        base
    }
}

/// Gets price multiplier for different sites.
fn get_site_multiplier(site: &str) -> f64 {
    match site {
        "Amazon" => 1.05,
        "eBay" => 0.95,
        "Jumia" => 1.02,
        "Konga" => 0.98,
        "Specialist" => 1.10,
        _ => 1.0,
    }
}

/// Generates a realistic product title.
fn generate_product_title(item: &str, site: &str) -> String {
    let brands = match item.to_lowercase().as_str() {
        s if s.contains("laptop") => vec!["Dell", "HP", "Lenovo", "Apple", "Asus"],
        s if s.contains("phone") => vec!["Samsung", "Apple", "Google", "OnePlus", "Xiaomi"],
        s if s.contains("watch") => vec!["Apple", "Samsung", "Garmin", "Fitbit", "Fossil"],
        s if s.contains("headphones") => {
            vec!["Sony", "Bose", "JBL", "Sennheiser", "Audio-Technica"]
        }
        _ => vec!["Premium", "Professional", "Ultimate", "Elite", "Advanced"],
    };

    let brand = brands[hash_string(item) as usize % brands.len()];
    let model = format!("Model {}", (hash_string(item) % 99) + 1);

    match site {
        "Amazon" => format!("{} {} - {} [Amazon Exclusive]", brand, item, model),
        "eBay" => format!("{} {} ({}) - Certified Refurbished", brand, item, model),
        "Jumia" => format!("{} {} - {} - Original", brand, item, model),
        "Konga" => format!("{} {} {} - Brand New", brand, model, item),
        _ => format!("{} {} {}", brand, model, item),
    }
}

/// Simple string hash function.
fn hash_string(s: &str) -> u32 {
    s.bytes()
        .fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_mock_price_structure() {
        let result = generate_mock_price("laptop", "Amazon");
        if let Ok(price) = result {
            assert_eq!(price.site, "Amazon");
            assert!(price.price > Decimal::ZERO);
            assert!(!price.title.is_empty());
            assert!(price.link.starts_with("https://"));
            assert!(price.image.is_some());
        }
    }

    #[test]
    fn test_calculate_base_price_consistency() {
        let price1 = calculate_base_price("laptop");
        let price2 = calculate_base_price("laptop");
        assert_eq!(price1, price2);
    }

    #[test]
    fn test_site_multiplier() {
        assert_eq!(get_site_multiplier("Amazon"), 1.05);
        assert_eq!(get_site_multiplier("eBay"), 0.95);
    }

    #[test]
    fn test_hash_string_consistency() {
        let hash1 = hash_string("test");
        let hash2 = hash_string("test");
        assert_eq!(hash1, hash2);
    }
}
