use super::SecretStore;
use anyhow::{anyhow, Context};
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

struct CachedToken {
    token: String,
    expires_at: std::time::Instant,
}

/// Implémentation du stockage de secrets via Infisical (Universal Auth).
///
/// Convention : 1 secret par remote, nommé `<remote_id>` au chemin configuré.
/// La valeur est un JSON object stocké tel quel.
pub struct InfisicalSecretStore {
    host: String,
    client_id: String,
    client_secret: String,
    project_id: String,
    environment: String,
    secret_path: String,
    client: reqwest::Client,
    token_cache: Arc<RwLock<Option<CachedToken>>>,
}

impl InfisicalSecretStore {
    pub fn new(
        host: String,
        client_id: String,
        client_secret: String,
        project_id: String,
        environment: String,
        secret_path: String,
    ) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .expect("reqwest client build");
        Self {
            host: host.trim_end_matches('/').to_string(),
            client_id,
            client_secret,
            project_id,
            environment,
            secret_path,
            client,
            token_cache: Arc::new(RwLock::new(None)),
        }
    }

    fn secret_name(remote_id: Uuid) -> String {
        // Infisical exige des noms en SCREAMING_SNAKE_CASE
        format!("RCLONE_UI_{}", remote_id.to_string().replace('-', "_").to_uppercase())
    }

    async fn access_token(&self) -> anyhow::Result<String> {
        {
            let cache = self.token_cache.read().await;
            if let Some(t) = cache.as_ref() {
                if t.expires_at > std::time::Instant::now() {
                    return Ok(t.token.clone());
                }
            }
        }

        let url = format!("{}/api/v1/auth/universal-auth/login", self.host);
        let resp = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "clientId": self.client_id,
                "clientSecret": self.client_secret,
            }))
            .send()
            .await
            .context("Infisical login")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Infisical login error {status}: {body}"));
        }

        #[derive(Deserialize)]
        struct LoginResponse {
            #[serde(rename = "accessToken")]
            access_token: String,
            #[serde(rename = "expiresIn")]
            expires_in: u64,
        }
        let parsed: LoginResponse = resp.json().await.context("Infisical login parse")?;

        let mut cache = self.token_cache.write().await;
        *cache = Some(CachedToken {
            token: parsed.access_token.clone(),
            expires_at: std::time::Instant::now()
                + std::time::Duration::from_secs(parsed.expires_in.saturating_sub(60)),
        });
        Ok(parsed.access_token)
    }
}

#[async_trait]
impl SecretStore for InfisicalSecretStore {
    fn is_active(&self) -> bool {
        true
    }

    async fn get(&self, remote_id: Uuid) -> anyhow::Result<Option<HashMap<String, String>>> {
        let token = self.access_token().await?;
        let url = format!("{}/api/v3/secrets/raw/{}", self.host, Self::secret_name(remote_id));
        let resp = self
            .client
            .get(&url)
            .bearer_auth(&token)
            .query(&[
                ("workspaceId", self.project_id.as_str()),
                ("environment", self.environment.as_str()),
                ("secretPath", self.secret_path.as_str()),
            ])
            .send()
            .await
            .context("Infisical get secret")?;

        if resp.status().as_u16() == 404 {
            return Ok(None);
        }
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Infisical get error {status}: {body}"));
        }

        #[derive(Deserialize)]
        struct GetResponse {
            secret: SecretData,
        }
        #[derive(Deserialize)]
        struct SecretData {
            #[serde(rename = "secretValue")]
            secret_value: String,
        }
        let parsed: GetResponse = resp.json().await.context("Infisical get parse")?;
        let map: HashMap<String, String> =
            serde_json::from_str(&parsed.secret.secret_value).context("parse Infisical secret JSON")?;
        Ok(Some(map))
    }

    async fn put(&self, remote_id: Uuid, fields: HashMap<String, String>) -> anyhow::Result<()> {
        let token = self.access_token().await?;
        let secret_name = Self::secret_name(remote_id);
        let value = serde_json::to_string(&fields).context("serialize Infisical secret JSON")?;
        let url = format!("{}/api/v3/secrets/raw/{}", self.host, secret_name);

        // Tenter un POST (création). Si déjà existant, faire un PATCH (update).
        let create_body = serde_json::json!({
            "workspaceId": self.project_id,
            "environment": self.environment,
            "secretPath": self.secret_path,
            "secretValue": value,
            "type": "shared",
        });
        let resp = self
            .client
            .post(&url)
            .bearer_auth(&token)
            .json(&create_body)
            .send()
            .await
            .context("Infisical create secret")?;

        // 400 = secret existe déjà → on fait un PATCH
        if resp.status().as_u16() == 400 {
            let resp_update = self
                .client
                .patch(&url)
                .bearer_auth(&token)
                .json(&serde_json::json!({
                    "workspaceId": self.project_id,
                    "environment": self.environment,
                    "secretPath": self.secret_path,
                    "secretValue": value,
                    "type": "shared",
                }))
                .send()
                .await
                .context("Infisical update secret")?;
            if !resp_update.status().is_success() {
                let status = resp_update.status();
                let body = resp_update.text().await.unwrap_or_default();
                return Err(anyhow!("Infisical update error {status}: {body}"));
            }
        } else if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Infisical create error {status}: {body}"));
        }
        Ok(())
    }

    async fn delete(&self, remote_id: Uuid) -> anyhow::Result<()> {
        let token = self.access_token().await?;
        let url = format!("{}/api/v3/secrets/raw/{}", self.host, Self::secret_name(remote_id));
        let resp = self
            .client
            .delete(&url)
            .bearer_auth(&token)
            .json(&serde_json::json!({
                "workspaceId": self.project_id,
                "environment": self.environment,
                "secretPath": self.secret_path,
            }))
            .send()
            .await
            .context("Infisical delete secret")?;
        if !resp.status().is_success() && resp.status().as_u16() != 404 {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Infisical delete error {status}: {body}"));
        }
        Ok(())
    }
}
