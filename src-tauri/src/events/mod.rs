use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventPayload<T: Serialize> {
    pub event_id: String,
    pub timestamp: String,
    pub entity_id: String,
    pub event_type: String,
    pub data: T,
}

#[derive(Clone)]
pub struct AppEventEmitter {
    handle: Option<AppHandle>,
}

impl AppEventEmitter {
    pub fn new(handle: AppHandle) -> Self {
        Self {
            handle: Some(handle),
        }
    }

    pub fn noop() -> Self {
        Self { handle: None }
    }

    pub fn emit<T: Serialize + Clone>(&self, event_name: &str, entity_id: &str, data: T) {
        if let Some(h) = &self.handle {
            let payload = EventPayload {
                event_id: uuid::Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                entity_id: entity_id.to_string(),
                event_type: event_name.to_string(),
                data,
            };
            let _ = h.emit(event_name, payload);
        }
    }
}
