use log::error;

use crate::state::AppState;

pub mod clear_expired_pending_oauth;

pub fn clear_expired_pending_oauth_task(state: AppState) {
    let state = state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60 * 60)); // Run every hour
        loop {
            interval.tick().await;
            if let Err(e) = clear_expired_pending_oauth::clear_expired_pending_oauth(state.clone()).await {
                error!("Error clearing expired pending OAuth data: {}", e);
            }
        }
    });
}

pub fn register_tasks(state: AppState) {
    clear_expired_pending_oauth_task(state);
}