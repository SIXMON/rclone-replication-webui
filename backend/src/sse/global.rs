use serde::Serialize;
use tokio::sync::broadcast;
use uuid::Uuid;

const CHANNEL_CAPACITY: usize = 64;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GlobalEvent {
    TaskStarted { task_id: Uuid },
    TaskFinished { task_id: Uuid, status: String },
}

#[derive(Debug)]
pub struct GlobalBroadcaster {
    tx: broadcast::Sender<GlobalEvent>,
}

impl GlobalBroadcaster {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(CHANNEL_CAPACITY);
        GlobalBroadcaster { tx }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<GlobalEvent> {
        self.tx.subscribe()
    }

    pub fn publish(&self, event: GlobalEvent) {
        let _ = self.tx.send(event);
    }
}
