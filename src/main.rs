use std::env;
use diesel::PgConnection;
use diesel::r2d2::ConnectionManager;
use dotenvy::dotenv;
use crate::db::models::album::*;
use crate::db::models::photo::NewPhoto;
use crate::db::operations::album::{create_album, get_all_albums};
use crate::db::operations::album_photo_join::{add_photo_to_album, get_photos_in_album};
use crate::db::operations::photo::create_photo;

#[macro_use]
extern crate lazy_static;

mod db;
mod endpoints;


type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;


lazy_static! {
    static ref DB_POOL: Pool = establish_connection_pool();
}

// Function to establish database connection pool
fn establish_connection_pool() -> Pool {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder()
        .build(manager)
        .expect("Failed to create pool")
}


fn main() {

    let pool = establish_connection_pool();


    // TESTING TODO: REMOVE LATER

    let new_album = create_album_example(&pool);
    println!("Created album with ID: {}", new_album.id);
    
    println!("Hello, world!");

    list_all_albums_test(&pool)

}


// TESTING TODO: REMOVE LATER

fn create_album_example(pool: &Pool) -> DBAlbum {
    let mut conn = pool.get().expect("Failed to get connection from pool");

    let album = NewAlbum {
        album_name: "Vacation 2025".to_string(),
    };

    let alb = create_album(&mut conn, album).expect("Failed to create album");
    let newtp = NewPhoto {
        thumbnail_url: "".to_string(),
        file_name: "test pic 1".to_string(),
        file_path: "test pic 1".to_string(),
        size_on_disk: "".to_string(),
        photo_date: Default::default(),
        photo_timezone: "".to_string(),
        resolution: vec![],
        mime_type: "".to_string(),
        camera_model: "".to_string(),
        lens_model: "".to_string(),
        shutter_count: 0,
        focal_length: 0,
        iso: 0,
        shutter_speed: "".to_string(),
        aperture: 0.0,
    };

    let tp = create_photo(&mut conn, newtp).expect("Failed to create photo");

    add_photo_to_album(&mut conn, alb.id, tp.id).expect("Failed to add photo");

    alb
}


fn list_all_albums_test(pool: &Pool) {
    let mut conn = pool.get().expect("Failed to get connection from pool");

    let albs = get_all_albums(&mut conn).expect("Failed to get all albums");
    println!("{} {}", albs.len(), albs.last().unwrap().id);

    let albphotos = get_photos_in_album(&mut conn, albs.last().unwrap().id).expect("Failed to get photos");
    println!("{:?}", albphotos)

}