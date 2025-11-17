use std::fs;
use std::io::Error;
use std::path::{Path, PathBuf};
use crate::models::db::album::Album;
use crate::models::db::photo::Photo;

/// Deletes an album, moving its children to the root (albums) / unfiled (photos).
///
/// # Arguments
/// * `album_path` - Path to the album directory, relative to $STORAGE_ROOT
/// * `child_photos` - Photos linked to the album
/// * `child_albums` - Subalbums linked to the album
///
/// # Returns
/// Ok if the album was deleted successfully and its children moved, or an error if deletion failed.
pub fn delete_album_fs(album_path: PathBuf, child_photos: &[Photo], child_albums: &[Album]) -> Result<(), Error> {
    let storage_root = PathBuf::from(std::env::var("STORAGE_ROOT").unwrap());
    let full_album_path = storage_root.join(&album_path);
    let unfiled_path = storage_root.join("unfiled");

    // Move child photos to unfiled directory
    for photo in child_photos {
        let photo_filename = &photo.file_name;

        // Extract the base name (without extension) from the photo filename
        let base_name = Path::new(&photo_filename)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(photo_filename);

        // Find all files in the album directory that match the pattern <base_name>*
        fs::read_dir(&full_album_path)?
            .flatten()
            .filter(|entry| entry.file_type().map(|ft| ft.is_file()).unwrap_or(false))
            .filter_map(|entry| entry.file_name().into_string().ok())
            .filter(|file_name| file_name.starts_with(base_name))
            .try_for_each(|file_name| {
                // Moves file to the new directory
                fs::rename(
                    full_album_path.join(&file_name),
                    unfiled_path.join(&file_name)
                )
            })?;
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

