use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{database::assets::{AssetType, asset::Asset, asset_like::AssetLike}, extractor::auth::AuthUser, route::error::{AppError, ErrorResponse}, state::AppState};

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LikeAssetRequest {
    asset_type: AssetType,
    asset_author: String,
    asset_name: String,
    like: bool
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LikeAssetResponse {
    likes: i64
}

#[utoipa::path(
    post,
    operation_id = "setAssetLiked",
    path = "/assets/set_like",
    params (
        (
            "Authorization" = String,
            Header,
            description = "Bearer token for user authentication"
        )
    ),
    responses(
        (status = 200, description = "Liked status of asset changed", body = LikeAssetResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "No asset found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "asset-curation"
)]
pub async fn set_asset_liked(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<LikeAssetRequest>
) -> Result<Json<LikeAssetResponse>, AppError> {
    let pool = &state.db_pool.clone();
    let asset = Asset::get_author_name(pool, &body.asset_author, body.asset_type, &body.asset_name).await?
        .ok_or(AppError::NotFound)?;

    if AssetLike::user_has_liked_asset(pool, asset.id, auth.user_id).await? {
        if body.like {
            return Err(AppError::BadRequest("User has already liked this asset.".into()))
        } else {
            AssetLike::remove_like_from_asset_from_user(pool, asset.id, auth.user_id).await?;
        }
    } else {
        if body.like {
            AssetLike::add_like_to_asset_from_user(pool, asset.id, auth.user_id).await?;
        } else {
            return Err(AppError::BadRequest("User has not liked this asset.".into()))
        }
    }

    let new = asset.change_likes(pool, body.like).await?;

    Ok(Json(LikeAssetResponse {
        likes: new.total_likes
    }))
}
