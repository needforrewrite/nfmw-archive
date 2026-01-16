use axum::Json;
use reqwest::{Client, Method, StatusCode};
use serde_json::json;

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

pub async fn exchange_code_for_token(
    req_client: &Client,
    client_id: i64,
    client_secret: &str,
    code: String,
    redirect_uri: String,
) -> Result<String, (StatusCode, Json<serde_json::Value>)> {
    let data = DiscordTokenExchangeParams {
        grant_type: "authorization_code".to_owned(),
        code,
        redirect_uri,
        client_id: client_id.to_string(),
        client_secret: client_secret.to_owned(),
    };

    // Need to turn the auth code into an access token
    let request = req_client
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

    return Ok(res.access_token);
}


#[derive(serde::Deserialize)]
pub struct DiscordUserIdResponse {
    id: i64
}


pub async fn get_user_id_from_token(client: &Client, token: &str) -> Result<i64, (StatusCode, Json<serde_json::Value>)> {
    let url = "https://discord.com/api/v10/users/@me";
    let user_agent = "NFMWAPI";

    let res = client.request(Method::GET, url)
        .bearer_auth(token)
        .header("user-agent", user_agent)
        .send()
        .await
        .map_err(|e|
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"status": format!("failed to query discord user id: {e}")})))
        )?
        .error_for_status()
        .map_err(|e|
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"status": format!("failed to query discord user id: {e}")})))
        )?
        .json::<DiscordUserIdResponse>()
        .await
        .map_err(|e|
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"status": format!("failed to query discord user id: {e}")})))
        )?;

    Ok(res.id)
}