use crate::_utils::json_map::JsonMap;
use crate::db::operations::paths::get_photo_path;
use crate::db::operations::photo::{delete_photo, get_photo};
use crate::db::operations::thumbnail::get_thumbnail;
use crate::fs_operations::photo::delete_photo_fs;
use crate::models::photo::Photo;
use crate::{msg, unwrap_err, unwrap_ret, DB_POOL};
use rocket::http::Status;
use rocket::serde::json::{Json, Value};
use rocket::{delete, post};
use std::path::PathBuf;

/// Delete multiple photos from the database by their IDs
///
/// # Route  
/// `DELETE /photo/delete`
///
/// # Request Body
/// JSON object with:
/// - `photo_ids`: JSON array of photo IDs to delete
///
/// # Returns
/// - `Status::Ok` (200) if the photos were successfully deleted
///   (will succeed even if some photos did not exist)
/// - `Status::InternalServerError` (500) if deletion fails for reasons other than missing photos
#[delete("/photo/delete", format = "json", data = "<input>")]
pub fn del_photo(input: Json<Value>) -> (Status, Json<Value>) {
    let photo_ids = unwrap_ret!(input.get_value::<Vec<i64>>("photo_ids"), Status::BadRequest);
    let mut conn = unwrap_ret!(DB_POOL.get(), Status::InternalServerError);
    
    // Delete photos from DB
    let deleted = unwrap_ret!(delete_photo(&mut conn, &photo_ids), Status::InternalServerError);
    
    // Also delete photos & thumbnail from filesystem, and ignore nonexistent/permission errors
    for photo in deleted {
        let photo_path = unwrap_ret!(get_photo_path(&mut conn, photo.id), Status::InternalServerError);
        let thumb_path = unwrap_ret!(get_thumbnail(&mut conn, photo.id), Status::InternalServerError).thumbnail_path;

        unwrap_ret!(delete_photo_fs(&photo_path, &PathBuf::from(thumb_path)), Status::InternalServerError);
    }

    (Status::Ok, msg!("Success"))
}


/// Retrieve multiple photos by their IDs
///
/// # Route
/// `POST /photo/get`
///
/// # Request Body
/// JSON object with:
/// - `photo_ids`: JSON array of photo IDs to retrieve
///
/// # Returns
/// - `Ok(Json<Vec<ApiReturnPhoto>>)` containing an array of found photos
///   (skips any IDs that don't exist)
/// - `Status::InternalServerError` (500) if retrieval fails
#[post("/photo/get", format = "json", data = "<input>")]
pub fn get_photos(input: Json<Value>) -> Result<Json<Vec<Photo>>, (Status, Json<Value>)> {
    let photo_ids = unwrap_err!(input.get_value::<Vec<i64>>("photo_ids"), Status::BadRequest);
    let mut conn = unwrap_err!(DB_POOL.get(), Status::InternalServerError);

    let photos = unwrap_err!(get_photo(&mut conn, &photo_ids), Status::InternalServerError);
    Ok(Json(photos))
}
