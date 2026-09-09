use serde_json::Value;

use super::model_reasoning_contract::{ModelReasoningContract, ReasoningControl};
use crate::services::reasoning_continuity::contract::ReasoningModeId;

const MAX_REASONING_EFFORTS: usize = 8;

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct CatalogCapabilities {
    pub tools: Option<bool>,
    pub vision: Option<bool>,
    pub thinking: Option<bool>,
}

pub(super) fn capabilities(
    model: &Value,
    parameters: Option<&[String]>,
    reasoning_present: bool,
) -> CatalogCapabilities {
    let has_parameter = |names: &[&str]| {
        parameters.map(|values| values.iter().any(|value| names.contains(&value.as_str())))
    };
    let input = &model["architecture"]["input_modalities"];
    let vision = model["capabilities"]["vision"]
        .as_bool()
        .or_else(|| {
            input
                .as_array()
                .map(|values| values.iter().any(|value| value.as_str() == Some("image")))
        })
        .or_else(|| {
            model["architecture"]["modality"]
                .as_str()
                .map(|value| value.contains("image->") || value.contains("image+"))
        });
    CatalogCapabilities {
        tools: model["capabilities"]["function_calling"]
            .as_bool()
            .or_else(|| has_parameter(&["tools"])),
        vision,
        thinking: reasoning_present
            .then_some(true)
            .or_else(|| has_parameter(&["reasoning", "reasoning_effort", "include_reasoning"])),
    }
}
const GATEWAY_EFFORTS: [ReasoningModeId; 7] = [
    ReasoningModeId::Off,
    ReasoningModeId::Minimal,
    ReasoningModeId::Low,
    ReasoningModeId::Medium,
    ReasoningModeId::High,
    ReasoningModeId::Xhigh,
    ReasoningModeId::Max,
];
/// One authority for the gateway catalog's priority over upstream defaults.
pub(super) fn owns_catalog_metadata(provider_id: &str) -> bool {
    provider_id == "openrouter"
}

/// Batch variants use OpenRouter's asynchronous batches API, not synchronous chat.
pub(super) fn supports_synchronous_chat(model_id: &str) -> bool {
    !model_id.ends_with(":batch")
}

pub(super) fn reasoning(value: &Value) -> Option<ModelReasoningContract> {
    let object = value.as_object()?;
    let mandatory = optional_bool(object.get("mandatory"))?;
    let default_enabled = optional_bool(object.get("default_enabled"))?;
    let supports_max_tokens = optional_bool(object.get("supports_max_tokens"))?;
    let mut efforts = match object.get("supported_efforts") {
        Some(Value::Null) => Some(
            GATEWAY_EFFORTS
                .into_iter()
                .filter(|mode| mandatory != Some(true) || *mode != ReasoningModeId::Off)
                .collect(),
        ),
        Some(Value::Array(values)) => Some(parse_efforts(values, mandatory)?),
        Some(_) => return None,
        None => None,
    };
    if mandatory == Some(false) {
        if let Some(values) = efforts.as_mut().filter(|values| !values.is_empty()) {
            if !values.contains(&ReasoningModeId::Off) {
                values.insert(0, ReasoningModeId::Off);
            }
        }
    }
    let default_effort = match object.get("default_effort") {
        Some(Value::Null) | None => None,
        Some(value) => Some(parse_effort(value.as_str()?)?),
    };
    if default_effort.is_some_and(|default| {
        efforts
            .as_ref()
            .is_some_and(|values| !values.contains(&default))
            || (mandatory == Some(true) && default == ReasoningModeId::Off)
    }) {
        return None;
    }
    let control = match efforts {
        Some(efforts) if !efforts.is_empty() => ReasoningControl::Efforts(efforts),
        Some(_) | None if mandatory == Some(false) => ReasoningControl::Toggle,
        Some(_) | None => ReasoningControl::ProviderDefault,
    };
    Some(ModelReasoningContract {
        mandatory,
        default_enabled,
        supports_max_tokens,
        default_effort,
        control,
    })
}

fn optional_bool(value: Option<&Value>) -> Option<Option<bool>> {
    match value {
        None | Some(Value::Null) => Some(None),
        Some(value) => value.as_bool().map(Some),
    }
}

fn parse_efforts(values: &[Value], mandatory: Option<bool>) -> Option<Vec<ReasoningModeId>> {
    if values.len() > MAX_REASONING_EFFORTS {
        return None;
    }
    let mut efforts = Vec::with_capacity(values.len());
    for value in values {
        let effort = parse_effort(value.as_str()?)?;
        if (mandatory == Some(true) && effort == ReasoningModeId::Off) || efforts.contains(&effort)
        {
            return None;
        }
        efforts.push(effort);
    }
    Some(efforts)
}

fn parse_effort(value: &str) -> Option<ReasoningModeId> {
    ReasoningModeId::from_name(Some(if value == "none" { "off" } else { value }))
        .filter(|mode| !matches!(mode, ReasoningModeId::Auto | ReasoningModeId::Ultra))
}
