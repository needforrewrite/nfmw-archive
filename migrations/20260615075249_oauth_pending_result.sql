-- Add migration script here

ALTER TABLE oauth_sessions
    ADD COLUMN result_status TEXT,
    ADD COLUMN result_payload TEXT;