use sqlx::types::time::OffsetDateTime;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PendingOauthRegistration {
    pub id: i64,
    pub provider: String,
    pub provider_subject: String,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub token_expires_at: Option<OffsetDateTime>,
    pub email: Option<String>,
    pub temp_token: String,
    pub created_at: OffsetDateTime,
    pub expires_at: OffsetDateTime,
}
impl PendingOauthRegistration {
    pub async fn insert(
        pool: &sqlx::PgPool,
        provider: &str,
        provider_subject: &str,
        access_token: Option<&str>,
        refresh_token: Option<&str>,
        token_expires_at: Option<OffsetDateTime>,
        email: Option<&str>,
        temp_token: &str,
    ) -> Result<Self, sqlx::Error> {
        let registration = sqlx::query_as!(
            PendingOauthRegistration,
            r#"
            INSERT INTO pending_oauth_registrations (provider, provider_subject, access_token, refresh_token, token_expires_at, email, temp_token)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
            provider,
            provider_subject,
            access_token,
            refresh_token,
            token_expires_at,
            email,
            temp_token
        )
        .fetch_one(pool)
        .await?;
        Ok(registration)
    }

    pub async fn get_by_temp_token(
        pool: &sqlx::PgPool,
        temp_token: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        let registration = sqlx::query_as!(
            PendingOauthRegistration,
            r#"
            SELECT * FROM pending_oauth_registrations
            WHERE temp_token = $1
            "#,
            temp_token
        )
        .fetch_optional(pool)
        .await?;
        Ok(registration)
    }

    pub async fn delete(&self, pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"DELETE FROM pending_oauth_registrations WHERE id = $1"#,
            self.id
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}