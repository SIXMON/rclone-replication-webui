pub use sea_orm_migration::prelude::*;

mod m001_create_remotes;
mod m002_create_notification_channels;
mod m003_create_tasks;
mod m004_create_task_runs;
mod m005_add_stats_to_task_runs;
mod m006_add_notify_on_to_tasks;
mod m007_add_retry_to_tasks;
mod m008_add_skipped_status;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m001_create_remotes::Migration),
            Box::new(m002_create_notification_channels::Migration),
            Box::new(m003_create_tasks::Migration),
            Box::new(m004_create_task_runs::Migration),
            Box::new(m005_add_stats_to_task_runs::Migration),
            Box::new(m006_add_notify_on_to_tasks::Migration),
            Box::new(m007_add_retry_to_tasks::Migration),
            Box::new(m008_add_skipped_status::Migration),
        ]
    }
}
