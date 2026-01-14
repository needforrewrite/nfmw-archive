use axum::{Json, extract::State, http::StatusCode};
use reqwest::{Method, RequestBuilder};
use serde_json::json;

use crate::state::ThreadSafeState;

#[derive(serde::Deserialize)]
pub struct DiscordLoginPayload {
    pub code: String,
    pub redirect_uri: String,
}

#[derive(serde::Serialize)]
pub struct DiscordTokenExchangeParams {
    pub grant_type: String,
    pub code: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

#[derive(serde::Deserialize)]
pub struct DiscordTokenExchangeResponse {
    pub access_token: String,
}

#[derive(serde::Deserialize)]
pub struct DiscordUserResponse {
    pub id: u32
}

pub async fn login(
    State(state): State<ThreadSafeState>,
    Json(payload): Json<DiscordLoginPayload>,
) -> axum::response::Result<(StatusCode, Json<serde_json::Value>)> {
    if payload.code.is_empty() || payload.redirect_uri.is_empty() {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({"status": "authorization code or redirect uri was empty"})),
        ));
    }

    let data = DiscordTokenExchangeParams {
        grant_type: "authorization_code".to_owned(),
        code: payload.code,
        redirect_uri: urlencoding::encode(&payload.redirect_uri).to_string(),
        client_id: state.lock().await.config.discord.client_id.to_string(),
        client_secret: state.lock().await.config.discord.client_secret.to_string(),
    };

    let user_agent = "NFMWAPI";

    // Need to turn the auth code into an access token
    let request = state
        .lock()
        .await
        .req_client
        .request(Method::POST, "https://discord.com/api/v10/oauth2/token");
    let request = request.form(&data);
    let request = request.header("user-agent", "NFMW");
    let res = request
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": format!("Unable to send Discord request: {e}")})),
            )
        })?
        .error_for_status()
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": format!("Discord returned an error: {e}")})),
            )
        })?
        .json::<DiscordTokenExchangeResponse>()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": format!("Failed to process Discord response: {e}")})),
            )
        })?;
    
    // TODO: Get user from api /users/@me
    // then use that ID to see if it exists in the oauth2 table mapping it to an internal user ID
    // if yes: issue token, if no: respond with 404

    todo!()
}
