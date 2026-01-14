use sqlx::PgPool;

#[derive(sqlx::FromRow, Debug)]
pub struct ArchiveTag {
    pub tag_id: i32,
    pub tag_name: String,
}
impl ArchiveTag {
    pub async fn create(pool: &PgPool, tag_name: String) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO archive_tags(tag_name)
            VALUES ($1)
            "#,
            tag_name
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn get_id_from_name(pool: &PgPool, name: String) -> Result<Option<i32>, sqlx::Error> {
        let out = sqlx::query_as!(ArchiveTag, r#"
            SELECT * from archive_tags
            WHERE tag_name = $1
            "#,
            &name
        )
        .fetch_optional(pool)
        .await?
        .map(|x| x.tag_id);

        Ok(out)
    }

    pub async fn remove(&self, pool: &PgPool) -> Result<(), sqlx::Error> {
        // When we remove an entry here, all references to it in archive_item_tags
        // are auto-removed since we declared that table with the references as ON
        // DELETE CASCADE.

        sqlx::query!(
            r#"
            DELETE FROM archive_tags
            WHERE tag_id = $1
            "#,
            self.tag_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}