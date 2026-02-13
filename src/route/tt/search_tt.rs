use axum::{Json, extract::State, http::HeaderMap};
use reqwest::StatusCode;
use serde_json::{json, to_string};
use sqlx::types::Uuid;

use crate::{db::{tt::tt_entry::TimeTrialEntry, user::User}, state::ThreadSafeState};

#[derive(Debug, serde::Deserialize)]
pub struct SearchTTRequest {
    pub username: Option<String>,
    // uuid to be parsed
    pub car_id: Option<String>,
    // uuid to be parsed
    pub stage_id: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct SearchTTResponse {
    pub id: String,
    pub ticks: i32,
    pub username: String,
    pub car_id: String,
    pub stage_id: String,
    /// ISO 8601 format
    pub created_at: String,
}
impl SearchTTResponse {
    pub fn from_time_trial_entry(entry: TimeTrialEntry, username: String) -> Self {
        SearchTTResponse {
            id: entry.id.to_string(),
            ticks: entry.total_ticks,
            username,
            car_id: entry.car_id.to_string(),
            stage_id: entry.stage_id.to_string(),
            created_at: entry.created_at.map_or_else(|| String::new(), |dt| dt.to_string()),
        }
    }
}

fn validate_search_tt_request(req: &SearchTTRequest) -> Result<(), String> {
    if req.username.is_none() && req.car_id.is_none() && req.stage_id.is_none() {
        return Err("At least one search parameter must be provided".to_string());
    }
    Ok(())
}

// TODO: sort by fastest first; for now just return in whatever order the database gives us
// TODO: limit number of results returned and add pagination; since we aren't storing many things yet it's not an issue
pub async fn search_tt(State(state): State<ThreadSafeState>, headers: HeaderMap, axum::Json(req): axum::Json<SearchTTRequest>) -> axum::response::Result<(StatusCode, axum::Json<Vec<SearchTTResponse>>)> {
    validate_search_tt_request(&req).map_err(|e| (axum::http::StatusCode::BAD_REQUEST, axum::Json(json!({"status": e}))))?;

    let pool= &state.lock().await.db_pool;
    let car_uuid = req.car_id.as_ref()
        .map(|s| Uuid::parse_str(s)).transpose()
        .map_err(|_| (axum::http::StatusCode::BAD_REQUEST, axum::Json(json!({"status": "invalid car_id uuid"}))))?;
    let stage_uuid = req.stage_id.as_ref()
        .map(|s| Uuid::parse_str(s)).transpose()
        .map_err(|_| (axum::http::StatusCode::BAD_REQUEST, axum::Json(json!({"status": "invalid stage_id uuid"}))))?;

    let user_id = if let Some(username) = req.username {
        let user = User::get_id_from_username(pool, &username)
            .await
            .map_err(|_| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, axum::Json(json!({"status": "internal database error"}))))?;
        
        if let Some(u) = user {
            Some(u)
        } else {
            return Err((axum::http::StatusCode::NOT_FOUND, axum::Json(json!({"status": "user not found"}))).into());
        }
    } else {
        None
    };

    let raw_tts = TimeTrialEntry::filter(pool, user_id, car_uuid, stage_uuid)
        .await
        .map_err(|_| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, axum::Json(json!({"status": "internal database error"}))))?;

    let mut tts = Vec::new();
    for tt in raw_tts {
        let user = User::get_by_user_id(pool, tt.user_id)
            .await
            .map_err(|_| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, axum::Json(json!({"status": "internal database error"}))))?;
        let username = user.map_or_else(|| "Unknown".to_string(), |u| u.username);
        tts.push(SearchTTResponse::from_time_trial_entry(tt, username));
    }
        
    Ok((StatusCode::OK, Json(tts)))
}