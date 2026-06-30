use crate::_utils::path_prefix::PathPrefix;
use std::fs;
use std::io::Error;
use std::path::{Path, PathBuf};

/// Represents an asset (photo/file) on the filesystem, identified by its path relative to
/// $STORAGE_ROOT and its optional thumbnail path relative to $THUMBNAIL_ROOT.
pub struct Asset {
    /// Path to the asset, relative to $STORAGE_ROOT
    pub asset_path: PathBuf,
    /// Path to the thumbnail, relative to $THUMBNAIL_ROOT (required for deletion)
    pub thumb_path: Option<PathBuf>,
}

impl Asset {
    pub fn new(asset_path: &Path, thumb_path: &Path) -> Self {
        Self {
            asset_path: asset_path.to_path_buf(),
            thumb_path: Some(thumb_path.to_path_buf()),
        }
    }

    pub fn new_without_thumb(asset_path: &Path) -> Self {
        Self {
            asset_path: asset_path.to_path_buf(),
            thumb_path: None,
        }
    }

    /// Move the asset and its associated files to a new collection directory.
    ///
    /// # Arguments
    /// * `dest_path` - Path to the destination collection, relative to $STORAGE_ROOT
    ///
    /// # Returns
    /// Ok if all files were moved successfully, or an error if something failed.
    pub fn move_to(&self, dest_path: &Path) -> Result<(), Error> {
        let storage_root = PathBuf::from(std::env::var("STORAGE_ROOT").unwrap());
        let full_asset_path = self.asset_path.prefix(&storage_root);
        let full_dest_path = dest_path.prefix(&storage_root);

        // Extract the base name (without extension) from the asset filename
        let base_name = full_asset_path
            .file_prefix()
            .unwrap_or(self.asset_path.file_name().unwrap());

        // Find all files in the collection directory that match the pattern <base_name>* and move them
        fs::read_dir(full_asset_path.parent().unwrap())?
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

    /// Deletes the asset, its thumbnail, and associated files from the filesystem, clearing any
    /// empty thumbnail directories.
    ///
    /// # Returns
    /// Ok if all files were deleted successfully, or an error if deletion failed.
    /// Also removes empty parent directories from the thumbnail path.
    pub fn delete(self) -> Result<(), Error> {
        let storage_root = PathBuf::from(std::env::var("STORAGE_ROOT").unwrap());
        let thumbnail_root = PathBuf::from(std::env::var("THUMBNAIL_ROOT").unwrap());

        let full_asset_path = self.asset_path.prefix(&storage_root);
        let mut full_thumb_path = self.thumb_path
            .expect("thumb_path is required for delete")
            .prefix(&thumbnail_root);

        // Delete thumbnail from hard drive
        fs::remove_file(&full_thumb_path)?;
        full_thumb_path.pop();

        // Delete the asset itself and also other associated files (e.g. exports, metadata, etc.)
        // Basically, anything with the same file step without the extension.
        
        // First, extract the base name (without extension) from the asset filename
        let base_name = full_asset_path
            .file_prefix()
            .unwrap_or(full_asset_path.file_name().unwrap());

        // Find all files in the parent directory that match the pattern <base_name>* and delete them
        fs::read_dir(full_asset_path.parent().unwrap())?
            .flatten()
            // Filter out non-files
            .filter(|entry| entry.file_type().map(|ft| ft.is_file()).unwrap_or(false))
            // Filter out files that don't match the same file prefix (e.g. "IMG_20210101_123456" vs "IMG_20210101_123456.jpg"
            .filter(|entry| entry.path().file_prefix().map(|prefix| prefix == base_name).unwrap_or(false))
            // Delete files
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
}
