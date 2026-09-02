use crate::services::agent_local::types_ollama::StreamEvent;
use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};
use tauri::{AppHandle, Emitter};

pub const AGENT_STREAM_EVENT: &str = "agent-stream-event";
static STREAM_GENERATION: AtomicU64 = AtomicU64::new(0);

pub fn next_generation() -> u64 {
    STREAM_GENERATION.fetch_add(1, Ordering::Relaxed)
}

#[derive(Clone)]
pub struct AgentEventEmitter {
    #[cfg(not(test))]
    app: AppHandle,
    #[cfg(test)]
    app: Option<AppHandle>,
    session_id: String,
    generation: Option<u64>,
    permission_emitter: Option<Box<AgentEventEmitter>>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct StreamEventPayload {
    session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation: Option<u64>,
    event: StreamEvent,
}

impl AgentEventEmitter {
    pub fn new(app: AppHandle, session_id: String) -> Self {
        #[cfg(test)]
        let app = Some(app);
        Self {
            app,
            session_id,
            generation: None,
            permission_emitter: None,
        }
    }

    #[cfg(test)]
    pub fn test(session_id: String) -> Self {
        Self {
            app: None,
            session_id,
            generation: None,
            permission_emitter: None,
        }
    }

    pub fn with_generation(app: AppHandle, session_id: String, generation: u64) -> Self {
        #[cfg(test)]
        let app = Some(app);
        Self {
            app,
            session_id,
            generation: Some(generation),
            permission_emitter: None,
        }
    }

    pub fn with_permission_emitter(mut self, emitter: AgentEventEmitter) -> Self {
        self.permission_emitter = Some(Box::new(emitter));
        self
    }

    pub fn start_mascot_session(&self) -> Option<crate::services::mascot::MascotSession> {
        let app = self.app()?;
        self.generation.map(|generation| {
            crate::services::mascot::MascotSession::start(
                app,
                self.session_id.clone(),
                generation,
            )
        })
    }

    pub fn send(&self, event: StreamEvent) -> Result<(), String> {
        if is_permission_request(&event) {
            if let Some(emitter) = self.permission_emitter.as_deref() {
                return emitter.send(event);
            }
        }
        let Some(app) = self.app() else {
            return Ok(());
        };
        crate::services::mascot::observe_stream_event(
            app,
            &self.session_id,
            self.generation,
            &event,
        );
        app.emit(
            AGENT_STREAM_EVENT,
            StreamEventPayload {
                session_id: self.session_id.clone(),
                generation: self.generation,
                event,
            },
        )
        .map_err(|_| "Emission evenement impossible".to_string())
    }

    #[cfg(not(test))]
    fn app(&self) -> Option<&AppHandle> {
        Some(&self.app)
    }

    #[cfg(test)]
    fn app(&self) -> Option<&AppHandle> {
        self.app.as_ref()
    }
}

fn is_permission_request(event: &StreamEvent) -> bool {
    matches!(event, StreamEvent::PermissionRequest(..))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_permission_requests_use_the_parent_route() {
        assert!(is_permission_request(&StreamEvent::PermissionRequest(
            super::super::permission_request::native(
                "request".into(),
                "bash",
                &serde_json::json!({}),
            ),
        )));
        assert!(!is_permission_request(&StreamEvent::Notice {
            message_key: "child-content".into(),
        }));
    }
}
