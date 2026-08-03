/// Attempts to unwrap a `Result`, returning from the parent function if it is an `Err`.
///
/// This macro is useful for early returns in functions that do not return a `Result`
/// themselves but need to short-circuit on error with an HTTP status code.
///
/// # Parameters
/// - `$expr`: A `Result<T, E>` expression to unwrap.
/// - `$err_return`: The `axum::http::StatusCode` to return from the function if `Err`.
///
/// # Returns
/// - `T` if `Ok`
/// - Triggers an early return from the parent function with `(StatusCode, Json<Value>)` if `Err`
///
/// # Example
/// ```
/// let val = unwrap_ret!(some_result, StatusCode::BAD_REQUEST);
/// ```
#[macro_export]
macro_rules! unwrap_ret {
    ($expr:expr, $err_return:expr) => {
        match $expr {
            Ok(val) => val,
            Err(err) => return ($err_return, msg!(err.to_string())),
        }
    };
}

/// Same as `unwrap_ret`, but wraps the result in `Err(...)`.
/// Attempts to unwrap a `Result`, returning `Err((StatusCode, Json<Value>))` if `Err`.
///
/// # Parameters
/// - `$expr`: A `Result<T, E>` expression to unwrap.
/// - `$err_return`: The `axum::http::StatusCode` to return from the function if `Err`.
///
/// # Returns
/// - `T` if `Ok`
/// - Triggers an early return from the parent function with `Err((StatusCode, Json<Value>))` if `Err`
///
/// # Example
/// ```
/// let val = unwrap_err!(some_result, StatusCode::BAD_REQUEST);
/// ```
#[macro_export]
macro_rules! unwrap_err {
    ($expr:expr, $err_return:expr) => {
        match $expr {
            Ok(val) => val,
            Err(err) => return Err(($err_return, msg!(err.to_string()))),
        }
    };
}
