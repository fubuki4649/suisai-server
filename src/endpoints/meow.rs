use rocket::get;
use rocket::http::Status;

/// Simple health check endpoint to verify API is responding
///
/// # Endpoint
/// `GET /meow`
///
/// # Returns
/// - `418 I'm a teapot`: API is up and running
#[get("/meow")]
pub fn health_check() -> (Status, &'static str) {
    (Status::ImATeapot, "/ᐠ ˵> ⩊ <˵ マ\n")
}