use axum::{Json, extract::State, http::StatusCode};

use crate::{crypto::generate_base64_authentication_token, db::{token::UserToken, user::User}, state::ThreadSafeState};

#[derive(serde::Deserialize)]
pub struct LoginPayload {
    pub username: String,
    pub password: String,
}

pub async fn login(State(state): State<ThreadSafeState>, Json(payload): Json<LoginPayload>) -> (StatusCode, Json<serde_json::Value>) {
    let user = User::new_from_password(payload.username, payload.password, Some(false));

    let pool = &state.lock().await.db_pool;

    let user_exists = user.check_username_exists(pool).await;
    if let Err(e) = user_exists {
        eprintln!("Database error on username lookup: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"status": "internal server error"})));
    } else if !user_exists.unwrap_or(false) {
        return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"status": "invalid credentials"})));
    }

    if user.must_change_password.unwrap_or(false) {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({"status": "password change required"})));
    }

    // Generate a new token
    let token = generate_base64_authentication_token();
    let user_token = UserToken::new(user.id, token.clone());

    // Remove any existing tokens for this user
    UserToken::remove_all(user.id, pool).await.unwrap();

    // Insert the new token
    user_token.insert(pool).await.unwrap();

    (StatusCode::OK, Json(serde_json::json!({"status": "login successful", "token": token})))
}