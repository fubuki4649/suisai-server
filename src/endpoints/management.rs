use crate::_utils::json_map::JsonMap;
use crate::db::operations::join_album_album::{add_album_to_album, remove_album_from_album};
use crate::db::operations::join_album_photo::{add_photo_to_album, remove_photo_from_album};
use crate::{unwrap_or_return, DB_POOL};
use rocket::http::Status;
use rocket::post;
use rocket::serde::json::{Json, Value};

/// Removes photos from all albums they are currently assigned to
///
/// # Endpoint
/// `POST /management/photo/unfile`
///
/// # Request Body
/// JSON object with:
/// - `photo_ids`: Array of photo IDs to remove from all albums
///
/// # Returns
/// - `200 OK`: Photos were successfully removed from all albums
/// - `500 Internal Server Error`: Database or other server error occurred 
#[post("/management/photo/unfile", format = "json", data = "<input>")]
pub fn unfile_photo(input: Json<Value>) -> Status {
    let photo_ids = unwrap_or_return!(input.get_value::<Vec<i64>>("photo_ids"), Status::BadRequest);
    let mut conn = unwrap_or_return!(DB_POOL.get(), Status::InternalServerError);
    
    match remove_photo_from_album(&mut conn, &photo_ids) {
        Ok(_) => Status::Ok,
        Err(_) => Status::InternalServerError,
    }
}

/// Moves photos from their current album(s) to a different album
///
/// # Endpoint
/// `POST /management/photo/reassign`
///
/// # Request Body
/// JSON object with:
/// - `album_id`: The ID of the destination album (i32) 
/// - `photo_ids`: Array of photo IDs to move to the new album (Vec<i64>)
///
/// # Returns
/// - `200 OK`: Photos were successfully moved to the new album
/// - `400 Bad Request`: Missing or invalid album_id or photo_ids in request body
/// - `500 Internal Server Error`: Database error or other server error occurred
#[post("/management/reassign", format = "json", data = "<input>")]
pub fn reassign_photo(input: Json<Value>) -> Status {
    let album_id = unwrap_or_return!(input.get_value::<i32>("album_id"), Status::BadRequest);
    let photo_ids = unwrap_or_return!(input.get_value::<Vec<i64>>("photo_ids"), Status::BadRequest);

    let mut conn = unwrap_or_return!(DB_POOL.get(), Status::InternalServerError);

    // Delete existing photo-album associations
    if remove_photo_from_album(&mut conn, &photo_ids).is_err() {
        return Status::InternalServerError;
    }

    // Create a new photo-album association
    if add_photo_to_album(&mut conn, album_id, &photo_ids).is_err() {
        return Status::InternalServerError;
    }
    
    Status::Ok
}


/// Remove a album from an album (turns album into a root album)
///
/// # Endpoint
/// `POST /management/album/<id>/unfile`
///
/// # Request Body
/// JSON object with:
/// - `album_ids`: Array of photo IDs to remove from all albums
///
/// # Returns
/// - `200 OK`: Album successfully removed from parent (move to root)
/// - `500 Internal Server Error`: Database or other server error occurred
#[post("/management/album/unfile", format = "json", data = "<input>")]
pub fn unfile_album(input: Json<Value>) -> Status {
    let album_ids = unwrap_or_return!(input.get_value::<Vec<i32>>("album_ids"), Status::BadRequest);
    let mut conn = unwrap_or_return!(DB_POOL.get(), Status::InternalServerError);

    match remove_album_from_album(&mut conn, &album_ids) {
        Ok(_) => Status::Ok,
        Err(_) => Status::InternalServerError,
    }
}

/// Moves albums from their current parent album(s) to a different one
///
/// # Endpoint
/// `POST /management/album/reassign`
///
/// # Request Body
/// JSON object with:
/// - `parent_id`: The ID of the destination album (i32)
/// - `album_ids`: Array of photo IDs to move to the new album (Vec<i64>)
///
/// # Returns
/// - `200 OK`: Photos were successfully moved to the new album
/// - `400 Bad Request`: Missing or invalid album_id or photo_ids in request body
/// - `500 Internal Server Error`: Database error or other server error occurred
#[post("/management/album/reassign", format = "json", data = "<input>")]
pub fn reassign_album(input: Json<Value>) -> Status {
    let parent_id = unwrap_or_return!(input.get_value::<i32>("album_id"), Status::BadRequest);
    let album_ids = unwrap_or_return!(input.get_value::<Vec<i32>>("photo_ids"), Status::BadRequest);

    let mut conn = unwrap_or_return!(DB_POOL.get(), Status::InternalServerError);

    // Delete existing photo-album associations
    if remove_album_from_album(&mut conn, &album_ids).is_err() {
        return Status::InternalServerError;
    }

    // Create a new photo-album association
    if add_album_to_album(&mut conn, parent_id, &album_ids).is_err() {
        return Status::InternalServerError;
    }

    Status::Ok
}

