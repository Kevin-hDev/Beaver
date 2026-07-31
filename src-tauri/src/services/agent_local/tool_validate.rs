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
        Ty::Int => val.is_u64() || val.is_i64(),
        Ty::Float => val.is_f64() || val.is_u64() || val.is_i64(),
        Ty::Arr => val.is_array(),
        Ty::Obj => val.is_object(),
        Ty::Bool => val.is_boolean(),
    }
}

fn ty_label(ty: Ty) -> &'static str {
    match ty {
        Ty::Str => "string",
        Ty::Int => "integer",
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
    validate_shell_numbers(tool, obj)?;
    if tool == "todo_delete" {
        let has_id = matches!(obj.get("id"), Some(value) if !value.is_null());
        let active = obj.get("active").and_then(Value::as_bool).unwrap_or(false);
        match (has_id, active) {
            (true, true) => return Err("utiliser soit 'id', soit active=true".to_string()),
            (false, false) => return Err("paramètre 'id' ou active=true requis".to_string()),
            _ => {}
        }
    }

    let mut cleaned = serde_json::Map::with_capacity(specs.len());
    for (key, val) in obj {
        if specs.iter().any(|(n, _, _)| *n == key.as_str()) {
            cleaned.insert(key.clone(), val.clone());
        } else {
            eprintln!("[tool-validate] argument inconnu ignoré : {tool}.{key}");
        }
    }
    Ok(Value::Object(cleaned))
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
        for name in ["yield-time_ms", "yield-time-ms"] {
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
