use super::SecretStore;
use anyhow::{anyhow, Context};
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use uuid::Uuid;

/// Implémentation du stockage de secrets via Doppler.
///
/// Convention : 1 secret par remote, nommé `RCLONE_UI_<UUID_SANS_TIRETS>` (Doppler exige des noms
/// en SCREAMING_SNAKE_CASE). La valeur est un JSON object stocké tel quel.
pub struct DopplerSecretStore {
    token: String,
    project: String,
    config: String,
    client: reqwest::Client,
}

impl DopplerSecretStore {
    pub fn new(token: String, project: String, config: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .expect("reqwest client build");
        Self { token, project, config, client }
    }

    fn secret_name(remote_id: Uuid) -> String {
        format!("RCLONE_UI_{}", remote_id.to_string().replace('-', "").to_uppercase())
    }

    fn base_url() -> &'static str {
        "https://api.doppler.com"
    }
}

#[async_trait]
impl SecretStore for DopplerSecretStore {
    fn is_active(&self) -> bool {
        true
    }

    async fn get(&self, remote_id: Uuid) -> anyhow::Result<Option<HashMap<String, String>>> {
        let url = format!("{}/v3/configs/config/secret", Self::base_url());
        let resp = self
            .client
            .get(&url)
            .bearer_auth(&self.token)
            .query(&[
                ("project", self.project.as_str()),
                ("config", self.config.as_str()),
                ("name", &Self::secret_name(remote_id)),
            ])
            .send()
            .await
            .context("Doppler get secret")?;

        if resp.status().as_u16() == 404 {
            return Ok(None);
        }
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            // Doppler peut renvoyer 400 si le secret n'existe pas
            if body.contains("not found") || body.contains("not_found") {
                return Ok(None);
            }
            return Err(anyhow!("Doppler get error {status}: {body}"));
        }

        #[derive(Deserialize)]
        struct GetResponse {
            value: SecretValue,
        }
        #[derive(Deserialize)]
        struct SecretValue {
            raw: Option<String>,
        }
        let parsed: GetResponse = resp.json().await.context("Doppler get parse")?;
        match parsed.value.raw {
            Some(raw) if !raw.is_empty() => {
                let map: HashMap<String, String> =
                    serde_json::from_str(&raw).context("parse Doppler secret JSON")?;
                Ok(Some(map))
            }
            _ => Ok(None),
        }
    }

    async fn put(&self, remote_id: Uuid, fields: HashMap<String, String>) -> anyhow::Result<()> {
        let value = serde_json::to_string(&fields).context("serialize Doppler secret JSON")?;
        let url = format!("{}/v3/configs/config/secrets", Self::base_url());
        let resp = self
            .client
            .post(&url)
            .bearer_auth(&self.token)
            .json(&serde_json::json!({
                "project": self.project,
                "config": self.config,
                "secrets": { Self::secret_name(remote_id): value },
            }))
            .send()
            .await
            .context("Doppler put secret")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Doppler put error {status}: {body}"));
        }
        Ok(())
    }

    async fn delete(&self, remote_id: Uuid) -> anyhow::Result<()> {
        // Doppler : envoyer la valeur null pour supprimer
        let url = format!("{}/v3/configs/config/secrets", Self::base_url());
        let resp = self
            .client
            .post(&url)
            .bearer_auth(&self.token)
            .json(&serde_json::json!({
                "project": self.project,
                "config": self.config,
                "secrets": { Self::secret_name(remote_id): serde_json::Value::Null },
            }))
            .send()
            .await
            .context("Doppler delete secret")?;
        if !resp.status().is_success() && resp.status().as_u16() != 404 {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Doppler delete error {status}: {body}"));
        }
        Ok(())
    }
}
