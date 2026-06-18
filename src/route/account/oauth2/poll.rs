use axum::{Json, extract::{Path, State}};
use serde::Serialize;
use utoipa::ToSchema;

use crate::{database::account::oauth2::session::OauthSession, route::error::{AppError, ErrorResponse}, state::AppState};

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PollResponse {
    pub status: String,
    pub payload: Option<String>,
}

#[utoipa::path(
    get,
    operation_id = "pollOauth",
    path = "/auth/poll/{poll_id}",
    params(
        ("poll_id" = String, Path, description = "OAuth2 poll session ID returned by /auth/discord/start")
    ),
    responses(
        (status = 200, description = "Poll result (pending or final)", body = PollResponse),
        (status = 404, description = "Poll session not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    tag = "oauth-poll"
)]
pub async fn poll_oauth(State(state): State<AppState>, Path(poll_id): Path<String>) -> Result<Json<PollResponse>, AppError> {
    let pool = state.db_pool.clone();
    
    let session = OauthSession::get_by_poll_id(&pool, &poll_id)
        .await?
        .ok_or_else(|| AppError::NotFound)?;

    if let Some(ref result_status) = session.result_status {
        session.delete(&pool).await?;
        Ok(Json(PollResponse {
            status: result_status.clone(),
            payload: session.result_payload,
        }))
    } else {
        Ok(Json(PollResponse {
            status: "pending".into(),
            payload: None,
        }))
    }
}