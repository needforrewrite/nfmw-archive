-- =============================================================================
-- Racing Game Asset & Account Server — PostgreSQL Schema
-- Designed for use with sqlx (Rust)
-- =============================================================================


-- =============================================================================
-- SECTION 1: ROLES
-- =============================================================================

-- The authoritative set of roles that exist in the system.
-- Roles are independent — no hierarchy or inheritance between them.
-- Examples: 'admin', 'moderator', 'verified_creator', etc.
CREATE TABLE roles (
    id                  SERIAL      PRIMARY KEY,
    name                TEXT        NOT NULL UNIQUE,
    description         TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Many-to-many: which roles a user currently holds.
-- Role assignment/revocation is managed by application logic (e.g. by admins).
CREATE TABLE user_roles (
    user_id             BIGINT      NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id             INTEGER     NOT NULL REFERENCES roles(id) ON DELETE RESTRICT,
    -- Who granted this role, and when.
    granted_by          BIGINT      REFERENCES users(id) ON DELETE SET NULL,
    granted_at          TIMESTAMPTZ NOT NULL DEFAULT now(),

    PRIMARY KEY (user_id, role_id)
);


-- =============================================================================
-- SECTION 2: USERS
-- =============================================================================

-- Records every username that has ever been actively held by any user,
-- and how many distinct users have held it. This is consulted when a new
-- user claims a username to determine whether a suffix is needed.
--
-- Lifecycle:
--   - On first claim of "james": INSERT (username='james', claim_count=1)
--   - User "james" renames to "bob": the row stays (history is permanent).
--   - A new user claims "james": claim_count becomes 2, their assets are
--     authored as "james-1".
--   - That user renames away; yet another user claims "james": count → 3,
--     authored as "james-2". And so on.
CREATE TABLE username_claim_history (
    username            TEXT        NOT NULL,

    -- How many *different* users have ever held this username.
    -- The suffix for the Nth claimer is (N - 1), so:
    --   claim_count=1 → suffix 0 → plain "username"
    --   claim_count=2 → suffix 1 → "username-1"
    claim_count         INTEGER     NOT NULL DEFAULT 1 CHECK (claim_count >= 1),

    PRIMARY KEY (username)
);

-- Core user table. Supports both local and OAuth2 accounts.
CREATE TABLE users (
    id                  BIGSERIAL   PRIMARY KEY,

    -- The user's current, live username. Mutable.
    username            TEXT        NOT NULL UNIQUE,

    -- Separate display name (mutable, no uniqueness requirement).
    display_name        TEXT,

    email               TEXT        UNIQUE,

    -- The suffix this user inherited when they claimed their *current* username.
    -- 0  → no suffix  → new assets are authored as "username"
    -- N  → suffix N   → new assets are authored as "username-N"
    --
    -- Updated each time the user changes their username:
    -- re-read username_claim_history for the new name and store (claim_count - 1).
    current_author_suffix   INTEGER NOT NULL DEFAULT 0 CHECK (current_author_suffix >= 0),

    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Local (password-based) credentials. At most one row per user.
CREATE TABLE local_credentials (
    user_id             BIGINT      PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    -- Store a bcrypt / argon2 hash; never plaintext.
    password_hash       TEXT        NOT NULL,
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- OAuth2 identities. A single user may link multiple providers.
CREATE TABLE oauth_identities (
    id                  BIGSERIAL   PRIMARY KEY,
    user_id             BIGINT      NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- e.g. "google", "github", "discord"
    provider            TEXT        NOT NULL,
    -- Stable subject identifier from the provider's token / userinfo endpoint.
    provider_subject    TEXT        NOT NULL,

    -- Store tokens only if you need to make API calls on the user's behalf.
    access_token        TEXT,
    refresh_token       TEXT,
    token_expires_at    TIMESTAMPTZ,

    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE (provider, provider_subject)
);


-- =============================================================================
-- SECTION 3: TAGS
-- =============================================================================

-- The pre-approved list of tags users may apply to assets.
CREATE TABLE tags (
    id                  SERIAL      PRIMARY KEY,
    name                TEXT        NOT NULL UNIQUE,
    description         TEXT,

    -- NULL → any authenticated user may apply this tag.
    -- Non-null → the applying user must hold this role.
    -- Enforced in application logic by checking user_roles.
    required_role_id    INTEGER     REFERENCES roles(id) ON DELETE RESTRICT,

    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);


-- =============================================================================
-- SECTION 4: ASSETS
-- =============================================================================

-- Asset type enum — extend as the game grows.
CREATE TYPE asset_type AS ENUM (
    'track',
    'track_piece',
    'car',
    'wheel',
    'texture',
    'music',
    'script',
    -- Details of this not worked out yet.
    -- Will likely be a separate "campaign" asset type with its own table for campaign metadata.
    'campaign'
);

-- Every user-uploaded asset.
--
-- Author name snapshotting
-- ─────────────────────────
-- author_name is copied from the user's *current* username (with suffix if
-- applicable) at the moment the asset is created. It is immutable thereafter.
-- This means:
--   - While "james" is "james", new assets get author_name = "james".
--   - If "james" renames to "alice", new assets get author_name = "alice"
--     (or "alice-N" if alice was previously claimed).
--   - Existing assets keep whatever author_name they had at creation.
--
-- Dependency resolution is handled by the monolithic archive; the
-- asset_dependencies table is informational / for display only.
CREATE TABLE assets (
    id                  BIGSERIAL   PRIMARY KEY,

    -- Owning user. Ownership follows the user even through renames.
    owner_id            BIGINT      NOT NULL REFERENCES users(id) ON DELETE RESTRICT,

    -- Snapshot of the owner's effective author name at creation time.
    -- Computed in application logic as:
    --   current_author_suffix = 0 → users.username
    --   current_author_suffix = N → users.username || '-' || N
    -- Never updated after insert.
    author_name         TEXT        NOT NULL,

    -- Immutable after creation. Together with author_name forms the
    -- stable client-facing reference "author_name/asset_name".
    asset_name          TEXT        NOT NULL,

    display_name        TEXT        NOT NULL,
    description         TEXT,
    asset_type          asset_type  NOT NULL,

    -- Path / key to the monolithic archive in object storage (e.g. an S3 key).
    -- NULL for assets that exist only as declared dependencies and have no
    -- standalone archive.
    archive_path        TEXT,

    -- Visibility flag for soft-delete / unlisted assets.
    is_public           BOOLEAN     NOT NULL DEFAULT true,

    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- The client-facing "author_name/asset_name" reference must be globally unique.
    -- Because both halves are immutable after insert, this constraint is stable.
    UNIQUE (author_name, asset_name)
);

-- Declared dependency graph (informational; not used for archive serving).
-- Row (dependent_id=A, dependency_id=B) means asset A declares a dependency on B.
CREATE TABLE asset_dependencies (
    dependent_id        BIGINT      NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    dependency_id       BIGINT      NOT NULL REFERENCES assets(id) ON DELETE RESTRICT,

    PRIMARY KEY (dependent_id, dependency_id),

    -- Prevent trivial self-loops.
    CHECK (dependent_id <> dependency_id)
);

-- Junction table: tags applied to assets. Maximum 5 tags per asset.
-- Tag permission checks (required_role_id) are enforced in application logic.
CREATE TABLE asset_tags (
    asset_id            BIGINT      NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    tag_id              INTEGER     NOT NULL REFERENCES tags(id)   ON DELETE RESTRICT,

    applied_by          BIGINT      REFERENCES users(id) ON DELETE SET NULL,
    applied_at          TIMESTAMPTZ NOT NULL DEFAULT now(),

    PRIMARY KEY (asset_id, tag_id)
);

-- Trigger: reject an insert that would push an asset past 5 tags.
CREATE OR REPLACE FUNCTION check_asset_tag_limit()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    IF (SELECT COUNT(*) FROM asset_tags WHERE asset_id = NEW.asset_id) >= 5 THEN
        RAISE EXCEPTION 'Asset % already has the maximum of 5 tags.', NEW.asset_id;
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER enforce_asset_tag_limit
    BEFORE INSERT ON asset_tags
    FOR EACH ROW EXECUTE FUNCTION check_asset_tag_limit();


-- =============================================================================
-- SECTION 5: INDEXES
-- =============================================================================

-- Role lookups
CREATE INDEX idx_user_roles_user_id         ON user_roles(user_id);
CREATE INDEX idx_user_roles_role_id         ON user_roles(role_id);

-- User lookups
CREATE INDEX idx_users_username             ON users(username);
CREATE INDEX idx_oauth_user_id              ON oauth_identities(user_id);

-- Asset lookups
CREATE INDEX idx_assets_owner               ON assets(owner_id);
CREATE INDEX idx_assets_author_name         ON assets(author_name);
CREATE INDEX idx_assets_type                ON assets(asset_type);
CREATE INDEX idx_assets_created_at          ON assets(created_at DESC);
CREATE INDEX idx_assets_updated_at          ON assets(updated_at DESC);

-- Tag lookups
CREATE INDEX idx_tags_required_role         ON tags(required_role_id);
CREATE INDEX idx_asset_tags_tag_id          ON asset_tags(tag_id);
CREATE INDEX idx_asset_tags_asset_id        ON asset_tags(asset_id);

-- Dependency graph traversal
CREATE INDEX idx_deps_dependency_id         ON asset_dependencies(dependency_id);


-- =============================================================================
-- SECTION 6: UPDATED_AT TRIGGERS
-- =============================================================================

CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$;

CREATE TRIGGER trg_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER trg_local_creds_updated_at
    BEFORE UPDATE ON local_credentials
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER trg_oauth_updated_at
    BEFORE UPDATE ON oauth_identities
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER trg_assets_updated_at
    BEFORE UPDATE ON assets
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();