use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;
use serde_json::Value;

use crate::_utils::json_map::JsonMap;
use crate::db::operations::asset::get_assets_by_parent;
use crate::db::operations::collection::{delete_collection, get_all_collections, new_collection, update_collection};
use crate::db::operations::paths::get_collection_path;
use crate::fs_operations::collection::Collection as FsCollection;
use crate::models::asset::Asset;
use crate::models::collection::{Collection, NewCollection, UpdateCollection};
use crate::{msg, state::AppState};

type Response = Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)>;

/// A collection, but as a tree-shaped data structure (as opposed to the original inverse tree)
#[derive(Debug, Serialize)]
pub struct CollectionTree {
    pub id: String,
    pub label: String,
    pub children: Vec<CollectionTree>,
}

impl From<Collection> for CollectionTree {
    fn from(c: Collection) -> Self {
        CollectionTree { id: c.id, label: c.label, children: vec![] }
    }
}


/// Retrieves the collection tree structure from the database
///
/// # Route
/// `GET /collection/tree`
///
/// # Returns
/// - `200 OK`: A tree structure representing all collections
/// - `500 Internal Server Error`: Database error
///
/// # Response Body
/// A tree node containing:
/// - `id`: Collection UUID (`"-1"` for the synthetic root node)
/// - `label`: Collection label (root node label is a placeholder — ignore it)
/// - `children`: Nested `CollectionTree` nodes
pub async fn get_collection_tree(State(state): State<AppState>) -> Result<Json<CollectionTree>, (StatusCode, Json<Value>)> {

    // Recursively fill the children, grandchildren, etc of the node
    fn build_subtree(parent_id: Option<&str>, all: &[Collection]) -> Vec<CollectionTree> {
        all.iter()
            .filter(|c| c.parent_id.as_deref() == parent_id)
            .map(|c| CollectionTree {
                id: c.id.clone(),
                label: c.label.clone(),
                children: build_subtree(Some(&c.id), all),
            })
            .collect()
    }

    let all = get_all_collections(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))?;

    let tree = CollectionTree {
        id: "-1".to_string(),
        label: "Root Node - Not a Collection!!!".to_string(),
        children: build_subtree(None, &all),
    };

    Ok(Json(tree))
}

/// Retrieves all collections from the database in a flat list
///
/// # Route
/// `GET /collection/flat`
///
/// # Returns
/// - `200 OK`: A flat array of all collections
/// - `500 Internal Server Error`: Database error
pub async fn get_collection_flat(State(state): State<AppState>) -> Result<Json<Vec<Collection>>, (StatusCode, Json<Value>)> {
    get_all_collections(&state.db).await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))
}

/// Creates a new collection and its backing directory on disk
///
/// # Route
/// `POST /collection/new`
///
/// # Request Body
/// JSON object with:
/// - `label`: Name for the new collection
/// - `parentId`: UUID of the parent collection (**optional** — omit or pass `null` to create at root)
///
/// # Returns
/// - `201 Created`: Collection was successfully created; body contains the new UUID
/// - `400 Bad Request`: Missing or invalid `label`
/// - `500 Internal Server Error`: Database or filesystem error
pub async fn new_collection_handler(State(state): State<AppState>, input: Json<Value>) -> Response {
    let label = input.get_value::<String>("label")
        .map_err(|e| (StatusCode::BAD_REQUEST, msg!(e.to_string())))?;

    let parent_id = input.get_value::<Option<String>>("parent_id")
        .map_err(|e| (StatusCode::BAD_REQUEST, msg!(e.to_string())))?;

    // Resolve the parent path so we know where to create the directory
    let parent_path = match &parent_id {
        Some(id) => get_collection_path(&state.db, id.clone()).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))?,
        None => std::path::PathBuf::new(),
    };

    // Create the directory on disk (path is relative to `$STORAGE_ROOT`, which Collection::create handles internally)
    FsCollection::create(&parent_path.join(&label).to_string_lossy())
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))?;

    // Create the record in the database
    let id = new_collection(&state.db, NewCollection { label, parent_id }).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))?;

    Ok((StatusCode::CREATED, msg!("Created collection {}", id)))
}

