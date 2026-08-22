use std::collections::HashSet;

use serde_json::{Map, Value};

const MAX_MODELS: usize = 500;
const MAX_NAME_CHARS: usize = 128;
const MAX_CONTEXT_TOKENS: u32 = 4_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XaiBackend {
    ChatCompletions,
    Responses,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XaiCatalogModel {
    pub id: String,
    pub display_name: String,
    pub backend: XaiBackend,
    pub context_window: u32,
    pub max_output_tokens: Option<u32>,
    pub reasoning_modes: Vec<String>,
    pub default_reasoning_mode: Option<String>,
}

pub fn parse_catalog(body: &Value) -> Result<Vec<XaiCatalogModel>, &'static str> {
    let data = body.get("data").and_then(Value::as_array).ok_or("data")?;
    if data.is_empty() || data.len() > MAX_MODELS {
        return Err("model_count");
    }
    let mut ids = HashSet::with_capacity(data.len());
    let mut models = Vec::with_capacity(data.len());
    for value in data {
        let object = value.as_object().ok_or("model")?;
        reject_remote_route(object)?;
        let id = string(object, &["model", "modelId", "id"]).ok_or("model_id")?;
        if !crate::services::llm::runtime_models::valid_model_id(id) {
            return Err("model_id");
        }
        if !ids.insert(id.to_string()) {
            return Err("duplicate_model");
        }
        if !is_visible_text_chat_model(object)? {
            continue;
        }
        let model = parse_model(object, id)?;
        models.push(model);
    }
    if models.is_empty() {
        Err("model_count")
    } else {
        Ok(models)
    }
}

fn is_visible_text_chat_model(object: &Map<String, Value>) -> Result<bool, &'static str> {
    if optional_bool(object, &["hidden"])?.unwrap_or(false)
        || optional_bool(object, &["supportedInApi", "supported_in_api"])? == Some(false)
    {
        return Ok(false);
    }
    Ok(
        has_text_modality(object, &["inputModalities", "input_modalities"])?
            && has_text_modality(object, &["outputModalities", "output_modalities"])?,
    )
}

fn has_text_modality(object: &Map<String, Value>, fields: &[&str]) -> Result<bool, &'static str> {
    let Some(value) = fields.iter().find_map(|field| object.get(*field)) else {
        // Le catalogue /models-v2 officiel omet encore ces champs pour certains modèles.
        return Ok(true);
    };
    let values = value.as_array().ok_or("modalities")?;
    if values.is_empty() || values.len() > 16 {
        return Err("modalities");
    }
    let mut has_text = false;
    for value in values {
        let modality = value.as_str().ok_or("modalities")?;
        if modality.is_empty() || modality.len() > 32 || modality.chars().any(char::is_control) {
            return Err("modalities");
        }
        has_text |= modality.eq_ignore_ascii_case("text");
    }
    Ok(has_text)
}

fn optional_bool(
    object: &Map<String, Value>,
    fields: &[&str],
) -> Result<Option<bool>, &'static str> {
    fields
        .iter()
        .find_map(|field| object.get(*field))
        .map(|value| value.as_bool().ok_or("model_visibility"))
        .transpose()
}

fn parse_model(object: &Map<String, Value>, id: &str) -> Result<XaiCatalogModel, &'static str> {
    let display_name = string(object, &["name"]).unwrap_or(id);
    if display_name.is_empty()
        || display_name.chars().count() > MAX_NAME_CHARS
        || display_name.chars().any(char::is_control)
    {
        return Err("display_name");
    }
    let context_window = number(object, &["contextWindow", "context_window"])
        .and_then(|value| u32::try_from(value).ok())
        .filter(|value| (1..=MAX_CONTEXT_TOKENS).contains(value))
        .ok_or("context_window")?;
    let max_output_tokens = match number(object, &["maxCompletionTokens", "max_completion_tokens"])
    {
        Some(value) => Some(
            u32::try_from(value)
                .ok()
                .filter(|value| *value > 0)
                .ok_or("output_limit")?,
        ),
        None => None,
    };
    if max_output_tokens.is_some_and(|value| value > context_window) {
        return Err("output_limit");
    }
    let backend = match string(object, &["apiBackend", "api_backend"]) {
        Some("chat_completions") => XaiBackend::ChatCompletions,
        Some("responses") => XaiBackend::Responses,
        _ => return Err("backend"),
    };
    let reasoning_modes = reasoning_modes(object)?;
    let default_reasoning_mode =
        string(object, &["reasoningEffort", "reasoning_effort"]).map(str::to_string);
    if default_reasoning_mode
        .as_ref()
        .is_some_and(|mode| !reasoning_modes.iter().any(|candidate| candidate == mode))
    {
        return Err("reasoning_default");
    }
    Ok(XaiCatalogModel {
        id: id.to_string(),
        display_name: display_name.to_string(),
        backend,
        context_window,
        max_output_tokens,
        reasoning_modes,
        default_reasoning_mode,
    })
}

fn reasoning_modes(object: &Map<String, Value>) -> Result<Vec<String>, &'static str> {
    let Some(values) = object
        .get("reasoningEfforts")
        .or_else(|| object.get("reasoning_efforts"))
    else {
        return Ok(Vec::new());
    };
    let values = values.as_array().ok_or("reasoning_modes")?;
    if values.len() > 8 {
        return Err("reasoning_modes");
    }
    let mut seen = HashSet::with_capacity(values.len());
    let mut modes = Vec::with_capacity(values.len());
    for value in values {
        let mode = value
            .as_str()
            .or_else(|| value.get("value").and_then(Value::as_str))
            .ok_or("reasoning_modes")?;
        if !matches!(mode, "low" | "medium" | "high" | "xhigh") || !seen.insert(mode) {
            return Err("reasoning_modes");
        }
        modes.push(mode.to_string());
    }
    Ok(modes)
}

fn reject_remote_route(object: &Map<String, Value>) -> Result<(), &'static str> {
    for field in ["baseUrl", "base_url", "apiBaseUrl", "api_base_url"] {
        if let Some(route) = object.get(field).filter(|value| !value.is_null()) {
            let allowed = route.as_str().is_some_and(|value| {
                value.trim_end_matches('/')
                    == super::xai_headers::PROXY_BASE_URL.trim_end_matches('/')
            });
            if !allowed {
                return Err("remote_route");
            }
        }
    }
    if object
        .get("extraHeaders")
        .or_else(|| object.get("extra_headers"))
        .is_some_and(|value| !value.as_object().is_some_and(Map::is_empty) && !value.is_null())
    {
        return Err("remote_route");
    }
    Ok(())
}

fn string<'a>(object: &'a Map<String, Value>, fields: &[&str]) -> Option<&'a str> {
    fields
        .iter()
        .find_map(|field| object.get(*field).and_then(Value::as_str))
}

fn number(object: &Map<String, Value>, fields: &[&str]) -> Option<u64> {
    fields
        .iter()
        .find_map(|field| object.get(*field).and_then(Value::as_u64))
}
