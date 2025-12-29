use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use sqlx::postgres::{PgPool, PgPoolOptions};

use crate::{config::load_config};
use crate::db::user::User;

mod crypto;
mod config;
mod db;
mod state;
mod route;

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

    let state = Arc::new(state::State { db_pool: pool });

    let axum_router = Router::<()>::new()
        .route("/", get(route::root));

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, axum_router)
        .await
        .expect("Failed to start server");

    println!("Listening on {}", addr);
}
