use crate::db::models::album::*;
use crate::db::operations::album::{create_album, delete_album, get_all_albums, update_album};
use crate::DB_POOL;
use anyhow::Result;
use rocket::http::Status;
use rocket::serde::json::{Json, Value};
use rocket::{delete, get, patch, post};
use crate::db::operations::album_photo_join::get_photos_in_album;

#[get("/meow")]
pub fn health_check() -> (Status, &'static str) {
    (Status::ImATeapot, ">///<\n")
}

#[post("/album/new", format = "json", data = "<input>")]
pub fn new_album(input: Json<Value>) -> Status {
    crate::err_to_500!({
        let mut conn = DB_POOL.get()?;

        match input.get("album_name").and_then(|v| v.as_str()) {
            Some(album_name) => {
                create_album(&mut conn, NewAlbum {album_name: album_name.to_string()})?;
                Ok(Status::Created)
            }
            None => return Ok(Status::NotFound),
        }
    })
}

#[patch("/album/<id>/rename", format = "json", data = "<input>")]
pub fn rename_album(id: i32, input: Json<Value>) -> Status {
    crate::err_to_500!({
        let mut conn = DB_POOL.get()?;

        match input.get("album_name").and_then(|v| v.as_str()) {
            Some(album_name) => {
                update_album(&mut conn, id, DBAlbum {id, album_name: album_name.to_string()})?;
                Ok(Status::Ok)
            }
            None => return Ok(Status::NotFound),
        }
    })
}

#[delete("/album/<id>/delete")]
pub fn del_album(id: i32) -> Status {
    crate::err_to_500!({
        let mut conn = DB_POOL.get()?;

        delete_album(&mut conn, id)?;
        Ok(Status::Ok)
    })
}

#[get("/album/all")]
pub fn all_albums() -> Result<Json<Vec<Album>>, Status> {
    crate::err_to_result_500!({
        let mut conn = DB_POOL.get()?;
        
        let db_albums = get_all_albums(&mut conn)?;
        let mut albums: Vec<Album> = Vec::with_capacity(db_albums.len());
        
        for db_album in db_albums.into_iter() {
            let mut album = Album::from_db_album(&db_album);
            album.photos = get_photos_in_album(&mut conn, album.id)?;
            
            albums.push(album);
        }
        
        Ok(Json(albums))
    })
}
