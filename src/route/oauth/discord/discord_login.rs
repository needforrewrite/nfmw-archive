use axum::{Json, extract::State, http::StatusCode};
use reqwest::{Method, RequestBuilder};
use serde_json::json;

use crate::{
    route::oauth::discord::discord_token_exchange::{exchange_code_for_token, get_user_id_from_token}, state::ThreadSafeState,
};

#[derive(serde::Deserialize)]
pub struct DiscordLoginPayload {
    pub code: String,
    pub redirect_uri: String,
}

#[derive(serde::Deserialize)]
pub struct DiscordUserResponse {
    pub id: u32,
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

    let id = get_user_id_from_token(&token).await?;

    // TODO: lookup id in Oauth2 table, if it exists issue token. If not, reject with 404 and
    // ask user to create an account.

    todo!()
}
