use std::collections::HashMap;

use super::litellm_catalog::{get_lock, ModelEntry};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CatalogCapabilities {
    pub supports_tools: bool,
    pub supports_vision: bool,
    pub supports_thinking: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CatalogLimits {
    pub context_window: Option<u32>,
    pub max_output_tokens: Option<u32>,
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
            let (owner, stripped) = model_id.split_once('/')?;
            if provider_prefix(owner) != prefix {
                return None;
            }
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

pub async fn capabilities(provider_id: &str, model_id: &str) -> Option<CatalogCapabilities> {
    let registry = get_lock().read().await;
    capabilities_for(&registry, provider_id, model_id)
}

fn capabilities_for(
    registry: &HashMap<String, ModelEntry>,
    provider_id: &str,
    model_id: &str,
) -> Option<CatalogCapabilities> {
    let entries = [
        find_provider_entry(registry, provider_id, model_id),
        find_upstream_entry(registry, provider_id, model_id),
    ];
    let mut found = false;
    let mut capabilities = CatalogCapabilities::default();
    for entry in entries.into_iter().flatten().filter(|entry| is_chat(entry)) {
        found = true;
        capabilities.supports_tools |= entry.supports_function_calling;
        capabilities.supports_vision |= entry.supports_vision;
        capabilities.supports_thinking |= entry.supports_reasoning;
    }
    found.then_some(capabilities)
}

pub async fn limits(provider_id: &str, model_id: &str) -> Option<CatalogLimits> {
    let registry = get_lock().read().await;
    limits_for(&registry, provider_id, model_id)
}

fn limits_for(
    registry: &HashMap<String, ModelEntry>,
    provider_id: &str,
    model_id: &str,
) -> Option<CatalogLimits> {
    find_upstream_entry(registry, provider_id, model_id)
        .or_else(|| find_provider_entry(registry, provider_id, model_id))
        .filter(|entry| is_chat(entry))
        .map(limits_from_entry)
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

fn limits_from_entry(entry: &ModelEntry) -> CatalogLimits {
    let context_window = entry
        .max_input_tokens
        .or(entry.max_tokens)
        .and_then(|tokens| u32::try_from(tokens).ok())
        .filter(|tokens| *tokens > 0);
    let max_output_tokens = entry
        .max_output_tokens
        .and_then(|tokens| u32::try_from(tokens).ok())
        .filter(|tokens| *tokens > 0)
        .filter(|tokens| context_window.is_none_or(|context| *tokens < context));
    CatalogLimits {
        context_window,
        max_output_tokens,
    }
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
#[path = "litellm_catalog_lookup_tests.rs"]
mod tests;
