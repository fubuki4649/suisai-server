use crate::_utils::json_map::JsonMap;
use crate::db::operations::album::{create_album, delete_album, get_root_albums, rename_album as rename_album_db};
use crate::db::operations::paths::get_album_path;
use crate::db::operations::query::{get_albums_in_album, get_photos_in_album, get_photos_unfiled};
use crate::fs_operations::album::delete_album_fs;
use crate::models::db::album::{Album as DbAlbum, NewAlbum};
use crate::models::webapi::album::Album;
use crate::models::webapi::photo::Photo;
use crate::{unwrap_or_return, DB_POOL};
use anyhow::Result;
use diesel::result::Error;
use rocket::http::Status;
use rocket::serde::json::{Json, Value};
use rocket::{delete, get, patch, post};

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

    match rename_album_db(&mut conn, DbAlbum {id, album_name}) {
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

    // Delete album from DB
    match delete_album(&mut conn, id) {
        Err(Error::NotFound) => Status::NotFound,
        Err(_) => Status::InternalServerError,
        Ok(album) => {
            // Also delete album from the filesystem, moving its children to root
            let album_path = unwrap_or_return!(get_album_path(&mut conn, album.id), Status::InternalServerError);
            let child_photos = unwrap_or_return!(get_photos_in_album(&mut conn, album.id), Status::InternalServerError);
            let child_albums = unwrap_or_return!(get_albums_in_album(&mut conn, album.id), Status::InternalServerError);

            unwrap_or_return!(delete_album_fs(&album_path, &child_photos, &child_albums), Status::InternalServerError);

            Status::Ok
        },
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
    let albums = unwrap_or_return!(get_root_albums(&mut conn), Err(Status::InternalServerError));
    
    Ok(Json(albums.into_iter().map(Album::from).collect()))
}


/// Retrieves all photos linked to a given album
///
/// # Endpoint
/// `GET /album/<id>/photos`
///
/// # Returns
/// - `200 OK`: JSON array of unfiled photos
/// - `500 Internal Server Error`: Database or another server error occurred
///
/// # Response Body
/// Array of webapi::Photo objects containing metadata for each photo in the album
#[get("/album/<id>/photos")]
pub fn album_photos(id: i32) -> Result<Json<Vec<Photo>>, Status> {
    let mut conn = unwrap_or_return!(DB_POOL.get(), Err(Status::InternalServerError));

    let album_photos =  unwrap_or_return!(get_photos_in_album(&mut conn, id), Err(Status::InternalServerError));
    Ok(Json(album_photos.into_iter().map(Photo::from).collect()))
}


/// Retrieves all subalbums linked to a given album
///
/// # Endpoint
/// `GET /album/<id>/photos`
///
/// # Returns
/// - `200 OK`: JSON array of unfiled photos
/// - `500 Internal Server Error`: Database or another server error occurred
///
/// # Response Body
/// Array of api::Album objects containing metadata for each photo in the album
#[get("/album/<id>/albums")]
pub fn album_albums(id: i32) -> Result<Json<Vec<Album>>, Status> {
    let mut conn = unwrap_or_return!(DB_POOL.get(), Err(Status::InternalServerError));

    let album_albums =  unwrap_or_return!(get_albums_in_album(&mut conn, id), Err(Status::InternalServerError));
    Ok(Json(album_albums.into_iter().map(Album::from).collect()))
}


/// Retrieves all photos that are not assigned to any album
///
/// # Endpoint
/// `GET /album/unfiled/photos`
///
/// # Returns
/// - `200 OK`: JSON array of unfiled photos
/// - `500 Internal Server Error`: Database or another server error occurred
///
/// # Response Body
/// Array of webapi::Photo objects containing metadata for each unfiled photo
#[get("/album/unfiled/photos")]
pub fn unfiled_photos() -> Result<Json<Vec<Photo>>, Status> {
    let mut conn = unwrap_or_return!(DB_POOL.get(), Err(Status::InternalServerError));

    let unfiled_photos =  unwrap_or_return!(get_photos_unfiled(&mut conn), Err(Status::InternalServerError));
    Ok(Json(unfiled_photos.into_iter().map(Photo::from).collect()))
}