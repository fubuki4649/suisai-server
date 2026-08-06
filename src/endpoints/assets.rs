use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde_json::Value;
use std::path::Path;

use crate::_utils::json_map::JsonMap;
use crate::db::operations::asset::{delete_asset, get_assets as db_get_assets};
use crate::db::operations::paths::get_collection_path;
use crate::fs_operations::asset::Asset as FsAsset;
use crate::models::asset::Asset;
use crate::{msg, state::AppState};

type Response = Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)>;

/// Delete multiple assets from the database by their IDs
///
/// Removes the asset file (and any associated sidecar/export files with the same stem),
/// the thumbnail, and cleans up empty thumbnail directories.
///
/// # Route
/// `DELETE /asset/delete`
///
/// # Request Body
/// JSON object with:
/// - `assetIds`: array of asset UUID strings to delete
///
/// # Returns
/// - `200 OK` on success
/// - `400 Bad Request` if `assetIds` is missing or malformed
/// - `500 Internal Server Error` if the database deletion or any filesystem operation fails
pub async fn del_asset(State(state): State<AppState>, input: Json<Value>) -> Response {
    let asset_ids = input.get_value::<Vec<String>>("asset_ids")
        .map_err(|e| (StatusCode::BAD_REQUEST, msg!(e.to_string())))?;

    let deleted = delete_asset(&state.db, asset_ids).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))?;

    // Delete each asset and its associated files from disk
    for asset in deleted {
        // Get the full path to the asset, so we can delete it from the disk
        let mut asset_path = match asset.parent_id {
            Some(parent_id) => get_collection_path(&state.db, parent_id).await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))?,
            None => std::path::PathBuf::new(),
        };
        asset_path.push(&asset.file_name);

        match asset.thumbnail_path {
            Some(ref thumb) => FsAsset::new(&asset_path, Path::new(thumb)).delete(),
            None => FsAsset::new_without_thumb(&asset_path).delete(),
        }.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))?;
    }

    Ok((StatusCode::OK, msg!("Success")))
}

/// Retrieve multiple assets by their IDs
///
/// # Route
/// `POST /asset/get`
///
/// # Request Body
/// JSON object with:
/// - `assetIds`: array of asset UUID strings to retrieve
///
/// # Returns
/// - `200 OK` with a JSON array of matching `Asset` objects (skips IDs that don't exist)
/// - `400 Bad Request` if `assetIds` is missing or malformed
/// - `500 Internal Server Error` if the query fails
pub async fn get_assets(State(state): State<AppState>, input: Json<Value>) -> Result<Json<Vec<Asset>>, (StatusCode, Json<Value>)> {
    let asset_ids = input.get_value::<Vec<String>>("asset_ids")
        .map_err(|e| (StatusCode::BAD_REQUEST, msg!(e.to_string())))?;

    db_get_assets(&state.db, &asset_ids).await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))
}
