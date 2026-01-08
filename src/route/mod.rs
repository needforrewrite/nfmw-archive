use axum::{body::Body, extract::State, http::{Response, StatusCode}, response::{IntoResponse, Json}};
use serde::Serialize;
use serde_json::Value;

use crate::state::ThreadSafeState;

pub mod archive;
pub mod create_account;
pub mod login;

pub async fn root(State(_): State<ThreadSafeState>) -> Json<Value> {
    Json(serde_json::json!({"status": "healthy"}))
}