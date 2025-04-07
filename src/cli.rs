use clap::{Parser, Subcommand};
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
    }
}

pub async fn run_cli() {
    let cli = Cli::parse();

    match cli.command {
        Commands::StartServer {} => start_server().await,
        Commands::Ingest {path} => {
            println!("Ingesting files from: {}", path);
        }
    };
}
