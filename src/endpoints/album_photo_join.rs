use crate::_utils::json_map::JsonMap;
use crate::db::operations::album_photo_join::{add_photo_to_album, remove_photo_from_all_albums};
use crate::{unwrap_or_return, DB_POOL};
use rocket::http::Status;
use rocket::post;
use rocket::serde::json::{Json, Value};

#[post("/photo/<id>/assign-album", format = "json", data = "<input>")]
pub fn photo_assign_album(id: i64, input: Json<Value>) -> Status {
    let album_id = unwrap_or_return!(input.get_value::<i32>("album_id"), Status::BadRequest);
    crate::err_to_500!({
        let mut conn = DB_POOL.get()?;
        add_photo_to_album(&mut conn, id, album_id)?;
        Ok(Status::Ok)
    })
}

#[post("/photo/<id>/clear-album")]
pub fn photo_clear_album(id: i64) -> Status {
    crate::err_to_500!({
        let mut conn = DB_POOL.get()?;
        remove_photo_from_all_albums(&mut conn, id)?;
        Ok(Status::Ok)
    })
}

#[post("/photo/<id>/move-album", format = "json", data = "<input>")]
pub fn photo_move_album(id: i64, input: Json<Value>) -> Status {
    let album_id = unwrap_or_return!(input.get_value::<i32>("album_id"), Status::BadRequest);
    crate::err_to_500!({
        let mut conn = DB_POOL.get()?;
        
        // Remove from previous albums
        remove_photo_from_all_albums(&mut conn, id)?;
        
        // Add to new album
        add_photo_to_album(&mut conn, id, album_id)?;
        
        Ok(Status::Ok)
    })
}