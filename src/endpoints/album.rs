use crate::_utils::json_map::JsonMap;
use crate::db::models::album::*;
use crate::db::operations::album::{create_album, delete_album, get_all_albums, update_album};
use crate::db::operations::album_photo_join::get_photos_in_album;
use crate::{unwrap_or_return, DB_POOL};
use anyhow::Result;
use diesel::result::Error;
use rocket::http::Status;
use rocket::serde::json::{Json, Value};
use rocket::{delete, get, patch, post};
use crate::db::models::photo::Photo;
use crate::db::operations::photo::get_unfiled_photos;

#[get("/meow")]
pub fn health_check() -> (Status, &'static str) {
    (Status::ImATeapot, ">///<\n")
}

#[post("/album/new", format = "json", data = "<input>")]
pub fn new_album(input: Json<Value>) -> Status {
    let album_name = unwrap_or_return!(input.get_value::<String>("album_name"), Status::BadRequest);
    
    crate::err_to_500!({
        let mut conn = DB_POOL.get()?;
        
        create_album(&mut conn, NewAlbum {album_name: album_name.to_string()})?;
        Ok(Status::Created)
    })
}

#[patch("/album/<id>/rename", format = "json", data = "<input>")]
pub fn rename_album(id: i32, input: Json<Value>) -> Status {
    let album_name = unwrap_or_return!(input.get_value::<String>("album_name"), Status::BadRequest);

    crate::err_to_500!({
        let mut conn = DB_POOL.get()?;

        match update_album(&mut conn, id, DBAlbum {id, album_name}) {
            Ok(_) => Ok(Status::Ok),
            Err(Error::NotFound) => Ok(Status::NotFound),
            Err(e) => Err(e.into()),
        }
    })
}

#[delete("/album/<id>/delete")]
pub fn del_album(id: i32) -> Status {
    crate::err_to_500!({
        let mut conn = DB_POOL.get()?;

        match delete_album(&mut conn, id) {
            Ok(_) => Ok(Status::Ok),
            Err(Error::NotFound) => Ok(Status::NotFound),
            Err(e) => Err(e.into()),
        }
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
        
        Ok(Ok(Json(albums)))
    })
}

#[get("/album/unfiled")]
pub fn get_unfiled() -> Result<Json<Vec<Photo>>, Status> {
    crate::err_to_result_500!({
        let mut conn = DB_POOL.get()?;
        
        let unfiled_photos = get_unfiled_photos(&mut conn)?;
        Ok(Ok(Json(unfiled_photos)))
    })
}
