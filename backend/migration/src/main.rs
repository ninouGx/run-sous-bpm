use sea_orm_migration::prelude::*;

#[tokio::main]
async fn main() {
    // When DATABASE_URL is not set (prod Docker Secrets mode), compose it from components.
    if std::env::var("DATABASE_URL").is_err() {
        let user = std::env::var("DB_USER").expect("DB_USER or DATABASE_URL must be set");
        let password = read_secret("DB_PASSWORD");
        let host = std::env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string());
        let port = std::env::var("DB_PORT").unwrap_or_else(|_| "5432".to_string());
        let db = std::env::var("DB_NAME").expect("DB_NAME must be set");
        std::env::set_var(
            "DATABASE_URL",
            format!("postgresql://{user}:{password}@{host}:{port}/{db}"),
        );
    }
    cli::run_cli(migration::Migrator).await;
}

fn read_secret(var: &str) -> String {
    if let Ok(path) = std::env::var(format!("{var}_FILE")) {
        std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read {var}_FILE at {path}: {e}"))
            .trim_end()
            .to_string()
    } else {
        std::env::var(var).unwrap_or_else(|_| panic!("Either {var} or {var}_FILE must be set"))
    }
}
