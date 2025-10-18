//! Currency detection, conversion, and exchange rate management.
//!
//! Provides accurate currency handling using Decimal for precision,
//! with real-time exchange rates cached in Redis.

use crate::AppError;
use deadpool_redis::Pool;
use reqwest::Client;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

/// Supported currencies with ISO 4217 codes.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Currency {
    USD, // US Dollar
    EUR, // Euro
    GBP, // British Pound
    NGN, // Nigerian Naira
    INR, // Indian Rupee
    CAD, // Canadian Dollar
    AUD, // Australian Dollar
    JPY, // Japanese Yen
}

impl Currency {
    /// Returns the currency symbol.
    pub fn symbol(&self) -> &str {
        match self {
            Currency::USD => "$",
            Currency::EUR => "€",
            Currency::GBP => "£",
            Currency::NGN => "₦",
            Currency::INR => "₹",
            Currency::CAD => "C$",
            Currency::AUD => "A$",
            Currency::JPY => "¥",
        }
    }

    /// Returns the currency code (ISO 4217).
    pub fn code(&self) -> &str {
        match self {
            Currency::USD => "USD",
            Currency::EUR => "EUR",
            Currency::GBP => "GBP",
            Currency::NGN => "NGN",
            Currency::INR => "INR",
            Currency::CAD => "CAD",
            Currency::AUD => "AUD",
            Currency::JPY => "JPY",
        }
    }

    /// Returns static fallback conversion rate to USD.
    /// Used only when exchange rate API is unavailable.
    pub fn fallback_to_usd_rate(&self) -> Decimal {
        match self {
            Currency::USD => Decimal::from(1),
            Currency::EUR => Decimal::from_str("1.08").unwrap(),
            Currency::GBP => Decimal::from_str("1.27").unwrap(),
            Currency::NGN => Decimal::from_str("0.0013").unwrap(),
            Currency::INR => Decimal::from_str("0.012").unwrap(),
            Currency::CAD => Decimal::from_str("0.74").unwrap(),
            Currency::AUD => Decimal::from_str("0.66").unwrap(),
            Currency::JPY => Decimal::from_str("0.0067").unwrap(),
        }
    }
}

impl FromStr for Currency {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "USD" => Ok(Currency::USD),
            "EUR" => Ok(Currency::EUR),
            "GBP" => Ok(Currency::GBP),
            "NGN" => Ok(Currency::NGN),
            "INR" => Ok(Currency::INR),
            "CAD" => Ok(Currency::CAD),
            "AUD" => Ok(Currency::AUD),
            "JPY" => Ok(Currency::JPY),
            _ => Err(AppError::Parse(format!("Unsupported currency: {}", s))),
        }
    }
}

/// Exchange rate API response structure (ExchangeRate-API format).
#[derive(Debug, Deserialize)]
struct ExchangeRateResponse {
    result: String,
    base_code: String,
    conversion_rates: HashMap<String, f64>,
}

/// Cached exchange rates with timestamp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeRates {
    pub base: String,
    pub rates: HashMap<String, Decimal>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Currency service for handling conversions and exchange rates.
pub struct CurrencyService {
    redis_pool: Pool,
    http_client: Client,
    api_url: String,
    cache_ttl_hours: u64,
}

impl CurrencyService {
    /// Creates a new currency service.
    ///
    /// # Arguments
    /// * `redis_pool` - Redis connection pool for caching
    /// * `http_client` - HTTP client for API requests
    /// * `api_url` - Exchange rate API URL (default: ExchangeRate-API)
    /// * `cache_ttl_hours` - Cache time-to-live in hours
    pub fn new(
        redis_pool: Pool,
        http_client: Client,
        api_url: String,
        cache_ttl_hours: u64,
    ) -> Self {
        Self {
            redis_pool,
            http_client,
            api_url,
            cache_ttl_hours,
        }
    }

    /// Fetches exchange rates from cache or API.
    ///
    /// # Returns
    /// * `Ok(ExchangeRates)` - Current exchange rates
    /// * `Err(AppError)` - If both cache and API fail
    pub async fn get_exchange_rates(&self) -> Result<ExchangeRates, AppError> {
        // Try to get from cache first
        if let Ok(cached) = self.get_cached_rates().await {
            tracing::debug!("Using cached exchange rates from {}", cached.updated_at);
            return Ok(cached);
        }

        // Cache miss or expired - fetch from API
        tracing::info!("Fetching fresh exchange rates from API");
        self.fetch_and_cache_rates().await
    }

