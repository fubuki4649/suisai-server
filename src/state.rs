use sea_orm::DatabaseConnection;

#[derive(Clone)] // Axum requires State to be Clone
pub struct AppState {
    pub db: DatabaseConnection,
    // Add other "global-level" things here later: s3_client, mailer, etc.
}