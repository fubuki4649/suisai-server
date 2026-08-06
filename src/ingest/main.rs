use crate::db::operations::asset::{check_hash, new_asset};
use crate::ingest::helpers::extract_thumbnail::extract_thumbnail_full;
use crate::ingest::helpers::search_path::search_path_for_assets;
use crate::ingest::traits::SuisaiAsset;
use chrono::Datelike;
use sea_orm::DatabaseConnection;
use std::env;
use std::path::{Path, PathBuf};


/// Ingests photos from a directory as a suisai asset, including database storage and thumbnail generation
pub async fn ingest(db: &DatabaseConnection, path: String, dry: bool, no_preserve: bool) {
    println!("Ingesting files from: {path}");
    if dry {
        println!("Running in dry mode");
    }

    // Get a list of images from the source directory
    let paths = search_path_for_assets(Path::new(&path));

    // In dry run mode, just print what would happen without making changes
    if dry {
        for path in paths {
            println!("{}", serde_json::to_string_pretty(&path.to_db_entry()).unwrap());
        }
        return;
    }

    // Initialize storage paths
    let storage_root = PathBuf::from(env::var("STORAGE_ROOT").unwrap());
    let thumbnail_root = PathBuf::from(env::var("THUMBNAIL_ROOT").unwrap());

    // Iterate over all found paths
    for path in paths {
        // Skip if this image is already in the database
        let hash = {
            let path = path.clone();
            tokio::task::spawn_blocking(move || path.get_hash()).await.unwrap()
        };

        match check_hash(db, &hash).await {
            Ok(Some(_)) => {
                println!("Hash {hash} already exists in database, skipping");
                continue;
            },
            Ok(None) => (),
            Err(e) => panic!("Database Error: {e}"),
        }

        // Prepare destination directory (`$STORAGE_ROOT/unfiled`), creating it if necessary
        let dest_directory = storage_root.join("unfiled");
        tokio::fs::create_dir_all(&dest_directory).await
            .unwrap_or_else(|_| panic!("Failed to create directory {}", dest_directory.display()));

        // Copy or move the image file to the storage location
        let filename = path.file_name().unwrap_or_default().to_string_lossy();
        let new_path = dest_directory.join(filename.to_string());
        if no_preserve {
            // Move if the `--no-preserve` flag is set
            match tokio::fs::rename(&path, &new_path).await {
                Err(e) => {
                    println!("Error moving {} to {}: {}", filename, dest_directory.display(), e);
                    continue
                },
                Ok(_) => println!("Moved {} to {}", filename, dest_directory.display())
            }
        } else {
            // Copy, otherwise
            match tokio::fs::copy(&path, &new_path).await {
                Err(e) => {
                    println!("Error copying {} to {}: {}", filename, dest_directory.display(), e);
                    continue
                },
                Ok(bytes) => println!("Copied {} to {} ({} bytes)", filename, dest_directory.display(), bytes)
            }
        }

        // Generate and store a JPEG thumbnail at `$THUMBNAIL_ROOT/yyyymm/FILENAME.jpeg`
        // Metadata extraction (exiftool) and thumbnail generation (dcraw/cjpeg) are blocking
        // subprocess calls — run them off the async executor.
        let new_path_clone = new_path.clone();
        let thumbnail_root_clone = thumbnail_root.clone();
        let thumbnail_path = tokio::task::spawn_blocking(move || {
            let date = new_path_clone.get_photo_date();
            let thumbnail_subdir = format!("{}{:02}", date.year(), date.month());
            let thumbnail_dir = thumbnail_root_clone.join(&thumbnail_subdir);
            let thumbnail_filename = format!("{}.jpeg", new_path_clone.file_stem().unwrap().to_string_lossy());
            let thumbnail_dir_str = thumbnail_dir.to_string_lossy().to_string();

            match extract_thumbnail_full(new_path_clone.to_str().unwrap(), &thumbnail_dir_str, &thumbnail_filename) {
                Ok(()) => {
                    // Store as relative path: `yyyymm/FILENAME.jpeg`
                    let relative = format!("{thumbnail_subdir}/{thumbnail_filename}");
                    println!("Thumbnail created at {}", thumbnail_dir.join(&thumbnail_filename).display());
                    Some(relative)
                },
                Err(e) => {
                    println!("Error creating thumbnail for {}: {e}", new_path_clone.display());
                    None
                }
            }
        }).await.unwrap();

        // Build the db entry from metadata (also blocking — exiftool subprocess calls)
        let new_path_clone = new_path.clone();
        let mut asset = tokio::task::spawn_blocking(move || new_path_clone.to_db_entry()).await.unwrap();
        asset.thumbnail_path = thumbnail_path;
        println!("{}", serde_json::to_string_pretty(&asset).unwrap());

        println!("Adding {} to database", asset.file_name);
        let new_asset_id = match new_asset(db, asset).await {
            Err(e) => {
                println!("Error: {e}");
                return;
            },
            Ok(id) => id
        };

        println!("Created asset with database ID {new_asset_id}");
    }

    println!("Finished");
}