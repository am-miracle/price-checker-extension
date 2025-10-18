-- Create price_history table to track historical price data
CREATE TABLE IF NOT EXISTS price_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    site VARCHAR(50) NOT NULL,
    product_title VARCHAR(500) NOT NULL,

    -- Store both original price and USD equivalent
    price_original DECIMAL(12, 2) NOT NULL,
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',
    price_usd DECIMAL(12, 2) NOT NULL,
    exchange_rate DECIMAL(10, 6),

    product_link TEXT NOT NULL,
    image_url TEXT,
    search_query VARCHAR(255) NOT NULL,
    scraped_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),

    -- Constraints
    CONSTRAINT price_original_positive CHECK (price_original >= 0),
    CONSTRAINT price_usd_positive CHECK (price_usd >= 0),
    CONSTRAINT exchange_rate_positive CHECK (exchange_rate IS NULL OR exchange_rate > 0)
);

CREATE INDEX idx_price_history_site ON price_history(site);
CREATE INDEX idx_price_history_search_query ON price_history(search_query);
CREATE INDEX idx_price_history_scraped_at ON price_history(scraped_at DESC);
CREATE INDEX idx_price_history_query_site ON price_history(search_query, site, scraped_at DESC);
CREATE INDEX idx_price_history_currency ON price_history(currency);
CREATE INDEX idx_price_history_price_usd ON price_history(price_usd);

-- Add comments for documentation
COMMENT ON TABLE price_history IS 'Stores historical price data scraped from various e-commerce platforms';
COMMENT ON COLUMN price_history.site IS 'E-commerce platform name (e.g., Amazon, eBay, Jumia)';
COMMENT ON COLUMN price_history.search_query IS 'Original user search query that generated this result';
COMMENT ON COLUMN price_history.scraped_at IS 'Timestamp when the price was scraped';
COMMENT ON COLUMN price_history.price_original IS 'Price in original currency from the site';
COMMENT ON COLUMN price_history.currency IS 'ISO 4217 currency code (USD, EUR, GBP, NGN, etc.)';
COMMENT ON COLUMN price_history.price_usd IS 'Price converted to USD for comparison';
COMMENT ON COLUMN price_history.exchange_rate IS 'Exchange rate used for conversion (from original to USD)';
