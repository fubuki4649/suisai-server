use crate::db::models::photo::*;
use crate::db::schema::album_photos;
use crate::db::schema::photos::dsl::photos;
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

/// Gets a photo by ID from the database
///
/// # Arguments
/// * `conn` - Database connection pool
/// * `photo_id` - ID of the photo to retrieve
///
/// # Returns
/// Photo with the given ID if found, otherwise NotFound error 
pub fn get_photo(conn: &mut PgConnection, photo_id: i64) -> Result<Photo, Error> {
    photos
        .find(photo_id)
        .select(Photo::as_select())
        .first(conn)
}

/// Gets all photos from the database
///
/// # Arguments
/// * `conn` - Database connection pool
///
/// # Returns
/// All photos found in the database, or an error if the query fails
pub fn get_all_photos(conn: &mut PgConnection) -> Result<Vec<Photo>, Error> {
    photos
        .select(Photo::as_select())
        .load(conn)
}

/// Updates an existing photo's fields in the database based on the provided photo's `id`
///
/// # Arguments
/// * `conn` - Database connection pool
/// * `photo` - Photo with updated fields and ID of record to update
///
/// # Returns
/// The updated photo if successful, or error if the photo was not found or if update fails
pub fn update_photo(conn: &mut PgConnection, photo: Photo) -> Result<Photo, Error> {
    diesel::update(photos.find(photo.id))
        .set(&photo)
        .returning(Photo::as_returning())
        .get_result(conn)
}

/// Deletes a photo from the database by its ID
///
/// # Arguments
/// * `conn` - Database connection pool
/// * `photo_id` - ID of the photo to delete
///
/// # Returns
/// Number of rows affected (1 if successful, 0 if photo not found)
pub fn delete_photo(conn: &mut PgConnection, photo_id: i64) -> Result<usize, Error> {
    diesel::delete(photos.find(photo_id))
        .execute(conn)
}

/// Gets all photos from the database that are not currently part of any album
///
/// Photos are compared against album_photos join table using a left outer join
/// to find records with no associated album entries.
///
/// # Arguments
/// * `conn` - Database connection pool  
///
/// # Returns
/// Vector of all photos not belonging to any album, or error if query fails
pub fn get_unfiled_photos(conn: &mut PgConnection) -> Result<Vec<Photo>, Error> {
    crate::db::schema::photos::table
        .left_outer_join(album_photos::table.on(album_photos::photo_id.eq(crate::db::schema::photos::id)))
        .filter(album_photos::photo_id.is_null()) // Only those with no match
        .select(crate::db::schema::photos::all_columns) // Select all fields from `photos`
        .load::<Photo>(conn)
}