// A user can only have one active token at a time. Issuing a new token
// invalidates any previous token.
#[derive(sqlx::FromRow, Debug)]
pub struct UserToken {
    pub user_id: i32,
    pub token: String,
}
impl UserToken {
    pub fn new(user_id: i32, token: String) -> Self {
        UserToken {
            user_id,
            token,
        }
    }

    pub async fn insert(&self, pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO user_tokens (user_id, token)
            VALUES ($1, $2)
            "#,
        )
        .bind(self.user_id)
        .bind(&self.token)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn remove_all(user_id: i32, pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM user_tokens WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .execute(pool)
        .await?;
        Ok(())
    }
}