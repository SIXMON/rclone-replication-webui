use crate::{
    entities::{notification_channel, task},
    errors::{AppError, AppResult},
    models::notification::{ChannelWithTaskCount, CreateChannelRequest, UpdateChannelRequest},
    services::apprise,
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

pub async fn list(
    State(state): State<AppState>,
) -> AppResult<Json<Vec<ChannelWithTaskCount>>> {
    let channels = notification_channel::Entity::find()
        .order_by_asc(notification_channel::Column::Name)
        .all(&state.db)
        .await
        .map_err(|e| AppError::Database(e.into()))?;

    let mut result = Vec::with_capacity(channels.len());
    for ch in channels {
        let count = task::Entity::find()
            .filter(task::Column::NotificationChannelId.eq(ch.id))
            .count(&state.db)
            .await
            .map_err(|e| AppError::Database(e.into()))? as i64;

        result.push(ChannelWithTaskCount {
            id: ch.id,
            name: ch.name,
            apprise_url: ch.apprise_url,
            enabled: ch.enabled,
            created_at: ch.created_at.into(),
            updated_at: ch.updated_at.into(),
            task_count: count,
        });
    }
    Ok(Json(result))
}

pub async fn create(
    State(state): State<AppState>,
    Json(req): Json<CreateChannelRequest>,
) -> AppResult<(StatusCode, Json<notification_channel::Model>)> {
    if req.name.is_empty() || req.apprise_url.is_empty() {
        return Err(AppError::BadRequest("name and apprise_url are required".into()));
    }
    let model = notification_channel::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(req.name),
        apprise_url: Set(req.apprise_url),
        enabled: Set(req.enabled),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };
    let result = model.insert(&state.db).await.map_err(|e| AppError::Database(e.into()))?;
    Ok((StatusCode::CREATED, Json(result)))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<notification_channel::Model>> {
    let channel = notification_channel::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Database(e.into()))?
        .ok_or_else(|| AppError::NotFound(format!("Channel {id} not found")))?;
    Ok(Json(channel))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateChannelRequest>,
) -> AppResult<Json<notification_channel::Model>> {
    let existing = notification_channel::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Database(e.into()))?
        .ok_or_else(|| AppError::NotFound(format!("Channel {id} not found")))?;

    let mut model: notification_channel::ActiveModel = existing.into();
    model.name = Set(req.name);
    model.apprise_url = Set(req.apprise_url);
    model.enabled = Set(req.enabled);
    model.updated_at = Set(chrono::Utc::now().into());

    let result = model.update(&state.db).await.map_err(|e| AppError::Database(e.into()))?;
    Ok(Json(result))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    let count = task::Entity::find()
        .filter(task::Column::NotificationChannelId.eq(id))
        .count(&state.db)
        .await
        .map_err(|e| AppError::Database(e.into()))?;

    if count > 0 {
        return Err(AppError::Conflict(
            "Cannot delete channel: it is used by one or more tasks".into(),
        ));
    }

    let result = notification_channel::Entity::delete_by_id(id)
        .exec(&state.db)
        .await
        .map_err(|e| AppError::Database(e.into()))?;

    if result.rows_affected == 0 {
        return Err(AppError::NotFound(format!("Channel {id} not found")));
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn test_notification(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let channel = notification_channel::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Database(e.into()))?
        .ok_or_else(|| AppError::NotFound(format!("Channel {id} not found")))?;

    match apprise::send_notification(
        &state.config.apprise_bin,
        &[channel.apprise_url],
        "Test — rclone-ui",
        "Ce message confirme que le canal de notification **fonctionne correctement**.\n\n_Envoyé depuis rclone-ui._",
    )
    .await
    {
        Ok(_) => Ok(Json(json!({"success": true, "message": "Test notification sent"}))),
        Err(e) => Ok(Json(json!({"success": false, "message": e.to_string()}))),
    }
}
