pub mod account;
pub mod error;

use axum::{extract::State, response::Json};
use serde::Serialize;
use utoipa::ToSchema;

use crate::state::ThreadSafeState;

#[derive(Serialize, ToSchema)]
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
pub async fn root(State(_state): State<ThreadSafeState>) -> Json<HealthResponse> {
    Json(HealthResponse { status: "healthy".into() })
}