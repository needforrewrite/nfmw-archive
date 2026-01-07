use axum::{Json, extract::State, http::StatusCode};

use crate::{db::user::User, state::ThreadSafeState};

#[derive(serde::Deserialize)]
pub struct CreateAccountPayload {
    pub username: String,
    pub password: String,
}

pub async fn create_account(State(state): State<ThreadSafeState>, Json(payload): Json<CreateAccountPayload>) -> (StatusCode, Json<serde_json::Value>) {
    let password = payload.password.clone();
    let user = User::new_from_password(payload.username, payload.password, Some(false));

    let pool = &state.lock().await.db_pool;

    let username_exists = user.check_username_exists(pool).await;
    if let Err(e) = username_exists {
        eprintln!("Database error on username lookup: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"status": "internal server error"})));
    } else if username_exists.unwrap_or(false) {
        return (StatusCode::CONFLICT, Json(serde_json::json!({"status": "username already exists"})));
    } else if !User::validate_password(&password) {
        return (StatusCode::BAD_REQUEST, 
            Json(serde_json::json!(
                {"status": "password does not meet complexity requirements of at least: 8 characters, one uppercase letter, one lowercase letter, one digit, no spaces, ASCII only"}
            ))
        );
    }

    let insert_result = user.insert_or_update(pool).await;
    if let Err(e) = insert_result {
        eprintln!("Database error on user insert: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"status": "internal server error"})));
    } 

    (StatusCode::OK, Json(serde_json::json!({"status": "account created"})))
}