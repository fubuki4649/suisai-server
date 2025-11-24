use crate::db::operations::thumbnail::delete_thumbnail;
use crate::db::schema::photos::dsl::{id, photos};
use crate::models::photo::{NewPhoto, Photo};
use diesel::insert_into;
use diesel::mysql::MysqlConnection;
use diesel::prelude::*;
use diesel::result::Error;

/// Creates a new photo entry in the database with associated metadata fields
///
/// # Arguments
/// * `conn` - Database connection
/// * `new_photo` - Photo details and metadata for creation
///
/// # Returns
/// Ok(id) if successful with the ID of the new photo, or error if the insert fails
pub fn create_photo(conn: &mut MysqlConnection, new_photo: NewPhoto) -> Result<i64, Error> {
    // Insert the new photo
    insert_into(photos)
        .values(&new_photo)
        .execute(conn)?;

    // Get the last inserted ID
    diesel::select(diesel::dsl::sql::<diesel::sql_types::BigInt>("LAST_INSERT_ID()"))
        .get_result(conn)
}

/// Checks if a photo with the given hash value already exists in the database
///
/// # Arguments
/// * `conn` - Database connection
/// * `hash` - xxh3-128 hash value to check for duplicates
///
/// # Returns
/// * `Ok(Some(Photo))` - If a photo with a matching hash is found
/// * `Ok(None)` - If no photo with this hash exists
/// * `Err` - If the database query fails
pub fn check_hash(conn: &mut MysqlConnection, hash: &str) -> Result<Option<Photo>, Error> {
    photos
        .filter(crate::db::schema::photos::dsl::hash.eq(hash))
        .first::<Photo>(conn)
        .map(Some)
        .or_else(|e| match e {
            Error::NotFound => Ok(None),
            err => Err(err),
        })
}

/// Gets photos by ID from the database
///
/// # Arguments
/// * `conn` - Database connection
/// * `photo_ids` - Slice of IDs to retrieve
///
/// # Returns
/// Vec<Photo> of found photos, which is empty if nothing is found
pub fn get_photo(conn: &mut MysqlConnection, photo_ids: &[i64]) -> Result<Vec<Photo>, Error> {
    if photo_ids.is_empty() { return Ok(vec![]); }

    photos
        .filter(id.eq_any(photo_ids))
        .load::<Photo>(conn)
}

/// Deletes a photo from the database by its ID
///
/// # Arguments
/// * `conn` - Database connection
/// * `photo_ids` - Slice of IDs to delete
///
/// # Returns
/// Vec<Photo> of deleted photos, which is empty if none of the photos are found
pub fn delete_photo(conn: &mut MysqlConnection, photo_ids: &[i64]) -> Result<Vec<Photo>, Error> {
    if photo_ids.is_empty() { return Ok(vec![]); }

    // First fetch the photos
    let deleted = photos
        .filter(id.eq_any(photo_ids))
        .load::<Photo>(conn)?;

    // Then delete them
    diesel::delete(photos.filter(id.eq_any(photo_ids)))
        .execute(conn)?;

    // Also delete associated thumbnails
    delete_thumbnail(conn, photo_ids)?;

    Ok(deleted)
}
