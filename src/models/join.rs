use crate::db::schema::{album_album_join, album_photo_join};
use diesel::{AsChangeset, Insertable, Queryable, Selectable};

/// The `AlbumPhoto` struct corresponds to the `album_photos` table, a join table between
/// `Album` and `Photo` in the database.
///
/// It exists exclusively for internal use within `crate::db::operations`
#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug)]
#[diesel(table_name = album_photo_join)]
pub struct AlbumPhoto {
    pub parent_id: i32,
    pub photo_id: i64,
}

/// The `AlbumAlbum` struct corresponds to the `album_album` table, a join table between
/// two `Album`s in the database.
///
/// It exists exclusively for internal use within `crate::db::operations`
#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug)]
#[diesel(table_name = album_album_join)]
pub struct AlbumAlbum {
    pub parent_id: i32,
    pub album_id: i32,
}