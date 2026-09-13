use serde_json::Value;

use crate::services::provider_usage::RequestUsage;

pub(super) fn index(value: &Value) -> Result<usize, String> {
    value
        .get("index")
        .and_then(Value::as_u64)
        .and_then(|value| value.try_into().ok())
        .ok_or_else(invalid)
}

pub(super) fn append_field(value: &mut Value, field: &str, suffix: &str) -> Result<(), String> {
    match value.get_mut(field) {
        Some(Value::String(target)) => {
            target.push_str(suffix);
            Ok(())
        }
        _ => Err(invalid()),
    }
}

pub(super) fn bounded_string(value: &Value, field: &str) -> Result<String, String> {
    let text = value
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| crate::services::provider_usage::valid_provider_metadata(value))
        .ok_or_else(invalid)?;
    Ok(text.to_string())
}

pub(super) fn validate_usage_counts(value: &Value) -> Result<(), String> {
    for path in [
        "/input_tokens",
        "/output_tokens",
        "/cache_read_input_tokens",
        "/cache_creation_input_tokens",
        "/cache_creation/ephemeral_5m_input_tokens",
        "/cache_creation/ephemeral_1h_input_tokens",
    ] {
        if let Some(raw) = value.pointer(path) {
            let count = raw.as_u64().ok_or_else(invalid)?;
            if count > crate::services::provider_usage::MAX_REQUEST_TOKENS {
                return Err(invalid());
            }
        }
    }
    Ok(())
}

pub(super) fn merge_usage(current: &mut RequestUsage, update: RequestUsage) {
    if update.input_tokens.is_some() {
        current.input_tokens = update.input_tokens;
    }
    if update.output_tokens.is_some() {
        current.output_tokens = update.output_tokens;
    }
    if update.cached_input_tokens.is_some() {
        current.cached_input_tokens = update.cached_input_tokens;
    }
    if update.cache_write_input_tokens.is_some() {
        current.cache_write_input_tokens = update.cache_write_input_tokens;
    }
    if update.cache_status != Default::default() {
        current.cache_status = update.cache_status;
    }
    current.total_tokens = current
        .input_tokens
        .zip(current.output_tokens)
        .map(|(input, output)| input.saturating_add(output));
}

pub(super) fn serialized_len(value: &Value) -> Result<usize, String> {
    serde_json::to_vec(value)
        .map(|bytes| bytes.len())
        .map_err(|_| invalid())
}

pub(super) fn invalid() -> String {
    "provider_stream_invalid".into()
}
