use sqlx::{PgPool, types::Uuid};

#[derive(sqlx::FromRow, Debug)]
pub struct ArchiveItemTag {
    pub archive_item_id: Uuid,
    pub tag_id: i32,
}
impl ArchiveItemTag {
    pub async fn insert(&self, pool: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO archive_item_tags (archive_item_id, tag_id)
            VALUES ($1, $2)
            "#,
            self.archive_item_id,
            self.tag_id,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn remove(&self, pool: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            DELETE FROM archive_item_tags
            WHERE archive_item_id = $1 AND tag_id = $2
            "#,
            self.archive_item_id,
            self.tag_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}