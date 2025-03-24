/// Runs code, propagating all errors to return value Status::InternalServerError
/// 
/// Use this macro if you don't need to return any data besides the HTTP status code.
/// If you need to return other data, `err_to_result_500!()` is available
/// 
/// `() -> Status`
#[macro_export]
macro_rules! err_to_500 {
    ($logic:block) => {
        {
            let f: Box<dyn FnOnce() -> Result<Status, anyhow::Error>> = Box::new(|| $logic);
            f().unwrap_or_else(|_| Status::InternalServerError)
        }
    };
}

/// Runs code, propagating all errors to return type Err(Status::InternalServerError)
/// 
/// Use this macro if you also want to return an object that isn't rocket::http::Status. 
/// Otherwise, `err_to_500!()` is available.
/// 
/// `() -> Result<_, Status>
#[macro_export]
macro_rules! err_to_result_500 {
    ($logic:block) => {
        {
            let f: Box<dyn FnOnce() -> Result<_, anyhow::Error>> = Box::new(|| $logic);
            match f() {
                Ok(v) => Ok(v),
                Err(_) => Err(Status::InternalServerError),
            }
        }
    };
}