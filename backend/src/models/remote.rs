use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Vue enrichie pour la liste (avec le nombre de tâches référençant ce remote)
#[derive(Debug, Serialize)]
pub struct RemoteWithTaskCount {
    pub id: Uuid,
    pub name: String,
    pub remote_type: String,
    pub config: Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub task_count: i64,
}

/// Vue légère pour le service rclone (génération de config)
#[derive(Debug, Clone)]
pub struct RcloneRemote {
    pub id: Uuid,
    pub name: String,
    pub remote_type: String,
    pub config: Value,
}

#[derive(Debug, Deserialize)]
pub struct CreateRemoteRequest {
    pub name: String,
    pub remote_type: String,
    pub config: Value,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRemoteRequest {
    pub name: String,
    pub remote_type: String,
    pub config: Value,
}
