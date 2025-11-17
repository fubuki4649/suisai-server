use crate::db::schema::album_album_join;
use crate::db::schema::albums::dsl as albums_dsl;
use crate::db::schema::albums::dsl::albums;
use crate::models::db::album::{Album, NewAlbum};
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

/// Deletes the specified album from the database
///
/// # Arguments
/// * `conn` - Database connection
/// * `album_id` - ID of the album to delete
///
/// # Returns
/// Number of rows affected (1 if successful, 0 if album not found)
pub fn delete_album(conn: &mut MysqlConnection, album_id: i32) -> Result<usize, Error> {
    diesel::delete(albums.find(album_id))
        .execute(conn)
}
