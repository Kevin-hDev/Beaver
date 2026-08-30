use std::path::Path;

use super::profile_types::{CompressionWindowBand, ImageBudget};
use super::snapshot::CompressionSnapshot;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::StreamEvent;

pub(super) fn image_budget(snapshot: &CompressionSnapshot) -> ImageBudget {
    match snapshot.profile.band(snapshot.context_window) {
        Some(CompressionWindowBand::Under64K) => snapshot.profile.profile.under_64k.images,
        Some(CompressionWindowBand::Large) => snapshot.profile.profile.large.images,
        Some(CompressionWindowBand::Compact) | None => snapshot.profile.profile.compact.images,
    }
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

pub(super) fn is_git_repository(working_dir: &Path) -> bool {
    git2::Repository::discover(working_dir).is_ok()
}

pub(super) fn send(on_event: &AgentEventEmitter, status: &str) {
    let _ = on_event.send(StreamEvent::Compressing {
        status: status.to_string(),
    });
}
