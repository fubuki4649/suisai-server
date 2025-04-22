use crate::_utils::json_map::JsonMap;
use crate::db::models::album::*;
use crate::db::models::photo::Photo;
use crate::db::operations::album::{create_album, delete_album, get_all_albums, update_album};
use crate::db::operations::photo::get_unfiled_photos;
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

    crate::err_to_500!({
        let mut conn = DB_POOL.get()?;

        create_album(&mut conn, NewAlbum {album_name: album_name.to_string()})?;
        Ok(Status::Created)
    })
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

    crate::err_to_500!({
        let mut conn = DB_POOL.get()?;

        match update_album(&mut conn, Album {id, album_name, photos: vec![]}) {
            Ok(_) => Ok(Status::Ok),
            Err(Error::NotFound) => Ok(Status::NotFound),
            Err(e) => Err(e.into()),
        }
    })
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
    crate::err_to_500!({
        let mut conn = DB_POOL.get()?;

        match delete_album(&mut conn, id) {
            Ok(_) => Ok(Status::Ok),
            Err(Error::NotFound) => Ok(Status::NotFound),
            Err(e) => Err(e.into()),
        }
    })
}

/// Retrieves all albums from the database
///
/// # Endpoint
/// `GET /album/all`
///
/// # Returns
/// - `200 OK`: JSON array of all albums
/// - `500 Internal Server Error`: Database or other server error occurred
///
/// # Response Body
/// Array of Album objects, each containing:
/// - `albumId`: Album's unique identifier (i32)
/// - `albumName`: Name of the album (String)
/// - `photos`: Array of photo IDs contained in the album (Vec<i64>)
#[get("/album/all")]
pub fn all_albums() -> Result<Json<Vec<Album>>, Status> {
    crate::err_to_result_500!({
        let mut conn = DB_POOL.get()?;
        let albums = get_all_albums(&mut conn)?;
        
        Ok(Ok(Json(albums)))
    })
}

/// Retrieves all photos that are not assigned to any album
///
/// # Endpoint
/// `GET /album/unfiled`
///
/// # Returns
/// - `200 OK`: JSON array of unfiled photos
/// - `500 Internal Server Error`: Database or another server error occurred
///
/// # Response Body
/// Array of Photo objects containing metadata about each unfiled photo
#[get("/album/unfiled")]
pub fn get_unfiled() -> Result<Json<Vec<Photo>>, Status> {
    crate::err_to_result_500!({
        let mut conn = DB_POOL.get()?;

        let unfiled_photos = get_unfiled_photos(&mut conn)?;
        Ok(Ok(Json(unfiled_photos)))
    })
}
