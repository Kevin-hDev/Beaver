use serde_json::Value;
use std::collections::HashSet;

const INVALID: &str = "schéma MCP invalide";
const MAX_DEPTH: usize = 16;
const MAX_NODES: usize = 256;

const ALLOWED: &[&str] = &[
    "$id",
    "$schema",
    "title",
    "description",
    "default",
    "examples",
    "deprecated",
    "readOnly",
    "writeOnly",
    "format",
    "type",
    "properties",
    "required",
    "additionalProperties",
    "minProperties",
    "maxProperties",
    "items",
    "minItems",
    "maxItems",
    "uniqueItems",
    "minLength",
    "maxLength",
    "pattern",
    "minimum",
    "maximum",
    "exclusiveMinimum",
    "exclusiveMaximum",
    "multipleOf",
    "enum",
    "const",
    "allOf",
    "anyOf",
    "oneOf",
    "not",
];

pub fn validate(schema: &Value) -> Result<(), String> {
    super::schema_limits::validate(schema)?;
    let mut nodes = 0;
    visit(schema, 0, &mut nodes)
}

fn visit(schema: &Value, depth: usize, nodes: &mut usize) -> Result<(), String> {
    if depth > MAX_DEPTH {
        return Err(INVALID.to_string());
    }
    *nodes += 1;
    if *nodes > MAX_NODES {
        return Err(INVALID.to_string());
    }
    let object = schema.as_object().ok_or_else(|| INVALID.to_string())?;
    if object.keys().any(|key| !ALLOWED.contains(&key.as_str())) {
        return Err(INVALID.to_string());
    }
    validate_type(object.get("type"))?;
    validate_required(object.get("required"))?;
    validate_scalar_keywords(object)?;
    visit_children(object, depth, nodes)
}

fn validate_type(kind: Option<&Value>) -> Result<(), String> {
    let Some(kind) = kind else {
        return Ok(());
    };
    let valid = |name: &str| {
        matches!(
            name,
            "object" | "array" | "string" | "integer" | "number" | "boolean" | "null"
        )
    };
    match kind {
        Value::String(name) if valid(name) => Ok(()),
        Value::Array(names) if !names.is_empty() => {
            let mut seen = HashSet::with_capacity(names.len());
            if names.iter().all(|name| {
                name.as_str()
                    .is_some_and(|name| valid(name) && seen.insert(name))
            }) {
                Ok(())
            } else {
                Err(INVALID.to_string())
            }
        }
        _ => Err(INVALID.to_string()),
    }
}

fn validate_required(required: Option<&Value>) -> Result<(), String> {
    let Some(required) = required else {
        return Ok(());
    };
    let items = required.as_array().ok_or_else(|| INVALID.to_string())?;
    let mut seen = HashSet::with_capacity(items.len());
    if items.len() > MAX_NODES
        || !items
            .iter()
            .all(|item| item.as_str().is_some_and(|name| seen.insert(name)))
    {
        return Err(INVALID.to_string());
    }
    Ok(())
}

fn validate_scalar_keywords(object: &serde_json::Map<String, Value>) -> Result<(), String> {
    for key in [
        "minProperties",
        "maxProperties",
        "minItems",
        "maxItems",
        "minLength",
        "maxLength",
    ] {
        if object
            .get(key)
            .is_some_and(|value| value.as_u64().is_none())
        {
            return Err(INVALID.to_string());
        }
    }
    for key in [
        "minimum",
        "maximum",
        "exclusiveMinimum",
        "exclusiveMaximum",
        "multipleOf",
    ] {
        if object.get(key).is_some_and(|value| !value.is_number()) {
            return Err(INVALID.to_string());
        }
    }
    if object
        .get("uniqueItems")
        .is_some_and(|value| !value.is_boolean())
    {
        return Err(INVALID.to_string());
    }
    if let Some(pattern) = object.get("pattern") {
        let pattern = pattern.as_str().ok_or_else(|| INVALID.to_string())?;
        if pattern.len() > 256 || regex::Regex::new(pattern).is_err() {
            return Err(INVALID.to_string());
        }
    }
    if object
        .get("enum")
        .is_some_and(|value| value.as_array().is_none_or(Vec::is_empty))
    {
        return Err(INVALID.to_string());
    }
    Ok(())
}

fn visit_children(
    object: &serde_json::Map<String, Value>,
    depth: usize,
    nodes: &mut usize,
) -> Result<(), String> {
    if let Some(properties) = object.get("properties") {
        for child in properties
            .as_object()
            .ok_or_else(|| INVALID.to_string())?
            .values()
        {
            visit(child, depth + 1, nodes)?;
        }
    }
    if let Some(items) = object.get("items") {
        visit(items, depth + 1, nodes)?;
    }
    if let Some(additional) = object.get("additionalProperties") {
        if !additional.is_boolean() {
            visit(additional, depth + 1, nodes)?;
        }
    }
    for keyword in ["allOf", "anyOf", "oneOf"] {
        if let Some(branches) = object.get(keyword) {
            let branches = branches
                .as_array()
                .filter(|items| !items.is_empty())
                .ok_or_else(|| INVALID.to_string())?;
            for branch in branches {
                visit(branch, depth + 1, nodes)?;
            }
        }
    }
    if let Some(not_schema) = object.get("not") {
        visit(not_schema, depth + 1, nodes)?;
    }
    Ok(())
}