    /// Fetches rates from cache.
    async fn get_cached_rates(&self) -> Result<ExchangeRates, AppError> {
        let mut conn = self
            .redis_pool
            .get()
            .await
            .map_err(|e| AppError::Cache(format!("Redis connection failed: {}", e)))?;

        let cached_json: String = redis::cmd("GET")
            .arg("exchange_rates:usd")
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::Cache(format!("Cache read failed: {}", e)))?;

        serde_json::from_str(&cached_json)
            .map_err(|e| AppError::Cache(format!("Invalid cached data: {}", e)))
    }

    /// Fetches rates from API and caches them.
    async fn fetch_and_cache_rates(&self) -> Result<ExchangeRates, AppError> {
        // Fetch from API
        let response = self
            .http_client
            .get(&self.api_url)
            .send()
            .await
            .map_err(|e| AppError::Network(format!("Exchange rate API request failed: {}", e)))?;

        if !response.status().is_success() {
            // API failed - try to use fallback rates
            tracing::warn!(
                "Exchange rate API returned {}, using fallback rates",
                response.status()
            );
            return Ok(self.fallback_rates());
        }

        let api_response: ExchangeRateResponse = response
            .json()
            .await
            .map_err(|e| AppError::Network(format!("Failed to parse API response: {}", e)))?;

        if api_response.result != "success" {
            tracing::warn!("Exchange rate API returned non-success result, using fallback");
            return Ok(self.fallback_rates());
        }

        // Convert f64 rates to Decimal
        let mut rates = HashMap::new();
        for (code, rate) in api_response.conversion_rates {
            if let Some(decimal_rate) = Decimal::from_f64_retain(rate) {
                rates.insert(code, decimal_rate);
            }
        }

        let exchange_rates = ExchangeRates {
            base: api_response.base_code,
            rates,
            updated_at: chrono::Utc::now(),
        };

        // Cache the rates
        if let Err(e) = self.cache_rates(&exchange_rates).await {
            tracing::warn!("Failed to cache exchange rates: {}", e);
        }

        Ok(exchange_rates)
    }

    /// Caches exchange rates in Redis with TTL.
    async fn cache_rates(&self, rates: &ExchangeRates) -> Result<(), AppError> {
        let json = serde_json::to_string(rates)
            .map_err(|e| AppError::Cache(format!("Failed to serialize rates: {}", e)))?;

        let mut conn = self
            .redis_pool
            .get()
            .await
            .map_err(|e| AppError::Cache(format!("Redis connection failed: {}", e)))?;

        let ttl_seconds = self.cache_ttl_hours * 3600;

        let _: () = redis::cmd("SETEX")
            .arg("exchange_rates:usd")
            .arg(ttl_seconds)
            .arg(json)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::Cache(format!("Cache write failed: {}", e)))?;

        Ok(())
    }

    /// Returns fallback exchange rates when API is unavailable.
    fn fallback_rates(&self) -> ExchangeRates {
        let mut rates = HashMap::new();

        // Add all supported currencies with fallback rates
        for currency in [
            Currency::USD,
            Currency::EUR,
            Currency::GBP,
            Currency::NGN,
            Currency::INR,
            Currency::CAD,
            Currency::AUD,
            Currency::JPY,
        ] {
            rates.insert(currency.code().to_string(), currency.fallback_to_usd_rate());
        }

        ExchangeRates {
            base: "USD".to_string(),
            rates,
            updated_at: chrono::Utc::now(),
        }
    }

    /// Converts an amount from one currency to another.
    ///
    /// # Arguments
    /// * `amount` - Amount to convert
    /// * `from` - Source currency
    /// * `to` - Target currency
    ///
    /// # Returns
    /// * Converted amount in target currency
    pub async fn convert(
        &self,
        amount: Decimal,
        from: &Currency,
        to: &Currency,
    ) -> Result<Decimal, AppError> {
        if from == to {
            return Ok(amount);
        }

        let rates = self.get_exchange_rates().await?;

        // Convert from source to USD, then USD to target
        let from_rate = rates
            .rates
            .get(from.code())
            .copied()
            .unwrap_or_else(|| from.fallback_to_usd_rate());

        let to_rate = rates
            .rates
            .get(to.code())
            .copied()
            .unwrap_or_else(|| to.fallback_to_usd_rate());

        // If base is USD: amount_in_target = amount_in_source / from_rate * to_rate
        // Since ExchangeRate-API uses USD as base, rates are already USD-based
        let amount_in_usd = amount / from_rate;
        let result = amount_in_usd * to_rate;

        Ok(result)
    }

    /// Converts an amount to USD using current rates.
    pub async fn convert_to_usd(
        &self,
        amount: Decimal,
        from: &Currency,
    ) -> Result<Decimal, AppError> {
        self.convert(amount, from, &Currency::USD).await
    }
}

