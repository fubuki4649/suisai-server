use crate::db::schema::albums;
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

#[derive(Insertable, Deserialize, Debug)]
#[diesel(table_name = albums)]
pub struct NewAlbum {
    pub album_name: String,
}
