use crate::db::entities::{assets, collections};
use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr, Schema};
use std::env;
use std::fs;
use std::path::PathBuf;

/// Checks and creates required directory structure for the application.
///
/// Verifies that all directories required by the application exist under `$STORAGE_ROOT`
/// and `$THUMBNAIL_ROOT`, creating any that are missing. Returns an error if `$STORAGE_ROOT`
/// or `$THUMBNAIL_ROOT` are not set, or if any expected path exists but is not a directory.
pub fn check_directories() -> Result<(), anyhow::Error> {
    let storage_root = PathBuf::from(env::var("STORAGE_ROOT").map_err(|_| anyhow::anyhow!("$STORAGE_ROOT not set"))?);
    let thumbnail_root = PathBuf::from(env::var("THUMBNAIL_ROOT").map_err(|_| anyhow::anyhow!("$THUMBNAIL_ROOT not set"))?);

    let paths = [
        storage_root.join("unfiled"),
        thumbnail_root,
    ];

    for path in paths {
        if path.exists() {
            if path.is_dir() {
                println!("Found existing directory: {}", path.display());
            } else {
                return Err(anyhow::anyhow!("{} exists but is not a directory", path.display()));
            }
        } else {
            fs::create_dir_all(&path)?;
            println!("Created new directory: {}", path.display());
        }
    }

    Ok(())
}

/// Verifies that required external CLI dependencies are installed and available in `$PATH`.
pub fn check_cli_deps() -> Result<(), anyhow::Error> {
    let required_tools = ["dcraw", "cjpeg"];
    let mut missing = Vec::new();

    let path_var = env::var_os("PATH").ok_or_else(|| anyhow::anyhow!("$PATH is not set"))?;

    // Check for missing CLI deps
    for tool in required_tools {
        let found = env::split_paths(&path_var).any(|dir| dir.join(tool).is_file());
        if !found {
            missing.push(tool);
        }
    }

    // Report missing CLI deps if any, and do not continue
    if !missing.is_empty() {
        return Err(anyhow::anyhow!(
            "Missing required external CLI tools: {}. Please install them and ensure they are in your $PATH.",
            missing.join(", ")
        ));
    }

    println!("Found required external CLI tools: {}", required_tools.join(", "));
    Ok(())
}

/// Ensures the database parent directory exists, connects to the database, and auto-creates
/// required tables (`collections` & `assets`) if they do not exist.
pub async fn check_database() -> Result<DatabaseConnection, DbErr> {
    let database_url = env::var("DATABASE_URL")
        .map_err(|_| DbErr::Custom("$DATABASE_URL not set".into()))?;

    if let Some(sqlite_path) = database_url.strip_prefix("sqlite://") {
        let clean_path = sqlite_path.split('?').next().unwrap_or(sqlite_path);
        let path = PathBuf::from(clean_path);

        // Create database directory if it does not exist
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty() && !p.exists()) {
            fs::create_dir_all(parent).map_err(|e| DbErr::Custom(e.to_string()))?;
            println!("Created new directory: {}", parent.display());
        }
    }

    let db = sea_orm::Database::connect(&database_url).await?;
    let schema = Schema::new(db.get_database_backend());

    db.execute(schema.create_table_from_entity(collections::Entity).if_not_exists()).await?;
    db.execute(schema.create_table_from_entity(assets::Entity).if_not_exists()).await?;

    Ok(db)
}
