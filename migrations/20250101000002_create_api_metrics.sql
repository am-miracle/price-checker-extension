-- Create api_metrics table to track API usage and performance
CREATE TABLE IF NOT EXISTS api_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    endpoint VARCHAR(100) NOT NULL,
    search_query VARCHAR(255),
    response_time_ms INTEGER NOT NULL,
    status_code INTEGER NOT NULL,
    results_count INTEGER DEFAULT 0,
    error_message TEXT,
    requested_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),

    CONSTRAINT response_time_positive CHECK (response_time_ms >= 0),
    CONSTRAINT status_code_valid CHECK (status_code >= 100 AND status_code < 600)
);

CREATE INDEX idx_api_metrics_endpoint ON api_metrics(endpoint);
CREATE INDEX idx_api_metrics_requested_at ON api_metrics(requested_at DESC);
CREATE INDEX idx_api_metrics_status_code ON api_metrics(status_code);

-- Add comments for documentation
COMMENT ON TABLE api_metrics IS 'Tracks API performance metrics and usage statistics';
COMMENT ON COLUMN api_metrics.endpoint IS 'API endpoint path (e.g., /api/compare)';
COMMENT ON COLUMN api_metrics.response_time_ms IS 'Total response time in milliseconds';
COMMENT ON COLUMN api_metrics.results_count IS 'Number of results returned in the response';
