use crate::cli::run_cli;
use crate::preflight::check_directories;
use crate::state::AppState;
use dotenvy::dotenv;
use std::env;

mod db;
mod endpoints;
mod cli;
mod ingest;
mod _utils;
mod preflight;
mod models;
mod fs_operations;
mod state;



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