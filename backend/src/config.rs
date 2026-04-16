use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub bind_addr: String,
    pub rclone_bin: String,
    pub apprise_bin: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();
        Ok(Config {
            database_url: env::var("DATABASE_URL")
                .map_err(|_| anyhow::anyhow!("DATABASE_URL is required"))?,
            bind_addr: env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string()),
            rclone_bin: env::var("RCLONE_BIN").unwrap_or_else(|_| "rclone".to_string()),
            apprise_bin: env::var("APPRISE_BIN").unwrap_or_else(|_| "apprise".to_string()),
        })
    }
}
