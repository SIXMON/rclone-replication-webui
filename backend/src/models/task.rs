use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Vue enrichie avec les noms des remotes et le dernier run
#[derive(Debug, Serialize)]
pub struct TaskWithMeta {
    pub id: Uuid,
    pub name: String,
    pub source_remote_id: Uuid,
    pub source_remote_name: String,
    pub source_path: String,
    pub dest_remote_id: Uuid,
    pub dest_remote_name: String,
    pub dest_path: String,
    pub cron_expression: Option<String>,
    pub enabled: bool,
    pub rclone_flags: Vec<String>,
    pub notification_channel_id: Option<Uuid>,
    pub notify_on: Vec<String>,
    pub max_retries: i32,
    pub retry_delay_seconds: i32,
    pub last_run: Option<LastRunSummary>,
    pub running: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct LastRunSummary {
    pub status: String,
    pub started_at: DateTime<Utc>,
    pub duration_ms: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub name: String,
    pub source_remote_id: Uuid,
    pub source_path: String,
    pub dest_remote_id: Uuid,
    pub dest_path: String,
    pub cron_expression: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub rclone_flags: Vec<String>,
    pub notification_channel_id: Option<Uuid>,
    #[serde(default = "default_notify_on")]
    pub notify_on: Vec<String>,
    #[serde(default = "default_max_retries")]
    pub max_retries: i32,
    #[serde(default = "default_retry_delay")]
    pub retry_delay_seconds: i32,
}

#[derive(Debug, Deserialize)]
pub struct PatchTaskRequest {
    pub name: Option<String>,
    pub cron_expression: Option<Option<String>>,
    pub enabled: Option<bool>,
    pub rclone_flags: Option<Vec<String>>,
    pub notification_channel_id: Option<Option<Uuid>>,
    pub notify_on: Option<Vec<String>>,
    pub max_retries: Option<i32>,
    pub retry_delay_seconds: Option<i32>,
}

fn default_true() -> bool {
    true
}

fn default_notify_on() -> Vec<String> {
    vec!["error".to_string()]
}

fn default_max_retries() -> i32 {
    3
}

fn default_retry_delay() -> i32 {
    15
}
