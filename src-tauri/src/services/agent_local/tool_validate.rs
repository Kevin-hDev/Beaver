use serde_json::Value;

#[path = "tool_validate_definition.rs"]
mod definition;
#[path = "tool_validate_schema.rs"]
mod schema;
use schema::{schema, Ty};

pub(crate) use definition::validate as validate_definition;

fn type_ok(val: &Value, ty: Ty) -> bool {
    match ty {
        Ty::Str => val.is_string(),
        Ty::Int => val.is_u64(),
        Ty::Float => val.is_f64() || val.is_u64() || val.is_i64(),
        Ty::Arr => val.is_array(),
        Ty::Obj => val.is_object(),
        Ty::Bool => val.is_boolean(),
    }
}

fn ty_label(ty: Ty) -> &'static str {
    match ty {
        Ty::Str => "string",
        Ty::Int => "entier positif ou nul",
        Ty::Float => "number",
        Ty::Arr => "array",
        Ty::Obj => "object",
        Ty::Bool => "boolean",
    }
}

pub fn validate(tool: &str, args: &Value) -> Result<Value, String> {
    if let Some(definition) = super::tool_definitions_forecast::definition_for_tool(tool) {
        return validate_definition(tool, args, &definition);
    }
    let specs = match schema(tool) {
        Some(s) => s,
        None => return Ok(args.clone()),
    };

    let obj = match args.as_object() {
        Some(o) => o,
        None => return Err("les arguments doivent être un objet JSON".into()),
    };

    for &(name, ty, required) in specs {
        match obj.get(name) {
            None | Some(Value::Null) if required => {
                return Err(format!("paramètre '{name}' requis"));
            }
            Some(v) if !v.is_null() && !type_ok(v, ty) => {
                return Err(format!("'{name}' doit être de type {}", ty_label(ty)));
            }
            _ => {}
        }
    }
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

    if let Some(key) = obj
        .keys()
        .find(|key| !specs.iter().any(|(name, _, _)| *name == key.as_str()))
    {
        let accepted = specs
            .iter()
            .map(|(name, _, _)| *name)
            .collect::<Vec<_>>()
            .join(", ");
        return Err(format!(
            "paramètre '{key}' inconnu; paramètres acceptés: {accepted}"
        ));
    }
    Ok(args.clone())
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

fn validate_shell_control(
    tool: &str,
    args: &serde_json::Map<String, Value>,
) -> Result<(), String> {
    if tool != "bash_write" {
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

fn validate_shell_text(
    tool: &str,
    args: &serde_json::Map<String, Value>,
) -> Result<(), String> {
    match tool {
        "bash" => super::tool_bash::validate_command(
            args.get("command")
                .and_then(Value::as_str)
                .unwrap_or_default(),
        ),
        "bash_write" => args
            .get("chars")
            .and_then(Value::as_str)
            .map(super::tool_bash::validate_input)
            .transpose()
            .map(|_| ()),
        _ => Ok(()),
    }
}

fn validate_shell_numbers(
    tool: &str,
    args: &serde_json::Map<String, Value>,
) -> Result<(), String> {
    if tool == "bash" {
        if let Some(timeout) = args.get("timeout").filter(|value| !value.is_null()) {
            if timeout.as_u64().is_none_or(|seconds| seconds == 0) {
                return Err("'timeout' doit être un entier positif".to_string());
            }
        }
    }
    if matches!(tool, "bash" | "bash_write") {
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
#[path = "tool_validate_tests.rs"]
mod tests;
#[cfg(test)]
#[path = "tool_validate_forecast_tests.rs"]
mod forecast_tests;
