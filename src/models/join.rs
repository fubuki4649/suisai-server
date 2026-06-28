use crate::db::schema::{album_album_join, album_photo_join};
use diesel::{AsChangeset, Insertable, Queryable, Selectable};
use sea_orm::DerivePartialModel;

/// The `AlbumPhoto` struct corresponds to the `album_photos` table, a join table between
/// `Album` and `Photo` in the database.
///
/// It exists exclusively for internal use within `crate::db::operations`
#[derive(DerivePartialModel, Queryable, Selectable, Insertable, AsChangeset, Debug)]
#[sea_orm(entity = "crate::db::entities::album_photo_join::Entity")]
#[diesel(table_name = album_photo_join)]
pub struct AlbumPhoto {
    pub parent_id: String,
    pub photo_id: String,
}

/// The `AlbumAlbum` struct corresponds to the `album_album` table, a join table between
/// two `Album`s in the database.
///
/// It exists exclusively for internal use within `crate::db::operations`
#[derive(DerivePartialModel, Queryable, Selectable, Insertable, AsChangeset, Debug)]
#[sea_orm(entity = "crate::db::entities::album_album_join::Entity")]
#[diesel(table_name = album_album_join)]
pub struct AlbumAlbum {
    pub parent_id: String,
    pub album_id: String,
}