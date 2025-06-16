use crate::endpoints::album::*;
use crate::endpoints::album_query::{get_album_photos, get_unfiled_photos};
use crate::endpoints::join_photo_album::*;
use crate::endpoints::photo::*;
use crate::endpoints::thumbnail::get_thumbnail;
use crate::preflight::check_directories;
use rocket::routes;
use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions};
use std::str::FromStr;

pub async fn start_server() {

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
    rocket::build().attach(cors).mount("/api", routes![
        // General
        health_check,
        
        // Album endpoints
        new_album,
        rename_album,
        del_album,
        all_albums,
        
        // Album content query endpoints
        get_album_photos,
        get_unfiled_photos,
        
        // Photo endpoints
        del_photo,
        get_photos,
        
        // Photo-Album relation endpoints
        photo_clear_album,
        photo_move_album,
        
        // Thumbnail serving endpoints
        get_thumbnail,
    ]).launch().await.expect("Failed to launch server");

}