/// Renames an existing collection and moves its directory on disk
///
/// # Route
/// `PATCH /collection/:id/rename`
///
/// # URL Parameters
/// - `id`: UUID of the collection to rename
///
/// # Request Body
/// JSON object with:
/// - `label`: New name for the collection
///
/// # Returns
/// - `200 OK`: Collection was successfully renamed
/// - `400 Bad Request`: Missing or invalid `label`
/// - `404 Not Found`: Collection with the specified ID does not exist
/// - `500 Internal Server Error`: Database or filesystem error
pub async fn rename_collection(Path(id): Path<String>, State(state): State<AppState>, input: Json<Value>) -> Response {
    let label = input.get_value::<String>("label")
        .map_err(|e| (StatusCode::BAD_REQUEST, msg!(e.to_string())))?;

    // Resolve old path and compute new path (same parent, new label)
    let old_path = get_collection_path(&state.db, id.clone()).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))?;

    let new_path = old_path.parent()
        .ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, msg!("Failed to resolve the parent dir of the collection!")))?
        .join(&label);

    // Move the directory on disk
    FsCollection::new(&old_path).move_to(&new_path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))?;

    // Update the label in the database
    update_collection(&state.db, id, UpdateCollection { label: Some(label), ..Default::default() }).await
        .map(|_| (StatusCode::OK, msg!("Success")))
        .map_err(|e| match e {
            sea_orm::DbErr::RecordNotFound(_) => (StatusCode::NOT_FOUND, msg!("Collection not found")),
            e => (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())),
        })
}

/// Deletes a collection from the database and moves its children out on disk
///
/// Sub-collections are moved to root; asset files are moved to the unfiled directory.
///
/// # Route
/// `DELETE /collection/:id/delete`
///
/// # URL Parameters
/// - `id`: UUID of the collection to delete
///
/// # Returns
/// - `200 OK`: Collection was successfully deleted
/// - `404 Not Found`: Collection with the specified ID does not exist
/// - `500 Internal Server Error`: Database or filesystem error
pub async fn del_collection(Path(id): Path<String>, State(state): State<AppState>) -> Response {
    // Resolve the path before deleting the DB record
    let collection_path = get_collection_path(&state.db, id.clone()).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))?;

    // Move children out and remove the directory on disk
    FsCollection::new(&collection_path).delete()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))?;

    // Delete the record from the database
    delete_collection(&state.db, id).await
        .map(|_| (StatusCode::OK, msg!("Success")))
        .map_err(|e| match e {
            sea_orm::DbErr::RecordNotFound(_) => (StatusCode::NOT_FOUND, msg!("Collection not found")),
            e => (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())),
        })
}

/// Retrieves all assets belonging to a given collection
///
/// # Route
/// `GET /collection/:id/assets`
///
/// # URL Parameters
/// - `id`: UUID of the collection
///
/// # Returns
/// - `200 OK`: JSON array of assets in the collection
/// - `500 Internal Server Error`: Database error
pub async fn assets_in_collection(Path(id): Path<String>, State(state): State<AppState>) -> Result<Json<Vec<Asset>>, (StatusCode, Json<Value>)> {
    get_assets_by_parent(&state.db, Some(id)).await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))
}

/// Retrieves all assets not assigned to any collection
///
/// # Route
/// `GET /collection/unfiled/assets`
///
/// # Returns
/// - `200 OK`: JSON array of unfiled assets
/// - `500 Internal Server Error`: Database error
pub async fn unfiled_assets(State(state): State<AppState>) -> Result<Json<Vec<Asset>>, (StatusCode, Json<Value>)> {
    get_assets_by_parent(&state.db, None).await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))
}
