use std::sync::Arc;
use axum::extract::FromRef;
use tokio::sync::Mutex;
use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: sqlx::PgPool,
    pub request_client: reqwest::Client,
    pub config: Arc<Config>
}

impl FromRef<AppState> for sqlx::PgPool {
    fn from_ref(state: &AppState) -> Self {
        state.db_pool.clone()
    }
}