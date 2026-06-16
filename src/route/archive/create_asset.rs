use anyhow::anyhow;
use axum::{Json, extract::{Multipart, State}};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{database::{account::User, assets::{Asset, AssetType}}, extractor::auth::AuthUser, route::error::{AppError, ErrorResponse}, state::AppState};

#[derive(ToSchema)]
pub struct CreateAssetMultipart {
    pub metadata: CreateAssetBody,
    pub file: Vec<u8>,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateAssetBody {
    asset_name: String,
    display_name: String,
    description: Option<String>,
    tags: Option<Vec<String>>,
    asset_type: AssetType
}

#[derive(Serialize, ToSchema)]
pub struct CreateAssetResponse {
    asset_id: i64,
    canonical_name: String,
}

#[utoipa::path(
    post,
    path = "/assets/create",
    params (
        (
            "Authorization" = String,
            Header,
            description = "Bearer token for user authentication"
        )
    ),
    request_body (     
        content = CreateAssetMultipart,
        description = "Multipart form data containing asset metadata and file radpack bytes",
        content_type = "multipart/form-data"
    ),
    responses(
        (status = 200, description = "Asset created successfully", body = CreateAssetResponse),
        (status = 400, description = "Invalid request body", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden (likely due to attempted assignment of privileged tag)", body = ErrorResponse),
        (status = 409, description = "Asset name already used for this asset type by the current user", body = ErrorResponse),
        (status = 413, description = "Asset is larger than the maximum allowed size", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    tag = "asset-management"
)]
pub async fn create_asset(
    auth: AuthUser,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<CreateAssetResponse>, AppError> {
    let mut json_metadata: Option<CreateAssetBody> = None;
    let mut file_data: Option<Vec<u8>> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::BadRequest(e.to_string()))? {
        let name = field.name().unwrap_or_default();
        match name {
            "metadata" => {
                let data = field.text().await.map_err(|e| AppError::BadRequest(e.to_string()))?;
                json_metadata = Some(serde_json::from_str(&data).map_err(|e| AppError::BadRequest(e.to_string()))?);
            }
            "file" => {
                let data = field.bytes().await.map_err(|e| AppError::BadRequest(e.to_string()))?;
                file_data = Some(data.to_vec());
            }
            _ => {}
        }
    }

    let (metadata, file_data) = match (json_metadata, file_data) {
        (Some(meta), Some(file)) => (meta, file),
        _ => return Err(AppError::BadRequest("Missing required metadata or file".to_string())),
    };

    let pool = &state.db_pool;
    let existing = Asset::get_owned_by_of_type_with_name(pool, auth.user_id, metadata.asset_type, &metadata.asset_name).await?.is_some();

    if existing {
        return Err(AppError::Conflict("You already own an asset of this type with the same asset name.".into()))
    }

    if file_data.len() >= (state.config.asset_max_size_kb * 1000) as usize {
        return Err(AppError::ContentTooLarge(state.config.asset_max_size_kb))
    }

    let author_name = User::get_canonical_author_name_from_user_id(pool, auth.user_id).await?;
    if author_name.is_none() {
        return Err(AppError::Other(anyhow!("No user was found for the associated authenticated user_id {}. This should never happen.", auth.user_id)));
    }
    let author_name = author_name.unwrap();
    let canonical_name = format!("{author_name}/{}", metadata.asset_name.clone());

    let asset = Asset::insert(
        pool,
        auth.user_id, 
        &author_name, 
        &metadata.asset_name, 
        &metadata.display_name, 
        metadata.description.as_ref().map(|x| x.as_str()), 
        metadata.asset_type, 
        todo!(),
        true).await?;

    Ok(Json(CreateAssetResponse { asset_id: asset.id, canonical_name }))
}