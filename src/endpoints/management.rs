use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde_json::Value;
use std::path::Path;

use crate::_utils::json_map::JsonMap;
use crate::db::operations::asset::{get_assets, update_asset};
use crate::db::operations::collection::{get_collections, update_collection};
use crate::db::operations::paths::{get_asset_path, get_collection_path};
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

    for asset in assets {
        let current_path = get_asset_path(&state.db, asset.id.clone()).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))?;

        FsAsset::new_without_thumb(&current_path).move_to(Path::new("unfiled"))
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))?;

        update_asset(&state.db, asset.id, UpdateAsset { parent_id: Some(None), ..Default::default() }).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))?;
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

    let dest_path = get_collection_path(&state.db, collection_id.clone()).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))?;

    let assets = get_assets(&state.db, &asset_ids).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))?;

    for asset in assets {
        let current_path = get_asset_path(&state.db, asset.id.clone()).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))?;

        FsAsset::new_without_thumb(&current_path).move_to(&dest_path)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))?;

        update_asset(&state.db, asset.id, UpdateAsset { parent_id: Some(Some(collection_id.clone())), ..Default::default() }).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))?;
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
        let current_path = get_collection_path(&state.db, collection.id.clone()).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))?;

        FsCollection::new(&current_path).move_to(Path::new(&collection.label))
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))?;

        update_collection(&state.db, collection.id, UpdateCollection { parent_id: Some(None), ..Default::default() }).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))?;
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

    let dest_path = get_collection_path(&state.db, parent_id.clone()).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))?;

    let collections = get_collections(&state.db, &collection_ids).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))?;

    for collection in collections {
        let current_path = get_collection_path(&state.db, collection.id.clone()).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))?;

        FsCollection::new(&current_path).move_to(&dest_path.join(&collection.label))
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))?;

        update_collection(&state.db, collection.id, UpdateCollection { parent_id: Some(Some(parent_id.clone())), ..Default::default() }).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))?;
    }

    Ok((StatusCode::OK, msg!("Success")))
}
