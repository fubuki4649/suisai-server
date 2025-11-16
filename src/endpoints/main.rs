use crate::preflight::check_directories;
use crate::endpoints::album::*;
use crate::endpoints::management::*;
use crate::endpoints::meow::health_check;
use crate::endpoints::photo::*;
use crate::endpoints::thumbnail::get_thumbnail;
use rocket::routes;
use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions};
use std::str::FromStr;

pub async fn start_webserver() {

    // Run preflight checks
    check_directories().unwrap();

    // Set CORS options
    let cors = CorsOptions {
        allowed_origins: AllowedOrigins::all(),
        allowed_methods: ["Get", "Post", "Put", "Patch", "Delete"]
            .iter()
            .map(|s| FromStr::from_str(s).unwrap())
            .collect(),
        allowed_headers: AllowedHeaders::all(),
        ..Default::default()
    }
    .to_cors()
    .expect("Failed to create CORS");

    // Start server with appropriate endpoints
    rocket::build().attach(cors).mount("/", routes![
        // General
        health_check,
        
        // Album endpoints
        new_album,
        rename_album,
        del_album,
        all_albums,

        // Album queries
        album_photos,
        album_albums,
        unfiled_photos,

        // Photo endpoints
        del_photo,
        get_photos,
        
        // Photo/album management endpoints
        unfile_photo,
        reassign_photo,
        unfile_album,
        reassign_album,
        
        // Thumbnail serving endpoints
        get_thumbnail,
    ]).launch().await.expect("Failed to launch server");

}