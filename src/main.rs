use std::sync::Arc;

use axum::{Router, routing::{get, post}};
use tokio::sync::Mutex;

use crate::config::load_config;

pub mod ffi;
pub mod database;
pub mod route;
pub mod state;
pub mod config;
pub mod crypto;

#[tokio::main]
async fn main() {
    env_logger::init();

    let config = load_config();
    let port = config.port;

    let db_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in environment variables");

    let db_pool = sqlx::PgPool::connect(&db_url).await.expect("Failed to create postgres connection pool");

    let state = Arc::new(Mutex::new(state::State {
        db_pool,
        request_client: reqwest::Client::new(),
        config
    }));

    let router = Router::new()
        .route("/", get(route::root))
        .route("/auth/discord/start", get(route::account::oauth2::discord::init::handler))
        .route("/auth/discord/callback", get(route::account::oauth2::discord::callback::handler))
        .route("/auth/discord/create_account", post(route::account::oauth2::discord::create::handler))
        .route("/auth/local/login", post(route::account::local::login::handler))
        .route("/auth/local/create_account", post(route::account::local::create::handler))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, router)
        .await
        .expect("Failed to start server");
}