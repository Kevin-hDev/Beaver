use serde_json::Value;

use crate::services::llm::provider_model_registry::{self, ProviderModelConfig};
use crate::services::llm::types::{LlmError, ModelInfo};

const PROVIDER_ID: &str = "anthropic";
const MAX_MODELS: usize = 500;

pub(super) fn parse_catalog(body: &Value) -> Result<Vec<ModelInfo>, LlmError> {
    let data = body
        .get("data")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid_catalog("missing_data"))?;
    if data.len() > MAX_MODELS {
        return Err(invalid_catalog("model_count"));
    }
    data.iter()
        .filter(|item| {
            let valid = item
                .get("id")
                .and_then(Value::as_str)
                .is_some_and(crate::services::llm::runtime_models::valid_model_id);
            if !valid {
                log::warn!(
                    "provider=anthropic event=model_catalog_entry_skipped reason=invalid_model_id"
                );
            }
            valid
        })
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
    let id = item
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| invalid_catalog("model_id"))?;
    let local = provider_model_registry::lookup(PROVIDER_ID, id);
    let supports_thinking = capability(item, &["thinking"])
        .or_else(|| local.as_ref().map(|model| model.supports_thinking))
        .unwrap_or(false);
    let (reasoning_modes, default_reasoning_mode) =
        reasoning_contract(item, local.as_ref(), supports_thinking);
    Ok(ModelInfo {
        id: id.to_string(),
        display_name: optional_text(item, "display_name")?,
        owned_by: Some(PROVIDER_ID.to_string()),
        context_length: optional_u32(item, "max_input_tokens")?
            .or_else(|| local.as_ref().map(|model| model.context_window)),
        max_output_tokens: optional_u32(item, "max_tokens")?
            .or_else(|| local.as_ref().and_then(|model| model.max_output_tokens)),
        supports_tools: capability(item, &["tools", "tool_use"])
            .or_else(|| local.as_ref().map(|model| model.supports_tools))
            // L'API Models Anthropic ne publie pas cette capacité. Tous les
            // modèles Claude disponibles le 2026-08-29 acceptent les tools ;
            // un refus explicite distant ou embarqué garde toutefois priorité.
            .unwrap_or(true),
        supports_vision: capability(item, &["image_input"])
            .or_else(|| local.as_ref().map(|model| model.supports_vision))
            .unwrap_or(false),
        supports_thinking,
        supports_fast_mode: false,
        reasoning_modes,
        default_reasoning_mode,
        context_usage_includes_reasoning: true,
        is_free: local.as_ref().is_some_and(|model| model.is_free),
    })
}

fn reasoning_contract(
    item: &Value,
    local: Option<&ProviderModelConfig>,
    supports_thinking: bool,
) -> (Vec<String>, Option<String>) {
    if !supports_thinking {
        return (Vec::new(), None);
    }
    if nested_supported(item, &["capabilities", "thinking", "types", "adaptive"]) {
        let mut modes = if always_adaptive(item) {
            vec!["auto".to_string()]
        } else {
            vec!["off".to_string(), "auto".to_string()]
        };
        for effort in ["low", "medium", "high", "xhigh", "max"] {
            if nested_supported(item, &["capabilities", "effort", effort]) {
                modes.push(effort.to_string());
            }
        }
        let default = modes
            .iter()
            .any(|mode| mode == "high")
            .then(|| "high".to_string())
            .or_else(|| Some("auto".to_string()));
        return (modes, default);
    }
    if nested_supported(item, &["capabilities", "thinking", "types", "enabled"]) {
        return (
            ["off", "low", "medium", "high"]
                .into_iter()
                .map(str::to_string)
                .collect(),
            Some("medium".to_string()),
        );
    }
    local.map_or_else(
        || (Vec::new(), None),
        |model| {
            (
                model.reasoning_modes.clone(),
                model.default_reasoning_mode.clone(),
            )
        },
    )
}

fn always_adaptive(item: &Value) -> bool {
    matches!(
        item.get("id").and_then(Value::as_str),
        Some("claude-fable-5" | "claude-mythos-5" | "claude-mythos-preview")
    )
}

fn nested_supported(item: &Value, path: &[&str]) -> bool {
    path.iter()
        .try_fold(item, |value, key| value.get(*key))
        .and_then(|value| value.get("supported"))
        .and_then(Value::as_bool)
        .unwrap_or(false)
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
    if value.is_null() {
        return Ok(None);
    }
    let raw = value.as_u64().ok_or_else(|| invalid_catalog(field))?;
    if raw == 0 {
        return Ok(None);
    }
    let parsed = u32::try_from(raw).map_err(|_| invalid_catalog(field))?;
    Ok(Some(parsed))
}

fn optional_text(item: &Value, field: &str) -> Result<Option<String>, LlmError> {
    let Some(value) = item.get(field) else {
        return Ok(None);
    };
    let text = value.as_str().ok_or_else(|| invalid_catalog(field))?;
    if text.is_empty() || text.len() > 160 || text.chars().any(char::is_control) {
        return Err(invalid_catalog(field));
    }
    Ok(Some(text.to_string()))
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
