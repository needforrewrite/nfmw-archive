use serde::Deserialize;
use sqlx::{PgPool, types::time::OffsetDateTime};
use utoipa::ToSchema;

pub mod asset;
pub mod asset_like;

#[derive(Debug, Copy, Clone, PartialEq, Eq, sqlx::Type, ToSchema, Deserialize)]
#[sqlx(type_name = "asset_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
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