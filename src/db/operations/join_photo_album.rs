use crate::db::models::album::AlbumPhoto;
use crate::db::schema::album_photos;
use diesel::insert_into;
use diesel::prelude::*;
use diesel::result::Error;


/// Adds a photo to an album by creating a new album-photo association
///
/// # Arguments
/// * `conn` - Database connection pool
/// * `photo_id` - ID of the photo to add
/// * `album_id` - ID of the album to add the photo to
///
/// # Returns
/// Number of rows affected (1 if successful, 0 if query fails)
pub fn add_photo_to_album(conn: &mut PgConnection, photo_id: i64, album_id: i32) -> Result<usize, Error> {
    let album_photo = AlbumPhoto {
        album_id,
        photo_id,
    };

    insert_into(album_photos::table)
        .values(&album_photo)
        .execute(conn)
}

/// Deletes a specific photo-album association from the database
///
/// # Arguments
/// * `conn` - Database connection pool
/// * `photo_id` - ID of the photo to remove
/// * `album_id` - ID of the album to remove photo from
///
/// # Returns
/// Number of associations removed (1 if successful, 0 if photo was not in album)
pub fn remove_photo_from_album(conn: &mut PgConnection, photo_id: i64, album_id: i32) -> Result<usize, Error> {
    let filter = album_photos::table
        .filter(album_photos::album_id.eq(album_id))
        .filter(album_photos::photo_id.eq(photo_id));

    diesel::delete(filter)
        .execute(conn)
}

/// Removes all album associations for a specific photo
///
/// # Arguments
/// * `conn` - Database connection pool
/// * `photo_id` - ID of the photo to remove from all albums
///
/// # Returns
/// Number of album associations removed for the photo
pub fn remove_photo_from_all_albums(conn: &mut PgConnection, photo_id: i64) -> Result<usize, Error> {
    let filter = album_photos::table.filter(album_photos::photo_id.eq(photo_id));
    
    diesel::delete(filter)
        .execute(conn)
}

/// Removes all photo associations from the specified album
///
/// # Arguments
/// * `conn` - Database connection pool
/// * `album_id` - ID of the album to clear
///
/// # Returns
/// Number of photo associations removed from the album
pub fn clear_album(conn: &mut PgConnection, album_id: i32) -> Result<usize, Error> {
    diesel::delete(album_photos::table.filter(album_photos::album_id.eq(album_id)))
        .execute(conn)
}
