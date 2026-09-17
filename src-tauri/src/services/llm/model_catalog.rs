use std::collections::HashSet;

use super::openai_compat::OpenAiCompatProvider;
use super::route_profile::ClientSelector;
use super::types::{LlmError, ModelInfo};

pub async fn list_models_for(provider_id: &str) -> Result<Vec<ModelInfo>, LlmError> {
    if super::openrouter_model_metadata::owns_catalog_metadata(provider_id) {
        // The public loader already enriches and atomically publishes this list.
        return super::openrouter_catalog::list_models().await;
    }
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
    let canonical = super::route::canonical_provider_id(provider_id);
    let limit = super::catalog_limits::max_dynamic_models(canonical);
    if super::openrouter_model_metadata::owns_catalog_metadata(canonical) && models.len() > limit {
        return Err(invalid_catalog());
    }
    models.truncate(limit);
    let count_before_validation = models.len();
    models.retain(|model| crate::services::model_identifier::is_valid_model_id(&model.id));
    let invalid_ids = count_before_validation - models.len();
    if invalid_ids > 0 {
        ::log::warn!("[model catalog] ignored invalid identifiers count={invalid_ids}");
    }
    let mut seen = HashSet::with_capacity(models.len());
    let count_before_deduplication = models.len();
    // Keep native alias normalization (notably Google's resource names) unchanged.
    models.retain(|model| seen.insert(model.id.clone()));
    if super::openrouter_model_metadata::owns_catalog_metadata(canonical)
        && models.len() != count_before_deduplication
    {
        return Err(invalid_catalog());
    }
    let mut filtered = Vec::with_capacity(models.len());
    for model in models {
        // OpenRouter rows already crossed the output-modality filter; its batch
        // variants belong to the documented asynchronous API, not this chat path.
        let accepted = if super::openrouter_model_metadata::owns_catalog_metadata(canonical) {
            super::openrouter_model_metadata::supports_synchronous_chat(&model.id)
        } else {
            super::provider_model_lookup::is_chat_model(canonical, &model.id).await
        };
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
        repair_reasoning_default(model);
    }
    super::runtime_models::replace_provider(canonical, &filtered).map_err(|_| invalid_catalog())?;
    Ok(filtered)
}

async fn enrich_compat_model(provider_id: &str, model: &mut ModelInfo) {
    let remote_reasoning = model.reasoning_contract.clone();
    let remote_selection = remote_reasoning
        .as_ref()
        .map(super::model_reasoning_contract::ModelReasoningContract::selection);
    let local = super::provider_model_lookup::local_capabilities(provider_id, &model.id).is_some();
    let authoritative = super::openrouter_model_metadata::owns_catalog_metadata(provider_id);
    let resolved = if authoritative {
        super::provider_model_capabilities::resolve_for_catalog(provider_id, &model.id).await
    } else {
        super::provider_model_lookup::resolve(provider_id, &model.id).await
    };
    model.supports_fast_mode = resolved
        .as_ref()
        .is_some_and(|value| value.supports_fast_mode);
    if let Some(limits) = super::provider_model_lookup::local_limits(provider_id, &model.id) {
        if !authoritative {
            model.context_length = limits.context_window;
            model.max_output_tokens = limits.max_output_tokens;
        } else {
            model.context_length = model.context_length.or(limits.context_window);
            model.max_output_tokens = model.max_output_tokens.or(limits.max_output_tokens);
        }
    }
    let Some(capabilities) = resolved else { return };
    if authoritative {
        // Presence is independent for each capability: parameters do not describe vision.
        model.supports_tools = model
            .catalog_capabilities
            .tools
            .unwrap_or(capabilities.supports_tools);
        model.supports_vision = model
            .catalog_capabilities
            .vision
            .unwrap_or(capabilities.supports_vision);
        model.supports_thinking = model
            .catalog_capabilities
            .thinking
            .unwrap_or(capabilities.supports_thinking);
    } else if local {
        model.supports_tools = capabilities.supports_tools;
        model.supports_vision = capabilities.supports_vision;
        model.supports_thinking = capabilities.supports_thinking;
        let (base_modes, base_default) = capabilities
            .reasoning_contract
            .as_ref()
            .map(super::model_reasoning_contract::ModelReasoningContract::selection)
            .unwrap_or_default();
        let remote_modes = remote_selection.as_ref().map(|(modes, _)| modes.as_slice());
        let modes = crate::services::reasoning::restrict_to_dynamic_modes(base_modes, remote_modes);
        let default_mode = remote_selection
            .and_then(|(_, default)| default)
            .filter(|mode| modes.contains(mode))
            .or_else(|| base_default.filter(|mode| modes.contains(mode)));
        model.reasoning_contract =
            super::model_reasoning_contract::ModelReasoningContract::from_modes(
                model.supports_thinking,
                &modes,
                default_mode.as_deref(),
            );
    } else {
        model.supports_tools |= capabilities.supports_tools;
        model.supports_vision |= capabilities.supports_vision;
        model.supports_thinking |= capabilities.supports_thinking;
    }
    if !model.supports_thinking {
        model.reasoning_contract = None;
    } else if model.reasoning_contract.is_none() {
        model.reasoning_contract = capabilities.reasoning_contract.or_else(|| {
            super::model_reasoning_contract::ModelReasoningContract::from_modes(true, &[], None)
        });
    }
}

fn repair_reasoning_default(model: &mut ModelInfo) {
    if !model.supports_thinking {
        model.reasoning_contract = None;
    } else if model.reasoning_contract.is_none() {
        model.reasoning_contract =
            super::model_reasoning_contract::ModelReasoningContract::from_modes(true, &[], None);
    }
}

fn configuration_error() -> LlmError {
    LlmError::KnownProvider(super::provider_error::ProviderErrorCode::ProviderConfigurationInvalid)
}

fn invalid_catalog() -> LlmError {
    LlmError::Parse("catalogue distant invalide".to_string())
}
