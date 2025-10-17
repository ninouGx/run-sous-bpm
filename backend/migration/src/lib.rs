pub use sea_orm_migration::prelude::*;

mod m20251005_175514_create_table_oauth_token;
mod m20251005_175525_create_table_user;
mod m20251014_110258_add_password_to_user;
mod m20251015_112925_create_table_activities;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20251005_175514_create_table_oauth_token::Migration),
            Box::new(m20251005_175525_create_table_user::Migration),
            Box::new(m20251014_110258_add_password_to_user::Migration),
            Box::new(m20251015_112925_create_table_activities::Migration),
        ]
    }
}
