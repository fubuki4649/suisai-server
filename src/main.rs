use crate::cli::run_cli;
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
    run_cli().await;
}