use crate::cli::run_cli;
use crate::preflight::check_directories;
use diesel::r2d2::ConnectionManager;
use diesel::MysqlConnection;
use dotenvy::dotenv;
use std::env;
use std::sync::LazyLock;

mod db;
mod endpoints;
mod cli;
mod ingest;
mod _utils;
mod preflight;
mod models;

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


#[rocket::main]
async fn main() {
    dotenv().ok();
    
    // Run preflight checks
    check_directories().unwrap();
    
    run_cli().await;
}