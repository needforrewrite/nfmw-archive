-- Add migration script here

ALTER TABLE user_sessions RENAME COLUMN token TO token_hash;

CREATE OR REPLACE FUNCTION touch_session(p_token_hash TEXT, p_window INTERVAL DEFAULT '30 days')
RETURNS VOID LANGUAGE plpgsql AS $$
BEGIN
    UPDATE user_sessions
       SET last_seen_at = now(),
           expires_at   = now() + p_window
     WHERE token_hash = p_token_hash
       AND expires_at > now();
END;
$$;