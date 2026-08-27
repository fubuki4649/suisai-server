use axum::http::StatusCode;

/// Simple health check endpoint to verify API is responding
///
/// # Endpoint
/// `GET /meow`
///
/// # Returns
/// - `418 I'm a teapot`: API is up and running
pub async fn health_check() -> (StatusCode, &'static str) {
    (StatusCode::IM_A_TEAPOT, "/ᐠ ˵> ⩊ <˵ マ\n")
}