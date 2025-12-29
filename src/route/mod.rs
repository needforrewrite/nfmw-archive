use std::sync::Arc;

use axum::{extract::State, response::Json};
use serde_json::Value;

pub mod create_account;
pub mod login;

pub async fn root(State(_): State<Arc<crate::state::State>>) -> Json<Value> {
    Json(serde_json::json!({"status": "healthy"}))
}