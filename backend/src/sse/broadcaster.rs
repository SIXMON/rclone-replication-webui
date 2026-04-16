use dashmap::DashMap;
use tokio::sync::broadcast;
use uuid::Uuid;

const CHANNEL_CAPACITY: usize = 256;

#[derive(Debug, Clone)]
pub enum SseEvent {
    Log(String),
    Done { status: String, exit_code: Option<i32>, duration_ms: i64 },
}

#[derive(Debug)]
pub struct SseBroadcaster {
    channels: DashMap<Uuid, broadcast::Sender<SseEvent>>,
}

impl SseBroadcaster {
    pub fn new() -> Self {
        SseBroadcaster {
            channels: DashMap::new(),
        }
    }

    pub fn subscribe(&self, task_id: Uuid) -> broadcast::Receiver<SseEvent> {
        if let Some(sender) = self.channels.get(&task_id) {
            return sender.subscribe();
        }
        let (tx, rx) = broadcast::channel(CHANNEL_CAPACITY);
        self.channels.insert(task_id, tx);
        rx
    }

    pub fn publish(&self, task_id: Uuid, event: SseEvent) {
        if let Some(sender) = self.channels.get(&task_id) {
            let _ = sender.send(event);
        }
    }

    pub fn close(&self, task_id: &Uuid) {
        self.channels.remove(task_id);
    }
}
