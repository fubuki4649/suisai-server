use crate::db::models::album::*;
use crate::db::schema::albums::dsl::albums;
use diesel::insert_into;
use diesel::prelude::*;
use diesel::result::Error;

pub fn create_album(conn: &mut PgConnection, album: NewAlbum) -> Result<DBAlbum, Error> {
    insert_into(albums)
        .values(&album)
        .returning(DBAlbum::as_returning())
        .get_result(conn)
}

pub fn get_album(conn: &mut PgConnection, album_id: i32) -> Result<DBAlbum, Error> {
    albums
        .find(album_id)
        .select(DBAlbum::as_select())
        .first(conn)
}

pub fn get_all_albums(conn: &mut PgConnection) -> Result<Vec<DBAlbum>, Error> {
    albums
        .select(DBAlbum::as_select())
        .load(conn)
}

pub fn update_album(conn: &mut PgConnection, album_id: i32, album: DBAlbum) -> Result<DBAlbum, Error> {
    diesel::update(albums.find(album_id))
        .set(&album)
        .returning(DBAlbum::as_returning())
        .get_result(conn)
}

pub fn delete_album(conn: &mut PgConnection, album_id: i32) -> Result<usize, Error> {
    diesel::delete(albums.find(album_id))
        .execute(conn)
}
