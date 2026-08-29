use std::collections::HashSet;

use super::openai_compat::OpenAiCompatProvider;
use super::route_profile::ClientSelector;
use super::types::{LlmError, ModelInfo};

pub async fn list_models_for(provider_id: &str) -> Result<Vec<ModelInfo>, LlmError> {
    let profile = super::route_profile::find(provider_id).ok_or_else(configuration_error)?;
    let models = match profile.client {
        ClientSelector::Anthropic => super::anthropic::list_models().await?,
        ClientSelector::OpenAiCompat
        | ClientSelector::OpenAiResponses
        | ClientSelector::XaiOauth => {
            OpenAiCompatProvider::new(provider_id)?
                .list_models()
                .await?
        }
        ClientSelector::Codex | ClientSelector::OllamaLocal => {
            return Err(configuration_error());
        }
    };
    enrich_models(
        provider_id,
        models,
        profile.client == ClientSelector::Anthropic,
    )
    .await
}

pub async fn test_connection_for(provider_id: &str) -> Result<(), LlmError> {
    let profile = super::route_profile::find(provider_id).ok_or_else(configuration_error)?;
    match profile.client {
        ClientSelector::Anthropic => super::anthropic::test_connection().await,
        ClientSelector::OpenAiCompat
        | ClientSelector::OpenAiResponses
        | ClientSelector::XaiOauth => {
            OpenAiCompatProvider::new(provider_id)?
                .test_connection()
                .await
        }
        ClientSelector::Codex | ClientSelector::OllamaLocal => Err(configuration_error()),
    }
}

pub(super) async fn enrich_models(
    provider_id: &str,
    mut models: Vec<ModelInfo>,
    preserve_native_metadata: bool,
) -> Result<Vec<ModelInfo>, LlmError> {
    models.truncate(500);
    let mut seen = HashSet::with_capacity(models.len());
    models.retain(|model| seen.insert(model.id.clone()));
    let canonical = super::route::canonical_provider_id(provider_id);
    let mut filtered = Vec::with_capacity(models.len());
    for model in models {
        let accepted = super::provider_model_lookup::is_chat_model(canonical, &model.id).await;
        if accepted {
            filtered.push(model);
        }
    }
    for model in &mut filtered {
        model.context_usage_includes_reasoning =
            super::context_usage_includes_reasoning(provider_id).unwrap_or(true);
        if !preserve_native_metadata {
            enrich_compat_model(canonical, model).await;
        }
        restrict_to_live_reasoning(canonical, model);
        repair_reasoning_default(model);
    }
    super::runtime_models::replace_provider(canonical, &filtered);
    Ok(filtered)
}

fn restrict_to_live_reasoning(provider_id: &str, model: &mut ModelInfo) {
    let Some(route_id) =
        crate::services::reasoning_continuity::contract::RouteId::from_provider_id(provider_id)
    else {
        return;
    };
    model.reasoning_modes =
        crate::services::reasoning_continuity::registry::effective_reasoning_modes(
            route_id,
            &model.id,
            &model.reasoning_modes,
        );
    model.supports_thinking = !model.reasoning_modes.is_empty();
}

async fn enrich_compat_model(provider_id: &str, model: &mut ModelInfo) {
    let remote_modes = model.reasoning_modes.clone();
    let local = super::provider_model_lookup::local_capabilities(provider_id, &model.id).is_some();
    let resolved = super::provider_model_lookup::resolve(provider_id, &model.id).await;
    model.supports_fast_mode = resolved
        .as_ref()
        .is_some_and(|value| value.supports_fast_mode);
    if let Some(limits) = super::provider_model_lookup::local_limits(provider_id, &model.id) {
        model.context_length = limits.context_window;
        model.max_output_tokens = limits.max_output_tokens;
    }
    let Some(capabilities) = resolved else { return };
    if local {
        model.supports_tools = capabilities.supports_tools;
        model.supports_vision = capabilities.supports_vision;
        model.supports_thinking = capabilities.supports_thinking;
        model.reasoning_modes = crate::services::reasoning::restrict_to_dynamic_modes(
            capabilities.reasoning_modes.clone(),
            (!remote_modes.is_empty()).then_some(remote_modes.as_slice()),
        );
    } else {
        model.supports_tools |= capabilities.supports_tools;
        model.supports_vision |= capabilities.supports_vision;
        model.supports_thinking |= capabilities.supports_thinking;
    }
    if !model.supports_thinking {
        model.reasoning_modes.clear();
    } else if model.reasoning_modes.is_empty() {
        model.reasoning_modes = capabilities.reasoning_modes;
    }
    model.default_reasoning_mode = model
        .default_reasoning_mode
        .take()
        .filter(|mode| model.reasoning_modes.contains(mode))
        .or_else(|| {
            capabilities
                .default_reasoning_mode
                .filter(|mode| model.reasoning_modes.contains(mode))
        });
}

fn repair_reasoning_default(model: &mut ModelInfo) {
    if !model.supports_thinking {
        model.reasoning_modes.clear();
        model.default_reasoning_mode = None;
    } else if model
        .default_reasoning_mode
        .as_ref()
        .is_some_and(|mode| !model.reasoning_modes.contains(mode))
    {
        model.default_reasoning_mode = None;
    }
}

fn configuration_error() -> LlmError {
    LlmError::KnownProvider(super::provider_error::ProviderErrorCode::ProviderConfigurationInvalid)
}
