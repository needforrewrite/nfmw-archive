use axum::{response::Json};
use serde_json::Value;

pub async fn root() -> Json<Value> {
    Json(serde_json::json!({"status": "healthy"}))
}