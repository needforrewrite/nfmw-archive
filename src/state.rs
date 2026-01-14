use std::sync::Arc;

use tokio::sync::Mutex;

use crate::config::Config;

#[derive(Clone)]
pub struct State {
    pub db_pool: sqlx::PgPool,
    pub index_state: IndexState,
    pub config: Config,
    pub req_client: reqwest::Client
}

#[derive(Clone)]
pub enum IndexState {
    /// Archive is searchable
    Healthy,
    /// Archive is unsearchable; index is regenerating
    Regenerating,
    /// Index is incomplete; searching is disabled. Only happens in the event of an error when indexing.
    Unhealthy
}

pub type ThreadSafeState = Arc<Mutex<State>>;