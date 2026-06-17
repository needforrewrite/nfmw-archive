-- Add migration script here

CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE INDEX IF NOT EXISTS idx_asset_display_name_lower ON assets (LOWER(display_name));
-- Sppeed up LIKE and ILIKE
CREATE INDEX IF NOT EXISTS idx_asset_display_name_trgm ON assets USING gin (display_name gin_trgm_ops);