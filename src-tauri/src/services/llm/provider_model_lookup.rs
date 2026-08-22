use super::provider_model_registry::{self, ProviderModelConfig};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ModelCapabilities {
    pub supports_tools: bool,
    pub supports_vision: bool,
    pub supports_thinking: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ModelLimits {
    pub context_window: Option<u32>,
    pub max_output_tokens: Option<u32>,
    pub default_output_tokens: Option<u32>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModelReasoning {
    pub modes: Vec<String>,
    pub default_mode: Option<String>,
}

pub async fn capabilities(provider_id: &str, model_id: &str) -> Option<ModelCapabilities> {
    if let Some(capabilities) = local_capabilities(provider_id, model_id) {
        return Some(capabilities);
    }
    super::litellm_catalog_lookup::capabilities(provider_id, model_id)
        .await
        .map(|capabilities| ModelCapabilities {
            supports_tools: capabilities.supports_tools,
            supports_vision: capabilities.supports_vision,
            supports_thinking: capabilities.supports_thinking,
        })
}

pub async fn limits(provider_id: &str, model_id: &str) -> Option<ModelLimits> {
    if let Some(limits) = local_limits(provider_id, model_id) {
        return Some(limits);
    }
    super::litellm_catalog_lookup::limits(provider_id, model_id)
        .await
        .map(|limits| ModelLimits {
            context_window: limits.context_window,
            max_output_tokens: limits.max_output_tokens,
            default_output_tokens: None,
        })
}

pub fn local_limits(provider_id: &str, model_id: &str) -> Option<ModelLimits> {
    let model = local_entry(provider_id, model_id)?;
    Some(ModelLimits {
        context_window: Some(model.context_window),
        max_output_tokens: model.max_output_tokens,
        default_output_tokens: model.default_output_tokens,
    })
}

pub fn local_reasoning(provider_id: &str, model_id: &str) -> Option<ModelReasoning> {
    let model = local_entry(provider_id, model_id)?;
    Some(ModelReasoning {
        modes: model.reasoning_modes,
        default_mode: model.default_reasoning_mode,
    })
}

pub fn supports_fast_mode(provider_id: &str, model_id: &str) -> bool {
    provider_id == crate::services::llm::providers::openai::PROVIDER_ID
        && provider_model_registry::lookup(provider_id, model_id)
            .is_some_and(|model| model.supports_fast_mode)
}

pub async fn is_chat_model(provider_id: &str, model_id: &str) -> bool {
    local_entry(provider_id, model_id).is_some()
        || super::litellm_catalog_lookup::is_chat_model(provider_id, model_id).await
}

pub fn local_capabilities(provider_id: &str, model_id: &str) -> Option<ModelCapabilities> {
    let entries = [
        direct_entry(provider_id, model_id),
        upstream_entry(provider_id, model_id),
    ];
    let mut found = false;
    let mut result = ModelCapabilities::default();
    for entry in entries.into_iter().flatten() {
        found = true;
        result.supports_tools |= entry.supports_tools;
        result.supports_vision |= entry.supports_vision;
        result.supports_thinking |= entry.supports_thinking;
    }
    found.then_some(result)
}

fn local_entry(provider_id: &str, model_id: &str) -> Option<ProviderModelConfig> {
    direct_entry(provider_id, model_id).or_else(|| upstream_entry(provider_id, model_id))
}

fn direct_entry(provider_id: &str, model_id: &str) -> Option<ProviderModelConfig> {
    provider_model_registry::lookup(provider_id, model_id).or_else(|| {
        let (owner, stripped) = model_id.split_once('/')?;
        (canonical_owner(owner) == provider_id)
            .then(|| provider_model_registry::lookup(provider_id, stripped))
            .flatten()
    })
}

fn upstream_entry(provider_id: &str, model_id: &str) -> Option<ProviderModelConfig> {
    if !provider_model_registry::inherits_upstream(provider_id) {
        return None;
    }
    let (owner, model) = model_id.split_once('/')?;
    provider_model_registry::lookup(canonical_owner(owner), model)
}

fn canonical_owner(owner: &str) -> &str {
    match owner {
        "x-ai" => "xai",
        "z-ai" => "zai",
        "moonshotai" => "moonshot",
        other => other,
    }
}

#[cfg(test)]
#[path = "provider_model_lookup_tests.rs"]
mod tests;
