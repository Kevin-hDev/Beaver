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
    let context_length = local_limits
        .and_then(|limits| limits.context_window)
        .or_else(|| remote_context(model));
    let max_output_tokens = local_limits
        .map(|limits| limits.max_output_tokens)
        .unwrap_or_else(|| super::model_metadata::output_limit(model));
    let supported_parameters = supported_parameters(model);
    let has_param = |name: &str| {
        supported_parameters
            .iter()
            .any(|parameter| parameter == name)
    };
    let supports_tools = has_param("tools")
        || model["capabilities"]["function_calling"]
            .as_bool()
            .unwrap_or(false)
        || super::tool_capable::supports_tools(provider_id, id);
    let is_chat = model["capabilities"]["completion_chat"]
        .as_bool()
        .unwrap_or(true);
    if !is_chat && model["capabilities"].is_object() {
        return None;
    }
    let supports_vision = model["capabilities"]["vision"].as_bool().unwrap_or(false)
        || architecture_supports_vision(model)
        || super::tool_capable::supports_vision(provider_id, id);
    let supports_thinking = has_param("reasoning")
        || has_param("reasoning_effort")
        || has_param("include_reasoning")
        || super::tool_capable::supports_thinking(provider_id, id);
    let reasoning_modes = if supports_thinking {
        crate::services::reasoning::supported_modes(provider_id, id, true)
            .iter()
            .map(|mode| mode.to_string())
            .collect()
    } else {
        Vec::new()
    };

    Some(ModelInfo {
        id: id.to_string(),
        display_name: None,
        owned_by: safe_owner(&model["owned_by"]),
        context_length,
        max_output_tokens,
        supports_tools,
        supports_vision,
        supports_thinking,
        reasoning_modes,
        default_reasoning_mode: None,
        // Un badge gratuit exige un tarif nul explicite pour toutes les unités facturées.
        is_free: has_zero_pricing(&model["pricing"]),
    })
}

fn remote_context(model: &Value) -> Option<u32> {
    [
        &model["context_length"],
        &model["context_window"],
        &model["max_context_length"],
    ]
    .into_iter()
    .find_map(super::model_metadata::positive_u32)
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
