use crate::ingest::ingest::ingest;
use crate::server::start_server;
use clap::{Parser, Subcommand};


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
        #[arg(long, help = "Move instead of move files to their new destination (default behavior is copy)")]
        no_preserve: bool,
    }
}

pub async fn run_cli() {
    let cli = Cli::parse();

    match cli.command {
        Commands::StartServer {} => start_server().await,
        Commands::Ingest {path, dry, no_preserve} => ingest(path, dry, no_preserve),
    }
}
