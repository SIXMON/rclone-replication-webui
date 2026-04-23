use crate::{
    entities::{remote, task, task_run},
    errors::{AppError, AppResult},
    models::task::{CreateTaskRequest, LastRunSummary, PatchTaskRequest, TaskWithMeta},
    services::{scheduler, task_executor},
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

pub async fn list(State(state): State<AppState>) -> AppResult<Json<Vec<TaskWithMeta>>> {
    let tasks = task::Entity::find()
        .order_by_asc(task::Column::Name)
        .all(&state.db)
        .await?;

    let mut result = Vec::with_capacity(tasks.len());
    for t in tasks {
        // Fetch remote names
        let src_name = remote::Entity::find_by_id(t.source_remote_id)
            .one(&state.db)
            .await?
            .map(|r| r.name)
            .unwrap_or_default();
        let dst_name = remote::Entity::find_by_id(t.dest_remote_id)
            .one(&state.db)
            .await?
            .map(|r| r.name)
            .unwrap_or_default();

        // Fetch last run
        let last_run = task_run::Entity::find()
            .filter(task_run::Column::TaskId.eq(t.id))
            .order_by_desc(task_run::Column::StartedAt)
            .one(&state.db)
            .await?
            .map(|r| LastRunSummary {
                status: r.status,
                started_at: r.started_at.into(),
                duration_ms: r.duration_ms,
            });

        let running = state.running_tasks.contains_key(&t.id);

        result.push(TaskWithMeta {
            id: t.id,
            name: t.name,
            source_remote_id: t.source_remote_id,
            source_remote_name: src_name,
            source_path: t.source_path,
            dest_remote_id: t.dest_remote_id,
            dest_remote_name: dst_name,
            dest_path: t.dest_path,
            cron_expression: t.cron_expression,
            enabled: t.enabled,
            rclone_flags: t.rclone_flags,
            notification_channel_id: t.notification_channel_id,
            notify_on: t.notify_on,
            max_retries: t.max_retries,
            retry_delay_seconds: t.retry_delay_seconds,
            last_run,
            running,
            created_at: t.created_at.into(),
            updated_at: t.updated_at.into(),
        });
    }

    Ok(Json(result))
}

pub async fn create(
    State(state): State<AppState>,
    Json(req): Json<CreateTaskRequest>,
) -> AppResult<(StatusCode, Json<task::Model>)> {
    if req.name.is_empty() {
        return Err(AppError::BadRequest("name is required".into()));
    }
    // Validate remotes exist
    for remote_id in [req.source_remote_id, req.dest_remote_id] {
        if remote::Entity::find_by_id(remote_id).one(&state.db).await?.is_none() {
            return Err(AppError::NotFound(format!("Remote {remote_id} not found")));
        }
    }

    let model = task::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(req.name),
        source_remote_id: Set(req.source_remote_id),
        source_path: Set(req.source_path),
        dest_remote_id: Set(req.dest_remote_id),
        dest_path: Set(req.dest_path),
        cron_expression: Set(req.cron_expression),
        enabled: Set(req.enabled),
        rclone_flags: Set(req.rclone_flags),
        notification_channel_id: Set(req.notification_channel_id),
        notify_on: Set(req.notify_on),
        max_retries: Set(req.max_retries),
        retry_delay_seconds: Set(req.retry_delay_seconds),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };

    let result = model.insert(&state.db).await?;

    if result.cron_expression.is_some() && result.enabled {
        scheduler::rebuild_scheduler(state.clone()).await;
    }

    Ok((StatusCode::CREATED, Json(result)))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<task::Model>> {
    let task = task::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Task {id} not found")))?;
    Ok(Json(task))
}

pub async fn patch(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<PatchTaskRequest>,
) -> AppResult<Json<task::Model>> {
    let existing = task::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Task {id} not found")))?;

    let mut model: task::ActiveModel = existing.clone().into();
    model.name = Set(req.name.unwrap_or(existing.name));
    model.cron_expression = Set(req.cron_expression.unwrap_or(existing.cron_expression));
    model.enabled = Set(req.enabled.unwrap_or(existing.enabled));
    model.rclone_flags = Set(req.rclone_flags.unwrap_or(existing.rclone_flags));
    model.notification_channel_id = Set(req.notification_channel_id.unwrap_or(existing.notification_channel_id));
    model.notify_on = Set(req.notify_on.unwrap_or(existing.notify_on));
    model.max_retries = Set(req.max_retries.unwrap_or(existing.max_retries));
    model.retry_delay_seconds = Set(req.retry_delay_seconds.unwrap_or(existing.retry_delay_seconds));
    model.updated_at = Set(chrono::Utc::now().into());

    let result = model.update(&state.db).await?;

    scheduler::rebuild_scheduler(state.clone()).await;
    Ok(Json(result))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    let result = task::Entity::delete_by_id(id)
        .exec(&state.db)
        .await?;

    if result.rows_affected == 0 {
        return Err(AppError::NotFound(format!("Task {id} not found")));
    }
    scheduler::rebuild_scheduler(state.clone()).await;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn trigger(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let run_id = task_executor::spawn_task(state, id, task_executor::ExecutionMode::Manual).await?;
    Ok(Json(json!({"run_id": run_id, "message": "Task started"})))
}

pub async fn restore(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let run_id = task_executor::spawn_task(state, id, task_executor::ExecutionMode::Restore).await?;
    Ok(Json(json!({"run_id": run_id, "message": "Restore started"})))
}

pub async fn status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    if let Some(running) = state.running_tasks.get(&id) {
        Ok(Json(json!({
            "running": true,
            "run_id": running.run_id,
            "triggered_by": running.triggered_by,
            "started_at": running.started_at,
        })))
    } else {
        Ok(Json(json!({"running": false})))
    }
}
