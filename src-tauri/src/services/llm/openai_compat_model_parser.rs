use super::types::{LlmError, ModelInfo};
use serde_json::Value;

pub(super) fn parse_models_list(
    body: &Value,
    provider_id: &str,
) -> Result<Vec<ModelInfo>, LlmError> {
    let data = body["data"].as_array().ok_or_else(|| {
        LlmError::Parse(format!("champ 'data' absent ou invalide ({provider_id})"))
    })?;

    Ok(data
        .iter()
        .take(500)
        .filter_map(|model| parse_model(model, provider_id))
        .collect())
}

fn parse_model(model: &Value, provider_id: &str) -> Option<ModelInfo> {
    let id = model["id"].as_str()?;
    if !super::runtime_models::valid_model_id(id) {
        return None;
    }
    let local_limits = super::provider_model_lookup::local_limits(provider_id, id);
    let authoritative = super::openrouter_model_metadata::owns_catalog_metadata(provider_id);
    let context_length = if authoritative {
        remote_context(model, true)
            .or_else(|| local_limits.and_then(|limits| limits.context_window))
    } else {
        local_limits
            .and_then(|limits| limits.context_window)
            .or_else(|| remote_context(model, false))
    };
    let max_output_tokens = if authoritative {
        remote_output_limit(model)
            .or_else(|| local_limits.and_then(|limits| limits.max_output_tokens))
    } else {
        local_limits
            .and_then(|limits| limits.max_output_tokens)
            .or_else(|| super::model_metadata::output_limit(model))
    };
    let supported_parameters = supported_parameters(model);
    let has_param = |name: &str| {
        supported_parameters
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
    let reasoning_metadata = if authoritative {
        super::openrouter_model_metadata::reasoning(&model["reasoning"])
    } else {
        None
    };
    let (reasoning_modes, default_reasoning_mode) = reasoning_metadata
        .clone()
        .unwrap_or_else(|| (Vec::new(), None));

    Some(ModelInfo {
        id: id.to_string(),
        display_name: None,
        owned_by: safe_owner(&model["owned_by"]),
        context_length,
        max_output_tokens,
        supports_tools,
        supports_vision,
        supports_thinking,
        reasoning_metadata_present: reasoning_metadata.is_some(),
        supports_fast_mode: false,
        reasoning_modes,
        default_reasoning_mode,
        context_usage_includes_reasoning: true,
        // Un badge gratuit exige un tarif nul explicite pour toutes les unités facturées.
        is_free: has_zero_pricing(&model["pricing"]),
    })
}

fn remote_output_limit(model: &Value) -> Option<u32> {
    [
        model.pointer("/top_provider/max_completion_tokens"),
        model.pointer("/limits/max_completion_tokens"),
        model.get("max_output_tokens"),
        model.get("max_completion_tokens"),
        model.get("max_tokens"),
    ]
    .into_iter()
    .flatten()
    .filter_map(super::model_metadata::positive_u32)
    .min()
}

fn remote_context(model: &Value, use_top_provider_limit: bool) -> Option<u32> {
    let advertised = [
        &model["context_length"],
        &model["context_window"],
        &model["max_context_length"],
    ]
    .into_iter()
    .find_map(super::model_metadata::positive_u32);
    if !use_top_provider_limit {
        return advertised;
    }
    if advertised.is_none() {
        return super::model_metadata::positive_u32(&model["top_provider"]["context_length"]);
    }
    let top_provider =
        super::model_metadata::positive_u32(&model["top_provider"]["context_length"]);
    match (advertised, top_provider) {
        (Some(advertised), Some(top_provider)) => Some(advertised.min(top_provider)),
        (Some(advertised), None) => Some(advertised),
        (None, Some(top_provider)) => Some(top_provider),
        (None, None) => None,
    }
}

fn architecture_supports_vision(model: &Value) -> bool {
    model["architecture"]["modality"]
        .as_str()
        .is_some_and(|value| value.contains("image->") || value.contains("image+"))
        || model["architecture"]["input_modalities"]
            .as_array()
            .is_some_and(|values| values.iter().any(|value| value.as_str() == Some("image")))
}

fn supported_parameters(model: &Value) -> Vec<String> {
    model["supported_parameters"]
        .as_array()
        .map(|values| {
            values
                .iter()
                .filter_map(|value| safe_text(value, 64))
                .take(64)
                .collect()
        })
        .unwrap_or_default()
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

fn has_zero_pricing(pricing: &Value) -> bool {
    let Some(prices) = pricing.as_object() else {
        return false;
    };
    let Some(prompt) = prices.get("prompt") else {
        return false;
    };
    let Some(completion) = prices.get("completion") else {
        return false;
    };
    price_is_zero(prompt) && price_is_zero(completion) && prices.values().all(price_is_zero)
}

fn price_is_zero(value: &Value) -> bool {
    let price = value
        .as_str()
        .and_then(|raw| raw.parse::<f64>().ok())
        .or_else(|| value.as_f64());
    price.is_some_and(|amount| amount.is_finite() && amount == 0.0)
}
