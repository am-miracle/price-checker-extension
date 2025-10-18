-- Create price_history table to track historical price data
CREATE TABLE IF NOT EXISTS price_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    site VARCHAR(50) NOT NULL,
    product_title VARCHAR(500) NOT NULL,
    price DECIMAL(12, 2) NOT NULL,
    currency VARCHAR(3) DEFAULT 'USD',
    product_link TEXT NOT NULL,
    image_url TEXT,
    search_query VARCHAR(255) NOT NULL,
    scraped_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),

    -- Indexes for common queries
    CONSTRAINT price_positive CHECK (price >= 0)
);

CREATE INDEX idx_price_history_site ON price_history(site);
CREATE INDEX idx_price_history_search_query ON price_history(search_query);
CREATE INDEX idx_price_history_scraped_at ON price_history(scraped_at DESC);
CREATE INDEX idx_price_history_query_site ON price_history(search_query, site, scraped_at DESC);

-- Add comments for documentation
COMMENT ON TABLE price_history IS 'Stores historical price data scraped from various e-commerce platforms';
COMMENT ON COLUMN price_history.site IS 'E-commerce platform name (e.g., Amazon, eBay, Jumia)';
COMMENT ON COLUMN price_history.search_query IS 'Original user search query that generated this result';
COMMENT ON COLUMN price_history.scraped_at IS 'Timestamp when the price was scraped';
