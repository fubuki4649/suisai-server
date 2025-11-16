use crate::db::schema::album_album_join::dsl as join_dsl;
use crate::db::schema::albums::dsl as albums_dsl;
use crate::db::schema::albums::dsl::albums;
use crate::db::schema::{album_photo_join, photos};
use crate::models::album::Album;
use crate::models::photo::Photo;
use diesel::prelude::*;
use diesel::result::Error;

/// Retrieves all photos associated with the specified album
///
/// # Arguments
/// * `conn` - Database connection pool
/// * `album_id` - ID of the album to get photos from
///
/// # Returns
/// Vec of all photos belonging to the album, or error if query fails
pub fn get_photos_in_album(conn: &mut MysqlConnection, album_id: i32) -> Result<Vec<Photo>, Error> {
    album_photo_join::table
        .filter(album_photo_join::parent_id.eq(album_id))
        .inner_join(photos::table)
        .select(photos::all_columns) // Select all fields from `photos`
        .load::<Photo>(conn)
}

/// Retrieves all subalbums associated with the specified album
///
/// # Arguments
/// * `conn` - Database connection pool
/// * `album_id` - ID of the album to get photos from
///
/// # Returns
/// Vec of all subalbums belonging to the album, or error if query fails
pub fn get_albums_in_album(conn: &mut MysqlConnection, album_id: i32) -> Result<Vec<Album>, Error> {
    join_dsl::album_album_join
        .filter(join_dsl::parent_id.eq(album_id))
        .inner_join(albums.on(join_dsl::album_id.eq(albums_dsl::id)))
        .select(albums::all_columns())
        .load::<Album>(conn)
}

/// Gets all photos from the database that are not currently part of any album
///
/// Photos are compared against `album_photos` join table using a left outer join
/// to find records with no associated album entries.
///
/// # Arguments
/// * `conn` - Database connection pool
///
/// # Returns
/// Vec of all photos not belonging to any album, or error if query fails
pub fn get_photos_unfiled(conn: &mut MysqlConnection) -> Result<Vec<Photo>, Error> {
    photos::table
        .left_outer_join(album_photo_join::table.on(album_photo_join::photo_id.eq(photos::id)))
        .filter(album_photo_join::photo_id.is_null()) // Only those with no match
        .select(photos::all_columns) // Select all fields from `photos`
        .load::<Photo>(conn)
}
