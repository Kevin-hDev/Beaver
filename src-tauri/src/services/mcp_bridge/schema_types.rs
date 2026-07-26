use serde_json::{Map, Value};
use std::collections::HashSet;

const INVALID: &str = "arguments MCP invalides";

pub fn validate(schema: &Map<String, Value>, value: &Value) -> Result<(), String> {
    match value {
        Value::Object(object) => validate_object(schema, object),
        Value::Array(items) => validate_array(schema, items),
        Value::String(text) => validate_string(schema, text),
        Value::Number(number) => validate_number(schema, number),
        _ => Ok(()),
    }
}

fn validate_object(schema: &Map<String, Value>, object: &Map<String, Value>) -> Result<(), String> {
    validate_usize_bound(schema, "minProperties", object.len(), |actual, bound| {
        actual >= bound
    })?;
    validate_usize_bound(schema, "maxProperties", object.len(), |actual, bound| {
        actual <= bound
    })?;
    if let Some(required) = schema.get("required").and_then(Value::as_array) {
        if required
            .iter()
            .filter_map(Value::as_str)
            .any(|name| !object.contains_key(name))
        {
            return Err(INVALID.to_string());
        }
    }
    let properties = schema.get("properties").and_then(Value::as_object);
    for (name, value) in object {
        if let Some(child) = properties.and_then(|known| known.get(name)) {
            super::schema::validate_value(child, value)?;
            continue;
        }
        match schema.get("additionalProperties") {
            Some(Value::Bool(false)) => return Err(INVALID.to_string()),
            Some(extra) if !extra.is_boolean() => super::schema::validate_value(extra, value)?,
            _ => {}
        }
    }
    Ok(())
}

fn validate_array(schema: &Map<String, Value>, items: &[Value]) -> Result<(), String> {
    validate_usize_bound(schema, "minItems", items.len(), |actual, bound| {
        actual >= bound
    })?;
    validate_usize_bound(schema, "maxItems", items.len(), |actual, bound| {
        actual <= bound
    })?;
    if schema.get("uniqueItems").and_then(Value::as_bool) == Some(true) {
        let mut unique = HashSet::with_capacity(items.len());
        for item in items {
            let encoded = serde_json::to_string(item).map_err(|_| INVALID.to_string())?;
            if !unique.insert(encoded) {
                return Err(INVALID.to_string());
            }
        }
    }
    if let Some(item_schema) = schema.get("items") {
        for item in items {
            super::schema::validate_value(item_schema, item)?;
        }
    }
    Ok(())
}

fn validate_string(schema: &Map<String, Value>, text: &str) -> Result<(), String> {
    let length = text.chars().count();
    validate_usize_bound(schema, "minLength", length, |actual, bound| actual >= bound)?;
    validate_usize_bound(schema, "maxLength", length, |actual, bound| actual <= bound)?;
    if let Some(pattern) = schema.get("pattern").and_then(Value::as_str) {
        let regex = regex::Regex::new(pattern).map_err(|_| INVALID.to_string())?;
        if !regex.is_match(text) {
            return Err(INVALID.to_string());
        }
    }
    Ok(())
}

fn validate_number(schema: &Map<String, Value>, number: &serde_json::Number) -> Result<(), String> {
    let actual = number.as_f64().ok_or_else(|| INVALID.to_string())?;
    validate_number_bound(schema, "minimum", actual, |a, b| a >= b)?;
    validate_number_bound(schema, "maximum", actual, |a, b| a <= b)?;
    validate_number_bound(schema, "exclusiveMinimum", actual, |a, b| a > b)?;
    validate_number_bound(schema, "exclusiveMaximum", actual, |a, b| a < b)?;
    if let Some(multiple) = schema.get("multipleOf").and_then(Value::as_f64) {
        if multiple <= 0.0 || (actual / multiple - (actual / multiple).round()).abs() > 1e-9 {
            return Err(INVALID.to_string());
        }
    }
    Ok(())
}

fn validate_usize_bound(
    schema: &Map<String, Value>,
    key: &str,
    actual: usize,
    compare: impl FnOnce(usize, usize) -> bool,
) -> Result<(), String> {
    let Some(bound) = schema.get(key).and_then(Value::as_u64) else {
        return Ok(());
    };
    let bound = usize::try_from(bound).map_err(|_| INVALID.to_string())?;
    compare(actual, bound)
        .then_some(())
        .ok_or_else(|| INVALID.to_string())
}

fn validate_number_bound(
    schema: &Map<String, Value>,
    key: &str,
    actual: f64,
    compare: impl FnOnce(f64, f64) -> bool,
) -> Result<(), String> {
    let Some(bound) = schema.get(key).and_then(Value::as_f64) else {
        return Ok(());
    };
    compare(actual, bound)
        .then_some(())
        .ok_or_else(|| INVALID.to_string())
}
