use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;

use crate::{
    crypto::{generate_base64_authentication_token, hash_token},
    database::account::{
        OauthIdentity, User, UserSessions, oauth2::pendingregistration::PendingOauthRegistration,
    },
    route::{
        account::validate_username,
        error::{AppError, ErrorResponse},
    },
    state::AppState,
};

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateAccountBody {
    pub temp_token: String,
    pub username: String,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DiscordCreateAccountResponse {
    pub session_token: String,
    pub username: String,
}

#[utoipa::path(
    post,
    operation_id = "createDiscordAccount",
    path = "/auth/discord/create_account",
    request_body = CreateAccountBody,
    responses(
        (status = 200, description = "Discord account created and session started", body = DiscordCreateAccountResponse),
        (status = 400, description = "Invalid or expired temp token, or username taken", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    tag = "discord-oauth"
)]
pub async fn discord_create_account(
    State(state): State<AppState>,
    Json(body): Json<CreateAccountBody>,
) -> Result<Json<DiscordCreateAccountResponse>, AppError> {
    let pool = state.db_pool.clone();

    let pending = PendingOauthRegistration::get_by_temp_token(&pool, &body.temp_token)
        .await?
        .ok_or_else(|| AppError::BadRequest("invalid or expired temp token".to_string()))?;

    if pending.expires_at < OffsetDateTime::now_utc() {
        return Err(AppError::BadRequest("temp token has expired".to_string()));
    }

    validate_username(&body.username).map_err(|e| AppError::BadRequest(e))?;

    if User::username_taken(&pool, &body.username).await? {
        return Err(AppError::BadRequest("username already taken".to_string()));
    }

    let user = User::create(&pool, &body.username, pending.email.as_deref())
        .await
        .map_err(|e| {
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
    let session_token_hash = hash_token(&session_token);

    UserSessions::create_archive_session(&pool, user.id, &session_token_hash, "discord").await?;

    Ok(Json(DiscordCreateAccountResponse {
        session_token,
        username: user.username,
    }))
}