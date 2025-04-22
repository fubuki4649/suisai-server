use crate::db::models::album::*;
use crate::db::models::db_album::DBAlbum;
use crate::db::schema::albums::dsl::albums;
use diesel::insert_into;
use diesel::prelude::*;
use diesel::result::Error;

pub fn create_album(conn: &mut PgConnection, album: NewAlbum) -> Result<Album, Error> {
    insert_into(albums)
        .values(&album)
        .returning(DBAlbum::as_returning())
        .get_result(conn)?
        .try_into_album(conn)
}

pub fn get_album(conn: &mut PgConnection, album_id: i32) -> Result<Album, Error> {
    albums
        .find(album_id)
        .select(DBAlbum::as_select())
        .first::<DBAlbum>(conn)?
        .try_into_album(conn)
}

pub fn get_all_albums(conn: &mut PgConnection) -> Result<Vec<Album>, Error> {
    let db_albums = albums
        .select(DBAlbum::as_select())
        .load(conn)?;

    db_albums
        .into_iter()
        .map(|db_album| db_album.try_into_album(conn))
        .collect::<Result<Vec<Album>, Error>>()
}

pub fn update_album(conn: &mut PgConnection, album: Album) -> Result<Album, Error> {
    diesel::update(albums.find(album.id))
        .set::<DBAlbum>((&album).into())
        .returning(DBAlbum::as_returning())
        .get_result(conn)?;

    Ok(album)
}

pub fn delete_album(conn: &mut PgConnection, album_id: i32) -> Result<usize, Error> {
    diesel::delete(albums.find(album_id))
        .execute(conn)
}
