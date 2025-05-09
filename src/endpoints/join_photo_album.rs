use crate::_utils::json_map::JsonMap;
use crate::db::operations::join_photo_album::{add_photo_to_album, remove_photo_from_album};
use crate::{unwrap_or_return, DB_POOL};
use rocket::http::Status;
use rocket::post;
use rocket::serde::json::{Json, Value};


/// Removes photos from all albums they are currently assigned to
///
/// # Endpoint
/// `POST /photo/album/unfile`
///
/// # URL Parameters
/// - `ids`: The IDs of the photos to remove from all albums (Vec<i64>)
///
/// # Returns
/// - `200 OK`: Photos were successfully removed from all albums
/// - `500 Internal Server Error`: Database or other server error occurred 
#[post("/photo/album/unfile", format = "json", data = "<ids>")]
pub fn photo_clear_album(ids: Json<Vec<i64>>) -> Status {
    crate::err_to_500!({
        let id_vec = ids.into_inner();
        let mut conn = DB_POOL.get()?;

        remove_photo_from_album(&mut conn, &id_vec)?;
        Ok(Status::Ok)
    })
}

/// Moves photos from their current album(s) to a different album
///
/// # Endpoint
/// `POST /photo/album/reassign`
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
#[post("/photo/album/reassign", format = "json", data = "<input>")]
pub fn photo_move_album(input: Json<Value>) -> Status {
    let album_id = unwrap_or_return!(input.get_value::<i32>("album_id"), Status::BadRequest);
    let photo_ids = unwrap_or_return!(input.get_value::<Vec<i64>>("album_id"), Status::BadRequest);

    crate::err_to_500!({
        let mut conn = DB_POOL.get()?;

        // Delete existing photo-album associations
        remove_photo_from_album(&mut conn, &photo_ids)?;

        // Create a new photo-album association
        add_photo_to_album(&mut conn, album_id, &photo_ids)?;

        Ok(Status::Ok)
    })
}
