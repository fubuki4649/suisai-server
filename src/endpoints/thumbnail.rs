use crate::db::operations::photo::check_hash;
use crate::{unwrap_or_return, DB_POOL};
use rocket::fs::NamedFile;
use rocket::get;
use rocket::http::Status;

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
pub async fn get_thumbnail(hash: &str) -> Result<NamedFile, Status> {
    let mut conn = unwrap_or_return!(DB_POOL.get(), Err(Status::InternalServerError));
    let photo = unwrap_or_return!(check_hash(&mut conn, hash), Err(Status::InternalServerError));

    match photo {
        Some(photo) => Ok(unwrap_or_return!(NamedFile::open(photo.thumbnail_path).await, Err(Status::InternalServerError))),
        None => Err(Status::NotFound),
    }
}