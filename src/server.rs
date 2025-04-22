use crate::endpoints::album::*;
use crate::endpoints::album_photo_join::*;
use crate::endpoints::photo::*;
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
        
        // Photo endpoints
        new_photo,
        del_photo,
        get_photo_single,
        get_photo_multi,
        get_unfiled,
        
        // Photo-Album relation endpoints
        photo_assign_album,
        photo_clear_album,
        photo_move_album
    ]).launch().await.expect("Failed to launch server");

}