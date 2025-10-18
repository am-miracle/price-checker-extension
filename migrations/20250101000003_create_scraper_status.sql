-- Create scraper_status table to monitor scraper health and failures
CREATE TABLE IF NOT EXISTS scraper_status (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    site VARCHAR(50) NOT NULL,
    status VARCHAR(20) NOT NULL, -- 'success', 'failure', 'timeout', 'blocked'
    search_query VARCHAR(255),
    error_type VARCHAR(50), -- 'network', 'parse', 'missing_field', 'rate_limit'
    error_message TEXT,
    response_time_ms INTEGER,
    checked_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),

    CONSTRAINT status_valid CHECK (status IN ('success', 'failure', 'timeout', 'blocked', 'rate_limited'))
);

CREATE INDEX idx_scraper_status_site ON scraper_status(site);
CREATE INDEX idx_scraper_status_checked_at ON scraper_status(checked_at DESC);
CREATE INDEX idx_scraper_status_site_status ON scraper_status(site, status, checked_at DESC);

-- Add comments for documentation
COMMENT ON TABLE scraper_status IS 'Monitors health and failure patterns of individual scrapers';
COMMENT ON COLUMN scraper_status.status IS 'Outcome of the scraping attempt';
COMMENT ON COLUMN scraper_status.error_type IS 'Category of error for failed attempts';
