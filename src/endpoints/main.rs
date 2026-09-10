use crate::endpoints::assets::{del_asset, get_assets};
use crate::endpoints::collection::{assets_in_collection, del_collection, get_collection_flat, get_collection_tree, new_collection_handler, rename_collection, unfiled_assets};
use crate::endpoints::management::{reassign_asset, reassign_collection, unfile_asset, unfile_collection};
use crate::endpoints::meow::health_check;
use crate::endpoints::thumbnail::get_thumbnail;
use crate::state::AppState;
use axum::routing::{delete, get, patch, post};
use axum::Router;
use tower_http::cors::CorsLayer;

/// Initializes and launches the Axum HTTP web server.
///
/// Runs system preflight checks, mounts all API route handlers onto the Axum router,
/// attaches the global `AppState` (containing database pools and handles), and binds
/// to port `8000`.
///
/// # Arguments
/// * `state` - The application state instance containing `SeaORM` database connection
pub async fn start_webserver(state: AppState) {
    
    // Build the Axum router and attach routes & shared AppState
    let app = Router::new()
        .route("/meow", get(health_check))
        .route("/thumbnail/{hash}", get(get_thumbnail))
        .route("/asset/get", post(get_assets))
        .route("/asset/delete", delete(del_asset))
        .route("/collection/tree", get(get_collection_tree))
        .route("/collection/flat", get(get_collection_flat))
        .route("/collection/new", post(new_collection_handler))
        .route("/collection/unfiled/assets", get(unfiled_assets))
        .route("/collection/{id}/rename", patch(rename_collection))
        .route("/collection/{id}/delete", delete(del_collection))
        .route("/collection/{id}/assets", get(assets_in_collection))
        .route("/management/asset/unfile", post(unfile_asset))
        .route("/management/asset/reassign", post(reassign_asset))
        .route("/management/collection/unfile", post(unfile_collection))
        .route("/management/collection/reassign", post(reassign_collection))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "8000".to_string());
    let addr = format!("0.0.0.0:{}", port);

    // Bind TCP listener
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|_| panic!("Failed to bind to address {}", addr));

    println!("Server running on http://{}", addr);

    // Start serving requests
    axum::serve(listener, app)
        .await
        .expect("Failed to launch server");
}