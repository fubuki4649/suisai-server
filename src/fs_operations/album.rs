use crate::fs_operations::photo::move_photo_fs;
use crate::models::album::Album;
use crate::models::photo::Photo;
use std::fs;
use std::io::Error;
use std::path::{Path, PathBuf};


/// Creates a directory for an album at the storage root
///
/// # Arguments
/// * `album_name` - The name of the new album
///
/// # Returns
/// Ok if the album was successfully created at `$STORAGE_ROOT/album_name`
pub fn create_album_fs(album_name: &str) -> Result<(), Error> {
    let storage_root = PathBuf::from(std::env::var("STORAGE_ROOT").unwrap());

    // Create the album directory
    fs::create_dir(storage_root.join(album_name))?;
    Ok(())
}

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
/// * `destination_path` - Path to the new album, relative to $STORAGE_ROOT. This must not be
///   a child of `album_path`.
///
/// # Returns
/// Ok if successful, or Error if validation fails or filesystem operation fails.
///
/// # Examples
///
/// Move an album to a different parent:
/// ```no_run
/// use std::path::Path;
///
/// // Move "2023/vacation" to "archived/2023/vacation"
/// move_album_fs(
///     Path::new("2023/vacation"),
///     Path::new("archived/2023/vacation")
/// )?;
/// # Ok::<(), std::io::Error>(())
/// ```
///
/// Move an album to the root level:
/// ```no_run
/// use std::path::Path;
///
/// // Move "2023/events/birthday" to "birthday"
/// move_album_fs(
///     Path::new("2023/events/birthday"),
///     Path::new("birthday")
/// )?;
/// # Ok::<(), std::io::Error>(())
/// ```
///
/// This will fail because the destination is a child of the source:
/// ```should_panic
/// use std::path::Path;
///
/// // This panics - cannot move "photos" into "photos/archive"
/// move_album_fs(
///     Path::new("photos"),
///     Path::new("photos/archive")
/// ).unwrap();
/// ```
pub fn move_album_fs(album_path: &Path, destination_path: &Path) -> Result<(), Error> {
    let storage_root = PathBuf::from(std::env::var("STORAGE_ROOT").unwrap());
    let src_path = storage_root.join(album_path);
    let dest_path = storage_root.join(destination_path);

    // Make sure the source exists and is a directory
    if !src_path.is_dir() {
        return Err(Error::new(std::io::ErrorKind::NotFound, format!("Source album directory {} does not exist or is not a directory", src_path.display())));
    }

    // Make sure the destination's parent exists and is a directory
    if !dest_path.parent().map(|parent| parent.is_dir()).unwrap_or(false) {
        return Err(Error::new(std::io::ErrorKind::NotFound, format!("Destination album directory {} not found", destination_path.display())));
    }

    // Make sure the destination path is clear
    if dest_path.exists() {
        return Err(Error::new(std::io::ErrorKind::AlreadyExists, format!("Destination {} already exists", destination_path.display())));
    }

    // Make sure the target album is not a child of the source album
    if dest_path.starts_with(&src_path) {
        return Err(Error::new(std::io::ErrorKind::InvalidInput, "Target album is a child of the album to be moved"));
    }

    println!("Moving album {} to {}", src_path.display(), dest_path.display());
    fs::rename(src_path, dest_path)?;

    Ok(())
}

