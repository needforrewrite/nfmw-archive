use axum::{Json, extract::State, http::StatusCode};
use serde_json::json;

use crate::{
    crypto::generate_base64_authentication_token, db::{discord_oauth2::DiscordOauth2AccountEntry, token::UserToken}, route::oauth::discord::discord_token_exchange::{
        exchange_code_for_token, get_user_id_from_token,
    }, state::ThreadSafeState
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

    let token = exchange_code_for_token(
        &state.lock().await.req_client,
        state.lock().await.config.discord.client_id,
        &state.lock().await.config.discord.client_secret,
        payload.code,
        payload.redirect_uri,
    )
    .await?;

    let id = get_user_id_from_token(&state.lock().await.req_client, &token).await?;

    let pool = &state.lock().await.db_pool;

    // lookup id in Oauth2 table, if it exists issue token. If not, reject with 404 and
    // ask user to create an account.
    let lookup =
        DiscordOauth2AccountEntry::lookup_discord_user_id(pool, id as i64)
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
        UserToken::remove_all(l.user_id, pool).await
            .map_err(|_| (
                StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"status": "database error on token invalidation"}))
            ))?;
        // Insert the new token
        UserToken::insert(pool, l.user_id, &token).await
            .map_err(|_| (
                StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"status": "database error on token register"}))
            ))?;

        
        Ok((
            StatusCode::OK,
            Json(serde_json::json!({"status": "login successful", "token": token})),
        ))
    } else {
        Ok((
            StatusCode::NOT_FOUND,
            Json(
                json!({"status": "No user exists for this discord account, please use the /discord/create_account endpoint first"}),
            ),
        ))
    }
}
