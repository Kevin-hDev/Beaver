//! Safe projection, not a copy of OpenRouter's extensible metadata document.
use serde::Serialize;
use serde_json::Value;

const MAX_ITEMS: usize = 16;
const MAX_LABEL_BYTES: usize = 128;

#[derive(Serialize)]
pub(in super::super) struct RoutingMetadata {
    attempt: Option<u32>,
    endpoint_count: Option<u32>,
    endpoints: Vec<Endpoint>,
    attempts: Vec<Attempt>,
    pipeline: Vec<Stage>,
    truncated: bool,
}

#[derive(Serialize)]
struct Endpoint {
    provider: Option<String>,
    model: Option<String>,
    selected: Option<bool>,
}

#[derive(Serialize)]
struct Attempt {
    provider: Option<String>,
    model: Option<String>,
    status: Option<u16>,
}

#[derive(Serialize)]
struct Stage {
    kind: Option<&'static str>,
    name: Option<String>,
    action: Option<&'static str>,
    flagged: Option<bool>,
}

pub(in super::super) fn project(value: &Value) -> Option<RoutingMetadata> {
    let metadata = value.get("openrouter_metadata")?.as_object()?;
    let endpoints = metadata.get("endpoints");
    let available = endpoints.and_then(|value| value.get("available"));
    let attempts = metadata.get("attempts");
    let pipeline = metadata.get("pipeline");
    Some(RoutingMetadata {
        attempt: number(metadata.get("attempt")),
        endpoint_count: number(endpoints.and_then(|value| value.get("total"))),
        endpoints: items(available)
            .map(|value| Endpoint {
                provider: label(value.get("provider")),
                model: label(value.get("model")),
                selected: value.get("selected").and_then(Value::as_bool),
            })
            .collect(),
        attempts: items(attempts)
            .map(|value| Attempt {
                provider: label(value.get("provider")),
                model: label(value.get("model")),
                status: value
                    .get("status")
                    .and_then(Value::as_u64)
                    .filter(|status| (100..=599).contains(status))
                    .map(|status| status as u16),
            })
            .collect(),
        pipeline: items(pipeline)
            .map(|value| Stage {
                kind: match value["type"].as_str() {
                    Some("guardrail") => Some("guardrail"),
                    Some("plugin") => Some("plugin"),
                    Some("server_tools") => Some("server_tools"),
                    Some("response_healing") => Some("response_healing"),
                    Some("context_compression") => Some("context_compression"),
                    _ => None,
                },
                name: label(value.get("name")),
                action: match value.pointer("/data/action").and_then(Value::as_str) {
                    Some("blocked") => Some("blocked"),
                    Some("allowed") => Some("allowed"),
                    _ => None,
                },
                flagged: value.pointer("/data/flagged").and_then(Value::as_bool),
            })
            .collect(),
        truncated: [available, attempts, pipeline].into_iter().any(|value| {
            value
                .and_then(Value::as_array)
                .is_some_and(|items| items.len() > MAX_ITEMS)
        }),
    })
}

fn number(value: Option<&Value>) -> Option<u32> {
    value?.as_u64()?.try_into().ok()
}

fn items(value: Option<&Value>) -> impl Iterator<Item = &Value> {
    value
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .take(MAX_ITEMS)
        .filter(|value| value.is_object())
}

fn label(value: Option<&Value>) -> Option<String> {
    safe_label(value?.as_str()?)
}

pub(super) fn safe_label(value: &str) -> Option<String> {
    if value.is_empty()
        || value.len() > MAX_LABEL_BYTES
        || value.contains("..")
        || !value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(
                    byte,
                    b' ' | b'-' | b'_' | b'.' | b'/' | b':' | b'(' | b')' | b'+' | b'~'
                )
        })
    {
        return None;
    }
    // Even syntactically valid labels may contain credentials. Never persist a redacted fragment.
    (crate::services::llm::sanitize_log_body(value) == value).then(|| value.to_owned())
}

pub(super) fn generation_id(value: &str) -> Option<String> {
    (value.starts_with("gen-")
        && value.len() > "gen-".len()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_')))
    .then(|| safe_label(value))
    .flatten()
}
