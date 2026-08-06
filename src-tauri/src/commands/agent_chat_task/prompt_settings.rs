use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::system_prompt_store::SystemPromptSettings;
use crate::services::agent_local::types_ollama::StreamEvent;

pub(super) fn load(on_event: &AgentEventEmitter) -> SystemPromptSettings {
    let snapshot = crate::services::agent_local::system_prompt_store::snapshot_for_runtime();
    if let Some(message_key) = snapshot.notice_key {
        let _ = on_event.send(StreamEvent::Notice {
            message_key: message_key.to_string(),
        });
    }
    snapshot.settings
}
