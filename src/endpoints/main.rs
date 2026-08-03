use crate::endpoints::assets::{del_asset, get_assets_handler};
use crate::endpoints::meow::health_check;
use crate::endpoints::thumbnail::get_thumbnail;
use crate::preflight::check_directories;
use crate::state::AppState;
use axum::{routing::get, Router};
use tower_http::cors::CorsLayer;

/// Initializes and launches the Axum HTTP web server.
///
/// Runs system preflight checks, mounts all API route handlers onto the Axum router,
/// attaches the global `AppState` (containing database pools and handles), and binds
/// to port `8000`.
///
/// # Arguments
/// * `state` - The application state instance containing SeaORM database connection
pub async fn start_webserver(state: AppState) {
    // Run preflight checks (directory validation & setup)
    check_directories().unwrap();

    // Build the Axum router and attach routes & shared AppState
    let app = Router::new()
        .route("/meow", get(health_check))
        .route("/thumbnail/{hash}", get(get_thumbnail))
        .route("/asset/get", axum::routing::post(get_assets_handler))
        .route("/asset/delete", axum::routing::delete(del_asset))
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Bind TCP listener on port 8000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000")
        .await
        .expect("Failed to bind to address 0.0.0.0:8000");

    println!("Server running on http://0.0.0.0:8000");

    // Start serving requests
    axum::serve(listener, app)
        .await
        .expect("Failed to launch server");
}