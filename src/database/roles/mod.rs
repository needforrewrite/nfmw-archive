use sqlx::{PgPool, types::time::OffsetDateTime};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Role {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserRole {
    pub user_id: i64,
    pub role_id: i32,
    pub granted_by: Option<i64>,
    pub granted_at: OffsetDateTime,
}
impl UserRole {
    pub async fn get_roles_for_user_id(pool: &PgPool, user_id: i64) -> Result<Vec<Self>, sqlx::Error> {
        let roles = sqlx::query_as!(
            UserRole,
            r#"SELECT * FROM user_roles
                WHERE user_id = $1"#,
            &user_id
        )
        .fetch_all(pool)
        .await?;

        Ok(roles)
    }
}