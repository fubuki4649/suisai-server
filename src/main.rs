use std::env;
use diesel::PgConnection;
use diesel::r2d2::ConnectionManager;
use dotenvy::dotenv;
use crate::cli::run_cli;
use crate::db::models::album::*;
use crate::db::models::photo::NewPhoto;
use crate::db::operations::album::{create_album, get_all_albums};
use crate::db::operations::album_photo_join::{add_photo_to_album, get_photos_in_album};
use crate::db::operations::photo::create_photo;

#[macro_use]
extern crate lazy_static;

mod db;
mod endpoints;
mod cli;
mod server;
mod ingest;

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


#[rocket::main]
async fn main() {
    
    run_cli().await;

}