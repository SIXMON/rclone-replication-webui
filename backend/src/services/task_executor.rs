use crate::{
    entities::{notification_channel, remote, task, task_run},
    errors::{AppError, AppResult},
    models::remote::RcloneRemote,
    services::{apprise, rclone},
    sse::broadcaster::SseEvent,
    sse::global::GlobalEvent,
    state::{AppState, RunningTask},
};
use chrono::Utc;
use sea_orm::*;
use sea_orm::sea_query::Expr;
use uuid::Uuid;

#[derive(Debug, Clone, Copy)]
pub enum ExecutionMode {
    Normal,
    Restore,
}

impl ExecutionMode {
    pub fn label(&self) -> &'static str {
        match self {
            ExecutionMode::Normal => "manual",
            ExecutionMode::Restore => "restore",
        }
    }
}

struct TaskSnapshot {
    source_remote_id: Uuid,
    source_path: String,
    dest_remote_id: Uuid,
    dest_path: String,
    rclone_flags: Vec<String>,
    notification_channel_id: Option<Uuid>,
    notify_on: Vec<String>,
    max_retries: i32,
    retry_delay_seconds: i32,
}

pub async fn spawn_task(
    state: AppState,
    task_id: Uuid,
    mode: ExecutionMode,
) -> AppResult<Uuid> {
    if state.running_tasks.contains_key(&task_id) {
        return Err(AppError::AlreadyRunning);
    }

    let t = task::Entity::find_by_id(task_id)
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound(format!("Task {task_id} not found")))?;

    let snapshot = TaskSnapshot {
        source_remote_id: t.source_remote_id,
        source_path: t.source_path,
        dest_remote_id: t.dest_remote_id,
        dest_path: t.dest_path,
        rclone_flags: t.rclone_flags,
        notification_channel_id: t.notification_channel_id,
        notify_on: t.notify_on,
        max_retries: t.max_retries,
        retry_delay_seconds: t.retry_delay_seconds,
    };

    let run = task_run::ActiveModel {
        id: Set(Uuid::new_v4()),
        task_id: Set(task_id),
        triggered_by: Set(mode.label().to_string()),
        status: Set("running".to_string()),
        started_at: Set(Utc::now().into()),
        ..Default::default()
    };
    let run_model = run.insert(&state.db).await.map_err(AppError::Database)?;
    let run_id = run_model.id;

    state.running_tasks.insert(
        task_id,
        RunningTask {
            run_id,
            triggered_by: mode.label().to_string(),
            started_at: Utc::now(),
        },
    );

    state.global_broadcaster.publish(GlobalEvent::TaskStarted { task_id });

    tokio::spawn(async move {
        execute_task_background(state, task_id, run_id, snapshot, mode).await;
    });

    Ok(run_id)
}

