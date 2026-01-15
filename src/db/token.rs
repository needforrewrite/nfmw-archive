use sqlx::PgPool;

// A user can only have one active token at a time. Issuing a new token
// invalidates any previous token.
#[derive(sqlx::FromRow, Debug)]
pub struct UserToken {
    pub token_id: i32,
    pub user_id: i32,
    pub token: String,
}
impl UserToken {
    pub async fn insert(pool: &sqlx::PgPool, user_id: i32, token: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO user_tokens (user_id, token)
            VALUES ($1, $2)
            "#,
            user_id,
            token
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn get_user_by_token(pool: &PgPool, token: String) -> Result<Option<UserToken>, sqlx::Error> {
        let entry = sqlx::query_as!(Self, r#"
            SELECT * FROM user_tokens
            WHERE token = $1
            LIMIT 1
            "#,
            token
        )
        .fetch_optional(pool)
        .await?;

        Ok(entry)
    }

    pub async fn remove_all(user_id: i32, pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            DELETE FROM user_tokens WHERE user_id = $1
            "#,
            user_id
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}