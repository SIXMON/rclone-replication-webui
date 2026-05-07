use crate::services::secrets::{self, SecretStore};
use crate::sse::broadcaster::SseBroadcaster;
use crate::sse::global::GlobalBroadcaster;
use dashmap::DashMap;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_cron_scheduler::JobScheduler;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct RunningTask {
    pub run_id: Uuid,
    pub triggered_by: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub running_tasks: Arc<DashMap<Uuid, RunningTask>>,
    pub sse_broadcaster: Arc<SseBroadcaster>,
    pub global_broadcaster: Arc<GlobalBroadcaster>,
    pub config: Arc<crate::config::Config>,
    pub scheduler_handle: Arc<Mutex<Option<JobScheduler>>>,
    pub secret_store: Arc<dyn SecretStore>,
}

impl AppState {
    pub async fn new(db: DatabaseConnection, config: crate::config::Config) -> Self {
        let secret_store = secrets::build(config.secret_manager.as_ref()).await;
        AppState {
            db,
            running_tasks: Arc::new(DashMap::new()),
            sse_broadcaster: Arc::new(SseBroadcaster::new()),
            global_broadcaster: Arc::new(GlobalBroadcaster::new()),
            config: Arc::new(config),
            scheduler_handle: Arc::new(Mutex::new(None)),
            secret_store,
        }
    }
}
