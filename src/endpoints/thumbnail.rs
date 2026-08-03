use axum::extract::{Path, Request, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::Value;
use tower::ServiceExt;
use tower_http::services::ServeFile;

use crate::db::operations::asset::check_hash;
use crate::msg;
use crate::state::AppState;

/// Hash-based thumbnail serving endpoint for Axum
///
/// Looks up an asset by its `xxh3_128` content hash in the database,
/// extracts the stored `thumbnail_path` from the `Asset` model, and streams
/// the JPEG file back to the client.
///
/// # Route
/// `GET /thumbnail/{hash}`
///
/// # URL Parameters
/// - `hash`: The 32-character hexadecimal xxh3 content hash of the asset
///
/// # Returns
/// - `200 OK`: The thumbnail image file (JPEG)
/// - `404 Not Found`: No asset matches the provided hash, or thumbnail has not been generated
/// - `500 Internal Server Error`: Database query error or file reading failure
pub async fn get_thumbnail(Path(hash): Path<String>, State(state): State<AppState>, req: Request) -> Result<Response, (StatusCode, Json<Value>)> {
    // Fetch asset from database using the content hash
    let asset = check_hash(&state.db, &hash).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, msg!("No photo with hash {} found", hash)))?;

    // Extract integrated thumbnail_path from asset struct
    let thumb_path = asset.thumbnail_path
        .ok_or_else(|| (StatusCode::NOT_FOUND, msg!("No thumbnail generated for asset with hash {}", hash)))?;

    // Serve the thumbnail file
    Ok(ServeFile::new(thumb_path).oneshot(req).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))?
        .into_response())
}