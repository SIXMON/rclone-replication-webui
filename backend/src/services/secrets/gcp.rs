use super::SecretStore;
use anyhow::{anyhow, Context};
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use gcp_auth::TokenProvider;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Implémentation du stockage de secrets via Google Cloud Secret Manager.
///
/// Convention : 1 secret par remote, nommé `rclone-ui-<remote_id>` dans le projet GCP configuré.
/// La valeur est un JSON object encodé en base64 (format attendu par l'API).
pub struct GoogleCloudSecretStore {
    project_id: String,
    token_provider: Arc<dyn TokenProvider>,
    client: reqwest::Client,
}

impl GoogleCloudSecretStore {
    pub async fn new(project_id: String) -> anyhow::Result<Self> {
        // gcp_auth détecte automatiquement les credentials :
        // GOOGLE_APPLICATION_CREDENTIALS, gcloud, metadata server (GCE/GKE), etc.
        let token_provider = gcp_auth::provider().await.context("GCP auth provider")?;
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .expect("reqwest client build");
        Ok(Self {
            project_id,
            token_provider,
            client,
        })
    }

    fn secret_id(remote_id: Uuid) -> String {
        format!("rclone-ui-{}", remote_id)
    }

    async fn access_token(&self) -> anyhow::Result<String> {
        let scopes = &["https://www.googleapis.com/auth/cloud-platform"];
        let token = self
            .token_provider
            .token(scopes)
            .await
            .context("GCP fetch token")?;
        Ok(token.as_str().to_string())
    }
}

#[async_trait]
impl SecretStore for GoogleCloudSecretStore {
    fn is_active(&self) -> bool {
        true
    }

    async fn get(&self, remote_id: Uuid) -> anyhow::Result<Option<HashMap<String, String>>> {
        let token = self.access_token().await?;
        let url = format!(
            "https://secretmanager.googleapis.com/v1/projects/{}/secrets/{}/versions/latest:access",
            self.project_id,
            Self::secret_id(remote_id)
        );
        let resp = self
            .client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await
            .context("GCP access secret")?;

        if resp.status().as_u16() == 404 {
            return Ok(None);
        }
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("GCP access error {status}: {body}"));
        }

        #[derive(Deserialize)]
        struct AccessResponse {
            payload: Payload,
        }
        #[derive(Deserialize)]
        struct Payload {
            data: String,
        }
        let parsed: AccessResponse = resp.json().await.context("GCP access parse")?;
        let bytes = BASE64.decode(parsed.payload.data).context("base64 decode")?;
        let map: HashMap<String, String> =
            serde_json::from_slice(&bytes).context("parse GCP secret JSON")?;
        Ok(Some(map))
    }

    async fn put(&self, remote_id: Uuid, fields: HashMap<String, String>) -> anyhow::Result<()> {
        let token = self.access_token().await?;
        let secret_id = Self::secret_id(remote_id);
        let payload = serde_json::to_vec(&fields).context("serialize GCP secret JSON")?;
        let data_b64 = BASE64.encode(&payload);

        // Étape 1 : créer le secret (conteneur) si absent
        let create_url = format!(
            "https://secretmanager.googleapis.com/v1/projects/{}/secrets?secretId={}",
            self.project_id, secret_id
        );
        let create_resp = self
            .client
            .post(&create_url)
            .bearer_auth(&token)
            .json(&serde_json::json!({
                "replication": { "automatic": {} },
                "labels": { "managed-by": "rclone-ui" }
            }))
            .send()
            .await
            .context("GCP create secret")?;

        // 409 = déjà existant, on ignore
        if !create_resp.status().is_success() && create_resp.status().as_u16() != 409 {
            let status = create_resp.status();
            let body = create_resp.text().await.unwrap_or_default();
            return Err(anyhow!("GCP create secret error {status}: {body}"));
        }

        // Étape 2 : ajouter une nouvelle version
        let version_url = format!(
            "https://secretmanager.googleapis.com/v1/projects/{}/secrets/{}:addVersion",
            self.project_id, secret_id
        );
        let version_resp = self
            .client
            .post(&version_url)
            .bearer_auth(&token)
            .json(&serde_json::json!({ "payload": { "data": data_b64 } }))
            .send()
            .await
            .context("GCP add version")?;
        if !version_resp.status().is_success() {
            let status = version_resp.status();
            let body = version_resp.text().await.unwrap_or_default();
            return Err(anyhow!("GCP add version error {status}: {body}"));
        }
        Ok(())
    }

    async fn delete(&self, remote_id: Uuid) -> anyhow::Result<()> {
        let token = self.access_token().await?;
        let url = format!(
            "https://secretmanager.googleapis.com/v1/projects/{}/secrets/{}",
            self.project_id,
            Self::secret_id(remote_id)
        );
        let resp = self
            .client
            .delete(&url)
            .bearer_auth(&token)
            .send()
            .await
            .context("GCP delete secret")?;
        if !resp.status().is_success() && resp.status().as_u16() != 404 {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("GCP delete error {status}: {body}"));
        }
        Ok(())
    }
}
