use diesel::prelude::*;
use crate::db::schema::photos;
use chrono::NaiveDateTime;
use rocket::serde::{Deserialize, Serialize};


#[derive(Queryable, Selectable, AsChangeset, Serialize, Debug)]
#[diesel(table_name = photos)]
#[serde(rename_all = "camelCase")]
pub struct Photo {
    #[serde(rename = "photoId")]
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

#[derive(Insertable, Deserialize, Serialize, Debug)]
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