use axum::{Json, extract::State};
use reqwest::StatusCode;
use serde_json::json;

use crate::{
    crypto::generate_base64_authentication_token,
    db::{
        oauth2::{
            discord_oauth2::DiscordOauth2AccountEntry, oauth2_temptoken::DiscordOauth2TokenMapping,
        },
        token::UserToken,
        user::User,
    },
    route::oauth2::discord::discord_token_exchange::{
        exchange_code_for_token, get_user_id_from_token,
    },
    state::ThreadSafeState,
};

#[derive(serde::Deserialize)]
pub struct DiscordCreateAccountPayload {
    pub temp_token: String,
    pub username: String,
}

pub async fn create_account(
    State(state): State<ThreadSafeState>,
    Json(payload): Json<DiscordCreateAccountPayload>,
) -> axum::response::Result<(StatusCode, Json<serde_json::Value>)> {
    if (&payload.temp_token).is_empty() || (&payload.username).is_empty() {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({"status": "a body parameter was empty"})),
        ));
    }

    let u = User::validate_username(&payload.username);
    if let Some(u) = u {
        return Ok(
            (
                StatusCode::BAD_REQUEST,
                Json(json!({"status": u}))
            )
        )
    }

    let lock = state.lock().await;
    let pool = &lock.db_pool;

    let discord_info = DiscordOauth2TokenMapping::get_from_session_token(pool, payload.temp_token)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": format!("Failed to lookup temp token entry: {e}")})),
            )
        })?;

    if discord_info.is_none() {
        return Ok((
            StatusCode::UNAUTHORIZED,
            Json(json!({"status": "temp_token was invalid"}))
        ));
    }

    let discord_info = discord_info.unwrap();
    let discord_id = discord_info.discord_user_id;

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

    Ok((
        StatusCode::OK,
        Json(json!({"status": "account created", "token": token})),
    ))
}
