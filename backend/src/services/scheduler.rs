use crate::{
    entities::{notification_channel, task, task_run},
    services::task_executor,
    state::AppState,
};
use chrono::Utc;
use sea_orm::*;
use tokio_cron_scheduler::{Job, JobScheduler};

/// Normalise une expression cron vers le format 6 champs attendu par tokio-cron-scheduler.
/// Le format standard Unix à 5 champs (`min hour day month weekday`) est converti en
/// `0 min hour day month weekday` (ajout du champ secondes à 0).
/// Les expressions déjà en 6 champs ou les macros (`@hourly`, etc.) sont laissées telles quelles.
fn normalize_cron(expr: &str) -> String {
    let trimmed = expr.trim();
    if trimmed.starts_with('@') {
        return trimmed.to_string();
    }
    let field_count = trimmed.split_whitespace().count();
    if field_count == 5 {
        format!("0 {trimmed}")
    } else {
        trimmed.to_string()
    }
}

pub async fn rebuild_scheduler(state: AppState) {
    let tasks = match task::Entity::find()
        .filter(
            Condition::all()
                .add(task::Column::Enabled.eq(true))
                .add(task::Column::CronExpression.is_not_null()),
        )
        .all(&state.db)
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("scheduler: failed to load tasks: {e}");
            return;
        }
    };

    let sched = match JobScheduler::new().await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("scheduler: failed to create scheduler: {e}");
            return;
        }
    };

    let mut job_count = 0;
    for t in tasks {
        let cron = match t.cron_expression {
            Some(c) => c,
            None => continue,
        };
        // tokio-cron-scheduler attend un format cron à 6 champs (avec secondes).
        // Les utilisateurs entrent le format standard à 5 champs → on prepend "0 " pour les secondes.
        let normalized_cron = normalize_cron(&cron);
        let task_id = t.id;
        let notify_on = t.notify_on.clone();
        let notification_channel_id = t.notification_channel_id;

        let state_clone = state.clone();
        let job = Job::new_async(normalized_cron.as_str(), move |_uuid, _lock| {
            let s = state_clone.clone();
            let notify_on = notify_on.clone();
            Box::pin(async move {
                // Vérifier si la tâche est déjà en cours
                if s.running_tasks.contains_key(&task_id) {
                    tracing::warn!("scheduler: task {task_id} is already running, skipping");
                    log_skipped_run(&s, task_id, notification_channel_id, &notify_on).await;
                    return;
                }

                tracing::info!("scheduler: triggering task {task_id}");
                if let Err(e) =
                    task_executor::spawn_task(s, task_id, task_executor::ExecutionMode::Scheduled).await
                {
                    tracing::warn!("scheduler: failed to trigger task {task_id}: {e}");
                }
            })
        });

        match job {
            Ok(j) => {
                if let Err(e) = sched.add(j).await {
                    tracing::error!("scheduler: failed to add job for task {task_id}: {e}");
                } else {
                    job_count += 1;
                }
            }
            Err(e) => tracing::error!(
                "scheduler: invalid cron '{}' (normalized: '{}') for task {task_id}: {e}",
                cron,
                normalized_cron
            ),
        }
    }

    if let Err(e) = sched.start().await {
        tracing::error!("scheduler: failed to start: {e}");
        return;
    }

    let mut guard = state.scheduler_handle.lock().await;
    if let Some(mut old) = guard.take() {
        let _ = old.shutdown().await;
    }
    *guard = Some(sched);

    tracing::info!("scheduler: rebuilt with {job_count} job(s)");
}

/// Enregistre un run "skipped" quand la tâche est déjà en cours et notifie si demandé.
async fn log_skipped_run(
    state: &AppState,
    task_id: uuid::Uuid,
    notification_channel_id: Option<uuid::Uuid>,
    notify_on: &[String],
) {
    let now = Utc::now();
    let run = task_run::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        task_id: Set(task_id),
        triggered_by: Set("scheduler".to_string()),
        status: Set("skipped".to_string()),
        started_at: Set(now.into()),
        finished_at: Set(Some(now.into())),
        duration_ms: Set(Some(0)),
        exit_code: Set(None),
        log_output: Set(Some("Exécution ignorée : la tâche précédente est encore en cours.".to_string())),
        stats: Set(None),
    };

    if let Err(e) = run.insert(&state.db).await {
        tracing::error!("scheduler: failed to log skipped run for task {task_id}: {e}");
    }

    // Notifier si "skipped" est dans notify_on
    if let Some(channel_id) = notification_channel_id {
        if notify_on.iter().any(|n| n == "skipped") {
            let channel = match notification_channel::Entity::find_by_id(channel_id)
                .filter(notification_channel::Column::Enabled.eq(true))
                .one(&state.db)
                .await
            {
                Ok(Some(ch)) => ch,
                _ => return,
            };

            let body = format!(
                "**Tâche** : `{task_id}`\n\
                 **Raison** : L'exécution précédente est encore en cours\n\
                 \n\
                 La planification cron a tenté de lancer cette tâche, \
                 mais la synchronisation précédente n'est pas terminée. \
                 L'exécution a été _ignorée_."
            );
            let _ = crate::services::apprise::send_notification(
                &state.config.apprise_bin,
                &[channel.apprise_url],
                "Tâche de réplication ignorée",
                &body,
            )
            .await;
        }
    }
}
