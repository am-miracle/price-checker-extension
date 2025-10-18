//! Application configuration management.
//!
//! Loads configuration from environment variables using dotenvy.

use serde::Deserialize;
use std::env;

/// Application configuration loaded from environment variables.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub cache: CacheConfig,
    pub currency: CurrencyConfig,
    pub scraper: ScraperConfig,
}

/// HTTP server configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

/// Database connection configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

/// Redis connection configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub url: String,
}

/// Cache behavior configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct CacheConfig {
    pub ttl_seconds: u64,
}

/// Currency and exchange rate configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct CurrencyConfig {
    pub base_currency: String,
    pub api_url: String,
    pub cache_ttl_hours: u64,
}

/// Scraper behavior configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct ScraperConfig {
    pub user_agent: String,
    pub request_timeout_seconds: u64,
    pub max_retries: u32,
    pub rate_limit_per_second: u32,
    pub zenrows_api_key: Option<String>,
    pub product_match_min_confidence: u8,
    pub ebay: EbayConfig,
    pub amazon: AmazonConfig,
    pub jumia: JumiaConfig,
    pub konga: KongaConfig,
    pub use_mock_data: bool,
}

/// eBay API configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct EbayConfig {
    pub app_id: Option<String>,
    pub cert_id: Option<String>,
    pub dev_id: Option<String>,
    pub enabled: bool,
}

/// Amazon Product Advertising API configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct AmazonConfig {
    pub access_key: Option<String>,
    pub secret_key: Option<String>,
    pub partner_tag: Option<String>,
    pub marketplace: String,
    pub enabled: bool,
}

/// Jumia affiliate configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct JumiaConfig {
    pub affiliate_id: Option<String>,
    pub enabled: bool,
}

/// Konga affiliate configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct KongaConfig {
    pub affiliate_id: Option<String>,
    pub api_key: Option<String>,
    pub enabled: bool,
}

impl Config {
    /// Loads configuration from environment variables.
    ///
    /// # Returns
    /// * `Ok(Config)` - Successfully loaded configuration
    /// * `Err(String)` - Missing or invalid environment variable
    pub fn from_env() -> Result<Self, String> {
        Ok(Config {
            server: ServerConfig {
                // Always bind to 0.0.0.0 for cloud deployments (Render, Docker, etc.)
                // Only use SERVER_HOST for local development if explicitly set to something else
                host: env::var("SERVER_HOST")
                    .unwrap_or_else(|_| "0.0.0.0".to_string())
                    .replace("127.0.0.1", "0.0.0.0"), // Force 0.0.0.0 if 127.0.0.1
                // Render uses PORT env var, fallback to SERVER_PORT, then 8080
                port: env::var("PORT")
                    .or_else(|_| env::var("SERVER_PORT"))
                    .unwrap_or_else(|_| "8080".to_string())
                    .parse()
                    .map_err(|e| format!("Invalid PORT: {}", e))?,
            },
            database: DatabaseConfig {
                url: env::var("DATABASE_URL").map_err(|_| "DATABASE_URL not set".to_string())?,
            },
            redis: RedisConfig {
                url: env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            },
            cache: CacheConfig {
                ttl_seconds: env::var("CACHE_TTL_SECONDS")
                    .unwrap_or_else(|_| "300".to_string())
                    .parse()
                    .map_err(|e| format!("Invalid CACHE_TTL_SECONDS: {}", e))?,
            },
            currency: CurrencyConfig {
                base_currency: env::var("BASE_CURRENCY").unwrap_or_else(|_| "USD".to_string()),
                api_url: env::var("EXCHANGE_RATE_API_URL").unwrap_or_else(|_| {
                    "https://api.exchangerate-api.com/v4/latest/USD".to_string()
                }),
                cache_ttl_hours: env::var("EXCHANGE_RATE_CACHE_TTL_HOURS")
                    .unwrap_or_else(|_| "24".to_string())
                    .parse()
                    .map_err(|e| format!("Invalid EXCHANGE_RATE_CACHE_TTL_HOURS: {}", e))?,
            },
            scraper: ScraperConfig {
                user_agent: env::var("USER_AGENT")
                    .unwrap_or_else(|_| "PriceCheckerBot/1.0".to_string()),
                request_timeout_seconds: env::var("REQUEST_TIMEOUT_SECONDS")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()
                    .map_err(|e| format!("Invalid REQUEST_TIMEOUT_SECONDS: {}", e))?,
                max_retries: env::var("MAX_RETRIES")
                    .unwrap_or_else(|_| "3".to_string())
                    .parse()
                    .map_err(|e| format!("Invalid MAX_RETRIES: {}", e))?,
                rate_limit_per_second: env::var("RATE_LIMIT_PER_SECOND")
                    .unwrap_or_else(|_| "2".to_string())
                    .parse()
                    .map_err(|e| format!("Invalid RATE_LIMIT_PER_SECOND: {}", e))?,
                zenrows_api_key: env::var("ZENROWS_API_KEY").ok(),
                product_match_min_confidence: env::var("PRODUCT_MATCH_MIN_CONFIDENCE")
                    .unwrap_or_else(|_| "70".to_string())
                    .parse()
                    .map_err(|e| format!("Invalid PRODUCT_MATCH_MIN_CONFIDENCE: {}", e))?,
                ebay: EbayConfig {
                    app_id: env::var("EBAY_APP_ID").ok(),
                    cert_id: env::var("EBAY_CERT_ID").ok(),
                    dev_id: env::var("EBAY_DEV_ID").ok(),
                    enabled: env::var("EBAY_ENABLED")
                        .unwrap_or_else(|_| "false".to_string())
                        .parse()
                        .unwrap_or(false),
                },
                amazon: AmazonConfig {
                    access_key: env::var("AMAZON_ACCESS_KEY").ok(),
                    secret_key: env::var("AMAZON_SECRET_KEY").ok(),
                    partner_tag: env::var("AMAZON_PARTNER_TAG").ok(),
                    marketplace: env::var("AMAZON_MARKETPLACE")
                        .unwrap_or_else(|_| "US".to_string()),
                    enabled: env::var("AMAZON_ENABLED")
                        .unwrap_or_else(|_| "false".to_string())
                        .parse()
                        .unwrap_or(false),
                },
                jumia: JumiaConfig {
                    affiliate_id: env::var("JUMIA_AFFILIATE_ID").ok(),
                    enabled: env::var("JUMIA_ENABLED")
                        .unwrap_or_else(|_| "false".to_string())
                        .parse()
                        .unwrap_or(false),
                },
                konga: KongaConfig {
                    affiliate_id: env::var("KONGA_AFFILIATE_ID").ok(),
                    api_key: env::var("KONGA_API_KEY").ok(),
                    enabled: env::var("KONGA_ENABLED")
                        .unwrap_or_else(|_| "false".to_string())
                        .parse()
                        .unwrap_or(false),
                },
                use_mock_data: env::var("USE_MOCK_DATA")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
            },
        })
    }
}
