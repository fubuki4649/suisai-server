use rocket::http::Status;
use rocket::{delete, get, post};
use rocket::serde::json::Json;
use crate::db::models::{NewPhoto, Photo};
use crate::db::operations::photo::{create_photo, delete_photo, get_photo};
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
fn get_photo_single(id: i64) -> Result<Json<Photo>, Status> {
    crate::err_to_result_500!({
        let mut conn = DB_POOL.get()?;
        let photo = get_photo(&mut conn, id)?;

        Ok(Json(photo))
    })
}

#[get("/photo/get", format = "json", data = "<ids>")]
fn get_photo_multi(ids: Json<Vec<i64>>) -> Result<Json<Vec<Photo>>, Status> {
    crate::err_to_result_500!({
        let id_vec = ids.into_inner();
        
        let mut conn = DB_POOL.get()?;
        let mut photos: Vec<Photo> = Vec::with_capacity(id_vec.len());

        for id in id_vec.iter() {
            photos.push(get_photo(&mut conn, *id)?);
        }

        Ok(Json(photos))
    })
}
