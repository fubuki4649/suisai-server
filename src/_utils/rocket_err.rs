/// `err_to_500!` is a convenience macro for executing a block of code that returns a `Result`
/// containing a `Status`, automatically converting any error into a `Status::InternalServerError`.
///
/// ### Signature
/// `err_to_500!({ ... }) -> Status`
///
/// ### Function Block (`{...}`)
/// The block must evaluate to a `Result<Status, anyhow::Error>`.
/// On `Ok(status)`, the macro returns `status`.
/// On `Err(_)`, the macro returns `Status::InternalServerError`.
///
/// ### Example
/// ```rust
/// let status = err_to_500!({
///     // some fallible operation
///     do_something_that_might_fail()?;
///     Ok(Status::Ok)
/// });
/// ```
#[macro_export]
macro_rules! err_to_500 {
    ($logic:block) => {
        {
            let f: Box<dyn FnOnce() -> Result<Status, anyhow::Error>> = Box::new(|| $logic);
            f().unwrap_or_else(|_| Status::InternalServerError)
        }
    };
}

/// `err_to_result_500!` is a convenience macro for running fallible code that may return either a value or a `Status`.
/// If an unexpected error occurs, it automatically converts it into a `Status::InternalServerError`.
///
/// ### Signature
/// `err_to_result_500!({ ... }) -> Result<T, Status>`
///
/// ### Function Block (`{...}`)
/// The block must evaluate to a `Result<Result<T, Status>, anyhow::Error>`.
/// - On `Ok(Ok(value))`, the macro returns `Ok(value)`.
/// - On `Ok(Err(status))`, the macro returns `Err(status)`.
/// - On `Err(_)`, the macro returns `Err(Status::InternalServerError)`.
///
/// ### Example
/// ```rust
/// let result: Result<String, Status> = err_to_result_500!({
///     // some fallible operation returning Result<Result<T, Status>, anyhow::Error>
///     fetch_data().map(|res| res.map(|data| data.to_string()))
/// });
/// ```
#[macro_export]
macro_rules! err_to_result_500 {
    ($logic:block) => {
        {
            let f: Box<dyn FnOnce() -> Result<Result<_, Status>, anyhow::Error>> = Box::new(|| $logic);
            match f() {
                Ok(value) => value,
                Err(_) => Err(Status::InternalServerError),
            }
        }
    };
}