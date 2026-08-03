use crate::cli::run_cli;
use crate::preflight::check_directories;
use diesel::r2d2::ConnectionManager;
use diesel::MysqlConnection;
use dotenvy::dotenv;
use std::env;
use std::sync::LazyLock;
use crate::state::AppState;

mod db;
mod endpoints;
mod cli;
mod ingest;
mod _utils;
mod preflight;
mod models;
mod fs_operations;
mod state;

type Pool = r2d2::Pool<ConnectionManager<MysqlConnection>>;

static DB_POOL: LazyLock<Pool> = LazyLock::new(|| {
    establish_connection_pool()
});


/// Function to establish database connection pool
fn establish_connection_pool() -> Pool {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<MysqlConnection>::new(database_url);
    Pool::builder()
        .build(manager)
        .expect("Failed to create pool")
}


#[tokio::main]
async fn main() {
    dotenv().ok();
    let db = sea_orm::Database::connect(env::var("DATABASE_URL").expect("DATABASE_URL must be set")).await.unwrap();

    // Initialize global state
    let state = AppState { db };
    
    // Run preflight checks
    check_directories().unwrap();

    // TODO: REPLACE CLI
    run_cli(state).await;
}