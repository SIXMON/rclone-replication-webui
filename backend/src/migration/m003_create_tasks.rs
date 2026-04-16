use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.get_connection().execute_unprepared(
            "CREATE TABLE IF NOT EXISTS tasks (
                id                       UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                name                     TEXT NOT NULL,
                source_remote_id         UUID NOT NULL REFERENCES remotes(id),
                source_path              TEXT NOT NULL DEFAULT '',
                dest_remote_id           UUID NOT NULL REFERENCES remotes(id),
                dest_path                TEXT NOT NULL DEFAULT '',
                cron_expression          TEXT,
                enabled                  BOOLEAN NOT NULL DEFAULT TRUE,
                rclone_flags             TEXT[] NOT NULL DEFAULT '{}',
                notification_channel_id  UUID REFERENCES notification_channels(id) ON DELETE SET NULL,
                created_at               TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at               TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )",
        ).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.get_connection().execute_unprepared("DROP TABLE IF EXISTS tasks").await?;
        Ok(())
    }
}
