use anyhow::anyhow;
use axum::{Json, extract::{Multipart, State}};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{database::{account::User, assets::{Asset, AssetType}, roles::UserRole, tags::Tag}, extractor::auth::AuthUser, ffi::{ValidateRadpackArgs, nfmw_validate_radpack}, route::error::{AppError, ErrorResponse}, state::AppState, store::AssetStore};

#[derive(ToSchema)]
pub struct CreateAssetMultipart {
    pub metadata: CreateAssetRequest,
    pub file: Vec<u8>,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateAssetRequest {
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
    operation_id = "createAsset",
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
    let mut json_metadata: Option<CreateAssetRequest> = None;
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
    let existing = Asset::get_owner_id(pool, auth.user_id, metadata.asset_type, &metadata.asset_name).await?.is_some();

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

    let mut nonexistent_tags = vec![];
    let mut existing_tags = vec![];

    if let Some(tags) = metadata.tags {
        for tag in tags {
            let existing_tag = Tag::get_from_name(pool, &tag).await?;
            if existing_tag.is_none() {
                nonexistent_tags.push(tag.clone());
            } else {
                existing_tags.push(existing_tag.unwrap());
            }
        }
    }

    let radpack_validation = {
        let radpack_args = ValidateRadpackArgs {
            radpack_data: file_data.as_ptr(),
            radpack_data_length: file_data.len() as i32
        };
        unsafe { nfmw_validate_radpack(&radpack_args as *const _) }
    };
    if radpack_validation.has_error {
        let exception = radpack_validation.exception;
        return Err(AppError::BadRequest(format!("Invalid radpack file: {}", String::from_utf8_lossy(&exception.message))))
    }

    if !nonexistent_tags.is_empty() {
        return Err(AppError::BadRequest(format!("The following tags do not exist: {}. Please create them first.", nonexistent_tags.join(", "))))
    }

    if !existing_tags.is_empty() {
        let user_roles = UserRole::get_roles_for_user_id(pool, auth.user_id).await?;
        for tag in existing_tags {
            if let Some(role_id) = tag.required_role_id {
                if !user_roles.iter().any(|x| x.role_id == role_id) {
                    return Err(AppError::Forbidden(format!("Cannot assign privileged tag {} as you do not have the correct role to do so.", tag.name)));
                }
            }
        }
    }

    let object_ref = AssetStore::new_key(metadata.asset_type);

    state.asset_store.put(&object_ref.1, &file_data, "application/octet-stream").await
        .map_err(|e| AppError::Other(e))?;

    let asset = Asset::insert(
        pool,
        auth.user_id, 
        &author_name, 
        &metadata.asset_name, 
        &metadata.display_name, 
        metadata.description.as_ref().map(|x| x.as_str()), 
        metadata.asset_type, 
        &object_ref.1,
        true).await?;

    Ok(Json(CreateAssetResponse { asset_id: asset.id, canonical_name }))
}
