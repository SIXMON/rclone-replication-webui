use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared("ALTER TABLE task_runs ADD COLUMN IF NOT EXISTS stats JSONB").await?;

        // Backfill : extraire les stats JSON rclone des logs existants
        db.execute_unprepared(
            "DO $$
            DECLARE
                r RECORD;
                line TEXT;
                parsed JSONB;
            BEGIN
                FOR r IN SELECT id, log_output FROM task_runs
                         WHERE log_output IS NOT NULL AND stats IS NULL
                LOOP
                    FOR line IN SELECT unnest(string_to_array(r.log_output, E'\\n'))
                    LOOP
                        BEGIN
                            parsed := line::jsonb;
                            IF parsed ? 'stats' THEN
                                UPDATE task_runs SET stats = parsed->'stats' WHERE id = r.id;
                            END IF;
                        EXCEPTION WHEN OTHERS THEN
                            NULL;
                        END;
                    END LOOP;
                END LOOP;
            END;
            $$",
        ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.get_connection().execute_unprepared("ALTER TABLE task_runs DROP COLUMN IF EXISTS stats").await?;
        Ok(())
    }
}
