#[macro_export]
macro_rules! err_to_500 {
    ($logic:block) => {
        {
            let f: Box<dyn FnOnce() -> Result<Status>> = Box::new(|| $logic);
            f().unwrap_or_else(|_| Status::InternalServerError)
        }
    };
}