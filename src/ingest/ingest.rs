use std::env;
use std::fs::{copy, create_dir_all, rename};
use std::path::Path;
use chrono::Datelike;
use rocket::serde::json::serde_json;
use crate::db::operations::photo::{check_hash, create_photo};
use crate::DB_POOL;
use crate::ingest::get_images::get_images;
use crate::ingest::trait_suisai_image::SuisaiImage;

pub fn ingest(path: String, dry: bool, preserve: bool) {
    println!("Ingesting files from: {}", path);
    if dry {
        println!("Running in dry mode");
    }

    // Get a list of images 
    let paths = get_images(Path::new(&path));
    
    if dry {
        for path in paths {
            println!("{}", serde_json::to_string_pretty(&path.to_db_entry()).unwrap());
        }
        return;
    }
    
    // Actually ingest
    let mut conn = DB_POOL.get().expect("Failed to get connection from pool");
    let raw_storage_dir = format!("{}/raws", env::var("STORAGE_ROOT").unwrap());
    let raw_storage_path = Path::new(&raw_storage_dir);

    for path in paths {
        // Check if photo is already in DB
        let hash = path.get_hash();
        if check_hash(&mut conn, &hash).unwrap_or_else(|_| panic!("Database error while checking hash: {}", hash)).is_some() {
            println!("Hash {} already exists in database, skipping", hash);
            continue;
        }

        // Relocate photo to `$STORAGE_ROOT/raws`
        // Create/verify destination directory exists
        let date = path.get_photo_date();
        let dest_directory = raw_storage_path.join(format!("{}{:02}", date.year(), date.month()));
        create_dir_all(&dest_directory).unwrap_or_else(|_| panic!("Failed to create directory {}", dest_directory.to_str().unwrap()));

        // Copy/Move photo as appropriate
        let filename = path.file_name().unwrap_or_default().to_string_lossy();
        let new_path = dest_directory.join(filename.to_string());
        match preserve {
            true => {
                // Copy
                let copy_result = copy(&path, &new_path);
                match copy_result {
                    Err(e) => println!("Error copying {} to {}: {}", filename, dest_directory.to_str().unwrap(), e),
                    Ok(bytes) => println!("Copied {} to {} ({} bytes)", filename, dest_directory.to_str().unwrap(), bytes)
                }
            },
            false => {
                // Move
                let move_result = rename(&path, &new_path);
                match move_result {
                    Err(e) => println!("Error moving {} to {}: {}", filename, dest_directory.to_str().unwrap(), e),
                    _ => println!("Moved {} to {}", filename, dest_directory.to_str().unwrap())
                }
            }
        }

        // Convert to DB entry
        let photo = new_path.to_db_entry();
        println!("{}", serde_json::to_string_pretty(&photo).unwrap());

        // Add to DB
        println!("Adding {} to database", photo.file_path);
        match create_photo(&mut conn, photo) {
            Err(e) => println!("Error: {}", e),
            _ => println!("Success")
        };
    }
    
    println!("Done");
}

fn relocate_image(path: String) {
    
}