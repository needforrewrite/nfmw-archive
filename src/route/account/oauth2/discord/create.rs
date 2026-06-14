use axum::{Json, extract::State};
use serde::Deserialize;
use serde_json::{Value, json};
use time::OffsetDateTime;

use crate::{
    crypto::generate_base64_authentication_token,
    database::account::{
        OauthIdentity, User, UserSessions,
        oauth2::pendingregistration::PendingOauthRegistration,
    },
    route::{account::validate_username, error::AppError},
    state::ThreadSafeState,
};

#[derive(Deserialize)]
pub struct CreateAccountBody {
    pub temp_token: String,
    pub username: String,
}

pub async fn handler(
    State(state): State<ThreadSafeState>,
    Json(body): Json<CreateAccountBody>,
) -> Result<Json<Value>, AppError> {
    let pool = {
        let g = state.lock().await;
        g.db_pool.clone()
    };

    let pending = PendingOauthRegistration::get_by_temp_token(&pool, &body.temp_token)
        .await?
        .ok_or_else(|| AppError::BadRequest("invalid or expired temp token".to_string()))?;

    if pending.expires_at < OffsetDateTime::now_utc() {
        return Err(AppError::BadRequest("temp token has expired".to_string()));
    }

    validate_username(&body.username).map_err(|e| AppError::BadRequest(e))?;

    let user = User::create(&pool, &body.username, pending.email.as_deref()).await.map_err(|e| {
        if let sqlx::Error::Database(ref dbe) = e {
            // 23505 is the Postgres error code for unique_violation, which we can use to detect if the username is already taken.
            if dbe.code().as_deref() == Some("23505") {
                return AppError::BadRequest("username already taken".to_string());
            }
        }
        AppError::Database(e)
    })?;

    OauthIdentity::insert(
        &pool,
        user.id,
        &pending.provider,
        &pending.provider_subject,
        pending.access_token.as_deref(),
        pending.refresh_token.as_deref(),
        pending.token_expires_at,
    )
    .await?;

    pending.delete(&pool).await?;

    let session_token = generate_base64_authentication_token();
    UserSessions::upsert(&pool, user.id, &session_token, "discord").await?;

    Ok(Json(json!({
        "session_token": session_token,
        "username": user.username,
    })))
}
