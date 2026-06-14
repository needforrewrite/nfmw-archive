pub mod account;
pub mod error;

use axum::{extract::State, response::Json};
use serde_json::Value;

use crate::state::ThreadSafeState;


pub async fn root(State(_): State<ThreadSafeState>) -> Json<Value> {
    Json(serde_json::json!({"status": "healthy"}))
}