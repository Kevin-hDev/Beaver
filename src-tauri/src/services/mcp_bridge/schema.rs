use serde_json::{Map, Value};

const INVALID: &str = "schéma MCP invalide";

pub fn validate(schema: &Value, value: &Value) -> Result<(), String> {
    super::schema_definition::validate(schema)?;
    if !root_accepts_object(schema) {
        return Err(INVALID.to_string());
    }
    validate_value(schema, value)
}

pub fn validate_definition(schema: &Value) -> Result<(), String> {
    super::schema_definition::validate(schema)
}

pub(crate) fn validate_value(schema: &Value, value: &Value) -> Result<(), String> {
    let object = schema.as_object().ok_or_else(|| INVALID.to_string())?;
    validate_enum_and_const(object, value)?;
    validate_combinations(object, value)?;
    if let Some(kind) = object.get("type") {
        validate_type(kind, value)?;
    }
    super::schema_types::validate(object, value)
}

fn validate_enum_and_const(schema: &Map<String, Value>, value: &Value) -> Result<(), String> {
    if let Some(allowed) = schema.get("enum").and_then(Value::as_array) {
        if !allowed.iter().any(|candidate| candidate == value) {
            return Err(INVALID.to_string());
        }
    }
    if schema
        .get("const")
        .is_some_and(|expected| expected != value)
    {
        return Err(INVALID.to_string());
    }
    Ok(())
}

fn validate_combinations(schema: &Map<String, Value>, value: &Value) -> Result<(), String> {
    for keyword in ["allOf", "anyOf", "oneOf"] {
        let Some(branches) = schema.get(keyword).and_then(Value::as_array) else {
            continue;
        };
        let matches = branches
            .iter()
            .filter(|branch| validate_value(branch, value).is_ok())
            .count();
        let valid = match keyword {
            "allOf" => matches == branches.len(),
            "anyOf" => matches > 0,
            _ => matches == 1,
        };
        if !valid {
            return Err(INVALID.to_string());
        }
    }
    if let Some(not_schema) = schema.get("not") {
        if validate_value(not_schema, value).is_ok() {
            return Err(INVALID.to_string());
        }
    }
    Ok(())
}

fn validate_type(kind: &Value, value: &Value) -> Result<(), String> {
    let matches = match kind {
        Value::String(name) => type_matches(name, value),
        Value::Array(names) => names
            .iter()
            .filter_map(Value::as_str)
            .any(|name| type_matches(name, value)),
        _ => false,
    };
    matches.then_some(()).ok_or_else(|| INVALID.to_string())
}

fn type_matches(kind: &str, value: &Value) -> bool {
    match kind {
        "object" => value.is_object(),
        "array" => value.is_array(),
        "string" => value.is_string(),
        "integer" => value.as_i64().is_some() || value.as_u64().is_some(),
        "number" => value.is_number(),
        "boolean" => value.is_boolean(),
        "null" => value.is_null(),
        _ => false,
    }
}

fn root_accepts_object(schema: &Value) -> bool {
    let Some(kind) = schema.get("type") else {
        return true;
    };
    match kind {
        Value::String(name) => name == "object",
        Value::Array(names) => names.iter().any(|name| name.as_str() == Some("object")),
        _ => false,
    }
}
