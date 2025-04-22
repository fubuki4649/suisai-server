use crate::db::models::photo::*;
use crate::db::schema::album_photos;
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

pub fn check_hash(conn: &mut PgConnection, hash: &str) -> Result<Option<Photo>, Error> {
    photos
        .filter(crate::db::schema::photos::dsl::hash.eq(hash))
        .select(Photo::as_select())
        .first(conn)
        .map(Some)
        .or_else(|e| match e {
            Error::NotFound => Ok(None),
            err => Err(err),
        })
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

pub fn update_photo(conn: &mut PgConnection, photo: Photo) -> Result<Photo, Error> {
    diesel::update(photos.find(photo.id))
        .set(&photo)
        .returning(Photo::as_returning())
        .get_result(conn)
}

pub fn delete_photo(conn: &mut PgConnection, photo_id: i64) -> Result<usize, Error> {
    diesel::delete(photos.find(photo_id))
        .execute(conn)
}

pub fn get_unfiled_photos(conn: &mut PgConnection) -> Result<Vec<Photo>, Error> {
    crate::db::schema::photos::table
        .left_outer_join(album_photos::table.on(album_photos::photo_id.eq(crate::db::schema::photos::id)))
        .filter(album_photos::photo_id.is_null()) // Only those with no match
        .select(crate::db::schema::photos::all_columns) // Select all fields from `photos`
        .load::<Photo>(conn)
}