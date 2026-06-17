use axum::extract::{Path, State};

use crate::{database::{assets::{Asset, AssetType}}, extractor::auth::AuthUser, route::error::{AppError, ErrorResponse}, state::AppState};

#[utoipa::path(
    get,
    operation_id = "getAsset",
    path = "/assets/{asset_type}/{asset_author}/{asset_name}.radpack",
    params (
        (
            "Authorization" = String,
            Header,
            description = "Bearer token for user authentication"
        )
    ),
    responses(
        (status = 200, description = "Asset retrieved successfully", body = Vec<u8>),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "No asset found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "asset-management"
)]
pub async fn get_asset(
    _: AuthUser,
    State(state): State<AppState>,
    Path((asset_type, asset_author, asset_file)): Path<(String, String, String)>
) -> Result<Vec<u8>, AppError> {
    let asset_name = if let Some(asset_name) = asset_file.strip_suffix(".radpack") { asset_name.to_owned() } else { return Err(AppError::NotFound) };

    let asset_type = AssetType::try_from(asset_type)
        .map_err(|_| AppError::NotFound)?;

    let pool = &state.db_pool.clone();
    let asset = Asset::get_author_name(pool, &asset_author, asset_type, &asset_name).await?
        .ok_or(AppError::NotFound)?;

    let object_id = &asset.archive_path;
    let data = state.asset_store.get(object_id).await
        .map_err(|e| AppError::Other(e))?;

    Ok(data)
}
