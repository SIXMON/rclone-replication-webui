use std::env;

#[derive(Clone, Debug)]
pub struct ScalewayConfig {
    pub secret_key: String,
    pub project_id: String,
    pub region: String,
    pub path: String,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub bind_addr: String,
    pub rclone_bin: String,
    pub apprise_bin: String,
    /// Scaleway Secret Manager configuration. None = secrets stockés en BDD (mode legacy).
    pub scaleway: Option<ScalewayConfig>,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let scaleway = if env::var("SCW_SECRET_MANAGER_ENABLED")
            .ok()
            .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
            .unwrap_or(false)
        {
            Some(ScalewayConfig {
                secret_key: env::var("SCW_SECRET_KEY")
                    .map_err(|_| anyhow::anyhow!("SCW_SECRET_KEY is required when SCW_SECRET_MANAGER_ENABLED=true"))?,
                project_id: env::var("SCW_PROJECT_ID")
                    .map_err(|_| anyhow::anyhow!("SCW_PROJECT_ID is required when SCW_SECRET_MANAGER_ENABLED=true"))?,
                region: env::var("SCW_DEFAULT_REGION").unwrap_or_else(|_| "fr-par".to_string()),
                path: env::var("SCW_SECRET_PATH").unwrap_or_else(|_| "/rclone-ui".to_string()),
            })
        } else {
            None
        };

        Ok(Config {
            database_url: env::var("DATABASE_URL")
                .map_err(|_| anyhow::anyhow!("DATABASE_URL is required"))?,
            bind_addr: env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string()),
            rclone_bin: env::var("RCLONE_BIN").unwrap_or_else(|_| "rclone".to_string()),
            apprise_bin: env::var("APPRISE_BIN").unwrap_or_else(|_| "apprise".to_string()),
            scaleway,
        })
    }
}
