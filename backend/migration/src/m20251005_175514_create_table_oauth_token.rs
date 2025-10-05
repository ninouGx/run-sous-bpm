use sea_orm_migration::{ prelude::*, schema::* };

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(
            Table::create()
                .table(OauthToken::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(OauthToken::Id)
                        .uuid()
                        .not_null()
                        .primary_key()
                        .default(Expr::cust("gen_random_uuid()"))
                )
                .col(ColumnDef::new(OauthToken::UserId).uuid().not_null())
                .col(
                    ColumnDef::new(OauthToken::Provider)
                        .text()
                        .not_null()
                        .check(
                            Expr::col(OauthToken::Provider).is_in(
                                vec!["strava", "spotify", "lastfm"]
                            )
                        )
                )
                .col(ColumnDef::new(OauthToken::AccessToken).text().not_null())
                .col(ColumnDef::new(OauthToken::RefreshToken).text())
                .col(ColumnDef::new(OauthToken::ExpiresAt).timestamp_with_time_zone())
                .col(ColumnDef::new(OauthToken::Scopes).array(ColumnType::Text))
                .col(
                    ColumnDef::new(OauthToken::CreatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::cust("NOW()"))
                )
                .col(
                    ColumnDef::new(OauthToken::UpdatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::cust("NOW()"))
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk-oauth_token-user_id")
                        .from(OauthToken::Table, OauthToken::UserId)
                        .to(User::Table, User::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                )
                .index(
                    Index::create()
                        .name("idx-oauth_token-user_id-provider")
                        .table(OauthToken::Table)
                        .col(OauthToken::UserId)
                        .col(OauthToken::Provider)
                        .unique()
                )
                .to_owned()
        ).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(OauthToken::Table).to_owned()).await
    }
}

/*
CREATE TABLE oauth_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES user(id) ON DELETE CASCADE,
    provider TEXT NOT NULL CHECK (provider IN ('strava', 'spotify', 'lastfm')),
    access_token TEXT NOT NULL,
    refresh_token TEXT,
    expires_at TIMESTAMPTZ,
    scopes TEXT[],
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE 
); 
*/

#[derive(DeriveIden)]
enum OauthToken {
    Table,
    Id,
    UserId,
    Provider,
    AccessToken,
    RefreshToken,
    ExpiresAt,
    Scopes,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
}
