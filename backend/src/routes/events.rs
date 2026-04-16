use crate::{errors::AppResult, state::AppState};
use axum::{
    extract::State,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
};
use futures::stream::StreamExt;
use std::convert::Infallible;
use tokio_stream::wrappers::BroadcastStream;

/// Endpoint SSE global : diffuse les événements de cycle de vie des tâches
/// à tous les clients connectés (task_started, task_finished).
pub async fn stream_events(
    State(state): State<AppState>,
) -> AppResult<impl IntoResponse> {
    let rx = state.global_broadcaster.subscribe();

    let stream = BroadcastStream::new(rx).filter_map(|msg| async move {
        match msg {
            Ok(event) => {
                let data = serde_json::to_string(&event).ok()?;
                let event_type = match &event {
                    crate::sse::global::GlobalEvent::TaskStarted { .. } => "task_started",
                    crate::sse::global::GlobalEvent::TaskFinished { .. } => "task_finished",
                };
                Some(Ok::<Event, Infallible>(
                    Event::default().event(event_type).data(data),
                ))
            }
            Err(_) => None,
        }
    });

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}
