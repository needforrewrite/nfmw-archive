// Fetch a TT by its UUID. Returns the TT file data, as well as the metadata, using multipart:
// "metadata" based on a subset of GetTTInfoResult, and "file" containing the raw TT file bytes.

use axum::{Json, extract::State, http::HeaderMap, response::IntoResponse};
use axum_extra::response::multiple::{MultipartForm, Part};

use crate::ffi::GetTTInfoArgs;

#[derive(Debug, serde::Deserialize)]
pub struct FetchTTRequest {
    pub tt_id: String,
}

#[derive(serde::Serialize)]
pub struct FetchTTMetadata {
    pub checkpoint_count: i32,
    pub tick_count: i32,
    pub replay_version: i32,
    pub backend_version: i32,
}

// TODO: restrict to logged in users; for now no real need
pub async fn fetch_tt(State(state): State<crate::state::ThreadSafeState>, headers: HeaderMap, Json(request): Json<FetchTTRequest>) -> axum::response::Result<axum::response::Response> {
    let tt_uuid = sqlx::types::Uuid::parse_str(&request.tt_id)
        .map_err(|_| axum::response::Response::builder().status(400).body("Invalid TT ID".to_owned()).unwrap())?;

    let tt_bytes = crate::tt::read_tt_file(&state, tt_uuid).await
        .map_err(|_| axum::response::Response::builder().status(404).body("TT not found".to_owned()).unwrap())?;

    let args = GetTTInfoArgs {
        time_trial_data: tt_bytes.as_ptr(),
        time_trial_data_length: tt_bytes.len() as i32,
    };

    let info = unsafe { crate::ffi::nfmw_get_tt_info(&args as *const _) };

    if info.has_error {
        return Err(axum::response::Response::builder().status(500).body(format!("Failed to read TT info: {}", String::from_utf8_lossy(&info.exception.message))).unwrap().into());
    }

    let metadata = FetchTTMetadata {
        checkpoint_count: info.checkpoint_count,
        tick_count: info.tick_count,
        replay_version: info.replay_version,
        backend_version: info.backend_version,
    };

    let multipart_parts = vec![
        Part::text("metadata".to_owned(), &serde_json::to_string(&metadata).unwrap()),
        Part::file("file", "timetrial", tt_bytes)
    ];

    Ok(MultipartForm::with_parts(multipart_parts).into_response())
}