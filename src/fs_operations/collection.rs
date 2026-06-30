use crate::_utils::path_prefix::PathPrefix;
use std::fs;
use std::io::Error;
use std::path::{Path, PathBuf};


/// Represents a collection (album/directory) on the filesystem, identified by its path relative
/// to $STORAGE_ROOT.
pub struct Collection {
    /// Path to the collection directory, relative to $STORAGE_ROOT
    pub collection_path: PathBuf,
}

impl Collection {
    pub fn new(collection_path: &Path) -> Self {
        Self {
            collection_path: collection_path.to_path_buf(),
        }
    }

    /// Creates a new collection directory at the storage root.
    ///
    /// # Arguments
    /// * `collection_name` - The name of the new collection
    ///
    /// # Returns
    /// Ok if the collection was successfully created at `$STORAGE_ROOT/collection_name`
    pub fn create(collection_name: &str) -> Result<(), Error> {
        let storage_root = PathBuf::from(std::env::var("STORAGE_ROOT").unwrap());
        fs::create_dir(storage_root.join(collection_name))?;
        Ok(())
    }

    /// Deletes the collection, moving its children to the root (collections) / unfiled (assets).
    ///
    /// # Returns
    /// Ok if the collection was deleted successfully and its children moved, or an error if deletion failed.
    pub fn delete(self) -> Result<(), Error> {
        let storage_root = PathBuf::from(std::env::var("STORAGE_ROOT").unwrap());
        let full_collection_path = self.collection_path.prefix(&storage_root);
        let unfiled_path = storage_root.join("unfiled");

        // Iterate over direct children and move them out
        for entry in fs::read_dir(&full_collection_path)?.flatten() {
            let entry_path = entry.path();
            if entry_path.is_file() {
                // Move asset files to the unfiled directory
                let dest = unfiled_path.join(entry.file_name());
                fs::rename(&entry_path, dest)?;
            } else if entry_path.is_dir() {
                // Move sub-collections to root
                let dest = storage_root.join(entry.file_name());
                fs::rename(&entry_path, dest)?;
            }
        }

        // Delete the now-empty collection directory
        fs::remove_dir_all(full_collection_path)?;

        Ok(())
    }

    /// Moves the entire collection (and its children) to a new location.
    ///
    /// # Arguments
    /// * `destination_path` - Path to the destination, relative to $STORAGE_ROOT. This must not be
    ///   a child of the collection's current path.
    ///
    /// # Returns
    /// Ok if successful, or Error if validation fails or filesystem operation fails.
    ///
    /// # Examples
    ///
    /// Move a collection to a different parent:
    /// ```no_run
    /// use std::path::Path;
    /// use suisai_server::fs_operations::collection::Collection;
    ///
    /// // Move "2023/vacation" to "archived/2023/vacation"
    /// Collection::new(Path::new("2023/vacation"))
    ///     .move_to(Path::new("archived/2023/vacation"))?;
    /// # Ok::<(), std::io::Error>(())
    /// ```
    ///
    /// Move a collection to the root level:
    /// ```no_run
    /// use std::path::Path;
    /// use suisai_server::fs_operations::collection::Collection;
    ///
    /// // Move "2023/events/birthday" to "birthday"
    /// Collection::new(Path::new("2023/events/birthday"))
    ///     .move_to(Path::new("birthday"))?;
    /// # Ok::<(), std::io::Error>(())
    /// ```
    ///
    /// This will fail because the destination is a child of the source:
    /// ```should_panic
    /// use std::path::Path;
    /// use suisai_server::fs_operations::collection::Collection;
    ///
    /// // This panics - cannot move "photos" into "photos/archive"
    /// Collection::new(Path::new("photos"))
    ///     .move_to(Path::new("photos/archive"))
    ///     .unwrap();
    /// ```
    pub fn move_to(&self, destination_path: &Path) -> Result<(), Error> {
        let storage_root = PathBuf::from(std::env::var("STORAGE_ROOT").unwrap());
        let src_path = self.collection_path.prefix(&storage_root);
        let dest_path = destination_path.prefix(&storage_root);

        // Make sure the source exists and is a directory
        if !src_path.is_dir() {
            return Err(Error::new(std::io::ErrorKind::NotFound, format!("Source collection directory {} does not exist or is not a directory", src_path.display())));
        }

        // Make sure the destination's parent exists and is a directory
        if !dest_path.parent().map(|parent| parent.is_dir()).unwrap_or(false) {
            return Err(Error::new(std::io::ErrorKind::NotFound, format!("Destination collection directory {} not found", destination_path.display())));
        }

        // Make sure the destination path is clear
        if dest_path.exists() {
            return Err(Error::new(std::io::ErrorKind::AlreadyExists, format!("Destination {} already exists", destination_path.display())));
        }

        // Make sure the target collection is not a child of the source collection
        if dest_path.starts_with(&src_path) {
            return Err(Error::new(std::io::ErrorKind::InvalidInput, "Target collection is a child of the collection to be moved"));
        }

        println!("Moving collection {} to {}", src_path.display(), dest_path.display());
        fs::rename(src_path, dest_path)?;

        Ok(())
    }
}
