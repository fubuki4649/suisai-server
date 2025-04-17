use std::path::Path;
use clap::{Parser, Subcommand};
use rocket::serde::json::serde_json;
use crate::ingest::get_img_paths::get_paths;
use crate::ingest::trait_suisai_image::SuisaiImage;
use crate::server::start_server;

#[derive(Parser)]
#[command(name = "suisai", version = "1.0", about = "Backend server for suisai")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
#[command(rename_all = "kebab-case")]
enum Commands {
    #[command(about = "Start the web server used by the frontend")]
    StartServer {},
    #[command(about = "Ingest camera raws from a directory")]
    Ingest {
        #[arg(help = "Path to a directory containing camera raws")]
        path: String,
        #[arg(long, help = "Run ingestion in dry mode (no actual changes to DB or filesystem)")]
        dry: bool,
    }
}

pub async fn run_cli() {
    let cli = Cli::parse();

    match cli.command {
        Commands::StartServer {} => start_server().await,
        Commands::Ingest {path, dry} => {
            println!("Ingesting files from: {}", path);

            println!("Running in dry mode");
            let paths = get_paths(Path::new(&path));
            for path in paths {
                println!("{}", serde_json::to_string_pretty(&path.to_db_entry()).unwrap())
            }
        }
    };
}
