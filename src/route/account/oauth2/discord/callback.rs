use axum::{Json, extract::{Query, State}};
use serde::Deserialize;
use serde_json::{Value, json};
use time::{Duration, OffsetDateTime};

use crate::{
    crypto::generate_base64_authentication_token,
    database::account::{
        OauthIdentity, User, UserSessions,
        oauth2::{pendingregistration::PendingOauthRegistration, session::OauthSession},
    },
    route::error::AppError,
    state::ThreadSafeState,
};

#[derive(Deserialize)]
pub struct CallbackQuery {
    code: String,
    state: String,
}

#[derive(Deserialize)]
struct DiscordTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: i64,
}

#[derive(Deserialize)]
struct DiscordUser {
    id: String,
    email: Option<String>,
}

pub async fn handler(
    State(state): State<ThreadSafeState>,
    Query(params): Query<CallbackQuery>,
) -> Result<Json<Value>, AppError> {
    let (pool, config, client) = {
        let g = state.lock().await;
        (g.db_pool.clone(), g.config.clone(), g.request_client.clone())
    };

    let session = OauthSession::get_by_state(&pool, &params.state)
        .await?
        .ok_or_else(|| AppError::BadRequest("invalid state".to_string()))?;
    session.delete(&pool).await?;

    let client_id = config.discord.client_id.to_string();
    let token_resp = client
        .post("https://discord.com/api/oauth2/token")
        .form(&[
            ("client_id", client_id.as_str()),
            ("client_secret", config.discord.client_secret.as_str()),
            ("grant_type", "authorization_code"),
            ("code", params.code.as_str()),
            ("redirect_uri", config.discord.redirect_uri.as_str()),
        ])
        .send()
        .await
        .map_err(|e| AppError::Other(e.into()))?
        .json::<DiscordTokenResponse>()
        .await
        .map_err(|e| AppError::Other(e.into()))?;

    let discord_user = client
        .get("https://discord.com/api/users/@me")
        .bearer_auth(&token_resp.access_token)
        .send()
        .await
        .map_err(|e| AppError::Other(e.into()))?
        .json::<DiscordUser>()
        .await
        .map_err(|e| AppError::Other(e.into()))?;

    let token_expires_at = OffsetDateTime::now_utc() + Duration::seconds(token_resp.expires_in);

    if let Some(identity) =
        OauthIdentity::get_by_provider_subject(&pool, "discord", &discord_user.id).await?
    {
        let user = User::get_by_id(&pool, identity.user_id)
            .await?
            .ok_or_else(|| AppError::Other(anyhow::anyhow!("orphaned oauth identity")))?;

        let session_token = generate_base64_authentication_token();
        UserSessions::upsert(&pool, user.id, &session_token, "discord").await?;

        return Ok(Json(json!({
            "session_token": session_token,
            "username": user.username,
        })));
    }

    let temp_token = generate_base64_authentication_token();
    PendingOauthRegistration::insert(
        &pool,
        "discord",
        &discord_user.id,
        Some(token_resp.access_token.as_str()),
        token_resp.refresh_token.as_deref(),
        Some(token_expires_at),
        discord_user.email.as_deref(),
        &temp_token,
    )
    .await?;

    Ok(Json(json!({ "temp_token": temp_token })))
}
