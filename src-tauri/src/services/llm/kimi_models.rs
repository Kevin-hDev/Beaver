use super::types::{LlmError, ModelInfo};
use serde_json::Value;

const MAX_MODELS: usize = 500;
const MAX_EFFORTS: usize = 8;
const MAX_DISPLAY_NAME_CHARS: usize = 96;

pub fn parse_models_list(body: &Value) -> Result<Vec<ModelInfo>, LlmError> {
    let data = body["data"]
        .as_array()
        .ok_or_else(|| LlmError::Parse("catalogue Kimi invalide".to_string()))?;

    data.iter()
        .take(MAX_MODELS)
        .filter_map(parse_model)
        .collect()
}

fn parse_model(value: &Value) -> Option<Result<ModelInfo, LlmError>> {
    let id = value["id"].as_str()?.to_string();
    if !super::runtime_models::valid_model_id(&id) {
        return None;
    }
    let context_length = match positive_u32(&value["context_length"]) {
        Some(length) => length,
        None => return Some(Err(LlmError::Parse("catalogue Kimi invalide".to_string()))),
    };
    let thinking_type = value["supports_thinking_type"].as_str();
    let mut reasoning_modes = parse_efforts(&value["think_efforts"]);
    let supports_thinking = match thinking_type {
        Some("only" | "both") => true,
        Some("no") => false,
        _ => {
            declared_bool(value, "supports_reasoning", false)
                || !reasoning_modes.is_empty()
                || super::providers::moonshot::supports_thinking(&id)
        }
    };
    if !supports_thinking {
        reasoning_modes.clear();
    } else if reasoning_modes.is_empty() {
        reasoning_modes = crate::services::reasoning::supported_modes("moonshot", &id, true)
            .iter()
            .map(|mode| mode.to_string())
            .collect();
    }
    if thinking_type == Some("only") {
        reasoning_modes.retain(|mode| mode != "off");
    } else if thinking_type == Some("both")
        && !reasoning_modes.is_empty()
        && !reasoning_modes.iter().any(|mode| mode == "off")
    {
        reasoning_modes.insert(0, "off".to_string());
    }
    let default_reasoning_mode = parse_default_effort(&value["think_efforts"])
        .filter(|mode| reasoning_modes.contains(mode))
        .or_else(|| super::providers::moonshot::default_reasoning_mode(&id).map(str::to_string));

    Some(Ok(ModelInfo {
        display_name: safe_display_name(&value["display_name"])
            .or_else(|| known_display_name(&id).map(str::to_string)),
        owned_by: Some("moonshot".to_string()),
        context_length: Some(context_length),
        supports_tools: declared_bool(
            value,
            "supports_tool_use",
            super::providers::moonshot::supports_tools(&id),
        ),
        supports_vision: declared_bool(
            value,
            "supports_image_in",
            super::providers::moonshot::supports_vision(&id),
        ),
        supports_thinking,
        reasoning_modes,
        default_reasoning_mode,
        is_free: true,
        id,
    }))
}

fn parse_efforts(value: &Value) -> Vec<String> {
    if value["support"].as_bool() != Some(true) {
        return Vec::new();
    }
    let mut modes = Vec::new();
    for raw in value["valid_efforts"]
        .as_array()
        .into_iter()
        .flatten()
        .take(MAX_EFFORTS)
        .filter_map(Value::as_str)
    {
        let Some(mode) = crate::services::reasoning::sanitize_mode(Some(raw.to_string())) else {
            continue;
        };
        if !modes.contains(&mode) {
            modes.push(mode);
        }
    }
    modes
}

fn parse_default_effort(value: &Value) -> Option<String> {
    if value["support"].as_bool() != Some(true) {
        return None;
    }
    crate::services::reasoning::sanitize_mode(value["default_effort"].as_str().map(str::to_string))
}

fn declared_bool(value: &Value, key: &str, fallback: bool) -> bool {
    match value.get(key) {
        None => fallback,
        Some(declared) => declared.as_bool().unwrap_or(false),
    }
}

fn positive_u32(value: &Value) -> Option<u32> {
    let length = value.as_u64()?;
    if length == 0 {
        return None;
    }
    length.try_into().ok()
}

fn safe_display_name(value: &Value) -> Option<String> {
    let name = value.as_str()?.trim();
    let count = name.chars().count();
    (count > 0 && count <= MAX_DISPLAY_NAME_CHARS && !name.chars().any(char::is_control))
        .then(|| name.to_string())
}

fn known_display_name(model: &str) -> Option<&'static str> {
    match model {
        "k3" => Some("K3"),
        "kimi-for-coding" => Some("K2.7 Coding"),
        "kimi-for-coding-highspeed" => Some("K2.7 Coding Highspeed"),
        _ => None,
    }
}
