use anyhow::anyhow;
use argon2::{
    Argon2, PasswordHasher,
    password_hash::{SaltString, rand_core::OsRng},
};
use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    database::account::{LocalCredentials, User},
    route::{
        account::validate_username,
        error::{AppError, ErrorResponse},
    },
    state::AppState,
};

pub fn validate_local_password(password: &str) -> Result<(), String> {
    if password.len() < 8 {
        return Err("Password must be at least 8 characters long".to_string());
    }

    if !password.chars().any(|c| c.is_ascii_uppercase()) {
        return Err("Password must contain at least one uppercase letter".to_string());
    }

    if !password.chars().any(|c| c.is_ascii_lowercase()) {
        return Err("Password must contain at least one lowercase letter".to_string());
    }

    if !password.chars().any(|c| c.is_ascii_digit()) {
        return Err("Password must contain at least one digit".to_string());
    }

    if !password.chars().any(|c| !c.is_ascii_alphanumeric()) {
        return Err("Password must contain at least one special character".to_string());
    }

    Ok(())
}

#[derive(Deserialize, ToSchema)]
pub struct CreateLocalAccountRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct CreateLocalAccountResponse {
    pub username: String,
}

#[utoipa::path(
    post,
    operation_id = "createLocalAccount",
    path = "/auth/local/create_account",
    request_body = CreateLocalAccountRequest,
    responses(
        (status = 200, description = "Account created", body = CreateLocalAccountResponse),
        (status = 400, description = "Validation error or username already taken", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    tag = "local-auth"
)]
pub async fn create_local_account(
    State(state): State<AppState>,
    Json(body): Json<CreateLocalAccountRequest>,
) -> Result<Json<CreateLocalAccountResponse>, AppError> {
    let pool = &state.db_pool;

    validate_username(&body.username).map_err(|e| AppError::BadRequest(e))?;
    validate_local_password(&body.password).map_err(|e| AppError::BadRequest(e))?;

    if User::username_taken(&pool, &body.username).await? {
        return Err(AppError::BadRequest("username already taken".to_string()));
    }

    let user = User::create(&pool, &body.username, None)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(ref dbe) = e {
                // 23505 is the Postgres error code for unique_violation, which we can use to detect if the username is already taken.
                if dbe.code().as_deref() == Some("23505") {
                    return AppError::BadRequest("username already taken".to_string());
                }
            }
            AppError::Database(e)
        })?;

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let argon2_hash = argon2
        .hash_password(body.password.as_bytes(), &salt)
        .map_err(|e| AppError::Other(anyhow!(e.to_string())))?
        .to_string();

    let local_cred_res = LocalCredentials::insert(&pool, user.id, &argon2_hash)
        .await
        .map_err(|e| AppError::Database(e));
    // On error we need to remove the inserted user.
    if let Err(e) = local_cred_res {
        user.delete(&pool)
            .await
            .map_err(|e| AppError::Database(e))?;
        return Err(e);
    }

    Ok(Json(CreateLocalAccountResponse {
        username: user.username,
    }))
}
