use std::path::Path;

use super::profile_types::CompressionWindowBand;
use super::snapshot::CompressionSnapshot;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::StreamEvent;

pub(super) fn image_count(snapshot: &CompressionSnapshot) -> u16 {
    let configured = match snapshot.profile.band(snapshot.context_window) {
        Some(CompressionWindowBand::Under64K) => snapshot.profile.profile.under_64k.image_count,
        Some(CompressionWindowBand::Large) => snapshot.profile.profile.large.image_count,
        Some(CompressionWindowBand::Compact) | None => snapshot.profile.profile.compact.image_count,
    };
    configured.min(crate::services::llm::vision::MAX_IMAGES_PER_MESSAGE as u16)
}

pub(super) fn tool_names(tools: &[serde_json::Value]) -> Vec<String> {
    tools
        .iter()
        .filter_map(|tool| {
            tool.pointer("/function/name")
                .and_then(serde_json::Value::as_str)
        })
        .take(256)
        .map(str::to_string)
        .collect()
}

pub(super) fn system_head_tokens(
    provider_id: &str,
    messages: &[crate::services::agent_local::types_ollama::ChatMessage],
    tools: &[serde_json::Value],
) -> u32 {
    let system_messages = messages
        .iter()
        .filter(|message| message.role == "system")
        .cloned()
        .collect::<Vec<_>>();
    super::token_estimate::estimate_textual_request_tokens_for_provider(
        provider_id,
        &system_messages,
        tools,
    )
    .min(u32::MAX as usize) as u32
}

pub(super) fn is_git_repository(working_dir: &Path) -> bool {
    git2::Repository::discover(working_dir).is_ok()
}

pub(super) fn send(on_event: &AgentEventEmitter, status: &str) {
    let _ = on_event.send(StreamEvent::Compressing {
        status: status.to_string(),
    });
}
