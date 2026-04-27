use crate::config::secret::read_secret;
use dotenvy::dotenv;
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use std::time::Duration;
use tracing::log::LevelFilter;

/// Establishes connection to the database.
///
/// Reads `DATABASE_URL` directly, or composes it from `DB_USER`, `DB_NAME`, `DB_HOST`, `DB_PORT`,
/// and `DB_PASSWORD` / `DB_PASSWORD_FILE` (Docker Secrets pattern).
///
/// # Errors
///
/// Returns an error if database connection fails
///
/// # Panics
///
/// Panics if neither `DATABASE_URL` nor the required `DB_*` components are set.
pub async fn establish_db_connection() -> Result<DatabaseConnection, DbErr> {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        let user = std::env::var("DB_USER").expect("DB_USER or DATABASE_URL must be set");
        let password = read_secret("DB_PASSWORD");
        let host = std::env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string());
        let port = std::env::var("DB_PORT").unwrap_or_else(|_| "5432".to_string());
        let name = std::env::var("DB_NAME").expect("DB_NAME must be set");
        format!("postgresql://{user}:{password}@{host}:{port}/{name}")
    });

    let mut opt = ConnectOptions::new(database_url);
    opt.sqlx_logging(true) // Enable SQLx query logging
        .sqlx_logging_level(LevelFilter::Debug) // Set level to Debug
        .sqlx_slow_statements_logging_settings(LevelFilter::Warn, Duration::from_secs(1)); // Warn on slow queries

    Database::connect(opt).await
}
