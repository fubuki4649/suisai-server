use crate::db::models::db_album::{AlbumPhoto, DBAlbum};
use crate::db::schema::{album_photos, albums, photos};
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
/// Created album-photo association record if successful
pub fn add_photo_to_album(conn: &mut PgConnection, photo_id: i64, album_id: i32) -> Result<AlbumPhoto, Error> {
    let album_photo = AlbumPhoto {
        album_id,
        photo_id,
    };

    insert_into(album_photos::table)
        .values(&album_photo)
        .returning(AlbumPhoto::as_returning())
        .get_result(conn)
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
    diesel::delete(
        album_photos::table
            .filter(album_photos::photo_id.eq(photo_id))
    )
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
    diesel::delete(
        album_photos::table
            .filter(album_photos::album_id.eq(album_id))
            .filter(album_photos::photo_id.eq(photo_id))
    )
        .execute(conn)
}

/// Retrieves all photo IDs associated with the specified album
///
/// # Arguments
/// * `conn` - Database connection pool
/// * `album_id` - ID of the album to get photos from
///
/// # Returns
/// Vec of photo IDs belonging to the album, or error if query fails
pub fn get_photos_in_album(conn: &mut PgConnection, album_id: i32) -> Result<Vec<i64>, Error> {
    album_photos::table
        .filter(album_photos::album_id.eq(album_id))
        .inner_join(photos::table)
        .select(album_photos::photo_id)
        .load::<i64>(conn)
}

/// Retrieves all albums that contain a specific photo
///
/// # Arguments
/// * `conn` - Database connection pool
/// * `photo_id` - ID of the photo to check
///
/// # Returns
/// Vec of album IDs containing the specified photo
pub fn get_albums_containing_photo(conn: &mut PgConnection, photo_id: i64) -> Result<Vec<i32>, Error> {
    let db_albums = album_photos::table
        .filter(album_photos::photo_id.eq(photo_id))
        .inner_join(albums::table)
        .select(DBAlbum::as_select())
        .load(conn)?;

    Ok(
        db_albums
            .into_iter()
            .map(|db_album| db_album.id)
            .collect()
    )
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
