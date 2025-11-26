/// Attempts to unwrap a `Result`, returning from the parent function if it is an `Err`.
///
/// This macro is useful for early returns in functions that do not return a `Result`
/// themselves but need to short-circuit on error.
///
/// # Parameters
/// - `$expr`: A `Result<T, E>` expression to unwrap.
/// - `$err_return`: The value to return from the function if the result is `Err`.
///
/// # Returns
/// - `T` if `Ok`
/// - Triggers a return from the parent function with `(E, String)` if `Err`, where `String` is an error message
///
/// # Example
/// ```
/// let val = unwrap_ret!(some_result, Status::BadRequest);
/// ```
#[macro_export]
macro_rules! unwrap_ret {
    ($expr:expr, $err_return:expr) => {
        match $expr {
            Ok(val) => val,
            Err(err) => return ($err_return, msg!(err.to_string()).into()),
        }
    };
}

/// Same as `unwrap_ret`, but wraps the result in an `Err`.
/// Attempts to unwrap a `Result`, returning from the parent function if it is an `Err`.
///
/// This macro is useful for early returns in functions that do not return a `Result`
/// themselves but need to short-circuit on error.
///
/// # Parameters
/// - `$expr`: A `Result<T, E>` expression to unwrap.
/// - `$err_return`: The value to return from the function if the result is `Err`.
///
/// # Returns
/// - `T` if `Ok`
/// - Triggers a return from the parent function with `Err(E, String)` if `Err`, where `String` is an error message
///
/// # Example
/// ```
/// let val = unwrap_err!(some_result, Status::BadRequest);
/// ```
#[macro_export]
macro_rules! unwrap_err {
    ($expr:expr, $err_return:expr) => {
        match $expr {
            Ok(val) => val,
            Err(err) => return Err(($err_return, msg!(err.to_string()).into())),
        }
    };
}
