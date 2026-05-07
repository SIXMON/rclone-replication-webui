use super::SecretStore;
use anyhow::Context;
use async_trait::async_trait;
use aws_sdk_secretsmanager::Client as AwsClient;
use std::collections::HashMap;
use uuid::Uuid;

/// Implémentation du stockage de secrets via AWS Secrets Manager.
///
/// Convention : 1 secret par remote, nommé `<prefix><remote_id>`.
/// Le SDK AWS gère automatiquement l'auth (variables d'env, IAM role, IRSA, etc.).
pub struct AwsSecretStore {
    client: AwsClient,
    prefix: String,
}

impl AwsSecretStore {
    pub async fn new(region: String, prefix: String) -> anyhow::Result<Self> {
        let region_provider = aws_config::Region::new(region);
        let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(region_provider)
            .load()
            .await;
        Ok(Self {
            client: AwsClient::new(&config),
            prefix,
        })
    }

    fn secret_id(&self, remote_id: Uuid) -> String {
        format!("{}{}", self.prefix, remote_id)
    }
}

#[async_trait]
impl SecretStore for AwsSecretStore {
    fn is_active(&self) -> bool {
        true
    }

    async fn get(&self, remote_id: Uuid) -> anyhow::Result<Option<HashMap<String, String>>> {
        let result = self
            .client
            .get_secret_value()
            .secret_id(self.secret_id(remote_id))
            .send()
            .await;

        match result {
            Ok(out) => match out.secret_string() {
                Some(s) if !s.is_empty() => {
                    let map: HashMap<String, String> =
                        serde_json::from_str(s).context("parse AWS secret JSON")?;
                    Ok(Some(map))
                }
                _ => Ok(None),
            },
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("ResourceNotFoundException") {
                    return Ok(None);
                }
                Err(anyhow::anyhow!("AWS get secret error: {msg}"))
            }
        }
    }

    async fn put(&self, remote_id: Uuid, fields: HashMap<String, String>) -> anyhow::Result<()> {
        let value = serde_json::to_string(&fields).context("serialize AWS secret JSON")?;
        let secret_id = self.secret_id(remote_id);

        // Tenter PutSecretValue (si existe). Sinon CreateSecret.
        let put_result = self
            .client
            .put_secret_value()
            .secret_id(&secret_id)
            .secret_string(&value)
            .send()
            .await;

        match put_result {
            Ok(_) => Ok(()),
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("ResourceNotFoundException") {
                    self.client
                        .create_secret()
                        .name(&secret_id)
                        .secret_string(&value)
                        .description("Managed by rclone-ui")
                        .send()
                        .await
                        .map_err(|err| anyhow::anyhow!("AWS create secret error: {err}"))?;
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("AWS put secret error: {msg}"))
                }
            }
        }
    }

    async fn delete(&self, remote_id: Uuid) -> anyhow::Result<()> {
        let result = self
            .client
            .delete_secret()
            .secret_id(self.secret_id(remote_id))
            .force_delete_without_recovery(true)
            .send()
            .await;
        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("ResourceNotFoundException") {
                    return Ok(());
                }
                Err(anyhow::anyhow!("AWS delete secret error: {msg}"))
            }
        }
    }
}
