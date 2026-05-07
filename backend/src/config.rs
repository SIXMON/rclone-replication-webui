use std::env;

#[derive(Clone, Debug)]
pub enum SecretManagerConfig {
    Scaleway {
        secret_key: String,
        project_id: String,
        region: String,
        path: String,
    },
    AzureKeyVault {
        tenant_id: String,
        client_id: String,
        client_secret: String,
        vault_url: String,
    },
    Aws {
        region: String,
        prefix: String,
    },
    Vault {
        addr: String,
        token: String,
        mount_path: String,
        path_prefix: String,
    },
    Infisical {
        host: String,
        client_id: String,
        client_secret: String,
        project_id: String,
        environment: String,
        secret_path: String,
    },
    /// Google Cloud Secret Manager — l'auth utilise les credentials par défaut
    /// (variable d'env `GOOGLE_APPLICATION_CREDENTIALS`, gcloud, metadata server GCE/GKE...).
    GoogleCloud {
        project_id: String,
    },
    Doppler {
        token: String,
        project: String,
        config: String,
    },
}

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub bind_addr: String,
    pub rclone_bin: String,
    pub apprise_bin: String,
    /// Secret Manager configuration. None = secrets stockés en BDD (mode legacy).
    pub secret_manager: Option<SecretManagerConfig>,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let secret_manager = parse_secret_manager()?;

        Ok(Config {
            database_url: env::var("DATABASE_URL")
                .map_err(|_| anyhow::anyhow!("DATABASE_URL is required"))?,
            bind_addr: env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string()),
            rclone_bin: env::var("RCLONE_BIN").unwrap_or_else(|_| "rclone".to_string()),
            apprise_bin: env::var("APPRISE_BIN").unwrap_or_else(|_| "apprise".to_string()),
            secret_manager,
        })
    }
}

fn parse_secret_manager() -> anyhow::Result<Option<SecretManagerConfig>> {
    // Backward compatibility : SCW_SECRET_MANAGER_ENABLED=true → provider scaleway
    let provider = env::var("SECRET_MANAGER_PROVIDER")
        .ok()
        .map(|v| v.trim().to_ascii_lowercase())
        .or_else(|| {
            if env::var("SCW_SECRET_MANAGER_ENABLED")
                .ok()
                .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
                .unwrap_or(false)
            {
                Some("scaleway".to_string())
            } else {
                None
            }
        });

    let provider = match provider.as_deref() {
        None | Some("none") | Some("") => return Ok(None),
        Some(p) => p,
    };

    let cfg = match provider {
        "scaleway" => SecretManagerConfig::Scaleway {
            secret_key: required("SCW_SECRET_KEY")?,
            project_id: required("SCW_PROJECT_ID")?,
            region: env::var("SCW_DEFAULT_REGION").unwrap_or_else(|_| "fr-par".to_string()),
            path: env::var("SCW_SECRET_PATH").unwrap_or_else(|_| "/rclone-ui".to_string()),
        },
        "azure" | "azure-keyvault" | "azure_keyvault" => SecretManagerConfig::AzureKeyVault {
            tenant_id: required("AZURE_TENANT_ID")?,
            client_id: required("AZURE_CLIENT_ID")?,
            client_secret: required("AZURE_CLIENT_SECRET")?,
            vault_url: required("AZURE_VAULT_URL")?,
        },
        "aws" | "aws-secrets-manager" => SecretManagerConfig::Aws {
            region: env::var("AWS_REGION")
                .or_else(|_| env::var("AWS_DEFAULT_REGION"))
                .unwrap_or_else(|_| "eu-west-1".to_string()),
            prefix: env::var("AWS_SECRET_PREFIX").unwrap_or_else(|_| "rclone-ui/".to_string()),
        },
        "vault" | "hashicorp-vault" => SecretManagerConfig::Vault {
            addr: required("VAULT_ADDR")?,
            token: required("VAULT_TOKEN")?,
            mount_path: env::var("VAULT_MOUNT_PATH").unwrap_or_else(|_| "secret".to_string()),
            path_prefix: env::var("VAULT_PATH_PREFIX").unwrap_or_else(|_| "rclone-ui".to_string()),
        },
        "infisical" => SecretManagerConfig::Infisical {
            host: env::var("INFISICAL_HOST").unwrap_or_else(|_| "https://app.infisical.com".to_string()),
            client_id: required("INFISICAL_CLIENT_ID")?,
            client_secret: required("INFISICAL_CLIENT_SECRET")?,
            project_id: required("INFISICAL_PROJECT_ID")?,
            environment: env::var("INFISICAL_ENVIRONMENT").unwrap_or_else(|_| "prod".to_string()),
            secret_path: env::var("INFISICAL_SECRET_PATH").unwrap_or_else(|_| "/rclone-ui".to_string()),
        },
        "gcp" | "google" | "google-cloud" => SecretManagerConfig::GoogleCloud {
            project_id: required("GCP_PROJECT_ID")?,
        },
        "doppler" => SecretManagerConfig::Doppler {
            token: required("DOPPLER_TOKEN")?,
            project: required("DOPPLER_PROJECT")?,
            config: env::var("DOPPLER_CONFIG").unwrap_or_else(|_| "prd".to_string()),
        },
        other => {
            return Err(anyhow::anyhow!(
                "Unknown SECRET_MANAGER_PROVIDER: '{other}'. Supported: scaleway, azure, aws, vault, infisical, gcp, doppler"
            ));
        }
    };

    Ok(Some(cfg))
}

fn required(key: &str) -> anyhow::Result<String> {
    env::var(key).map_err(|_| anyhow::anyhow!("{key} is required when SECRET_MANAGER_PROVIDER is enabled"))
}
