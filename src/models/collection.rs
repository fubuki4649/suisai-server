use crate::db::entities::collections;

/// Represents a collection with a unique identifier and label.
///
/// # Fields
/// * `id`: Collection's unique UUIDv7
/// * `label`: Collection label
/// * `parent_id`: Optional parent collection ID
#[derive(Debug, sea_orm::prelude::DerivePartialModel)]
#[sea_orm(entity = "collections::Entity")]
pub struct Collection {
    pub id: String,
    pub label: String,
    pub parent_id: Option<String>,
}

impl From<collections::Model> for Collection {
    fn from(model: collections::Model) -> Self {
        Collection {
            id: model.id,
            label: model.label,
            parent_id: model.parent_id,
        }
    }
}

/// A variant of `Collection` without an ID, used for creating new collection instances.
///
/// This struct only contains the label and optional parent since IDs are auto-generated.
///
/// # Fields
/// * `label`: The label for the new collection
/// * `parent_id`: Optional parent collection ID
#[derive(Debug)]
pub struct NewCollection {
    pub label: String,
    pub parent_id: Option<String>,
}

/// An update payload for a collection, with all fields optional to support partial updates.
///
/// # Fields
/// * `label`: Optional new label for the collection
/// * `parent_id`: Optional new parent collection ID
#[derive(Debug, Default)]
pub struct UpdateCollection {
    pub label: Option<String>,
    pub parent_id: Option<Option<String>>,
}
