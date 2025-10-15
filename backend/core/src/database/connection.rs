use dotenvy::dotenv;
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use std::time::Duration;
use tracing::log::LevelFilter;

/// Establishes connection to the database using `DATABASE_URL` environment variable
///
/// # Errors
///
/// Returns an error if database connection fails
///
/// # Panics
///
/// Panics if `DATABASE_URL` environment variable is not set
pub async fn establish_db_connection() -> Result<DatabaseConnection, DbErr> {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let mut opt = ConnectOptions::new(database_url);
    opt.sqlx_logging(true) // Enable SQLx query logging
        .sqlx_logging_level(LevelFilter::Debug) // Set level to Debug
        .sqlx_slow_statements_logging_settings(LevelFilter::Warn, Duration::from_secs(1)); // Warn on slow queries

    Database::connect(opt).await
}
