use sqlx::types::time::OffsetDateTime;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct OauthSession {
    // Serial primary key
    pub id: i64,
    pub provider: String,
    pub state: String,
    // only some providers require PKCE, so this is optional
    // some support but dont require, like Discord. don't use it then
    pub pkce_verifier: Option<String>,
    // bsky only
    pub par_request_uri: Option<String>,
    // bsky only
    pub dpop_private_key_jwk: Option<String>,
    pub created_at: OffsetDateTime,
    pub expires_at: OffsetDateTime,
}
impl OauthSession {
    pub async fn insert_base(
        pool: &sqlx::PgPool,
        provider: &str,
        state: &str,
    ) -> Result<Self, sqlx::Error> {
        Self::insert_full(pool, provider, state, None, None, None).await
    }

    pub async fn insert_with_pkce(
        pool: &sqlx::PgPool,
        provider: &str,
        state: &str,
        pkce_verifier: &str,
    ) -> Result<Self, sqlx::Error> {
        Self::insert_full(pool, provider, state, Some(pkce_verifier), None, None).await
    }

    pub async fn insert_full(
        pool: &sqlx::PgPool,
        provider: &str,
        state: &str,
        pkce_verifier: Option<&str>,
        par_request_uri: Option<&str>,
        dpop_private_key_jwk: Option<&str>,
    ) -> Result<Self, sqlx::Error> {
        let session = sqlx::query_as!(
            OauthSession,
            r#"
            INSERT INTO oauth_sessions (provider, state, pkce_verifier, par_request_uri, dpop_private_key_jwk)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
            provider,
            state,
            pkce_verifier,
            par_request_uri,
            dpop_private_key_jwk
        )
        .fetch_one(pool)
        .await?;
        Ok(session)
    }

    pub async fn get_by_state(pool: &sqlx::PgPool, state: &str) -> Result<Option<Self>, sqlx::Error> {
        let session = sqlx::query_as!(
            OauthSession,
            r#"
            SELECT * FROM oauth_sessions WHERE state = $1
            "#,
            state
        )
        .fetch_optional(pool)
        .await?;
        Ok(session)
    }

    pub async fn delete(&self, pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            DELETE FROM oauth_sessions WHERE id = $1
            "#,
            self.id
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}
