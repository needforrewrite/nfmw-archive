use axum::{Json, extract::State, http::StatusCode};

use crate::{crypto::generate_base64_authentication_token, db::{token::UserToken, user::User}, state::ThreadSafeState};

#[derive(serde::Deserialize)]
pub struct LoginPayload {
    pub username: String,
    pub password: String,
}

pub async fn login(State(state): State<ThreadSafeState>, Json(payload): Json<LoginPayload>) -> (StatusCode, Json<serde_json::Value>) {
    let pool = &state.lock().await.db_pool;

    let user = User::get_by_username_and_local_password(pool, &payload.username, &payload.password).await;

    if let Err(e) = user {
        eprintln!("Database error on username lookup: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"status": "internal server error"})));
    } else if user.as_ref().unwrap().is_none() {
        return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"status": "invalid credentials"})));
    }

    let user = user.unwrap().unwrap();

    if user.must_change_password.unwrap_or(false) {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({"status": "password change required"})));
    }

    // Generate a new token
    let token = generate_base64_authentication_token();

    // Remove any existing tokens for this user
    UserToken::remove_all(user.id, pool).await.unwrap();

    // Insert the new token
    UserToken::insert(pool, user.id, &token).await.unwrap();

    (StatusCode::OK, Json(serde_json::json!({"status": "login successful", "token": token})))
}