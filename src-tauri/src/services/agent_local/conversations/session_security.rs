use super::sensitive_data::{
    redact_high_confidence_text, redact_json_high_confidence_preserving_shape,
};
use super::types_session::SubagentLastActivity;

pub(crate) const MAX_CONTEXT_SNAPSHOT_TOKENS: u32 = 16 * 1024 * 1024;

pub fn sanitize_index_value(value: &mut serde_json::Value) {
    redact_json_high_confidence_preserving_shape(value);
}

pub fn bound_context_snapshot(value: &mut serde_json::Value) {
    let Some(tokens) = value.get_mut("context_tokens") else {
        return;
    };
    let bounded = tokens
        .as_u64()
        .unwrap_or(0)
        .min(MAX_CONTEXT_SNAPSHOT_TOKENS as u64);
    *tokens = serde_json::Value::from(bounded);
}

pub fn redacted_optional(value: &Option<String>) -> Option<String> {
    value.as_deref().map(redact_high_confidence_text)
}

pub fn redacted_activity(value: &Option<SubagentLastActivity>) -> Option<SubagentLastActivity> {
    value.as_ref().map(|activity| SubagentLastActivity {
        kind: redact_high_confidence_text(&activity.kind),
        label: redact_high_confidence_text(&activity.label),
        detail: redacted_optional(&activity.detail),
        updated_at: activity.updated_at,
    })
}

#[cfg(test)]
#[path = "session_security_tests.rs"]
mod tests;
