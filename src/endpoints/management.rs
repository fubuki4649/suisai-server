use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::_utils::json_map::JsonMap;
use crate::db::operations::asset::{get_assets, update_asset};
use crate::db::operations::collection::{get_collections, update_collection};
use crate::db::operations::paths::get_collection_path;
use crate::fs_operations::asset::Asset as FsAsset;
use crate::fs_operations::collection::Collection as FsCollection;
use crate::models::asset::UpdateAsset;
use crate::models::collection::UpdateCollection;
use crate::{msg, state::AppState};

type Response = Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)>;

/// Removes assets from their current collection and moves them to the unfiled directory
///
/// # Route
/// `POST /management/asset/unfile`
///
/// # Request Body
/// JSON object with:
/// - `assetIds`: array of asset UUID strings to unfile
///
/// # Returns
/// - `200 OK`: Assets successfully moved to unfiled
/// - `400 Bad Request`: Missing or invalid `assetIds`
/// - `500 Internal Server Error`: Database or filesystem error
pub async fn unfile_asset(State(state): State<AppState>, input: Json<Value>) -> Response {
    let asset_ids = input.get_value::<Vec<String>>("asset_ids")
        .map_err(|e| (StatusCode::BAD_REQUEST, msg!(e.to_string())))?;

    let assets = get_assets(&state.db, &asset_ids).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))?;

    let mut collection_paths: HashMap<String, PathBuf> = HashMap::new();

    for asset in assets {
        let Some(ref pid) = asset.parent_id else { continue; };
        let parent_path = if let Some(cached) = collection_paths.get(pid) {
            cached.clone()
        } else {
            let path = get_collection_path(&state.db, pid).await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))?;
            collection_paths.insert(pid.clone(), path.clone());
            path
        };

        let current_path = parent_path.join(&asset.file_name);

        FsAsset::new_without_thumb(&current_path).move_to(Path::new("unfiled"))
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))?;

        if let Err(e) = update_asset(&state.db, &asset.id, UpdateAsset { parent_id: Some(None), ..Default::default() }).await {
            let _ = FsAsset::new_without_thumb(&Path::new("unfiled").join(&asset.file_name)).move_to(&parent_path);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())));
        }
    }

    Ok((StatusCode::OK, msg!("Success")))
}

/// Moves assets from their current collection to a different one
///
/// # Route
/// `POST /management/asset/reassign`
///
/// # Request Body
/// JSON object with:
/// - `collectionId`: UUID of the destination collection
/// - `assetIds`: array of asset UUID strings to move
///
/// # Returns
/// - `200 OK`: Assets successfully moved to the new collection
/// - `400 Bad Request`: Missing or invalid `collectionId` or `assetIds`
/// - `500 Internal Server Error`: Database or filesystem error
pub async fn reassign_asset(State(state): State<AppState>, input: Json<Value>) -> Response {
    let collection_id = input.get_value::<String>("collection_id")
        .map_err(|e| (StatusCode::BAD_REQUEST, msg!(e.to_string())))?;
    let asset_ids = input.get_value::<Vec<String>>("asset_ids")
        .map_err(|e| (StatusCode::BAD_REQUEST, msg!(e.to_string())))?;

    let dest_path = get_collection_path(&state.db, &collection_id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))?;

    let assets = get_assets(&state.db, &asset_ids).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))?;

    let mut collection_paths: HashMap<String, PathBuf> = HashMap::new();

    for asset in assets {
        if asset.parent_id.as_deref() == Some(&collection_id) {
            continue;
        }

        let src_parent_path = match &asset.parent_id {
            Some(pid) => {
                if let Some(cached) = collection_paths.get(pid) {
                    cached.clone()
                } else {
                    let path = get_collection_path(&state.db, pid).await
                        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))?;
                    collection_paths.insert(pid.clone(), path.clone());
                    path
                }
            }
            None => PathBuf::from("unfiled"),
        };

        let current_path = src_parent_path.join(&asset.file_name);

        FsAsset::new_without_thumb(&current_path).move_to(&dest_path)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))?;

        if let Err(e) = update_asset(&state.db, &asset.id, UpdateAsset { parent_id: Some(Some(collection_id.clone())), ..Default::default() }).await {
            let _ = FsAsset::new_without_thumb(&dest_path.join(&asset.file_name)).move_to(&src_parent_path);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())));
        }
    }

    Ok((StatusCode::OK, msg!("Success")))
}

/// Removes collections from their parent and moves them to the root level
///
/// # Route
/// `POST /management/collection/unfile`
///
/// # Request Body
/// JSON object with:
/// - `collectionIds`: array of collection UUID strings to unfile
///
/// # Returns
/// - `200 OK`: Collections successfully moved to root
/// - `400 Bad Request`: Missing or invalid `collectionIds`
/// - `500 Internal Server Error`: Database or filesystem error
pub async fn unfile_collection(State(state): State<AppState>, input: Json<Value>) -> Response {
    let collection_ids = input.get_value::<Vec<String>>("collection_ids")
        .map_err(|e| (StatusCode::BAD_REQUEST, msg!(e.to_string())))?;

    let collections = get_collections(&state.db, &collection_ids).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))?;

    for collection in collections {
        if collection.parent_id.is_none() {
            continue;
        }

        let current_path = get_collection_path(&state.db, &collection.id).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))?;
        let root_dest = PathBuf::from(&collection.label);

        FsCollection::new(&current_path).move_to(&root_dest)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))?;

        if let Err(e) = update_collection(&state.db, &collection.id, UpdateCollection { parent_id: Some(None), ..Default::default() }).await {
            let _ = FsCollection::new(&root_dest).move_to(&current_path);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())));
        }
    }

    Ok((StatusCode::OK, msg!("Success")))
}

/// Moves collections from their current parent to a different one
///
/// # Route
/// `POST /management/collection/reassign`
///
/// # Request Body
/// JSON object with:
/// - `parentId`: UUID of the destination parent collection
/// - `collectionIds`: array of collection UUID strings to move
///
/// # Returns
/// - `200 OK`: Collections successfully moved to the new parent
/// - `400 Bad Request`: Missing or invalid `parentId` or `collectionIds`
/// - `500 Internal Server Error`: Database or filesystem error
pub async fn reassign_collection(State(state): State<AppState>, input: Json<Value>) -> Response {
    let parent_id = input.get_value::<String>("parent_id")
        .map_err(|e| (StatusCode::BAD_REQUEST, msg!(e.to_string())))?;
    let collection_ids = input.get_value::<Vec<String>>("collection_ids")
        .map_err(|e| (StatusCode::BAD_REQUEST, msg!(e.to_string())))?;

    if collection_ids.contains(&parent_id) {
        return Err((StatusCode::BAD_REQUEST, msg!("Cannot move a collection into itself")));
    }

    let dest_path = get_collection_path(&state.db, &parent_id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))?;

    let collections = get_collections(&state.db, &collection_ids).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))?;

    for collection in collections {
        if collection.parent_id.as_deref() == Some(&parent_id) {
            continue;
        }

        let current_path = get_collection_path(&state.db, &collection.id).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))?;
        let target_path = dest_path.join(&collection.label);

        FsCollection::new(&current_path).move_to(&target_path)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))?;

        if let Err(e) = update_collection(&state.db, &collection.id, UpdateCollection { parent_id: Some(Some(parent_id.clone())), ..Default::default() }).await {
            let _ = FsCollection::new(&target_path).move_to(&current_path);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())));
        }
    }

    Ok((StatusCode::OK, msg!("Success")))
}
