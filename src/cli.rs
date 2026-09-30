use crate::endpoints::main::start_webserver;
use crate::ingest::main::ingest;
use crate::preflight::{check_database, check_directories};
use crate::state::AppState;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "suisai",
    version = concat!(env!("CARGO_PKG_VERSION"), " (build ", env!("SUISAI_COMMIT_HASH"), ")"),
    about = "Backend server for suisai"
)]
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
    #[command(
        about = "Ingest camera raws from a directory (dumps into 'unfiled' by default; use --preserve-structure to organize into collections)",
        long_about = "Ingest camera raws from a directory.\n\nBy default, all discovered photos are dumped flat into the 'unfiled' collection. Use --preserve-structure to preserve the source directory structure as nested collections under a dated 'ingest-MM-DD-YYYY' root collection."
    )]
    Ingest {
        #[arg(help = "Path to a directory containing camera raws")]
        source: String,
        #[arg(long, help = "Move instead of copy files to their new destination (default behavior is copy)")]
        no_preserve: bool,
        #[arg(
            long,
            visible_alias = "preserve-structures",
            help = "Preserve directory structure as collections under an ingest-MM-DD-YYYY root collection instead of dumping into 'unfiled'"
        )]
        preserve_structure: bool,
    }
}

pub async fn run_cli() {
    let cli = Cli::parse();

    // Run directory preflight checks (creates storage dirs & database parent dirs)
    if let Err(e) = check_directories() {
        eprintln!("Error: Directory preflight check failed: {e}");
        std::process::exit(1);
    }

    // Initialize DB
    let db = match check_database().await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Error: Database initialization failed: {e}");
            std::process::exit(1);
        }
    };

    // Initialize global state
    let state = AppState { db };

    match cli.command {
        Commands::StartServer { } => start_webserver(state).await,
        Commands::Ingest { source, no_preserve, preserve_structure } => ingest(&state.db, source, no_preserve, preserve_structure).await
    }
}
