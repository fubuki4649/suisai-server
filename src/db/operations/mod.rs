pub mod paths;
pub mod collection;
pub mod asset;


/// Applies partial updates to an `ActiveModel` from `Option` fields in a single block.
///
/// Each field listed is read from `$update.$field`; if it is `Some`, the corresponding
/// `$active.$field` is set via `sea_orm::Set`. Fields marked with `: convert` have
/// `.into()` called on the unwrapped value before passing it to `Set`, which is useful
/// for types that require a conversion (e.g. `chrono` types stored differently in the entity).
///
/// For nullable fields (`Option<Option<T>>`), pass `Some(None)` to explicitly clear the field.
///
/// # Usage
/// ```ignore
/// patch_fields!(active_model, update, {
///     field_a,
///     field_b: convert,  // calls .into() on the value
/// });
/// ```
#[macro_export]
macro_rules! patch_fields {
    // Base case: do nothing when the list is empty
    ($model:expr, $update:expr, {}) => {};

    // Handle a standard field
    ($model:expr, $update:expr, { $field:ident, $($tail:tt)* }) => {
        if let Some(v) = $update.$field {
            $model.$field = sea_orm::ActiveValue::Set(v.into());
        }
        patch_fields!($model, $update, { $($tail)* });
    };
}