use async_trait::async_trait;
use std::collections::HashMap;
use uuid::Uuid;

/// Abstraction pour le stockage des credentials sensibles.
///
/// Une "secret" représente l'ensemble des champs sensibles d'un remote, sérialisé en JSON.
/// La clé est l'UUID du remote.
#[async_trait]
pub trait SecretStore: Send + Sync {
    /// Indique si l'implémentation stocke réellement les secrets dans un service externe.
    fn is_active(&self) -> bool;

    /// Récupère les champs sensibles d'un remote. Retourne `None` si aucun secret n'existe.
    async fn get(&self, remote_id: Uuid) -> anyhow::Result<Option<HashMap<String, String>>>;

    /// Crée ou met à jour les champs sensibles d'un remote.
    async fn put(&self, remote_id: Uuid, fields: HashMap<String, String>) -> anyhow::Result<()>;

    /// Supprime tous les secrets associés à un remote.
    async fn delete(&self, remote_id: Uuid) -> anyhow::Result<()>;
}
