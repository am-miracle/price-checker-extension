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

## Observability and Monitoring

This project includes a comprehensive observability stack with Prometheus metrics, structured logging, and Grafana dashboards.

### Quick Start

```bash
# Start full stack with observability
docker-compose up -d

# Access dashboards
# Application: http://localhost:8080
# Prometheus: http://localhost:9090
# Grafana: http://localhost:3000 (admin/admin)
# Metrics: http://localhost:8080/metrics
```

### Key Metrics

- **HTTP Request Rate & Duration**: Track API performance
- **Price Comparisons**: Monitor comparison requests and cache hit rates
- **Scraper Activity**: Per-site scraping success rates and duration
- **Database Performance**: Query latencies and connection pool stats
- **Cache Operations**: Redis hit/miss rates
- **Currency Conversions**: Track conversion activity

For detailed information, see [OBSERVABILITY.md](./OBSERVABILITY.md)

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
