use crate::db::schema::thumbnails::dsl::{id, thumbnail_path, thumbnails};
use crate::models::db::thumbnail::Thumbnail;
use diesel::insert_into;
use diesel::prelude::*;
use diesel::result::Error;
use diesel::MysqlConnection;

/// Get thumbnail by Photo ID from the database
///
/// # Arguments
/// * `conn` - Database connection
/// * `photo_id` - Photo IDs to retrieve thumbnail for
///
/// # Returns
/// `Thumbnail` of found photo
pub fn get_thumbnail(conn: &mut MysqlConnection, photo_id: i64) -> Result<Thumbnail, Error> {
    thumbnails
        .find(photo_id)
        .get_result::<Thumbnail>(conn)
}

/// Create a thumbnail entry in the database
///
/// Inserts a row into the `thumbnails` table using the provided `Thumbnail` data.
/// The `id` corresponds to the associated photo's ID.
///
/// # Arguments
/// * `conn` - Database connection
/// * `thumb` - Thumbnail model containing `id` (photo id) and `thumbnail_path`
///
/// # Returns
/// * `Ok(())` on success
/// * `Err` if the insert fails (e.g., foreign key violation or duplicate key)
pub fn create_thumbnail(conn: &mut MysqlConnection, thumb: &Thumbnail) -> Result<(), Error> {
    insert_into(thumbnails)
        .values((id.eq(thumb.id), thumbnail_path.eq(&thumb.thumbnail_path)))
        .execute(conn)
        .map(|_| ())
}

/// Delete thumbnail entries by photo IDs
///
/// Removes rows from `thumbnails` whose `id` (photo id) matches any value in `photo_ids`.
/// If the provided slice is empty, this function is a no-op and returns `Ok(0)`.
///
/// # Arguments
/// * `conn` - Database connection
/// * `photo_ids` - Slice of photo IDs whose thumbnails should be deleted
///
/// # Returns
/// Number of rows deleted, or an error if the operation fails
pub fn delete_thumbnail(conn: &mut MysqlConnection, photo_ids: &[i64]) -> Result<usize, Error> {
    if photo_ids.is_empty() {
        return Ok(0);
    }

    diesel::delete(thumbnails.filter(id.eq_any(photo_ids))).execute(conn)
}

