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
    pub reasoning_contract: Option<super::model_reasoning_contract::ModelReasoningContract>,
    pub provenance: CapabilityProvenance,
}

#[cfg(test)]
impl ResolvedModelCapabilities {
    pub fn reasoning_modes(&self) -> Vec<String> {
        self.reasoning_contract
            .as_ref()
            .map(super::model_reasoning_contract::ModelReasoningContract::selection)
            .map(|(modes, _)| modes)
            .unwrap_or_default()
    }

    pub fn default_reasoning_mode(&self) -> Option<String> {
        self.reasoning_contract
            .as_ref()
            .and_then(|contract| contract.selection().1)
    }
}

pub fn resolve_local(provider_id: &str, model_id: &str) -> Option<ResolvedModelCapabilities> {
    if super::openrouter_model_metadata::owns_catalog_metadata(provider_id) {
        if let Some(model) = super::runtime_models::lookup(provider_id, model_id) {
            // The same enriched object feeds UI and backend, including its default.
            return Some(from_runtime(model));
        }
    }
    if provider_id == crate::services::codex_client::PROVIDER_ID {
        if let Some(model) = super::runtime_models::lookup(provider_id, model_id) {
            return Some(from_runtime(model));
        }
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

pub(super) async fn resolve_for_catalog(
    provider_id: &str,
    model_id: &str,
) -> Option<ResolvedModelCapabilities> {
    if let Some(model) = super::provider_model_lookup::local_entry(provider_id, model_id) {
        return Some(from_embedded_unrestricted(model));
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
        .and_then(|resolved| resolved.reasoning_contract)
        .map(|contract| contract.selection().0)
        .unwrap_or_default()
}

fn from_embedded(
    provider_id: &str,
    model_id: &str,
    model: ProviderModelConfig,
) -> ResolvedModelCapabilities {
    let mut resolved = from_embedded_unrestricted(model);
    resolved.reasoning_contract =
        restrict_runtime(provider_id, model_id, resolved.reasoning_contract);
    resolved
}

fn from_embedded_unrestricted(model: ProviderModelConfig) -> ResolvedModelCapabilities {
    let reasoning_contract = super::model_reasoning_contract::ModelReasoningContract::from_modes(
        model.supports_thinking,
        &model.reasoning_modes,
        model.default_reasoning_mode.as_deref(),
    );
    ResolvedModelCapabilities {
        supports_tools: model.supports_tools,
        supports_vision: model.supports_vision,
        supports_thinking: model.supports_thinking,
        supports_fast_mode: model.supports_fast_mode,
        reasoning_contract,
        provenance: CapabilityProvenance::EmbeddedRegistry,
    }
}

fn from_runtime(model: super::types::ModelInfo) -> ResolvedModelCapabilities {
    ResolvedModelCapabilities {
        supports_tools: model.supports_tools,
        supports_vision: model.supports_vision,
        supports_thinking: model.supports_thinking,
        supports_fast_mode: model.supports_fast_mode,
        reasoning_contract: model.reasoning_contract,
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
        reasoning_contract: super::model_reasoning_contract::ModelReasoningContract::from_modes(
            model.supports_thinking,
            &[],
            None,
        ),
        provenance: CapabilityProvenance::ValidatedRuntime,
    }
}

fn restrict_runtime(
    provider_id: &str,
    model_id: &str,
    contract: Option<super::model_reasoning_contract::ModelReasoningContract>,
) -> Option<super::model_reasoning_contract::ModelReasoningContract> {
    let contract = contract?;
    let Some(runtime) = super::runtime_models::lookup(provider_id, model_id) else {
        return Some(contract);
    };
    let Some(runtime_contract) = runtime.reasoning_contract else {
        return Some(contract);
    };
    let (runtime_modes, _) = runtime_contract.selection();
    let (modes, default_mode) = contract.selection();
    let modes = modes
        .into_iter()
        .filter(|mode| runtime_modes.contains(mode))
        .collect::<Vec<_>>();
    let default_mode = default_mode.filter(|mode| modes.contains(mode));
    super::model_reasoning_contract::ModelReasoningContract::from_modes(
        true,
        &modes,
        default_mode.as_deref(),
    )
}
