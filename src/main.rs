use std::env;
use diesel::PgConnection;
use diesel::r2d2::ConnectionManager;
use dotenvy::dotenv;
use crate::db::models::Album;
use crate::db::operations::album::create_album;

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

}


// TESTING TODO: REMOVE LATER

fn create_album_example(pool: &Pool) -> Album {
    let mut conn = pool.get().expect("Failed to get connection from pool");

    let album = db::models::NewAlbum {
        album_name: "Vacation 2025".to_string(),
    };

    create_album(&mut conn, album).expect("Failed to create album")
}
