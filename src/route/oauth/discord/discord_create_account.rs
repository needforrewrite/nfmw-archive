use axum::{Json, extract::State};
use reqwest::StatusCode;
use serde_json::json;

use crate::{
    crypto::generate_base64_authentication_token, db::{discord_oauth2::DiscordOauth2AccountEntry, token::UserToken, user::User}, route::oauth::discord::discord_token_exchange::{
        exchange_code_for_token, get_user_id_from_token,
    }, state::ThreadSafeState
};

pub struct DiscordCreateAccountPayload {
    pub code: String,
    pub redirect_uri: String,
    pub username: String,
}

pub async fn create_account(
    State(state): State<ThreadSafeState>,
    Json(payload): Json<DiscordCreateAccountPayload>,
) -> axum::response::Result<(StatusCode, Json<serde_json::Value>)> {
    if (&payload.code).is_empty()
        || (&payload.redirect_uri).is_empty()
        || (&payload.username).is_empty()
    {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({"status": "a body parameter was empty"})),
        ));
    }

    // TODO: validate username

    let token = exchange_code_for_token(
        &state.lock().await.req_client,
        state.lock().await.config.discord.client_id,
        &state.lock().await.config.discord.client_secret,
        payload.code,
        payload.redirect_uri,
    )
    .await?;

    let discord_id = get_user_id_from_token(&state.lock().await.req_client, &token).await?;

    let pool = &state.lock().await.db_pool;

    // lookup id in Oauth2 table, if it exists already then reject.
    let lookup = DiscordOauth2AccountEntry::lookup_discord_user_id(pool, discord_id as i64)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": format!("Failed to lookup oauth2 entry: {e}")})),
            )
        })?;

    if let Some(u) = lookup {
        return Ok((
            StatusCode::CONFLICT,
            Json(
                json!({"status": format!("Discord account already tied to Discord user {}", u.discord_user_id)}),
            ),
        ));
    }

    let user_id = User::create_oauth(pool, &payload.username)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": format!("failed to register user on user insert: {e}")})),
            )
        })?;

    DiscordOauth2AccountEntry::create_oauth2_link(pool, user_id, discord_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": format!("failed to register user on oauth link: {e}")})),
            )
        })?;

    // login on same call to prevent user from having to log into discord again
    let token = generate_base64_authentication_token();
    UserToken::insert(pool, user_id, &token)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "database error on token register"})),
            )
        })?;

    Ok((StatusCode::OK, Json(json!({"status": "account created", "token": token}))))
}
