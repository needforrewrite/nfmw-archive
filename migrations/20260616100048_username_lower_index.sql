-- Add migration script here

CREATE INDEX IF NOT EXISTS idx_username_lower ON users (LOWER(username));