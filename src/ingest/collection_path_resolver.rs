use crate::db::operations::collection::new_collection;
use crate::models::collection::NewCollection;
use sea_orm::DatabaseConnection;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::sync::Mutex;

/// Result of resolving a relative source directory into a collection.
#[derive(Debug, Clone)]
pub struct ResolvedCollection {
    /// Database ID of the innermost collection.
    pub collection_id: String,
    /// Absolute filesystem path under `$STORAGE_ROOT` where files should be placed.
    pub dest_dir: PathBuf,
}

/// Resolves a source directory's relative path into a nested chain of database collections
/// and on-disk directories rooted under the current ingest batch collection.
///
/// Ingest workers run in parallel and may process files from the same directory at the same
/// time. Because there is no database-level unique constraint on collection labels under
/// a parent, this resolver uses an internal mutex-protected cache to ensure intermediate
/// collections are created exactly once.
pub struct CollectionPathResolver {
    db: DatabaseConnection,
    ingest_root_id: String,
    ingest_root_label: String,
    storage_root: PathBuf,
    cache: Mutex<HashMap<PathBuf, String>>,
}

impl CollectionPathResolver {
    /// Creates a new resolver for the given ingest batch collection.
    ///
    /// # Arguments
    /// * `db` - Database connection for inserting collection records.
    /// * `ingest_root_id` - Database UUID of the top-level ingest batch collection (e.g. `ingest-09-29-2026`).
    /// * `ingest_root_label` - Label/folder name of the top-level ingest collection.
    /// * `storage_root` - Base storage root directory path (`$STORAGE_ROOT`).
    pub fn new(
        db: DatabaseConnection,
        ingest_root_id: String,
        ingest_root_label: String,
        storage_root: PathBuf,
    ) -> Self {
        Self {
            db,
            ingest_root_id,
            ingest_root_label,
            storage_root,
            cache: Mutex::new(HashMap::new()),
        }
    }

    /// Resolves `rel_dir` (e.g. `2024/Vacation`) into a database collection and storage destination directory.
    ///
    /// If `rel_dir` is empty, returns the ingest batch root collection and directory directly.
    /// Otherwise, creates any missing intermediate collections in the database and directories on disk.
    pub async fn resolve(&self, rel_dir: &Path) -> ResolvedCollection {
        let components: Vec<&str> = rel_dir
            .components()
            .filter_map(|c| c.as_os_str().to_str())
            .collect();

        if components.is_empty() {
            return ResolvedCollection {
                collection_id: self.ingest_root_id.clone(),
                dest_dir: self.storage_root.join(&self.ingest_root_label),
            };
        }

        let mut parent_id = self.ingest_root_id.clone();
        let mut fs_path = PathBuf::from(&self.ingest_root_label);

        for component in &components {
            let key = fs_path.join(component);

            let mut guard = self.cache.lock().await;
            if let Some(id) = guard.get(&key) {
                parent_id = id.clone();
            } else {
                let id = new_collection(
                    &self.db,
                    NewCollection {
                        label: component.to_string(),
                        parent_id: Some(parent_id.clone()),
                    },
                )
                .await
                .unwrap_or_else(|e| panic!("Failed to create collection '{component}': {e}"));

                crate::fs_operations::collection::Collection::create(&key)
                    .unwrap_or_else(|e| panic!("Failed to create directory '{}': {e}", key.display()));

                println!("Created sub-collection: {} ({id})", key.display());

                guard.insert(key.clone(), id.clone());
                parent_id = id;
            }
            fs_path = key;
        }

        ResolvedCollection {
            collection_id: parent_id,
            dest_dir: self.storage_root.join(&fs_path),
        }
    }
}
