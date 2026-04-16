use crate::{
    entities::{task, task_run},
    errors::{AppError, AppResult},
    models::task_run::TaskRunSummary,
    state::AppState,
};
use axum::{
    extract::{Path, State},
    Json,
};
use sea_orm::*;
use uuid::Uuid;

pub async fn list_for_task(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
) -> AppResult<Json<Vec<TaskRunSummary>>> {
    // Verify task exists
    if task::Entity::find_by_id(task_id).one(&state.db).await?.is_none() {
        return Err(AppError::NotFound(format!("Task {task_id} not found")));
    }

    let runs = task_run::Entity::find()
        .filter(task_run::Column::TaskId.eq(task_id))
        .order_by_desc(task_run::Column::StartedAt)
        .limit(100)
        .all(&state.db)
        .await?;

    let summaries = runs
        .into_iter()
        .map(|r| TaskRunSummary {
            id: r.id,
            task_id: r.task_id,
            triggered_by: r.triggered_by,
            status: r.status,
            started_at: r.started_at.into(),
            finished_at: r.finished_at.map(|t| t.into()),
            duration_ms: r.duration_ms,
            exit_code: r.exit_code,
            stats: r.stats,
        })
        .collect();

    Ok(Json(summaries))
}

pub async fn get_run(
    State(state): State<AppState>,
    Path(run_id): Path<Uuid>,
) -> AppResult<Json<task_run::Model>> {
    let run = task_run::Entity::find_by_id(run_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Run {run_id} not found")))?;
    Ok(Json(run))
}
