use crate::db::models::{Album, NewAlbum};
use crate::db::operations::album::{create_album, delete_album, update_album};
use crate::DB_POOL;
use anyhow::Result;
use rocket::http::Status;
use rocket::serde::json::{Json, Value};
use rocket::{delete, patch, post};

#[post("/meow")]
fn health_check() -> (Status, &'static str) {
    (Status::ImATeapot, ">///<")
}

#[post("/album/new", format = "json", data = "<input>")]
fn new_album(input: Json<Value>) -> Status {
    crate::err_to_500!({
        let mut conn = DB_POOL.get()?;

        match input.get("album_name").and_then(|v| v.as_str()) {
            Some(album_name) => {
                create_album(&mut conn,NewAlbum {album_name: album_name.to_string()})?;
                Ok(Status::Created)
            }
            None => return Ok(Status::NotFound),
        }
    })
}

#[patch("/album/<id>/rename", format = "json", data = "<input>")]
fn rename_album(id: i32, input: Json<Value>) -> Status {
    crate::err_to_500!({
        let mut conn = DB_POOL.get()?;

        match input.get("album_name").and_then(|v| v.as_str()) {
            Some(album_name) => {
                update_album(&mut conn,id,Album {id,album_name: album_name.to_string()})?;
                Ok(Status::Ok)
            }
            None => return Ok(Status::NotFound),
        }
    })
}

#[delete("/album/<id>/delete")]
fn del_album(id: i32) -> Status {
    crate::err_to_500!({
        let mut conn = DB_POOL.get()?;

        delete_album(&mut conn, id)?;
        Ok(Status::Ok)
    })
}
