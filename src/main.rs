use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};
use sqlx::postgres::{PgPool, PgPoolOptions};
use tokio::sync::Mutex;

use crate::archive::index::index_archive;
use crate::archive::parse::parse_line;
use crate::config::load_config;
use crate::db::user::User;

mod archive;
mod config;
mod crypto;
mod db;
mod ffi;
mod route;
mod state;

#[tokio::main]
async fn main() {
    let config = load_config();
    println!("Filestore path: {}", config.filestore);

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(config.database.connection_string().as_str())
        .await
        .expect("Failed to create Postgres connection pool");

    println!(
        "Connected to database on {}:{}",
        config.database.host, config.database.port
    );

    // TODO: for now, fully regenerate archive db on restart,
    // but in future we can probably do some sanity checks
    // to see if this is necessary as the size of the archive grows
    let state = Arc::new(Mutex::new(state::State {
        db_pool: pool,
        index_state: state::IndexState::Regenerating,
        config: load_config()
    }));


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
            post(route::create_account::create_account),
        )
        .route("/login", post(route::login::login))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, axum_router)
        .await
        .expect("Failed to start server");
}
