use crate::db::schema::{album_photos, albums};
use diesel::prelude::*;
use rocket::serde::{Deserialize, Serialize};


#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Album {
    #[serde(rename = "albumId")]
    pub id: i32,
    pub album_name: String,
    pub photos: Vec<i64>
}

impl Album {
    pub fn from_db_album(db_album: &DBAlbum) -> Self {
        Album {
            id: db_album.id,
            album_name: db_album.album_name.clone(),
            photos: vec![],
        }
    }
}

#[derive(Queryable, Selectable, AsChangeset, Debug)]
#[diesel(table_name = albums)]
pub struct DBAlbum {
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