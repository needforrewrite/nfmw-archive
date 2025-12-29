use sqlx::postgres::{PgPool, PgPoolOptions};

use crate::{config::load_config};
use crate::db::user::User;

mod crypto;
mod config;
mod db;

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
}
