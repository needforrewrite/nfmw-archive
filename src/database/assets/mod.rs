use serde::Deserialize;
use sqlx::{PgPool, types::time::OffsetDateTime};
use utoipa::ToSchema;

#[derive(Debug, Copy, Clone, PartialEq, Eq, sqlx::Type, ToSchema, Deserialize)]
#[sqlx(type_name = "asset_type", rename_all = "snake_case")]
pub enum AssetType {
    Track,
    Car,
    TrackPiece,
    Texture,
    Sound,
    Campaign,
    Wheel
}
impl ToString for AssetType {
    fn to_string(&self) -> String {
        match self {
            AssetType::Campaign => "campaign".into(),
            AssetType::Car => "car".into(),
            AssetType::Sound => "sound".into(),
            AssetType::Texture => "texture".into(),
            AssetType::Track => "track".into(),
            AssetType::TrackPiece => "track_piece".into(),
            AssetType::Wheel => "wheel".into()
        }
    }
}
impl TryFrom<String> for AssetType {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_ascii_lowercase().as_ref() {
            "campaign" => Ok(AssetType::Campaign),
            "car" => Ok(AssetType::Car),
            "sound" => Ok(AssetType::Sound),
            "texture" => Ok(AssetType::Texture),
            "track" => Ok(AssetType::Track),
            "track_piece" => Ok(AssetType::TrackPiece),
            "wheel" => Ok(AssetType::Wheel),
            _ => Err(anyhow::anyhow!("Invalid asset type"))
        }
    }
}
impl serde::Serialize for AssetType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}


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
                 RETURNING id, owner_id, author_name, asset_name, display_name, description, asset_type as "asset_type: AssetType", archive_path, is_public, created_at, updated_at"#,
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
            r#"SELECT id, owner_id, author_name, asset_name, display_name, description, asset_type as "asset_type: AssetType", archive_path, is_public, created_at, updated_at
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
            r#"SELECT id, owner_id, author_name, asset_name, display_name, description, asset_type as "asset_type: AssetType", archive_path, is_public, created_at, updated_at
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
            r#"SELECT id, owner_id, author_name, asset_name, display_name, description, asset_type as "asset_type: AssetType", archive_path, is_public, created_at, updated_at
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
            r#"SELECT id, owner_id, author_name, asset_name, display_name, description, asset_type as "asset_type: AssetType", archive_path, is_public, created_at, updated_at
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
            r#"SELECT id, owner_id, author_name, asset_name, display_name, description, asset_type as "asset_type: AssetType", archive_path, is_public, created_at, updated_at
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
            r#"SELECT id, owner_id, author_name, asset_name, display_name, description, asset_type as "asset_type: AssetType", archive_path, is_public, created_at, updated_at
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
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AssetTag {
    pub asset_id: i64,
    pub tag_id: i32,
    pub applied_by: Option<i64>,
    pub applied_at: OffsetDateTime,
}
impl AssetTag {
    pub async fn assign_tag_to_asset(pool: &PgPool, asset_id: i64, tag_id: i32, applied_by: i64) -> Result<Self, sqlx::Error> {
        let asset_tag = sqlx::query_as!(
            AssetTag,
            r#"INSERT INTO asset_tags (asset_id, tag_id, applied_by)
                VALUES ($1, $2, $3)
                RETURNING *"#,
            asset_id,
            tag_id,
            applied_by
        )
        .fetch_one(pool)
        .await?;

        Ok(asset_tag)
    }

    pub async fn get_asset_ids_for_tag(pool: &PgPool, tag_id: i32) -> Result<Vec<Self>, sqlx::Error> {
        let asset_tags = sqlx::query_as!(
            AssetTag,
            r#"SELECT *
                FROM asset_tags
                WHERE tag_id = $1"#,
            tag_id,
        )
        .fetch_all(pool)
        .await?;

        Ok(asset_tags)
    }

    pub async fn asset_id_has_all_tag_ids(pool: &PgPool, asset_id: i64, tag_ids: &[i32]) -> Result<bool, sqlx::Error> {
        if tag_ids.is_empty() {
            return Ok(true);
        }

        let result = sqlx::query_scalar!(
            r#"SELECT COUNT(DISTINCT tag_id) = $2 FROM asset_tags WHERE asset_id = $1 AND tag_id = ANY($3)"#,
            asset_id,
            tag_ids.len() as i64,
            tag_ids
        )
        .fetch_one(pool)
        .await?;

        Ok(result.unwrap_or(false))
    }

    pub async fn get_asset_ids_with_all_tag_ids(pool: &PgPool, tag_ids: &[i32]) -> Result<Vec<Self>, sqlx::Error> {
        let asset_tags = sqlx::query_as!(
            AssetTag,
            r#"SELECT *
                FROM asset_tags
                WHERE tag_id = ANY($1)
                GROUP BY asset_id, tag_id, applied_by, applied_at
                HAVING COUNT(DISTINCT tag_id) >= $2"#,
            tag_ids,
            tag_ids.len() as i64
        )
        .fetch_all(pool)
        .await?;

        Ok(asset_tags)
    }
}