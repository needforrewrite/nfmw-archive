use anyhow::Ok;
use log::info;

use crate::{database::account::oauth2::{pendingregistration::PendingOauthRegistration, session::OauthSession}, state::ThreadSafeState};

pub async fn clear_expired_pending_oauth(state: ThreadSafeState) -> Result<(), anyhow::Error> {
    info!("Starting task: Clear expired pending OAuth registrations and sessions");

    let pool = state.lock().await.db_pool.clone();

    PendingOauthRegistration::delete_expired(&pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to clear expired pending OAuth registrations: {}", e))?;

    OauthSession::delete_expired(&pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to clear expired OAuth sessions: {}", e))?;

    info!("Task completed: Cleared expired pending OAuth registrations and sessions");

    Ok(())
}