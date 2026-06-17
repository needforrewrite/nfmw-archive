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

    pub async fn get_tags_for_asset_id(pool: &sqlx::PgPool, asset_id: i64) -> Result<Vec<Self>, sqlx::Error> {
        let tags = sqlx::query_as!(
            Tag,
            r#"SELECT t.id, t.name, t.description, t.required_role_id, t.created_at
                FROM tags t
                INNER JOIN asset_tags at ON at.tag_id = t.id
                WHERE at.asset_id = $1"#,
            asset_id
        )
        .fetch_all(pool)
        .await?;

        Ok(tags)
    }
}