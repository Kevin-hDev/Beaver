use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::{ChatMessage, StreamEvent};
use crate::services::llm::vision;

pub(super) fn sanitize_images(
    on_event: &AgentEventEmitter,
    messages: &mut [ChatMessage],
    supports_vision: bool,
) {
    let image_report = vision::sanitize_messages(messages, supports_vision);
    if image_report.unsupported_removed > 0 {
        let _ = on_event.send(StreamEvent::Notice {
            message_key: vision::NOTICE_UNSUPPORTED_MODEL.to_string(),
        });
    } else if image_report.invalid_removed > 0 {
        let _ = on_event.send(StreamEvent::Notice {
            message_key: vision::NOTICE_IMAGE_SKIPPED.to_string(),
        });
    }
}
