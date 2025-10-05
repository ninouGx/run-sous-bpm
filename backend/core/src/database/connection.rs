use sea_orm::{ Database, DbErr, DatabaseConnection };
use dotenv::dotenv;

pub async fn establish_db_connection() -> Result<DatabaseConnection, DbErr> {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    //let _db_name = std::env::var("DB_NAME").expect("DB_NAME must be set");
    Database::connect(&database_url).await
}
