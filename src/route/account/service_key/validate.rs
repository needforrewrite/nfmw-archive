use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    crypto::hash_token,
    database::{account::{User, UserSessions}, roles::UserRole},
    extractor::service::SignedJson,
    route::error::{AppError, ErrorResponse},
    state::AppState,
};

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ValidateServiceKeyRequest {
    /// The key the player presented to the calling service.
    pub service_key: String,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ValidateServiceKeyResponse {
    pub user_id: i64,
    pub username: String,
    /// The player's role names. A caller with no database of its own gets
    /// everything it needs to make authorisation decisions from this response.
    pub roles: Vec<String>,
}

#[utoipa::path(
    post,
    path = "/auth/service-key/validate",
    operation_id = "validateServiceKey",
    request_body = ValidateServiceKeyRequest,
    responses(
        (status = 200, description = "Key redeemed; the player is who they claim to be", body = ValidateServiceKeyResponse),
        (status = 401, description = "Unsigned/badly signed request, or an invalid, expired, already-redeemed, or wrongly-scoped key", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("service_hmac" = [])),
    tag = "service-keys"
)]
pub async fn validate_service_key(
    State(state): State<AppState>,
    SignedJson { caller, body }: SignedJson<ValidateServiceKeyRequest>,
) -> Result<Json<ValidateServiceKeyResponse>, AppError> {
    let pool = &state.db_pool;

    // Scoped to the caller's own service id, so a compromised service cannot
    // redeem (or merely destroy) keys minted for another. Redeeming consumes the
    // key, making a stolen one worthless the moment the real player uses it.
    let session = UserSessions::consume_service_key(pool, &hash_token(&body.service_key), &caller.service_id)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let user = User::get_by_id(pool, session.user_id)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let roles = UserRole::get_role_names_for_user_id(pool, user.id).await?;

    Ok(Json(ValidateServiceKeyResponse {
        user_id: user.id,
        username: user.username,
        roles,
    }))
}
