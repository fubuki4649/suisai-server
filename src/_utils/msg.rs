/// Creates a JSON response with a custom key-value pair.
///
/// This macro expands to `Json<serde_json::Value>` containing a single key-value pair.
///
/// # Forms
/// - `msg!($msg)` - Creates `{"message": $msg}`
/// - `msg!($key, $msg)` - Creates `{$key: $msg}`
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
/// // In a handler
/// fn handler() -> (Status, Json<serde_json::Value>) {
///     (Status::Ok, msg!("User created successfully"))
/// }
/// ```
#[macro_export]
macro_rules! msg {
    ($msg:expr) => {
        rocket::serde::json::Json(serde_json::json!({
            "message": $msg
        }))
    };
    ($key:expr, $msg:expr) => {
        rocket::serde::json::Json(serde_json::json!({
            $key: $msg
        }))
    };
}