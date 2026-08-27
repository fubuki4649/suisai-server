use crate::_utils::path_prefix::PathPrefix;
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

/// Represents an asset (photo/file) on the filesystem, identified by its path relative to
/// `$STORAGE_ROOT` and its optional thumbnail path relative to `$THUMBNAIL_ROOT`.
pub struct Asset {
    /// Path to the asset, relative to `$STORAGE_ROOT`
    pub asset_path: PathBuf,
    /// Path to the thumbnail, relative to `$THUMBNAIL_ROOT` (required for deletion)
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
    /// * `dest_path` - Path to the destination collection, relative to `$STORAGE_ROOT`
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
        match full_asset_path.parent().filter(|p| p.exists()) {
            Some(parent) => {
                fs::read_dir(parent)?
                    .flatten()
                    .filter(|entry| entry.file_type().is_ok_and(|ft| ft.is_file()))
                    .filter(|entry| entry.path().file_prefix().is_some_and(|prefix| prefix == base_name))
                    .try_for_each(|entry| {
                        fs::rename(
                            entry.path(),
                            full_dest_path.join(entry.file_name()),
                        )
                    })?;

                Ok(())
            }
            None => {
                Err(Error::new(ErrorKind::NotFound, format!("Parent directory of {} does not exist!", full_asset_path.to_string_lossy())))
            }
        }

    }

    /// Deletes the asset, its thumbnail (if present), and associated files from the filesystem,
    /// clearing any empty thumbnail directories.
    ///
    /// # Returns
    /// Ok if all files were deleted successfully, or an error if deletion failed.
    pub fn delete(self) -> Result<(), Error> {
        let storage_root = PathBuf::from(std::env::var("STORAGE_ROOT").unwrap());
        let full_asset_path = self.asset_path.prefix(&storage_root);

        // Delete thumbnail from hard drive if thumbnail_path is set
        if let Some(thumb_path) = self.thumb_path {
            let thumbnail_root = PathBuf::from(std::env::var("THUMBNAIL_ROOT").unwrap());
            let mut full_thumb_path = thumb_path.prefix(&thumbnail_root);

            let _ = fs::remove_file(&full_thumb_path);
            full_thumb_path.pop();

            // Delete empty thumbnail directories
            while full_thumb_path != thumbnail_root {
                if fs::remove_dir(&full_thumb_path).is_err() {
                    break;
                }
                full_thumb_path.pop();
            }
        }

        // Delete the asset itself and also other associated files (e.g. exports, metadata, etc.)
        // Basically, anything with the same file stem without the extension.
        let base_name = full_asset_path
            .file_prefix()
            .unwrap_or_else(|| full_asset_path.file_name().unwrap());

        if let Some(parent) = full_asset_path.parent().filter(|p| p.exists()) {
            fs::read_dir(parent)?
                .flatten()
                .filter(|entry| entry.file_type().is_ok_and(|ft| ft.is_file()))
                .filter(|entry| entry.path().file_prefix().is_some_and(|prefix| prefix == base_name))
                .try_for_each(|entry| fs::remove_file(entry.path()))?;
        }

        Ok(())
    }
}
