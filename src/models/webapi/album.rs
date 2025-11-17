use crate::models::db::album::Album as DbAlbum;
use serde::Serialize;

/// The version of the Album struct that is returned by the API with certain fields removed
/// (currently, directory)
///
/// # Fields
/// * `id`: Album's unique ID, serialized as `albumId` in JSON
/// * `album_name`: Album name
///
/// # Example
/// ```
/// let album = Album {
///     id: 1,
///     album_name: "Vacation".into(),
/// };
/// ```
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Album {
    #[serde(rename = "albumId")]
    pub id: i32,
    pub album_name: String
}

impl From<DbAlbum> for Album {
    fn from(value: DbAlbum) -> Self {
        Album {
            id: value.id,
            album_name: value.album_name,
        }
    }
}