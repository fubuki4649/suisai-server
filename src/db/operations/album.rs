use crate::db::models::album::*;
use crate::db::models::db_album::DBAlbum;
use crate::db::schema::albums::dsl::albums;
use diesel::insert_into;
use diesel::prelude::*;
use diesel::result::Error;

/// Creates a new album in the database
///
/// # Arguments  
/// * `conn` - Database connection pool
/// * `album` - Album details for creation
///
/// # Returns
/// Created album with generated ID and photos list (initially empty)
pub fn create_album(conn: &mut PgConnection, album: NewAlbum) -> Result<Album, Error> {
    insert_into(albums)
        .values(&album)
        .returning(DBAlbum::as_returning())
        .get_result(conn)?
        .try_into_album(conn)
}

/// Gets an album by ID from the database
///
/// # Arguments
/// * `conn` - Database connection pool
/// * `album_id` - ID of the album to retrieve
///
/// # Returns
/// Album with the given ID if found, otherwise NotFound error
pub fn get_album(conn: &mut PgConnection, album_id: i32) -> Result<Album, Error> {
    albums
        .find(album_id)
        .select(DBAlbum::as_select())
        .first::<DBAlbum>(conn)?
        .try_into_album(conn)
}

/// Gets all albums from the database with their associated photos
///
/// # Arguments
/// * `conn` - Database connection pool
///
/// # Returns
/// All albums found in the database, or an error if query fails
pub fn get_all_albums(conn: &mut PgConnection) -> Result<Vec<Album>, Error> {
    let db_albums = albums
        .select(DBAlbum::as_select())
        .load(conn)?;

    db_albums
        .into_iter()
        .map(|db_album| db_album.try_into_album(conn))
        .collect::<Result<Vec<Album>, Error>>()
}

/// Updates an existing album's fields in the database based on the provided album's `id`
///
/// # Arguments
/// * `conn` - Database connection pool 
/// * `album` - Album with updated fields and ID of record to update
///
/// # Returns
/// Updated album if successful, or error if album is not found, if or update fails
pub fn update_album(conn: &mut PgConnection, album: Album) -> Result<Album, Error> {
    diesel::update(albums.find(album.id))
        .set::<DBAlbum>((&album).into())
        .returning(DBAlbum::as_returning())
        .get_result(conn)?;

    Ok(album)
}

/// Deletes the specified album from the database
///
/// # Arguments
/// * `conn` - Database connection pool
/// * `album_id` - ID of the album to delete
///
/// # Returns
/// Number of rows affected (1 if successful, 0 if album not found)
pub fn delete_album(conn: &mut PgConnection, album_id: i32) -> Result<usize, Error> {
    diesel::delete(albums.find(album_id))
        .execute(conn)
}
