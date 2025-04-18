use crate::db::operations::photo::create_photo;
use crate::ingest::get_img_paths::get_paths;
use crate::ingest::trait_suisai_image::SuisaiImage;
use crate::server::start_server;
use clap::{Parser, Subcommand};
use rocket::serde::json::serde_json;
use std::path::Path;
use rocket::outcome::IntoOutcome;
use crate::DB_POOL;

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
            if dry {
                println!("Running in dry mode");
            }
            
            // Iterate over files
            let paths = get_paths(Path::new(&path));
            for path in paths {
                let photo = path.to_db_entry();
                println!("{}", serde_json::to_string_pretty(&photo).unwrap());
                
                // Add to DB
                if !dry {
                    let mut conn = DB_POOL.get().expect("Failed to get connection from pool");
                    println!("Adding {} to database", photo.file_path);
                    match create_photo(&mut conn, photo) {
                        Err(e) => println!("Error: {}", e),
                        _ => println!("Success")
                    };
                }
            }
        }
    };
}
