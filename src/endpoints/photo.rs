use crate::db::models::photo::{Photo};
use crate::db::operations::photo::{delete_photo, get_photo};
use crate::DB_POOL;
use diesel::result::Error;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{delete, get};


/// Delete multiple photos from the database by their IDs
///
/// # Route  
/// `DELETE /photo/delete`
///
/// # Parameters
/// JSON array of photo IDs to delete
///
/// # Returns
/// - `Status::Ok` (200) if the photos were successfully deleted
///   (will succeed even if some photos did not exist)
/// - `Status::InternalServerError` (500) if deletion fails for reasons other than missing photos
#[delete("/photo/delete", format = "json", data = "<ids>")]
pub fn del_photo(ids: Json<Vec<i64>>) -> Status {
    crate::err_to_500!({
        let id_vec = ids.into_inner();
        let mut conn = DB_POOL.get()?;

        for id in id_vec.iter() {
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
/// # Input
/// JSON array containing photo IDs to retrieve
///
/// # Returns
/// - `Ok(Json<Vec<Photo>>)` containing an array of found photos
///   (skips any IDs that don't exist)
/// - `Status::InternalServerError` (500) if retrieval fails
#[get("/photo/get", format = "json", data = "<ids>")]
pub fn get_photos(ids: Json<Vec<i64>>) -> Result<Json<Vec<Photo>>, Status> {
    crate::err_to_result_500!({
        let id_vec = ids.into_inner();
        let mut conn = DB_POOL.get()?;

        let photos = get_photo(&mut conn, &id_vec)?;
        Ok(Ok(Json(photos)))
    })
}
