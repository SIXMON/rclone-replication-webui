use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Ajouter 'skipped' au CHECK constraint du statut
        db.execute_unprepared(
            "ALTER TABLE task_runs DROP CONSTRAINT IF EXISTS task_runs_status_check",
        ).await?;
        db.execute_unprepared(
            "ALTER TABLE task_runs ADD CONSTRAINT task_runs_status_check
             CHECK (status IN ('running', 'success', 'failure', 'skipped'))",
        ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            "ALTER TABLE task_runs DROP CONSTRAINT IF EXISTS task_runs_status_check",
        ).await?;
        db.execute_unprepared(
            "ALTER TABLE task_runs ADD CONSTRAINT task_runs_status_check
             CHECK (status IN ('running', 'success', 'failure'))",
        ).await?;
        Ok(())
    }
}
