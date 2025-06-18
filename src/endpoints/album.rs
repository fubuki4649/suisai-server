use crate::_utils::json_map::JsonMap;
use crate::models::album::*;
use crate::db::operations::album::{create_album, delete_album, get_all_albums, update_album};
use crate::{unwrap_or_return, DB_POOL};
use anyhow::Result;
use diesel::result::Error;
use rocket::http::Status;
use rocket::serde::json::{Json, Value};
use rocket::{delete, get, patch, post};


/// Simple health check endpoint to verify API is responding
///
/// # Endpoint  
/// `GET /meow`
///
/// # Returns
/// - `418 I'm a teapot`: API is up and running
#[get("/meow")]
pub fn health_check() -> (Status, &'static str) {
    (Status::ImATeapot, ">///<\n")
}

/// Creates a new album in the database
///
/// # Endpoint
/// `POST /album/new`
///
/// # Request Body
/// JSON object with:
/// - `album_name`: Name for the new album (String)
///
/// # Returns
/// - `201 Created`: Album was successfully created
/// - `400 Bad Request`: Missing or invalid album_name in request body
/// - `500 Internal Server Error`: Database or other server error occurred
#[post("/album/new", format = "json", data = "<input>")]
pub fn new_album(input: Json<Value>) -> Status {
    let album_name = unwrap_or_return!(input.get_value::<String>("album_name"), Status::BadRequest);
    let mut conn = unwrap_or_return!(DB_POOL.get(), Status::InternalServerError);
    
    match create_album(&mut conn, NewAlbum {album_name: album_name.to_string()}) {
        Ok(rows) => {
            match rows {
                1 => Status::Created,
                0 => Status::Conflict,
                _ => Status::InternalServerError,
            }
        },
        Err(_) => Status::InternalServerError,
    }
}

/// Renames an existing album in the database
///
/// # Endpoint
/// `PATCH /album/<id>/rename`
///
/// # URL Parameters
/// - `id`: The ID of the album to rename (i32)
///
/// # Request Body
/// JSON object with:
/// - `album_name`: New name for the album (String)
///
/// # Returns
/// - `200 OK`: Album was successfully renamed
/// - `400 Bad Request`: Missing or invalid album_name in the request body
/// - `404 Not Found`: Album with the specified ID does not exist
/// - `500 Internal Server Error`: Database or other server error occurred
#[patch("/album/<id>/rename", format = "json", data = "<input>")]
pub fn rename_album(id: i32, input: Json<Value>) -> Status {
    let album_name = unwrap_or_return!(input.get_value::<String>("album_name"), Status::BadRequest);
    let mut conn = unwrap_or_return!(DB_POOL.get(), Status::InternalServerError);

    match update_album(&mut conn, Album {id, album_name}) {
        Ok(_) => Status::Ok,
        Err(Error::NotFound) => Status::NotFound,
        Err(_) => Status::InternalServerError,
    }
}

/// Deletes an album from the database by ID
///
/// # Endpoint
/// `DELETE /album/<id>/delete`
///
/// # URL Parameters
/// - `id`: The ID of the album to delete (i32)
///
/// # Returns
/// - `200 OK`: Album was successfully deleted
/// - `404 Not Found`: Album with the specified ID does not exist
/// - `500 Internal Server Error`: Database or other server error occurred
#[delete("/album/<id>/delete")]
pub fn del_album(id: i32) -> Status {
    let mut conn = unwrap_or_return!(DB_POOL.get(), Status::InternalServerError);

    match delete_album(&mut conn, id) {
        Ok(_) => Status::Ok,
        Err(Error::NotFound) => Status::NotFound,
        Err(_) => Status::InternalServerError,
    }
}

/// Retrieves all albums from the database
///
/// # Endpoint
/// `GET /album/all`
///
/// # Returns
/// - `200 OK`: JSON array of all albums
/// - `500 Internal Server Error`: Database or another server error occurred
///
/// # Response Body
/// Array of Album objects, each containing:
/// - `albumId`: Album's unique identifier (i32)
/// - `albumName`: Name of the album (String)
#[get("/album/all")]
pub fn all_albums() -> Result<Json<Vec<Album>>, Status> {
    let mut conn = unwrap_or_return!(DB_POOL.get(), Err(Status::InternalServerError));
    let albums = unwrap_or_return!(get_all_albums(&mut conn), Err(Status::InternalServerError));
    
    Ok(Json(albums))
}
