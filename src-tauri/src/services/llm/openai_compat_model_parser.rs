use super::model_metadata::has_zero_pricing;
use super::types::{LlmError, ModelInfo};
use serde_json::Value;
use std::collections::HashSet;

pub(super) fn parse_models_list(
    body: &Value,
    provider_id: &str,
) -> Result<Vec<ModelInfo>, LlmError> {
    let data = body["data"].as_array().ok_or_else(|| {
        LlmError::Parse(format!("champ 'data' absent ou invalide ({provider_id})"))
    })?;
    let limit = super::catalog_limits::max_dynamic_models(provider_id);
    if provider_id == "openrouter" && data.len() > limit {
        return Err(invalid_catalog(provider_id));
    }

    let mut models = Vec::with_capacity(data.len().min(limit));
    let mut raw_ids = HashSet::with_capacity(data.len().min(limit));
    let mut invalid_ids = 0usize;
    let mut degraded_reasoning_contracts = 0usize;
    for model in data.iter().take(limit) {
        let raw_id = model["id"].as_str();
        if !raw_id.is_some_and(super::runtime_models::valid_model_id) {
            invalid_ids += 1;
            continue;
        }
        if provider_id == "openrouter" && raw_id.is_some_and(|id| !raw_ids.insert(id)) {
            return Err(invalid_catalog(provider_id));
        }
        if let Some(model) = parse_model(model, provider_id, &mut degraded_reasoning_contracts) {
            models.push(model);
        }
    }
    if invalid_ids > 0 {
        ::log::warn!("[model catalog] ignored invalid identifiers count={invalid_ids}");
    }
    if degraded_reasoning_contracts > 0 {
        ::log::warn!(
            "event=openrouter_reasoning_metadata_degraded reason=unknown_contract count={degraded_reasoning_contracts}"
        );
    }
    Ok(models)
}

