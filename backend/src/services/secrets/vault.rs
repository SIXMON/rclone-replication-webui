use super::SecretStore;
use anyhow::{anyhow, Context};
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use uuid::Uuid;

/// Implémentation du stockage de secrets via HashiCorp Vault (KV v2 secrets engine).
///
/// Convention : 1 secret par remote au chemin `<path_prefix>/<remote_id>`.
pub struct VaultSecretStore {
    addr: String,
    token: String,
    mount_path: String,
    path_prefix: String,
    client: reqwest::Client,
}

impl VaultSecretStore {
    pub fn new(addr: String, token: String, mount_path: String, path_prefix: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .expect("reqwest client build");
        Self {
            addr: addr.trim_end_matches('/').to_string(),
            token,
            mount_path,
            path_prefix: path_prefix.trim_matches('/').to_string(),
            client,
        }
    }

    fn data_url(&self, remote_id: Uuid) -> String {
        format!(
            "{}/v1/{}/data/{}/{}",
            self.addr, self.mount_path, self.path_prefix, remote_id
        )
    }

    fn metadata_url(&self, remote_id: Uuid) -> String {
        format!(
            "{}/v1/{}/metadata/{}/{}",
            self.addr, self.mount_path, self.path_prefix, remote_id
        )
    }
}

#[async_trait]
impl SecretStore for VaultSecretStore {
    fn is_active(&self) -> bool {
        true
    }

    async fn get(&self, remote_id: Uuid) -> anyhow::Result<Option<HashMap<String, String>>> {
        let resp = self
            .client
            .get(self.data_url(remote_id))
            .header("X-Vault-Token", &self.token)
            .send()
            .await
            .context("Vault read")?;

        if resp.status().as_u16() == 404 {
            return Ok(None);
        }
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Vault read error {status}: {body}"));
        }

        #[derive(Deserialize)]
        struct ReadResponse {
            data: ReadData,
        }
        #[derive(Deserialize)]
        struct ReadData {
            data: HashMap<String, String>,
        }
        let parsed: ReadResponse = resp.json().await.context("Vault read parse")?;
        Ok(Some(parsed.data.data))
    }

    async fn put(&self, remote_id: Uuid, fields: HashMap<String, String>) -> anyhow::Result<()> {
        let resp = self
            .client
            .post(self.data_url(remote_id))
            .header("X-Vault-Token", &self.token)
            .json(&serde_json::json!({ "data": fields }))
            .send()
            .await
            .context("Vault write")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Vault write error {status}: {body}"));
        }
        Ok(())
    }

    async fn delete(&self, remote_id: Uuid) -> anyhow::Result<()> {
        // DELETE sur metadata supprime le secret et toutes ses versions
        let resp = self
            .client
            .delete(self.metadata_url(remote_id))
            .header("X-Vault-Token", &self.token)
            .send()
            .await
            .context("Vault delete")?;
        if !resp.status().is_success() && resp.status().as_u16() != 404 {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Vault delete error {status}: {body}"));
        }
        Ok(())
    }
}
