use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use super::request_purpose::RequestPurpose;
use super::route::LlmRoute;

const CACHE_KEY_PREFIX: &str = "bv1_";
const CACHE_KEY_BYTES: usize = 16;
const MAX_SESSION_ID_BYTES: usize = 128;
// The local tokenizer is approximate, so keep a 25% margin above OpenAI's
// 1,024-token minimum before disabling the safer automatic cache mode.
const MIN_EXPLICIT_PREFIX_ESTIMATED_TOKENS: usize = 1_280;

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
            payload["session_id"] = key.into();
        }
        "mistral" | "moonshot" | "moonshot-oauth" => {
            payload["prompt_cache_key"] = key.into();
        }
        _ => {}
    }
}

pub(super) fn request_headers(
    route: &LlmRoute,
    model: Option<&str>,
    session_id: Option<&str>,
    purpose: RequestPurpose,
) -> Result<HeaderMap, String> {
    let mut headers = HeaderMap::new();
    if route.chat_provider_id == "google" {
        headers.insert(
            "x-goog-api-client",
            HeaderValue::from_static(concat!("beaver-desktop/", env!("CARGO_PKG_VERSION"))),
        );
    }
    if route.chat_provider_id == "openrouter"
        && model.is_some()
        && purpose != RequestPurpose::AccountMetadata
    {
        headers.insert("x-openrouter-metadata", HeaderValue::from_static("enabled"));
    }
    if route.chat_provider_id != "xai" {
        return Ok(headers);
    }
    if let Some(key) = model.and_then(|model| cache_key(route, model, session_id)) {
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
    let Some(text) = message["content"].as_str() else {
        return false;
    };
    let (stable, dynamic) = split_stable_system_content(text);
    if crate::services::token_counting::estimate_text_tokens(stable)
        < MIN_EXPLICIT_PREFIX_ESTIMATED_TOKENS
    {
        return false;
    }
    let mut blocks = vec![json!({
        "type": "text",
        "text": stable,
        "prompt_cache_breakpoint": { "mode": "explicit" },
    })];
    if let Some(dynamic) = dynamic {
        blocks.push(json!({
            "type": "text",
            "text": dynamic,
        }));
    }
    message["content"] = Value::Array(blocks);
    true
}

fn split_stable_system_content(content: &str) -> (&str, Option<&str>) {
    content
        .find(crate::services::agent_local::web_search_status::SECTION_START)
        .map(|index| (&content[..index], Some(&content[index..])))
        .unwrap_or((content, None))
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
