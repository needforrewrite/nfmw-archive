use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};
use log::info;
use tokio::sync::Mutex;
use utoipa::OpenApi;

use crate::{config::load_config, ffi::nfmw_load, store::AssetStore};

pub mod config;
pub mod crypto;
pub mod database;
pub mod ffi;
pub mod middleware;
pub mod openapi;
pub mod route;
pub mod state;
pub mod task;
pub mod extractor;
pub mod store;

#[tokio::main]
async fn main() {
    env_logger::init();

    let config = load_config();
    let port = config.port;

    dotenvy::dotenv().ok();
    let db_url =
        env::var("DATABASE_URL").expect("DATABASE_URL must be set in environment variables");

    unsafe { nfmw_load(); };

    let db_pool = sqlx::PgPool::connect(&db_url)
        .await
        .expect("Failed to create postgres connection pool");
    info!("Connected to database at {}", db_url);

    sqlx::migrate!()
        .run(&db_pool)
        .await
        .expect("Failed to run migrations");
    info!("Migrations applied");

    let state = state::AppState {
        db_pool,
        request_client: reqwest::Client::new(),
        config: Arc::new(config.clone()),
        asset_store: Arc::new(AssetStore::new(config.bucket).unwrap())
    };

    let router = Router::new()
        .route("/", get(route::root))
        .route(
            "/auth/discord/start",
            get(route::account::oauth2::discord::init::discord_oauth_start),
        )
        .route(
            "/auth/discord/callback",
            get(route::account::oauth2::discord::callback::discord_login_callback),
        )
        .route(
            "/auth/discord/create_account",
            post(route::account::oauth2::discord::create_account::discord_create_account),
        )
        .route(
            "/auth/local/login",
            post(route::account::local::login::login_local_account),
        )
        .route(
            "/auth/local/create_account",
            post(route::account::local::create::create_local_account),
        )
        .route(
            "/auth/poll/{poll_id}",
            get(route::account::oauth2::poll::poll_oauth),
        )
        .route(
            "/api-docs/openapi.json",
            get(|| async { axum::Json(openapi::ApiDoc::openapi()) }),
        )
        .route(
            "/assets/create",
            post(route::archive::create_asset::create_asset),
        )
        .with_state(state.clone());

    task::register_tasks(state.clone());

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
