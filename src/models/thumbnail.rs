use crate::db::schema::thumbnails;
use diesel::{AsChangeset, Queryable, Selectable};
use serde::{Deserialize, Serialize};

/// Represents a thumbnail, with the photo ID it's associated with and the path to the thumbnail on disk.
///
/// # Fields
/// * `id`: Album's unique ID, serialized as `albumId` in JSON
/// * `thumbnail_path`: Location of the thumbnail on disk
///
/// # Example
/// ```
/// let album = Album {
///     id: 1,
///     directory: "/home/user/.thumbnails/202506/IMG_001.JPG".into(),
/// };
/// ```
#[derive(Queryable, Selectable, AsChangeset, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Thumbnail {
    #[serde(rename = "photoId")]
    pub id: i64,
    pub thumbnail_path: String
}