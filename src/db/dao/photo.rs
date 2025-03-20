use diesel::prelude::*;
use crate::db::models::{Photo, NewPhoto};
use crate::db::schema::photos;
use std::error::Error;

pub struct PhotoDao {
    conn: PgConnection,
}

impl PhotoDao {
    pub fn new(conn: PgConnection) -> Self {
        PhotoDao { conn }
    }

    pub fn create(&mut self, new_photo: NewPhoto) -> Result<Photo, Box<dyn Error>> {
        let result = diesel::insert_into(photos::table)
            .values(&new_photo)
            .get_result(&mut self.conn)?;

        Ok(result)
    }

    pub fn find_by_id(&mut self, photo_id: i64) -> Result<Option<Photo>, Box<dyn Error>> {
        let result = photos::table
            .find(photo_id)
            .first(&mut self.conn)
            .optional()?;

        Ok(result)
    }

    pub fn update(&mut self, photo_id: i64, photo: NewPhoto) -> Result<Photo, Box<dyn Error>> {
        let result = diesel::update(photos::table.find(photo_id))
            .set(&photo)
            .get_result(&mut self.conn)?;

        Ok(result)
    }

    pub fn delete(&mut self, photo_id: i64) -> Result<usize, Box<dyn Error>> {
        let count = diesel::delete(photos::table.find(photo_id))
            .execute(&mut self.conn)?;

        Ok(count)
    }
}