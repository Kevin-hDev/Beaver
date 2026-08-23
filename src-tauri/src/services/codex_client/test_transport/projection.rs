use serde_json::Value;

use super::RequestProjection;

const MAX_CAPTURE_ITEMS: usize = 2_048;
const MAX_TIER_BYTES: usize = 16;
const MAX_TYPE_BYTES: usize = 32;

pub(super) fn parse(body_bytes: &[u8]) -> Result<RequestProjection, String> {
    if body_bytes.len() > crate::services::secure_http::LLM_BODY_LIMIT {
        return Err(invalid());
    }
    let body: Value = serde_json::from_slice(body_bytes).map_err(|_| invalid())?;
    let model = body
        .get("model")
        .and_then(Value::as_str)
        .filter(|model| crate::services::llm::runtime_models::valid_model_id(model))
        .ok_or_else(invalid)?
        .to_string();
    let service_tier = optional_safe_string(body.get("service_tier"), MAX_TIER_BYTES)?;
    let envelope_type = optional_safe_string(body.get("type"), MAX_TYPE_BYTES)?;
    let input_count = bounded_array_len(body.get("input"))?;
    let tool_count = bounded_array_len(body.get("tools"))?;
    let forbidden_field_present = contains_forbidden_field(&body);

    Ok(RequestProjection {
        model,
        service_tier,
        envelope_type,
        input_count,
        tool_count,
        forbidden_field_present,
        body_bytes: body_bytes.len(),
    })
}

fn optional_safe_string(value: Option<&Value>, max_bytes: usize) -> Result<Option<String>, String> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = value.as_str().ok_or_else(invalid)?;
    if value.is_empty()
        || value.len() > max_bytes
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
    {
        return Err(invalid());
    }
    Ok(Some(value.to_string()))
}

fn bounded_array_len(value: Option<&Value>) -> Result<usize, String> {
    let Some(value) = value else {
        return Ok(0);
    };
    let len = value.as_array().ok_or_else(invalid)?.len();
    if len > MAX_CAPTURE_ITEMS {
        return Err(invalid());
    }
    Ok(len)
}

fn contains_forbidden_field(value: &Value) -> bool {
    match value {
        Value::Object(fields) => fields.iter().any(|(name, value)| {
            matches!(
                name.as_str(),
                "access_token" | "refresh_token" | "authorization"
            ) || contains_forbidden_field(value)
        }),
        Value::Array(values) => values.iter().any(contains_forbidden_field),
        _ => false,
    }
}

fn invalid() -> String {
    "provider_configuration_invalid".to_string()
}