/// Detects currency from a price string.
///
/// Looks for currency symbols and codes in the string.
///
/// # Arguments
/// * `price_str` - Price string like "$1,299.99" or "₦50,000"
/// * `site_hint` - Optional site name to help detect currency (e.g., "Jumia" -> NGN)
///
/// # Returns
/// * Detected currency or USD as default
pub fn detect_currency(price_str: &str, site_hint: Option<&str>) -> Currency {
    // Check for currency symbols (order matters - check specific symbols first)
    if price_str.contains("C$") {
        return Currency::CAD;
    }
    if price_str.contains("A$") {
        return Currency::AUD;
    }
    if price_str.contains('$') {
        return Currency::USD;
    }
    if price_str.contains("€") {
        return Currency::EUR;
    }
    if price_str.contains("£") {
        return Currency::GBP;
    }
    if price_str.contains("₦") {
        return Currency::NGN;
    }
    if price_str.contains("₹") {
        return Currency::INR;
    }
    if price_str.contains("¥") {
        return Currency::JPY;
    }

    // Check for currency codes
    let upper = price_str.to_uppercase();
    if upper.contains("USD") {
        return Currency::USD;
    }
    if upper.contains("EUR") {
        return Currency::EUR;
    }
    if upper.contains("GBP") {
        return Currency::GBP;
    }
    if upper.contains("NGN") {
        return Currency::NGN;
    }
    if upper.contains("INR") {
        return Currency::INR;
    }
    if upper.contains("CAD") {
        return Currency::CAD;
    }
    if upper.contains("AUD") {
        return Currency::AUD;
    }
    if upper.contains("JPY") {
        return Currency::JPY;
    }

    // Use site hint if available
    if let Some(site) = site_hint {
        let site_lower = site.to_lowercase();
        if site_lower.contains("jumia") || site_lower.contains("konga") {
            return Currency::NGN;
        }
        if site_lower.contains("amazon.co.uk") || site_lower.contains("ebay.co.uk") {
            return Currency::GBP;
        }
        if site_lower.contains("amazon.de") || site_lower.contains("amazon.fr") {
            return Currency::EUR;
        }
        if site_lower.contains("amazon.ca") {
            return Currency::CAD;
        }
        if site_lower.contains("amazon.com.au") {
            return Currency::AUD;
        }
        if site_lower.contains("amazon.in") {
            return Currency::INR;
        }
        if site_lower.contains("amazon.co.jp") {
            return Currency::JPY;
        }
    }

    // Default to USD
    Currency::USD
}

/// Parses price string and extracts numeric value with currency.
///
/// # Arguments
/// * `price_str` - Price string like "$1,299.99" or "€1.299,99"
/// * `site_hint` - Optional site name for currency detection
///
/// # Returns
/// * Tuple of (numeric_value, detected_currency)
pub fn parse_price_with_currency(
    price_str: &str,
    site_hint: Option<&str>,
) -> Result<(Decimal, Currency), AppError> {
    let currency = detect_currency(price_str, site_hint);

    // Remove all non-numeric characters except . and ,
    let cleaned = price_str
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.' || *c == ',')
        .collect::<String>();

    if cleaned.is_empty() {
        return Err(AppError::Parse(format!(
            "No numeric value found in price: {}",
            price_str
        )));
    }

    // Handle European format (comma as decimal separator)
    // European: 1.299,99 -> 1299.99
    // US: 1,299.99 -> 1299.99
    // NGN: 50,000 -> 50000
    let normalized = if cleaned.contains('.') && cleaned.contains(',') {
        // Both separators present - determine which is decimal
        let dot_pos = cleaned.rfind('.').unwrap();
        let comma_pos = cleaned.rfind(',').unwrap();
        if comma_pos > dot_pos {
            // Comma is decimal separator (European: 1.299,99)
            cleaned.replace('.', "").replace(',', ".")
        } else {
            // Dot is decimal separator (US: 1,299.99)
            cleaned.replace(',', "")
        }
    } else if cleaned.contains(',') && !cleaned.contains('.') {
        // Only comma - could be thousands separator (50,000) or decimal (50,00)
        // If there are digits after comma, check the count
        if let Some(comma_pos) = cleaned.find(',') {
            let after_comma = &cleaned[comma_pos + 1..];
            if after_comma.len() == 2 {
                // Likely decimal separator (European: 50,00)
                cleaned.replace(',', ".")
            } else {
                // Likely thousands separator (50,000 or 1,000,000)
                cleaned.replace(',', "")
            }
        } else {
            cleaned
        }
    } else {
        // No comma or only dot - use as is
        cleaned
    };

    let value = Decimal::from_str(&normalized)
        .map_err(|_| AppError::Parse(format!("Invalid price format: {}", price_str)))?;

    Ok((value, currency))
}

