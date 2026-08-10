use crate::db::operations::asset::{check_hash, new_asset};
use crate::ingest::helpers::extract_thumbnail::extract_thumbnail_full;
use crate::ingest::helpers::hash_and_transfer::hash_and_transfer;
use crate::ingest::helpers::search_path::search_path_for_assets;
use crate::ingest::traits::SuisaiAsset;
use chrono::Datelike;
use sea_orm::DatabaseConnection;
use std::env;
use std::num::NonZero;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::{create_dir_all, remove_file};
use tokio::sync::Mutex;
use tokio::task::JoinSet;

/// Ingests photos from a directory as a suisai asset, including database storage and thumbnail generation
pub async fn ingest(db: &DatabaseConnection, path: String, no_preserve: bool) {

    // Set up destination directories
    let storage_root = PathBuf::from(env::var("STORAGE_ROOT").unwrap());
    let thumbnail_root = PathBuf::from(env::var("THUMBNAIL_ROOT").unwrap());
    let dest_dir = storage_root.join("unfiled");
    create_dir_all(&dest_dir).await.unwrap_or_else(|_| panic!("Failed to create directory {}", dest_dir.display()));

    // Set up send/receive channels for multithreading
    let (tx, rx) = tokio::sync::mpsc::channel::<PathBuf>(100);
    let shared_rx = Arc::new(Mutex::new(rx));

    // Launch producer to probe for files to ingest
    tokio::task::spawn_blocking(move || {
        println!("Ingesting files from: {path}");
        search_path_for_assets(&PathBuf::from(path), &tx).unwrap_or_else(|err| {
            eprintln!("Error searching for assets: {}", err);
        });
        drop(tx);
    });

    let available_threads = std::thread::available_parallelism().unwrap_or(NonZero::new(8).unwrap()).get();
    let mut workers = JoinSet::new();

    // Launch workers equal to the number of threads to ingest in parallel
    for _ in 0..available_threads {
        let rx = shared_rx.clone();
        let db = db.clone();
        let dest_dir = dest_dir.clone();
        let thumbnail_root = thumbnail_root.clone();

        workers.spawn(async move {
            loop {
                let path = {
                    let mut guard = rx.lock().await;
                    guard.recv().await
                };

                let Some(path) = path else { break };

                let filename = path.file_name().unwrap_or_default().to_string_lossy();
                let asset_new_path = dest_dir.join(filename.as_ref());

                // Copy and read the hash in the same pass (we can delete it later if we don't need it)
                // Much better for ingesting straight from SD cards/slow network sources
                let (hash, bytes_transferred) = match hash_and_transfer(&path, &asset_new_path, no_preserve).await {
                    Ok(result) => result,
                    Err(e) => {
                        println!("Error transferring {filename}: {e}");
                        continue;
                    }
                };

                println!("{} {filename} to {} ({bytes_transferred} bytes)", if no_preserve { "Moved" } else { "Copied" }, dest_dir.display());

                // Check for duplicate after transfer — if duplicate, discard the transferred file
                match check_hash(&db, &hash).await {
                    Err(e) => panic!("Database Error: {e}"),
                    Ok(Some(_)) => {
                        println!("Hash {hash} already exists in database, discarding");
                        if let Err(e) = remove_file(&asset_new_path).await {
                            println!("Warning: failed to remove duplicate file {}: {e}", asset_new_path.display());
                        }
                        continue;
                    },
                    Ok(None) => (),
                }

                // Generate thumbnail and build DB entry
                let thumbnail_root = thumbnail_root.clone();
                let new_db_asset = tokio::task::spawn_blocking(move || {
                    // Build DB entry
                    let mut new_db_asset = asset_new_path.to_db_entry();

                    // Generate Thumbnail
                    let date = asset_new_path.get_photo_date();
                    let thumbnail_filename = format!("{}.jpeg", asset_new_path.file_stem().unwrap().to_string_lossy());
                    let thumbnail_path_rel = PathBuf::from(format!("/{}{:02}", date.year(), date.month())).join(&thumbnail_filename);
                    let thumbnail_path_abs = thumbnail_root.join(&thumbnail_path_rel);

                    match extract_thumbnail_full(asset_new_path.to_str().unwrap(), thumbnail_path_abs.parent().unwrap().to_string_lossy().as_ref(), &thumbnail_filename) {
                        Ok(()) => {
                            println!("Thumbnail created at {}", thumbnail_path_abs.join(&thumbnail_filename).display());
                            new_db_asset.thumbnail_path = Some(thumbnail_path_rel.to_string_lossy().to_string());
                        },
                        Err(e) => println!("Error creating thumbnail for {}: {e}", asset_new_path.display()),
                    };

                    new_db_asset
                }).await.unwrap();

                // Insert into DB
                println!("Adding {filename} to database");
                match new_asset(&db, new_db_asset).await {
                    Err(e) => println!("Error: {e}"),
                    Ok(id) => println!("Created asset with database ID {id}")
                };
            }
        });
    }

    workers.join_all().await;
    println!("Finished");
}