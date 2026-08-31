use std::collections::BTreeSet;

use crate::services::agent_local::types_session::AgentMessage;

pub const MAX_IMAGE_CANDIDATES: usize = 64;

#[derive(Debug, Clone)]
pub struct CheckpointImage {
    pub source_message_id: String,
    pub provider_payload: String,
    pub estimated_bytes: u64,
}

pub fn collect_images_with_limits(
    messages: &[AgentMessage],
    profile_max_images: usize,
    profile_max_bytes: u64,
    provider_max_images: usize,
) -> Vec<CheckpointImage> {
    let max_images = profile_max_images
        .min(provider_max_images)
        .min(MAX_IMAGE_CANDIDATES);
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
            if seen.contains(&identity) {
                continue;
            }
            let bytes = payload.len().saturating_mul(3).div_ceil(4) as u64;
            if selected.len() >= max_images || total_bytes.saturating_add(bytes) > max_bytes {
                continue;
            }
            seen.insert(identity);
            total_bytes = total_bytes.saturating_add(bytes);
            selected.push(CheckpointImage {
                source_message_id: message.id.clone(),
                provider_payload: payload,
                estimated_bytes: bytes,
            });
        }
    }
    selected.reverse();
    selected
}

pub fn retain_images_for_messages(
    images: &[CheckpointImage],
    source_message_ids: &BTreeSet<String>,
    profile_max_images: usize,
    profile_max_bytes: u64,
) -> Vec<CheckpointImage> {
    let max_images = profile_max_images.min(16);
    let max_bytes = profile_max_bytes.min(32 * 1024 * 1024);
    let mut total_bytes = 0_u64;
    images
        .iter()
        .filter(|image| source_message_ids.contains(&image.source_message_id))
        .filter(|image| {
            if total_bytes.saturating_add(image.estimated_bytes) > max_bytes {
                return false;
            }
            total_bytes = total_bytes.saturating_add(image.estimated_bytes);
            true
        })
        .take(max_images)
        .cloned()
        .collect()
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
