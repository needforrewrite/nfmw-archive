use axum::{Json, extract::State};
use serde::Serialize;
use utoipa::ToSchema;

use crate::{
    crypto::generate_base64_authentication_token,
    database::account::oauth2::session::OauthSession,
    route::error::{AppError, ErrorResponse},
    state::ThreadSafeState,
};

#[derive(Serialize, ToSchema)]
pub struct DiscordInitResponse {
    pub url: String,
    pub poll_id: String,
}

#[utoipa::path(
    get,
    path = "/auth/discord/start",
    responses(
        (status = 200, description = "Discord OAuth2 authorization URL and poll ID", body = DiscordInitResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    tag = "discord-oauth"
)]
pub async fn handler(State(state): State<ThreadSafeState>) -> Result<Json<DiscordInitResponse>, AppError> {
    let (pool, config) = {
        let g = state.lock().await;
        (g.db_pool.clone(), g.config.clone())
    };

    let oauth_state = generate_base64_authentication_token();
    let session = OauthSession::insert_base(&pool, "discord", &oauth_state).await?;

    let redirect_uri = urlencoding::encode(&config.discord.redirect_uri).into_owned();
    let state_encoded = urlencoding::encode(&oauth_state).into_owned();

    let url = format!(
        "https://discord.com/api/oauth2/authorize?client_id={}&redirect_uri={}&response_type=code&scope=identify%20email&state={}",
        config.discord.client_id, redirect_uri, state_encoded
    );

    Ok(Json(DiscordInitResponse {
        url,
        poll_id: session.poll_id,
    }))
}
