use serde_json::{Map, Value};

pub(super) fn contribution(owner: &str, value: &mut Value) -> Result<(), String> {
    let object = value.as_object_mut().ok_or_else(invalid)?;
    normalize_id(owner, object, "id", false)?;
    if object.contains_key("actionId") {
        normalize_id(owner, object, "actionId", false)?;
    }
    if object.contains_key("targetId") {
        normalize_id(owner, object, "targetId", true)?;
    }
    for key in ["list", "detail"] {
        if let Some(view) = object.get_mut(key) {
            let mut nodes = 0usize;
            normalize_view(owner, view, 1, &mut nodes)?;
        }
    }
    Ok(())
}

fn normalize_view(
    owner: &str,
    value: &mut Value,
    depth: usize,
    nodes: &mut usize,
) -> Result<(), String> {
    *nodes = nodes.checked_add(1).ok_or_else(limit)?;
    if *nodes > super::ui_contract::MAX_VIEW_NODES || depth > super::ui_contract::MAX_VIEW_DEPTH {
        return Err(limit());
    }
    let object = value.as_object_mut().ok_or_else(invalid)?;
    let kind = object
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(invalid)?
        .to_string();
    if ["textField", "numberField", "select", "toggle", "button"].contains(&kind.as_str()) {
        normalize_id(owner, object, "id", false)?;
    }
    if kind == "button" {
        normalize_id(owner, object, "actionId", false)?;
    }
    if let Some(children) = object.get_mut("children").and_then(Value::as_array_mut) {
        for child in children {
            normalize_view(owner, child, depth + 1, nodes)?;
        }
    }
    Ok(())
}

fn normalize_id(
    owner: &str,
    object: &mut Map<String, Value>,
    key: &str,
    allow_beaver: bool,
) -> Result<(), String> {
    let value = object
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(invalid)?;
    super::validation::identifier(value)?;
    if value.starts_with(&format!("{owner}.")) || (allow_beaver && value.starts_with("beaver.")) {
        return Ok(());
    }
    object.insert(key.to_string(), Value::String(format!("{owner}.{value}")));
    Ok(())
}

fn invalid() -> String {
    "ui_contribution_invalid".to_string()
}

fn limit() -> String {
    "ui_limit_exceeded".to_string()
}
