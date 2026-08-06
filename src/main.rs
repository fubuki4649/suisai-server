use crate::cli::run_cli;
use crate::preflight::{check_database, check_directories};
use crate::state::AppState;
use dotenvy::dotenv;

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

    // Run directory preflight checks (creates storage dirs & database parent dirs)
    check_directories().unwrap();

    // Initialize DB
    let db = check_database().await.unwrap();

    // Initialize global state
    let state = AppState { db };
    
    run_cli(state).await;
}