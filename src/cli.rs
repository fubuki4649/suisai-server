use crate::endpoints::main::start_webserver;
use crate::ingest::ingest::ingest;
use clap::{Parser, Subcommand};
use rocket::tokio;

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
    StartServer {
    },
    #[command(about = "Ingest camera raws from a directory")]
    Ingest {
        #[arg(help = "Path to a directory containing camera raws")]
        source: String,
        #[arg(long, help = "Run ingestion in dry mode (no actual changes to DB or filesystem)")]
        dry: bool,
        #[arg(long, help = "Move instead of move files to their new destination (default behavior is copy)")]
        no_preserve: bool,
    }
}

pub async fn run_cli() {
    let cli = Cli::parse();

    match cli.command {
        Commands::StartServer { } => {
            // Spawn the endpoints on its own async task
            let web_handle = tokio::spawn(async move {
                start_webserver().await;
            });

            // Await the endpoints to keep the process alive
            let _ = web_handle.await;
        }
        Commands::Ingest { source, dry, no_preserve } => ingest(source, dry, no_preserve),
    }
}
