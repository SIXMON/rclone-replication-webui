use serde::Serialize;

/// Vue sans log_output pour les listes (inclut stats pour le tableau)
#[derive(Debug, Serialize)]
pub struct TaskRunSummary {
    pub id: uuid::Uuid,
    pub task_id: uuid::Uuid,
    pub triggered_by: String,
    pub status: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub finished_at: Option<chrono::DateTime<chrono::Utc>>,
    pub duration_ms: Option<i64>,
    pub exit_code: Option<i32>,
    pub stats: Option<serde_json::Value>,
}
