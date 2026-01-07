use sqlx::{PgPool, types::{Uuid, time::PrimitiveDateTime}};

#[derive(sqlx::FromRow, Debug)]
pub struct ArchiveItem {
    // primary key
    pub archive_item_id: Uuid,
    pub path: String,
    pub r#type: String,
    pub name: String,
    pub created_at: PrimitiveDateTime,
    pub author: Option<String>,
}
impl ArchiveItem {
    pub async fn search_name(
        &self,
        pool: &PgPool,
        query: String,
    ) -> Result<Vec<Self>, sqlx::Error> {
        todo!()
    }

    pub async fn insert(&self, pool: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO archive_items (archive_item_id, path, type, name, author)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            self.archive_item_id,
            self.path,
            self.r#type,
            self.name,
            self.author
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
