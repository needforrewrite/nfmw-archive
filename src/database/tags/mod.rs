use sqlx::types::time::OffsetDateTime;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Tag {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub required_role_id: Option<i32>,
    pub created_at: OffsetDateTime,
}
impl Tag {
    pub fn is_privileged(&self) -> bool {
        self.required_role_id.is_some()
    }

    pub async fn get_from_name(pool: &sqlx::PgPool, name: &str) -> Result<Option<Self>, sqlx::Error> {
        let tag = sqlx::query_as!(
            Tag,
            r#"
            SELECT id, name, description, required_role_id, created_at
            FROM tags
            WHERE name = $1
            "#,
            name
        ).fetch_optional(pool).await?;

        Ok(tag)
    }
}