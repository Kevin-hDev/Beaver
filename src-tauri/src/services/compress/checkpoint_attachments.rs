#![allow(
    dead_code,
    reason = "the compression orchestrator consumes checkpoint attachments in Task 10"
)]

use std::collections::BTreeSet;

use crate::services::agent_local::types_session::{AgentMessage, FileAttachment};

#[derive(Debug, Clone)]
pub struct CheckpointImage {
    pub source_message_id: String,
    pub file: FileAttachment,
    pub provider_payload: String,
    pub estimated_bytes: u64,
}

pub fn collect_images(
    messages: &[AgentMessage],
    context_window: u64,
    provider_max_images: usize,
) -> Vec<CheckpointImage> {
    let (profile_max, max_bytes) = if context_window < 64_000 {
        (8usize, 16 * 1024 * 1024u64)
    } else {
        (16usize, 32 * 1024 * 1024u64)
    };
    collect_images_with_limits(messages, profile_max, max_bytes, provider_max_images)
}

pub fn collect_images_with_limits(
    messages: &[AgentMessage],
    profile_max_images: usize,
    profile_max_bytes: u64,
    provider_max_images: usize,
) -> Vec<CheckpointImage> {
    let max_images = profile_max_images.min(provider_max_images).min(16);
    let max_bytes = profile_max_bytes.min(32 * 1024 * 1024);
    let mut seen = BTreeSet::new();
    let mut total_bytes = 0u64;
    let mut selected = Vec::new();
    for message in messages.iter().rev() {
        for file in message.files.iter().rev() {
            let Some(thumbnail) = file.thumbnail.as_deref() else {
                continue;
            };
            let Some(payload) = validated_payload(thumbnail) else {
                continue;
            };
            let identity = format!("{}:{}:{}", file.name, file.size, payload.len());
            if !seen.insert(identity) {
                continue;
            }
            let bytes = payload.len().saturating_mul(3).div_ceil(4) as u64;
            if selected.len() >= max_images || total_bytes.saturating_add(bytes) > max_bytes {
                continue;
            }
            total_bytes = total_bytes.saturating_add(bytes);
            selected.push(CheckpointImage {
                source_message_id: message.id.clone(),
                file: file.clone(),
                provider_payload: payload,
                estimated_bytes: bytes,
            });
        }
    }
    selected.reverse();
    selected
}

fn validated_payload(thumbnail: &str) -> Option<String> {
    let mut message = crate::services::agent_local::types_ollama::ChatMessage::user(String::new());
    message.images = Some(vec![thumbnail.to_string()]);
    let report =
        crate::services::llm::vision::sanitize_messages(std::slice::from_mut(&mut message), true);
    (report.invalid_removed == 0)
        .then(|| message.images.and_then(|images| images.into_iter().next()))
        .flatten()
}
