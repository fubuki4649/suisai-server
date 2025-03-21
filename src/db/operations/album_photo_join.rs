use diesel::insert_into;
use diesel::prelude::*;
use diesel::result::Error;
use crate::db::models::{Album, AlbumPhoto, Photo};
use crate::db::schema::{album_photos, photos, albums};

pub fn add_photo_to_album(conn: &mut PgConnection, album_id: i32, photo_id: i64) -> Result<AlbumPhoto, Error> {
    let album_photo = AlbumPhoto {
        album_id,
        photo_id,
    };

    insert_into(album_photos::table)
        .values(&album_photo)
        .returning(AlbumPhoto::as_returning())
        .get_result(conn)
}

pub fn remove_photo_from_album(conn: &mut PgConnection, album_id: i32, photo_id: i64) -> Result<usize, Error> {
    diesel::delete(
        album_photos::table
            .filter(album_photos::album_id.eq(album_id))
            .filter(album_photos::photo_id.eq(photo_id))
    )
        .execute(conn)
}

pub fn get_photos_in_album(conn: &mut PgConnection, album_id: i32) -> Result<Vec<Photo>, Error> {
    album_photos::table
        .filter(album_photos::album_id.eq(album_id))
        .inner_join(photos::table)
        .select(Photo::as_select())
        .load(conn)
}

pub fn get_albums_containing_photo(conn: &mut PgConnection, photo_id: i64) -> Result<Vec<Album>, Error> {
    album_photos::table
        .filter(album_photos::photo_id.eq(photo_id))
        .inner_join(albums::table)
        .select(Album::as_select())
        .load(conn)
}

pub fn clear_album(conn: &mut PgConnection, album_id: i32) -> Result<usize, Error> {
    diesel::delete(album_photos::table.filter(album_photos::album_id.eq(album_id)))
        .execute(conn)
}