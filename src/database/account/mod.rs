pub mod oauth2;

use sqlx::types::time::OffsetDateTime;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: Option<String>,
    pub current_author_suffix: i32,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl User {
    pub async fn get_canonical_author_name_from_user_id(
        pool: &sqlx::PgPool,
        user_id: i64
    ) -> Result<Option<String>, sqlx::Error> {
        let user = Self::get_by_id(pool, user_id).await?;
        if let Some(user) = user {
            return Ok(Some(format!("{}-{}", user.username, user.current_author_suffix)))
        }

        return Ok(None)
    }

    pub async fn create(
        pool: &sqlx::PgPool,
        username: &str,
        email: Option<&str>,
    ) -> Result<Self, sqlx::Error> {
        let mut tx = pool.begin().await?;

        let claim_count: i32 = sqlx::query_scalar!(
            r#"
            INSERT INTO username_claim_history (username, claim_count)
            VALUES ($1, 1)
            ON CONFLICT (username) DO UPDATE
                SET claim_count = username_claim_history.claim_count + 1
            RETURNING claim_count
            "#,
            username.to_ascii_lowercase()
        )
        .fetch_one(&mut *tx)
        .await?;

        let user = sqlx::query_as!(
            User,
            r#"INSERT INTO users (username, current_author_suffix, email) VALUES ($1, $2, $3) RETURNING *"#,
            username,
            claim_count - 1,
            email,
        )
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(user)
    }

    pub async fn get_by_id(pool: &sqlx::PgPool, id: i64) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(User, r#"SELECT * FROM users WHERE id = $1"#, id)
            .fetch_optional(pool)
            .await
    }

    pub async fn get_by_username(pool: &sqlx::PgPool, username: &str) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(User, r#"SELECT * FROM users WHERE username = $1"#, username)
            .fetch_optional(pool)
            .await
    }

    pub async fn delete(&self, pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
        sqlx::query!(r#"DELETE FROM users WHERE id = $1"#, self.id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn username_taken(pool: &sqlx::PgPool, username: &str) -> Result<bool, sqlx::Error> {
        let exists = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM users WHERE LOWER(username) = LOWER($1))"#,
            username
        )
        .fetch_one(pool)
        .await?;

        Ok(exists.unwrap_or(false))
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct LocalCredentials {
    pub user_id: i64,
    pub password_hash: String,
    pub updated_at: OffsetDateTime,
}
impl LocalCredentials {
    pub async fn insert(
        pool: &sqlx::PgPool,
        user_id: i64,
        password_hash: &str,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            LocalCredentials,
            r#"INSERT INTO local_credentials (user_id, password_hash) VALUES ($1, $2) RETURNING *"#,
            user_id,
            password_hash
        )
        .fetch_one(pool)
        .await
    }

    pub async fn get_by_user_id(pool: &sqlx::PgPool, user_id: i64) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(LocalCredentials, r#"SELECT * FROM local_credentials WHERE user_id = $1"#, user_id)
            .fetch_optional(pool)
            .await
    }
}

/// Audience of a session that acts on this server directly. Anything else is a
/// service id from config, and such a token is only ever redeemable by that
/// service — never here.
pub const ARCHIVE_AUDIENCE: &str = "archive";

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserSessions {
    pub user_id: i64,
    pub token_hash: String,
    pub audience: String,
    pub parent_token_hash: Option<String>,
    pub auth_provider: String,
    pub created_at: OffsetDateTime,
    pub expires_at: OffsetDateTime,
    pub last_seen_at: OffsetDateTime,
}
impl UserSessions {
    /// Logging in replaces the user's archive session outright. The delete
    /// cascades through parent_token_hash, so every service key minted from the
    /// old session dies with it.
    pub async fn create_archive_session(
        pool: &sqlx::PgPool,
        user_id: i64,
        token_hash: &str,
        auth_provider: &str,
    ) -> Result<Self, sqlx::Error> {
        let mut tx = pool.begin().await?;

        sqlx::query!(
            r#"DELETE FROM user_sessions WHERE user_id = $1 AND audience = $2"#,
            user_id,
            ARCHIVE_AUDIENCE,
        )
        .execute(&mut *tx)
        .await?;

        let session = sqlx::query_as!(
            UserSessions,
            r#"
            INSERT INTO user_sessions (user_id, token_hash, audience, auth_provider)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
            user_id,
            token_hash,
            ARCHIVE_AUDIENCE,
            auth_provider,
        )
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(session)
    }

    /// Mints a key scoped to `audience`, descended from the archive session that
    /// asked for it. Short-lived by design: it is handed to a third party, and
    /// [`Self::consume_service_key`] destroys it on first use.
    pub async fn create_service_key(
        pool: &sqlx::PgPool,
        parent: &UserSessions,
        token_hash: &str,
        audience: &str,
        ttl_seconds: i64,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            UserSessions,
            r#"
            INSERT INTO user_sessions
                (user_id, token_hash, audience, parent_token_hash, auth_provider, expires_at)
            VALUES ($1, $2, $3, $4, $5, now() + make_interval(secs => $6))
            RETURNING *
            "#,
            parent.user_id,
            token_hash,
            audience,
            parent.token_hash,
            parent.auth_provider,
            ttl_seconds as f64,
        )
        .fetch_one(pool)
        .await
    }

    /// Redeems a service key for the session behind it, atomically destroying it
    /// so a second redemption of the same key finds nothing. Matching on
    /// `audience` is what stops one service redeeming a key minted for another.
    pub async fn consume_service_key(
        pool: &sqlx::PgPool,
        token_hash: &str,
        audience: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            UserSessions,
            r#"
            DELETE FROM user_sessions
            WHERE token_hash = $1 AND audience = $2 AND expires_at > now()
            RETURNING *
            "#,
            token_hash,
            audience,
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn get_by_token_hash(pool: &sqlx::PgPool, token_hash: &str) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            UserSessions,
            r#"SELECT * FROM user_sessions WHERE token_hash = $1"#,
            token_hash
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn delete(&self, pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
        sqlx::query!(r#"DELETE FROM user_sessions WHERE token_hash = $1"#, self.token_hash)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn delete_expired(pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
        sqlx::query!(r#"DELETE FROM user_sessions WHERE expires_at < now()"#)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn touch_session(pool: &sqlx::PgPool, token_hash: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "SELECT touch_session($1)",
            token_hash,
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct OauthIdentity {
    pub id: i64,
    pub user_id: i64,
    pub provider: String,
    pub provider_subject: String,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub token_expires_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl OauthIdentity {
    pub async fn get_by_provider_subject(
        pool: &sqlx::PgPool,
        provider: &str,
        provider_subject: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            OauthIdentity,
            r#"SELECT * FROM oauth_identities WHERE provider = $1 AND provider_subject = $2"#,
            provider,
            provider_subject,
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn insert(
        pool: &sqlx::PgPool,
        user_id: i64,
        provider: &str,
        provider_subject: &str,
        access_token: Option<&str>,
        refresh_token: Option<&str>,
        token_expires_at: Option<OffsetDateTime>,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            OauthIdentity,
            r#"
            INSERT INTO oauth_identities
                (user_id, provider, provider_subject, access_token, refresh_token, token_expires_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
            user_id,
            provider,
            provider_subject,
            access_token,
            refresh_token,
            token_expires_at,
        )
        .fetch_one(pool)
        .await
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UsernameClaimHistory {
    pub username: String,
    pub claim_count: i32,
}
