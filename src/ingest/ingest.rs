use std::path::Path;
use rocket::serde::json::serde_json;
use crate::db::operations::photo::{check_hash, create_photo};
use crate::DB_POOL;
use crate::ingest::get_images::get_images;
use crate::ingest::trait_suisai_image::SuisaiImage;

pub fn ingest(path: String, dry: bool) {
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
    
    for path in paths {
        // Check if image is already in DB
        let hash = path.get_hash();
        if check_hash(&mut conn, &hash).unwrap_or_else(|_| panic!("Database error while checking hash: {}", hash)).is_some() {
            println!("Hash {} already exists in database, skipping", hash);
            continue;
        }

        // Convert to DB entry
        let photo = path.to_db_entry();
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