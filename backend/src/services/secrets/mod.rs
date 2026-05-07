pub mod migration;
pub mod sensitive;
mod store;
mod noop;
mod scaleway;

pub use store::SecretStore;
pub use noop::NoopSecretStore;
pub use scaleway::ScalewaySecretStore;

use crate::config::ScalewayConfig;
use std::sync::Arc;

/// Construit l'implémentation de SecretStore selon la configuration.
pub fn build(scaleway: Option<&ScalewayConfig>) -> Arc<dyn SecretStore> {
    match scaleway {
        Some(cfg) => {
            tracing::info!("Secret Manager : Scaleway activé (région {}, projet {})", cfg.region, cfg.project_id);
            Arc::new(ScalewaySecretStore::new(cfg.clone()))
        }
        None => {
            tracing::info!("Secret Manager : désactivé — credentials stockés en BDD");
            Arc::new(NoopSecretStore)
        }
    }
}
