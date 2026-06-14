use std::sync::OnceLock;

use argon2::{PasswordHasher, PasswordVerifier, password_hash::{SaltString, rand_core::OsRng}};
use axum::{Json, extract::State};
use serde::Deserialize;

use crate::{crypto::generate_base64_authentication_token, database::account::{LocalCredentials, User, UserSessions}, route::error::AppError, state::ThreadSafeState};

// Computed once on first use; gives us a real argon2 hash with the same parameters as user
// passwords so we can always call verify_password and avoid leaking whether a username exists
// via response-time differences.
static DUMMY_HASH: OnceLock<String> = OnceLock::new();

fn dummy_password_hash() -> &'static str {
    DUMMY_HASH.get_or_init(|| {
        let salt = SaltString::generate(&mut OsRng);
        argon2::Argon2::default()
            .hash_password(b"dummy", &salt)
            .expect("dummy hash generation failed")
            .to_string()
    })
}

#[derive(Deserialize)]
pub struct LoginLocalAccountRequest {
     pub username: String,
     pub password: String,
 }

pub async fn handler(
    State(state): State<ThreadSafeState>,
    Json(body): Json<LoginLocalAccountRequest>) -> Result<Json<serde_json::Value>, AppError> {
    let pool = &state.lock().await.db_pool;

    let user = User::get_by_username(pool, &body.username).await.map_err(|e| AppError::Database(e))?;

    let credentials = if let Some(ref user) = user {
        LocalCredentials::get_by_user_id(pool, user.id).await.map_err(|e| AppError::Database(e))?
    } else {
        None
    };

    let argon2 = argon2::Argon2::default();
    let hash_str = credentials
        .as_ref()
        .map(|c| c.password_hash.clone())
        .unwrap_or_else(|| dummy_password_hash().to_owned());
    let password_hash = argon2::PasswordHash::new(&hash_str).map_err(|e| AppError::Other(anyhow::anyhow!(e.to_string())))?;

    // Always verify to maintain constant time regardless of whether the user or credentials exist.
    let verified = argon2.verify_password(body.password.as_bytes(), &password_hash).is_ok();

    if user.is_some() && credentials.is_some() && verified {
        let user = user.unwrap();
        let session_token = generate_base64_authentication_token();
        UserSessions::upsert(pool, user.id, &session_token, "local")
            .await
            .map_err(|e| AppError::Database(e))?;

        Ok(Json(serde_json::json!({
            "username": user.username,
            "session_token": session_token,
        })))
    } else {
        Err(AppError::BadRequest("invalid username or password".to_string()))
    }
}