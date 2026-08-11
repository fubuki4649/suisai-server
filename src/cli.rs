use crate::endpoints::main::start_webserver;
use crate::ingest::main::ingest;
use crate::state::AppState;
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
    StartServer {
    },
    #[command(about = "Ingest camera raws from a directory")]
    Ingest {
        #[arg(help = "Path to a directory containing camera raws")]
        source: String,
        #[arg(long, help = "Move instead of copy files to their new destination (default behavior is copy)")]
        no_preserve: bool,
    }
}

pub async fn run_cli(state: AppState) {
    let cli = Cli::parse();

    match cli.command {
        Commands::StartServer { } => start_webserver(state).await,
        Commands::Ingest { source, no_preserve } => ingest(&state.db, source, no_preserve).await
    }
}
