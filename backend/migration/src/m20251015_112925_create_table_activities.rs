use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    #[allow(clippy::too_many_lines)]
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Activity::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Activity::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(Activity::UserId).uuid().not_null())
                    .col(
                        ColumnDef::new(Activity::ExternalId)
                            .big_integer()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Activity::Name).text().not_null())
                    .col(ColumnDef::new(Activity::Description).text())
                    .col(ColumnDef::new(Activity::Type).string().not_null())
                    .col(
                        ColumnDef::new(Activity::StartTime)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Activity::MovingTime).integer().not_null())
                    .col(ColumnDef::new(Activity::ElapsedTime).integer().not_null())
                    .col(ColumnDef::new(Activity::Timezone).string().not_null())
                    .col(ColumnDef::new(Activity::Distance).float().not_null())
                    .col(
                        ColumnDef::new(Activity::TotalElevationGain)
                            .float()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Activity::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Activity::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-activity-user_id")
                            .from(Activity::Table, Activity::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes separately
        manager
            .create_index(
                Index::create()
                    .name("idx-activities-user_id")
                    .table(Activity::Table)
                    .col(Activity::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-activity-start_time")
                    .table(Activity::Table)
                    .col(Activity::StartTime)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-activity-external_id")
                    .table(Activity::Table)
                    .col(Activity::ExternalId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-activity-user_start")
                    .table(Activity::Table)
                    .col(Activity::UserId)
                    .col(Activity::StartTime)
                    .to_owned(),
            )
            .await?;

        // Create activity_streams table
        manager
            .create_table(
                Table::create()
                    .table(ActivityStream::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(ActivityStream::ActivityId).uuid().not_null())
                    .col(
                        ColumnDef::new(ActivityStream::Time)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ActivityStream::Latitude).double())
                    .col(ColumnDef::new(ActivityStream::Longitude).double())
                    .col(ColumnDef::new(ActivityStream::Altitude).float())
                    .col(ColumnDef::new(ActivityStream::HeartRate).integer())
                    .col(ColumnDef::new(ActivityStream::Cadence).integer())
                    .col(ColumnDef::new(ActivityStream::Watts).float())
                    .col(ColumnDef::new(ActivityStream::Velocity).float())
                    .col(ColumnDef::new(ActivityStream::Distance).float())
                    .col(ColumnDef::new(ActivityStream::Temperature).float())
                    .primary_key(
                        Index::create()
                            .col(ActivityStream::ActivityId)
                            .col(ActivityStream::Time),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-activity_stream-activity_id")
                            .from(ActivityStream::Table, ActivityStream::ActivityId)
                            .to(Activity::Table, Activity::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create hypertable (TimescaleDB-specific)
        manager
            .get_connection()
            .execute_unprepared(
                "SELECT create_hypertable('activity_stream', 'time', if_not_exists => TRUE);",
            )
            .await?;

        // Compression policy (saves space after 30 days)
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE activity_stream SET (
                    timescaledb.compress,
                    timescaledb.compress_segmentby = 'activity_id'
                );",
            )
            .await?;
        manager
            .get_connection()
            .execute_unprepared(
                "SELECT add_compression_policy('activity_stream', INTERVAL '30 days');",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Activity::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ActivityStream::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Activity {
    Table,
    Id,
    UserId,
    ExternalId, // ID from external service (e.g., Strava)
    Name,
    Description,
    Type,
    StartTime,          // UTC timestamp in ISO 8601 format
    MovingTime,         // in seconds
    ElapsedTime,        // in seconds
    Timezone,           // IANA timezone string
    Distance,           // in meters
    TotalElevationGain, // in meters
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum ActivityStream {
    Table,
    Time,        // UTC timestamp in ISO 8601 format
    ActivityId,  // Foreign key to activities.id
    Latitude,    // in decimal degrees
    Longitude,   // in decimal degrees
    Altitude,    // in meters
    HeartRate,   // in bpm
    Cadence,     // in rpm
    Watts,       // in watts
    Velocity,    // in m/s
    Distance,    // cumulative distance in meters
    Temperature, // in celsius
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
}
