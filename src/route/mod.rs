pub mod account;
pub mod error;
pub mod archive;

use axum::{extract::State, response::Json};
use serde::Serialize;
use utoipa::ToSchema;

use crate::state::AppState;

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HealthResponse {
    pub status: String,
}

#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse)
    ),
    tag = "health"
)]
pub async fn root(State(_state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse { status: "healthy".into() })
}