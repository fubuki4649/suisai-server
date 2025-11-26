/// Creates a JSON response with a custom key-value pair.
///
/// This macro expands to `Json<serde_json::Value>` containing a single key-value pair.
///
/// # Forms
/// - `msg!($msg)` - Creates `{"message": $msg}`
/// - `msg!($key, $msg)` - Creates `{$key: $msg}`
/// - `msg!($fmt, $($args:tt)*)` - Creates `{"message": format!($fmt, $($args)*)}`
/// - `msg!($key, $fmt, $($args:tt)*)` - Creates `{$key: format!($fmt, $($args)*)}`
///
/// # Returns
/// `rocket::serde::json::Json<serde_json::Value>`
///
/// # Examples
/// ```
/// use rocket::http::Status;
/// use rocket::serde::json::Json;
///
/// // Simple message (uses "message" as key)
/// let response: Json<serde_json::Value> = msg!("Operation successful");
/// // Expands to: Json(json!({"message": "Operation successful"}))
///
/// // Custom key
/// let response = msg!("error", "Invalid input");
/// // Expands to: Json(json!({"error": "Invalid input"}))
///
/// // Format string with default key
/// let user_id = 42;
/// let response = msg!("User {} created", user_id);
/// // Expands to: Json(json!({"message": "User 42 created"}))
///
/// // Format string with custom key
/// let count = 5;
/// let response = msg!("info", "Found {} items", count);
/// // Expands to: Json(json!({"info": "Found 5 items"}))
///
/// // In a handler
/// fn handler(id: i32) -> (Status, Json<serde_json::Value>) {
///     (Status::Ok, msg!("User {} created successfully", id))
/// }
/// ```
#[macro_export]
macro_rules! msg {
    // Single expression with default "message" key
    ($msg:expr) => {
        rocket::serde::json::Json(serde_json::json!({
            "message": $msg
        }))
    };
    // Format string with default "message" key
    ($fmt:expr, $($args:tt)+) => {
        rocket::serde::json::Json(serde_json::json!({
            "message": format!($fmt, $($args)+)
        }))
    };
    // Custom key with single expression (must use trailing comma to disambiguate)
    ($key:expr, $msg:expr,) => {
        rocket::serde::json::Json(serde_json::json!({
            $key: $msg
        }))
    };
}