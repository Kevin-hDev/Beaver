use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use super::route::LlmRoute;

const CACHE_KEY_PREFIX: &str = "bv1_";
const CACHE_KEY_BYTES: usize = 16;
const MAX_SESSION_ID_BYTES: usize = 128;

pub(super) fn apply_payload(
    payload: &mut Value,
    route: &LlmRoute,
    model: &str,
    session_id: Option<&str>,
) {
    let Some(key) = cache_key(route, model, session_id) else {
        return;
    };
    match route.chat_provider_id {
        "openai" if super::providers::openai::is_gpt_56(model) => {
            apply_gpt_56(payload, key);
        }
        "openrouter" => {
            payload["session_id"] = key.clone().into();
            if openrouter_gpt_56(model) {
                apply_gpt_56(payload, key);
            }
        }
        "mistral" | "moonshot" | "moonshot-oauth" => {
            payload["prompt_cache_key"] = key.into();
        }
        _ => {}
    }
}

pub(super) fn request_headers(
    route: &LlmRoute,
    model: &str,
    session_id: Option<&str>,
) -> Result<HeaderMap, String> {
    let mut headers = HeaderMap::new();
    if route.chat_provider_id == "google" {
        headers.insert(
            "x-goog-api-client",
            HeaderValue::from_static(concat!("beaver-desktop/", env!("CARGO_PKG_VERSION"))),
        );
    }
    if route.chat_provider_id == "openrouter" && model != "metadata" {
        headers.insert("x-openrouter-metadata", HeaderValue::from_static("enabled"));
    }
    if route.chat_provider_id != "xai" {
        return Ok(headers);
    }
    if let Some(key) = cache_key(route, model, session_id) {
        let value = HeaderValue::from_str(&key).map_err(|_| "provider_configuration_invalid")?;
        headers.insert("x-grok-conv-id", value);
    }
    Ok(headers)
}

pub(super) fn include_usage(route: &LlmRoute) -> bool {
    matches!(
        route.chat_provider_id,
        "groq"
            | "google"
            | "cerebras"
            | "openai"
            | "deepseek"
            | "xai"
            | "xai-oauth"
            | "moonshot"
            | "moonshot-oauth"
    )
}

pub(crate) fn routing_key(
    connection_id: &str,
    model: &str,
    session_id: Option<&str>,
) -> Option<String> {
    let route_id = if connection_id == crate::services::codex_client::PROVIDER_ID {
        connection_id
    } else {
        super::route::resolve(connection_id)?.chat_provider_id
    };
    derive_cache_key(route_id, model, session_id)
}

fn apply_gpt_56(payload: &mut Value, key: String) {
    payload["prompt_cache_key"] = key.into();
    if mark_stable_prefix(payload) {
        payload["prompt_cache_options"] = json!({
            "mode": "explicit",
            "ttl": "30m",
        });
    }
}

fn mark_stable_prefix(payload: &mut Value) -> bool {
    let Some(messages) = payload.get_mut("messages").and_then(Value::as_array_mut) else {
        return false;
    };
    let stable_end = messages
        .iter()
        .position(|message| !matches!(message["role"].as_str(), Some("system" | "developer")))
        .unwrap_or(messages.len());
    let Some(message) = messages[..stable_end]
        .iter_mut()
        .rev()
        .find(|message| message["content"].is_string())
    else {
        return false;
    };
    let Some(text) = message["content"].as_str().map(str::to_string) else {
        return false;
    };
    message["content"] = json!([{
        "type": "text",
        "text": text,
        "prompt_cache_breakpoint": { "mode": "explicit" },
    }]);
    true
}

fn openrouter_gpt_56(model: &str) -> bool {
    model
        .strip_prefix("openai/")
        .is_some_and(super::providers::openai::is_gpt_56)
}

fn cache_key(route: &LlmRoute, model: &str, session_id: Option<&str>) -> Option<String> {
    derive_cache_key(route.chat_provider_id, model, session_id)
}

fn derive_cache_key(route_id: &str, model: &str, session_id: Option<&str>) -> Option<String> {
    let session_id = session_id.filter(|value| valid_session_id(value))?;
    let mut hash = Sha256::new();
    hash.update(b"beaver-provider-cache-v1\0");
    hash.update(route_id.as_bytes());
    hash.update(b"\0");
    hash.update(model.as_bytes());
    hash.update(b"\0");
    hash.update(session_id.as_bytes());
    let digest = hash.finalize();
    Some(format!(
        "{CACHE_KEY_PREFIX}{}",
        hex::encode(&digest[..CACHE_KEY_BYTES])
    ))
}

fn valid_session_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_SESSION_ID_BYTES
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
}
