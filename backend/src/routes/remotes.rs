use crate::{
    entities::{remote, task},
    errors::{AppError, AppResult},
    models::remote::{CreateRemoteRequest, RemoteWithTaskCount, UpdateRemoteRequest},
    services::rclone,
    state::AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sea_orm::*;
use serde_json::json;
use uuid::Uuid;

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

        result.push(RemoteWithTaskCount {
            id: r.id,
            name: r.name,
            remote_type: r.remote_type,
            config: r.config,
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
    let model = remote::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(req.name),
        remote_type: Set(req.remote_type),
        config: Set(req.config),
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
    let remote = remote::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Database(e.into()))?
        .ok_or_else(|| AppError::NotFound(format!("Remote {id} not found")))?;
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

    let mut model: remote::ActiveModel = existing.into();
    model.name = Set(req.name);
    model.remote_type = Set(req.remote_type);
    model.config = Set(req.config);
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

    // Convert SeaORM models to the format rclone service expects
    let rclone_remotes: Vec<crate::models::remote::RcloneRemote> = all_remotes
        .iter()
        .map(|r| crate::models::remote::RcloneRemote {
            id: r.id,
            name: r.name.clone(),
            remote_type: r.remote_type.clone(),
            config: r.config.clone(),
        })
        .collect();

    match rclone::test_remote(&state.config.rclone_bin, &rclone_remotes, &remote.name).await {
        Ok(output) => Ok(Json(json!({"success": true, "message": output}))),
        Err(e) => Ok(Json(json!({"success": false, "message": e.to_string()}))),
    }
}
