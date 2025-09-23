use crate::ingest::ingest::ingest;
use crate::virtfs::fuser::mount_fuse;
use crate::webserver::webserver::start_webserver;
use clap::{Parser, Subcommand};
use rocket::tokio;
use std::thread;

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
        #[arg(help = "Path where to mount the FUSE directory", required_unless_present = "disable_fuse")]
        fuse_mount: Option<String>,
        #[arg(long, help = "Do not mount a FUSE directory")]
        disable_fuse: bool,
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
        Commands::StartServer { fuse_mount, disable_fuse } => {
            // Spawn the webserver on its own async task
            let web_handle = tokio::spawn(async move {
                start_webserver().await;
            });

            // Optionally mount FUSE on a separate OS thread
            if !disable_fuse {
                // fuse_mount should never be None if disable_fuse is false; The CLI should prevent this from happening
                let mountpoint = fuse_mount.unwrap();

                thread::spawn(move || {
                    if let Err(e) = mount_fuse(&mountpoint) {
                        eprintln!("Failed to mount FUSE at '{}': {}", mountpoint, e);
                    }
                });
            }

            // Await the webserver to keep the process alive
            let _ = web_handle.await;
        }
        Commands::Ingest { source, dry, no_preserve } => ingest(source, dry, no_preserve),
    }
}
