use crate::{
    db::{token::UserToken, tt::tt_entry::TimeTrialEntry},
    state::ThreadSafeState,
    tt::{get_tt_version, validate_upload_tt_file, write_tt_file},
};
use axum::{Json, extract::{State, multipart}, http::HeaderMap};
use reqwest::StatusCode;
use serde_json::json;
use sqlx::types::Uuid;

#[derive(Debug, serde::Deserialize)]
pub struct UploadTTMetadata {
    pub car_id: String,
    pub stage_id: String,
}

// Time trial uploaded as multipart: UploadTTMetadata in "metadata" field, file data in "file" field.

pub fn validate_upload_tt_metadata(metadata: &UploadTTMetadata) -> Result<(Uuid, Uuid), String> {
    let car_uuid =
        Uuid::parse_str(&metadata.car_id).map_err(|_| "Invalid car_id UUID".to_string())?;
    let stage_uuid =
        Uuid::parse_str(&metadata.stage_id).map_err(|_| "Invalid stage_id UUID".to_string())?;
    Ok((car_uuid, stage_uuid))
}

pub async fn upload_tt(
    State(state): State<ThreadSafeState>,
    headers: HeaderMap,
    mut multipart: axum::extract::Multipart,
) -> axum::response::Result<(StatusCode, Json<serde_json::Value>)> {
    let metadata = multipart
        .next_field()
        .await?
        .ok_or((StatusCode::BAD_REQUEST, Json(json!({"status": "missing metadata field"}))))?;
    let metadata_bytes = metadata.bytes().await?;
    let metadata: UploadTTMetadata = serde_json::from_slice(&metadata_bytes)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({"status": "invalid metadata"}))))?;

    let file_bytes = multipart
        .next_field()
        .await?
        .ok_or((StatusCode::BAD_REQUEST, Json(json!({"status": "missing file field"}))))?
        .bytes()
        .await?;

    let authorization = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or((
            StatusCode::UNAUTHORIZED,
            Json(json!({"status": "missing or invalid Authorization header"})),
        ))?;

    let pool = &state.lock().await.db_pool;
    let user_id = UserToken::get_user_by_token(pool, authorization)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "internal database error"})),
            )
        })?
        .ok_or((
            StatusCode::UNAUTHORIZED,
            Json(json!({"status": "invalid token"})),
        ))?
        .user_id;

    let (car_uuid, stage_uuid) = validate_upload_tt_metadata(&metadata)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(json!({"status": e}))))?;
    let res = validate_upload_tt_file(&file_bytes)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(json!({"status": e}))))?;

    // Need to look for existing TTs from this user for the same car and stage.
    // We only want to store the fastest TT for each user/car/stage combination, so if an existing TT is found, we will only update the DB entry if the new TT is faster.
    // The file will be replaced in either case since the old one is now invalid.
    let existing = TimeTrialEntry::filter(pool, Some(user_id), Some(car_uuid), Some(stage_uuid))
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "internal database error"})),
            )
        })?
        .pop();

    let ver = get_tt_version(&file_bytes)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(json!({"status": e}))))?;

    if let Some(existing) = existing {
        // We can assume that the existing TT is valid.
        if existing.total_ticks >= res.elapsed_ticks {
            TimeTrialEntry::update(pool, existing.id, ver, res.elapsed_ticks)
                .await
                .map_err(|_| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({"status": "internal database error"})),
                    )
                })?;

            // The file will be replaced by the caller since the same TT ID is returned for the same user/car/stage combination.
            write_tt_file(&state, existing.id, &file_bytes)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({"status": format!("failed to write TT file: {}", e)})),
                    )
                })?;
        }
    } else {
        let new_entry = TimeTrialEntry::insert(pool, user_id, car_uuid, stage_uuid, ver, res.elapsed_ticks)
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"status": "internal database error"})),
                )
            })?;

        write_tt_file(&state, new_entry.id, &file_bytes)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"status": format!("failed to write TT file: {}", e)})),
                )
            })?;
    }

    Ok((
        StatusCode::OK,
        Json(json!({"status": "file validated successfully"})),
    ))
}
