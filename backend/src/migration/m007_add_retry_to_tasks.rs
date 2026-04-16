use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            "ALTER TABLE tasks ADD COLUMN IF NOT EXISTS max_retries INT NOT NULL DEFAULT 3",
        ).await?;
        db.execute_unprepared(
            "ALTER TABLE tasks ADD COLUMN IF NOT EXISTS retry_delay_seconds INT NOT NULL DEFAULT 15",
        ).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("ALTER TABLE tasks DROP COLUMN IF EXISTS max_retries").await?;
        db.execute_unprepared("ALTER TABLE tasks DROP COLUMN IF EXISTS retry_delay_seconds").await?;
        Ok(())
    }
}
