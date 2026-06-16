use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("resource not found")]
    NotFound,

    #[error("invalid input: {0}")]
    BadRequest(String),

    #[error("unauthorized")]
    Unauthorized,

    #[error(transparent)]
    Database(#[from] sqlx::Error),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("provided content or asset is larger than the maximum allowed of {0}kB")]
    ContentTooLarge(u32),

    #[error("forbidden: {0}")]
    Forbidden(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl AppError {
    fn status(&self) -> StatusCode {
        match self {
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::ContentTooLarge(_) => StatusCode::PAYLOAD_TOO_LARGE,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::Database(_) | AppError::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status();

        // Log the full error server-side, but don't leak internals to clients.
        if status == StatusCode::INTERNAL_SERVER_ERROR {
            log::error!("internal error: {:?}", self);
        }

        let message = match status {
            StatusCode::INTERNAL_SERVER_ERROR => "internal server error".to_string(),
            _ => self.to_string(),
        };

        let error_response = ErrorResponse { error: message.clone() };
        let body = Json(serde_json::to_string(&error_response).unwrap());
        (status, body).into_response()
    }
}