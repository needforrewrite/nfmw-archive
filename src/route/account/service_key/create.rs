use axum::{Json, extract::{Query, State}};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    crypto::{generate_base64_authentication_token, hash_token},
    database::account::UserSessions,
    extractor::auth::AuthUser,
    route::error::{AppError, ErrorResponse},
    state::AppState,
};

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateServiceKeyQuery {
    /// A service id from the server's config, e.g. `lobby`. Deliberately not a
    /// URL: the client names *who* it wants to talk to, and we tell it where
    /// that service currently lives.
    pub service: String,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ServiceDescriptor {
    pub id: String,
    pub url: String,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateServiceKeyResponse {
    /// Single-use, short-lived, and worthless to anyone but `service`. Hand it
    /// straight to that service.
    pub service_key: String,
    pub expires_in_seconds: i64,
    pub service: ServiceDescriptor,
}

#[utoipa::path(
    get,
    path = "/auth/service-key",
    operation_id = "createServiceKey",
    params(
        ("service" = String, Query, description = "Service id to scope the key to, e.g. `lobby`")
    ),
    responses(
        (status = 200, description = "Service key minted", body = CreateServiceKeyResponse),
        (status = 401, description = "Missing or invalid session", body = ErrorResponse),
        (status = 403, description = "Session is not scoped to this server", body = ErrorResponse),
        (status = 404, description = "No such service", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "service-keys"
)]
pub async fn create_service_key(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<CreateServiceKeyQuery>,
) -> Result<Json<CreateServiceKeyResponse>, AppError> {
    let pool = &state.db_pool;

    let service = state
        .config
        .service(&query.service)
        .ok_or(AppError::NotFound)?;

    // AuthUser has already established this is an archive-scoped session, which is
    // what stops a service key being traded up for keys to other services.
    let parent = UserSessions::get_by_token_hash(pool, &auth.token_hash)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let service_key = generate_base64_authentication_token();
    let ttl = state.config.service_key_ttl_seconds;

    UserSessions::create_service_key(pool, &parent, &hash_token(&service_key), &service.id, ttl).await?;

    Ok(Json(CreateServiceKeyResponse {
        service_key,
        expires_in_seconds: ttl,
        service: ServiceDescriptor {
            id: service.id.clone(),
            url: service.url.clone(),
        },
    }))
}
