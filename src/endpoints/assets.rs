use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde_json::Value;
use crate::_utils::json_map::JsonMap;
use crate::db::operations::asset::{delete_asset, get_assets};
use crate::models::asset::Asset;
use crate::{msg, state::AppState};


type Response = Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)>;

/// Delete multiple assets from the database by their IDs
///
/// Also removes associated thumbnail files from the filesystem; missing files are silently ignored.
/// Note: raw asset files are not tracked by path in the DB — only thumbnails are cleaned up here.
///
/// # Route
/// `DELETE /asset/delete`
///
/// # Request Body
/// JSON object with:
/// - `assetIds`: array of asset UUID strings to delete
///
/// # Returns
/// - `200 OK` on success (even if some IDs didn't exist)
/// - `400 Bad Request` if `assetIds` is missing or malformed
/// - `500 Internal Server Error` if deletion fails
pub async fn del_asset(State(state): State<AppState>, input: Json<Value>) -> Response {
    let asset_ids = input.get_value::<Vec<String>>("asset_ids")
        .map_err(|e| (StatusCode::BAD_REQUEST, msg!(e.to_string())))?;

    let deleted = delete_asset(&state.db, asset_ids).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))?;

    // Clean up thumbnails from the filesystem, ignoring missing/permission errors
    deleted.iter()
        .filter_map(|a| a.thumbnail_path.as_deref())
        .for_each(|path| { let _ = std::fs::remove_file(path); });

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
pub async fn get_assets_handler(State(state): State<AppState>, input: Json<Value>) -> Result<Json<Vec<Asset>>, (StatusCode, Json<Value>)> {
    let asset_ids = input.get_value::<Vec<String>>("asset_ids")
        .map_err(|e| (StatusCode::BAD_REQUEST, msg!(e.to_string())))?;

    get_assets(&state.db, &asset_ids).await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))
}
