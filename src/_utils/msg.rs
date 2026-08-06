/// Creates an `axum::Json` response with a message body.
///
/// This macro expands to `axum::Json<serde_json::Value>` containing a single key-value pair.
///
/// # Forms
/// - `msg!($msg)` — Creates `{"message": $msg}`
/// - `msg!($fmt, $($args:tt)+)` — Creates `{"message": format!($fmt, $($args)+)}`
/// - `msg!($key, $msg,)` — Creates `{$key: $msg}` (note: trailing comma required to disambiguate from the format form)
///
/// # Returns
/// `axum::Json<serde_json::Value>`
///
/// # Examples
/// ```
/// use axum::http::StatusCode;
/// use axum::Json;
///
/// // Simple message (uses "message" as key)
/// let response: Json<serde_json::Value> = msg!("Operation successful");
/// // Expands to: Json(json!({"message": "Operation successful"}))
///
/// // Format string with default key
/// let user_id = 42;
/// let response = msg!("User {} created", user_id);
/// // Expands to: Json(json!({"message": "User 42 created"}))
///
/// // Custom key (trailing comma required)
/// let response = msg!("error", "Invalid input",);
/// // Expands to: Json(json!({"error": "Invalid input"}))
///
/// // In a handler
/// fn handler(id: i32) -> (StatusCode, Json<serde_json::Value>) {
///     (StatusCode::OK, msg!("User {} created successfully", id))
/// }
/// ```
#[macro_export]
macro_rules! msg {
    // Single expression with default "message" key
    ($msg:expr) => {
        axum::Json(serde_json::json!({
            "message": $msg
        }))
    };
    // Format string with default "message" key
    ($fmt:expr, $($args:tt)+) => {
        axum::Json(serde_json::json!({
            "message": format!($fmt, $($args)+)
        }))
    };
    // Custom key with single expression (trailing comma required to disambiguate from format arm)
    ($key:expr, $msg:expr,) => {
        axum::Json(serde_json::json!({
            $key: $msg
        }))
    };
}