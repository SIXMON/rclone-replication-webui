use super::SecretStore;
use anyhow::{anyhow, Context};
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::Deserialize;
use std::collections::HashMap;
use uuid::Uuid;

/// Implémentation du stockage de secrets via l'API Scaleway Secret Manager.
///
/// Convention : 1 secret par remote, nommé `<remote_id>` dans le path configuré.
/// La valeur est un JSON object `{"<field_name>": "<value>", ...}` encodé en base64.
pub struct ScalewaySecretStore {
    secret_key: String,
    project_id: String,
    region: String,
    path: String,
    client: reqwest::Client,
}

impl ScalewaySecretStore {
    pub fn new(secret_key: String, project_id: String, region: String, path: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .expect("reqwest client build");
        Self { secret_key, project_id, region, path, client }
    }

    fn base_url(&self) -> String {
        format!(
            "https://api.scaleway.com/secret-manager/v1beta1/regions/{}",
            self.region
        )
    }

    /// Trouve l'ID d'un secret par son nom dans le path configuré, s'il existe.
    async fn find_secret_id(&self, name: &str) -> anyhow::Result<Option<String>> {
        let url = format!("{}/secrets", self.base_url());
        let resp = self
            .client
            .get(&url)
            .header("X-Auth-Token", &self.secret_key)
            .query(&[
                ("project_id", self.project_id.as_str()),
                ("name", name),
                ("path", self.path.as_str()),
            ])
            .send()
            .await
            .context("Scaleway SM list secrets")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Scaleway SM list error {status}: {body}"));
        }

        #[derive(Deserialize)]
        struct ListResponse {
            secrets: Vec<SecretRef>,
        }
        #[derive(Deserialize)]
        struct SecretRef {
            id: String,
            name: String,
        }
        let parsed: ListResponse = resp.json().await.context("Scaleway SM list parse")?;
        Ok(parsed.secrets.into_iter().find(|s| s.name == name).map(|s| s.id))
    }

    fn name_for(remote_id: Uuid) -> String {
        remote_id.to_string()
    }
}

#[async_trait]
impl SecretStore for ScalewaySecretStore {
    fn is_active(&self) -> bool {
        true
    }

    async fn get(&self, remote_id: Uuid) -> anyhow::Result<Option<HashMap<String, String>>> {
        let name = Self::name_for(remote_id);
        let secret_id = match self.find_secret_id(&name).await? {
            Some(id) => id,
            None => return Ok(None),
        };

        let url = format!(
            "{}/secrets/{}/versions/latest_enabled/access",
            self.base_url(),
            secret_id
        );
        let resp = self
            .client
            .get(&url)
            .header("X-Auth-Token", &self.secret_key)
            .send()
            .await
            .context("Scaleway SM access version")?;

        if resp.status().as_u16() == 404 {
            return Ok(None);
        }
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Scaleway SM access error {status}: {body}"));
        }

        #[derive(Deserialize)]
        struct AccessResponse {
            data: String,
        }
        let parsed: AccessResponse = resp.json().await.context("Scaleway SM access parse")?;
        let bytes = BASE64.decode(parsed.data).context("base64 decode secret")?;
        let map: HashMap<String, String> =
            serde_json::from_slice(&bytes).context("parse secret JSON")?;
        Ok(Some(map))
    }

    async fn put(&self, remote_id: Uuid, fields: HashMap<String, String>) -> anyhow::Result<()> {
        let name = Self::name_for(remote_id);
        let payload = serde_json::to_vec(&fields).context("serialize secret JSON")?;
        let data_b64 = BASE64.encode(&payload);

        // Étape 1 : récupérer ou créer le secret (conteneur)
        let secret_id = match self.find_secret_id(&name).await? {
            Some(id) => id,
            None => {
                // Créer un nouveau secret vide
                let url = format!("{}/secrets", self.base_url());
                let resp = self
                    .client
                    .post(&url)
                    .header("X-Auth-Token", &self.secret_key)
                    .json(&serde_json::json!({
                        "project_id": self.project_id,
                        "name": name,
                        "path": self.path,
                        "tags": ["rclone-ui"],
                    }))
                    .send()
                    .await
                    .context("Scaleway SM create secret")?;
                if !resp.status().is_success() {
                    let status = resp.status();
                    let body = resp.text().await.unwrap_or_default();
                    return Err(anyhow!("Scaleway SM create secret error {status}: {body}"));
                }
                #[derive(Deserialize)]
                struct CreateResponse {
                    id: String,
                }
                let parsed: CreateResponse = resp.json().await.context("Scaleway SM create parse")?;
                parsed.id
            }
        };

        // Étape 2 : ajouter une nouvelle version avec les données
        let url = format!("{}/secrets/{}/versions", self.base_url(), secret_id);
        let resp = self
            .client
            .post(&url)
            .header("X-Auth-Token", &self.secret_key)
            .json(&serde_json::json!({ "data": data_b64 }))
            .send()
            .await
            .context("Scaleway SM create version")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Scaleway SM new version error {status}: {body}"));
        }
        Ok(())
    }

    async fn delete(&self, remote_id: Uuid) -> anyhow::Result<()> {
        let name = Self::name_for(remote_id);
        let secret_id = match self.find_secret_id(&name).await? {
            Some(id) => id,
            None => return Ok(()),
        };
        let url = format!("{}/secrets/{}", self.base_url(), secret_id);
        let resp = self
            .client
            .delete(&url)
            .header("X-Auth-Token", &self.secret_key)
            .send()
            .await
            .context("Scaleway SM delete")?;
        if !resp.status().is_success() && resp.status().as_u16() != 404 {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Scaleway SM delete error {status}: {body}"));
        }
        Ok(())
    }
}
