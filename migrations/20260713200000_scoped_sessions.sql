-- Scope session tokens to an audience.
--
-- 'archive' is a token that acts on this service directly (what every existing
-- session is). Any other audience is a service id from config.toml — a
-- short-lived, single-use key the player hands to that service, which redeems
-- it via /auth/service-key/validate. A service key is useless anywhere but the
-- audience it names, so handing one to a lobby grants the lobby nothing here.
--
-- Service keys are children of the archive session that minted them: the
-- self-referencing FK cascades, so logging out or logging in again destroys
-- every key derived from that session.

ALTER TABLE user_sessions DROP CONSTRAINT user_sessions_pkey;

-- Existing sessions are all archive-scoped; the default backfills them.
ALTER TABLE user_sessions
    ADD COLUMN audience          TEXT NOT NULL DEFAULT 'archive',
    ADD COLUMN parent_token_hash TEXT;

ALTER TABLE user_sessions ALTER COLUMN audience DROP DEFAULT;

ALTER TABLE user_sessions ADD PRIMARY KEY (token_hash);

-- Named for the pre-rename `token` column, superseded by the primary key above.
ALTER TABLE user_sessions DROP CONSTRAINT IF EXISTS user_sessions_token_key;

ALTER TABLE user_sessions
    ADD CONSTRAINT user_sessions_parent_token_hash_fkey
    FOREIGN KEY (parent_token_hash) REFERENCES user_sessions(token_hash) ON DELETE CASCADE;

-- Still one archive session per user, as the old user_id primary key enforced.
-- Service keys are deliberately unconstrained in count.
CREATE UNIQUE INDEX user_sessions_one_archive_per_user
    ON user_sessions (user_id) WHERE audience = 'archive';

CREATE INDEX user_sessions_user_id_idx ON user_sessions (user_id);
CREATE INDEX user_sessions_parent_token_hash_idx ON user_sessions (parent_token_hash);

-- Only archive sessions get a sliding expiry. Service keys are single-use and
-- live for minutes; extending them on use would defeat both properties.
CREATE OR REPLACE FUNCTION touch_session(p_token TEXT, p_window INTERVAL DEFAULT '30 days')
RETURNS VOID LANGUAGE plpgsql AS $$
BEGIN
    UPDATE user_sessions
       SET last_seen_at = now(),
           expires_at   = now() + p_window
     WHERE token_hash = p_token
       AND audience = 'archive'
       AND expires_at > now();
END;
$$;
