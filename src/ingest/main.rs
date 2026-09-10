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
    let storage_root = PathBuf::from(env::var("STORAGE_ROOT").expect("$STORAGE_ROOT not set"));
    let thumbnail_root = PathBuf::from(env::var("THUMBNAIL_ROOT").expect("$THUMBNAIL_ROOT not set"));
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

    println!("Starting ingest with {} threads", available_threads);

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

                let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                let temp_filename = format!(".ingest_{}_{}", uuid::Uuid::now_v7(), filename);
                let temp_path = dest_dir.join(&temp_filename);

                // Copy and read the hash in the same pass into a temporary file
                let (hash, bytes_transferred) = match hash_and_transfer(&path, &temp_path, no_preserve).await {
                    Ok(result) => result,
                    Err(e) => {
                        println!("Error transferring {filename}: {e}");
                        continue;
                    }
                };

                println!("{} {filename} to {} ({bytes_transferred} bytes)", if no_preserve { "Moved" } else { "Copied" }, dest_dir.display());

                // Check for duplicate after transfer — if duplicate, discard only the temporary staged file
                match check_hash(&db, &hash).await {
                    Err(e) => {
                        let _ = remove_file(&temp_path).await;
                        panic!("Database Error: {e}");
                    }
                    Ok(Some(_)) => {
                        println!("Hash {hash} already exists in database, discarding");
                        if let Err(e) = remove_file(&temp_path).await {
                            println!("Warning: failed to remove duplicate file {}: {e}", temp_path.display());
                        }
                        continue;
                    }
                    Ok(None) => (),
                }

                // Determine final collision-free path in dest_dir
                let final_path = {
                    let direct = dest_dir.join(&filename);
                    if !direct.exists() {
                        direct
                    } else {
                        let stem = path.file_stem().unwrap_or_default().to_string_lossy();
                        let ext = path.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
                        let mut counter = 1;
                        loop {
                            let candidate = dest_dir.join(format!("{stem}_{counter}{ext}"));
                            if !candidate.exists() {
                                break candidate;
                            }
                            counter += 1;
                        }
                    }
                };

                let final_filename = final_path.file_name().unwrap_or_default().to_string_lossy().to_string();
                if let Err(e) = tokio::fs::rename(&temp_path, &final_path).await {
                    println!("Error finalizing {filename} to {}: {e}", final_path.display());
                    let _ = remove_file(&temp_path).await;
                    continue;
                }

                // Generate thumbnail and build DB entry using already computed hash and size
                let size_on_disk = bytes_transferred.div_ceil(1024) as i64;
                let thumbnail_root = thumbnail_root.clone();
                let final_path_clone = final_path.clone();
                let final_filename_clone = final_filename.clone();
                let new_db_asset = tokio::task::spawn_blocking(move || {
                    let mut new_db_asset = final_path_clone.to_db_entry(hash, size_on_disk);
                    new_db_asset.file_name = final_filename_clone;

                    // Generate Thumbnail
                    let date = new_db_asset.photo_date;
                    let thumbnail_filename = format!("{}.jpeg", final_path_clone.file_stem().unwrap_or_default().to_string_lossy());
                    let thumbnail_path_rel = PathBuf::from(format!("{}{:02}", date.year(), date.month())).join(&thumbnail_filename);
                    let thumbnail_path_abs = thumbnail_root.join(&thumbnail_path_rel);

                    match extract_thumbnail_full(&final_path_clone, &thumbnail_path_abs) {
                        Ok(()) => {
                            println!("Thumbnail created at {}", thumbnail_path_abs.display());
                            new_db_asset.thumbnail_path = Some(thumbnail_path_rel.to_string_lossy().to_string());
                        }
                        Err(e) => println!("Error creating thumbnail for {}: {e}", final_path_clone.display()),
                    };

                    new_db_asset
                }).await.unwrap();

                // Insert into DB
                println!("Adding {final_filename} to database");
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