use crate::webserver::album::*;
use crate::webserver::album_query::{get_album_photos, get_unfiled_photos};
use crate::webserver::join_photo_album::*;
use crate::webserver::photo::*;
use crate::webserver::thumbnail::get_thumbnail;
use crate::preflight::check_directories;
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

    // Start server with appropriate webserver
    rocket::build().attach(cors).mount("/api", routes![
        // General
        health_check,
        
        // Album webserver
        new_album,
        rename_album,
        del_album,
        all_albums,
        
        // Album content query webserver
        get_album_photos,
        get_unfiled_photos,
        
        // Photo webserver
        del_photo,
        get_photos,
        
        // Photo-Album relation webserver
        photo_clear_album,
        photo_move_album,
        
        // Thumbnail serving webserver
        get_thumbnail,
    ]).launch().await.expect("Failed to launch server");

}