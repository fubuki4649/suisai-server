use crate::db::operations::photo::check_hash;
use crate::db::operations::thumbnail::get_thumbnail as get_thumbs;
use crate::{msg, unwrap_err, DB_POOL};
use rocket::fs::NamedFile;
use rocket::get;
use rocket::http::Status;
use rocket::serde::json::Json;
use serde_json::Value;

/// Hash-based thumbnail serving
///
/// # Route
/// `GET /thumbnail/<hash>`
///
/// # Returns
/// - `200 OK`: The thumbnail for the image with <hash>, in JPEG format
/// - `404 Not Found`: No image with hash <hash> was found
/// - `500 Internal Server Error`: Database or other server error occurred
#[get("/thumbnail/<hash>")]
pub async fn get_thumbnail(hash: &str) -> Result<NamedFile, (Status, Json<Value>)> {
    let mut conn = unwrap_err!(DB_POOL.get(), Status::InternalServerError);
    let photo = unwrap_err!(check_hash(&mut conn, hash), Status::InternalServerError);

    match photo {
        Some(photo) => {
            let thumb = unwrap_err!(get_thumbs(&mut conn, photo.id), Status::InternalServerError);
            Ok(unwrap_err!(NamedFile::open(thumb.thumbnail_path).await, Status::InternalServerError))
        },
        None => Err((Status::NotFound, msg!("No photo with hash {} found"))),
    }
}