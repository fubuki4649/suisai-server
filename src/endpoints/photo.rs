use rocket::http::Status;
use rocket::{delete, get, post};
use rocket::serde::json::Json;
use crate::db::models::NewPhoto;
use crate::db::operations::photo::{create_photo, delete_photo};
use crate::DB_POOL;

#[post("/photo/new", format = "json", data = "<input>")]
fn new_photo(input: Json<NewPhoto>) -> Status {
    crate::err_to_500!({
        let mut conn = DB_POOL.get()?;
        create_photo(&mut conn, input.into_inner())?;
        
        Ok(Status::Created)
    })
}

#[delete("/photo/<id>/delete")]
fn del_photo(id: i64) -> Status {
    crate::err_to_500!({
        let mut conn = DB_POOL.get()?;
        delete_photo(&mut conn, id)?;
        
        Ok(Status::Ok)
    })
}

#[get("/photo/<id>")]
fn get_photo(id: i64) -> Status {
    crate::err_to_500!({
        
        Ok(Status::Ok)
    })
}