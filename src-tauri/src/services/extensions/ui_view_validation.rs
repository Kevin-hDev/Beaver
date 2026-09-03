use serde_json::{Map, Value};
use std::collections::HashSet;

pub(super) fn validate_view(
    owner: &str,
    root: &Value,
    actions: &mut HashSet<String>,
) -> Result<(), String> {
    let mut nodes = 0usize;
    let mut fields = 0usize;
    visit(owner, root, 1, &mut nodes, &mut fields, actions)
}

fn visit(
    owner: &str,
    value: &Value,
    depth: usize,
    nodes: &mut usize,
    fields: &mut usize,
    actions: &mut HashSet<String>,
) -> Result<(), String> {
    *nodes = nodes.checked_add(1).ok_or_else(invalid)?;
    if *nodes > super::ui_contract::MAX_VIEW_NODES || depth > super::ui_contract::MAX_VIEW_DEPTH {
        return Err(limit());
    }
    let object = value.as_object().ok_or_else(invalid)?;
    let kind = string(object, "type")?;
    if !super::ui_contract::UI_PRIMITIVES.contains(&kind) {
        return Err(invalid());
    }
    match kind {
        "stack" | "row" => {
            exact(object, &["type", "children"])?;
            let children = object
                .get("children")
                .and_then(Value::as_array)
                .ok_or_else(invalid)?;
            for child in children {
                visit(owner, child, depth + 1, nodes, fields, actions)?;
            }
        }
        "heading" | "text" | "badge" => {
            exact(object, &["type", "text"])?;
            localized(object.get("text").ok_or_else(invalid)?)?;
        }
        "separator" => exact(object, &["type"])?,
        "button" => {
            exact(object, &["type", "id", "label", "actionId"])?;
            field(owner, object, fields)?;
            let action = string(object, "actionId")?;
            owned_id(owner, action)?;
            actions.insert(action.to_string());
        }
        "textField" | "numberField" | "toggle" => {
            exact(object, &["type", "id", "label", "value"])?;
            field(owner, object, fields)?;
            field_value(object.get("value").ok_or_else(invalid)?)?;
        }
        "select" => {
            exact(object, &["type", "id", "label", "value", "options"])?;
            field(owner, object, fields)?;
            field_value(object.get("value").ok_or_else(invalid)?)?;
            let options = object
                .get("options")
                .and_then(Value::as_array)
                .ok_or_else(invalid)?;
            if options.len() > super::ui_contract::MAX_OPTIONS_PER_FIELD {
                return Err(limit());
            }
            for option in options {
                validate_option(option)?;
            }
        }
        _ => return Err(invalid()),
    }
    Ok(())
}

fn field(owner: &str, object: &Map<String, Value>, count: &mut usize) -> Result<(), String> {
    *count = count.checked_add(1).ok_or_else(invalid)?;
    if *count > super::ui_contract::MAX_FIELDS_PER_VIEW {
        return Err(limit());
    }
    owned_id(owner, string(object, "id")?)?;
    localized(object.get("label").ok_or_else(invalid)?)
}

fn validate_option(value: &Value) -> Result<(), String> {
    let object = value.as_object().ok_or_else(invalid)?;
    exact(object, &["value", "label"])?;
    bounded_text(string(object, "value")?)?;
    localized(object.get("label").ok_or_else(invalid)?)
}

pub(super) fn localized(value: &Value) -> Result<(), String> {
    let object = value.as_object().ok_or_else(invalid)?;
    if object.is_empty()
        || !object.contains_key("default")
        || object
            .keys()
            .any(|key| key != "default" && !super::ui_contract::UI_LOCALES.contains(&key.as_str()))
    {
        return Err(invalid());
    }
    for value in object.values() {
        text(value.as_str().ok_or_else(invalid)?)?;
    }
    Ok(())
}

pub(super) fn owned_id(owner: &str, value: &str) -> Result<(), String> {
    super::validation::identifier(value)?;
    value
        .strip_prefix(owner)
        .is_some_and(|tail| tail.starts_with('.'))
        .then_some(())
        .ok_or_else(invalid)
}

pub(super) fn exact(object: &Map<String, Value>, allowed: &[&str]) -> Result<(), String> {
    (object.len() <= allowed.len() && object.keys().all(|key| allowed.contains(&key.as_str())))
        .then_some(())
        .ok_or_else(invalid)
}

pub(super) fn string<'a>(object: &'a Map<String, Value>, key: &str) -> Result<&'a str, String> {
    object.get(key).and_then(Value::as_str).ok_or_else(invalid)
}

fn field_value(value: &Value) -> Result<(), String> {
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) => Ok(()),
        Value::String(value) => bounded_text(value),
        _ => Err(invalid()),
    }
}

fn text(value: &str) -> Result<(), String> {
    (!value.is_empty()).then_some(()).ok_or_else(invalid)?;
    bounded_text(value)
}

fn bounded_text(value: &str) -> Result<(), String> {
    (value.chars().count() <= super::ui_contract::MAX_TEXT_CHARS)
        .then_some(())
        .ok_or_else(invalid)
}

fn invalid() -> String {
    "ui_contribution_invalid".to_string()
}
fn limit() -> String {
    "ui_limit_exceeded".to_string()
}
