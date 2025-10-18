-- Add currency tracking fields to price_history table
-- This migration adds support for multi-currency prices with USD normalization

-- Rename existing price column to price_original
ALTER TABLE price_history
RENAME COLUMN price TO price_original;

-- Add new currency tracking columns
ALTER TABLE price_history
ADD COLUMN IF NOT EXISTS price_usd DECIMAL(12, 2),
ADD COLUMN IF NOT EXISTS exchange_rate DECIMAL(10, 6);

-- Update existing records to have USD values
-- (Assumes existing data is in USD or the currency column already indicates the currency)
UPDATE price_history
SET price_usd = price_original
WHERE price_usd IS NULL;

-- Make price_usd NOT NULL after backfilling
ALTER TABLE price_history
ALTER COLUMN price_usd SET NOT NULL;

-- Update currency column to NOT NULL with proper default
ALTER TABLE price_history
ALTER COLUMN currency SET NOT NULL;

-- Add constraints for new fields
ALTER TABLE price_history
ADD CONSTRAINT price_usd_positive CHECK (price_usd >= 0),
ADD CONSTRAINT exchange_rate_positive CHECK (exchange_rate IS NULL OR exchange_rate > 0);

-- Create indexes for the new columns
CREATE INDEX IF NOT EXISTS idx_price_history_currency ON price_history(currency);
CREATE INDEX IF NOT EXISTS idx_price_history_price_usd ON price_history(price_usd);

-- Add comments for new columns
COMMENT ON COLUMN price_history.price_original IS 'Price in original currency from the site';
COMMENT ON COLUMN price_history.currency IS 'ISO 4217 currency code (USD, EUR, GBP, NGN, etc.)';
COMMENT ON COLUMN price_history.price_usd IS 'Price converted to USD for comparison';
COMMENT ON COLUMN price_history.exchange_rate IS 'Exchange rate used for conversion (from original to USD)';
