use sqlx::PgPool;
use time::OffsetDateTime;

pub struct AssetLike {
    pub id: i64,
    pub asset_id: i64,
    pub liked_by_user_id: i64,
    pub created_at: OffsetDateTime
}
impl AssetLike {
    pub async fn user_has_liked_asset(pool: &PgPool, asset_id: i64, user_id: i64) -> Result<bool, sqlx::Error> {
        let exists = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM asset_likes WHERE asset_id = $1 AND liked_by_user_id = $2)",
            asset_id,
            user_id
        )
        .fetch_one(pool)
        .await?;

        Ok(exists.unwrap_or(false))
    }

    pub async fn add_like_to_asset_from_user(pool: &PgPool, asset_id: i64, user_id: i64) -> Result<Self, sqlx::Error> {
        let row = sqlx::query_as!(
            AssetLike,
            r#"INSERT INTO asset_likes(asset_id, liked_by_user_id)
                VALUES($1, $2)
                RETURNING *"#,
            asset_id,
            user_id
        )
        .fetch_one(pool)
        .await?;

        Ok(row)
    }

    pub async fn remove_like_from_asset_from_user(pool: &PgPool, asset_id: i64, user_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query_scalar!(
            r#"DELETE FROM asset_likes
                WHERE asset_id = $1
                AND liked_by_user_id = $2"#,
            asset_id,
            user_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}