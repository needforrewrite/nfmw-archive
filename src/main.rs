use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};
use log::info;
use tokio::sync::Mutex;
use utoipa::OpenApi;

use crate::config::load_config;

pub mod config;
pub mod crypto;
pub mod database;
pub mod ffi;
pub mod middleware;
pub mod openapi;
pub mod route;
pub mod state;

#[tokio::main]
async fn main() {
    env_logger::init();

    let config = load_config();
    let port = config.port;

    dotenvy::dotenv().ok();
    let db_url =
        env::var("DATABASE_URL").expect("DATABASE_URL must be set in environment variables");

    let db_pool = sqlx::PgPool::connect(&db_url)
        .await
        .expect("Failed to create postgres connection pool");
    info!("Connected to database at {}", db_url);

    sqlx::migrate!()
        .run(&db_pool)
        .await
        .expect("Failed to run migrations");
    info!("Migrations applied");

    let state = Arc::new(Mutex::new(state::State {
        db_pool,
        request_client: reqwest::Client::new(),
        config,
    }));

    let router = Router::new()
        .route("/", get(route::root))
        .route(
            "/auth/discord/start",
            get(route::account::oauth2::discord::init::handler),
        )
        .route(
            "/auth/discord/callback",
            get(route::account::oauth2::discord::callback::handler),
        )
        .route(
            "/auth/discord/create_account",
            post(route::account::oauth2::discord::create_account::handler),
        )
        .route(
            "/auth/local/login",
            post(route::account::local::login::handler),
        )
        .route(
            "/auth/local/create_account",
            post(route::account::local::create::handler),
        )
        .route(
            "/auth/poll/{poll_id}",
            get(route::account::oauth2::poll::handle),
        )
        .route(
            "/api-docs/openapi.json",
            get(|| async { axum::Json(openapi::ApiDoc::openapi()) }),
        )
        .with_state(state);

    let router = router.layer(axum::middleware::from_fn(middleware::debug_logger));

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    info!("Server listening on {}", addr);

    axum::serve(listener, router)
        .await
        .expect("Failed to start server");
}
