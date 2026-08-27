use std::collections::VecDeque;

use base64::Engine;
use rand::RngCore;
use serde_json::Value;

use super::session_limits::{self, CURRENT_SESSION_SCHEMA_VERSION};

const LEGACY_ID_PREFIX: &str = "legacy-local-";

pub(super) fn migrate_value(value: &mut Value) -> Result<(), String> {
    let object = value.as_object_mut().ok_or_else(invalid)?;
    object.insert(
        "schema_version".to_string(),
        Value::from(CURRENT_SESSION_SCHEMA_VERSION),
    );
    let messages = object
        .get_mut("messages")
        .and_then(Value::as_array_mut)
        .ok_or_else(invalid)?;
    merge_consecutive_users(messages)?;
    assign_missing_ids(messages, true)
}

pub(super) fn normalize_future_view(value: &mut Value) -> Result<(), String> {
    let messages = value
        .as_object_mut()
        .and_then(|object| object.get_mut("messages"))
        .and_then(Value::as_array_mut)
        .ok_or_else(invalid)?;
    assign_missing_ids(messages, false)
}

pub(super) fn validate_required_v2_fields(value: &Value) -> Result<(), String> {
    let messages = value
        .as_object()
        .and_then(|object| object.get("messages"))
        .and_then(Value::as_array)
        .ok_or_else(invalid)?;
    for message in messages {
        let message = message.as_object().ok_or_else(invalid)?;
        validate_id(
            message
                .get("turn_id")
                .and_then(Value::as_str)
                .ok_or_else(invalid)?,
        )?;
        if message.get("role").and_then(Value::as_str) == Some("tool") {
            validate_id(
                message
                    .get("tool_call_id")
                    .and_then(Value::as_str)
                    .ok_or_else(invalid)?,
            )?;
        }
        if let Some(calls) = message.get("tool_calls").and_then(Value::as_array) {
            for call in calls {
                validate_id(
                    call.as_object()
                        .and_then(|call| call.get("id"))
                        .and_then(Value::as_str)
                        .ok_or_else(invalid)?,
                )?;
            }
        }
    }
    Ok(())
}

pub(super) fn validate_id(value: &str) -> Result<(), String> {
    crate::services::reasoning_continuity::limits::validate_provider_call_id(value)
        .map_err(|_| invalid())
}

#[cfg(test)]
pub(super) fn is_legacy_local_id(value: &str) -> bool {
    value.starts_with(LEGACY_ID_PREFIX)
}

fn assign_missing_ids(messages: &mut [Value], replace: bool) -> Result<(), String> {
    let mut pending = VecDeque::<(String, String)>::new();
    let mut active_turn = None::<String>;
    for message in messages {
        let object = message.as_object_mut().ok_or_else(invalid)?;
        assign_turn_id(object, replace, &mut active_turn);
        if replace {
            object.remove("continuation");
            object.remove("replay_source");
            object.remove("tool_call_id");
        }
        if let Some(calls) = object.get_mut("tool_calls").and_then(Value::as_array_mut) {
            for call in calls {
                let call = call.as_object_mut().ok_or_else(invalid)?;
                let name = call
                    .get("function")
                    .and_then(Value::as_object)
                    .and_then(|function| function.get("name"))
                    .and_then(Value::as_str)
                    .ok_or_else(invalid)?
                    .to_string();
                let id = if replace {
                    legacy_id("call")
                } else {
                    call.get("id")
                        .and_then(Value::as_str)
                        .map(str::to_string)
                        .unwrap_or_else(|| legacy_id("call"))
                };
                call.insert("id".into(), Value::String(id.clone()));
                pending.push_back((name, id));
            }
        }
        if object.get("role").and_then(Value::as_str) == Some("tool") {
            assign_tool_result_id(object, &mut pending, replace);
        }
    }
    Ok(())
}

/// Les anciennes compressions pouvaient produire plusieurs messages user à
/// la suite. Leur texte appartenait au même contexte logique : on le regroupe
/// avant d'allouer les tours v2 afin de rendre la session poursuivable.
fn merge_consecutive_users(messages: &mut Vec<Value>) -> Result<(), String> {
    let mut repaired = Vec::<Value>::with_capacity(messages.len());
    for message in messages.drain(..) {
        let is_user = message.get("role").and_then(Value::as_str) == Some("user");
        let previous_is_user = repaired
            .last()
            .and_then(|value| value.get("role"))
            .and_then(Value::as_str)
            == Some("user");
        if is_user && previous_is_user {
            merge_user_message(repaired.last_mut().ok_or_else(invalid)?, message)?;
        } else {
            repaired.push(message);
        }
    }
    *messages = repaired;
    Ok(())
}

fn merge_user_message(target: &mut Value, source: Value) -> Result<(), String> {
    let target = target.as_object_mut().ok_or_else(invalid)?;
    let source = source.as_object().ok_or_else(invalid)?;
    let extra = source
        .get("content")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if !extra.is_empty() {
        let content = target
            .entry("content")
            .or_insert_with(|| Value::String(String::new()))
            .as_str()
            .ok_or_else(invalid)?;
        let merged = if content.is_empty() {
            extra.to_string()
        } else {
            format!("{content}\n\n{extra}")
        };
        target.insert("content".into(), Value::String(merged));
    }
    for field in ["files", "skill_names", "skill_ids"] {
        let Some(extra) = source.get(field).and_then(Value::as_array) else {
            continue;
        };
        target
            .entry(field)
            .or_insert_with(|| Value::Array(Vec::new()))
            .as_array_mut()
            .ok_or_else(invalid)?
            .extend(extra.iter().cloned());
    }
    Ok(())
}

fn assign_turn_id(
    object: &mut serde_json::Map<String, Value>,
    replace: bool,
    active_turn: &mut Option<String>,
) {
    let existing = (!replace)
        .then(|| object.get("turn_id").and_then(Value::as_str))
        .flatten()
        .map(str::to_string);
    let starts_turn =
        object.get("role").and_then(Value::as_str) == Some("user") || active_turn.is_none();
    if starts_turn {
        *active_turn = Some(existing.clone().unwrap_or_else(|| legacy_id("turn")));
    }
    if replace || existing.is_none() {
        object.insert(
            "turn_id".into(),
            Value::String(active_turn.clone().unwrap_or_else(|| legacy_id("turn"))),
        );
    }
}

fn assign_tool_result_id(
    object: &mut serde_json::Map<String, Value>,
    pending: &mut VecDeque<(String, String)>,
    replace: bool,
) {
    let name = object.get("tool_name").and_then(Value::as_str);
    let position = pending
        .iter()
        .position(|(pending_name, _)| name.is_none_or(|name| pending_name == name));
    let id = position
        .and_then(|position| pending.remove(position))
        .map_or_else(|| legacy_id("result"), |(_, id)| id);
    if replace || !object.contains_key("tool_call_id") {
        object.insert("tool_call_id".into(), Value::String(id));
    }
}

fn legacy_id(kind: &str) -> String {
    let mut random = [0_u8; 24];
    rand::rngs::OsRng.fill_bytes(&mut random);
    format!(
        "{LEGACY_ID_PREFIX}{kind}-{}",
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(random)
    )
}

fn invalid() -> String {
    session_limits::invalid_session()
}
