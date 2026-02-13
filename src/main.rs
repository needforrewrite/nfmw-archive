use std::env::{self, VarError};
use std::sync::Arc;

use axum::Router;
use axum::routing::{get, patch, post};
use reqwest::Client;
use sqlx::postgres::{PgPool, PgPoolOptions};
use tokio::sync::Mutex;

use crate::archive::ensure_default_dirs_exist;
use crate::archive::index::index_archive;
use crate::archive::parse::parse_line;
use crate::config::load_config;
use crate::db::user::User;
use crate::ffi::{SimulateTimeTrialArgs, nfmw_simulate_tt};
use crate::route::oauth2::discord;

mod archive;
mod config;
mod crypto;
mod db;
mod ffi;
mod route;
mod state;
mod tt;

#[tokio::main]
async fn main() {
    let config = load_config();
    println!("Filestore path: {}", config.filestore);

    let _ = dotenvy::dotenv();

    let db_url = env::var("DATABASE_URL").or_else(|_| {
        env::var("DATABASE_URL")
    }).unwrap();

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&db_url)
        .await
        .expect("Failed to create Postgres connection pool");

    println!(
        "Connected to database on {db_url}"
    );

    // TODO: for now, fully regenerate archive db on restart,
    // but in future we can probably do some sanity checks
    // to see if this is necessary as the size of the archive grows
    let state = Arc::new(Mutex::new(state::State {
        db_pool: pool,
        index_state: state::IndexState::Regenerating,
        config: load_config(),
        req_client: Client::new()
    }));

    ensure_default_dirs_exist(&state.lock().await.config.filestore).unwrap();

    let c = state.clone();
    let idx = tokio::spawn(async {
        let index_result = index_archive(c).await;
        if let Err(e) = index_result {
            eprintln!("Failed to index database! {e}")
        }
    }).await;
    if let Err(e) = idx {
        eprintln!("Failed to index database! {e}")
    }

    let axum_router = Router::new()
        .route("/", get(route::root))
        .route(
            "/create_account",
            post(route::local_create_account::create_account),
        )
        .route("/login", post(route::local_login::login))
        .route("/archive/create_item", patch(route::archive::create_item::create_archive_item))
        .route("/discord/login", post(discord::discord_login::login))
        .route("/discord/create_account", post(discord::discord_create_account::create_account))
        .route("/tt/search", post(route::tt::search_tt::search_tt))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, axum_router)
        .await
        .expect("Failed to start server");
}
