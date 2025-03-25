use crate::db::models::photo::*;
use crate::db::schema::photos::dsl::photos;
use diesel::insert_into;
use diesel::prelude::*;
use diesel::result::Error;

pub fn create_photo(conn: &mut PgConnection, new_photo: NewPhoto) -> Result<Photo, Error> {
    insert_into(photos)
        .values(&new_photo)
        .returning(Photo::as_returning())
        .get_result(conn)
}

pub fn get_photo(conn: &mut PgConnection, photo_id: i64) -> Result<Photo, Error> {
    photos
        .find(photo_id)
        .select(Photo::as_select())
        .first(conn)
}

pub fn get_all_photos(conn: &mut PgConnection) -> Result<Vec<Photo>, Error> {
    photos
        .select(Photo::as_select())
        .load(conn)
}

pub fn update_photo(conn: &mut PgConnection, photo_id: i64, photo: Photo) -> Result<Photo, Error> {
    diesel::update(photos.find(photo_id))
        .set(&photo)
        .returning(Photo::as_returning())
        .get_result(conn)
}

pub fn delete_photo(conn: &mut PgConnection, photo_id: i64) -> Result<usize, Error> {
    diesel::delete(photos.find(photo_id))
        .execute(conn)
}