/// Price with currency information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceWithCurrency {
    pub amount: Decimal,
    pub currency: Currency,
    pub amount_usd: Decimal,
}

impl PriceWithCurrency {
    /// Creates a new price with a specific USD amount.
    /// Used when USD conversion is already known (e.g., from service).
    pub fn new(amount: Decimal, currency: Currency, amount_usd: Decimal) -> Self {
        Self {
            amount,
            currency,
            amount_usd,
        }
    }

    /// Creates a price from a string using fallback conversion rates.
    /// For production, use CurrencyService::convert() instead.
    pub fn from_string(price_str: &str, site_hint: Option<&str>) -> Result<Self, AppError> {
        let (amount, currency) = parse_price_with_currency(price_str, site_hint)?;
        let amount_usd = amount * currency.fallback_to_usd_rate();

        Ok(Self {
            amount,
            currency,
            amount_usd,
        })
    }

    /// Formats the price with its original currency.
    pub fn format(&self) -> String {
        format!("{}{}", self.currency.symbol(), self.amount.round_dp(2))
    }

    /// Formats the price in USD.
    pub fn format_usd(&self) -> String {
        format!("${}", self.amount_usd.round_dp(2))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_currency() {
        assert_eq!(detect_currency("$1,299.99", None), Currency::USD);
        assert_eq!(detect_currency("£999", None), Currency::GBP);
        assert_eq!(detect_currency("€1.299,99", None), Currency::EUR);
        assert_eq!(detect_currency("₦50,000", None), Currency::NGN);
        assert_eq!(detect_currency("C$100", None), Currency::CAD);
        assert_eq!(detect_currency("A$200", None), Currency::AUD);
    }

    #[test]
    fn test_parse_price_with_currency() {
        let (amount, currency) = parse_price_with_currency("$1,299.99", None).unwrap();
        assert_eq!(amount, Decimal::from_str("1299.99").unwrap());
        assert_eq!(currency, Currency::USD);

        let (amount, currency) = parse_price_with_currency("₦50,000", None).unwrap();
        assert_eq!(amount, Decimal::from_str("50000").unwrap());
        assert_eq!(currency, Currency::NGN);

        // European format
        let (amount, currency) = parse_price_with_currency("€1.299,99", None).unwrap();
        assert_eq!(amount, Decimal::from_str("1299.99").unwrap());
        assert_eq!(currency, Currency::EUR);
    }

    #[test]
    fn test_price_with_currency() {
        let price = PriceWithCurrency::from_string("£999", None).unwrap();
        assert_eq!(price.amount, Decimal::from_str("999").unwrap());
        assert_eq!(price.currency, Currency::GBP);
        // £999 * 1.27 ≈ $1268.73
        assert!(price.amount_usd > Decimal::from_str("1268").unwrap());
    }

    #[test]
    fn test_site_hint_detection() {
        let (_, currency) = parse_price_with_currency("50,000", Some("Jumia")).unwrap();
        assert_eq!(currency, Currency::NGN);

        let (_, currency) = parse_price_with_currency("999", Some("amazon.co.uk")).unwrap();
        assert_eq!(currency, Currency::GBP);
    }

    #[test]
    fn test_currency_from_str() {
        assert_eq!(Currency::from_str("USD").unwrap(), Currency::USD);
        assert_eq!(Currency::from_str("ngn").unwrap(), Currency::NGN);
        assert!(Currency::from_str("INVALID").is_err());
    }

    #[test]
    fn test_fallback_rates() {
        let usd = Currency::USD.fallback_to_usd_rate();
        assert_eq!(usd, Decimal::from(1));

        let gbp = Currency::GBP.fallback_to_usd_rate();
        assert!(gbp > Decimal::from(1)); // GBP is stronger than USD
    }
}
