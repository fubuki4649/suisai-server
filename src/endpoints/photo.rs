use crate::db::models::photo::*;
use crate::db::operations::photo::{create_photo, delete_photo, get_photo};
use crate::DB_POOL;
use diesel::result::Error;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{delete, get, post};


#[post("/photo/new", format = "json", data = "<input>")]
pub fn new_photo(input: Json<NewPhoto>) -> Status {
    crate::err_to_500!({
        let mut conn = DB_POOL.get()?;
        create_photo(&mut conn, input.into_inner())?;
        
        Ok(Status::Created)
    })
}

#[delete("/photo/<id>/delete")]
pub fn del_photo(id: i64) -> Status {
    crate::err_to_500!({
        let mut conn = DB_POOL.get()?;

        match delete_photo(&mut conn, id) {
            Ok(_) => Ok(Status::Ok),
            Err(Error::NotFound) => Ok(Status::NotFound),
            Err(e) => Err(e.into()),
        }
    })
}

#[get("/photo/<id>")]
pub fn get_photo_single(id: i64) -> Result<Json<Photo>, Status> {
    crate::err_to_result_500!({
        let mut conn = DB_POOL.get()?;
        match get_photo(&mut conn, id) {
            Ok(photo) => Ok(Ok(Json(photo))),
            Err(Error::NotFound) => Ok(Err(Status::NotFound)),
            Err(e) => Err(e.into()),
        }
    })
}

#[get("/photo/get", format = "json", data = "<ids>")]
pub fn get_photo_multi(ids: Json<Vec<i64>>) -> Result<Json<Vec<Photo>>, Status> {
    crate::err_to_result_500!({
        let id_vec = ids.into_inner();
        
        let mut conn = DB_POOL.get()?;
        let mut photos: Vec<Photo> = Vec::with_capacity(id_vec.len());

        for id in id_vec.iter() {
            if let Ok(photo) = get_photo(&mut conn, *id) {
                photos.push(photo);
            }
        }

        Ok(Ok(Json(photos)))
    })
}
