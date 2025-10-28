use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    #[allow(clippy::too_many_lines)]
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create tracks table (normalized track metadata)
        manager
            .create_table(
                Table::create()
                    .table(Track::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Track::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(Track::ArtistName).text().not_null())
                    .col(ColumnDef::new(Track::TrackName).text().not_null())
                    .col(ColumnDef::new(Track::AlbumName).text())
                    .col(ColumnDef::new(Track::ArtistMbid).text())
                    .col(ColumnDef::new(Track::TrackMbid).text())
                    .col(ColumnDef::new(Track::AlbumMbid).text())
                    .col(ColumnDef::new(Track::LastfmUrl).text())
                    .col(
                        ColumnDef::new(Track::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Track::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create unique index on (artist_name, track_name) for deduplication
        manager
            .create_index(
                Index::create()
                    .name("idx-track-artist-track-name")
                    .table(Track::Table)
                    .col(Track::ArtistName)
                    .col(Track::TrackName)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Create indexes for MusicBrainz IDs (for future Spotify enrichment)
        manager
            .create_index(
                Index::create()
                    .name("idx-track-track-mbid")
                    .table(Track::Table)
                    .col(Track::TrackMbid)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-track-artist-mbid")
                    .table(Track::Table)
                    .col(Track::ArtistMbid)
                    .to_owned(),
            )
            .await?;

        // Create listens table (listening events)
        manager
            .create_table(
                Table::create()
                    .table(Listen::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Listen::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(Listen::UserId).uuid().not_null())
                    .col(ColumnDef::new(Listen::TrackId).uuid().not_null())
                    .col(
                        ColumnDef::new(Listen::PlayedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Listen::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-listen-user_id")
                            .from(Listen::Table, Listen::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-listen-track_id")
                            .from(Listen::Table, Listen::TrackId)
                            .to(Track::Table, Track::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create composite index on (user_id, played_at) for time-range queries
        manager
            .create_index(
                Index::create()
                    .name("idx-listen-user-played-at")
                    .table(Listen::Table)
                    .col(Listen::UserId)
                    .col(Listen::PlayedAt)
                    .to_owned(),
            )
            .await?;

        // Create index on track_id for reverse lookups
        manager
            .create_index(
                Index::create()
                    .name("idx-listen-track-id")
                    .table(Listen::Table)
                    .col(Listen::TrackId)
                    .to_owned(),
            )
            .await?;

        // Create unique constraint to prevent duplicate listens
        manager
            .create_index(
                Index::create()
                    .name("idx-listen-unique-event")
                    .table(Listen::Table)
                    .col(Listen::UserId)
                    .col(Listen::TrackId)
                    .col(Listen::PlayedAt)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Listen::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Track::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
#[allow(clippy::enum_variant_names)]
enum Track {
    Table,
    Id,
    ArtistName,
    TrackName,
    AlbumName,
    ArtistMbid, // MusicBrainz ID for artist
    TrackMbid,  // MusicBrainz ID for track
    AlbumMbid,  // MusicBrainz ID for album
    LastfmUrl,  // Last.fm URL
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Listen {
    Table,
    Id,
    UserId,   // Foreign key to users.id
    TrackId,  // Foreign key to tracks.id
    PlayedAt, // Timestamp when the track was played
    CreatedAt,
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
}
