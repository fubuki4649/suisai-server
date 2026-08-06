use crate::db::entities::collections;
use crate::models::collection::{Collection, NewCollection, UpdateCollection};
use crate::patch_fields;
use sea_orm::{entity::*, DatabaseConnection, DbErr, QueryFilter, Set};

/// Creates a new collection in the database
///
/// # Arguments
/// * `db` - Database connection
/// * `collection` - Collection details for creation
///
/// # Returns
/// The UUID of the newly created collection, or an error
pub async fn new_collection(db: &DatabaseConnection, collection: NewCollection) -> Result<String, DbErr> {
    let id = uuid::Uuid::now_v7().to_string();

    let active_model = collections::ActiveModel {
        id: Set(id.clone()),
        label: Set(collection.label),
        parent_id: Set(collection.parent_id),
    };

    active_model.insert(db).await?;

    Ok(id)
}

/// Deletes the specified collection from the database
///
/// # Arguments
/// * `db` - Database connection
/// * `collection_id` - UUID of the collection to delete
///
/// # Returns
/// The deleted `Collection` if found, or an error if the collection doesn't exist
pub async fn delete_collection(db: &DatabaseConnection, collection_id: String) -> Result<Collection, DbErr> {
    // Fetch
    let collection = collections::Entity::find_by_id(collection_id.clone())
        .one(db)
        .await?
        .ok_or_else(|| DbErr::RecordNotFound(format!("Collection {} not found", collection_id)))?;

    // Delete
    collections::Entity::delete_by_id(collection_id).exec(db).await?;

    Ok(collection.into())
}

/// Updates an existing collection in the database using partial update (only set fields are changed)
///
/// # Arguments
/// * `db` - Database connection
/// * `collection_id` - UUID of the collection to update
/// * `update` - Fields to update; only `Some` fields are applied
///
/// # Returns
/// The updated `Collection`, or an error if the collection doesn't exist
pub async fn update_collection(db: &DatabaseConnection, collection_id: String, update: UpdateCollection) -> Result<Collection, DbErr> {
    let existing = collections::Entity::find_by_id(collection_id.clone())
        .one(db)
        .await?
        .ok_or_else(|| DbErr::RecordNotFound(format!("Collection {} not found", collection_id)))?;

    let mut active_model: collections::ActiveModel = existing.into();

    patch_fields!(active_model, update, {
        label,
        parent_id,
    });

    let updated = active_model.update(db).await?;

    Ok(updated.into())
}

/// Gets collections by their IDs
///
/// # Arguments
/// * `db` - Database connection
/// * `collection_ids` - Slice of collection UUIDs to retrieve
///
/// # Returns
/// Vector of collections matching the provided IDs, or an error
pub async fn get_collections(db: &DatabaseConnection, collection_ids: &[String]) -> Result<Vec<Collection>, DbErr> {
    if collection_ids.is_empty() { return Ok(vec![]); }

    collections::Entity::find()
        .filter(collections::Column::Id.is_in(collection_ids.to_vec()))
        .into_partial_model::<Collection>()
        .all(db)
        .await
}

/// Gets collections by parent ID, or root-level collections if `parent_id` is `None`
///
/// # Arguments
/// * `db` - Database connection
/// * `parent_id` - UUID of the parent collection, or `None` to get root-level collections
///
/// # Returns
/// A list of matching collections, or an error
pub async fn get_collections_by_parent(db: &DatabaseConnection, parent_id: Option<String>) -> Result<Vec<Collection>, DbErr> {
    let query = collections::Entity::find();
    let query = match parent_id {
        Some(id) => query.filter(collections::Column::ParentId.eq(id)),
        None => query.filter(collections::Column::ParentId.is_null()),
    };
    query.into_partial_model::<Collection>().all(db).await
}

/// Gets all collections in a flat list
///
/// # Arguments
/// * `db` - Database connection
///
/// # Returns
/// A list of all collections, or an error
pub async fn get_all_collections(db: &DatabaseConnection) -> Result<Vec<Collection>, DbErr> {
    collections::Entity::find()
        .into_partial_model::<Collection>()
        .all(db)
        .await
}
