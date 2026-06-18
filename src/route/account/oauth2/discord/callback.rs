use axum::extract::{Query, State};
use log::debug;
use serde::Deserialize;
use time::{Duration, OffsetDateTime};
use utoipa::IntoParams;

use crate::{
    crypto::{generate_base64_authentication_token, hash_token},
    database::account::{
        OauthIdentity, User, UserSessions,
        oauth2::{pendingregistration::PendingOauthRegistration, session::OauthSession},
    },
    route::error::AppError,
    state::AppState,
};
    
#[derive(Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct CallbackQuery {
    /// OAuth2 authorization code from Discord
    code: String,
    /// OAuth2 state parameter for CSRF protection
    state: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DiscordTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: i64,
}

#[derive(Deserialize, Debug)]
struct DiscordUser {
    id: String,
    email: Option<String>,
}

pub async fn discord_login_callback(
    State(state): State<AppState>,
    Query(params): Query<CallbackQuery>,
) -> Result<String, AppError> {
    let (pool, config, client) = {
        (
            state.db_pool.clone(),
            state.config.clone(),
            state.request_client.clone(),
        )
    };

    let session = OauthSession::get_by_state(&pool, &params.state)
        .await?
        .ok_or_else(|| AppError::BadRequest("invalid state".to_string()))?;

    debug!(
        "Discord OAuth2 callback received for state: {}",
        params.state
    );

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

    debug!(
        "Discord OAuth2 token response received for state: {} - {:?}",
        params.state, token_resp
    );

    let discord_user = client
        .get("https://discord.com/api/users/@me")
        .bearer_auth(&token_resp.access_token)
        .send()
        .await
        .map_err(|e| AppError::Other(e.into()))?
        .json::<DiscordUser>()
        .await
        .map_err(|e| AppError::Other(e.into()))?;

    debug!(
        "Discord user info retrieved for state: {} - {:?}",
        params.state, discord_user
    );

    let token_expires_at = OffsetDateTime::now_utc() + Duration::seconds(token_resp.expires_in);

    if let Some(identity) =
        OauthIdentity::get_by_provider_subject(&pool, "discord", &discord_user.id).await?
    {
        let user = User::get_by_id(&pool, identity.user_id)
            .await?
            .ok_or_else(|| AppError::Other(anyhow::anyhow!("orphaned oauth identity")))?;

        let session_token = generate_base64_authentication_token();
        let session_token_hash = hash_token(&session_token);

        UserSessions::upsert(&pool, user.id, &session_token_hash, "discord").await?;

        session.set_result(&pool, "login", &session_token).await?;

        debug!(
            "Existing user logged in via Discord OAuth2 for state: {} - user_id: {}",
            params.state, user.id
        );

        return Ok("Login successful. You can now return to the application.".to_string());
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

    session
        .set_result(&pool, "create_account", &temp_token)
        .await?;

    debug!(
        "New user registration initiated via Discord OAuth2 for state: {} - temp_token: {}",
        params.state, temp_token
    );

    Ok("Registration started. You can now return to the application.".to_string())
}
