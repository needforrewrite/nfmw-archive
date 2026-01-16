use axum::{Json, extract::State, http::StatusCode};
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
    route::oauth::discord::discord_token_exchange::{
        exchange_code_for_token, get_user_id_from_token,
    },
    state::ThreadSafeState,
};

#[derive(serde::Deserialize)]
pub struct DiscordLoginPayload {
    pub code: String,
    pub redirect_uri: String,
}

pub async fn login(
    State(state): State<ThreadSafeState>,
    Json(payload): Json<DiscordLoginPayload>,
) -> axum::response::Result<(StatusCode, Json<serde_json::Value>)> {
    if (&payload.code).is_empty() || (&payload.redirect_uri).is_empty() {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({"status": "authorization code or redirect uri was empty"})),
        ));
    }

    let lock = state.lock().await;

    let discord_token = exchange_code_for_token(
        &lock.req_client,
        lock.config.discord.client_id,
        &lock.config.discord.client_secret,
        payload.code,
        payload.redirect_uri,
    )
    .await?;

    let id = get_user_id_from_token(&lock.req_client, &discord_token).await?;

    let pool = &lock.db_pool;

    // lookup id in Oauth2 table, if it exists issue token. If not, reject with 404 and
    // ask user to create an account.
    let lookup = DiscordOauth2AccountEntry::lookup_discord_user_id(pool, id as i64)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": format!("Failed to lookup oauth2 entry: {e}")})),
            )
        })?;

    if let Some(l) = lookup {
        // Generate a new token
        let token = generate_base64_authentication_token();
        // Remove any existing tokens for this user
        UserToken::remove_all(l.user_id, pool).await.map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "database error on token invalidation"})),
            )
        })?;
        // Insert the new token
        UserToken::insert(pool, l.user_id, &token)
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"status": "database error on token register"})),
                )
            })?;

        let account = User::get_by_user_id(pool, l.user_id)
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"status": "database error on user lookup"})),
                )
            })?
            .ok_or((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "database error on user lookup: no user with that id"})),
            ))?;

        let username = account.username;

        Ok((
            StatusCode::OK,
            Json(
                serde_json::json!({"status": "login successful", "token": token, "username": username }),
            ),
        ))
    } else {
        // Create and add temp session token for creating account
        let s = generate_base64_authentication_token();
        DiscordOauth2TokenMapping::clear_for(pool, id)
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"status": "database error on temp token register"})),
                )
            })?;

        DiscordOauth2TokenMapping::create(pool, s.clone(), discord_token, id)
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"status": "database error on temp token register"})),
                )
            })?;

        Ok((
            StatusCode::NOT_FOUND,
            Json(json!({
                "status": "No user exists for this discord account, please use the /discord/create_account endpoint first",
                "temp_token": s
            })),
        ))
    }
}
