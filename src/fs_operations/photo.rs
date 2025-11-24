use std::fs;
use std::io::Error;
use std::path::{Path, PathBuf};


/// Deletes a photo, its thumbnail, andassociated files from the filesystem and clearing any empty
/// thumbnail directories.
///
/// # Arguments
/// * `photo_path` - Path to the photo, relative to $STORAGE_ROOT
/// * `thumb_path` - Path to the thumbnail, relative to $THUMBNAIL_ROOT
///
/// # Returns
/// Ok if all files were deleted successfully, or an error if deletion failed.
/// Also removes empty parent directories from the thumbnail path.
pub fn delete_photo_fs(photo_path: &Path, thumb_path: &Path) -> Result<(), Error> {
    let storage_root = PathBuf::from(std::env::var("STORAGE_ROOT").unwrap());
    let thumbnail_root = PathBuf::from(std::env::var("THUMBNAIL_ROOT").unwrap());

    let full_photo_path = storage_root.join(&photo_path);
    let mut full_thumb_path = thumbnail_root.join(thumb_path);

    // Delete photo & thumbnail from hard drive
    fs::remove_file(&full_photo_path)?;
    fs::remove_file(&full_thumb_path)?;
    full_thumb_path.pop();

    // Also delete other associated files (e.g. exports, editor metadata, etc.)
    // First extract the base name (without extension) from the photo filename
    let base_name = full_photo_path
        .file_prefix()
        .unwrap_or(full_photo_path.file_name().unwrap());

    // Find all files in the parent directory that match the pattern <base_name>* and deletes them
    fs::read_dir(full_photo_path.parent().unwrap())?
        .flatten()
        // Filter out non-files
        .filter(|entry| entry.file_type().map(|ft| ft.is_file()).unwrap_or(false))
        // Filter out files that don't match the same file prefix (e.g. "IMG_20210101_123456" vs "IMG_20210101_123456.jpg"
        .filter(|entry| entry.path().file_prefix().map(|prefix| prefix == base_name).unwrap_or(false))
        // Move files to the new directory
        .try_for_each(|entry| {
            fs::remove_file(entry.path())
        })?;

    // Delete empty thumbnail directories
    while full_thumb_path != thumbnail_root {
        if fs::remove_dir(&full_thumb_path).is_err() {
            break;
        }
        full_thumb_path.pop();
    }

    Ok(())
}



/// Move a photo and its associated files to a new album
///
/// # Arguments
/// * `photo_path` - Path to the photo, relative to $STORAGE_ROOT
/// * `dest_path` - Path to the destination album, relative to $STORAGE_ROOT
///
/// # Returns
/// Ok if all files were moved successfully, or an error if something failed.
pub fn move_photo_fs(photo_path: &Path, dest_path: &Path) -> Result<(), Error> {
    let storage_root = PathBuf::from(std::env::var("STORAGE_ROOT").unwrap());
    let full_photo_path = storage_root.join(&photo_path);
    let full_dest_path = storage_root.join(&dest_path);

    // Extract the base name (without extension) from the photo filename
    let base_name = full_photo_path
        .file_prefix()
        .unwrap_or(photo_path.file_name().unwrap());

    // Find all files in the album directory that match the pattern <base_name>* and move them to the unfiled directory
    fs::read_dir(&full_photo_path)?
        .flatten()
        // Filter out non-files
        .filter(|entry| entry.file_type().map(|ft| ft.is_file()).unwrap_or(false))
        // Filter out files that don't match the same file prefix (e.g. "IMG_20210101_123456" vs "IMG_20210101_123456.jpg"
        .filter(|entry| entry.path().file_prefix().map(|prefix| prefix == base_name).unwrap_or(false))
        // Move files to the new directory
        .try_for_each(|entry| {
            fs::rename(
                entry.path(),  // Already the full path
                full_dest_path.join(entry.file_name())
            )
        })?;

    Ok(())
}
