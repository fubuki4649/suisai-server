use crate::_utils::json_map::JsonMap;
use crate::db::operations::join_photo_album::{add_photo_to_album, remove_photo_from_all_albums};
use crate::{unwrap_or_return, DB_POOL};
use rocket::http::Status;
use rocket::post;
use rocket::serde::json::{Json, Value};


/// Assigns a photo to an album
///
/// # Endpoint
/// `POST /photo/<id>/assign-album`
///
/// # URL Parameters
/// - `id`: The ID of the photo to assign (i64)
///
/// # Request Body  
/// JSON object with:
/// - `album_id`: The ID of the album to assign the photo to (i32)
///
/// # Returns
/// - `200 OK`: Photo was successfully assigned to the album
/// - `400 Bad Request`: Missing or invalid album_id in request body
/// - `500 Internal Server Error`: Database or other server error occurred
#[post("/photo/<id>/assign-album", format = "json", data = "<input>")]
pub fn photo_assign_album(id: i64, input: Json<Value>) -> Status {
    let album_id = unwrap_or_return!(input.get_value::<i32>("album_id"), Status::BadRequest);
    crate::err_to_500!({
        let mut conn = DB_POOL.get()?;
        add_photo_to_album(&mut conn, id, album_id)?;
        Ok(Status::Ok)
    })
}

/// Removes a photo from all albums it is currently assigned to
///
/// # Endpoint
/// `POST /photo/<id>/clear-album`
///
/// # URL Parameters
/// - `id`: The ID of the photo to remove from all albums (i64)
///
/// # Returns
/// - `200 OK`: Photo was successfully removed from all albums
/// - `500 Internal Server Error`: Database or other server error occurred
#[post("/photo/<id>/clear-album")]
pub fn photo_clear_album(id: i64) -> Status {
    crate::err_to_500!({
        let mut conn = DB_POOL.get()?;
        remove_photo_from_all_albums(&mut conn, id)?;
        Ok(Status::Ok)
    })
}

/// Moves a photo from its current album(s) to a different album
///
/// # Endpoint
/// `POST /photo/<id>/move-album`
///
/// # Request Parameters
/// - `id`: The ID of the photo to move (i64)
///
/// # Request Body
/// JSON object with:
/// - `album_id`: The ID of the destination album (i32)
///
/// # Returns
/// - `200 OK`: Photo was successfully moved to the new album
/// - `400 Bad Request`: Missing or invalid album_id in request body
/// - `500 Internal Server Error`: Database or other server error occurred
#[post("/photo/<id>/move-album", format = "json", data = "<input>")]
pub fn photo_move_album(id: i64, input: Json<Value>) -> Status {
    let album_id = unwrap_or_return!(input.get_value::<i32>("album_id"), Status::BadRequest);
    crate::err_to_500!({
        let mut conn = DB_POOL.get()?;
        
        // Delete existing photo-album associations
        remove_photo_from_all_albums(&mut conn, id)?;
        
        // Create a new photo-album association
        add_photo_to_album(&mut conn, id, album_id)?;
        
        Ok(Status::Ok)
    })
}
