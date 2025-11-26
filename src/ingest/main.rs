use crate::db::operations::photo::{check_hash, create_photo};
use crate::db::operations::thumbnail::create_thumbnail;
use crate::ingest::extract_thumbnail::extract_thumbnail_full;
use crate::ingest::get_image_paths::get_image_paths;
use crate::ingest::trait_suisai_image_path::SuisaiImagePath;
use crate::models::thumbnail::Thumbnail;
use crate::DB_POOL;
use chrono::Datelike;
use rocket::serde::json::serde_json;
use std::env;
use std::fs::{copy, create_dir_all, rename};
use std::path::Path;

/// Ingests images from a directory into the photo library, including database storage and thumbnail generation
pub fn ingest(path: String, dry: bool, no_preserve: bool) {
    println!("Ingesting files from: {path}");
    if dry {
        println!("Running in dry mode");
    }

    // Get a list of images from the source directory 
    let paths = get_image_paths(Path::new(&path));

    // In dry run mode, just print what would happen without making changes
    if dry {
        for path in paths {
            println!("{}", serde_json::to_string_pretty(&path.to_db_entry()).unwrap());
        }
        return;
    }

    // Initialize DB connection and set up storage paths
    let mut conn = DB_POOL.get().expect("Failed to get connection from pool");
    let raw_storage_dir = format!("{}/", env::var("STORAGE_ROOT").unwrap());
    let raw_storage_path = Path::new(&raw_storage_dir);

    // Iterate over all found paths
    for path in paths {
        // Skip if this image is already in the database
        let hash = path.get_hash();
        if check_hash(&mut conn, &hash).unwrap_or_else(|_| panic!("Database error while checking hash: {hash}")).is_some() {
            println!("Hash {hash} already exists in database, skipping");
            continue;
        }

        // Prepare destination directory (`$STORAGE_ROOT/unfiled`), creating it if necessary
        let dest_directory = raw_storage_path.join("unfiled");
        create_dir_all(&dest_directory).unwrap_or_else(|_| panic!("Failed to create directory {}", dest_directory.to_str().unwrap()));

        // Copy or move the image file to the storage location
        let filename = path.file_name().unwrap_or_default().to_string_lossy();
        let new_path = dest_directory.join(filename.to_string());
        if no_preserve {
            // Move if the `--no-preserve` flag is set
            let move_result = rename(&path, &new_path);
            match move_result {
                Err(e) => println!("Error moving {} to {}: {}", filename, dest_directory.to_str().unwrap(), e),
                _ => println!("Moved {} to {}", filename, dest_directory.to_str().unwrap())
            }
        } else {
            // Copy, otherwise
            let copy_result = copy(&path, &new_path);
            match copy_result {
                Err(e) => println!("Error copying {} to {}: {}", filename, dest_directory.to_str().unwrap(), e),
                Ok(bytes) => println!("Copied {} to {} ({} bytes)", filename, dest_directory.to_str().unwrap(), bytes)
            }
        }

        // Generate and store a JPEG thumbnail at `THUMBNAIL_ROOT/yyyymm/FILENAME.jpeg`
        let date = new_path.get_photo_date();
        let thumbnail_dir = format!("{}/{}{:02}/", env::var("THUMBNAIL_ROOT").unwrap(), date.year(), date.month());
        let thumbnail_filename = format!("{}.jpeg", new_path.file_stem().unwrap().to_string_lossy());
        let mut thumbnail_path = format!("{thumbnail_dir}{thumbnail_filename}");
        // Create Thumbnail
        match extract_thumbnail_full(new_path.to_str().unwrap(), &thumbnail_dir, &thumbnail_filename) {
            Ok(()) => println!("Thumbnail created at {thumbnail_path}"),
            Err(e) => {
                thumbnail_path = String::new();
                println!("Error creating thumbnail for {filename}: {e}");
            }
        }

        // Create a database record for the image
        let photo = new_path.to_db_entry();
        println!("{}", serde_json::to_string_pretty(&photo).unwrap());

        println!("Adding {} to database", photo.file_name);
        let photo_id = match create_photo(&mut conn, photo) {
            Err(e) => {
                println!("Error: {e}");
                return;
            },
            Ok(id) => id
        };

        // Create a database record for the thumbnail, if any
        if !thumbnail_path.is_empty() {
            let thumbnail = Thumbnail { id: photo_id, thumbnail_path };
            create_thumbnail(&mut conn, &thumbnail).unwrap_or_else(|e| println!("Error: {e}"));
        }

        println!("Done");

    }

    println!("Finished");
}