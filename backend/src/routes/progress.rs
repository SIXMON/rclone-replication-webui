use crate::{
    errors::AppResult,
    sse::broadcaster::SseEvent,
    state::AppState,
};
use axum::{
    extract::{Path, State},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
};
use futures::stream::{self, Stream, StreamExt};
use serde_json::json;
use std::{convert::Infallible, pin::Pin};
use tokio_stream::wrappers::BroadcastStream;
use uuid::Uuid;

type SseStream = Pin<Box<dyn Stream<Item = Result<Event, Infallible>> + Send>>;

pub async fn stream_progress(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    let event_stream: SseStream = if !state.running_tasks.contains_key(&task_id) {
        let idle_event = Event::default()
            .event("idle")
            .data(json!({"message": "No task currently running"}).to_string());
        Box::pin(stream::once(async move { Ok::<Event, Infallible>(idle_event) }))
    } else {
        let rx = state.sse_broadcaster.subscribe(task_id);
        Box::pin(BroadcastStream::new(rx).filter_map(move |msg| {
            let event = match msg {
                Ok(SseEvent::Log(line)) => Some(
                    Event::default()
                        .event("log")
                        .data(json!({"line": line}).to_string()),
                ),
                Ok(SseEvent::Done { status, exit_code, duration_ms }) => Some(
                    Event::default()
                        .event("done")
                        .data(
                            json!({"status": status, "exit_code": exit_code, "duration_ms": duration_ms})
                                .to_string(),
                        ),
                ),
                Err(_) => None,
            };
            async move { event.map(|e| Ok::<Event, Infallible>(e)) }
        }))
    };

    Ok(Sse::new(event_stream).keep_alive(KeepAlive::default()))
}
