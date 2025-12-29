use std::fmt::Display;
use subtle::ConstantTimeEq;

#[derive(sqlx::FromRow, Debug)]
pub struct User {
    /// ID sequentially assigned by DB
    pub id: i32,
    pub username: String,
    pub phash: String,
    pub psalt: Vec<u8>,
    /// Indicates whether the user must change their password on next login
    pub must_change_password: bool,
}
impl User {
    pub fn new_from_password(username: String, password: String, must_change_password: bool) -> Self {
        let psalt = crate::crypto::generate_salt().to_vec();
        let phash = crate::crypto::hash_password(&password, &psalt);
        User {
            // ID assigned by DB on insert
            id: 0,
            username,
            phash,
            psalt,
            must_change_password,
        }
    }

    pub fn update_password(&mut self, new_password: String, must_change_password: bool) {
        let psalt = crate::crypto::generate_salt().to_vec();
        let phash = crate::crypto::hash_password(&new_password, &psalt);
        self.phash = phash;
        self.psalt = psalt;
        self.must_change_password = must_change_password;
    }

    pub async fn insert_or_update(&self, pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO users (username, phash, psalt, must_change_password)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (username) DO UPDATE
            SET username = EXCLUDED.username,
                phash = EXCLUDED.phash,
                psalt = EXCLUDED.psalt,
                must_change_password = EXCLUDED.must_change_password
            "#
        )
        .bind(&self.username)
        .bind(&self.phash)
        .bind(&self.psalt)
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
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, phash, psalt
            FROM users
            WHERE username = $1
            "#
        )
        .bind(username)
        .fetch_optional(pool)
        .await?;
        Ok(user)
    }

    pub async fn get_by_username_password(pool: &sqlx::PgPool, username: &str, password: &str) -> Result<Option<User>, sqlx::Error> {
        if let Some(user) = User::get_by_username(pool, username).await? {
            let computed_hash = crate::crypto::hash_password(password, &user.psalt);
            if computed_hash.as_bytes().ct_eq(user.phash.as_bytes()).unwrap_u8() == 1 {
                return Ok(Some(user));
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