use sqlx::types::time::OffsetDateTime;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Tag {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub required_role_id: Option<i32>,
    pub created_at: OffsetDateTime,
}
