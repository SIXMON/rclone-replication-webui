use super::SecretStore;
use async_trait::async_trait;
use std::collections::HashMap;
use uuid::Uuid;

/// Implémentation no-op : aucun stockage externe, tout reste en BDD.
pub struct NoopSecretStore;

#[async_trait]
impl SecretStore for NoopSecretStore {
    fn is_active(&self) -> bool {
        false
    }

    async fn get(&self, _remote_id: Uuid) -> anyhow::Result<Option<HashMap<String, String>>> {
        Ok(None)
    }

    async fn put(&self, _remote_id: Uuid, _fields: HashMap<String, String>) -> anyhow::Result<()> {
        Ok(())
    }

    async fn delete(&self, _remote_id: Uuid) -> anyhow::Result<()> {
        Ok(())
    }
}
