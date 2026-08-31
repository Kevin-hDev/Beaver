use serde_json::Value;

use super::session_limits::CURRENT_SESSION_SCHEMA_VERSION;

const SUMMARY_PREFIX: &str = "This session is being continued from a previous conversation";
const CONTEXT_PREFIX: &str = "Recent file context preserved across compression:";

pub(super) fn migrate_v2_markers(value: &mut Value) -> Result<(), String> {
    let object = value
        .as_object_mut()
        .ok_or_else(super::session_limits::invalid_session)?;
    if object.get("schema_version").and_then(Value::as_u64) != Some(2) {
        return Err(super::session_limits::invalid_session());
    }
    classify_legacy_markers(object)?;
    object.insert(
        "schema_version".into(),
        Value::from(CURRENT_SESSION_SCHEMA_VERSION),
    );
    Ok(())
}

pub(super) fn classify_markers_after_v1_migration(value: &mut Value) -> Result<(), String> {
    let object = value
        .as_object_mut()
        .ok_or_else(super::session_limits::invalid_session)?;
    classify_legacy_markers(object)
}

fn classify_legacy_markers(
    object: &mut serde_json::Map<String, Value>,
) -> Result<(), String> {
    let messages = object
        .get_mut("messages")
        .and_then(Value::as_array_mut)
        .ok_or_else(super::session_limits::invalid_session)?;
    for message in messages {
        let message = message
            .as_object_mut()
            .ok_or_else(super::session_limits::invalid_session)?;
        let role = message.get("role").and_then(Value::as_str);
        let content = message
            .get("content")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim_start();
        let kind = if role == Some("user")
            && (content.starts_with(SUMMARY_PREFIX) || content.starts_with(CONTEXT_PREFIX))
        {
            Some("compression_checkpoint")
        } else if role == Some("assistant")
            && content == crate::services::compress::checkpoint_boundary::CONTENT
        {
            Some("compression_boundary")
        } else {
            None
        };
        if let Some(kind) = kind {
            message.insert("message_kind".into(), Value::String(kind.into()));
        }
    }
    Ok(())
}
