use std::fmt::{Display, write};

use sqlx::PgPool;

#[derive(Debug)]
pub enum TagOwnershipError {
    UserIdCannotAssignTag,
    TagDoesNotExist,
    Sqlx(sqlx::Error),
    Other(String)
}
impl Display for TagOwnershipError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UserIdCannotAssignTag => write!(f, "You don't have permission to assign this tag to items."),
            Self::TagDoesNotExist => write!(f, "That tag doesn't exist"),
            Self::Sqlx(e) => write!(f, "SQL error: {e}"),
            Self::Other(s) => write!(f, "Generic tag lookup error: {s}")
        }
    }
}
impl std::error::Error for TagOwnershipError {}

pub async fn add_owner_user_id_to_tag_id(pool: &PgPool, tag_id: i32, user_id: i32) -> Result<(), TagOwnershipError> {
    todo!()
}

pub async fn user_can_assign_tag(pool: &PgPool, tag_id: i32, user_id: i32) -> Result<bool, TagOwnershipError>  {
    todo!()
}