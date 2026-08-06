use std::env;
use std::fs;
use std::path::PathBuf;


/// Checks and creates required directory structure for the application.
///
/// This function verifies that all necessary directories exist under `$STORAGE_ROOT`:
/// - thumbs/ : For storing thumbnail images
/// - raws/ : For storing raw photo files
/// - associated_files/ : For storing associated metadata files
///
/// Creates any missing directories as needed. Returns error if `$STORAGE_ROOT` is not set
/// or if expected paths exist but are not directories.
pub fn check_directories() -> Result<(), anyhow::Error> {
    let storage_root = PathBuf::from(env::var("STORAGE_ROOT").map_err(|_| anyhow::anyhow!("$STORAGE_ROOT not set"))?);
    let thumbnail_root = PathBuf::from(env::var("THUMBNAIL_ROOT").map_err(|_| anyhow::anyhow!("THUMBNAIL_ROOT not set"))?);

    // Check if `$STORAGE_ROOT`, `$STORAGE_ROOT/thumbs` and `$STORAGE_ROOT/raws`, 
    // and `$STORAGE_ROOT/associated_files` exist as directories.
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

/// Verifies and auto-creates required database tables (`collections` & `assets`) if they do not exist.
pub async fn check_database(db: &sea_orm::DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    use sea_orm::{ConnectionTrait, Schema};
    use crate::db::entities::{assets, collections};

    let builder = db.get_database_backend();
    let schema = Schema::new(builder);

    let mut stmt_collections = schema.create_table_from_entity(collections::Entity);
    stmt_collections.if_not_exists();
    let stmt_collections = stmt_collections;

    let mut stmt_assets = schema.create_table_from_entity(assets::Entity);
    stmt_assets.if_not_exists();
    let stmt_assets = stmt_assets;

    db.execute(&stmt_collections).await?;
    db.execute(&stmt_assets).await?;

    Ok(())
}
