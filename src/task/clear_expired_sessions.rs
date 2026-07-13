use anyhow::Ok;
use log::info;

use crate::{database::account::UserSessions, state::AppState};

/// Expired archive sessions are already dropped lazily when presented, but a
/// service key that is minted and never redeemed is never presented again, so
/// nothing would otherwise collect it.
pub async fn clear_expired_sessions(state: AppState) -> Result<(), anyhow::Error> {
    info!("Starting task: Clear expired sessions");

    UserSessions::delete_expired(&state.db_pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to clear expired sessions: {}", e))?;

    info!("Task completed: Cleared expired sessions");

    Ok(())
}
