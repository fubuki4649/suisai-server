use crate::fs_operations::photo::move_photo_fs;
use crate::models::album::Album;
use crate::models::photo::Photo;
use std::fs;
use std::io::Error;
use std::path::{Path, PathBuf};

/// Deletes an album, moving its children to the root (albums) / unfiled (photos).
///
/// # Arguments
/// * `album_path` - Path to the album directory, relative to $STORAGE_ROOT
/// * `child_photos` - Photos linked to the album
/// * `child_albums` - Subalbums linked to the album
///
/// # Returns
/// Ok if the album was deleted successfully and its children moved, or an error if deletion failed.
pub fn delete_album_fs(album_path: &Path, child_photos: &[Photo], child_albums: &[Album]) -> Result<(), Error> {
    let storage_root = PathBuf::from(std::env::var("STORAGE_ROOT").unwrap());
    let full_album_path = storage_root.join(album_path);

    // Move child photos to the unfiled directory
    for photo in child_photos {
        move_photo_fs(&album_path.join(&photo.file_name), &PathBuf::from("/unfiled"))?;
    }

    // Move child albums to root
    for album in child_albums {
        let child_album_name = &album.album_name;
        let current_child_path = full_album_path.join(child_album_name);
        let new_child_path = storage_root.join(child_album_name);
        
        if current_child_path.exists() {
            fs::rename(current_child_path, new_child_path)?;
        }
    }

    // Delete the now-empty album directory
    if full_album_path.exists() {
        fs::remove_dir_all(full_album_path)?;
    }

    Ok(())
}

/// Moves the entire album (and its children) to a new album.
///
/// # Arguments
/// * `album_path` - Path to the album to be moved, relative to $STORAGE_ROOT
/// * `destination_path` - Path to the new parent album, relative to $STORAGE_ROOT. This must not be
///   a child of `album_path`.
///
/// # Returns
/// Ok if successful, or Error if validation fails or filesystem operation fails.
pub fn move_album_fs(album_path: &Path, destination_path: &Path) -> Result<(), Error> {
    let storage_root = PathBuf::from(std::env::var("STORAGE_ROOT").unwrap());
    let src_path = storage_root.join(album_path);
    let dest_path = storage_root.join(destination_path);

    // Make sure the source and destination albums exist
    if !src_path.exists() {
        return Err(Error::new(std::io::ErrorKind::NotFound, "Source album not found"));
    }

    if !dest_path.exists() {
        return Err(Error::new(std::io::ErrorKind::NotFound, "Target parent album not found"));
    }

    // Make sure the target album is not a child of the source album
    if dest_path.starts_with(&src_path) {
        return Err(Error::new(std::io::ErrorKind::InvalidInput, "Target album is a child of the album to be moved"));
    }

    let album_name = src_path.file_name().ok_or_else(|| Error::new(std::io::ErrorKind::InvalidInput, "Invalid source path"))?;
    let new_path = dest_path.join(album_name);

    fs::rename(src_path, new_path)?;

    Ok(())
}

