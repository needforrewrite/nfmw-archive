use sqlx::{PgPool, types::time::OffsetDateTime};

use crate::database::assets::AssetType;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Asset {
    pub id: i64,
    pub owner_id: i64,
    pub author_name: String,
    pub asset_name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub asset_type: AssetType,
    pub archive_path: String,
    pub is_public: bool,
    pub downloads: i64,
    pub total_likes: i64,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}
impl Asset {
    pub async fn insert(
        pool: &PgPool,
        owner_id: i64,
        author_name: &str,
        asset_name: &str,
        display_name: &str,
        description: Option<&str>,
        asset_type: AssetType,
        archive_path: &str,
        is_public: bool) -> Result<Self, sqlx::Error> {
            let asset = sqlx::query_as!(
                Asset,
                r#"INSERT INTO assets (owner_id, author_name, asset_name, display_name, description, asset_type, archive_path, is_public)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                 RETURNING id, owner_id, author_name, asset_name, display_name, description, asset_type as "asset_type: AssetType", archive_path, is_public, downloads, total_likes, created_at, updated_at"#,
                owner_id,
                author_name,
                asset_name,
                display_name,
                description,
                asset_type as AssetType,
                archive_path,
                is_public
            )
            .fetch_one(pool)
            .await?;

            Ok(asset)
        }

    pub async fn get_owner_id(pool: &PgPool, owner_id: i64, asset_type: AssetType, asset_name: &str) -> Result<Option<Self>, sqlx::Error> {
        let asset = sqlx::query_as!(
            Asset,
            r#"SELECT id, owner_id, author_name, asset_name, display_name, description, asset_type as "asset_type: AssetType", archive_path, is_public, downloads, total_likes, created_at, updated_at
                FROM assets
                WHERE owner_id = $1
                AND asset_type = $2
                AND asset_name = $3"#,
            owner_id,
            asset_type as AssetType,
            asset_name
        )
        .fetch_optional(pool)
        .await?;

        Ok(asset)
    }

    pub async fn get_author_name(pool: &PgPool, author_name: &str, asset_type: AssetType, asset_name: &str) -> Result<Option<Self>, sqlx::Error> {
        let asset = sqlx::query_as!(
            Asset,
            r#"SELECT id, owner_id, author_name, asset_name, display_name, description, asset_type as "asset_type: AssetType", archive_path, is_public, downloads, total_likes, created_at, updated_at
                FROM assets
                WHERE author_name = $1
                AND asset_type = $2
                AND asset_name = $3"#,
            author_name,
            asset_type as AssetType,
            asset_name
        )
        .fetch_optional(pool)
        .await?;

        Ok(asset)
    }

    pub async fn get_owned_by_id(pool: &PgPool, owner_id: i64) -> Result<Vec<Self>, sqlx::Error> {
        let assets = sqlx::query_as!(
            Asset,
            r#"SELECT id, owner_id, author_name, asset_name, display_name, description, asset_type as "asset_type: AssetType", archive_path, is_public, downloads, total_likes, created_at, updated_at
                FROM assets
                WHERE owner_id = $1"#,
            owner_id
        )
        .fetch_all(pool)
        .await?;

        Ok(assets)
    }

    pub async fn filter_by_display_name(pool: &PgPool, display_name: &str) -> Result<Vec<Self>, sqlx::Error> {
        let search = format!("%{}%", display_name);
        
        let assets = sqlx::query_as!(
            Asset,
            r#"SELECT id, owner_id, author_name, asset_name, display_name, description, asset_type as "asset_type: AssetType", archive_path, downloads, total_likes, is_public, created_at, updated_at
                FROM assets
                WHERE display_name ILIKE $1"#,
            search
        )
        .fetch_all(pool)
        .await?;

        Ok(assets)
    }

    pub async fn get_all_with_asset_ids(pool: &PgPool, asset_ids: &[i64]) -> Result<Vec<Self>, sqlx::Error> {
        if asset_ids.is_empty() {
            return Ok(vec![]);
        }

        let assets = sqlx::query_as!(
            Asset,
            r#"SELECT id, owner_id, author_name, asset_name, display_name, description, asset_type as "asset_type: AssetType", archive_path, downloads, total_likes, is_public, created_at, updated_at
                FROM assets
                WHERE id = ANY($1)"#,
            asset_ids
        )
        .fetch_all(pool)
        .await?;

        let unique_count = asset_ids.iter().collect::<std::collections::HashSet<_>>().len();
        if assets.len() != unique_count {
            return Err(sqlx::Error::RowNotFound);
        }

        Ok(assets)
    }

    pub async fn get_all_with_tag_names(pool: &PgPool, tag_names: &[&str]) -> Result<Vec<Self>, sqlx::Error> {
        if tag_names.is_empty() {
            return Ok(vec![]);
        }

        let assets = sqlx::query_as!(
            Asset,
            r#"SELECT id, owner_id, author_name, asset_name, display_name, description, asset_type as "asset_type: AssetType", archive_path, downloads, total_likes, is_public, created_at, updated_at
                FROM assets
                WHERE id IN (
                    SELECT at.asset_id
                    FROM asset_tags at
                    INNER JOIN tags t ON t.id = at.tag_id
                    WHERE t.name = ANY($1)
                    GROUP BY at.asset_id
                    HAVING COUNT(DISTINCT t.name) >= $2
                )"#,
            tag_names as &[&str],
            tag_names.len() as i64
        )
        .fetch_all(pool)
        .await?;

        Ok(assets)
    }

    pub async fn change_likes(&self, pool: &PgPool, add_like: bool) -> Result<Self, sqlx::Error> {
        let new = sqlx::query_as!(
            Asset,
            r#"UPDATE assets 
                SET total_likes = total_likes + $1
                WHERE id = $2
                RETURNING id, owner_id, author_name, asset_name, display_name, description, asset_type as "asset_type: AssetType", archive_path, is_public, downloads, total_likes, created_at, updated_at"#,
            if add_like { 1 } else { -1 },
            self.id
        )
        .fetch_one(pool)
        .await?;

        Ok(new)
    }

    pub async fn increment_downloads(&self, pool: &PgPool) -> Result<Self, sqlx::Error> {
              let new = sqlx::query_as!(
            Asset,
            r#"UPDATE assets 
                SET downloads = downloads + 1
                WHERE id = $1
                RETURNING id, owner_id, author_name, asset_name, display_name, description, asset_type as "asset_type: AssetType", archive_path, is_public, downloads, total_likes, created_at, updated_at"#,
            self.id
        )
        .fetch_one(pool)
        .await?;

        Ok(new)  
    }
}