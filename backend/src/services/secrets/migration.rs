use crate::entities::remote;
use crate::services::secrets::sensitive::split_sensitive;
use crate::state::AppState;
use sea_orm::*;
use serde_json::Value;

/// Au démarrage, déplace les champs sensibles encore stockés dans la BDD vers le SecretStore actif.
///
/// Idempotent : si tous les champs sensibles d'un remote sont déjà absents de la config publique,
/// rien n'est fait. Si certains champs sensibles sont encore en BDD, ils sont migrés puis retirés.
pub async fn migrate_secrets_from_db(state: &AppState) -> anyhow::Result<()> {
    if !state.secret_store.is_active() {
        return Ok(());
    }

    let remotes = remote::Entity::find().all(&state.db).await?;
    let mut migrated_count = 0usize;

    for r in remotes {
        let Some(obj) = r.config.as_object() else {
            continue;
        };

        let (sensitive, public_cfg) = split_sensitive(&r.remote_type, obj);
        if sensitive.is_empty() {
            // Rien à migrer pour ce remote
            continue;
        }

        // Récupérer ce qui est déjà dans le SecretStore et fusionner
        // (les valeurs en BDD priment au cas où elles auraient été modifiées plus récemment)
        let mut merged = state.secret_store.get(r.id).await?.unwrap_or_default();
        for (k, v) in sensitive {
            merged.insert(k, v);
        }

        state.secret_store.put(r.id, merged).await?;

        // Mettre à jour la config en BDD : retirer les champs sensibles
        let mut model: remote::ActiveModel = r.clone().into();
        model.config = Set(Value::Object(public_cfg));
        model.updated_at = Set(chrono::Utc::now().into());
        model.update(&state.db).await?;

        migrated_count += 1;
        tracing::info!(
            "Secret Manager : secrets migr\u{00e9}s pour le remote '{}' ({})",
            r.name,
            r.id
        );
    }

    if migrated_count > 0 {
        tracing::info!(
            "Secret Manager : migration termin\u{00e9}e ({} remote(s) migr\u{00e9}s)",
            migrated_count
        );
    } else {
        tracing::info!("Secret Manager : aucun secret \u{00e0} migrer");
    }

    Ok(())
}
