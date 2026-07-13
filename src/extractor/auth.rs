use axum::{
    Json, extract::{FromRef, FromRequestParts}, http::{StatusCode, request::Parts}, response::{IntoResponse, Response}
};
use serde_json::json;
use sqlx::PgPool;

use crate::{crypto::hash_token, database::account::{ARCHIVE_AUDIENCE, UserSessions}};

/// Returned to any handler that requires authentication.
/// Contains only what handlers typically need — expand as required.
pub struct AuthUser {
    pub user_id: i64,
    /// The session this request authenticated with. Carried so a handler can mint
    /// a service key descended from it; see `route::account::service_key`.
    pub token_hash: String,
}

/// Rejection type returned when auth fails.
pub struct AuthError(StatusCode, &'static str);

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        (self.0, Json(json!({ "error": self.1 }))).into_response()
    }
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    PgPool: axum::extract::FromRef<S>,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let raw_token = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or(AuthError(StatusCode::UNAUTHORIZED, "missing or malformed Authorization header"))?;

        let token_hash = hash_token(raw_token);

        let pool = PgPool::from_ref(state);

        let session = UserSessions::get_by_token_hash(&pool, &token_hash)
            .await
            .map_err(|_| AuthError(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?
            .ok_or(AuthError(StatusCode::UNAUTHORIZED, "invalid or expired session"))?;

        // A service key is handed to a third party. It names them as its audience,
        // and it must buy exactly nothing here — otherwise a lobby holding a
        // player's key could act as that player against this server.
        if session.audience != ARCHIVE_AUDIENCE {
            return Err(AuthError(StatusCode::FORBIDDEN, "token is not scoped to this service"));
        }

        if session.expires_at < time::OffsetDateTime::now_utc() {
            session.delete(&pool).await
                .map_err(|_| AuthError(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;
            return Err(AuthError(StatusCode::UNAUTHORIZED, "invalid or expired session"));
        }

        UserSessions::touch_session(&pool, &token_hash)
            .await
            .map_err(|_| AuthError(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

        Ok(AuthUser { user_id: session.user_id, token_hash })
    }
}
