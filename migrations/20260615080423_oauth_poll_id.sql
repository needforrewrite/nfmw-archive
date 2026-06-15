-- Add migration script here

CREATE EXTENSION IF NOT EXISTS pgcrypto;

ALTER TABLE oauth_sessions
    ADD COLUMN poll_id TEXT NOT NULL DEFAULT encode(gen_random_bytes(64), 'hex');