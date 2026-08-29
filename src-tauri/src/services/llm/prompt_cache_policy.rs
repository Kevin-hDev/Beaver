use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use super::request_purpose::RequestPurpose;
use super::route_profile::{CachePolicy, ResolvedCachePolicy};

const CACHE_KEY_PREFIX: &str = "bv1_";
const CACHE_KEY_BYTES: usize = 16;
const MAX_SESSION_ID_BYTES: usize = 128;
// The local tokenizer is approximate, so keep a 25% margin above OpenAI's
// 1,024-token minimum before disabling the safer automatic cache mode.
const MIN_EXPLICIT_PREFIX_ESTIMATED_TOKENS: usize = 1_280;

pub(super) fn apply_payload(
    payload: &mut Value,
    policy: ResolvedCachePolicy,
    session_id: Option<&str>,
) {
    let Some(key) = cache_key(policy, session_id) else {
        return;
    };
    match policy.kind {
        CachePolicy::OpenAi56 => apply_gpt_56(payload, key),
        CachePolicy::OpenRouter => payload["session_id"] = key.into(),
        CachePolicy::PromptKey => payload["prompt_cache_key"] = key.into(),
        CachePolicy::None
        | CachePolicy::AnthropicAutomatic
        | CachePolicy::Google
        | CachePolicy::XaiHeader => {}
    }
}

pub(super) fn request_headers(
    policy: ResolvedCachePolicy,
    session_id: Option<&str>,
    purpose: RequestPurpose,
) -> Result<HeaderMap, String> {
    let mut headers = HeaderMap::new();
    if policy.kind == CachePolicy::Google {
        headers.insert(
            "x-goog-api-client",
            HeaderValue::from_static(concat!("beaver-desktop/", env!("CARGO_PKG_VERSION"))),
        );
    }
    if policy.kind == CachePolicy::OpenRouter && purpose != RequestPurpose::AccountMetadata {
        headers.insert("x-openrouter-metadata", HeaderValue::from_static("enabled"));
    }
    if policy.kind != CachePolicy::XaiHeader {
        return Ok(headers);
    }
    if let Some(key) = cache_key(policy, session_id) {
        let value = HeaderValue::from_str(&key).map_err(|_| "provider_configuration_invalid")?;
        headers.insert("x-grok-conv-id", value);
    }
    Ok(headers)
}

pub(super) const fn include_usage(policy: ResolvedCachePolicy) -> bool {
    policy.include_usage
}

pub(crate) fn routing_key(
    connection_id: &str,
    model: &str,
    session_id: Option<&str>,
) -> Option<String> {
    let route_id = super::route_profile::find(connection_id)?.id.provider_id();
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
    if crate::services::token_counting::estimate_text_tokens(text)
        < MIN_EXPLICIT_PREFIX_ESTIMATED_TOKENS
    {
        return false;
    }
    let blocks = vec![json!({
        "type": "text",
        "text": text,
        "prompt_cache_breakpoint": { "mode": "explicit" },
    })];
    message["content"] = Value::Array(blocks);
    true
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

fn cache_key(policy: ResolvedCachePolicy<'_>, session_id: Option<&str>) -> Option<String> {
    derive_cache_key(policy.route_id, policy.model, session_id)
}

fn valid_session_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_SESSION_ID_BYTES
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
}
