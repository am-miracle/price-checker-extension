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
