use axum::extract::{Path, Request, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect};
use serde_json::Value;
use std::path::PathBuf;
use tower::ServiceExt;
use tower_http::services::ServeFile;

use crate::db::entities::assets;
use crate::msg;
use crate::state::AppState;

/// Hash-based thumbnail serving endpoint for Axum
///
/// Looks up an asset by its `xxh3_128` content hash in the database,
/// extracts the stored `thumbnail_path` directly, and streams
/// the JPEG file back to the client with cache headers.
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
    // Fetch only thumbnail_path column from database using the content hash
    let (relative_thumb,): (Option<String>,) = assets::Entity::find()
        .filter(assets::Column::Hash.eq(&hash))
        .select_only()
        .column(assets::Column::ThumbnailPath)
        .into_tuple()
        .one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, msg!("No photo with hash {} found", hash)))?;

    let relative_thumb = relative_thumb
        .ok_or_else(|| (StatusCode::NOT_FOUND, msg!("No thumbnail generated for asset with hash {}", hash)))?;

    let thumbnail_root = std::env::var("THUMBNAIL_ROOT")
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, msg!("$THUMBNAIL_ROOT is not set")))?;

    let thumb_path = PathBuf::from(thumbnail_root).join(relative_thumb);

    // Serve the thumbnail file with caching headers
    let mut response = ServeFile::new(thumb_path).oneshot(req).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, msg!(e.to_string())))?
        .into_response();

    if response.status().is_success() {
        response.headers_mut().insert(
            axum::http::header::CACHE_CONTROL,
            axum::http::HeaderValue::from_static("public, max-age=31536000, immutable"),
        );
    }

    Ok(response)
}