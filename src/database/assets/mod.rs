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

    pub async fn get_owned_by_of_type_with_name(pool: &PgPool, owner_id: i64, asset_type: AssetType, asset_name: &str) -> Result<Option<Self>, sqlx::Error> {
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
}