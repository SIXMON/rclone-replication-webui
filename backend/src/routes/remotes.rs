use crate::{
    entities::{remote, task},
    errors::{AppError, AppResult},
    models::remote::{CreateRemoteRequest, RemoteWithTaskCount, UpdateRemoteRequest},
    services::{rclone, secrets::sensitive::split_sensitive},
    state::AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sea_orm::*;
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

/// Renvoie la liste des champs sensibles présents pour ce remote, masqués ("").
/// Permet au frontend de savoir qu'un secret est stocké sans en révéler la valeur.
async fn mask_with_stored_keys(
    state: &AppState,
    id: Uuid,
    config: &mut serde_json::Value,
) -> AppResult<()> {
    if !state.secret_store.is_active() {
        return Ok(());
    }
    if let Some(stored) = state
        .secret_store
        .get(id)
        .await
        .map_err(AppError::Internal)?
    {
        if let Some(obj) = config.as_object_mut() {
            for k in stored.keys() {
                obj.insert(k.clone(), json!(""));
            }
        }
    }
    Ok(())
}

pub async fn list(State(state): State<AppState>) -> AppResult<Json<Vec<RemoteWithTaskCount>>> {
    let remotes = remote::Entity::find()
        .order_by_asc(remote::Column::Name)
        .all(&state.db)
        .await
        .map_err(|e| AppError::Database(e.into()))?;

    let mut result = Vec::with_capacity(remotes.len());
    for r in remotes {
        let count = task::Entity::find()
            .filter(
                Condition::any()
                    .add(task::Column::SourceRemoteId.eq(r.id))
                    .add(task::Column::DestRemoteId.eq(r.id)),
            )
            .count(&state.db)
            .await
            .map_err(|e| AppError::Database(e.into()))? as i64;

        let mut config = r.config.clone();
        mask_with_stored_keys(&state, r.id, &mut config).await?;

        result.push(RemoteWithTaskCount {
            id: r.id,
            name: r.name,
            remote_type: r.remote_type,
            config,
            created_at: r.created_at.into(),
            updated_at: r.updated_at.into(),
            task_count: count,
        });
    }
    Ok(Json(result))
}

pub async fn create(
    State(state): State<AppState>,
    Json(req): Json<CreateRemoteRequest>,
) -> AppResult<(StatusCode, Json<remote::Model>)> {
    if req.name.is_empty() {
        return Err(AppError::BadRequest("name is required".into()));
    }

    let new_id = Uuid::new_v4();

    // Si le SecretStore est actif, on sépare les champs sensibles
    let public_config = if state.secret_store.is_active() {
        if let Some(obj) = req.config.as_object() {
            let (sensitive, public_cfg) = split_sensitive(&req.remote_type, obj);
            if !sensitive.is_empty() {
                state
                    .secret_store
                    .put(new_id, sensitive)
                    .await
                    .map_err(AppError::Internal)?;
            }
            serde_json::Value::Object(public_cfg)
        } else {
            req.config
        }
    } else {
        req.config
    };

    let model = remote::ActiveModel {
        id: Set(new_id),
        name: Set(req.name),
        remote_type: Set(req.remote_type),
        config: Set(public_config),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };
    let result = model.insert(&state.db).await.map_err(|e| AppError::Database(e.into()))?;
    Ok((StatusCode::CREATED, Json(result)))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<remote::Model>> {
    let mut remote = remote::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Database(e.into()))?
        .ok_or_else(|| AppError::NotFound(format!("Remote {id} not found")))?;

    mask_with_stored_keys(&state, id, &mut remote.config).await?;
    Ok(Json(remote))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateRemoteRequest>,
) -> AppResult<Json<remote::Model>> {
    let existing = remote::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Database(e.into()))?
        .ok_or_else(|| AppError::NotFound(format!("Remote {id} not found")))?;

    // Si le SecretStore est actif, fusionner les secrets : un champ vide signifie "ne pas modifier"
    let public_config = if state.secret_store.is_active() {
        if let Some(obj) = req.config.as_object() {
            let (mut new_sensitive, public_cfg) = split_sensitive(&req.remote_type, obj);

            // Récupérer les secrets existants
            let existing_secrets = state
                .secret_store
                .get(id)
                .await
                .map_err(AppError::Internal)?
                .unwrap_or_default();

            // Pour chaque champ sensible attendu mais vide/manquant dans la requête,
            // réutiliser la valeur existante
            let mut merged: HashMap<String, String> = existing_secrets.clone();
            for (k, v) in new_sensitive.drain() {
                merged.insert(k, v);
            }

            if !merged.is_empty() {
                state
                    .secret_store
                    .put(id, merged)
                    .await
                    .map_err(AppError::Internal)?;
            } else if !existing_secrets.is_empty() {
                // Si tous les champs sensibles ont été retirés, supprimer le secret
                state
                    .secret_store
                    .delete(id)
                    .await
                    .map_err(AppError::Internal)?;
            }
            serde_json::Value::Object(public_cfg)
        } else {
            req.config
        }
    } else {
        req.config
    };

    let mut model: remote::ActiveModel = existing.into();
    model.name = Set(req.name);
    model.remote_type = Set(req.remote_type);
    model.config = Set(public_config);
    model.updated_at = Set(chrono::Utc::now().into());

    let result = model.update(&state.db).await.map_err(|e| AppError::Database(e.into()))?;
    Ok(Json(result))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    let count = task::Entity::find()
        .filter(
            Condition::any()
                .add(task::Column::SourceRemoteId.eq(id))
                .add(task::Column::DestRemoteId.eq(id)),
        )
        .count(&state.db)
        .await
        .map_err(|e| AppError::Database(e.into()))?;

    if count > 0 {
        return Err(AppError::Conflict(
            "Cannot delete remote: it is used by one or more tasks".into(),
        ));
    }

    let result = remote::Entity::delete_by_id(id)
        .exec(&state.db)
        .await
        .map_err(|e| AppError::Database(e.into()))?;

    if result.rows_affected == 0 {
        return Err(AppError::NotFound(format!("Remote {id} not found")));
    }

    // Suppression du secret associé (best-effort)
    if let Err(e) = state.secret_store.delete(id).await {
        tracing::warn!("Secret store: failed to delete secret for {id}: {e}");
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn test_connectivity(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let remote = remote::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Database(e.into()))?
        .ok_or_else(|| AppError::NotFound(format!("Remote {id} not found")))?;

    let all_remotes = remote::Entity::find()
        .all(&state.db)
        .await
        .map_err(|e| AppError::Database(e.into()))?;

    // Charger les secrets et fusionner avec les configs publiques
    let mut rclone_remotes = Vec::with_capacity(all_remotes.len());
    for r in all_remotes {
        let mut config = r.config.clone();
        if state.secret_store.is_active() {
            if let Some(stored) = state
                .secret_store
                .get(r.id)
                .await
                .map_err(AppError::Internal)?
            {
                if let Some(obj) = config.as_object_mut() {
                    for (k, v) in stored {
                        obj.insert(k, json!(v));
                    }
                }
            }
        }
        rclone_remotes.push(crate::models::remote::RcloneRemote {
            id: r.id,
            name: r.name,
            remote_type: r.remote_type,
            config,
        });
    }

    match rclone::test_remote(&state.config.rclone_bin, &rclone_remotes, &remote.name).await {
        Ok(output) => Ok(Json(json!({"success": true, "message": output}))),
        Err(e) => Ok(Json(json!({"success": false, "message": e.to_string()}))),
    }
}
