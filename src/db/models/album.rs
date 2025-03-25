use crate::db::schema::{album_photos, albums};
use diesel::prelude::*;
use rocket::serde::{Deserialize, Serialize};


#[derive(Queryable, Selectable, AsChangeset, Serialize, Debug)]
#[diesel(table_name = albums)]
pub struct Album {
    pub id: i32,
    pub album_name: String,
}

#[derive(Insertable, Deserialize, Debug)]
#[diesel(table_name = albums)]
pub struct NewAlbum {
    pub album_name: String,
}

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug)]
#[diesel(table_name = album_photos)]
pub struct AlbumPhoto {
    pub album_id: i32,
    pub photo_id: i64,
}