async fn execute_task_background(
    state: AppState,
    task_id: Uuid,
    run_id: Uuid,
    snapshot: TaskSnapshot,
    mode: ExecutionMode,
) {
    let started_at = Utc::now();

    let remotes = match load_remotes(&state).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Failed to load remotes for task {task_id}: {e}");
            finish_run(&state, task_id, run_id, started_at, 1, format!("Error: {e}"), None).await;
            return;
        }
    };

    let (src_remote_id, src_path, dst_remote_id, dst_path) = match mode {
        ExecutionMode::Normal => (
            snapshot.source_remote_id,
            snapshot.source_path.as_str(),
            snapshot.dest_remote_id,
            snapshot.dest_path.as_str(),
        ),
        ExecutionMode::Restore => (
            snapshot.dest_remote_id,
            snapshot.dest_path.as_str(),
            snapshot.source_remote_id,
            snapshot.source_path.as_str(),
        ),
    };

    let find_remote = |id: Uuid| -> Option<&RcloneRemote> {
        remotes.iter().find(|r| r.id == id)
    };

    let build_rclone_path = |remote: &RcloneRemote, task_path: &str| -> String {
        let root = remote
            .config
            .as_object()
            .and_then(|o| o.get("root"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let full_path = if root.is_empty() {
            task_path.to_string()
        } else {
            format!(
                "{}/{}",
                root.trim_end_matches('/'),
                task_path.trim_start_matches('/')
            )
        };
        format!("{}:{}", remote.name, full_path)
    };

    let src_remote = find_remote(src_remote_id);
    let dst_remote = find_remote(dst_remote_id);

    let (src, dst) = match (src_remote, dst_remote) {
        (Some(sr), Some(dr)) => (build_rclone_path(sr, src_path), build_rclone_path(dr, dst_path)),
        _ => {
            let msg = "Source or destination remote not found".to_string();
            tracing::error!("{msg} for task {task_id}");
            finish_run(&state, task_id, run_id, started_at, 1, msg, None).await;
            return;
        }
    };

    let max_attempts = (snapshot.max_retries + 1).max(1) as u32; // au moins 1 tentative
    let base_delay = snapshot.retry_delay_seconds.max(1) as u64;
    let mut log_buffer = String::new();
    let mut exit_code = 1i32;
    let mut stats: Option<serde_json::Value> = None;
    let broadcaster = state.sse_broadcaster.clone();

    for attempt in 1..=max_attempts {
        if attempt > 1 {
            let delay_secs = base_delay * (attempt as u64 - 1);
            let msg = format!(
                "--- Tentative {attempt}/{max_attempts} dans {delay_secs}s ---"
            );
            tracing::info!("Task {task_id}: {msg}");
            broadcaster.publish(task_id, SseEvent::Log(msg.clone()));
            log_buffer.push_str(&msg);
            log_buffer.push('\n');
            tokio::time::sleep(std::time::Duration::from_secs(delay_secs)).await;
        }

        tracing::info!(
            "Starting rclone sync: {src} -> {dst} (run {run_id}, attempt {attempt}/{max_attempts})"
        );

        let process = match rclone::spawn_sync(
            &state.config.rclone_bin,
            &remotes,
            &src,
            &dst,
            &snapshot.rclone_flags,
        )
        .await
        {
            Ok(p) => p,
            Err(e) => {
                let msg = format!("Error: {e}");
                tracing::error!("Failed to spawn rclone for task {task_id}: {e}");
                log_buffer.push_str(&msg);
                log_buffer.push('\n');
                broadcaster.publish(task_id, SseEvent::Log(msg));
                exit_code = 1;
                continue; // retry
            }
        };

        exit_code = match rclone::stream_output(process, |line| {
            broadcaster.publish(task_id, SseEvent::Log(line.clone()));
            log_buffer.push_str(&line);
            log_buffer.push('\n');
        })
        .await
        {
            Ok(s) => s.code().unwrap_or(-1),
            Err(e) => {
                tracing::error!("rclone stream error for task {task_id}: {e}");
                1
            }
        };

        stats = extract_rclone_stats(&log_buffer);

        if exit_code == 0 {
            break; // succès, pas besoin de retry
        }

        if attempt < max_attempts {
            let msg = format!("--- Échec (code {exit_code}), nouvelle tentative... ---");
            broadcaster.publish(task_id, SseEvent::Log(msg.clone()));
            log_buffer.push_str(&msg);
            log_buffer.push('\n');
        }
    }

    let error_logs = if exit_code != 0 {
        extract_readable_logs(&log_buffer)
    } else {
        String::new()
    };
    finish_run(&state, task_id, run_id, started_at, exit_code, log_buffer, stats).await;

    // Notification uniquement à la fin (après tous les retries)
    if let Some(channel_id) = snapshot.notification_channel_id {
        let should_notify = if exit_code != 0 {
            snapshot.notify_on.iter().any(|n| n == "error")
        } else {
            snapshot.notify_on.iter().any(|n| n == "success")
        };

        if should_notify {
            let (subject, detail) = if exit_code != 0 {
                let logs_tail = error_logs
                    .lines()
                    .take(50)
                    .collect::<Vec<_>>()
                    .join("\n");

                let mut body = format!(
                    "**Tâche** : `{task_id}`\n\
                     **Run** : `{run_id}`\n\
                     **Tentatives** : {max_attempts}\n\
                     **Code de sortie** : `{exit_code}`\n\
                     \n\
                     **Logs** :\n\
                     ```\n\
                     {logs_tail}\n\
                     ```"
                );
                body.truncate(4000);
                ("Échec de la tâche de réplication", body)
            } else {
                let body = format!(
                    "**Tâche** : `{task_id}`\n\
                     **Run** : `{run_id}`\n\
                     \n\
                     La synchronisation s'est terminée avec succès."
                );
                ("Tâche de réplication terminée avec succès", body)
            };
            send_notification(&state, channel_id, subject, &detail).await;
        }
    }
}

/// Extrait les lignes lisibles des logs rclone pour inclusion dans les notifications.
fn extract_readable_logs(log_output: &str) -> String {
    let mut lines = Vec::new();
    for raw in log_output.lines() {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with('{') {
            if let Ok(obj) = serde_json::from_str::<serde_json::Value>(trimmed) {
                let level = obj.get("level").and_then(|v| v.as_str()).unwrap_or("");
                let msg = obj.get("msg").and_then(|v| v.as_str()).unwrap_or("").trim();
                if msg.is_empty() {
                    continue;
                }
                if level == "error" {
                    lines.push(format!("[ERROR] {msg}"));
                } else {
                    lines.push(msg.to_string());
                }
            }
        } else {
            lines.push(trimmed.to_string());
        }
    }
    lines.join("\n")
}

/// Parcourt les lignes du log rclone (JSON) depuis la fin pour trouver la dernière
/// ligne contenant un objet "stats" et l'extrait.
fn extract_rclone_stats(log_output: &str) -> Option<serde_json::Value> {
    for line in log_output.lines().rev() {
        let line = line.trim();
        if line.is_empty() || !line.starts_with('{') {
            continue;
        }
        if let Ok(obj) = serde_json::from_str::<serde_json::Value>(line) {
            if obj.get("stats").is_some() {
                return obj.get("stats").cloned();
            }
        }
    }
    None
}

async fn finish_run(
    state: &AppState,
    task_id: Uuid,
    run_id: Uuid,
    started_at: chrono::DateTime<Utc>,
    exit_code: i32,
    log_output: String,
    stats: Option<serde_json::Value>,
) {
    let finished_at = Utc::now();
    let duration_ms = (finished_at - started_at).num_milliseconds();
    let status = if exit_code == 0 { "success" } else { "failure" };

    let update_result = task_run::Entity::update_many()
        .col_expr(task_run::Column::Status, Expr::value(status))
        .col_expr(task_run::Column::FinishedAt, Expr::value(chrono::DateTime::<chrono::FixedOffset>::from(finished_at)))
        .col_expr(task_run::Column::DurationMs, Expr::value(duration_ms))
        .col_expr(task_run::Column::ExitCode, Expr::value(exit_code))
        .col_expr(task_run::Column::LogOutput, Expr::value(log_output))
        .col_expr(task_run::Column::Stats, Expr::value(stats))
        .filter(task_run::Column::Id.eq(run_id))
        .exec(&state.db)
        .await;

    if let Err(e) = update_result {
        tracing::error!("Failed to update run {run_id}: {e}");
    }

    state.sse_broadcaster.publish(
        task_id,
        SseEvent::Done {
            status: status.to_string(),
            exit_code: Some(exit_code),
            duration_ms,
        },
    );
    state.sse_broadcaster.close(&task_id);

    state.global_broadcaster.publish(GlobalEvent::TaskFinished {
        task_id,
        status: status.to_string(),
    });

    state.running_tasks.remove(&task_id);

    tracing::info!("Run {run_id} finished status={status} in {duration_ms}ms");
}

async fn send_notification(
    state: &AppState,
    channel_id: Uuid,
    subject: &str,
    detail: &str,
) {
    let channel = match notification_channel::Entity::find_by_id(channel_id)
        .filter(notification_channel::Column::Enabled.eq(true))
        .one(&state.db)
        .await
    {
        Ok(Some(ch)) => ch,
        _ => return,
    };

    let _ = apprise::send_notification(
        &state.config.apprise_bin,
        &[channel.apprise_url],
        subject,
        detail,
    )
    .await;
}

async fn load_remotes(state: &AppState) -> anyhow::Result<Vec<RcloneRemote>> {
    let remotes = remote::Entity::find()
        .all(&state.db)
        .await?;

    Ok(remotes
        .into_iter()
        .map(|r| RcloneRemote {
            id: r.id,
            name: r.name,
            remote_type: r.remote_type,
            config: r.config,
        })
        .collect())
}
