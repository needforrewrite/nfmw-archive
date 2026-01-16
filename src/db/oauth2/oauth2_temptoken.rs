use sqlx::{PgPool, types::time::PrimitiveDateTime};


/// This is used if a user calls into discord login but the account does not exist.
/// It prevents re-authenticating with Discord - instead of supplying a new code and
/// redirect_uri, it can provide a session_token and the service can retrieve the Discord
/// token from here.
#[derive(sqlx::FromRow)]
pub struct DiscordOauth2TokenMapping {
    pub session_token: String,
    pub discord_token: String,
    pub created_at: Option<PrimitiveDateTime>,
    pub discord_user_id: i64
}
impl DiscordOauth2TokenMapping {
    pub async fn create(pool: &PgPool, session_token: String, discord_token: String, discord_user_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO discord_oauth2_token_storage (session_token, discord_token, discord_user_id)
            VALUES ($1, $2, $3)
            "#,
            session_token,
            discord_token,
            discord_user_id
        ).execute(pool)
        .await?;

        Ok(())
    }

    pub async fn get_from_session_token(pool: &PgPool, session_token: String) -> Result<Option<Self>, sqlx::Error> {
        let res = sqlx::query_as!(Self, 
            r#"
            SELECT * FROM discord_oauth2_token_storage
            WHERE session_token = $1
            "#,
            session_token
        ).fetch_optional(pool)
        .await?;

        Ok(res)
    }

    pub async fn clear_for(pool: &PgPool, discord_user_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            DELETE FROM discord_oauth2_token_storage
            WHERE discord_user_id = $1
            "#,
            discord_user_id
        ).execute(pool)
        .await?;

        Ok(())
    }
}