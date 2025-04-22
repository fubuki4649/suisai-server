//! The `DBAlbum` struct corresponds to the `albums` table in the database.
//!
//! The `AlbumPhoto` struct corresponds to the `album_photos` table, a join table between
//! `Album` and `Photo` in the database.
//!
//! The following structs and impls exist exclusively for internal use within `crate::db::operations`
use crate::db::models::album::Album;
use crate::db::operations::album_photo_join::get_photos_in_album;
use crate::db::schema::{album_photos, albums};
use diesel::prelude::*;
use diesel::result::Error;

#[derive(Queryable, Selectable, AsChangeset, Debug)]
#[diesel(table_name = albums)]
pub struct DBAlbum {
    pub id: i32,
    pub album_name: String,
}

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug)]
#[diesel(table_name = album_photos)]
pub struct AlbumPhoto {
    pub album_id: i32,
    pub photo_id: i64,
}


impl Into<DBAlbum> for &Album {
    fn into(self) -> DBAlbum {
        DBAlbum {
            id: self.id,
            album_name: self.album_name.clone(),
        }
    }
}

impl DBAlbum {
    pub fn try_into_album(self, conn: &mut PgConnection) -> Result<Album, Error> {

        let album = Album {
            id: self.id,
            album_name: self.album_name,
            photos: get_photos_in_album(conn, self.id)?,
        };

        Ok(album)
    }
}