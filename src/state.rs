use std::sync::Arc;
use tokio::sync::Mutex;
use crate::config::Config;

#[derive(Clone)]
pub struct State {
    pub db_pool: sqlx::PgPool,
    pub request_client: reqwest::Client,
    pub config: Config
}

pub type ThreadSafeState = Arc<Mutex<State>>;