-- Add migration script here

ALTER TABLE assets ADD COLUMN downloads BIGINT NOT NULL DEFAULT 0;
ALTER TABLE assets ADD COLUMN total_likes BIGINT NOT NULL DEFAULT 0;

CREATE TABLE asset_likes (
    id                  BIGSERIAL   PRIMARY KEY,

    asset_id            BIGINT      NOT NULL REFERENCES assets(id) ON DELETE CASCADE,

    liked_by_user_id    BIGINT      NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE(asset_id, liked_by_user_id)
);

CREATE INDEX idx_asset_likes_asset_id            ON asset_likes(asset_id);
CREATE INDEX idx_asset_likes_liked_by_user_id    ON asset_likes(liked_by_user_id);

CREATE TABLE asset_features (
    id                  BIGSERIAL   PRIMARY KEY,

    asset_id            BIGINT      NOT NULL REFERENCES assets(id) ON DELETE CASCADE,

    featured_by_user_id BIGINT      NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),

    expires_at          TIMESTAMPTZ NOT NULL DEFAULT (now() + INTERVAL '30 days'),

    UNIQUE(asset_id, featured_by_user_id)
);

CREATE INDEX idx_asset_features_asset_id            ON asset_features(asset_id);
CREATE INDEX idx_asset_features_featured_by_user_id ON asset_features(featured_by_user_id);

CREATE TABLE user_stats (
    user_id             BIGINT      PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,

    total_likes         BIGINT      NOT NULL,

    total_downloads     BIGINT      NOT NULL,

    total_features      BIGINT      NOT NULL
);

CREATE INDEX idx_user_stats_user_id ON user_stats(user_id);