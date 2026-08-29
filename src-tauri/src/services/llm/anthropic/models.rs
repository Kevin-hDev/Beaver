use serde_json::Value;

use crate::services::llm::provider_model_registry::{self, ProviderModelConfig};
use crate::services::llm::types::{LlmError, ModelInfo};

const PROVIDER_ID: &str = "anthropic";
const VALIDATED_MODEL_ID: &str = "claude-haiku-4-5-20251001";
const MAX_MODELS: usize = 500;

pub(super) fn parse_and_intersect(body: &Value) -> Result<Vec<ModelInfo>, LlmError> {
    let data = body
        .get("data")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid_catalog("missing_data"))?;
    if data.len() > MAX_MODELS {
        return Err(invalid_catalog("model_count"));
    }
    for item in data {
        let id = item
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| invalid_catalog("model_id"))?;
        if !crate::services::llm::runtime_models::valid_model_id(id) {
            return Err(invalid_catalog("model_id"));
        }
    }
    data.iter()
        .filter(|item| item.get("id").and_then(Value::as_str) == Some(VALIDATED_MODEL_ID))
        .map(merge_remote_model)
        .collect()
}

pub(super) fn resolve_catalog(
    remote: Result<Vec<ModelInfo>, LlmError>,
) -> Result<Vec<ModelInfo>, LlmError> {
    match remote {
        Ok(models) => Ok(models),
        Err(
            LlmError::Network(_)
            | LlmError::Http {
                status: 502..=504, ..
            },
        ) => Ok(embedded_models()),
        Err(error) => Err(error),
    }
}

fn merge_remote_model(item: &Value) -> Result<ModelInfo, LlmError> {
    let local = provider_model_registry::lookup(PROVIDER_ID, VALIDATED_MODEL_ID)
        .ok_or_else(|| invalid_catalog("missing_embedded_model"))?;
    Ok(ModelInfo {
        id: VALIDATED_MODEL_ID.to_string(),
        display_name: item
            .get("display_name")
            .and_then(Value::as_str)
            .map(str::to_owned),
        owned_by: Some(PROVIDER_ID.to_string()),
        context_length: optional_u32(item, "max_input_tokens")?.or(Some(local.context_window)),
        max_output_tokens: optional_u32(item, "max_tokens")?.or(local.max_output_tokens),
        supports_tools: capability(item, &["tools", "tool_use"]).unwrap_or(local.supports_tools),
        supports_vision: capability(item, &["image_input"]).unwrap_or(local.supports_vision),
        supports_thinking: capability(item, &["thinking"]).unwrap_or(local.supports_thinking),
        supports_fast_mode: false,
        reasoning_modes: local.reasoning_modes,
        default_reasoning_mode: local.default_reasoning_mode,
        context_usage_includes_reasoning: true,
        is_free: local.is_free,
    })
}

fn capability(item: &Value, names: &[&str]) -> Option<bool> {
    let capabilities = item.get("capabilities")?.as_object()?;
    names.iter().find_map(|name| {
        let value = capabilities.get(*name)?;
        value
            .as_bool()
            .or_else(|| value.get("supported").and_then(Value::as_bool))
    })
}

fn optional_u32(item: &Value, field: &str) -> Result<Option<u32>, LlmError> {
    let Some(value) = item.get(field) else {
        return Ok(None);
    };
    let raw = value.as_u64().ok_or_else(|| invalid_catalog(field))?;
    let parsed = u32::try_from(raw).map_err(|_| invalid_catalog(field))?;
    (parsed > 0)
        .then_some(parsed)
        .ok_or_else(|| invalid_catalog(field))
        .map(Some)
}

pub(super) fn embedded_models() -> Vec<ModelInfo> {
    provider_model_registry::list(PROVIDER_ID)
        .into_iter()
        .map(from_embedded)
        .collect()
}

fn from_embedded(model: ProviderModelConfig) -> ModelInfo {
    ModelInfo {
        id: model.id,
        display_name: None,
        owned_by: Some(PROVIDER_ID.to_string()),
        context_length: Some(model.context_window),
        max_output_tokens: model.max_output_tokens,
        supports_tools: model.supports_tools,
        supports_vision: model.supports_vision,
        supports_thinking: model.supports_thinking,
        supports_fast_mode: false,
        reasoning_modes: model.reasoning_modes,
        default_reasoning_mode: model.default_reasoning_mode,
        context_usage_includes_reasoning: true,
        is_free: model.is_free,
    }
}

fn invalid_catalog(code: &str) -> LlmError {
    ::log::warn!("[anthropic models] catalogue refused code={code}");
    LlmError::KnownProvider(
        crate::services::llm::provider_error::ProviderErrorCode::ModelCatalogUnavailable,
    )
}
