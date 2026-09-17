use serde_json::Value;

#[path = "tool_validate_definition.rs"]
mod definition;
pub(crate) use definition::validate as validate_definition;

pub fn validate(tool: &str, args: &Value) -> Result<Value, String> {
    let Some(definition) = super::tool_definitions::native_tool_definitions()
        .into_iter()
        .find(|definition| {
            definition.pointer("/function/name").and_then(Value::as_str) == Some(tool)
        })
    else {
        return Ok(args.clone());
    };
    let normalized = without_legacy_shell_alias(tool, args);
    validate_definition(tool, normalized.as_ref().unwrap_or(args), &definition)?;
    let obj = args
        .as_object()
        .ok_or_else(|| "les arguments doivent être un objet JSON".to_string())?;
    validate_subagent_change_ids(tool, obj)?;
    validate_shell_numbers(tool, obj)?;
    validate_shell_text(tool, obj)?;
    validate_shell_control(tool, obj)?;
    if tool == "todo_delete" {
        let has_id = matches!(obj.get("id"), Some(value) if !value.is_null());
        let active = obj.get("active").and_then(Value::as_bool).unwrap_or(false);
        match (has_id, active) {
            (true, true) => return Err("utiliser soit 'id', soit active=true".to_string()),
            (false, false) => return Err("paramètre 'id' ou active=true requis".to_string()),
            _ => {}
        }
    }

    Ok(args.clone())
}

fn without_legacy_shell_alias(tool: &str, args: &Value) -> Option<Value> {
    if !matches!(tool, "bash" | "bash_control") || args.get("yield-time-ms").is_none() {
        return None;
    }
    let mut normalized = args.clone();
    let object = normalized.as_object_mut()?;
    object.remove("yield-time-ms")?;
    Some(normalized)
}

fn validate_subagent_change_ids(
    tool: &str,
    args: &serde_json::Map<String, Value>,
) -> Result<(), String> {
    if !matches!(
        tool,
        "inspect_subagent_changes" | "apply_subagent_changes" | "discard_subagent_changes"
    ) {
        return Ok(());
    }
    for (name, source) in [
        ("subagent_id", "subagent_id/child_session_id"),
        ("change_id", "change_id/id"),
    ] {
        let value = args
            .get(name)
            .and_then(Value::as_str)
            .ok_or_else(|| format!("paramètre '{name}' requis"))?;
        super::types_subagent_change::validate_uuid(value)
            .map_err(|_| format!("'{name}' doit être le UUID v4 '{source}' des métadonnées"))?;
    }
    Ok(())
}

fn validate_shell_control(tool: &str, args: &serde_json::Map<String, Value>) -> Result<(), String> {
    if tool != "bash_control" {
        return Ok(());
    }
    let session_id = args
        .get("session_id")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if uuid::Uuid::parse_str(session_id).is_err() {
        return Err("'session_id' invalide".to_string());
    }
    if args.get("stop").and_then(Value::as_bool) != Some(true) {
        return Ok(());
    }
    let has_input = args
        .get("chars")
        .and_then(Value::as_str)
        .is_some_and(|value| !value.is_empty());
    let has_eof = args.get("eof").and_then(Value::as_bool) == Some(true);
    if has_input || has_eof {
        return Err("'stop' ne peut pas être combiné avec 'chars' ou 'eof'".to_string());
    }
    Ok(())
}

fn validate_shell_text(tool: &str, args: &serde_json::Map<String, Value>) -> Result<(), String> {
    match tool {
        "bash" => super::tool_bash::validate_command(
            args.get("command")
                .and_then(Value::as_str)
                .unwrap_or_default(),
        ),
        "bash_control" => args
            .get("chars")
            .and_then(Value::as_str)
            .map(super::tool_bash::validate_input)
            .transpose()
            .map(|_| ()),
        _ => Ok(()),
    }
}

fn validate_shell_numbers(tool: &str, args: &serde_json::Map<String, Value>) -> Result<(), String> {
    if tool == "bash" {
        if let Some(timeout) = args.get("timeout").filter(|value| !value.is_null()) {
            if timeout.as_u64().is_none_or(|seconds| seconds == 0) {
                return Err("'timeout' doit être un entier positif".to_string());
            }
        }
    }
    if matches!(tool, "bash" | "bash_control") {
        for name in ["yield_time_ms", "yield-time-ms"] {
            if args
                .get(name)
                .filter(|value| !value.is_null())
                .is_some_and(|value| value.as_u64().is_none())
            {
                return Err(format!("'{name}' doit être un entier positif ou nul"));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "tool_validate_bash_tests.rs"]
mod bash_tests;
#[cfg(test)]
#[path = "tool_validate_forecast_tests.rs"]
mod forecast_tests;
#[cfg(test)]
#[path = "tool_validate_tests.rs"]
mod tests;
