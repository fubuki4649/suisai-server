use std::fs;
use std::io::Error;
use std::path::PathBuf;


/// Deletes a photo and its thumbnail from the filesystem, and clearing any empty thumbnail directories.
///
/// # Arguments
/// * `photo_path` - Path to the photo, relative to $STORAGE_ROOT
/// * `thumb_path` - Path to the thumbnail, relative to $THUMBNAIL_ROOT
///
/// # Returns
/// Ok if both files were deleted successfully, or an error if deletion failed.
/// Also removes empty parent directories from the thumbnail path.
pub fn delete_photo_fs(photo_path: PathBuf, thumb_path: PathBuf) -> Result<(), Error> {
    let storage_root = PathBuf::from(std::env::var("STORAGE_ROOT").unwrap());
    let thumbnail_root = PathBuf::from(std::env::var("THUMBNAIL_ROOT").unwrap());

    let mut full_thumb_path = thumbnail_root.join(thumb_path);

    // Delete photo & thumbnail from hard drive
    fs::remove_file(storage_root.join(photo_path))?;
    fs::remove_file(&full_thumb_path)?;
    full_thumb_path.pop();

    // Delete empty thumbnail directories
    while full_thumb_path != thumbnail_root {
        if fs::remove_dir(&full_thumb_path).is_err() {
            break;
        }
        full_thumb_path.pop();
    }

    Ok(())
}

