use rocket::{launch, routes};
use crate::endpoints::album::*;
use crate::endpoints::photo::*;

pub async fn start_server() {

    rocket::build().mount("/api", routes![
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
    ]).launch().await.expect("Failed to launch server");

}