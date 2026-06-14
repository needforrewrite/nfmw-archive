use sqlx::types::time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "asset_type", rename_all = "snake_case")]
pub enum AssetType {
    Track,
    Car,
    TrackPiece,
    Texture,
    Sound,
    Campaign,
    Other,
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
    pub archive_path: Option<String>,
    pub is_public: bool,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AssetDependency {
    pub dependent_id: i64,
    pub dependency_id: i64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AssetTag {
    pub asset_id: i64,
    pub tag_id: i32,
    pub applied_by: Option<i64>,
    pub applied_at: OffsetDateTime,
}
