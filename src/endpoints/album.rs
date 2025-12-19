use crate::_utils::json_map::JsonMap;
use crate::db::operations::album::{create_album, delete_album, get_album, get_root_albums, rename_album as rename_album_db};
use crate::db::operations::paths::get_album_path;
use crate::db::operations::query::{get_photos_in_album, get_photos_unfiled, get_subalbums};
use crate::fs_operations::album::{create_album_fs, delete_album_fs, move_album_fs};
use crate::models::album::{Album, AlbumTree, NewAlbum};
use crate::models::photo::Photo;
use crate::{msg, unwrap_err, unwrap_ret, DB_POOL};
use diesel::result::Error;
use rocket::http::Status;
use rocket::serde::json::{Json, Value};
use rocket::{delete, get, patch, post};


/// Retrieves the album tree structure from the database
///
/// # Endpoint
/// `GET /album/tree`
///
/// # Returns
/// - `200 OK`: A tree structure representing all the albums
/// - `500 Internal Server Error`: Database or another server error occurred
///
/// # Response Body
/// An album tree, with each node containing:
/// - `albumId`: Album's unique identifier (i32)
///     - (-1) for the root node (the root node is not an album)
/// - `albumName`: Name of the album (String)
///     - A placeholder value for the root node. This value can be anything and should not be used
/// - `children`: An album's children (Vec<AlbumTree>)
///     - For the root album, this is a list of the root-level albums
#[get("/album/tree")]
pub fn get_album_tree() -> Result<Json<AlbumTree>, (Status, Json<Value>)> {
    let mut conn = unwrap_err!(DB_POOL.get(), Status::InternalServerError);
    let mut tree = AlbumTree {
        id: -1,
        album_name: "Root Node - Not an Album!!!".to_string(),
        children: unwrap_err!(get_root_albums(&mut conn), Status::InternalServerError).into_iter().map(Into::into).collect(),
    };

    fn fill_tree(node: &mut AlbumTree) -> anyhow::Result<()> {
        let mut conn = DB_POOL.get()?;

        if node.children.is_empty() {
            node.children = get_subalbums(&mut conn, node.id)?.into_iter().map(Into::into).collect();
        }

        for child in &mut node.children {
            fill_tree(child)?;
        }

        Ok(())
    }

    unwrap_err!(fill_tree(&mut tree), Status::InternalServerError);
    Ok(Json(tree))
}

/// Creates a new "root" album at `$STORAGE_ROOT`
///
/// # Endpoint
/// `POST /album/new`
///
/// # Request Body
/// JSON object with:
/// - `album_name`: Name for the new album (String)
///
/// # Returns
/// - `201 Created`: Album was successfully created
/// - `400 Bad Request`: Missing or invalid album_name in request body
/// - `500 Internal Server Error`: Database or other server error occurred
#[post("/album/new", format = "json", data = "<input>")]
pub fn new_album(input: Json<Value>) -> (Status, Json<Value>) {
    let album_name = unwrap_ret!(input.get_value::<String>("album_name"), Status::BadRequest);
    let mut conn = unwrap_ret!(DB_POOL.get(), Status::InternalServerError);

    // Create the album directory in the filesystem
    unwrap_ret!(create_album_fs(&album_name), Status::InternalServerError);

    // Create albums and return the appropriate response based off the number of rows created
    let rows = unwrap_ret!(create_album(&mut conn, NewAlbum {album_name: album_name.to_string()}), Status::InternalServerError);
    match rows {
        1 => (Status::Created, msg!("Success")),
        0 => (Status::Conflict, msg!("Album already exists")),
        _ => (Status::InternalServerError, msg!("Something went horribly wrong. The database is probably corrupted (Database transacted > 1 rows)")),
    }
}

