use std::sync::atomic::{AtomicU64, Ordering};

use serde::Serialize;

use super::provider_model_registry::ProviderModelConfig;

static LEGACY_FALLBACK_HITS: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityProvenance {
    EmbeddedRegistry,
    ValidatedRuntime,
    LegacyNameFallback,
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

#[cfg(test)]
pub fn legacy_fallback_count() -> u64 {
    LEGACY_FALLBACK_HITS.load(Ordering::Relaxed)
}

pub fn resolve_local_or_legacy(
    provider_id: &str,
    model_id: &str,
) -> Option<ResolvedModelCapabilities> {
    if let Some(resolved) = resolved_without_fallback(provider_id, model_id) {
        return Some(resolved);
    }
    validate_route(provider_id)?;
    Some(legacy(provider_id, model_id))
}

pub fn resolve_remote_list_defaults(
    provider_id: &str,
    model_id: &str,
) -> Option<ResolvedModelCapabilities> {
    if let Some(model) = super::provider_model_lookup::direct_entry(provider_id, model_id) {
        return Some(from_embedded(provider_id, model_id, model));
    }
    validate_route(provider_id)?;
    Some(legacy(provider_id, model_id))
}

pub async fn resolve(provider_id: &str, model_id: &str) -> Option<ResolvedModelCapabilities> {
    if let Some(resolved) = resolved_without_fallback(provider_id, model_id) {
        return Some(resolved);
    }
    if let Some(capabilities) =
        super::litellm_catalog_lookup::capabilities(provider_id, model_id).await
    {
        return Some(from_litellm(provider_id, model_id, capabilities));
    }
    validate_route(provider_id)?;
    Some(legacy(provider_id, model_id))
}

pub fn resolve_reasoning_modes(
    provider_id: &str,
    model_id: &str,
    supports_thinking: bool,
) -> Vec<String> {
    if !supports_thinking {
        return Vec::new();
    }
    let Some(resolved) = resolve_local_or_legacy(provider_id, model_id) else {
        return Vec::new();
    };
    if !resolved.reasoning_modes.is_empty() {
        return resolved.reasoning_modes;
    }
    super::legacy_capability_fallback::reasoning_modes(provider_id, model_id)
}

fn validate_route(provider_id: &str) -> Option<()> {
    super::route_profile::find(provider_id).map(|_| ())
}

fn resolved_without_fallback(
    provider_id: &str,
    model_id: &str,
) -> Option<ResolvedModelCapabilities> {
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

fn from_embedded(
    provider_id: &str,
    model_id: &str,
    model: ProviderModelConfig,
) -> ResolvedModelCapabilities {
    let (reasoning_modes, used_fallback) = reasoning_modes(
        provider_id,
        model_id,
        model.supports_thinking,
        model.reasoning_modes,
    );
    ResolvedModelCapabilities {
        supports_tools: model.supports_tools,
        supports_vision: model.supports_vision,
        supports_thinking: model.supports_thinking,
        supports_fast_mode: model.supports_fast_mode,
        reasoning_modes,
        default_reasoning_mode: model.default_reasoning_mode.or_else(|| {
            super::legacy_capability_fallback::default_reasoning_mode(provider_id, model_id)
        }),
        provenance: if used_fallback {
            fallback_hit()
        } else {
            CapabilityProvenance::EmbeddedRegistry
        },
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
    provider_id: &str,
    model_id: &str,
    model: super::litellm_catalog_lookup::CatalogCapabilities,
) -> ResolvedModelCapabilities {
    let (reasoning_modes, used_fallback) =
        reasoning_modes(provider_id, model_id, model.supports_thinking, Vec::new());
    ResolvedModelCapabilities {
        supports_tools: model.supports_tools,
        supports_vision: model.supports_vision,
        supports_thinking: model.supports_thinking,
        supports_fast_mode: false,
        reasoning_modes,
        default_reasoning_mode: None,
        provenance: if used_fallback {
            fallback_hit()
        } else {
            CapabilityProvenance::ValidatedRuntime
        },
    }
}

fn legacy(provider_id: &str, model_id: &str) -> ResolvedModelCapabilities {
    let legacy = super::legacy_capability_fallback::resolve(provider_id, model_id);
    ResolvedModelCapabilities {
        supports_tools: legacy.supports_tools,
        supports_vision: legacy.supports_vision,
        supports_thinking: legacy.supports_thinking,
        supports_fast_mode: false,
        reasoning_modes: legacy.reasoning_modes,
        default_reasoning_mode: legacy.default_reasoning_mode,
        provenance: fallback_hit(),
    }
}

fn reasoning_modes(
    provider_id: &str,
    model_id: &str,
    supports_thinking: bool,
    modes: Vec<String>,
) -> (Vec<String>, bool) {
    if !supports_thinking || !modes.is_empty() {
        return (restrict_runtime(provider_id, model_id, modes), false);
    }
    let legacy = super::legacy_capability_fallback::reasoning_modes(provider_id, model_id);
    (restrict_runtime(provider_id, model_id, legacy), true)
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

fn fallback_hit() -> CapabilityProvenance {
    let count = LEGACY_FALLBACK_HITS.fetch_add(1, Ordering::Relaxed) + 1;
    if count.is_power_of_two() {
        ::log::debug!("[provider-capabilities] legacy_fallback_count={count}");
    }
    CapabilityProvenance::LegacyNameFallback
}
