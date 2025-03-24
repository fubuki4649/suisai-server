// models.rs
use diesel::prelude::*;
use crate::db::schema::{photos, albums, album_photos};
use chrono::NaiveDateTime;
use rocket::serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, AsChangeset, Serialize, Debug)]
#[diesel(table_name = photos)]
pub struct Photo {
    pub id: i64,
    pub thumbnail_url: String,
    pub file_name: String,
    pub file_path: String,
    pub size_on_disk: String,
    pub photo_date: NaiveDateTime,
    pub photo_timezone: String,
    pub resolution: Vec<Option<i32>>,
    pub mime_type: String,
    pub camera_model: String,
    pub lens_model: String,
    pub shutter_count: i32,
    pub focal_length: i32,
    pub iso: i32,
    pub shutter_speed: String,
    pub aperture: f32,
}

#[derive(Insertable, Deserialize, Debug)]
#[diesel(table_name = photos)]
pub struct NewPhoto {
    pub thumbnail_url: String,
    pub file_name: String,
    pub file_path: String,
    pub size_on_disk: String,
    pub photo_date: NaiveDateTime,
    pub photo_timezone: String,
    pub resolution: Vec<Option<i32>>,
    pub mime_type: String,
    pub camera_model: String,
    pub lens_model: String,
    pub shutter_count: i32,
    pub focal_length: i32,
    pub iso: i32,
    pub shutter_speed: String,
    pub aperture: f32,
}

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