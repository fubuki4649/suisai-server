use crate::db::entities::assets;
use chrono::DateTime;
use chrono::Utc;

/// Represents an asset with associated metadata stored in the database.
///
/// # Fields
/// - `id` (`String`): UUIDv7. Serialized as "assetId" in JSON
/// - `parent_id` (`Option<String>`): Optional parent collection ID
/// - `thumbnail_path` (`Option<String>`): Path to the generated thumbnail, if any
/// - `hash` (`String`): xxh3-128 hash value of the asset, used to ensure uniqueness
/// - `file_name` (`String`): Original filename of the asset
/// - `size_on_disk` (`i64`): Size of the asset on disk in KB
/// - `photo_date` (`DateTime<Utc>`): When the photo was taken
/// - `photo_timezone` (`String`): Timezone info for the photo timestamp
/// - `resolution_width` (`i64`): Asset width in pixels
/// - `resolution_height` (`i64`): Asset height in pixels
/// - `mime_type` (`String`): Media type of the asset (e.g. "image/x-sony-arw")
/// - `camera_model` (`String`): Make and model of the camera used
/// - `lens_model` (`String`): Make and model of the lens used
/// - `shutter_count` (`i64`): Camera's actuation count when photo was taken
/// - `focal_length` (`i16`): Focal length used in millimeters
/// - `iso` (`i64`): ISO sensitivity value
/// - `shutter_speed` (`String`): Exposure time as a string (e.g. "1/250")
/// - `aperture` (`f32`): F-stop value used
#[derive(Debug, sea_orm::prelude::DerivePartialModel)]
#[sea_orm(entity = "assets::Entity")]
pub struct Asset {
    pub id: String,
    pub parent_id: Option<String>,
    pub thumbnail_path: Option<String>,
    pub hash: String,
    pub file_name: String,
    pub size_on_disk: i64,
    pub photo_date: DateTime<Utc>,
    pub photo_timezone: String,
    pub resolution_width: i64,
    pub resolution_height: i64,
    pub mime_type: String,
    pub camera_model: String,
    pub lens_model: String,
    pub shutter_count: i64,
    pub focal_length: i16,
    pub iso: i64,
    pub shutter_speed: String,
    pub aperture: f32,
}

impl From<assets::Model> for Asset {
    fn from(model: assets::Model) -> Self {
        Asset {
            id: model.id,
            parent_id: model.parent_id,
            thumbnail_path: model.thumbnail_path,
            hash: model.hash,
            file_name: model.file_name,
            size_on_disk: model.size_on_disk,
            photo_date: model.photo_date.into(),
            photo_timezone: model.photo_timezone,
            resolution_width: model.resolution_width,
            resolution_height: model.resolution_height,
            mime_type: model.mime_type,
            camera_model: model.camera_model,
            lens_model: model.lens_model,
            shutter_count: model.shutter_count,
            focal_length: model.focal_length,
            iso: model.iso,
            shutter_speed: model.shutter_speed,
            aperture: model.aperture,
        }
    }
}

/// A variant of `Asset` without an `id` field, used for creating new asset instances.
///
/// IDs are auto-generated as UUIDv7 on insert.
///
/// # Fields
/// - `parent_id` (`Option<String>`): Optional parent collection ID
/// - `hash` (`String`): xxh3-128 hash value of the asset, used to ensure uniqueness
/// - `file_name` (`String`): Original filename of the asset
/// - `size_on_disk` (`i64`): Size of the asset on disk in Bytes
/// - `photo_date` (`DateTime<Utc>`): When the photo was taken
/// - `photo_timezone` (`String`): Timezone info for the photo timestamp
/// - `resolution_width` (`i64`): Asset width in pixels
/// - `resolution_height` (`i64`): Asset height in pixels
/// - `mime_type` (`String`): Media type of the asset (e.g. "image/x-sony-arw")
/// - `camera_model` (`String`): Make and model of the camera used
/// - `lens_model` (`String`): Make and model of the lens used
/// - `shutter_count` (`i64`): Camera's actuation count when photo was taken
/// - `focal_length` (`i16`): Focal length used in millimeters
/// - `iso` (`i64`): ISO sensitivity value
/// - `shutter_speed` (`String`): Exposure time as a string (e.g. "1/250")
/// - `aperture` (`f32`): F-stop value used
#[derive(Debug)]
pub struct NewAsset {
    pub parent_id: Option<String>,
    pub hash: String,
    pub file_name: String,
    pub size_on_disk: i64,
    pub photo_date: DateTime<Utc>,
    pub photo_timezone: String,
    pub resolution_width: i64,
    pub resolution_height: i64,
    pub mime_type: String,
    pub camera_model: String,
    pub lens_model: String,
    pub shutter_count: i64,
    pub focal_length: i16,
    pub iso: i64,
    pub shutter_speed: String,
    pub aperture: f32,
}

/// An update payload for an asset, with all fields optional to support partial updates.
///
/// # Fields
/// - `parent_id` (`Option<Option<String>>`): Optional new parent collection ID (inner `None` clears the field)
/// - `thumbnail_path` (`Option<Option<String>>`): Optional new thumbnail path (inner `None` clears the field)
/// - `file_name` (`Option<String>`): Optional new filename
/// - `size_on_disk` (`Option<i64>`): Optional new size on disk in KB
/// - `photo_date` (`Option<DateTime<Utc>>`): Optional new photo date
/// - `photo_timezone` (`Option<String>`): Optional new timezone
/// - `resolution_width` (`Option<i64>`): Optional new width in pixels
/// - `resolution_height` (`Option<i64>`): Optional new height in pixels
/// - `mime_type` (`Option<String>`): Optional new MIME type
/// - `camera_model` (`Option<String>`): Optional new camera model
/// - `lens_model` (`Option<String>`): Optional new lens model
/// - `shutter_count` (`Option<i64>`): Optional new shutter count
/// - `focal_length` (`Option<i16>`): Optional new focal length in millimeters
/// - `iso` (`Option<i64>`): Optional new ISO value
/// - `shutter_speed` (`Option<String>`): Optional new shutter speed string
/// - `aperture` (`Option<f32>`): Optional new aperture F-stop value
#[derive(Debug, Default)]
pub struct UpdateAsset {
    pub parent_id: Option<Option<String>>,
    pub thumbnail_path: Option<Option<String>>,
    pub file_name: Option<String>,
    pub size_on_disk: Option<i64>,
    pub photo_date: Option<DateTime<Utc>>,
    pub photo_timezone: Option<String>,
    pub resolution_width: Option<i64>,
    pub resolution_height: Option<i64>,
    pub mime_type: Option<String>,
    pub camera_model: Option<String>,
    pub lens_model: Option<String>,
    pub shutter_count: Option<i64>,
    pub focal_length: Option<i16>,
    pub iso: Option<i64>,
    pub shutter_speed: Option<String>,
    pub aperture: Option<f32>,
}
