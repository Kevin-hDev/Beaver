use serde::Serialize;

use super::provider_model_registry::ProviderModelConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityProvenance {
    EmbeddedRegistry,
    ValidatedRuntime,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResolvedModelCapabilities {
    pub supports_tools: bool,
    pub supports_vision: bool,
    pub supports_thinking: bool,
    pub supports_fast_mode: bool,
    pub reasoning_modes: Vec<String>,
    pub default_reasoning_mode: Option<String>,
    pub provenance: CapabilityProvenance,
}

pub fn resolve_local(provider_id: &str, model_id: &str) -> Option<ResolvedModelCapabilities> {
    if provider_id == crate::services::codex_client::PROVIDER_ID {
        let model = crate::services::codex_client::model_catalog::fallback_models()
            .into_iter()
            .find(|model| model.id == model_id)?;
        return Some(from_runtime(model));
    }
    if let Some(model) = super::provider_model_lookup::local_entry(provider_id, model_id) {
        return Some(from_embedded(provider_id, model_id, model));
    }
    super::runtime_models::lookup(provider_id, model_id).map(from_runtime)
}

pub fn resolve_remote_list_defaults(
    provider_id: &str,
    model_id: &str,
) -> Option<ResolvedModelCapabilities> {
    super::provider_model_lookup::direct_entry(provider_id, model_id)
        .map(|model| from_embedded(provider_id, model_id, model))
}

pub async fn resolve(provider_id: &str, model_id: &str) -> Option<ResolvedModelCapabilities> {
    if let Some(resolved) = resolve_local(provider_id, model_id) {
        return Some(resolved);
    }
    super::litellm_catalog_lookup::capabilities(provider_id, model_id)
        .await
        .map(from_litellm)
}

pub fn resolve_reasoning_modes(
    provider_id: &str,
    model_id: &str,
    supports_thinking: bool,
) -> Vec<String> {
    if !supports_thinking {
        return Vec::new();
    }
    resolve_local(provider_id, model_id)
        .map(|resolved| resolved.reasoning_modes)
        .unwrap_or_default()
}

fn from_embedded(
    provider_id: &str,
    model_id: &str,
    model: ProviderModelConfig,
) -> ResolvedModelCapabilities {
    ResolvedModelCapabilities {
        supports_tools: model.supports_tools,
        supports_vision: model.supports_vision,
        supports_thinking: model.supports_thinking,
        supports_fast_mode: model.supports_fast_mode,
        reasoning_modes: restrict_runtime(provider_id, model_id, model.reasoning_modes),
        default_reasoning_mode: model.default_reasoning_mode,
        provenance: CapabilityProvenance::EmbeddedRegistry,
    }
}

fn from_runtime(model: super::types::ModelInfo) -> ResolvedModelCapabilities {
    ResolvedModelCapabilities {
        supports_tools: model.supports_tools,
        supports_vision: model.supports_vision,
        supports_thinking: model.supports_thinking,
        supports_fast_mode: model.supports_fast_mode,
        reasoning_modes: model.reasoning_modes,
        default_reasoning_mode: model.default_reasoning_mode,
        provenance: CapabilityProvenance::ValidatedRuntime,
    }
}

fn from_litellm(
    model: super::litellm_catalog_lookup::CatalogCapabilities,
) -> ResolvedModelCapabilities {
    ResolvedModelCapabilities {
        supports_tools: model.supports_tools,
        supports_vision: model.supports_vision,
        supports_thinking: model.supports_thinking,
        supports_fast_mode: false,
        reasoning_modes: Vec::new(),
        default_reasoning_mode: None,
        provenance: CapabilityProvenance::ValidatedRuntime,
    }
}

fn restrict_runtime(provider_id: &str, model_id: &str, modes: Vec<String>) -> Vec<String> {
    let Some(runtime) = super::runtime_models::lookup(provider_id, model_id) else {
        return modes;
    };
    if runtime.reasoning_modes.is_empty() {
        return modes;
    }
    modes
        .into_iter()
        .filter(|mode| runtime.reasoning_modes.contains(mode))
        .collect()
}
