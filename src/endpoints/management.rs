use crate::_utils::json_map::JsonMap;
use crate::db::operations::album::{get_album, get_album_by_photo};
use crate::db::operations::join_album_album::{add_album_to_album, remove_album_from_album};
use crate::db::operations::join_album_photo::{add_photo_to_album, remove_photo_from_album};
use crate::db::operations::paths::get_album_path;
use crate::db::operations::photo::get_photo;
use crate::fs_operations::album::move_album_fs;
use crate::fs_operations::photo::move_photo_fs;
use crate::{msg, unwrap_ret, DB_POOL};
use anyhow::Error;
use rocket::http::Status;
use rocket::post;
use rocket::serde::json::{Json, Value};
use std::path::PathBuf;

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
/// - `400 Bad Request`: Missing or invalid album_ids or photo_ids in request body
/// - `500 Internal Server Error`: Full or partial error occurred. Query may not be fully executed
#[post("/management/photo/unfile", format = "json", data = "<input>")]
pub fn unfile_photo(input: Json<Value>) -> (Status, Json<Value>) {
    let photo_ids = unwrap_ret!(input.get_value::<Vec<i64>>("photo_ids"), Status::BadRequest);
    let mut conn = unwrap_ret!(DB_POOL.get(), Status::InternalServerError);

    let photos = unwrap_ret!(get_photo(&mut conn, &photo_ids), Status::InternalServerError);
    for photo in photos {
        // Get path to parent album
        let album_path =  get_album_by_photo(&mut conn, photo.id)
            .and_then(|album| get_album_path(&mut conn, album.id));

        match album_path {
            Ok(path) => {
                // Move photo and associated files
                unwrap_ret!(move_photo_fs(&path.join(photo.file_name), &PathBuf::from("/unfiled")), Status::InternalServerError);
                // Move photo and associated files
                unwrap_ret!(remove_photo_from_album(&mut conn, &[photo.id]), Status::InternalServerError);
            }
            Err(err) => {
                return (Status::InternalServerError, msg!(err.to_string()));
            }
        }
    }

    (Status::Ok, msg!("Success"))
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
/// - `500 Internal Server Error`: Full or partial error occurred. Query may not be fully executed
#[post("/management/photo/reassign", format = "json", data = "<input>")]
pub fn reassign_photo(input: Json<Value>) -> (Status, Json<Value>) {
    let album_id = unwrap_ret!(input.get_value::<i32>("album_id"), Status::BadRequest);
    let photo_ids = unwrap_ret!(input.get_value::<Vec<i64>>("photo_ids"), Status::BadRequest);

    let mut conn = unwrap_ret!(DB_POOL.get(), Status::InternalServerError);


    let photos = unwrap_ret!(get_photo(&mut conn, &photo_ids), Status::InternalServerError);
    for photo in photos {
        // Get album ID of photo
        let src_path = get_album_by_photo(&mut conn, photo.id).map_err(Error::from)
            .and_then(|album| get_album_path(&mut conn, album.id).map_err(Error::from));

        // Move photo and associated files
        match src_path {
            Ok(src) => {
                let dest_path = unwrap_ret!(get_album_path(&mut conn, album_id), Status::InternalServerError);
                unwrap_ret!(move_photo_fs(&src.join(photo.file_name), &dest_path), Status::InternalServerError);
            }
            Err(err) => {
                return (Status::InternalServerError, msg!(err.to_string()));
            }
        };

        // Delete existing photo-album associations
        unwrap_ret!(remove_photo_from_album(&mut conn, &[photo.id]), Status::InternalServerError);

        // Create a new photo-album association
        unwrap_ret!(add_photo_to_album(&mut conn, album_id, &[photo.id]), Status::InternalServerError);
    }
    
    (Status::Ok, msg!("Success"))
}


/// Remove an album from another (parent) album (turns album into a root album)
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
/// - `400 Bad Request`: Missing or invalid album_ids or photo_ids in request body
/// - `500 Internal Server Error`: Database or other server error occurred
#[post("/management/album/unfile", format = "json", data = "<input>")]
pub fn unfile_album(input: Json<Value>) -> (Status, Json<Value>) {
    let album_ids = unwrap_ret!(input.get_value::<Vec<i32>>("album_ids"), Status::BadRequest);
    let mut conn = unwrap_ret!(DB_POOL.get(), Status::InternalServerError);

    let albums = unwrap_ret!(get_album(&mut conn, &album_ids), Status::InternalServerError);
    for album in albums {
        // Move album to root
        let album_path = unwrap_ret!(get_album_path(&mut conn, album.id), Status::InternalServerError);
        unwrap_ret!(move_album_fs(&album_path, &PathBuf::from("/")), Status::InternalServerError);

        // Reflect change in DB
        unwrap_ret!(remove_album_from_album(&mut conn, &[album.id]), Status::InternalServerError);
    }

    (Status::Ok, msg!("Success"))
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
pub fn reassign_album(input: Json<Value>) -> (Status, Json<Value>) {
    let parent_id = unwrap_ret!(input.get_value::<i32>("album_id"), Status::BadRequest);
    let album_ids = unwrap_ret!(input.get_value::<Vec<i32>>("photo_ids"), Status::BadRequest);

    let mut conn = unwrap_ret!(DB_POOL.get(), Status::InternalServerError);

    let albums = unwrap_ret!(get_album(&mut conn, &album_ids), Status::InternalServerError);
    let dest_path = unwrap_ret!(get_album_path(&mut conn, parent_id), Status::InternalServerError);
    for album in albums {
        // Move album to new parent
        let album_path = unwrap_ret!(get_album_path(&mut conn, album.id), Status::InternalServerError);
        unwrap_ret!(move_album_fs(&album_path, &dest_path), Status::InternalServerError);

        // Reflect changes in DB
        unwrap_ret!(remove_album_from_album(&mut conn, &[album.id]), Status::InternalServerError);
        unwrap_ret!(add_album_to_album(&mut conn, parent_id, &[album.id]), Status::InternalServerError);
    }

    (Status::Ok, msg!("Success"))
}

