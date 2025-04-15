#[macro_use]
extern crate lazy_static;
use crate::cli::run_cli;
use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;
use dotenvy::dotenv;
use std::env;

mod db;
mod endpoints;
mod cli;
mod server;
mod ingest;
mod _utils;

type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;


lazy_static! {
    static ref DB_POOL: Pool = establish_connection_pool();
}

// Function to establish database connection pool
fn establish_connection_pool() -> Pool {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder()
        .build(manager)
        .expect("Failed to create pool")
}


#[rocket::main]
async fn main() {
    
    run_cli().await;

}