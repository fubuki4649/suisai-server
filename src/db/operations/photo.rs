use crate::models::photo::{NewPhoto, Photo};
use crate::db::schema::photos::dsl::photos;
use crate::db::schema::photos::id;
use diesel::insert_into;
use diesel::prelude::*;
use diesel::result::Error;

/// Creates a new photo entry in the database with associated metadata fields
///
/// # Arguments
/// * `conn` - Database connection pool
/// * `new_photo` - Photo details and metadata for creation
///
/// # Returns
/// Created photo record with generated ID if successful, or error if the insert fails
pub fn create_photo(conn: &mut PgConnection, new_photo: NewPhoto) -> Result<Photo, Error> {
    insert_into(photos)
        .values(&new_photo)
        .returning(Photo::as_returning())
        .get_result(conn)
}

/// Checks if a photo with the given hash value already exists in the database
///
/// # Arguments
/// * `conn` - Database connection pool
/// * `hash` - xxh3-128 hash value to check for duplicates
///
/// # Returns
/// * `Ok(Some(Photo))` - If a photo with a matching hash is found
/// * `Ok(None)` - If no photo with this hash exists
/// * `Err` - If the database query fails
pub fn check_hash(conn: &mut PgConnection, hash: &str) -> Result<Option<Photo>, Error> {
    photos
        .filter(crate::db::schema::photos::dsl::hash.eq(hash))
        .select(Photo::as_select())
        .first(conn)
        .map(Some)
        .or_else(|e| match e {
            Error::NotFound => Ok(None),
            err => Err(err),
        })
}

/// Gets photos by ID from the database
///
/// # Arguments
/// * `conn` - Database connection pool
/// * `photo_ids` - Slice of IDs to retrieve
///
/// # Returns
/// Vec<Photo> of found photos, which is empty if nothing is found
pub fn get_photo(conn: &mut PgConnection, photo_ids: &[i64]) -> Result<Vec<Photo>, Error> {
    if photo_ids.is_empty() { return Ok(vec![]); }
    
    photos
        .filter(id.eq_any(photo_ids))
        .select(Photo::as_select())
        .load(conn)
}

/// Deletes a photo from the database by its ID
///
/// # Arguments
/// * `conn` - Database connection pool
/// * `photo_ids` - Slice of IDs to delete
///
/// # Returns
/// Vec<Photo> of deleted photos, which is empty of none of the photos are found
pub fn delete_photo(conn: &mut PgConnection, photo_ids: &[i64]) -> Result<Vec<Photo>, Error> {
    diesel::delete(photos.filter(id.eq_any(photo_ids)))
        .get_results::<Photo>(conn)
}
