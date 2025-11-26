use crate::db::schema::album_album_join;
use crate::db::schema::album_photo_join;
use crate::db::schema::albums::dsl as albums_dsl;
use crate::db::schema::albums::dsl::albums;
use crate::models::album::{Album, NewAlbum};
use diesel::associations::HasTable;
use diesel::insert_into;
use diesel::prelude::*;
use diesel::result::Error;

/// Creates a new album in the database
///
/// # Arguments
/// * `conn` - Database connection
/// * `album` - Album details for creation
///
/// # Returns
/// The number of rows affected (1 if successful, 0 if failed).
pub fn create_album(conn: &mut MysqlConnection, album: NewAlbum) -> Result<usize, Error> {
    insert_into(albums)
        .values(&album)
        .execute(conn)
}

/// Gets all albums from the database with their associated photos
///
/// # Arguments
/// * `conn` - Database connection
///
/// # Returns
/// All albums found in the database, or an error if query fails
pub fn get_root_albums(conn: &mut MysqlConnection) -> Result<Vec<Album>, Error> {
    albums::table()
        // LEFT JOIN album_album_join ON album_album_join.album_id = albums.id
        .left_outer_join(
            album_album_join::table.on(album_album_join::album_id.eq(albums_dsl::id))
        )
        // keep only rows where no parent entry exists → root albums
        .filter(album_album_join::parent_id.is_null())
        // select only album columns
        .select(Album::as_select())
        .load(conn)
}

/// Updates an existing album's name in the database based on the provided album's `id`
///
/// # Arguments
/// * `conn` - Database connection
/// * `album` - Album with updated fields and ID of record to update
///
/// # Returns
/// The number of rows affected (1 if successful, 0 if failed).
pub fn rename_album(conn: &mut MysqlConnection, album: Album) -> Result<usize, Error> {
    diesel::update(albums.find(album.id))
        .set(albums_dsl::album_name.eq(album.album_name))
        .execute(conn)
}

/// Gets albums by their IDs
///
/// # Arguments
/// * `conn` - Database connection
/// * `album_ids` - List of album IDs to retrieve
///
/// # Returns
/// Vector of albums matching the provided IDs, or an error if query fails
pub fn get_album(conn: &mut MysqlConnection, album_ids: &[i32]) -> Result<Vec<Album>, Error> {
    albums
        .filter(albums_dsl::id.eq_any(album_ids))
        .load::<Album>(conn)
}

/// Gets the album that contains the specified photo
///
/// # Arguments
/// * `conn` - Database connection
/// * `photo_id` - ID of the photo to find the album for
///
/// # Returns
/// The album containing the photo, or an error if not found
///
/// IMPORTANT: If the photo is unfiled, this function will return a `NotFound` error
pub fn get_album_by_photo(conn: &mut MysqlConnection, photo_id: i64) -> Result<Album, Error> {
    albums
        .inner_join(album_photo_join::table.on(album_photo_join::parent_id.eq(albums_dsl::id)))
        .filter(album_photo_join::photo_id.eq(photo_id))
        .select(Album::as_select())
        .first(conn)
}

/// Deletes the specified album from the database
///
/// # Arguments
/// * `conn` - Database connection
/// * `album_id` - ID of the album to delete
///
/// # Returns
/// The deleted `Album` if found, or an error if the album doesn't exist
pub fn delete_album(conn: &mut MysqlConnection, album_id: i32) -> Result<Album, Error> {
    // First fetch the album
    let album = albums.find(album_id).first::<Album>(conn)?;

    // Then delete it
    diesel::delete(albums.find(album_id)).execute(conn)?;

    Ok(album)
}