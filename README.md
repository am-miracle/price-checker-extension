# Price Checker Extension

Backend API service for comparing product prices across e-commerce platforms. Built with Rust, Axum, PostgreSQL, and Redis.


## Quick Start

### Prerequisites

- **Rust** 1.75+ ([Install Rust](https://rustup.rs/))
- **Docker** & **Docker Compose** (for PostgreSQL and Redis)
- **SQLx CLI** (optional, for database migrations)

```bash
cargo install sqlx-cli --no-default-features --features postgres
```

### 1. Clone and Setup

```bash
git clone <repository-url>
cd price-checker-extension
cp .env.example .env
```

### 2. Configure Environment

Edit `.env` with your settings:

```bash
# Server Configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# Database Configuration
DATABASE_URL=postgresql://postgres:password@localhost:5432/price_checker

# Redis Configuration
REDIS_URL=redis://localhost:6379

# Cache Configuration
CACHE_TTL_SECONDS=300

# Currency Configuration
BASE_CURRENCY=USD
EXCHANGE_RATE_API_URL=https://api.exchangerate-api.com/v4/latest/USD
EXCHANGE_RATE_CACHE_TTL_HOURS=24

# Logging
RUST_LOG=info,price_checker_extension=debug

# Scraper Configuration
USER_AGENT=PriceCheckerBot/1.0 (+https://yoursite.com/bot)
REQUEST_TIMEOUT_SECONDS=30
MAX_RETRIES=3
RATE_LIMIT_PER_SECOND=2
```

### 3. Start Dependencies

```bash
docker-compose up -d
```

This starts:
- **PostgreSQL** on port 5432
- **Redis** on port 6379

### 4. Run Database Migrations

Migrations run automatically on server startup, or manually:

```bash
sqlx migrate run
```

### 5. Build and Run

```bash
# Development mode
cargo run

# Production build
cargo build --release
./target/release/price-checker-extension
```



## Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_parse_price
```

**Current Test Coverage:**
- Cache key generation
- Price parsing (USD, EUR, NGN)
- Rate limiter initialization
- HTML text/attribute extraction
- Currency detection and conversion
- Exchange rate caching

## Currency Handling

The service uses a robust currency handling system with the following features:

### Supported Currencies
- **USD** (US Dollar) - Base currency
- **EUR** (Euro)
- **GBP** (British Pound)
- **NGN** (Nigerian Naira)
- **INR** (Indian Rupee)
- **CAD** (Canadian Dollar)
- **AUD** (Australian Dollar)
- **JPY** (Japanese Yen)

### Key Features
1. **Decimal Precision**: Uses `rust_decimal::Decimal` instead of `f64` to avoid floating-point errors
2. **Real-time Exchange Rates**: Fetches current rates from ExchangeRate-API
3. **Redis Caching**: Caches exchange rates for 24 hours (configurable)
4. **Automatic Fallback**: Uses static fallback rates if API is unavailable
5. **Multi-currency Storage**: Stores both original price and USD equivalent in database

### Configuration

```bash
# Currency settings in .env
BASE_CURRENCY=USD
EXCHANGE_RATE_API_URL=https://api.exchangerate-api.com/v4/latest/USD
EXCHANGE_RATE_CACHE_TTL_HOURS=24
```

### Usage Example

```rust
use crate::services::currency::{parse_price_with_currency, Currency};
use rust_decimal::Decimal;

// Parse price with automatic currency detection
let (amount, currency) = parse_price_with_currency("£999.99", Some("amazon.co.uk"))?;

// Convert using CurrencyService
let currency_service = &state.currency_service;
let usd_amount = currency_service.convert_to_usd(amount, &currency).await?;

println!("Original: {}{}", currency.symbol(), amount);
println!("USD equivalent: ${}", usd_amount);
```

### Database Schema

Prices are stored with full currency information:

```sql
price_original DECIMAL(12, 2)  -- Original price (e.g., £999.99)
currency VARCHAR(3)             -- ISO 4217 code (GBP)
price_usd DECIMAL(12, 2)        -- Normalized USD price for comparison
exchange_rate DECIMAL(10, 6)    -- Rate used for conversion
```

### Exchange Rate API

By default, uses the free [ExchangeRate-API](https://exchangerate-api.com/):
- No API key required for basic tier
- 1,500 requests/month free
- Updated daily

For higher frequency updates, consider alternatives:
- [Open Exchange Rates](https://openexchangerates.org/) - Hourly updates
- [Fixer.io](https://fixer.io/) - Real-time rates

## Development

### Code Quality

```bash
# Format code
cargo fmt

# Lint with Clippy
cargo clippy -- -D warnings

# Check without building
cargo check
```
