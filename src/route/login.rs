use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};

use crate::{crypto::generate_base64_authentication_token, db::{token::UserToken, user::User}};

#[derive(serde::Deserialize)]
pub struct LoginPayload {
    pub username: String,
    pub password: String,
}

pub async fn login(State(state): State<Arc<crate::state::State>>, Json(payload): Json<LoginPayload>) -> (StatusCode, Json<serde_json::Value>) {
    let user = User::new_from_password(payload.username, payload.password, false);

    let user_exists = user.check_username_exists(&state.db_pool).await;
    if let Err(e) = user_exists {
        eprintln!("Database error on username lookup: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"status": "internal server error"})));
    } else if !user_exists.unwrap_or(false) {
        return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"status": "invalid credentials"})));
    }

    if user.must_change_password {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({"status": "password change required"})));
    }

    // Generate a new token
    let token = generate_base64_authentication_token();
    let user_token = UserToken::new(user.id, token.clone());

    // Remove any existing tokens for this user
    UserToken::remove_all(user.id, &state.db_pool).await.unwrap();

    // Insert the new token
    user_token.insert(&state.db_pool).await.unwrap();

    (StatusCode::OK, Json(serde_json::json!({"status": "login successful", "token": token})))
}