fn parse_model(
    model: &Value,
    provider_id: &str,
    degraded_reasoning_contracts: &mut usize,
) -> Option<ModelInfo> {
    let id = model["id"].as_str()?;
    let id = super::route_profile::catalog_model_id(provider_id, id);
    if !super::runtime_models::valid_model_id(id) {
        return None;
    }
    let local_limits = super::provider_model_lookup::local_limits(provider_id, id);
    let authoritative = super::openrouter_model_metadata::owns_catalog_metadata(provider_id);
    let context_length = if authoritative {
        super::openai_compat_model_limits::remote_context(model, true)
            .or_else(|| local_limits.and_then(|limits| limits.context_window))
    } else {
        local_limits
            .and_then(|limits| limits.context_window)
            .or_else(|| super::openai_compat_model_limits::remote_context(model, false))
    };
    let max_output_tokens = if authoritative {
        super::openai_compat_model_limits::remote_output_limit(model)
            .or_else(|| local_limits.and_then(|limits| limits.max_output_tokens))
    } else {
        local_limits
            .and_then(|limits| limits.max_output_tokens)
            .or_else(|| super::model_metadata::output_limit(model))
    };
    if authoritative && !supports_text_output(model) {
        return None;
    }
    let supported_parameters = supported_parameters(model)?;
    let has_param = |name: &str| {
        supported_parameters
            .as_deref()
            .unwrap_or_default()
            .iter()
            .any(|parameter| parameter == name)
    };
    let resolved = super::provider_model_lookup::resolve_remote_list_defaults(provider_id, id);
    let supports_tools = has_param("tools")
        || model["capabilities"]["function_calling"]
            .as_bool()
            .unwrap_or(false)
        || resolved
            .as_ref()
            .is_some_and(|capabilities| capabilities.supports_tools);
    let is_chat = model["capabilities"]["completion_chat"]
        .as_bool()
        .unwrap_or(true);
    if !is_chat && model["capabilities"].is_object() {
        return None;
    }
    let supports_vision = model["capabilities"]["vision"].as_bool().unwrap_or(false)
        || architecture_supports_vision(model)
        || resolved
            .as_ref()
            .is_some_and(|capabilities| capabilities.supports_vision);
    let supports_thinking = has_param("reasoning")
        || has_param("reasoning_effort")
        || has_param("include_reasoning")
        || resolved
            .as_ref()
            .is_some_and(|capabilities| capabilities.supports_thinking);
    // `supported_parameters` annonce une fonctionnalité, pas les niveaux permis.
    // Le catalogue dynamique reste vide tant qu'il ne publie pas ces valeurs.
    let reasoning_value = &model["reasoning"];
    let reasoning_metadata = if authoritative {
        match super::openrouter_model_metadata::reasoning(reasoning_value) {
            Some(contract) => Some(contract),
            None if !reasoning_value.is_null() => {
                // A new upstream control must not hide an otherwise usable model.
                // ProviderDefault sends no invented effort and keeps the route fail-closed.
                *degraded_reasoning_contracts = degraded_reasoning_contracts.saturating_add(1);
                Some(super::model_reasoning_contract::ModelReasoningContract {
                    mandatory: None,
                    default_enabled: None,
                    supports_max_tokens: None,
                    default_effort: None,
                    control: super::model_reasoning_contract::ReasoningControl::ProviderDefault,
                })
            }
            None => None,
        }
    } else {
        None
    };
    let (reasoning_modes, default_reasoning_mode) = reasoning_metadata
        .as_ref()
        .map(super::model_reasoning_contract::ModelReasoningContract::legacy_projection)
        .unwrap_or_else(|| (Vec::new(), None));

    let catalog_capabilities = if authoritative {
        super::openrouter_model_metadata::capabilities(
            model,
            supported_parameters.as_deref(),
            reasoning_metadata.is_some(),
        )
    } else {
        Default::default()
    };
    Some(ModelInfo {
        id: id.to_string(),
        display_name: None,
        owned_by: safe_owner(&model["owned_by"]),
        context_length,
        max_output_tokens,
        supported_parameters,
        catalog_capabilities,
        supports_tools: catalog_capabilities.tools.unwrap_or(supports_tools),
        supports_vision: catalog_capabilities.vision.unwrap_or(supports_vision),
        supports_thinking: catalog_capabilities.thinking.unwrap_or(supports_thinking),
        reasoning_contract: reasoning_metadata,
        supports_fast_mode: false,
        reasoning_modes,
        default_reasoning_mode,
        context_usage_includes_reasoning: true,
        // Un badge gratuit exige un tarif nul explicite pour toutes les unités facturées.
        is_free: has_zero_pricing(&model["pricing"]),
    })
}

fn invalid_catalog(provider_id: &str) -> LlmError {
    LlmError::Parse(format!("catalogue distant invalide ({provider_id})"))
}

fn architecture_supports_vision(model: &Value) -> bool {
    model["architecture"]["modality"]
        .as_str()
        .is_some_and(|value| value.contains("image->") || value.contains("image+"))
        || model["architecture"]["input_modalities"]
            .as_array()
            .is_some_and(|values| values.iter().any(|value| value.as_str() == Some("image")))
}

fn supported_parameters(model: &Value) -> Option<Option<Vec<String>>> {
    let Some(value) = model.get("supported_parameters") else {
        return Some(None);
    };
    if value.is_null() {
        return Some(None);
    }
    let values = value.as_array()?;
    if values.len() > 64 {
        return None;
    }
    values
        .iter()
        .map(|value| safe_text(value, 64))
        .collect::<Option<Vec<_>>>()
        .map(Some)
}

fn supports_text_output(model: &Value) -> bool {
    let value = &model["architecture"]["output_modalities"];
    if value.is_null() {
        return true;
    }
    let Some(values) = value.as_array().filter(|values| values.len() <= 8) else {
        return false;
    };
    values
        .iter()
        .map(|value| safe_text(value, 32))
        .collect::<Option<Vec<_>>>()
        .is_some_and(|modalities| modalities.iter().any(|modality| modality == "text"))
}

fn safe_owner(value: &Value) -> Option<String> {
    safe_text(value, 96)
}

fn safe_text(value: &Value, max_bytes: usize) -> Option<String> {
    value
        .as_str()
        .filter(|text| {
            !text.is_empty() && text.len() <= max_bytes && !text.chars().any(char::is_control)
        })
        .map(str::to_string)
}
