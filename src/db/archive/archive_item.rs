use sqlx::{PgPool, types::{Uuid, time::PrimitiveDateTime}};

#[derive(sqlx::FromRow, Debug)]
pub struct ArchiveItem {
    // primary key
    pub archive_item_id: Uuid,
    pub path: String,
    pub r#type: String,
    pub name: String,
    pub created_at: Option<PrimitiveDateTime>,
    pub author: Option<String>,
    pub owner_user_id: i32
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
            INSERT INTO archive_items (archive_item_id, path, type, name, author, owner_user_id)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            self.archive_item_id,
            self.path,
            self.r#type,
            self.name,
            self.author,
            self.owner_user_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
