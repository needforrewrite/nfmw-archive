use axum::{extract::State, response::Json};
use serde_json::Value;

use crate::state::ThreadSafeState;

pub mod create_account;
pub mod login;

pub async fn root(State(_): State<ThreadSafeState>) -> Json<Value> {
    Json(serde_json::json!({"status": "healthy"}))
}