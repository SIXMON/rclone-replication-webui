use super::SecretStore;
use anyhow::{anyhow, Context};
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

const API_VERSION: &str = "7.4";

/// Cache pour le token OAuth Azure (renouvelé automatiquement avant expiration).
struct CachedToken {
    token: String,
    expires_at: std::time::Instant,
}

/// Implémentation du stockage de secrets via Azure Key Vault.
///
/// Convention : 1 secret par remote, nommé `<remote_id>` (sans tirets — Key Vault ne les autorise pas).
/// La valeur est un JSON object `{"<field_name>": "<value>", ...}`.
pub struct AzureKeyVaultSecretStore {
    tenant_id: String,
    client_id: String,
    client_secret: String,
    vault_url: String,
    client: reqwest::Client,
    token_cache: Arc<RwLock<Option<CachedToken>>>,
}

impl AzureKeyVaultSecretStore {
    pub fn new(tenant_id: String, client_id: String, client_secret: String, vault_url: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .expect("reqwest client build");
        Self {
            tenant_id,
            client_id,
            client_secret,
            vault_url: vault_url.trim_end_matches('/').to_string(),
            client,
            token_cache: Arc::new(RwLock::new(None)),
        }
    }

    /// Nom du secret côté Key Vault. Les tirets sont autorisés en réalité, mais on garde le format UUID.
    fn secret_name(remote_id: Uuid) -> String {
        format!("rclone-ui-{}", remote_id)
    }

    /// Récupère un access token OAuth (cache 1 minute avant expiration).
    async fn access_token(&self) -> anyhow::Result<String> {
        {
            let cache = self.token_cache.read().await;
            if let Some(t) = cache.as_ref() {
                if t.expires_at > std::time::Instant::now() {
                    return Ok(t.token.clone());
                }
            }
        }

        let url = format!("https://login.microsoftonline.com/{}/oauth2/v2.0/token", self.tenant_id);
        let resp = self
            .client
            .post(&url)
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
                ("scope", "https://vault.azure.net/.default"),
                ("grant_type", "client_credentials"),
            ])
            .send()
            .await
            .context("Azure OAuth token request")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Azure token error {status}: {body}"));
        }

        #[derive(Deserialize)]
        struct TokenResponse {
            access_token: String,
            expires_in: u64,
        }
        let parsed: TokenResponse = resp.json().await.context("Azure token parse")?;

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
impl SecretStore for AzureKeyVaultSecretStore {
    fn is_active(&self) -> bool {
        true
    }

    async fn get(&self, remote_id: Uuid) -> anyhow::Result<Option<HashMap<String, String>>> {
        let token = self.access_token().await?;
        let url = format!(
            "{}/secrets/{}?api-version={}",
            self.vault_url,
            Self::secret_name(remote_id),
            API_VERSION
        );
        let resp = self
            .client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await
            .context("Azure get secret")?;

        if resp.status().as_u16() == 404 {
            return Ok(None);
        }
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Azure get error {status}: {body}"));
        }

        #[derive(Deserialize)]
        struct GetResponse {
            value: String,
        }
        let parsed: GetResponse = resp.json().await.context("Azure get parse")?;
        let map: HashMap<String, String> =
            serde_json::from_str(&parsed.value).context("parse Azure secret JSON")?;
        Ok(Some(map))
    }

    async fn put(&self, remote_id: Uuid, fields: HashMap<String, String>) -> anyhow::Result<()> {
        let token = self.access_token().await?;
        let value = serde_json::to_string(&fields).context("serialize Azure secret JSON")?;
        let url = format!(
            "{}/secrets/{}?api-version={}",
            self.vault_url,
            Self::secret_name(remote_id),
            API_VERSION
        );
        let resp = self
            .client
            .put(&url)
            .bearer_auth(&token)
            .json(&serde_json::json!({ "value": value, "tags": { "managed-by": "rclone-ui" } }))
            .send()
            .await
            .context("Azure put secret")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Azure put error {status}: {body}"));
        }
        Ok(())
    }

    async fn delete(&self, remote_id: Uuid) -> anyhow::Result<()> {
        let token = self.access_token().await?;
        let url = format!(
            "{}/secrets/{}?api-version={}",
            self.vault_url,
            Self::secret_name(remote_id),
            API_VERSION
        );
        let resp = self
            .client
            .delete(&url)
            .bearer_auth(&token)
            .send()
            .await
            .context("Azure delete secret")?;
        if !resp.status().is_success() && resp.status().as_u16() != 404 {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Azure delete error {status}: {body}"));
        }
        Ok(())
    }
}
