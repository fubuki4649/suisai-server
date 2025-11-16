use crate::db::schema::album_album_join;
use crate::models::album::AlbumAlbum;
use diesel::insert_into;
use diesel::mysql::MysqlConnection;
use diesel::prelude::*;
use diesel::result::Error;

/// Adds albums to a parent album by creating new album-album associations
///
/// # Arguments
/// * `conn` - Database connection
/// * `parent_id` - ID of the parent album
/// * `album_ids` - Slice of child album IDs to add
///
/// # Returns
/// Number of rows affected (number of albums successfully added)
pub fn add_album_to_album(conn: &mut MysqlConnection, parent_id: i32, album_ids: &[i32]) -> Result<usize, Error> {
    let album_albums: Vec<AlbumAlbum> = album_ids
        .iter()
        .map(|&album_id| AlbumAlbum {
            parent_id,
            album_id,
        })
        .collect();

    insert_into(album_album_join::table)
        .values(&album_albums)
        .execute(conn)
}

/// Removes album associations for the specified albums (turns them into root-level albums)
///
/// # Arguments
/// * `conn` - Database connection
/// * `album_ids` - Slice of album IDs to remove from all albums
///
/// # Returns
/// Number of rows affected (number of associations removed)
pub fn remove_album_from_album(conn: &mut MysqlConnection, album_ids: &[i32]) -> Result<usize, Error> {
    let filter = album_album_join::table.filter(album_album_join::album_id.eq_any(album_ids));

    diesel::delete(filter)
        .execute(conn)
}