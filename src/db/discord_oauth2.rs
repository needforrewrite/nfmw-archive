use sqlx::types::time::PrimitiveDateTime;

#[derive(sqlx::FromRow)]
pub struct DiscordOauth2AccountEntry {
    pub id: i32,
    pub user_id: i32,
    pub discord_user_id: i64,
    pub created_at: PrimitiveDateTime,
}
impl DiscordOauth2AccountEntry {

}