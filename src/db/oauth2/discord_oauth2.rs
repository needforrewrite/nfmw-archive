use sqlx::{PgPool, types::time::PrimitiveDateTime};

#[derive(sqlx::FromRow)]
pub struct DiscordOauth2AccountEntry {
    pub entry_id: i32,
    pub user_id: i32,
    pub discord_user_id: i64,
    pub created_at: Option<PrimitiveDateTime>,
}
impl DiscordOauth2AccountEntry {
    pub async fn lookup_discord_user_id(pool: &PgPool, discord_user_id: i64) -> Result<Option<Self>, sqlx::Error> {
        let entry = sqlx::query_as!(Self, r#"
            SELECT * FROM discord_oauth2
            WHERE discord_user_id = $1
            LIMIT 1
            "#,
            discord_user_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(entry)
    }

    pub async fn create_oauth2_link(pool: &PgPool, user_id: i32, discord_user_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query!(r#"
            INSERT INTO discord_oauth2 (user_id, discord_user_id)
            VALUES ($1, $2)
            "#,
            user_id,
            discord_user_id
        )
        .execute(pool)
        .await?;
        
        Ok(())
    }
}