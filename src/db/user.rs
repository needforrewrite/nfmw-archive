use std::fmt::Display;
use sqlx::types::time::{PrimitiveDateTime};
use subtle::ConstantTimeEq;

#[derive(sqlx::FromRow, Debug)]
pub struct User {
    /// ID sequentially assigned by DB
    pub id: i32,
    pub username: String,
    /// These can be null if using an oauth account.
    pub phash: Option<String>,
    pub psalt: Option<Vec<u8>>,
    pub created_at: Option<PrimitiveDateTime>,
    /// Indicates whether the user must change their password on next login
    pub must_change_password: Option<bool>,
}
impl User {
    pub fn new_local_from_password(username: String, password: String, must_change_password: Option<bool>) -> Self {
        let psalt = crate::crypto::generate_salt().to_vec();
        let phash = crate::crypto::hash_password(&password, &psalt);
        User {
            // ID assigned by DB on insert
            id: 0,
            username,
            phash: Some(phash),
            psalt: Some(psalt),
            must_change_password,
            /// Set on insert
            created_at: None
        }
    }

    /// Password must be at least 8 characters, one uppercase letter, one lowercase letter, one digit.
    /// Cannot contain spaces and must be ASCII.
    pub fn validate_password(password: &str) -> bool {
        password.len() >= 8 
            && password.chars().any(|c| c.is_uppercase()) 
            && password.chars().any(|c| c.is_lowercase()) 
            && password.chars().any(|c| c.is_digit(10))
            && !password.chars().any(|c| c.is_whitespace())
            && password.chars().all(|c| c.is_ascii())
    }

    /// Only does anything if this is a local account. Returns None if the account is not local.
    pub fn update_local_password(&mut self, new_password: String, must_change_password: bool) -> Option<()> {
        if self.phash.is_some() && self.psalt.is_some() {
            let psalt = crate::crypto::generate_salt().to_vec();
            let phash = crate::crypto::hash_password(&new_password, &psalt);
            self.phash = Some(phash);
            self.psalt = Some(psalt);
            self.must_change_password = Some(must_change_password);
            return Some(())
        }
        None
    }

    pub async fn check_username_exists(&self, pool: &sqlx::PgPool) -> Result<bool, sqlx::Error> {
        let record = sqlx::query_scalar!( 
            "SELECT COUNT(*) FROM users WHERE username = $1",
            self.username
        )
        .fetch_one(pool)
        .await?;

        Ok(record.unwrap_or(0) > 0)
    }

    pub async fn insert_or_update(&self, pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO users (username, phash, psalt, must_change_password)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (username) DO UPDATE
            SET username = EXCLUDED.username,
                phash = EXCLUDED.phash,
                psalt = EXCLUDED.psalt,
                must_change_password = EXCLUDED.must_change_password
            "#,
            self.username,
            self.phash,
            self.psalt,
            false
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Retrieve a user by username
    /// 
    /// Note: This does NOT verify the password
    /// 
    /// We don't mark this as `pub` because we want to enforce password checking via `get_by_username_password`
    async fn get_by_username(pool: &sqlx::PgPool, username: &str) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as!(User, 
            r#"
            SELECT id, username, phash, psalt, created_at, must_change_password
            FROM users
            WHERE username = $1
            "#,
            username
        )
        .fetch_optional(pool)
        .await?;
        Ok(user)
    }

    pub async fn get_by_username_and_local_password(pool: &sqlx::PgPool, username: &str, password: &str) -> Result<Option<User>, sqlx::Error> {
        if let Some(user) = User::get_by_username(pool, username).await? {
            if let Some(ref hash) = user.phash.clone() && let Some(ref salt) = user.psalt {
                let computed_hash = crate::crypto::hash_password(password, salt);
                if computed_hash.as_bytes().ct_eq(hash.as_bytes()).into() {
                    return Ok(Some(user));
                }
            } else {
                // Likely Oauth2 account
                return Ok(None)
            }
        }
        Ok(None)
    }
}
impl Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "User {{ id: {}, username: {} }}", self.id, self.username)
    }
}