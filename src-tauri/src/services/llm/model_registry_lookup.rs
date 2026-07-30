use std::collections::HashMap;

use super::model_registry::{get_lock, ModelEntry};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ModelCapabilities {
    pub supports_tools: bool,
    pub supports_vision: bool,
    pub supports_thinking: bool,
}

pub(crate) fn find_provider_entry<'a>(
    registry: &'a HashMap<String, ModelEntry>,
    provider_id: &str,
    model_id: &str,
) -> Option<&'a ModelEntry> {
    let prefix = provider_prefix(provider_id);
    registry
        .get(&format!("{prefix}/{model_id}"))
        .or_else(|| matching_bare_entry(registry, model_id, prefix))
        .or_else(|| {
            let stripped = strip_owner(model_id);
            registry
                .get(&format!("{prefix}/{stripped}"))
                .or_else(|| matching_bare_entry(registry, stripped, prefix))
        })
}

fn find_upstream_entry<'a>(
    registry: &'a HashMap<String, ModelEntry>,
    provider_id: &str,
    model_id: &str,
) -> Option<&'a ModelEntry> {
    if provider_id != "openrouter" {
        return None;
    }
    let (owner, model) = model_id.split_once('/')?;
    find_provider_entry(registry, owner, model)
}

pub async fn capabilities(provider_id: &str, model_id: &str) -> Option<ModelCapabilities> {
    let registry = get_lock().read().await;
    capabilities_for(&registry, provider_id, model_id)
}

fn capabilities_for(
    registry: &HashMap<String, ModelEntry>,
    provider_id: &str,
    model_id: &str,
) -> Option<ModelCapabilities> {
    let entries = [
        find_provider_entry(registry, provider_id, model_id),
        find_upstream_entry(registry, provider_id, model_id),
    ];
    let mut found = false;
    let mut capabilities = ModelCapabilities::default();
    for entry in entries.into_iter().flatten().filter(|entry| is_chat(entry)) {
        found = true;
        capabilities.supports_tools |= entry.supports_function_calling;
        capabilities.supports_vision |= entry.supports_vision;
        capabilities.supports_thinking |= entry.supports_reasoning;
    }
    found.then_some(capabilities)
}

pub async fn max_output_tokens(provider_id: &str, model_id: &str) -> Option<u32> {
    let registry = get_lock().read().await;
    max_output_tokens_for(&registry, provider_id, model_id)
}

fn max_output_tokens_for(
    registry: &HashMap<String, ModelEntry>,
    provider_id: &str,
    model_id: &str,
) -> Option<u32> {
    find_upstream_entry(registry, provider_id, model_id)
        .or_else(|| find_provider_entry(registry, provider_id, model_id))
        .filter(|entry| is_chat(entry))
        .and_then(output_limit)
}

pub async fn is_chat_model(provider_id: &str, model_id: &str) -> bool {
    let registry = get_lock().read().await;
    find_provider_entry(&registry, provider_id, model_id)
        .or_else(|| find_upstream_entry(&registry, provider_id, model_id))
        .map_or_else(|| !is_non_chat_name(model_id), is_chat)
}

fn matching_bare_entry<'a>(
    registry: &'a HashMap<String, ModelEntry>,
    model_id: &str,
    provider: &str,
) -> Option<&'a ModelEntry> {
    registry
        .get(model_id)
        .filter(|entry| entry.litellm_provider.as_deref() == Some(provider))
}

fn output_limit(entry: &ModelEntry) -> Option<u32> {
    entry
        .max_output_tokens
        .or(entry.max_tokens)
        .and_then(|tokens| u32::try_from(tokens).ok())
        .filter(|tokens| *tokens > 0)
}

fn is_chat(entry: &ModelEntry) -> bool {
    matches!(
        entry.mode.as_deref(),
        Some("chat") | Some("completion") | None
    )
}

fn provider_prefix(provider_id: &str) -> &str {
    match provider_id {
        "google" => "gemini",
        "x-ai" => "xai",
        _ => provider_id,
    }
}

fn strip_owner(model_id: &str) -> &str {
    model_id
        .rsplit_once('/')
        .map(|(_, model)| model)
        .unwrap_or(model_id)
}

fn is_non_chat_name(model_id: &str) -> bool {
    let id = model_id.to_lowercase();
    [
        "whisper",
        "dall-e",
        "tts",
        "embedding",
        "embed",
        "moderation",
        "rerank",
        "lyria",
        "imagen",
        "veo",
        "music",
        "sora",
        "gpt-image",
        "stable-diffusion",
    ]
    .iter()
    .any(|keyword| id.contains(keyword))
}

#[cfg(test)]
#[path = "model_registry_lookup_tests.rs"]
mod tests;
