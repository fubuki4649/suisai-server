use crate::db::models::photo::*;
use crate::db::operations::photo::{create_photo, delete_photo, get_photo};
use crate::DB_POOL;
use diesel::result::Error;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{delete, get, post};


/// Create a new photo in the database
///
/// # Route
/// `POST /photo/new`  
///
/// # Input
/// JSON request body containing photo metadata with fields:
/// - `thumbnail_url` - URL of the photo's thumbnail
/// - `hash` - xxh3-128 hash value of the photo  
/// - `file_name` - File name including extension
/// - `file_path` - Full path to storage location
/// - `size_on_disk` - Size of the photo file
/// - `photo_date` - When the photo was taken (ISO 8601 format)
/// - `photo_timezone` - Timezone of photo timestamp 
/// - `resolution` - Width and height dimensions [width, height]
/// - `mime_type` - MIME type of the photo
/// - `camera_model` - Camera used
/// - `lens_model` - Lens used  
/// - `shutter_count` - Camera shutter actuation count
/// - `focal_length` - Lens focal length
/// - `iso` - ISO sensitivity
/// - `shutter_speed` - Shutter speed as string 
/// - `aperture` - Lens aperture
///
/// # Returns
/// - `Status::Created` (201) if photo created successfully
/// - `Status::InternalServerError` (500) if creation fails
#[post("/photo/new", format = "json", data = "<input>")]
pub fn new_photo(input: Json<NewPhoto>) -> Status {
    crate::err_to_500!({
        let mut conn = DB_POOL.get()?;
        create_photo(&mut conn, input.into_inner())?;
        
        Ok(Status::Created)
    })
}

/// Delete a photo from the database by its ID
///
/// # Route  
/// `DELETE /photo/<id>/delete`
///
/// # Parameters
/// - `id`: The unique identifier of the photo to delete
///
/// # Returns
/// - `Status::Ok` (200) if the photo was successfully deleted
/// - `Status::NotFound` (404) if the photo does not exist
/// - `Status::InternalServerError` (500) if deletion fails
#[delete("/photo/<id>/delete")]
pub fn del_photo(id: i64) -> Status {
    crate::err_to_500!({
        let mut conn = DB_POOL.get()?;

        match delete_photo(&mut conn, id) {
            Ok(_) => Ok(Status::Ok),
            Err(Error::NotFound) => Ok(Status::NotFound),
            Err(e) => Err(e.into()),
        }
    })
}

/// Retrieve a single photo by its ID
///
/// # Route
/// `GET /photo/<id>`
///
/// # Parameters
/// - `id`: The unique identifier of the photo to retrieve
///
/// # Returns
/// - `Ok(Json<Photo>)` containing the photo data if found
/// - `Status::NotFound` (404) if the photo does not exist
/// - `Status::InternalServerError` (500) if retrieval fails
#[get("/photo/<id>")]
pub fn get_photo_single(id: i64) -> Result<Json<Photo>, Status> {
    crate::err_to_result_500!({
        let mut conn = DB_POOL.get()?;
        match get_photo(&mut conn, id) {
            Ok(photo) => Ok(Ok(Json(photo))),
            Err(Error::NotFound) => Ok(Err(Status::NotFound)),
            Err(e) => Err(e.into()),
        }
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
pub fn get_photo_multi(ids: Json<Vec<i64>>) -> Result<Json<Vec<Photo>>, Status> {
    crate::err_to_result_500!({
        let id_vec = ids.into_inner();
        
        let mut conn = DB_POOL.get()?;
        let mut photos: Vec<Photo> = Vec::with_capacity(id_vec.len());

        for id in id_vec.iter() {
            if let Ok(photo) = get_photo(&mut conn, *id) {
                photos.push(photo);
            }
        }

        Ok(Ok(Json(photos)))
    })
}
