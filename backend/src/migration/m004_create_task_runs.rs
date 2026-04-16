use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS task_runs (
                id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                task_id      UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
                triggered_by TEXT NOT NULL CHECK (triggered_by IN ('scheduler', 'manual', 'restore')),
                status       TEXT NOT NULL DEFAULT 'running' CHECK (status IN ('running', 'success', 'failure')),
                started_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                finished_at  TIMESTAMPTZ,
                duration_ms  BIGINT,
                exit_code    INTEGER,
                log_output   TEXT
            )",
        ).await?;

        db.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_task_runs_one_running
                ON task_runs (task_id) WHERE status = 'running'",
        ).await?;

        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_task_runs_task_started
                ON task_runs (task_id, started_at DESC)",
        ).await?;

        db.execute_unprepared(
            "CREATE OR REPLACE FUNCTION trim_task_runs() RETURNS TRIGGER AS $$
            BEGIN
                DELETE FROM task_runs
                WHERE task_id = NEW.task_id
                  AND id NOT IN (
                      SELECT id FROM task_runs
                      WHERE task_id = NEW.task_id
                      ORDER BY started_at DESC
                      LIMIT 100
                  );
                RETURN NULL;
            END;
            $$ LANGUAGE plpgsql",
        ).await?;

        // DROP + CREATE pour le trigger (pas de IF NOT EXISTS pour les triggers)
        db.execute_unprepared("DROP TRIGGER IF EXISTS trg_trim_task_runs ON task_runs").await?;
        db.execute_unprepared(
            "CREATE TRIGGER trg_trim_task_runs
            AFTER INSERT ON task_runs
            FOR EACH ROW EXECUTE FUNCTION trim_task_runs()",
        ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("DROP TRIGGER IF EXISTS trg_trim_task_runs ON task_runs").await?;
        db.execute_unprepared("DROP FUNCTION IF EXISTS trim_task_runs").await?;
        db.execute_unprepared("DROP TABLE IF EXISTS task_runs").await?;
        Ok(())
    }
}
