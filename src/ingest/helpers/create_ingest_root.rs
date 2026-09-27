use crate::db::operations::collection::{find_collection_by_label, new_collection};
use crate::models::collection::NewCollection;
use chrono::{Datelike, Utc};
use sea_orm::DatabaseConnection;

/// Creates the root `ingest-MM-DD-YYYY` collection, appending `-2`, `-3`, etc. if one already exists.
/// Returns `(collection_id, collection_label)`.
pub async fn create_ingest_root(db: &DatabaseConnection) -> (String, String) {
    let today = Utc::now();
    let base_label = format!("ingest-{:02}-{:02}-{}", today.month(), today.day(), today.year());

    // Find an unused label
    let mut label = base_label.clone();
    let mut suffix = 2u32;
    while find_collection_by_label(db, &label, None).await.ok().flatten().is_some() {
        label = format!("{base_label}-{suffix}");
        suffix += 1;
    }

    // Create in DB
    let id = new_collection(db, NewCollection { label: label.clone(), parent_id: None })
        .await
        .unwrap_or_else(|e| panic!("Failed to create ingest root collection: {e}"));

    // Create on disk
    crate::fs_operations::collection::Collection::create(&label)
        .unwrap_or_else(|e| panic!("Failed to create ingest root directory: {e}"));

    println!("Created root collection: {label} ({id})");
    (id, label)
}
