use crate::db::entities::{assets, collections};
use sea_orm::{DatabaseConnection, DbErr, EntityTrait, QuerySelect};
use std::collections::HashSet;
use std::path::PathBuf;

/// Gets a collection's path, relative to $STORAGE_ROOT
///
/// # Arguments
/// * `db` - Database connection
/// * `collection_id` - UUID of the collection to get a path for
///
/// # Returns
/// The path of the collection; Returns a `DbErr::Custom` if a cyclical path is detected
pub async fn get_collection_path(db: &DatabaseConnection, collection_id: String) -> Result<PathBuf, DbErr> {

    // Collect the chain of collection labels from the current collection up to root to build a
    // path by climbing an inverse tree
    let mut segments: Vec<String> = Vec::new();
    let mut current_id: Option<String> = Some(collection_id);
    let mut seen: HashSet<String> = HashSet::new();

    while let Some(cid) = current_id {
        // Check for cycles (shouldn't happen, but just in case)
        if !seen.insert(cid.clone()) {
            return Err(DbErr::Custom("Cycle detected in collection relations! This is a catastrophic error that usually means data corruption".to_owned()));
        }

        // Fetch the label and parent_id for this collection
        let (label, parent_id): (String, Option<String>) = collections::Entity::find_by_id(cid.clone())
            .select_only()
            .column(collections::Column::Label)
            .column(collections::Column::ParentId)
            .into_tuple()
            .one(db)
            .await?
            .ok_or_else(|| DbErr::RecordNotFound(format!("Collection {} not found", cid)))?;

        segments.push(label);
        current_id = parent_id;
    }

    // Build the path from root to leaf: segments were collected leaf->root, so reverse
    segments.reverse();
    let mut path = PathBuf::new();
    for seg in segments {
        path.push(seg);
    }
    Ok(path)
}


/// Gets an asset's path, relative to $STORAGE_ROOT
///
/// # Arguments
/// * `db` - Database connection
/// * `asset_id` - UUID of the asset to get path for
///
/// # Returns
/// The path of the asset
pub async fn get_asset_path(db: &DatabaseConnection, asset_id: String) -> Result<PathBuf, DbErr> {

    // Get the asset file name and parent collection (and confirm the asset exists)
    let (file_name, parent_id): (String, Option<String>) = assets::Entity::find_by_id(asset_id.clone())
        .select_only()
        .column(assets::Column::FileName)
        .column(assets::Column::ParentId)
        .into_tuple()
        .one(db)
        .await?
        .ok_or_else(|| DbErr::RecordNotFound(format!("Asset {} not found", asset_id)))?;

    // Build the path
    let mut path = match parent_id {
        Some(collection_id) => get_collection_path(db, collection_id).await?,
        None => PathBuf::new(),
    };

    path.push(file_name);
    Ok(path)
}
