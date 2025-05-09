use crate::db::models::album::AlbumPhoto;
use crate::db::schema::album_photos;
use diesel::insert_into;
use diesel::prelude::*;
use diesel::result::Error;


/// Adds photos to an album by creating new album-photo associations
///
/// # Arguments
/// * `conn` - Database connection pool
/// * `album_id` - ID of the album to add the photos to
/// * `photo_ids` - Array of photo IDs to add to the album
///
/// # Returns
/// Number of rows affected (matching number of photos if successful, 0 if empty array provided)
pub fn add_photo_to_album(conn: &mut PgConnection, album_id: i32, photo_ids: &[i64]) -> Result<usize, Error> {
    if photo_ids.is_empty() { return Ok(0); }

    // Create a Vec<AlbumPhoto> to insert
    let album_photos = photo_ids.iter().map(|photo_id| AlbumPhoto {
        album_id,
        photo_id: *photo_id,
    }).collect::<Vec<AlbumPhoto>>();

    insert_into(album_photos::table)
        .values(&album_photos)
        .execute(conn)
}

/// Removes all album associations for specified photos
///
/// # Arguments
/// * `conn` - Database connection pool
/// * `photo_ids` - Array of photo IDs to remove from all albums
///
/// # Returns
/// Number of album associations removed for the photos
pub fn remove_photo_from_album(conn: &mut PgConnection, photo_ids: &[i64]) -> Result<usize, Error> {
    if photo_ids.is_empty() { return Ok(0); }

    let filter = album_photos::table.filter(album_photos::photo_id.eq_any(photo_ids));

    diesel::delete(filter)
        .execute(conn)
}
