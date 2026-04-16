use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Vue enrichie pour la liste (avec le nombre de tâches référençant ce canal)
#[derive(Debug, Serialize)]
pub struct ChannelWithTaskCount {
    pub id: Uuid,
    pub name: String,
    pub apprise_url: String,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub task_count: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateChannelRequest {
    pub name: String,
    pub apprise_url: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateChannelRequest {
    pub name: String,
    pub apprise_url: String,
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}