/// Renames an existing album
///
/// # Endpoint
/// `PATCH /album/<id>/rename`
///
/// # URL Parameters
/// - `id`: The ID of the album to rename (i32)
///
/// # Request Body
/// JSON object with:
/// - `album_name`: New name for the album (String)
///
/// # Returns
/// - `200 OK`: Album was successfully renamed
/// - `400 Bad Request`: Missing or invalid album_name in the request body
/// - `404 Not Found`: Album with the specified ID does not exist
/// - `500 Internal Server Error`: Database or other server error occurred
#[patch("/album/<id>/rename", format = "json", data = "<input>")]
pub fn rename_album(id: i32, input: Json<Value>) -> (Status, Json<Value>) {
    let album_name = unwrap_ret!(input.get_value::<String>("album_name"), Status::BadRequest);
    let mut conn = unwrap_ret!(DB_POOL.get(), Status::InternalServerError);

    // Rename the album on disk
    let old_path = unwrap_ret!(get_album_path(&mut conn, id), Status::InternalServerError);
    let new_path = unwrap_ret!(old_path.parent().ok_or("Cannot rename the root directory itself!"), Status::InternalServerError).join(&album_name);
    unwrap_ret!(move_album_fs(&old_path, &new_path), Status::InternalServerError);

    // Rename the album in the DB
    match rename_album_db(&mut conn, Album {id, album_name}) {
        Ok(_) => (Status::Ok, msg!("Success")),
        Err(Error::NotFound) => (Status::NotFound, msg!("Album not found")),
        Err(err) => (Status::InternalServerError, msg!("Failed to rename album in database: {:#?}", err)),
    }
}

/// Deletes an album from the database by ID
///
/// # Endpoint
/// `DELETE /album/<id>/delete`
///
/// # URL Parameters
/// - `id`: The ID of the album to delete (i32)
///
/// # Returns
/// - `200 OK`: Album was successfully deleted
/// - `404 Not Found`: Album with the specified ID does not exist
/// - `500 Internal Server Error`: Database or other server error occurred
#[delete("/album/<id>/delete")]
pub fn del_album(id: i32) -> (Status, Json<Value>) {
    let mut conn = unwrap_ret!(DB_POOL.get(), Status::InternalServerError);

    // Delete album from disk, moving its children to root
    let album = unwrap_ret!(get_album(&mut conn, &[id]).and_then(|mut albums| albums.pop().ok_or(Error::NotFound)), Status::InternalServerError);
    let album_path = unwrap_ret!(get_album_path(&mut conn, album.id), Status::InternalServerError);
    let child_photos = unwrap_ret!(get_photos_in_album(&mut conn, album.id), Status::InternalServerError);
    let child_albums = unwrap_ret!(get_subalbums(&mut conn, album.id), Status::InternalServerError);

    unwrap_ret!(delete_album_fs(&album_path, &child_photos, &child_albums), Status::InternalServerError);

    // Delete album from DB
    match delete_album(&mut conn, id) {
        Err(Error::NotFound) => (Status::NotFound, msg!("Album not found")),
        Err(err) => (Status::InternalServerError, msg!("Failed to delete album from database: {:#?}", err)),
        Ok(_) => {
            (Status::Ok, msg!("Success"))
        },
    }
}


/// Retrieves all photos linked to a given album
///
/// # Endpoint
/// `GET /album/<id>/photos`
///
/// # Returns
/// - `200 OK`: JSON array of unfiled photos
/// - `500 Internal Server Error`: Database or another server error occurred
///
/// # Response Body
/// Array of webapi::Photo objects containing metadata for each photo in the album
#[get("/album/<id>/photos")]
pub fn album_photos(id: i32) -> Result<Json<Vec<Photo>>, (Status, Json<Value>)> {
    let mut conn = unwrap_err!(DB_POOL.get(), Status::InternalServerError);

    let album_photos =  unwrap_err!(get_photos_in_album(&mut conn, id), Status::InternalServerError);
    Ok(Json(album_photos))
}


/// Retrieves all photos that are not assigned to any album
///
/// # Endpoint
/// `GET /album/unfiled/photos`
///
/// # Returns
/// - `200 OK`: JSON array of unfiled photos
/// - `500 Internal Server Error`: Database or another server error occurred
///
/// # Response Body
/// Array of webapi::Photo objects containing metadata for each unfiled photo
#[get("/album/unfiled/photos")]
pub fn unfiled_photos() -> Result<Json<Vec<Photo>>, (Status, Json<Value>)> {
    let mut conn = unwrap_err!(DB_POOL.get(), Status::InternalServerError);

    let unfiled_photos =  unwrap_err!(get_photos_unfiled(&mut conn), Status::InternalServerError);
    Ok(Json(unfiled_photos))
}