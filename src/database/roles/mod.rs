use sqlx::types::time::OffsetDateTime;

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
