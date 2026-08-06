use crate::db::entities::assets;
use crate::models::asset::{Asset, NewAsset, UpdateAsset};
use crate::patch_fields;
use sea_orm::{entity::{EntityTrait, ActiveModelTrait, ColumnTrait}, DatabaseConnection, DbErr, QueryFilter, Set};

/// Creates a new asset in the database
///
/// # Arguments
/// * `db` - Database connection
/// * `asset` - Asset details for creation
///
/// # Returns
/// The UUID of the newly created asset, or an error
pub async fn new_asset(db: &DatabaseConnection, asset: NewAsset) -> Result<String, DbErr> {
    let id = uuid::Uuid::now_v7().to_string();

    let active_model = assets::ActiveModel {
        id: Set(id.clone()),
        parent_id: Set(asset.parent_id),
        thumbnail_path: Set(asset.thumbnail_path),
        hash: Set(asset.hash),
        file_name: Set(asset.file_name),
        size_on_disk: Set(asset.size_on_disk),
        photo_date: Set(asset.photo_date),
        photo_timezone: Set(asset.photo_timezone),
        resolution_width: Set(asset.resolution_width),
        resolution_height: Set(asset.resolution_height),
        mime_type: Set(asset.mime_type),
        camera_model: Set(asset.camera_model),
        lens_model: Set(asset.lens_model),
        shutter_count: Set(asset.shutter_count),
        focal_length: Set(asset.focal_length),
        iso: Set(asset.iso),
        shutter_speed: Set(asset.shutter_speed),
        aperture: Set(asset.aperture),
    };

    active_model.insert(db).await?;

    Ok(id)
}

/// Deletes the specified assets from the database
///
/// # Arguments
/// * `db` - Database connection
/// * `asset_ids` - UUIDs of the assets to delete
///
/// # Returns
/// The deleted `Asset`s that were found and removed, or an error
pub async fn delete_asset(db: &DatabaseConnection, asset_ids: Vec<String>) -> Result<Vec<Asset>, DbErr> {
    if asset_ids.is_empty() {
        return Ok(vec![]);
    }

    // Fetch
    let assets = assets::Entity::find()
        .filter(assets::Column::Id.is_in(asset_ids.clone()))
        .all(db)
        .await?;

    // Delete
    assets::Entity::delete_many()
        .filter(assets::Column::Id.is_in(asset_ids))
        .exec(db)
        .await?;

    Ok(assets.into_iter().map(Into::into).collect())
}

/// Updates an existing asset in the database using partial update (only set fields are changed)
///
/// # Arguments
/// * `db` - Database connection
/// * `asset_id` - UUID of the asset to update
/// * `update` - Fields to update; only `Some` fields are applied
///
/// # Returns
/// The updated `Asset`, or an error if the asset doesn't exist
pub async fn update_asset(db: &DatabaseConnection, asset_id: String, update: UpdateAsset) -> Result<Asset, DbErr> {
    let existing = assets::Entity::find_by_id(asset_id.clone())
        .one(db)
        .await?
        .ok_or_else(|| DbErr::RecordNotFound(format!("Asset {asset_id} not found")))?;

    let mut active_model: assets::ActiveModel = existing.into();

    patch_fields!(active_model, update, {
        parent_id,
        thumbnail_path,
        file_name,
        size_on_disk,
        photo_date,
        photo_timezone,
        resolution_width,
        resolution_height,
        mime_type,
        camera_model,
        lens_model,
        shutter_count,
        focal_length,
        iso,
        shutter_speed,
        aperture,
    });

    let updated = active_model.update(db).await?;

    Ok(updated.into())
}


/// Checks if a hash exists in the database
///
/// # Arguments
/// * `db` - Database connection
/// * `incoming_hash` - UUID of the asset
///
/// # Returns
/// `Ok(None)` if `incoming_hash` doesn't already exist; `Ok(Some(asset))` with the matching asset
/// if `incoming_hash` exits; `Err(DbErr)` otherwise
pub async fn check_hash(db: &DatabaseConnection, incoming_hash: &str) -> Result<Option<Asset>, DbErr> {
    let hash: Option<Asset> = assets::Entity::find()
        .filter(assets::Column::Hash.eq(incoming_hash))
        .into_partial_model::<Asset>()
        .one(db)
        .await?;

    Ok(hash)
}


/// Gets assets by their IDs
///
/// # Arguments
/// * `db` - Database connection
/// * `asset_ids` - Slice of asset UUIDs to retrieve
///
/// # Returns
/// Vector of assets matching the provided IDs, or an error if query fails
pub async fn get_assets(db: &DatabaseConnection, asset_ids: &[String]) -> Result<Vec<Asset>, DbErr> {
    if asset_ids.is_empty() { return Ok(vec![]); }

    assets::Entity::find()
        .filter(assets::Column::Id.is_in(asset_ids.to_vec()))
        .into_partial_model::<Asset>()
        .all(db)
        .await
}

/// Gets assets by parent collection ID, or unfiled assets if `parent_id` is `None`
///
/// # Arguments
/// * `db` - Database connection
/// * `parent_id` - UUID of the parent collection, or `None` to get unfiled assets
///
/// # Returns
/// A list of matching assets, or an error
pub async fn get_assets_by_parent(db: &DatabaseConnection, parent_id: Option<String>) -> Result<Vec<Asset>, DbErr> {
    let query = assets::Entity::find();
    let query = match parent_id {
        Some(id) => query.filter(assets::Column::ParentId.eq(id)),
        None => query.filter(assets::Column::ParentId.is_null()),
    };
    query.into_partial_model::<Asset>().all(db).await
}
