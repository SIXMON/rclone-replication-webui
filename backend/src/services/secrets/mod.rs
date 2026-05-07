pub mod migration;
pub mod sensitive;
mod store;
mod noop;
mod scaleway;
mod azure;
mod aws;
mod vault;
mod infisical;
mod doppler;
mod gcp;

pub use store::SecretStore;
pub use noop::NoopSecretStore;

use crate::config::SecretManagerConfig;
use std::sync::Arc;

/// Construit l'implémentation de SecretStore selon la configuration.
pub async fn build(cfg: Option<&SecretManagerConfig>) -> Arc<dyn SecretStore> {
    let Some(cfg) = cfg else {
        tracing::info!("Secret Manager : désactivé — credentials stockés en BDD");
        return Arc::new(NoopSecretStore);
    };

    match cfg {
        SecretManagerConfig::Scaleway { secret_key, project_id, region, path } => {
            tracing::info!("Secret Manager : Scaleway (région {}, projet {})", region, project_id);
            Arc::new(scaleway::ScalewaySecretStore::new(
                secret_key.clone(),
                project_id.clone(),
                region.clone(),
                path.clone(),
            ))
        }
        SecretManagerConfig::AzureKeyVault { tenant_id, client_id, client_secret, vault_url } => {
            tracing::info!("Secret Manager : Azure Key Vault ({})", vault_url);
            Arc::new(azure::AzureKeyVaultSecretStore::new(
                tenant_id.clone(),
                client_id.clone(),
                client_secret.clone(),
                vault_url.clone(),
            ))
        }
        SecretManagerConfig::Aws { region, prefix } => {
            tracing::info!("Secret Manager : AWS Secrets Manager (région {})", region);
            match aws::AwsSecretStore::new(region.clone(), prefix.clone()).await {
                Ok(s) => Arc::new(s),
                Err(e) => {
                    tracing::error!("Échec de l'initialisation AWS Secrets Manager : {e}. Fallback vers BDD.");
                    Arc::new(NoopSecretStore)
                }
            }
        }
        SecretManagerConfig::Vault { addr, token, mount_path, path_prefix } => {
            tracing::info!("Secret Manager : HashiCorp Vault ({})", addr);
            Arc::new(vault::VaultSecretStore::new(
                addr.clone(),
                token.clone(),
                mount_path.clone(),
                path_prefix.clone(),
            ))
        }
        SecretManagerConfig::Infisical { host, client_id, client_secret, project_id, environment, secret_path } => {
            tracing::info!("Secret Manager : Infisical ({})", host);
            Arc::new(infisical::InfisicalSecretStore::new(
                host.clone(),
                client_id.clone(),
                client_secret.clone(),
                project_id.clone(),
                environment.clone(),
                secret_path.clone(),
            ))
        }
        SecretManagerConfig::GoogleCloud { project_id } => {
            tracing::info!("Secret Manager : Google Cloud Secret Manager (projet {})", project_id);
            match gcp::GoogleCloudSecretStore::new(project_id.clone()).await {
                Ok(s) => Arc::new(s),
                Err(e) => {
                    tracing::error!("Échec de l'initialisation GCP Secret Manager : {e}. Fallback vers BDD.");
                    Arc::new(NoopSecretStore)
                }
            }
        }
        SecretManagerConfig::Doppler { token, project, config } => {
            tracing::info!("Secret Manager : Doppler (projet {}, config {})", project, config);
            Arc::new(doppler::DopplerSecretStore::new(
                token.clone(),
                project.clone(),
                config.clone(),
            ))
        }
    }
}
