use crate::_utils::json_map::JsonMap;
use crate::db::models::photo::Photo;
use crate::db::operations::photo::{delete_photo, get_photo};
use crate::{unwrap_or_return, DB_POOL};
use diesel::result::Error;
use rocket::http::Status;
use rocket::serde::json::{Json, Value};
use rocket::{delete, get};

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
pub fn del_photo(input: Json<Value>) -> Status {
    let photo_ids = unwrap_or_return!(input.get_value::<Vec<i64>>("photo_ids"), Status::BadRequest);

    crate::err_to_500!({
        let mut conn = DB_POOL.get()?;

        for id in photo_ids.iter() {
            // Ignore `Error::NotFound`
            if let Err(e) = delete_photo(&mut conn, id) {
                if e != Error::NotFound {
                    return Err(e.into());
                }
            }
        }

        Ok(Status::Ok)
    })
}


/// Retrieve multiple photos by their IDs
///
/// # Route
/// `GET /photo/get`
///
/// # Request Body
/// JSON object with:
/// - `photo_ids`: JSON array of photo IDs to retrieve
///
/// # Returns
/// - `Ok(Json<Vec<Photo>>)` containing an array of found photos
///   (skips any IDs that don't exist)
/// - `Status::InternalServerError` (500) if retrieval fails
#[get("/photo/get", format = "json", data = "<input>")]
pub fn get_photos(input: Json<Value>) -> Result<Json<Vec<Photo>>, Status> {
    let photo_ids = unwrap_or_return!(input.get_value::<Vec<i64>>("photo_ids"), Err(Status::BadRequest));

    crate::err_to_result_500!({
        let mut conn = DB_POOL.get()?;

        let photos = get_photo(&mut conn, &photo_ids)?;
        Ok(Ok(Json(photos)))
    })
}
