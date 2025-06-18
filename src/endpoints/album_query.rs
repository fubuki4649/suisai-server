use crate::db::operations::album_query::{get_photos_in_album, get_photos_unfiled};
use crate::DB_POOL;
use anyhow::Result;
use rocket::get;
use rocket::http::Status;
use rocket::serde::json::Json;
use crate::models::photo_api::ApiReturnPhoto;


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
/// Array of ApiReturnPhoto objects containing metadata for each photo in the album
#[get("/album/<id>/photos")]
pub fn get_album_photos(id: i32) -> Result<Json<Vec<ApiReturnPhoto>>, Status> {
    crate::err_to_result_500!({
        let mut conn = DB_POOL.get()?;

        let album_photos = get_photos_in_album(&mut conn, id)?;
        Ok(Ok(Json(album_photos.into_iter().map(ApiReturnPhoto::from).collect())))
    })
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
/// Array of ApiReturnPhoto objects containing metadata for each unfiled photo
#[get("/album/unfiled/photos")]
pub fn get_unfiled_photos() -> Result<Json<Vec<ApiReturnPhoto>>, Status> {
    crate::err_to_result_500!({
        let mut conn = DB_POOL.get()?;

        let unfiled_photos = get_photos_unfiled(&mut conn)?;
        Ok(Ok(Json(unfiled_photos.into_iter().map(ApiReturnPhoto::from).collect())))
    })
}