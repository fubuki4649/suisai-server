use clap::{error::ErrorKind, Error};
use diesel::prelude::*;
use diesel::MysqlConnection;
use std::collections::HashSet;
use std::path::PathBuf;

use crate::db::schema::{album_album_join, album_photo_join, albums, photos};

/// Gets an album's path, relative to $STORAGE_ROOT
///
/// # Arguments
/// * `conn` - Database connection
/// * `album_id` - ID of the album to get path for
///
/// # Returns
/// The path of the album
pub fn get_album_path(conn: &mut MysqlConnection, album_id: i32) -> Result<PathBuf, Error> {
    // Helper to map Diesel errors into clap::Error consistently
    let map_err = |e: diesel::result::Error| Error::raw(ErrorKind::InvalidValue, e.to_string());

    // Collect the chain of album names from the current album up to root
    let mut segments: Vec<String> = Vec::new();
    let mut current_id: Option<i32> = Some(album_id);
    let mut seen: HashSet<i32> = HashSet::new();

    while let Some(aid) = current_id {
        if !seen.insert(aid) {
            return Err(Error::raw(
                ErrorKind::InvalidValue,
                format!("Cycle detected in album hierarchy at album_id={}", aid),
            ));
        }

        // Fetch the album_name for this album id
        let name: String = albums::table
            .find(aid)
            .select(albums::album_name)
            .first::<String>(conn)
            .map_err(map_err)?;
        segments.push(name);

        // Find parent album, if any. If multiple parents exist, choose the one with the lowest parent_id for determinism.
        let parent: Option<i32> = album_album_join::table
            .filter(album_album_join::album_id.eq(aid))
            .select(album_album_join::parent_id)
            .order(album_album_join::parent_id.asc())
            .first::<i32>(conn)
            .optional()
            .map_err(map_err)?;

        current_id = parent;
    }

    // Build the path from root to leaf: segments were collected leaf->root, so reverse
    segments.reverse();
    let mut path = PathBuf::new();
    for seg in segments {
        path.push(seg);
    }
    Ok(path)
}


/// Gets a photo's path, relative to $STORAGE_ROOT
///
/// # Arguments
/// * `conn` - Database connection
/// * `photo_id` - ID of the photo to get path for
///
/// # Returns
/// The path of the photo
pub fn get_photo_path(conn: &mut MysqlConnection, photo_id: i64) -> Result<PathBuf, Error> {
    let map_err = |e: diesel::result::Error| Error::raw(ErrorKind::InvalidValue, e.to_string());

    // Get the photo file name (and confirm the photo exists)
    let file_name: String = photos::table
        .find(photo_id as i64)
        .select(photos::file_name)
        .first::<String>(conn)
        .map_err(map_err)?;

    // Resolve a parent album if any
    let parent_album: Option<i32> = album_photo_join::table
        .filter(album_photo_join::photo_id.eq(photo_id as i64))
        .select(album_photo_join::parent_id)
        .order(album_photo_join::parent_id.asc())
        .first::<i32>(conn)
        .optional()
        .map_err(map_err)?;

    let mut path = match parent_album {
        Some(album_id) => get_album_path(conn, album_id)?,
        None => PathBuf::new(), // Unfiled photo: place at root of storage
    };
    
    path.push(file_name);
    Ok(path)
}