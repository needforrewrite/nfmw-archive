use axum::{Json, extract::{Path, State}};
use serde_json::json;

use crate::{database::account::oauth2::session::OauthSession, state::ThreadSafeState, route::error::AppError};

pub async fn handle(State(state): State<ThreadSafeState>, Path(poll_id): Path<String>) -> Result<Json<serde_json::Value>, AppError> {
    let pool = {
        let g = state.lock().await;
        g.db_pool.clone()
    };

    let session = OauthSession::get_by_poll_id(&pool, &poll_id)
        .await?
        .ok_or_else(|| AppError::NotFound)?;

    if let Some(ref result_status) = session.result_status {
        session.delete(&pool).await?;
        Ok(Json(json!({ "status": result_status, "payload": session.result_payload })))
    } else {
        Ok(Json(json!({ "status": "pending", "payload": "" })))
    }
}